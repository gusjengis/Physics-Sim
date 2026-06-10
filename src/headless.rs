// Headless harness mode (Stage 0 of the DEM-GPU validation effort).
//
// Runs a scenario described by a JSON file, steps the existing GPU physics
// pipeline with no visible window and no event loop, and dumps per-step
// particle state to CSV for the Python validation suite.
//
// This file is harness-only: it drives Anthony's pipeline exactly as the
// interactive client does (WGPUProg::new -> state overwrite -> restore ->
// compute loop -> update_state readback). No physics is changed here.

use crate::scripts::ScriptManager;
use crate::settings::{Settings, Structure};
use crate::wgpu_config::WGPUConfig;
use crate::wgpu_prog::WGPUProg;
use crate::window_init;
use serde::Deserialize;
use std::fs::File;
use std::io::{BufWriter, Write};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

fn d_one_u() -> u32 {
    1
}
fn d_one_f() -> f32 {
    1.0
}
fn d_true() -> bool {
    true
}
fn d_bound() -> f32 {
    10.0
}
fn d_friction() -> f32 {
    0.5
}
fn d_damping() -> f32 {
    0.2
}
fn d_alpha() -> f32 {
    0.1
}

#[derive(Deserialize)]
pub struct ScenarioParticle {
    pub x: f32,
    pub y: f32,
    #[serde(default)]
    pub vx: f32,
    #[serde(default)]
    pub vy: f32,
    pub r: f32,
    #[serde(default)]
    pub rot: f32,
    #[serde(default)]
    pub rot_vel: f32,
    #[serde(default)]
    pub material: i32,
    #[serde(default)]
    pub fixed: bool,
}

