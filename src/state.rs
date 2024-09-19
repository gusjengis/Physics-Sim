use std::fmt::DebugTuple;
use std::fs::File;
use std::io::prelude::*;
use std::mem;
use std::path::PathBuf;

use crate::particle_def::Particle_Definition;
use crate::scripts::ScriptManager;
use crate::settings;
use crate::settings::BondType;
use crate::settings::ColorSource;
use crate::settings::Settings;
use crate::settings::Structure;
use crate::setup;
use bytemuck::{bytes_of, cast_slice};
use csv::*;
use flatbuffers::WIPOffset;
use naga::proc::index;
use rand::Rng;
use wgpu::Device;
use wgpu::Queue;
// use crate::
// use winit::*;
use crate::setup::*;
use crate::wgpu_config::*;
use crate::wgpu_prog::*;
use crate::wgpu_structs::*;

use wgpu::util::DeviceExt;

// import the flatbuffers runtime library
extern crate flatbuffers;

// import the generated code
#[allow(dead_code, unused_imports)]
#[path = "../schema_generated.rs"]
mod schema_generated;
pub use schema_generated::*;
#[path = "../new_schema_generated.rs"]
mod new_schema_generated;
pub use new_schema_generated::*;

pub struct GridInfo {
    pub total_cells: usize,
    pub cell_size: f32,
    pub cell_cap: i32,
    pub w: i32,
    pub h: i32,
}

impl GridInfo {
    pub fn new(total_cells: usize, cell_size: f32, cell_cap: i32, w: i32, h: i32) -> Self {
        Self {
            total_cells,
            cell_size,
            cell_cap,
            w,
            h,
        }
    }

    pub fn as_vec(&self) -> Vec<f32> {
        return vec![self.cell_size, bytemuck::cast(self.cell_cap), bytemuck::cast(self.w), bytemuck::cast(self.h)];
    }
}

pub const DATA_SIZE: usize = 7;

pub struct State {
    pub p_count: usize,
    pub new_p: usize,
    pub pos: Vec<f32>,
    pub del_pos: Vec<f32>,
    pub vel: Vec<f32>,
    pub acc: Vec<f32>,
    pub rot: Vec<f32>,
    pub del_rot: Vec<f32>,
    pub rot_vel: Vec<f32>,
    pub rot_acc: Vec<f32>,
    pub forces: Vec<f32>,
    pub radii: Vec<f32>,
    pub fixity: Vec<i32>,
    pub bonds: Vec<i32>,
    pub material_pointers: Vec<i32>,
    pub selections: Vec<i32>,
    pub groups: Vec<i32>,
    pub contacts: Vec<f32>,
    pub contact_pointers: Vec<i32>,
    pub data: Vec<f32>,
    pub flatbuffer: Vec<u8>,
    pub grid: Vec<i32>,
    pub grid_info: GridInfo,
}

const PREALLOC: usize = 2;

impl State {
    pub fn new(config: &mut WGPUConfig, settings: &mut Settings, script_manager: &ScriptManager) -> Self {
        // Create empty arrays for particle data
        let p_count = setup::p_count(settings);
        let mut pos = vec![0.0 as f32; p_count * PREALLOC * 2];
        let mut del_pos = vec![0.0 as f32; p_count * PREALLOC * 2];
        let mut vel = vec![0.0 as f32; p_count * PREALLOC * 2];
        let mut acc = vec![0.0 as f32; p_count * PREALLOC * 2];
        let mut rot = vec![0.0 as f32; p_count * PREALLOC];
        let mut del_rot = vec![0.0 as f32; p_count * PREALLOC];
        let mut rot_vel = vec![0.0 as f32; p_count * PREALLOC];
        let mut rot_acc = vec![0.0 as f32; p_count * PREALLOC];
        let mut forces = vec![0.0 as f32; p_count * PREALLOC * 6];
        let mut radii = vec![0.0 as f32; p_count * PREALLOC];
        let mut fixity = vec![0; p_count * PREALLOC * 6];
        let mut bonds = vec![-1; 1];
        let mut material_pointers = vec![0; p_count * PREALLOC];
        let mut selections = vec![0; p_count * PREALLOC];
        let mut groups = vec![0; p_count * PREALLOC];
        let mut contacts = vec![bytemuck::cast::<i32, f32>(-1); 6 * settings.setup.max_contacts * p_count * PREALLOC];
        let mut contact_pointers = vec![-1; settings.setup.max_contacts * p_count * PREALLOC];
        let mut data = vec![0.0; p_count * PREALLOC * DATA_SIZE];
        let flatbuffer = vec![0 as u8; 1];
        // Setup initial state
        setup::grid(settings, &mut pos, &mut vel, &mut radii, &mut fixity, &mut forces, &mut material_pointers);
        let grid_info_return = grid_capacity(&settings);

        let mut state = State {
            p_count,
            new_p: 0,
            pos,
            del_pos,
            vel,
            acc,
            rot,
            del_rot,
            rot_vel,
            rot_acc,
            forces,
            radii,
            fixity,
            bonds,
            material_pointers,
            selections,
            groups,
            contacts,
            contact_pointers,
            data,
            flatbuffer,
            grid: vec![0; 1],
            grid_info: GridInfo::new(grid_info_return.0, grid_info_return.1, grid_info_return.2, grid_info_return.3, grid_info_return.4),
        };

        // state.load_from_csv(PathBuf::from("./saved_states/particle_state.csv"), settings);
        state.regen_bonds(config, settings);
        state.save(config, settings, Some(script_manager));

        return state;
    }

