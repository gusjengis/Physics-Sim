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
    pub particles: Vec<ScenarioParticle>,
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
    settings.setup.variable_rad = false;
    settings.setup.max_radius = sc.particles.iter().map(|p| p.r).fold(0.0_f32, f32::max);
    settings.setup.min_radius = sc.particles.iter().map(|p| p.r).fold(f32::INFINITY, f32::min);
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

    let mut out = BufWriter::new(File::create(out_path).unwrap_or_else(|e| panic!("cannot create {}: {}", out_path, e)));
    write!(out, "step,t").unwrap();
    for i in 0..n {
        write!(out, ",p{i}_x,p{i}_y,p{i}_vx,p{i}_vy,p{i}_rot,p{i}_rot_vel").unwrap();
    }
    writeln!(out).unwrap();

    let dump = |out: &mut BufWriter<File>, step: u32, st: &crate::state::State| {
        write!(out, "{},{}", step, step as f64 * sc.timestep as f64).unwrap();
        for i in 0..n {
            write!(
                out,
                ",{},{},{},{},{},{}",
                st.pos[2 * i],
                st.pos[2 * i + 1],
                st.vel[2 * i],
                st.vel[2 * i + 1],
                st.rot[i],
                st.rot_vel[i]
            )
            .unwrap();
        }
        writeln!(out).unwrap();
    };

    // Row for the initial state, then one row per dump interval.
    dump(&mut out, 0, &prog.shader_prog.state);
    for step in 1..=sc.steps {
        prog.shader_prog.compute(&mut config, &settings);
        if step % sc.dump_every == 0 {
            let sp = &mut prog.shader_prog;
            sp.state.update_state(&mut config, &settings, &mut sp.buffers);
            dump(&mut out, step, &sp.state);
        }
    }
    out.flush().unwrap();
    println!("[headless] wrote {}", out_path);
}
