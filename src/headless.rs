// Headless harness mode (Stage 0 of the DEM-GPU validation effort).
//
// Runs a scenario described by a JSON file, steps the existing GPU physics
// pipeline with no visible window and no event loop, and dumps per-step
// particle state to CSV for the Python validation suite.
//
// This file is harness-only: it drives Anthony's pipeline exactly as the
// interactive client does (WGPUProg::new -> state overwrite -> restore ->
// compute loop -> update_state readback). No physics is changed here.

use crate::scenario::Scenario;
use crate::scripts::ScriptManager;
use crate::settings::Settings;
use crate::wgpu_config::WGPUConfig;
use crate::wgpu_prog::WGPUProg;
use crate::window_init;
use std::fs::File;
use std::io::{BufWriter, Write};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

// Scenario/ScenarioParticle moved verbatim to scenario.rs (Stage 5 UI
// presets); the headless path is unchanged and must stay byte-identical.

/// Per-row aggregate metrics for Stage-2 settling runs.
///
/// Overlap/neighbor metrics mirror the shader's broad phase exactly:
/// grid origin base_x = -cell_size*w/2, base_y = +cell_size*h/2 (y inverted),
/// AABB multi-cell insertion, usable slots per cell = cell_cap - 2
/// (one count word + one tick word). `cell_overflow` counts particle-cell
/// insertions the shader would silently drop; `slot_overflow` counts
/// particles touching more than the 14 contact slots the simulation shader
/// can hold.
fn dump_aggregate(
    out: &mut BufWriter<File>,
    step: u32,
    t: f64,
    st: &crate::state::State,
    n: usize,
    mass: &[f32],
    kn: &[f32],
    g: f32,
    floor_y: f32,
    bond_type: i32,
    bkn: f32,
    bks: f32,
) {
    use std::collections::HashSet;
    let gi = &st.grid_info;
    let (w, h, cs, cap) = (gi.w as usize, gi.h as usize, gi.cell_size, gi.cell_cap as usize);
    let base_x = -cs * w as f32 * 0.5;
    let base_y = cs * h as f32 * 0.5;

    let mut ke_trans = 0.0f64;
    let mut ke_rot = 0.0f64;
    let mut pe_grav = 0.0f64;
    let mut max_speed = 0.0f32;
    let mut nan_count = 0usize;
    for i in 0..n {
        let (x, y) = (st.pos[2 * i], st.pos[2 * i + 1]);
        let (vx, vy) = (st.vel[2 * i], st.vel[2 * i + 1]);
        let rv = st.rot_vel[i];
        if !(x.is_finite() && y.is_finite() && vx.is_finite() && vy.is_finite() && rv.is_finite()) {
            nan_count += 1;
            continue;
        }
        let v2 = vx * vx + vy * vy;
        ke_trans += 0.5 * (mass[i] * v2) as f64;
        ke_rot += 0.5 * (0.5 * mass[i] * st.radii[i] * st.radii[i] * rv * rv) as f64;
        pe_grav += (mass[i] * g * (y - floor_y)) as f64;
        max_speed = max_speed.max(v2.sqrt());
    }

    // CPU re-binning with the shader's exact grid geometry.
    let mut cells: Vec<Vec<u32>> = vec![Vec::new(); w * h];
    let mut cell_overflow = 0usize;
    for i in 0..n {
        let (x, y) = (st.pos[2 * i], st.pos[2 * i + 1]);
        let r = st.radii[i];
        if !(x.is_finite() && y.is_finite()) {
            continue;
        }
        let min_cx = (((x - r - base_x) / cs) as i64).max(0) as usize;
        let max_cx = ((((x + r - base_x) / cs) as i64).min(w as i64 - 1)).max(0) as usize;
        let min_cy = (((base_y - (y + r)) / cs) as i64).max(0) as usize;
        let max_cy = ((((base_y - (y - r)) / cs) as i64).min(h as i64 - 1)).max(0) as usize;
        for cy in min_cy..=max_cy {
            for cx in min_cx..=max_cx {
                let cell = &mut cells[cy * w + cx];
                if cell.len() >= cap - 2 {
                    cell_overflow += 1;
                } else {
                    cell.push(i as u32);
                }
            }
        }
    }

    // Bond bookkeeping + bond elastic energy (Stage 3c), parallel bonds only
    // (bond_type 3). Mirrors 2D_Simulation.wgsl linear_parallel_bonds():
    // R = 0.5*min(r_a, r_b), A = 2R (t = 1), I = (2/3)R^3.
    // Per SLOT (each side holds its own springs): shear PE = bft^2/(2*bks*A),
    // bending PE = 0.5*bkn*I*theta_b^2. Per PAIR (shared geometry): normal
    // PE = 0.5*bkn*A*U_n^2 with U_n = dist - bond reference length, plus the
    // superposed linear contact (material pair stiffness, also referenced to
    // bond length, compression only). Intact/broken counts come from the
    // bonds[] array itself (index >= 0 intact, < 0 torn).
    let mut pe_bond = 0.0f64;
    let mut bonds_intact = 0u32;
    let mut bonds_broken = 0u32;
    let mut bonded_pairs: HashSet<(u32, u32)> = HashSet::new();
    if st.bonds.len() >= 3 {
        for k in (0..st.bonds.len() - 2).step_by(3) {
            if st.bonds[k] >= 0 {
                bonds_intact += 1;
            } else if st.bonds[k] <= -9 {
                bonds_broken += 1;
            }
        }
    }
    if bond_type == 3 {
        for p in 0..n {
            for s in 0..14usize {
                let base = (p * 14 + s) * 6;
                if base + 5 >= st.contacts.len() {
                    break;
                }
                let b = bytemuck::cast::<f32, i32>(st.contacts[base + 1]);
                if b == -1 {
                    continue;
                }
                let bonded = bytemuck::cast::<f32, i32>(st.contacts[base + 5]);
                if bonded < 0 {
                    continue;
                }
                let a = bytemuck::cast::<f32, i32>(st.contacts[base]) as usize;
                let b = b as usize;
                let r_bond = 0.5 * st.radii[a].min(st.radii[b]);
                let area = 2.0 * r_bond;
                let inertia = 2.0 / 3.0 * r_bond * r_bond * r_bond;
                let bft = st.contacts[base + 2 + 1]; // bond_tangent_force
                let thb = st.contacts[base + 4]; // theta_b
                if bft.is_finite() && thb.is_finite() {
                    pe_bond += 0.5 * (bft * bft / (bks * area)) as f64;
                    pe_bond += 0.5 * (bkn * inertia * thb * thb) as f64;
                }
                let key = if a < b { (a as u32, b as u32) } else { (b as u32, a as u32) };
                if bonded_pairs.insert(key) {
                    let bidx = bonded as usize * 3;
                    if bidx + 2 < st.bonds.len() {
                        let bond_len = f32::from_bits(st.bonds[bidx + 2] as u32);
                        let dx = st.pos[2 * b] - st.pos[2 * a];
                        let dy = st.pos[2 * b + 1] - st.pos[2 * a + 1];
                        let un = (dx * dx + dy * dy).sqrt() - bond_len;
                        if un.is_finite() {
                            pe_bond += 0.5 * (bkn * area * un * un) as f64;
                            if un < 0.0 {
                                let kp = 1.0 / (1.0 / kn[a] + 1.0 / kn[b]);
                                pe_bond += 0.5 * (kp * un * un) as f64;
                            }
                        }
                    }
                }
            }
        }
    }

    let mut seen: HashSet<(u32, u32)> = HashSet::new();
    let mut pe_elast = 0.0f64;
    let mut max_overlap = 0.0f32;
    let mut neighbors = vec![0u32; n];
    for cell in &cells {
        for (ai, &a) in cell.iter().enumerate() {
            for &b in &cell[ai + 1..] {
                let key = if a < b { (a, b) } else { (b, a) };
                if !seen.insert(key) {
                    continue;
                }
                let (i, j) = (key.0 as usize, key.1 as usize);
                let dx = st.pos[2 * j] - st.pos[2 * i];
                let dy = st.pos[2 * j + 1] - st.pos[2 * i + 1];
                let dist = (dx * dx + dy * dy).sqrt();
                let u = st.radii[i] + st.radii[j] - dist;
                if u > 0.0 {
                    max_overlap = max_overlap.max(u);
                    neighbors[i] += 1;
                    neighbors[j] += 1;
                    // Bonded pairs' elastic energy is referenced to the bond
                    // length and accounted in pe_bond above; skip here.
                    if bonded_pairs.contains(&(key.0, key.1)) {
                        continue;
                    }
                    let kp = 1.0 / (1.0 / kn[i] + 1.0 / kn[j]);
                    pe_elast += 0.5 * (kp * u * u) as f64;
                }
            }
        }
    }
    let touching_pairs: u32 = neighbors.iter().sum::<u32>() / 2;
    let max_neighbors = neighbors.iter().copied().max().unwrap_or(0);
    let slot_overflow = neighbors.iter().filter(|&&c| c > 14).count();

    writeln!(
        out,
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        step,
        t,
        ke_trans,
        ke_rot,
        pe_grav,
        pe_elast,
        ke_trans + ke_rot + pe_grav + pe_elast + pe_bond,
        max_overlap,
        touching_pairs,
        max_neighbors,
        slot_overflow,
        cell_overflow,
        max_speed,
        nan_count,
        pe_bond,
        bonds_intact,
        bonds_broken
    )
    .unwrap();
}