    pub fn print_state(&self) {
        print!("Positions:");
        for i in 0..self.pos.len() {
            if i % 2 == 0 {
                print!("\n    ");
            }
            print!("{}, ", self.pos[i]);
        }

        print!("\nRadii:\n");
        for i in 0..self.radii.len() {
            print!("    {}, \n", self.radii[i]);
        }

        print!("Velocities:");
        for i in 0..self.vel.len() {
            if i % 2 == 0 {
                print!("\n    ");
            }
            print!("{}, ", self.vel[i]);
        }

        print!("\nRotations:");
        for i in 0..self.rot.len() {
            print!("    {}, \n", self.rot[i]);
        }

        print!("Rotational Velocities: \n");
        for i in 0..self.rot_vel.len() {
            print!("    {}, \n", self.rot_vel[i]);
        }

        print!("Forces:");
        for i in 0..self.forces.len() {
            if i % 6 == 0 {
                print!("\n    ");
            }
            print!("{}, ", self.forces[i]);
        }

        print!("\nFixity:");
        for i in 0..self.forces.len() {
            if i % 3 == 0 {
                print!("\n    ");
            }
            print!("{}, ", self.forces[i]);
        }
    }

    pub fn spawn_particle(&mut self, x: f32, y: f32, p_def: usize, settings: &mut Settings) {
        let p_def = &mut settings.create.particle_defs[p_def];
        self.pos[2 * self.p_count] = x;
        self.pos[2 * self.p_count + 1] = y;
        self.radii[self.p_count] = p_def.next_radius;
        self.rot[self.p_count] = 0.0;
        self.vel[2 * self.p_count] = p_def.x_vel;
        self.vel[2 * self.p_count + 1] = p_def.y_vel;
        self.rot_vel[self.p_count] = p_def.rot_vel;
        self.forces[6 * self.p_count] = p_def.x_force;
        self.forces[6 * self.p_count + 1] = p_def.y_force;
        self.forces[6 * self.p_count + 2] = p_def.rot_force;
        self.fixity[6 * self.p_count] = p_def.x_fixity as i32;
        self.fixity[6 * self.p_count + 1] = p_def.y_fixity as i32;
        self.fixity[6 * self.p_count + 2] = p_def.rot_fixity as i32;
        self.material_pointers[self.p_count] = p_def.material;
        self.p_count += 1;
        p_def.new_radius();
        settings.set_particles(settings.setup.particles + 1);
    }

    pub fn store_particle(&mut self, index: usize, x: f32, y: f32, p_def: usize, settings: &mut Settings) {
        let p_def = &mut settings.create.particle_defs[p_def];
        self.pos[2 * index] = x;
        self.pos[2 * index + 1] = y;
        self.radii[index] = p_def.next_radius;
        self.rot[index] = 0.0;
        self.vel[2 * index] = p_def.x_vel;
        self.vel[2 * index + 1] = p_def.y_vel;
        self.rot_vel[index] = p_def.rot_vel;
        self.forces[6 * index] = p_def.x_force;
        self.forces[6 * index + 1] = p_def.y_force;
        self.forces[6 * index + 2] = p_def.rot_force;
        self.fixity[6 * index] = p_def.x_fixity as i32;
        self.fixity[6 * index + 1] = p_def.y_fixity as i32;
        self.fixity[6 * index + 2] = p_def.rot_fixity as i32;
        self.material_pointers[index] = p_def.material;
    }