#[derive(Deserialize)]
pub struct Scenario {
    pub name: String,
    pub steps: u32,
    #[serde(default = "d_one_u")]
    pub dump_every: u32,
    pub timestep: f32,
    #[serde(default)]
    pub gravity: bool,
    /// Multiplier on 9.81 m/s^2 (matches the shader's `9.81 * gravity_acc`).
    #[serde(default = "d_one_f")]
    pub gravity_acceleration: f32,
    #[serde(default)]
    pub planet_mode: bool,
    /// NOTE: walls are hardcoded ON in 2D_LOM.wgsl (the `settings.walls`
    /// check is commented out upstream). Choose bounds large enough that the
    /// scenario never touches them unless wall contact is intended.
    #[serde(default = "d_bound")]
    pub hor_bound: f32,
    #[serde(default = "d_bound")]
    pub vert_bound: f32,
    #[serde(default = "d_true")]
    pub collisions: bool,
    #[serde(default = "d_friction")]
    pub friction_coefficient: f32,
    #[serde(default = "d_damping")]
    pub contact_damping: f32,
    #[serde(default)]
    pub local_damping: bool,
    #[serde(default = "d_alpha")]
    pub local_damping_alpha: f32,
    /// Raw materials vector (rows of `material_size` floats, first entry of a
    /// row is density; see settings.materials). Optional override.
    #[serde(default)]
    pub materials: Option<Vec<f32>>,
    /// Aggregate dump mode (Stage 2, large N): instead of per-particle columns,
    /// each row holds whole-system energies and contact-bookkeeping health
    /// metrics computed CPU-side from the readback. No physics touched.
    #[serde(default)]
    pub aggregate: bool,
    pub particles: Vec<ScenarioParticle>,
}

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
                    let kp = 1.0 / (1.0 / kn[i] + 1.0 / kn[j]);
                    pe_elast += 0.5 * (kp * u * u) as f64;
                    max_overlap = max_overlap.max(u);
                    neighbors[i] += 1;
                    neighbors[j] += 1;
                }
            }
        }
    }
    let touching_pairs: u32 = neighbors.iter().sum::<u32>() / 2;
    let max_neighbors = neighbors.iter().copied().max().unwrap_or(0);
    let slot_overflow = neighbors.iter().filter(|&&c| c > 14).count();

    writeln!(
        out,
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        step,
        t,
        ke_trans,
        ke_rot,
        pe_grav,
        pe_elast,
        ke_trans + ke_rot + pe_grav + pe_elast,
        max_overlap,
        touching_pairs,
        max_neighbors,
        slot_overflow,
        cell_overflow,
        max_speed,
        nan_count
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
    settings.set_particles(n);
    settings.setup.structure = Structure::Grid;
    settings.setup.max_radius = sc.particles.iter().map(|p| p.r).fold(0.0_f32, f32::max);
    settings.setup.min_radius = sc.particles.iter().map(|p| p.r).fold(f32::INFINITY, f32::min);
    // H3 fix (AUTOPSY): variable_rad=false collapsed min_rad to max_radius in
    // grid_capacity(), undersizing cell_cap (11 for the polydisperse T7 instead
    // of 51) -> silent grid-insertion drops -> missed contacts at large N.
    settings.setup.variable_rad = settings.setup.min_radius < settings.setup.max_radius;
    settings.setup.max_h_velocity = 0.0;
    settings.setup.min_h_velocity = 0.0;
    settings.setup.max_v_velocity = 0.0;
    settings.setup.min_v_velocity = 0.0;
    settings.simulation.timestep = sc.timestep;
    settings.simulation.gen_per_frame = 1; // exactly one physics step per compute() call
    settings.simulation.hor_bound = sc.hor_bound;
    settings.simulation.vert_bound = sc.vert_bound;
    settings.simulation.round_walls = false;
    settings.physics.gravity = sc.gravity;
    settings.physics.gravity_acceleration = sc.gravity_acceleration;
    settings.physics.planet_mode = sc.planet_mode;
    settings.physics.mouse_gravity = false;
    settings.physics.collisions = sc.collisions;
    settings.physics.collision_interval = 1;
    settings.physics.friction_coefficient = sc.friction_coefficient;
    settings.physics.contact_damping = sc.contact_damping;
    settings.physics.local_damping = sc.local_damping;
    settings.physics.local_damping_alpha = sc.local_damping_alpha;
    settings.physics.bonds = 0;
    if let Some(m) = &sc.materials {
        settings.materials = m.clone();
    }

    let script_manager = ScriptManager::new();
    let mut prog = WGPUProg::new(&mut config, &mut settings, (canvas.size.width as u32, canvas.size.height as u32), &script_manager);

    // Overwrite the generated scene with the scenario's exact initial state.
    {
        let st = &mut prog.shader_prog.state;
        for v in st.vel.iter_mut() {
            *v = 0.0;
        }
        for v in st.acc.iter_mut() {
            *v = 0.0;
        }
        for v in st.rot.iter_mut() {
            *v = 0.0;
        }
        for v in st.rot_vel.iter_mut() {
            *v = 0.0;
        }
        for v in st.rot_acc.iter_mut() {
            *v = 0.0;
        }
        for v in st.forces.iter_mut() {
            *v = 0.0;
        }
        for v in st.del_pos.iter_mut() {
            *v = 0.0;
        }
        for v in st.del_rot.iter_mut() {
            *v = 0.0;
        }
        for v in st.fixity.iter_mut() {
            *v = 0;
        }
        for v in st.contacts.iter_mut() {
            *v = bytemuck::cast::<i32, f32>(-1);
        }
        // No bonds: the upstream "empty" sentinel is a single -1 (4 bytes), but
        // the shaders bind bonds as array<Bond> (12-byte stride), so pad to one
        // full null Bond to satisfy wgpu's minimum binding size.
        st.bonds = vec![-1; 3];
        // State::new leaves grid as a 1-word placeholder (state.rs `grid: vec![0; 1]`);
        // only the file-load path sizes it. restore() would otherwise shrink the
        // GPU grid buffer to 4 bytes and silently kill the broad phase.
        st.grid = vec![0; (st.grid_info.w * st.grid_info.h * st.grid_info.cell_cap) as usize];
        for (i, p) in sc.particles.iter().enumerate() {
            st.pos[2 * i] = p.x;
            st.pos[2 * i + 1] = p.y;
            st.vel[2 * i] = p.vx;
            st.vel[2 * i + 1] = p.vy;
            st.radii[i] = p.r;
            st.rot[i] = p.rot;
            st.rot_vel[i] = p.rot_vel;
            st.material_pointers[i] = p.material;
            if p.fixed {
                st.fixity[6 * i] = 1; // x_vel
                st.fixity[6 * i + 1] = 1; // y_vel
                st.fixity[6 * i + 2] = 1; // rot_vel
            }
        }
    }
    prog.shader_prog.restore(&mut config, &mut settings);

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
        writeln!(out, "step,t,ke_trans,ke_rot,pe_grav,pe_elast,e_total,max_overlap,touching_pairs,max_neighbors,slot_overflow,cell_overflow,max_speed,nan_count").unwrap();
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
            dump_aggregate(out, step, step as f64 * sc.timestep as f64, st, n, &mass, &kn, g_eff, floor_y);
        } else {
            dump(out, step, st);
        }
    };
    do_dump(&mut out, 0, &prog.shader_prog.state);
    for step in 1..=sc.steps {
        prog.shader_prog.compute(&mut config, &settings);
        if step % sc.dump_every == 0 {
            let sp = &mut prog.shader_prog;
            sp.state.update_state(&mut config, &settings, &mut sp.buffers);
            do_dump(&mut out, step, &sp.state);
        }
    }
    out.flush().unwrap();
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
