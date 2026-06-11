// Shared scenario format + loaders (Stage 5 UI presets).
//
// Factored verbatim out of headless.rs so the windowed client can run the
// validation scenarios as preset experiments. Parsing is filesystem-free
// (from &str) so the same code runs on wasm32, where presets are embedded
// strings. The headless harness path MUST remain byte-identical: the bodies
// of apply_to_settings/install_state are the exact code previously inlined
// in headless::run, in the same order.

use crate::settings::{Settings, Structure};
use crate::wgpu_config::WGPUConfig;
use crate::wgpu_prog::WGPUProg;
use serde::Deserialize;

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
    /// Stage-3b contact dashpot (AUTOPSY 2026-06-11): viscous normal damping
    /// ratio at unbonded contacts, c_n = 2*beta*sqrt(m_eff*k_n_pair).
    /// 0 (default) = OFF = byte-identical legacy behavior.
    #[serde(default)]
    pub dashpot_beta: f32,
    /// If > 0, start the run with local damping OFF and switch it on at this
    /// step (host re-uploads the collision-settings uniform; no shader change).
    /// For bonded free-flight scenes: local damping rectifies internal bond
    /// vibration into net lift and fakes air resistance (LOGBOOK 2026-06-10),
    /// so damp only after first contact. 0 = damping state fixed for the run.
    /// NOTE: honored by the headless loop only; UI presets with this set
    /// would run with damping permanently OFF, so don't use it in presets.
    #[serde(default)]
    pub damping_on_after_step: u32,
    /// Raw materials vector (rows of `material_size` floats, first entry of a
    /// row is density; see settings.materials). Optional override.
    #[serde(default)]
    pub materials: Option<Vec<f32>>,
    /// Aggregate dump mode (Stage 2, large N): instead of per-particle columns,
    /// each row holds whole-system energies and contact-bookkeeping health
    /// metrics computed CPU-side from the readback. No physics touched.
    #[serde(default)]
    pub aggregate: bool,
    /// Bond configuration (Stage 3c). bond_type: 0 none, 1 normal-only,
    /// 2 linear contact, 3 parallel (PFC-style). bond_init = bond every pair
    /// within 1.05*(r_i+r_j) at t=0 via the upstream State::regen_bonds path.
    /// NOTE: aggregate bond elastic energy is only computed for bond_type 3.
    #[serde(default)]
    pub bond_type: i32,
    #[serde(default)]
    pub bond_init: bool,
    #[serde(default)]
    pub bond_tearing: bool,
    #[serde(default = "d_one_f")]
    pub bond_normal_stiffness: f32,
    #[serde(default = "d_one_f")]
    pub bond_shear_stiffness: f32,
    #[serde(default)]
    pub bond_normal_strength: f32,
    #[serde(default)]
    pub bond_shear_strength: f32,
    #[serde(default = "d_one_f")]
    pub moment_contribution_factor: f32,
    /// Binary snapshot stream for video rendering: every k steps append one
    /// frame of 4 f32 LE per particle (x, y, speed, intact_bond_count) to
    /// `<out>.snap`, plus a JSON sidecar `<out>.snap.json` with the layout.
    /// 0 = off. Headless-only; ignored by the UI preset path.
    #[serde(default)]
    pub snapshot_every: u32,
    pub particles: Vec<ScenarioParticle>,
}

impl Scenario {
    pub fn parse(txt: &str) -> Result<Scenario, serde_json::Error> {
        serde_json::from_str(txt)
    }

    /// Apply the scenario to settings BEFORE WGPUProg/WGPUComputeProg
    /// construction so buffer sizes, grid setup, and the collision-settings
    /// uniform are all built from these values.
    pub fn apply_to_settings(&self, settings: &mut Settings) {
        let n = self.particles.len();
        settings.set_particles(n);
        settings.setup.structure = Structure::Grid;
        settings.setup.max_radius = self.particles.iter().map(|p| p.r).fold(0.0_f32, f32::max);
        settings.setup.min_radius = self.particles.iter().map(|p| p.r).fold(f32::INFINITY, f32::min);
        // H3 fix (AUTOPSY): variable_rad=false collapsed min_rad to max_radius in
        // grid_capacity(), undersizing cell_cap (11 for the polydisperse T7 instead
        // of 51) -> silent grid-insertion drops -> missed contacts at large N.
        settings.setup.variable_rad = settings.setup.min_radius < settings.setup.max_radius;
        settings.setup.max_h_velocity = 0.0;
        settings.setup.min_h_velocity = 0.0;
        settings.setup.max_v_velocity = 0.0;
        settings.setup.min_v_velocity = 0.0;
        settings.simulation.timestep = self.timestep;
        settings.simulation.gen_per_frame = 1; // exactly one physics step per compute() call
        settings.simulation.hor_bound = self.hor_bound;
        settings.simulation.vert_bound = self.vert_bound;
        settings.simulation.round_walls = false;
        settings.physics.gravity = self.gravity;
        settings.physics.gravity_acceleration = self.gravity_acceleration;
        settings.physics.planet_mode = self.planet_mode;
        settings.physics.mouse_gravity = false;
        settings.physics.collisions = self.collisions;
        settings.physics.collision_interval = 1;
        settings.physics.friction_coefficient = self.friction_coefficient;
        settings.physics.contact_damping = self.contact_damping;
        settings.physics.local_damping = self.local_damping && self.damping_on_after_step == 0;
        settings.physics.local_damping_alpha = self.local_damping_alpha;
        settings.physics.dashpot_beta = self.dashpot_beta;
        settings.physics.bonds = self.bond_type;
        settings.physics.bond_tearing = self.bond_tearing;
        settings.physics.bond_normal_stiffness = self.bond_normal_stiffness;
        settings.physics.bond_shear_stiffness = self.bond_shear_stiffness;
        settings.physics.bond_normal_strength = self.bond_normal_strength;
        settings.physics.bond_shear_strength = self.bond_shear_strength;
        settings.physics.moment_contribution_factor = self.moment_contribution_factor;
        if let Some(m) = &self.materials {
            settings.materials = m.clone();
        }
    }

    /// Overwrite a freshly built compute program's generated scene with the
    /// scenario's exact initial state, then upload it (restore).
    pub fn install_state(&self, prog: &mut WGPUProg, config: &mut WGPUConfig, settings: &mut Settings) {
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
            for (i, p) in self.particles.iter().enumerate() {
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
            // Bond init (Stage 3c): reuse the upstream regen_bonds path, which
            // bonds every pair within 1.05*(r_i+r_j) and rebuilds the contacts
            // array with the bonded slots. Must run AFTER positions/radii are set.
            if self.bond_init {
                st.regen_bonds(config, settings);
                if st.bonds.len() < 3 {
                    // pad to one full 12-byte Bond for wgpu's minimum binding size
                    st.bonds = vec![-1; 3];
                }
                let nb = if st.bonds[0] == -1 { 0 } else { st.bonds.len() / 3 };
                println!("[scenario] bond_init: {} bonds created (bond_type {})", nb, self.bond_type);
            }
        }
        prog.shader_prog.restore(config, settings);
    }
}