    pub fn realloc(&mut self, settings: &mut Settings) {
        self.pos.resize(self.pos.len() * 2, 0.0);
        self.radii.resize(self.radii.len() * 2, 0.0);
        self.vel.resize(self.vel.len() * 2, 0.0);
        self.acc.resize(self.acc.len() * 2, 0.0);
        self.rot.resize(self.rot.len() * 2, 0.0);
        self.rot_vel.resize(self.rot_vel.len() * 2, 0.0);
        self.rot_acc.resize(self.rot_acc.len() * 2, 0.0);
        self.fixity.resize(self.fixity.len() * 2, 0);
        self.forces.resize(self.forces.len() * 2, 0.0);
        self.del_pos.resize(self.del_pos.len() * 2, 0.0);
        self.del_rot.resize(self.del_rot.len() * 2, 0.0);
        self.contacts.resize(self.contacts.len() * 2, bytemuck::cast::<i32, f32>(-1));
        self.material_pointers.resize(self.material_pointers.len() * 2, 0);
        self.selections.resize(self.selections.len() * 2, 0);
        self.groups.resize(self.groups.len() * 2, 0);
        self.data.resize(self.data.len() * 2, 0.0);
    }

    pub fn update_state(&mut self, config: &mut WGPUConfig, settings: &Settings, buffers: &mut BufferContainer) {
        self.p_count = settings.setup.particles;
        State::update_f32(&mut config.device, &mut config.queue, &mut self.pos, &mut buffers.pos_buffers.buffers[0]);
        State::update_f32(&mut config.device, &mut config.queue, &mut self.radii, &mut buffers.pos_buffers.buffers[1]);
        State::update_f32(&mut config.device, &mut config.queue, &mut self.vel, &mut buffers.mov_buffers.buffers[0]);
        State::update_f32(&mut config.device, &mut config.queue, &mut self.acc, &mut buffers.mov_buffers.buffers[1]);
        State::update_f32(&mut config.device, &mut config.queue, &mut self.rot, &mut buffers.mov_buffers.buffers[2]);
        State::update_f32(&mut config.device, &mut config.queue, &mut self.rot_vel, &mut buffers.mov_buffers.buffers[3]);
        State::update_f32(&mut config.device, &mut config.queue, &mut self.rot_acc, &mut buffers.mov_buffers.buffers[4]);
        State::update_i32(&mut config.device, &mut config.queue, &mut self.fixity, &mut buffers.mov_buffers.buffers[6]);
        State::update_f32(&mut config.device, &mut config.queue, &mut self.forces, &mut buffers.mov_buffers.buffers[7]);
        State::update_f32(&mut config.device, &mut config.queue, &mut self.del_pos, &mut buffers.mov_buffers.buffers[8]);
        State::update_f32(&mut config.device, &mut config.queue, &mut self.del_rot, &mut buffers.mov_buffers.buffers[9]);
        State::update_i32(&mut config.device, &mut config.queue, &mut self.bonds, &mut buffers.contact_buffers.buffers[0]);
        // State::update_i32(&mut config.device, &mut config.queue, &mut self.bond_info, &mut buffers.contact_buffers.buffers[1]);
        State::update_f32(&mut config.device, &mut config.queue, &mut self.contacts, &mut buffers.contact_buffers.buffers[1]);
        State::update_i32(&mut config.device, &mut config.queue, &mut self.material_pointers, &mut buffers.contact_buffers.buffers[3]);
        State::update_i32(&mut config.device, &mut config.queue, &mut self.selections, &mut buffers.selection_buffers.buffers[0]);
        State::update_i32(&mut config.device, &mut config.queue, &mut self.groups, &mut buffers.selection_buffers.buffers[1]);
        State::update_f32(&mut config.device, &mut config.queue, &mut self.data, &mut buffers.data_buffer.buffer);
        // State::update_i32(config.device, config.queue, &mut self.grid, &mut buffers.contact_buffers.buffers[4]);
        // for n in self.grid.iter() {
        //     println!("{}", n);

        // }
    }

