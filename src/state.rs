use std::fmt::DebugTuple;
use std::mem;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use bytemuck::{bytes_of, cast_slice};
use rand::Rng;
use crate::settings;
use crate::settings::BondType;
use crate::settings::ColorSource;
use crate::settings::Structure;
use crate::setup;
// use crate::
// use winit::*;
use crate::wgpu_structs::*;
use crate::wgpu_config::*;
use crate::wgpu_prog::*;
use crate::setup::*;

use wgpu::util::DeviceExt;

// import the flatbuffers runtime library
extern crate flatbuffers;

// import the generated code
#[allow(dead_code, unused_imports)]
#[path = "../schema_generated.rs"]
mod schema_generated;
pub use schema_generated::*;

pub struct State {
    pub p_count: usize,
    pub pos: Vec<f32>,
    pub vel: Vec<f32>,
    pub acc: Vec<f32>,
    pub rot: Vec<f32>,
    pub rot_vel: Vec<f32>,
    pub rot_acc: Vec<f32>,
    pub forces: Vec<f32>,
    pub radii: Vec<f32>,
    pub fixity: Vec<i32>,
    pub bonds: Vec<i32>,
    pub material_pointers: Vec<i32>,
    pub selections: Vec<i32>,
    pub contacts: Vec<f32>,
    pub contact_pointers: Vec<i32>,
    pub data: Vec<f32>,
    pub flatbuffer: Vec<u8>,
    pub grid: Vec<i32>
}