pub fn run(scenario_path: &str, out_path: &str) {
    let txt = std::fs::read_to_string(scenario_path)
        .unwrap_or_else(|e| panic!("cannot read scenario {}: {}", scenario_path, e));
    let sc: Scenario = serde_json::from_str(&txt)
        .unwrap_or_else(|e| panic!("cannot parse scenario {}: {}", scenario_path, e));
    let n = sc.particles.len();
    assert!(n > 0, "scenario has no particles");
    assert!(sc.dump_every > 0, "dump_every must be >= 1");

    println!("[headless] scenario '{}': {} particles, {} steps, dt = {}", sc.name, n, sc.steps, sc.timestep);

    // Window is required by WGPUConfig (surface-based adapter selection) but
    // stays invisible; there is no event loop.
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_visible(false).build(&event_loop).unwrap();
    window.set_title("Physics Simulator (headless)");
    let canvas = window_init::Canvas::new(window);
    let mut config = async_std::task::block_on(WGPUConfig::new(&canvas));
    let mut settings = Settings::new(&canvas);
    settings.f64_support = config.f64_support;

    // Apply scenario BEFORE WGPUProg::new so buffer sizes, grid setup, and the
    // collision-settings uniform are all built from these values.
    sc.apply_to_settings(&mut settings);

    let script_manager = ScriptManager::new();
    let mut prog = WGPUProg::new(&mut config, &mut settings, (canvas.size.width as u32, canvas.size.height as u32), &script_manager);

    // Overwrite the generated scene with the scenario's exact initial state
    // and upload it (includes the grid sizing + bond_init paths).
    sc.install_state(&mut prog, &mut config, &mut settings);

    // Per-particle mass and material normal stiffness for aggregate metrics
    // (material row = [r, g, b, density, k_n, k_s], see PFC_MODEL.md D3).
    let mat = &settings.materials;
    let (mass, kn): (Vec<f32>, Vec<f32>) = (0..n)
        .map(|i| {
            let m = sc.particles[i].material as usize;
            let density = mat[m * 6 + 3];
            let r = sc.particles[i].r;
            (density * std::f32::consts::PI * r * r, mat[m * 6 + 4])
        })
        .unzip();
    let g_eff = if sc.gravity { 9.81 * sc.gravity_acceleration } else { 0.0 };
    let floor_y = -sc.vert_bound / 2.0;

    let mut out = BufWriter::new(File::create(out_path).unwrap_or_else(|e| panic!("cannot create {}: {}", out_path, e)));
    if sc.aggregate {
        writeln!(out, "step,t,ke_trans,ke_rot,pe_grav,pe_elast,e_total,max_overlap,touching_pairs,max_neighbors,slot_overflow,cell_overflow,max_speed,nan_count,pe_bond,bonds_intact,bonds_broken").unwrap();
    } else {
        write!(out, "step,t").unwrap();
        for i in 0..n {
            write!(out, ",p{i}_x,p{i}_y,p{i}_vx,p{i}_vy,p{i}_rot,p{i}_rot_vel,p{i}_fn,p{i}_fs,p{i}_mom").unwrap();
        }
        writeln!(out).unwrap();
    }

    let dump = |out: &mut BufWriter<File>, step: u32, st: &crate::state::State| {
        write!(out, "{},{}", step, step as f64 * sc.timestep as f64).unwrap();
        for i in 0..n {
            // data[0..2] are the per-particle contact diagnostics summed in the
            // simulation shader: normal force, tangent force, moment.
            write!(
                out,
                ",{},{},{},{},{},{},{},{},{}",
                st.pos[2 * i],
                st.pos[2 * i + 1],
                st.vel[2 * i],
                st.vel[2 * i + 1],
                st.rot[i],
                st.rot_vel[i],
                st.data[7 * i],
                st.data[7 * i + 1],
                st.data[7 * i + 2]
            )
            .unwrap();
        }
        writeln!(out).unwrap();
    };

    // Row for the initial state, then one row per dump interval.
    let do_dump = |out: &mut BufWriter<File>, step: u32, st: &crate::state::State| {
        if sc.aggregate {
            dump_aggregate(out, step, step as f64 * sc.timestep as f64, st, n, &mass, &kn, g_eff, floor_y, sc.bond_type, sc.bond_normal_stiffness, sc.bond_shear_stiffness);
        } else {
            dump(out, step, st);
        }
    };
    // Binary snapshot stream for video rendering (Stage 3c). One frame =
    // 4 f32 LE per particle: x, y, speed, intact-bond count (from the contact
    // slots, bonded >= 0). Sidecar JSON records the layout for the renderer.
    let mut snap: Option<BufWriter<File>> = if sc.snapshot_every > 0 {
        let snap_path = format!("{}.snap", out_path);
        let sidecar = format!(
            "{{\"n\": {}, \"fields\": [\"x\", \"y\", \"speed\", \"intact_bonds\"], \"every\": {}, \"dt\": {}}}\n",
            n, sc.snapshot_every, sc.timestep
        );
        std::fs::write(format!("{}.snap.json", out_path), sidecar).unwrap();
        Some(BufWriter::new(File::create(&snap_path).unwrap_or_else(|e| panic!("cannot create {}: {}", snap_path, e))))
    } else {
        None
    };
    let write_snap = |snap: &mut BufWriter<File>, st: &crate::state::State| {
        let mut frame: Vec<f32> = Vec::with_capacity(4 * n);
        for i in 0..n {
            let (vx, vy) = (st.vel[2 * i], st.vel[2 * i + 1]);
            let mut intact = 0.0f32;
            for s in 0..14usize {
                let base = (i * 14 + s) * 6;
                if base + 5 >= st.contacts.len() {
                    break;
                }
                if bytemuck::cast::<f32, i32>(st.contacts[base + 1]) != -1
                    && bytemuck::cast::<f32, i32>(st.contacts[base + 5]) >= 0
                {
                    intact += 1.0;
                }
            }
            frame.push(st.pos[2 * i]);
            frame.push(st.pos[2 * i + 1]);
            frame.push((vx * vx + vy * vy).sqrt());
            frame.push(intact);
        }
        snap.write_all(bytemuck::cast_slice(&frame)).unwrap();
    };

    do_dump(&mut out, 0, &prog.shader_prog.state);
    if let Some(s) = snap.as_mut() {
        write_snap(s, &prog.shader_prog.state);
    }
    // Step batching: compute() already records gen_per_frame full physics
    // steps into ONE command submission; the per-submission host/driver
    // overhead (not GPU compute) dominates wall time at large N. Batch up to
    // MAX_BATCH steps per submission, breaking exactly at every dump,
    // snapshot, and damping-switch boundary so observable behavior is
    // identical to the old one-step-per-submission loop.
    const MAX_BATCH: u32 = 256;
    let mut step = 0u32; // steps completed
    while step < sc.steps {
        if sc.local_damping && sc.damping_on_after_step > 0 && step + 1 == sc.damping_on_after_step {
            settings.physics.local_damping = true;
            let cs = settings.collision_settings();
            prog.shader_prog
                .buffers
                .collision_settings
                .updateUniform(&config.device, bytemuck::cast_slice(&cs));
            println!(
                "[headless] local damping enabled at step {} (t = {:.3})",
                step + 1,
                (step + 1) as f64 * sc.timestep as f64
            );
        }
        let to_next = |every: u32| if every > 0 { every - (step % every) } else { u32::MAX };
        let mut batch = (sc.steps - step).min(MAX_BATCH).min(to_next(sc.dump_every));
        if sc.snapshot_every > 0 {
            batch = batch.min(to_next(sc.snapshot_every));
        }
        if sc.local_damping && sc.damping_on_after_step > step + 1 {
            batch = batch.min(sc.damping_on_after_step - (step + 1));
        }
        settings.simulation.gen_per_frame = batch as i32;
        prog.shader_prog.compute(&mut config, &settings);
        step += batch;
        let dump_due = step % sc.dump_every == 0;
        let snap_due = sc.snapshot_every > 0 && step % sc.snapshot_every == 0;
        if dump_due || snap_due {
            let sp = &mut prog.shader_prog;
            sp.state.update_state(&mut config, &settings, &mut sp.buffers);
            if dump_due {
                do_dump(&mut out, step, &sp.state);
            }
            if snap_due {
                if let Some(s) = snap.as_mut() {
                    write_snap(s, &sp.state);
                }
            }
        }
    }
    out.flush().unwrap();
    if let Some(mut s) = snap.take() {
        s.flush().unwrap();
        println!("[headless] wrote {}.snap (+.snap.json)", out_path);
    }
    println!("[headless] wrote {}", out_path);

    // Debug introspection of GPU-side contact state (harness-only).
    if std::env::var("HEADLESS_DEBUG").is_ok() {
        let sp = &mut prog.shader_prog;
        sp.state.update_state(&mut config, &settings, &mut sp.buffers);
        let st = &sp.state;
        println!("[debug] grid_info: cell_size={} cap={} w={} h={}", st.grid_info.cell_size, st.grid_info.cell_cap, st.grid_info.w, st.grid_info.h);
        println!("[debug] p_count={} contacts.len()={} data.len()={}", st.p_count, st.contacts.len(), st.data.len());
        for p in 0..n.min(4) {
            for s in 0..14 {
                let base = (p * 14 + s) * 6;
                if base + 5 >= st.contacts.len() {
                    break;
                }
                let a = bytemuck::cast::<f32, i32>(st.contacts[base]);
                let b = bytemuck::cast::<f32, i32>(st.contacts[base + 1]);
                if b != -1 {
                    println!(
                        "[debug] contact p{} slot{}: a={} b={} ft={} bft={} thb={} bonded={}",
                        p, s, a, b, st.contacts[base + 2], st.contacts[base + 3], st.contacts[base + 4],
                        bytemuck::cast::<f32, i32>(st.contacts[base + 5])
                    );
                }
            }
            let d = 7 * p;
            println!(
                "[debug] data p{}: fn={} fs={} mom={} d3={} intvx={} intvy={} introt={}",
                p, st.data[d], st.data[d + 1], st.data[d + 2], st.data[d + 3], st.data[d + 4], st.data[d + 5], st.data[d + 6]
            );
        }
        // state.grid readback is commented out upstream in update_state; read
        // the GPU grid buffer directly here.
        let mut gpu_grid: Vec<i32> = Vec::new();
        crate::state::State::update_i32(&mut config.device, &mut config.queue, &mut gpu_grid, &mut sp.buffers.contact_buffers.buffers[4]);
        let st = &sp.state;
        let gi = &st.grid_info;
        let mut nonzero = 0;
        for c in 0..(gi.w * gi.h) as usize {
            let base = c * gi.cell_cap as usize;
            if base < gpu_grid.len() && gpu_grid[base] != 0 {
                nonzero += 1;
                if nonzero <= 8 {
                    let cx = c % gi.w as usize;
                    let cy = c / gi.w as usize;
                    println!(
                        "[debug] grid cell ({},{}) count={} tick={} slots={:?}",
                        cx, cy, gpu_grid[base], gpu_grid[base + 1],
                        &gpu_grid[base + 2..(base + gi.cell_cap as usize).min(gpu_grid.len())]
                    );
                }
            }
        }
        println!("[debug] cells with nonzero count: {}", nonzero);
        let mut ticked = 0;
        let mut anynz = 0;
        for c in 0..(gi.w * gi.h) as usize {
            let base = c * gi.cell_cap as usize;
            if base + 1 < gpu_grid.len() && gpu_grid[base + 1] != 0 {
                ticked += 1;
                if ticked <= 6 {
                    println!("[debug] ticked cell {} tick={} count={}", c, gpu_grid[base + 1], gpu_grid[base]);
                }
            }
        }
        for (i, v) in gpu_grid.iter().enumerate() {
            if *v != 0 {
                anynz += 1;
                if anynz <= 10 {
                    println!("[debug] grid[{}] = {}", i, v);
                }
            }
        }
        println!("[debug] ticked cells: {}, nonzero grid words: {} of {}", ticked, anynz, gpu_grid.len());
        let mut cc: Vec<i32> = vec![0; 4];
        crate::state::State::update_i32(&mut config.device, &mut config.queue, &mut cc, &mut sp.buffers.contact_buffers.buffers[6]);
        println!("[debug] coll_cont = {:?}", cc);
    }
}