    pub fn regen_bonds(&mut self, config: &mut WGPUConfig, settings: &Settings) {
        let MAX_BONDS = settings.setup.max_bonds;
        let mut bonds = vec![-1; self.p_count * MAX_BONDS * 3 * PREALLOC];
        let mut contacts = vec![bytemuck::cast::<i32, f32>(-1); 6 * settings.setup.max_contacts * self.p_count * PREALLOC];
        let mut found_bonds = true;
        let mut bond_index = 0;
        for i in 0..self.p_count {
            let mut col_num = 0;
            for j in i..self.p_count {
                if i != j {
                    let distance = ((self.pos[j * 2] - self.pos[i * 2]).powf(2.0) + (self.pos[j * 2 + 1] - self.pos[i * 2 + 1]).powf(2.0)).sqrt();
                    let sum_of_radii = (self.radii[i] + self.radii[j]);
                    if distance < sum_of_radii * 1.05 {
                        // bond detected
                        if col_num < MAX_BONDS && bonds[(i * MAX_BONDS + col_num) * 3] == -1 {
                            // CREATE BOND
                            let delta = (self.pos[j * 2] - self.pos[i * 2], self.pos[j * 2 + 1] - self.pos[i * 2 + 1]);
                            let magnitude = (delta.0 * delta.0 + delta.1 * delta.1).powf(0.5);
                            let normalized_delta = (delta.0 / magnitude, delta.1 / magnitude);
                            let angle = normalized_delta.0.atan2(normalized_delta.1);
                            // println!("({}, {}) vs ({}, {})", normalized_delta.0, normalized_delta.1, angle.sin(), angle.cos());
                            bonds[bond_index * 3] = 1 as i32; // torn
                            bonds[bond_index * 3 + 1] = (angle).to_bits() as i32;
                            bonds[bond_index * 3 + 2] = (magnitude).to_bits() as i32;
                            // println!("{}, {}, {}", bonds[(i*MAX_BONDS+col_num)*3], angle, magnitude);

                            // CREATE CONTACTS
                            for k in settings.setup.max_contacts * i..settings.setup.max_contacts * (i + 1) {
                                // println!("{}", bytemuck::cast::<f32, i32>(contacts[4*k]));
                                if bytemuck::cast::<f32, i32>(contacts[6 * k]) == -1 {
                                    contacts[6 * k] = bytemuck::cast(i as i32); //a
                                    contacts[6 * k + 1] = bytemuck::cast(j as i32); //b
                                    contacts[6 * k + 2] = 0.0; // tangent force
                                    contacts[6 * k + 3] = 0.0; // tangent force
                                    contacts[6 * k + 4] = 0.0; // theta b
                                    contacts[6 * k + 5] = bytemuck::cast(bond_index as i32); // bonded
                                    break;
                                }
                            }

                            for k in settings.setup.max_contacts * j..settings.setup.max_contacts * (j + 1) {
                                if bytemuck::cast::<f32, i32>(contacts[6 * k]) == -1 {
                                    contacts[6 * k] = bytemuck::cast(j as i32);
                                    contacts[6 * k + 1] = bytemuck::cast(i as i32);
                                    contacts[6 * k + 2] = 0.0;
                                    contacts[6 * k + 3] = 0.0;
                                    contacts[6 * k + 4] = 0.0;
                                    contacts[6 * k + 5] = bytemuck::cast(bond_index as i32);
                                    break;
                                }
                            }

                            col_num += 1;
                            bond_index += 1;
                            found_bonds = true;
                        } else if col_num == MAX_BONDS {
                            break;
                        }
                    }
                }
            }
        }
        if found_bonds {
            bonds = (bonds).into_iter().filter(|num| *num != -1).collect();
        }
        if bonds.is_empty() {
            bonds = vec![-1; 1];
        }
        // for num in bonds.clone() {
        //     println!("Bonds: {}", num);
        // }
        // for num in contacts.clone() {
        //     println!("Contacts: {}", num);
        // }
        self.bonds = bonds;
        self.contacts = contacts;
    }