impl State {
    pub fn new(config: &mut WGPUConfig) -> Self {
        // Create empty arrays for particle data
        let p_count = setup::p_count(&mut config.prog_settings);
        let mut pos = vec![0.0 as f32; p_count*2];
        let mut vel = vec![0.0 as f32; p_count*2];
        let mut acc = vec![0.0 as f32; p_count*2];
        let mut rot = vec![0.0 as f32; p_count];
        let mut rot_vel = vec![0.0 as f32; p_count];
        let mut rot_acc = vec![0.0 as f32; p_count];
        let mut forces = vec![0.0 as f32; p_count*6];
        let mut radii = vec![0.0 as f32; p_count];
        let mut fixity = vec![0; p_count*6];
        let mut bonds = vec![-1; 1];
        // let mut bond_info = vec![-1; 1];
        let mut material_pointers = vec![0; p_count];
        let mut selections = vec![0; p_count];
        let mut contacts = vec![bytemuck::cast::<i32, f32>(-1); 6*config.prog_settings.max_contacts*p_count];
        let mut contact_pointers = vec![-1; config.prog_settings.max_contacts*p_count];
        let mut data = vec![0.0; p_count * 4];
        let flatbuffer = vec![0 as u8; 1];
        // Setup initial state
        setup::grid(&mut config.prog_settings, &mut pos, &mut vel, &mut radii, &mut fixity, &mut forces, &mut material_pointers);

        let mut state = State {
            p_count,
            pos,
            vel,
            acc,
            rot,
            rot_vel,
            rot_acc,
            forces,
            radii,
            fixity,
            bonds,
            material_pointers,
            selections,
            contacts,
            contact_pointers,
            data,
            flatbuffer,
            grid:  vec![0; 1]
        };

        state.regen_bonds(config);
        state.save(config);

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

    pub fn regen_bonds(&mut self, config: &mut WGPUConfig) {

        let MAX_BONDS = config.prog_settings.max_bonds;
        let mut bonds = vec![-1; self.p_count*MAX_BONDS*3];
        let mut contacts = vec![bytemuck::cast::<i32, f32>(-1); 6*config.prog_settings.max_contacts*self.p_count];
        let mut found_bonds = true;
        let mut bond_index = 0;
        for i in 0..self.p_count {
            let mut col_num = 0;
            for j in i..self.p_count {
                if i != j {
                    let distance = ((self.pos[j*2] - self.pos[i*2]).powf(2.0) + (self.pos[j*2+1] - self.pos[i*2+1]).powf(2.0)).sqrt();
                    let sum_of_radii = (self.radii[i] + self.radii[j]);
                    if distance < sum_of_radii*1.05 { // bond detected
                        if col_num < MAX_BONDS && bonds[(i*MAX_BONDS+col_num)*3] == -1 {
                            
                            // CREATE BOND
                            let delta = (self.pos[j*2] - self.pos[i*2], self.pos[j*2+1] - self.pos[i*2+1]);
                            let magnitude = (delta.0*delta.0 + delta.1*delta.1).powf(0.5);
                            let normalized_delta = (delta.0/magnitude, delta.1/magnitude);
                            let angle = normalized_delta.0.atan2(normalized_delta.1);
                            // println!("({}, {}) vs ({}, {})", normalized_delta.0, normalized_delta.1, angle.sin(), angle.cos());
                            bonds[bond_index*3] = 1 as i32; // torn
                            bonds[bond_index*3+1] = (angle).to_bits() as i32;
                            bonds[bond_index*3+2] = (magnitude).to_bits() as i32;
                            // println!("{}, {}, {}", bonds[(i*MAX_BONDS+col_num)*3], angle, magnitude);

                            // CREATE CONTACTS
                            for k in config.prog_settings.max_contacts*i..config.prog_settings.max_contacts*(i+1) {
                                // println!("{}", bytemuck::cast::<f32, i32>(contacts[4*k]));
                                if bytemuck::cast::<f32, i32>(contacts[6*k]) == -1 {
                                    contacts[6*k] = bytemuck::cast(i as i32);    //a
                                    contacts[6*k+1] = bytemuck::cast(j as i32);  //b
                                    contacts[6*k+2] = 0.0; // tangent force
                                    contacts[6*k+3] = 0.0; // tangent force
                                    contacts[6*k+4] = 0.0; // theta b 
                                    contacts[6*k+5] = bytemuck::cast(bond_index as i32); // bonded
                                    break;
                                }
                            }

                            for k in config.prog_settings.max_contacts*j..config.prog_settings.max_contacts*(j+1) {
                                if bytemuck::cast::<f32, i32>(contacts[6*k]) == -1 {
                                    contacts[6*k] = bytemuck::cast(j as i32);
                                    contacts[6*k+1] = bytemuck::cast(i as i32);
                                    contacts[6*k+2] = 0.0;
                                    contacts[6*k+3] = 0.0;
                                    contacts[6*k+4] = 0.0;
                                    contacts[6*k+5] = bytemuck::cast(bond_index as i32);
                                    break;
                                }
                            }

                            col_num += 1;
                            bond_index += 1;
                            found_bonds = true;
                        } else if col_num == MAX_BONDS{
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
        self.bonds     = bonds;
        self.contacts  = contacts;
    }

    pub fn save(&mut self, config: &mut WGPUConfig) {
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
        let materials = builder.create_vector(&config.prog_settings.materials);
        let wall_settings = schema_generated::Wall_Settings::new(
            config.prog_settings.maintain_ar,
            config.prog_settings.hor_bound/2.0,
            config.prog_settings.vert_bound/2.0,
        );
        let render_settings = schema_generated::Render_Settings::new(
            config.prog_settings.circular_particles,
            config.prog_settings.render_rot,
            config.prog_settings.render_bonds,
            true,// config.prog_settings.colors,
            false,//config.prog_settings.random_colors,
            config.prog_settings.color_code_rot,
        );
        let physics_settings = schema_generated::Physics_Settings::new(
            config.prog_settings.timestep,
            config.prog_settings.genPerFrame,
            config.prog_settings.gravity,
            config.prog_settings.planet_mode,
            config.prog_settings.gravity_acceleration,
            config.prog_settings.contact_damping,
            config.prog_settings.bondenum.as_i32(),
            config.prog_settings.bond_normal_stiffness,
            config.prog_settings.collisions,
            config.prog_settings.friction_coefficient,
        );
        let settings = schema_generated::Settings::new(
            &physics_settings,
            &render_settings,
            &wall_settings
        );
        let state = schema_generated::State::create(&mut builder, &schema_generated::StateArgs{
            particles: self.p_count as i32,
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
            settings: Some(&settings)
        });

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

    pub fn get_min_max_radii(self) -> (f32, f32) {
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        
        for num in self.radii {
            if num > max { max = num; }
            if num < min { min = num; }
        }

        return (min, max);
    }

    pub fn load(&mut self, config: &mut WGPUConfig, init: bool) {
        let state = schema_generated::root_as_state(self.flatbuffer.as_slice()).unwrap();
        let new_p_count = state.particles() as usize;
        self.p_count = new_p_count;
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
        if init {
            config.prog_settings.materials = State::f32_vec_from_vector(state.materials());
            let ws = state.settings().unwrap().wall_settings();
            let ps = state.settings().unwrap().physics_settings();
            let rs = state.settings().unwrap().render_settings();
            // wall settings
            config.prog_settings.maintain_ar = ws.maintain_ar();
            config.prog_settings.hor_bound  = ws.width()*2.0;
            config.prog_settings.vert_bound = ws.height()*2.0;
            // render settings
            config.prog_settings.circular_particles = rs.circular_particles();
            config.prog_settings.render_rot         = rs.render_rotation();
            config.prog_settings.render_bonds       = rs.render_bonds();
            config.prog_settings.color_source       = ColorSource::from_i32(rs.colors() as i32);
            // config.prog_settings.random_colors      = rs.random_colors();
            config.prog_settings.color_code_rot     = rs.color_code_rotation();
            // physics settings
            config.prog_settings.timestep = ps.timestep();
            config.prog_settings.genPerFrame = ps.gen_per_frame();
            config.prog_settings.gravity = ps.gravity();
            config.prog_settings.planet_mode = ps.planet_mode();
            config.prog_settings.gravity_acceleration = ps.g_force();
            config.prog_settings.contact_damping = ps.contact_damping();
            config.prog_settings.bondenum = BondType::from_i32(ps.bond());
            config.prog_settings.bond_normal_stiffness = ps.bond_stiffness();
            config.prog_settings.collisions = ps.collisions();
            config.prog_settings.friction_coefficient = ps.friction_coef();
            // set update flags
            config.prog_settings.changed_collision_settings = true;
            config.prog_settings.materials_changed = true;
            config.prog_settings.updateBonds();
        }
        self.selections = vec![0; self.p_count];
        self.data = vec![0.0; 4*self.p_count];
    }

    pub fn get_datum(&self, prop: &crate::settings::Property) -> Option<[f64;10]> {

        let mut sums = [0.0; 10];
        let mut count = 0;
        for i in 0..self.selections.len() {
            if self.selections[i] != 0 {
                count += 1;
                sums[0] += self.pos[i*2]   as f64;
                sums[1] += self.pos[i*2+1] as f64;
                sums[2] += self.vel[i*2]   as f64;
                sums[3] += self.vel[i*2+1] as f64;
                sums[4] += self.rot[i]     as f64;
                sums[5] += self.rot_vel[i] as f64;
                sums[6] += self.data[i*4] as f64;
                sums[7] += self.data[i*4+1] as f64;
                sums[8] += self.data[i*4+2] as f64;
                sums[9] += self.data[i*4+3] as f64;
            }
        }

        if count == 0 {
            return None;
        }
        for i in 0..sums.len() {
            sums[i] /= count as f64;
        }
        return Some(sums);
    }

    fn f32_vec_from_vector(vector: Option<flatbuffers::Vector<f32>>) -> Vec<f32> {
        let bytes = vector.unwrap().bytes();
        let f32_slice: &[f32] = unsafe {
            std::slice::from_raw_parts(
                bytes.as_ptr() as *const f32,
                bytes.len() / 4,
            )
        };
        return f32_slice.to_vec();
    }

    fn i32_vec_from_vector(vector: Option<flatbuffers::Vector<i32>>) -> Vec<i32> {
        let bytes = vector.unwrap().bytes();
        let i32_slice: &[i32] = unsafe {
            std::slice::from_raw_parts(
                bytes.as_ptr() as *const i32,
                bytes.len() / 4,
            )
        };
        return i32_slice.to_vec();
    }

    pub fn update_state(&mut self, config: &mut WGPUConfig, buffers: &mut BufferContainer) {

        self.p_count = config.prog_settings.particles;
        State::update_f32(config, &mut self.pos, &mut buffers.pos_buffers.buffers[0]);
        State::update_f32(config, &mut self.radii, &mut buffers.pos_buffers.buffers[1]);
        State::update_f32(config, &mut self.vel, &mut buffers.mov_buffers.buffers[0]);
        State::update_f32(config, &mut self.acc, &mut buffers.mov_buffers.buffers[1]);
        State::update_f32(config, &mut self.rot, &mut buffers.mov_buffers.buffers[2]);
        State::update_f32(config, &mut self.rot_vel, &mut buffers.mov_buffers.buffers[3]);
        State::update_f32(config, &mut self.rot_acc, &mut buffers.mov_buffers.buffers[4]);
        State::update_i32(config, &mut self.fixity, &mut buffers.mov_buffers.buffers[6]);
        State::update_f32(config, &mut self.forces, &mut buffers.mov_buffers.buffers[7]);
        State::update_i32(config, &mut self.bonds, &mut buffers.contact_buffers.buffers[0]);
        // State::update_i32(config, &mut self.bond_info, &mut buffers.contact_buffers.buffers[1]);
        State::update_f32(config, &mut self.contacts, &mut buffers.contact_buffers.buffers[1]);
        State::update_i32(config, &mut self.material_pointers, &mut buffers.contact_buffers.buffers[3]);
        State::update_i32(config, &mut self.selections, &mut buffers.selections.buffer);
        State::update_f32(config, &mut self.data, &mut buffers.data_buffer.buffer);
        // State::update_i32(config, &mut self.grid, &mut buffers.contact_buffers.buffers[4]);
        // for n in self.grid.iter() {
        //     println!("{}", n);

        // }
    }

    pub fn update_i32(config: &mut WGPUConfig, vector: &mut Vec<i32>, buffer: &mut wgpu::Buffer) {
        
        let buffer_size = (buffer.size());// as usize * mem::size_of::<i32>()) as u64;

        let staging_buffer = config.device.create_buffer(&wgpu::BufferDescriptor {
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            label: None,
            mapped_at_creation: false,
        });
        
        // Create a command encoder
        let mut encoder = config.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        
        // Copy from the GPU buffer to the staging buffer
        encoder.copy_buffer_to_buffer(&buffer, 0, &staging_buffer, 0, buffer_size);
        
        // Submit the commands to the queue
        config.queue.submit(Some(encoder.finish()));
        
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
        config.device.poll(wgpu::Maintain::Wait);
        
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

    pub fn update_f32(config: &mut WGPUConfig, vector: &mut Vec<f32>, buffer: &mut wgpu::Buffer) {
        
        let buffer_size = (buffer.size());// as usize * mem::size_of::<f32>()) as u64;

        let staging_buffer = config.device.create_buffer(&wgpu::BufferDescriptor {
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            label: None,
            mapped_at_creation: false,
        });
        
        // Create a command encoder
        let mut encoder = config.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        
        // Copy from the GPU buffer to the staging buffer
        encoder.copy_buffer_to_buffer(&buffer, 0, &staging_buffer, 0, buffer_size);
        
        // Submit the commands to the queue
        config.queue.submit(Some(encoder.finish()));
        
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
        config.device.poll(wgpu::Maintain::Wait);
        
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