    pub fn save(&mut self, config: &mut WGPUConfig, settings: &Settings, script_manager: Option<&ScriptManager>) {
        let mut builder = flatbuffers::FlatBufferBuilder::new();

        let pos = builder.create_vector(&self.pos);
        let vel = builder.create_vector(&self.vel);
        let acc = builder.create_vector(&self.acc);
        let rot = builder.create_vector(&self.rot);
        let rot_vel = builder.create_vector(&self.rot_vel);
        let rot_acc = builder.create_vector(&self.rot_acc);
        let forces = builder.create_vector(&self.forces);
        let radii = builder.create_vector(&self.radii);
        let fixity = builder.create_vector(&self.fixity);
        let bonds = builder.create_vector(&self.bonds);
        let contacts = builder.create_vector(&self.contacts);
        let material_pointers = builder.create_vector(&self.material_pointers);
        let materials = builder.create_vector(&settings.materials);
        let groups = builder.create_vector(&self.groups);
        let mut scripts;
        match script_manager {
            Some(sm) => {
                scripts = Some(builder.create_string(&sm.to_json()));
            }
            None => scripts = Some(builder.create_string(&String::from(""))),
        }
        let view_settings = new_schema_generated::View_Settings::new(
            settings.view.circular_particles,
            settings.view.render_rot,
            settings.view.render_bonds,
            settings.view.render_outline,
            settings.view.render_bp_grid,
            settings.view.color_code_rot,
            settings.view.use_particle_color_outline,
            settings.view.outline_color[0],
            settings.view.outline_color[1],
            settings.view.outline_color[2],
            settings.view.background_color[0],
            settings.view.background_color[1],
            settings.view.background_color[2],
            settings.view.color_source.as_i32(),
            settings.view.dim_slow_particles,
            settings.view.max_brightness_vel,
            settings.view.crt_res,
            settings.view.grain,
            settings.view.grain_strength,
            settings.view.grain_size,
            settings.view.sobel,
            settings.view.colored_sobel,
            settings.view.invert,
            settings.view.chrom_ab,
            settings.view.abb_strength,
            settings.view.bond_highlight_strength,
            settings.view.render_unbonded_contacts,
            settings.view.lighting,
            settings.view.show_hit_tex,
        );
        let setup_settings = new_schema_generated::Setup_Settings::new(
            settings.setup.particles as i32,
            settings.setup.max_radius,
            settings.setup.variable_rad,
            settings.setup.holeyness,
            settings.setup.max_h_velocity,
            settings.setup.min_h_velocity,
            settings.setup.max_v_velocity,
            settings.setup.min_v_velocity,
            settings.setup.grid_width,
            settings.setup.hex_grid,
        );
        let simulation_settings = new_schema_generated::Simulation_Settings::new(
            settings.simulation.timestep,
            settings.simulation.gen_per_frame,
            settings.simulation.auto_width,
            settings.simulation.hor_bound,
            settings.simulation.vert_bound,
            settings.simulation.maintain_ar,
            settings.simulation.round_walls,
            settings.simulation.wall_radius,
            settings.simulation.d3,
            settings.simulation.x_timesteps,
            settings.simulation.use_f64,
        );
        let physics_settings = new_schema_generated::Physics_Settings::new(
            settings.physics.gravity,
            settings.physics.gravity_acceleration,
            settings.physics.planet_mode,
            settings.physics.mouse_gravity,
            settings.physics.collisions,
            settings.physics.collision_interval,
            settings.physics.friction_coefficient,
            settings.physics.bondenum.as_i32(),
            settings.physics.bond_tearing,
            settings.physics.bond_normal_stiffness,
            settings.physics.bond_shear_stiffness,
            settings.physics.bond_normal_strength,
            settings.physics.bond_shear_strength,
            settings.physics.moment_contribution_factor,
            settings.physics.local_damping,
            settings.physics.local_damping_alpha,
        );
        let settings = new_schema_generated::Settings::new(&view_settings, &setup_settings, &simulation_settings, &physics_settings);
        let state = new_schema_generated::State::create(
            &mut builder,
            &new_schema_generated::StateArgs {
                pos: Some(pos),
                vel: Some(vel),
                acc: Some(acc),
                rot: Some(rot),
                rot_vel: Some(rot_vel),
                rot_acc: Some(rot_acc),
                forces: Some(forces),
                radii: Some(radii),
                fixity: Some(fixity),
                bonds: Some(bonds),
                contacts: Some(contacts),
                material_pointers: Some(material_pointers),
                materials: Some(materials),
                groups: Some(groups),
                scripts: scripts,
                settings: Some(&settings),
            },
        );

        builder.finish(state, None);

        self.flatbuffer = builder.finished_data().to_vec();
    }

    pub fn save_to_file(&self, path: std::path::PathBuf) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(&self.flatbuffer)?;
        Ok(())
    }

    pub fn load_from_file(&mut self, path: PathBuf) -> std::io::Result<()> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        self.flatbuffer = buffer;
        Ok(())
    }

    pub fn load_from_csv(&mut self, path: std::path::PathBuf, settings: &mut Settings) {
        let mut reader = csv::Reader::from_path(path.clone()).unwrap();

        self.p_count = reader.records().count();
        settings.set_particles(self.p_count);
        self.pos = vec![0.0 as f32; self.p_count * 2];
        self.vel = vec![0.0 as f32; self.p_count * 2];
        self.acc = vec![0.0 as f32; self.p_count * 2];
        self.rot = vec![0.0 as f32; self.p_count];
        self.rot_vel = vec![0.0 as f32; self.p_count];
        self.rot_acc = vec![0.0 as f32; self.p_count];
        self.forces = vec![0.0 as f32; self.p_count * 6];
        self.radii = vec![0.0 as f32; self.p_count];
        self.fixity = vec![0; self.p_count * 6];
        self.bonds = vec![-1; 1];
        self.material_pointers = vec![0; self.p_count];
        self.selections = vec![0; self.p_count];
        self.groups = vec![0; self.p_count];
        self.contacts = vec![bytemuck::cast::<i32, f32>(-1); 6 * settings.setup.max_contacts * self.p_count];
        self.contact_pointers = vec![-1; settings.setup.max_contacts * self.p_count];
        self.data = vec![0.0; self.p_count * DATA_SIZE];

        reader = csv::Reader::from_path(path).unwrap();
        let mut i = 0;
        for result in reader.records() {
            let record = result.unwrap();
            for (j, field) in record.iter().enumerate() {
                match j {
                    1 => match field {
                        " None" => {
                            self.groups[i] = 0;
                        }
                        " bottom_grip" => {
                            self.groups[i] = 1;
                            self.material_pointers[i] = 1;
                        }
                        " top_grip" => {
                            self.groups[i] = 2;
                            self.material_pointers[i] = 1;
                        }
                        _ => {}
                    },
                    2 => {
                        self.pos[i * 2] = field[1..].parse::<f32>().unwrap();
                    }
                    3 => {
                        self.pos[i * 2 + 1] = field[1..].parse::<f32>().unwrap();
                    }
                    4 => {
                        self.radii[i] = field[1..].parse::<f32>().unwrap();
                    }
                    _ => {}
                }
            }
            i = i + 1;
        }
    }

    pub fn get_min_max_radii(self) -> (f32, f32) {
        let mut min = f32::MAX;
        let mut max = f32::MIN;

        for num in self.radii {
            if num > max {
                max = num;
            }
            if num < min {
                min = num;
            }
        }

        return (min, max);
    }

    pub fn load(&mut self, config: &mut WGPUConfig, settings: &mut Settings, script_manager: Option<&mut ScriptManager>, init: bool) {
        let state = new_schema_generated::root_as_state(self.flatbuffer.as_slice()).unwrap();
        let vs = state.settings().unwrap().view_settings();
        let sts = state.settings().unwrap().setup_settings();
        let sms = state.settings().unwrap().sim_settings();
        let ps = state.settings().unwrap().physics_settings();

        if init {
            settings.materials = State::f32_vec_from_vector(state.materials());
            // view settings
            settings.view.circular_particles = vs.circular_particles();
            settings.view.render_rot = vs.render_rot();
            settings.view.render_bonds = vs.render_bonds();
            settings.view.render_outline = vs.render_outline();
            settings.view.render_bp_grid = vs.render_bp_grid();
            settings.view.color_code_rot = vs.color_code_rot();
            settings.view.use_particle_color_outline = vs.use_particle_color_outline();
            settings.view.outline_color[0] = vs.outline_color_r();
            settings.view.outline_color[1] = vs.outline_color_g();
            settings.view.outline_color[2] = vs.outline_color_b();
            settings.view.background_color[0] = vs.background_color_r();
            settings.view.background_color[1] = vs.background_color_g();
            settings.view.background_color[2] = vs.background_color_b();
            settings.view.color_source = ColorSource::from_i32(vs.color_source());
            settings.view.dim_slow_particles = vs.dim_slow_particles();
            settings.view.max_brightness_vel = vs.max_brightness_vel();
            settings.view.crt_res = vs.crt_res();
            settings.view.grain = vs.grain();
            settings.view.grain_strength = vs.grain_strength();
            settings.view.grain_size = vs.grain_size();
            settings.view.sobel = vs.sobel();
            settings.view.colored_sobel = vs.colored_sobel();
            settings.view.invert = vs.invert();
            settings.view.chrom_ab = vs.chrom_ab();
            settings.view.abb_strength = vs.abb_strength();
            settings.view.bond_highlight_strength = vs.bond_highlight_strength();
            settings.view.render_unbonded_contacts = vs.render_unbonded_contacts();
            settings.view.lighting = vs.lighting();
            settings.view.show_hit_tex = vs.show_hit_tex();
            // state settings
            settings.setup.max_radius = sts.max_radius();
            settings.setup.variable_rad = sts.variable_rad();
            settings.setup.holeyness = sts.holeyness();
            settings.setup.max_h_velocity = sts.max_h_velocity();
            settings.setup.min_h_velocity = sts.min_h_velocity();
            settings.setup.max_v_velocity = sts.max_v_velocity();
            settings.setup.min_v_velocity = sts.min_v_velocity();
            settings.setup.grid_width = sts.grid_width();
            settings.setup.hex_grid = sts.hex_grid();
            // simulation settings
            settings.simulation.timestep = sms.timestep();
            settings.simulation.gen_per_frame = sms.gen_per_frame();
            settings.simulation.auto_width = sms.auto_width();
            settings.simulation.hor_bound = sms.width();
            settings.simulation.vert_bound = sms.height();
            settings.simulation.maintain_ar = sms.maintain_ar();
            settings.simulation.round_walls = sms.round_walls();
            settings.simulation.wall_radius = sms.wall_radius();
            settings.simulation.d3 = sms.d3();
            settings.simulation.x_timesteps = sms.x_timesteps();
            settings.simulation.use_f64 = sms.use_f64();
            // physics settings
            settings.physics.gravity = ps.gravity();
            settings.physics.gravity_acceleration = ps.g_force();
            settings.physics.planet_mode = ps.planet_mode();
            settings.physics.mouse_gravity = ps.mouse_gravity();
            settings.physics.collisions = ps.collisions();
            settings.physics.collision_interval = ps.collision_interval();
            settings.physics.friction_coefficient = ps.friction_coef();
            settings.physics.bondenum = BondType::from_i32(ps.bond());
            settings.physics.bond_tearing = ps.bond_tearing();
            settings.physics.bond_normal_stiffness = ps.bond_normal_stiffness();
            settings.physics.bond_shear_stiffness = ps.bond_shear_stiffness();
            settings.physics.bond_normal_strength = ps.bond_normal_strength();
            settings.physics.bond_shear_strength = ps.bond_shear_strength();
            settings.physics.moment_contribution_factor = ps.moment_contribution_factor();
            settings.physics.local_damping = ps.local_damping();
            settings.physics.local_damping_alpha = ps.local_damping_alpha();
            // set update flags
            settings.changed_collision_settings = true;
            settings.materials_changed = true;
            settings.rebuild_shaders = true;
            settings.updateBonds();
            match script_manager {
                Some(sm) => {
                    sm.from_json(state.scripts().unwrap());
                    sm.auto_run();
                }
                None => {}
            }
        }
        self.p_count = sts.particles() as usize;
        settings.set_particles(self.p_count);

        self.pos = State::f32_vec_from_vector(state.pos());
        self.radii = State::f32_vec_from_vector(state.radii());
        self.vel = State::f32_vec_from_vector(state.vel());
        self.acc = State::f32_vec_from_vector(state.acc());
        self.rot = State::f32_vec_from_vector(state.rot());
        self.rot_vel = State::f32_vec_from_vector(state.rot_vel());
        self.rot_acc = State::f32_vec_from_vector(state.rot_acc());
        self.forces = State::f32_vec_from_vector(state.forces());
        self.fixity = State::i32_vec_from_vector(state.fixity());
        self.bonds = State::i32_vec_from_vector(state.bonds());
        self.contacts = State::f32_vec_from_vector(state.contacts());
        self.material_pointers = State::i32_vec_from_vector(state.material_pointers());
        self.selections = vec![0; self.p_count];
        self.data = vec![0.0; DATA_SIZE * self.p_count];

        let mut min = f32::MAX;
        let mut max = f32::MIN;
        for radius in &self.radii {
            min = radius.min(min);
            max = radius.max(max);
        }

        settings.setup.min_radius = min;
        settings.setup.max_radius = max;
        settings.setup.variable_rad = min != max;

        let grid_info_return = grid_capacity(&settings);
        self.grid = vec![0; grid_info_return.0 * grid_info_return.2 as usize];
        self.grid_info = GridInfo::new(grid_info_return.0, grid_info_return.1, grid_info_return.2, grid_info_return.3, grid_info_return.4);
    }

    pub fn get_datum(&self, prop: &crate::settings::Property) -> Option<[f64; 10]> {
        let mut sums = [0.0; 10];
        let mut count = 0;
        for i in 0..self.selections.len() {
            if self.selections[i] != 0 {
                count += 1;
                sums[0] += self.pos[i * 2] as f64;
                sums[1] += self.pos[i * 2 + 1] as f64;
                sums[2] += self.data[i * DATA_SIZE + 4] as f64;
                sums[3] += self.data[i * DATA_SIZE + 5] as f64;
                sums[4] += self.rot[i] as f64;
                sums[5] += self.data[i * DATA_SIZE + 6] as f64;
                sums[6] += (self.data[i * DATA_SIZE] as f64).abs();
                sums[7] += self.data[i * DATA_SIZE + 1] as f64;
                sums[8] += self.data[i * DATA_SIZE + 2] as f64;
                sums[9] += self.data[i * DATA_SIZE + 3] as f64;
            }
        }

        if count == 0 {
            return None;
        }
        for i in 0..sums.len() {
            if i == 6 {
                continue;
            }
            sums[i] /= count as f64;
        }
        return Some(sums);
    }

    fn f32_vec_from_vector(vector: Option<flatbuffers::Vector<f32>>) -> Vec<f32> {
        let bytes = vector.unwrap().bytes();
        let f32_slice: &[f32] = unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const f32, bytes.len() / 4) };
        return f32_slice.to_vec();
    }

    fn i32_vec_from_vector(vector: Option<flatbuffers::Vector<i32>>) -> Vec<i32> {
        let bytes = vector.unwrap().bytes();
        let i32_slice: &[i32] = unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const i32, bytes.len() / 4) };
        return i32_slice.to_vec();
    }

    pub fn update_selections(&mut self, device: &mut Device, queue: &mut Queue, buffers: &mut BufferContainer) {
        State::update_i32(device, queue, &mut self.selections, &mut buffers.selection_buffers.buffers[0]);
    }

    pub fn update_i32(device: &mut Device, queue: &mut Queue, vector: &mut Vec<i32>, buffer: &mut wgpu::Buffer) {
        let buffer_size = (buffer.size()); // as usize * mem::size_of::<i32>()) as u64;

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            label: None,
            mapped_at_creation: false,
        });

        // Create a command encoder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Copy from the GPU buffer to the staging buffer
        encoder.copy_buffer_to_buffer(&buffer, 0, &staging_buffer, 0, buffer_size);

        // Submit the commands to the queue
        queue.submit(Some(encoder.finish()));

        // Requesting to map the buffer for reading
        let buffer_slice = staging_buffer.slice(..); // Get a slice of the buffer

        // Request to map the buffer for reading
        buffer_slice.map_async(wgpu::MapMode::Read, |result| {
            match result {
                Ok(()) => {
                    // Mapping succeeded, handle the data
                }
                Err(e) => {
                    // Handle the error
                    eprintln!("Buffer map failed: {:?}", e);
                }
            }
        }); // buffer_size is the size of the buffer

        // Poll the device in a loop or in an event-driven manner
        device.poll(wgpu::Maintain::Wait);

        // Once the buffer is mapped, get the mapped range
        {
            let mapped_range = buffer_slice.get_mapped_range();

            // Access the data
            // For example, if your buffer contains byte data, you might convert it to a byte slice
            let data: &[u8] = mapped_range.as_ref();
            // You can now read from `data` as needed

            *vector = bytemuck::cast_slice(&data).to_vec();
        }
        // After you're done with the data, unmap the buffer
        staging_buffer.unmap();
    }

    pub fn update_f32(device: &mut Device, queue: &mut Queue, vector: &mut Vec<f32>, buffer: &mut wgpu::Buffer) {
        let buffer_size = (buffer.size()); // as usize * mem::size_of::<f32>()) as u64;

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            label: None,
            mapped_at_creation: false,
        });

        // Create a command encoder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Copy from the GPU buffer to the staging buffer
        encoder.copy_buffer_to_buffer(&buffer, 0, &staging_buffer, 0, buffer_size);

        // Submit the commands to the queue
        queue.submit(Some(encoder.finish()));

        // Requesting to map the buffer for reading
        let buffer_slice = staging_buffer.slice(..); // Get a slice of the buffer

        // Request to map the buffer for reading
        buffer_slice.map_async(wgpu::MapMode::Read, |result| {
            match result {
                Ok(()) => {
                    // Mapping succeeded, handle the data
                }
                Err(e) => {
                    // Handle the error
                    eprintln!("Buffer map failed: {:?}", e);
                }
            }
        }); // buffer_size is the size of the buffer

        // Poll the device in a loop or in an event-driven manner
        device.poll(wgpu::Maintain::Wait);

        // Once the buffer is mapped, get the mapped range
        {
            let mapped_range = buffer_slice.get_mapped_range();

            // Access the data
            // For example, if your buffer contains byte data, you might convert it to a byte slice
            let data: &[u8] = mapped_range.as_ref();
            // You can now read from `data` as needed

            *vector = bytemuck::cast_slice(&data).to_vec();
        }
        // After you're done with the data, unmap the buffer
        staging_buffer.unmap();
    }
}
