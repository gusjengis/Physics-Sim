// use crate::{audio_controller::*, sound::*};
use crate::particle_def::Particle_Definition;
use crate::scripts::{self, Key, ScriptManager};
use crate::timeline_widget::Timeline;
use crate::wgpu_config::WGPUConfig;
use crate::wgpu_prog::WGPUProg;
use egui::color_picker::Alpha;
use egui::*;
use egui_plot::{Line, Plot, PlotPoints};
use rfd::FileDialog;
use scripts::*;
use serde::{Deserialize, Serialize};
use serde_json::*;
use std::f32::consts::PI;
use std::ffi::OsStr;
use std::fs::*;
use std::io::Write;
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::*;
use std::{fmt::Debug, io::Read};
use wgpu::{Device, Queue};

#[cfg(not(target_arch = "wasm32"))]
use crate::{state::State, wgpu_structs::Uniform, window_init::Canvas};

pub struct Properties {
    pub set_x_force: bool,
    pub set_y_force: bool,
    pub set_rot_force: bool,
    pub set_material: bool,
    pub set_x_fixity: bool,
    pub set_y_fixity: bool,
    pub set_rot_fixity: bool,
    pub set_x_pos: bool,
    pub set_y_pos: bool,
    pub set_rot: bool,
    pub set_x_vel: bool,
    pub set_y_vel: bool,
    pub set_rot_vel: bool,
    pub set_radius: bool,
    pub x_force: f32,
    pub y_force: f32,
    pub rot_force: f32,
    pub material: i32,
    pub x_fixity: bool,
    pub y_fixity: bool,
    pub rot_fixity: bool,
    pub x_pos: f32,
    pub y_pos: f32,
    pub rot: f32,
    pub x_vel: f32,
    pub y_vel: f32,
    pub rot_vel: f32,
    pub radius: f32,
}

pub struct Data {
    pub x_pos_data: Vec<[f64; 2]>,
    pub y_pos_data: Vec<[f64; 2]>,
    pub x_vel_data: Vec<[f64; 2]>,
    pub y_vel_data: Vec<[f64; 2]>,
    pub rot_data: Vec<[f64; 2]>,
    pub rot_vel_data: Vec<[f64; 2]>,
    pub data1: Vec<[f64; 2]>,
    pub data2: Vec<[f64; 2]>,
    pub data3: Vec<[f64; 2]>,
    pub data4: Vec<[f64; 2]>,
    pub fps: Vec<[f64; 2]>,
    pub torn_bonds: Vec<[f64; 2]>,
}

impl Data {
    pub fn new() -> Self {
        return Data {
            x_pos_data: Vec::new(),
            y_pos_data: Vec::new(),
            x_vel_data: Vec::new(),
            y_vel_data: Vec::new(),
            rot_data: Vec::new(),
            rot_vel_data: Vec::new(),
            data1: Vec::new(),
            data2: Vec::new(),
            data3: Vec::new(),
            data4: Vec::new(),
            fps: Vec::new(),
            torn_bonds: Vec::new(),
        };
    }

    pub fn push(&mut self, timestamp: f64, datum: [f64; 11], fps: f64) {
        self.x_pos_data.push([timestamp, datum[0]]);
        self.y_pos_data.push([timestamp, datum[1]]);
        self.x_vel_data.push([timestamp, datum[2]]);
        self.y_vel_data.push([timestamp, datum[3]]);
        self.rot_data.push([timestamp, datum[4]]);
        self.rot_vel_data.push([timestamp, datum[5]]);
        self.data1.push([timestamp, datum[6]]);
        self.data2.push([timestamp, datum[7]]);
        self.data3.push([timestamp, datum[8]]);
        self.data4.push([timestamp, datum[9]]);
        self.fps.push([timestamp, fps]);
        self.torn_bonds.push([timestamp, datum[10]]);
    }
}

pub struct ViewSettings {
    pub settings_menu: bool,
    pub scale: f32,
    pub rendering: bool,
    pub circular_particles: bool,
    pub render_rot: bool,
    pub render_bonds: bool,
    pub render_outline: bool,
    pub render_bp_grid: bool,
    pub color_code_rot: bool,
    pub use_particle_color_outline: bool,
    pub outline_color: [f32; 3],
    pub background_color: [f32; 3],
    pub color_source: ColorSource,
    pub dim_slow_particles: bool,
    pub max_brightness_vel: f32,
    pub crt_res: i32,
    pub grain: bool,
    pub grain_strength: f32,
    pub grain_size: i32,
    pub sobel: bool,
    pub colored_sobel: bool,
    pub invert: bool,
    pub chrom_ab: bool,
    pub abb_strength: f32,
    pub bond_highlight_strength: f32,
    pub render_unbonded_contacts: bool,
    pub lighting: bool,
    pub show_hit_tex: bool,
    pub data_menu: bool,
    pub script_menu: bool,
    pub code_editor: bool,
    // pub audio: AudioView,
}

// pub struct AudioView {
//     pub menu: bool,
//     pub left_tab: usize,
//     pub current_sound: i32,
//     pub current_source: i32,
//     pub waveform: bool,
//     pub oscilloscope: bool,
//     pub osc_life: f32,
//     pub timeline_menu: bool,
// }

pub enum AV {
    WaveForm,
    Sound,
    SoundInstance,
    Sequence,
}

pub struct SetupSettings {
    pub particles: usize,
    pub workgroups: usize,
    pub workgroup_size: usize,
    pub max_radius: f32,
    pub min_radius: f32,
    pub variable_rad: bool,
    pub holeyness: f32,
    pub max_bonds: usize,
    pub max_contacts: usize,
    pub max_h_velocity: f32,
    pub min_h_velocity: f32,
    pub max_v_velocity: f32,
    pub min_v_velocity: f32,
    pub structure: Structure,
    pub grid_width: f32,
    pub hex_grid: bool,
}

pub struct SimulationSettings {
    pub timestep: f32,
    pub auto_timestep: bool,
    pub round_timestep: bool,
    pub gen_per_frame: i32,
    pub max_gen_per_frame: i32,
    pub auto_width: bool,
    pub walls: bool,
    pub hor_bound: f32,
    pub vert_bound: f32,
    pub maintain_ar: bool,
    pub round_walls: bool,
    pub wall_radius: f32,
    pub use_f64: bool,
    pub d3: bool,
    pub advance_x_timesteps: bool,
    pub x_timesteps: i32,
    pub deterministic: bool,
}

pub struct PhysicsSettings {
    pub gravity: bool,
    pub gravity_acceleration: f32,
    pub planet_mode: bool,
    pub mouse_gravity: bool,
    pub collisions: bool,
    pub collision_interval: i32,
    pub friction_coefficient: f32,
    pub bonds: i32,
    pub bondenum: BondType,
    pub bond_tearing: bool,
    pub bond_normal_stiffness: f32,
    pub bond_shear_stiffness: f32,
    pub bond_normal_strength: f32,
    pub bond_shear_strength: f32,
    pub contact_damping: f32,
    pub bond_damping: f32,
    pub drag: f32,
    pub moment_contribution_factor: f32,
    pub local_damping: bool,
    pub local_damping_alpha: f32,
}

pub struct CreateSettings {
    pub create_mode: bool,
    pub current_particle: usize,
    pub particle_defs: Vec<Particle_Definition>,
    pub quantity: u32,
    pub new_preview: bool,
    pub p_def_menu: bool,
}

pub struct Settings {
    pub view: ViewSettings,
    pub setup: SetupSettings,
    pub simulation: SimulationSettings,
    pub physics: PhysicsSettings,
    pub create: CreateSettings,
    pub changed_collision_settings: bool,
    pub materials: Vec<f32>,
    pub material_size: usize,
    pub materials_changed: bool,
    pub current_file: std::path::PathBuf,
    pub current_dir: std::path::PathBuf,
    pub load: bool,
    pub save: bool,
    pub regen_bonds: bool,
    pub properties: Properties,
    pub set_properties: bool,
    pub data: Data,
    pub gather_data: bool,
    pub auto_size_plot: bool,
    pub plotted_prop: Property,
    pub hz: f32,
    pub fps: f32,
    pub timed_recording: bool,
    pub recording_duration: f32,
    pub start_time: f32,
    pub sim_time: f32,
    pub recording: bool,
    pub wall_friction: f32,
    pub backup: bool,
    pub restore: bool,
    pub reset: bool,
    pub zoom_in: bool,
    pub zoom_out: bool,
    pub home: bool,
    pub simulating: bool,
    pub select_all: bool,
    pub fix: bool,
    pub drop: bool,
    pub speed_perc: f32,
    pub f64_support: bool,
    pub rebuild_shaders: bool,
    pub current_script: usize,
    pub just_set_line: bool,
    pub world_pos: (f32, f32),
    pub curr_shader: usize,
    pub json_scripts: bool,
    pub export_screenshot: bool,
    pub export_param_indices: (usize, usize),
    pub update_critical_timestep: bool,
    pub update_contacts: bool,
    pub contact_search_optimization: bool,
    // pub groups: i32,
    // pub set_group: i32, // pub paths: ReadDir,
}

impl Settings {
    pub fn new(canvas: &Canvas) -> Self {
        let particles = 25600;
        let workgroup_size = 256;
        //particle settings
        let max_radius = 0.025;
        let holeyness = 1.3;
        let max_bonds = 6;
        let vert_bound = 10.0;
        let hor_bound = vert_bound * 1.333;
        let materials = vec![1.0, 1.0, 1.0, 0.01, 100.0, 50.0, 1.0, 0.0, 0.0, 0.01, 100.0, 50.0];
        let mut settings = Settings {
            view: ViewSettings {
                settings_menu: true,
                scale: 2.0 / vert_bound,
                rendering: true,
                circular_particles: true,
                render_rot: false,
                render_bonds: false,
                render_outline: true,
                render_bp_grid: false,
                color_code_rot: false,
                use_particle_color_outline: true,
                outline_color: [0.0, 0.0, 0.0],
                background_color: [0.0, 0.0, 0.0],
                color_source: ColorSource::Direction,
                dim_slow_particles: true,
                max_brightness_vel: 1.0,
                crt_res: 1,
                grain: false,
                grain_strength: 0.005,
                grain_size: 1,
                sobel: false,
                colored_sobel: false,
                invert: false,
                chrom_ab: false,
                abb_strength: 0.005,
                bond_highlight_strength: 5.0,
                render_unbonded_contacts: false,
                lighting: false,
                show_hit_tex: false,
                data_menu: false,
                script_menu: false,
                code_editor: false,
                // audio: AudioView {
                //     menu: false,
                //     left_tab: 0,
                //     current_sound: -1,
                //     current_source: -1,
                //     waveform: false,
                //     oscilloscope: false,
                //     osc_life: 1.0 / 100.0,
                //     timeline_menu: false,
                // },
            },
            setup: SetupSettings {
                particles,
                workgroups: (particles as f32 / workgroup_size as f32).ceil() as usize,
                workgroup_size,
                max_radius,
                min_radius: max_radius / holeyness,
                variable_rad: true,
                holeyness,
                max_bonds,
                max_contacts: max_bonds + 8,
                max_h_velocity: 0.0,
                min_h_velocity: 0.0,
                max_v_velocity: 0.0,
                min_v_velocity: 0.0,
                structure: Structure::Grid,
                grid_width: 160.0,
                hex_grid: false,
            },
            simulation: SimulationSettings {
                timestep: 1.0 / 3240.0,
                auto_timestep: false,
                round_timestep: true,
                gen_per_frame: 27,
                max_gen_per_frame: 27,
                auto_width: true,
                walls: true,
                hor_bound,
                vert_bound,
                maintain_ar: true,
                round_walls: false,
                wall_radius: 1.0,
                use_f64: false,
                d3: false,
                advance_x_timesteps: false,
                x_timesteps: 1,
                deterministic: true,
            },
            physics: PhysicsSettings {
                gravity: true,
                gravity_acceleration: 1.0,
                planet_mode: true,
                mouse_gravity: false,
                collisions: true,
                collision_interval: 1,
                friction_coefficient: 0.5,
                bonds: 0,
                bondenum: BondType::Unbonded,
                bond_tearing: false,
                bond_normal_stiffness: 10.0,
                bond_shear_stiffness: 10.0,
                bond_normal_strength: 0.5,
                bond_shear_strength: 0.5,
                contact_damping: 0.2,
                bond_damping: 0.2,
                drag: 1.0,
                moment_contribution_factor: 1.0,
                local_damping: false,
                local_damping_alpha: 0.1,
            },
            create: CreateSettings {
                create_mode: false,
                current_particle: 0,
                particle_defs: vec![Particle_Definition::default(); 1],
                quantity: 1,
                new_preview: true,
                p_def_menu: false,
            },
            changed_collision_settings: false,
            materials,
            material_size: 6,
            materials_changed: false,
            current_file: std::path::PathBuf::new(),
            current_dir: std::path::PathBuf::new(),
            load: false,
            save: false,
            regen_bonds: false,
            properties: Properties {
                set_x_force: false,
                set_y_force: false,
                set_rot_force: false,
                set_material: false,
                set_x_fixity: false,
                set_y_fixity: false,
                set_rot_fixity: false,
                set_x_pos: false,
                set_y_pos: false,
                set_rot: false,
                set_x_vel: false,
                set_y_vel: false,
                set_rot_vel: false,
                set_radius: false,
                x_force: 0.0,
                y_force: 0.0,
                rot_force: 0.0,
                material: 0,
                x_fixity: false,
                y_fixity: false,
                rot_fixity: false,
                radius: 0.0,
                x_pos: 0.0,
                y_pos: 0.0,
                rot: 0.0,
                x_vel: 0.0,
                y_vel: 0.0,
                rot_vel: 0.0,
            },
            set_properties: false,
            data: Data::new(),
            gather_data: false,
            auto_size_plot: true,
            plotted_prop: Property::Y_Position,
            hz: 120.0,
            fps: 120.0,
            timed_recording: false,
            recording_duration: 0.0025,
            start_time: 0.0,
            sim_time: 0.0,
            recording: false,
            wall_friction: 0.0,
            backup: false,
            restore: false,
            reset: false,
            zoom_in: false,
            zoom_out: false,
            home: false,
            simulating: true,
            select_all: false,
            fix: false,
            drop: false,
            speed_perc: 100.0,
            f64_support: false,
            rebuild_shaders: false,
            current_script: 0,
            just_set_line: false,
            world_pos: (0.0, 0.0),
            curr_shader: 0,
            json_scripts: false,
            export_screenshot: false,
            export_param_indices: (0, 0),
            update_critical_timestep: true,
            update_contacts: false,
            contact_search_optimization: false,
        };
        settings.load_memory(false);
        return settings;
    }

    pub fn toggle_create(&mut self) {
        self.create.create_mode = !self.create.create_mode;
    }

    pub fn set_particles(&mut self, particles: usize) {
        self.setup.particles = particles;
        self.setup.workgroups = (self.setup.particles as f32 / self.setup.workgroup_size as f32).ceil() as usize;
    }

    pub fn update_world_pos(&mut self, world_pos: (f32, f32), ui_off: (f32, f32)) {
        self.world_pos = (world_pos.0 - ui_off.0, world_pos.1 - ui_off.1);
        if self.physics.planet_mode {
            self.changed_collision_settings = true;
        }
    }

    fn edit_menu(&mut self, ctx: &Context, ui: &mut Ui) {
        let create_shortcut = egui::KeyboardShortcut::new(Modifiers::CTRL, egui::Key::C);

        ui.menu_button("Edit", |ui| {
            ui.style_mut().wrap = Some(false);

            let min_x = 80.0;
            let min_y = 0.0;
            if ui.button("Particle Definitions").clicked() {
                self.create.p_def_menu = !self.create.p_def_menu;
            }
            ui.menu_button("Create Menu", |ui| {});
        });
    }

    pub fn grid_info(&mut self) -> (usize, f32, i32, i32, i32) {
        let width = self.simulation.hor_bound * 2.0;
        let height = self.simulation.vert_bound * 2.0;
        let max_rad = self.setup.max_radius * 2.0;
        let mut min_rad = self.setup.min_radius;
        if !self.setup.variable_rad {
            min_rad = self.setup.max_radius;
        }
        let w = (width / max_rad).ceil() as i32;
        let h = (height / max_rad).ceil() as i32;
        let cell_cap = ((max_rad / min_rad + 1.0).powf(2.0).ceil() as i32).min(self.setup.particles as i32) + 2;
        let total_size = w * h * cell_cap;
        if false {
            println!("Cell Capacity:   {}", cell_cap);
            println!("Cell Dimensions: {} x {}", w, h);
            println!("Total Cells:     {}", w * h);
            println!("Total Capacity:  {}", total_size);
            println!("Bytes:           {}", total_size * 4);
        }

        return ((w * h) as usize, max_rad, cell_cap, w, h);
    }

    // fn action_parameter_ui

    pub fn updateBonds(&mut self) {
        self.physics.bonds = match self.physics.bondenum {
            BondType::Unbonded => 0,
            BondType::Normal_Bonds => 1,
            BondType::Linear_Contact_Bond => 2,
            BondType::Parallel_Linear_Contact_Bond => 3,
        }
    }

    pub fn load(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let path = FileDialog::new().add_filter("Binary File", &["bin"]).set_directory(".").pick_file();

            match path {
                Some(path) => {
                    self.current_file = path.clone();
                    self.load = true;
                }
                None => {}
            };
        }
    }

    pub fn save(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let path = FileDialog::new().add_filter("Binary File", &["bin"]).pick_file();

            match path {
                Some(path) => {
                    self.current_file = path.clone();
                    // let mut ancestors = path.ancestors();
                    // println!("{}", ancestors.next().unwrap().to_str().unwrap());
                    // self.current_dir = std::path::PathBuf::from_str(ancestors.next().unwrap().to_str().unwrap()).unwrap();
                    self.save = true;
                }
                None => {}
            };
        }
    }

    pub fn save_data(&mut self, path_param: Option<PathBuf>) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let path = match path_param {
                Some(p) => Some(p),
                None => FileDialog::new().add_filter("CSV File", &["csv"]).set_directory(".").pick_file(),
            };

            if let Some(path) = path {
                let file_path = Path::new(&path);
                let mut file = File::create(file_path).expect("Unable to create file");

                // Write the header
                writeln!(file, "Timestamp,X Position,Y Position,X Velocity,Y Velocity,Rotation,Rotation Velocity,Data1,Data2,Data3,Data4,FPS").expect("Unable to write header");

                // Write the data rows
                for i in 0..self.data.x_pos_data.len() {
                    let timestamp = self.data.x_pos_data[i][0];
                    let x_pos = self.data.x_pos_data[i][1];
                    let y_pos = self.data.y_pos_data[i][1];
                    let x_vel = self.data.x_vel_data[i][1];
                    let y_vel = self.data.y_vel_data[i][1];
                    let rot = self.data.rot_data[i][1];
                    let rot_vel = self.data.rot_vel_data[i][1];
                    let data1 = self.data.data1[i][1];
                    let data2 = self.data.data2[i][1];
                    let data3 = self.data.data3[i][1];
                    let data4 = self.data.data4[i][1];
                    let bonds_torn = self.data.torn_bonds[i][1];
                    let fps = self.data.fps[i][1];

                    writeln!(
                        file,
                        "{},{},{},{},{},{},{},{},{},{},{},{},{}",
                        timestamp, x_pos, y_pos, x_vel, y_vel, rot, rot_vel, data1, data2, data3, data4, bonds_torn, fps
                    )
                    .expect("Unable to write data row");
                }

                println!("{} ticks of data saved to: {:?}", self.data.x_pos_data.len(), file_path);
            }
        }
    }

    pub fn update_memory(&mut self) {
        let memory = Memory {
            current_dir: self.current_dir.clone(),
        };

        let json_string = serde_json::to_string(&memory).unwrap();

        match fs::write("memory.json", json_string) {
            Ok(_) => {}
            Err(_) => {
                println!("Err: Failed to update memory.");
            }
        }
    }

    pub fn load_memory(&mut self, recursively_called: bool) {
        match fs::read_to_string("memory.json") {
            Ok(json_string) => match serde_json::from_str::<Memory>(&json_string) {
                Ok(loaded_memory) => {
                    self.current_dir = loaded_memory.current_dir;
                    println!("Memory loaded successfully.");
                }
                Err(_) => {
                    println!("Err: Failed to deserialize memory.");
                }
            },
            Err(_) => {
                println!("Err: Failed to read memory file. Creating new memory file.");
                let json_string = serde_json::to_string(&Memory::default()).unwrap();

                match fs::write("memory.json", json_string) {
                    Ok(_) => {}
                    Err(_) => {
                        println!("Err: Failed to create memory file.");
                    }
                }
                if !recursively_called {
                    self.load_memory(true);
                }
            }
        }
    }

    pub fn collision_settings(&mut self) -> Vec<f32> {
        self.changed_collision_settings = false;
        // self.update_critical_timestep = true;
        return vec![
            bytemuck::cast(self.simulation.walls as i32),
            self.simulation.hor_bound,
            self.simulation.vert_bound,
            bytemuck::cast(self.simulation.round_walls as i32),
            self.simulation.wall_radius,
            self.wall_friction,
            bytemuck::cast(self.physics.gravity as i32),
            bytemuck::cast(self.physics.planet_mode as i32),
            bytemuck::cast(self.physics.bonds),
            bytemuck::cast(self.physics.collisions as i32),
            self.physics.friction_coefficient,
            self.physics.gravity_acceleration,
            self.physics.bond_normal_stiffness,
            bytemuck::cast(self.physics.bond_tearing as i32),
            self.physics.bond_normal_strength,
            self.physics.contact_damping,
            self.physics.bond_damping,
            self.physics.drag,
            self.physics.bond_shear_strength,
            self.simulation.timestep,
            self.physics.bond_shear_stiffness,
            self.world_pos.0,
            self.world_pos.1,
            bytemuck::cast(self.physics.mouse_gravity as i32),
            self.physics.moment_contribution_factor,
            bytemuck::cast(self.physics.collision_interval as i32),
            bytemuck::cast(self.physics.local_damping as i32),
            self.physics.local_damping_alpha,
            bytemuck::cast(self.setup.particles as i32),
            bytemuck::cast(self.update_contacts as i32),
        ];
    }

    pub fn render_settings(&mut self) -> Vec<i32> {
        return vec![
            self.view.circular_particles as i32,
            self.view.render_rot as i32,
            self.view.color_code_rot as i32,
            self.view.color_source.as_i32(),
            (self.view.render_bonds) as i32,
            self.simulation.walls as i32,
            self.simulation.hor_bound.to_bits() as i32,
            self.simulation.vert_bound.to_bits() as i32,
            self.physics.bond_normal_stiffness.to_bits() as i32,
            self.view.render_bp_grid as i32,
            self.simulation.round_walls as i32,
            self.simulation.wall_radius.to_bits() as i32,
            self.view.render_outline as i32,
            self.view.use_particle_color_outline as i32,
            self.view.background_color[0].to_bits() as i32,
            self.view.background_color[1].to_bits() as i32,
            self.view.background_color[2].to_bits() as i32,
            self.view.outline_color[0].to_bits() as i32,
            self.view.outline_color[1].to_bits() as i32,
            self.view.outline_color[2].to_bits() as i32,
            self.view.dim_slow_particles as i32,
            self.view.max_brightness_vel.to_bits() as i32,
            self.view.crt_res,
            self.view.grain as i32,
            self.view.grain_strength.to_bits() as i32,
            self.view.grain_size,
            self.view.sobel as i32 + self.view.colored_sobel as i32 * 2,
            self.view.invert as i32,
            self.view.chrom_ab as i32,
            self.view.abb_strength.to_bits() as i32,
            self.view.bond_highlight_strength.to_bits() as i32,
            self.view.render_unbonded_contacts as i32,
        ];
    }

    pub fn properties(&mut self) -> Vec<f32> {
        return vec![
            bytemuck::cast(self.properties.set_x_pos as i32),
            bytemuck::cast(self.properties.set_y_pos as i32),
            bytemuck::cast(self.properties.set_rot as i32),
            bytemuck::cast(self.properties.set_x_vel as i32),
            bytemuck::cast(self.properties.set_y_vel as i32),
            bytemuck::cast(self.properties.set_rot_vel as i32),
            bytemuck::cast(self.properties.set_x_force as i32),
            bytemuck::cast(self.properties.set_y_force as i32),
            bytemuck::cast(self.properties.set_rot_force as i32),
            bytemuck::cast(self.properties.set_radius as i32),
            bytemuck::cast(self.properties.set_x_fixity as i32),
            bytemuck::cast(self.properties.set_y_fixity as i32),
            bytemuck::cast(self.properties.set_rot_fixity as i32),
            bytemuck::cast(self.properties.set_material as i32),
            self.properties.x_pos,
            self.properties.y_pos,
            self.properties.rot,
            self.properties.x_vel,
            self.properties.y_vel,
            self.properties.rot_vel,
            self.properties.x_force,
            self.properties.y_force,
            self.properties.rot_force,
            self.properties.radius,
            bytemuck::cast(self.properties.x_fixity as i32),
            bytemuck::cast(self.properties.y_fixity as i32),
            bytemuck::cast(self.properties.rot_fixity as i32),
            bytemuck::cast(self.properties.material as i32),
        ];
    }

    pub fn set_timestep(&mut self, critical_timestep: f32) {
        self.simulation.timestep = critical_timestep;
        if self.simulation.round_timestep {
            self.simulation.timestep = 1.0 / (((1.0 / self.simulation.timestep as f32) / 120.0).ceil() * 120.0);
        }
        self.changed_collision_settings = true;
    }

    // fn audio_menu(&mut self, ctx: &Context, ac: &mut AudioController) {
    //     if self.view.audio.menu {
    //         egui::Window::new("Audio").collapsible(false).resizable(true).show(ctx, |ui| {
    //             let sound_count = ac.sounds.lock().unwrap().len();
    //             ui.horizontal(|ui| {
    //                 ui.group(|ui| {
    //                     ui.vertical(|ui| {
    //                         ui.set_width(100.0);
    //                         ui.horizontal(|ui| {
    //                             if ui.selectable_label(self.view.audio.left_tab == 0, "Sounds").clicked() {
    //                                 self.view.audio.left_tab = 0;
    //                             }
    //                             if ui.selectable_label(self.view.audio.left_tab == 1, "Sequences").clicked() {
    //                                 self.view.audio.left_tab = 1;
    //                             }
    //                         });
    //                         ui.separator();
    //                         if self.view.audio.left_tab == 0 {
    //                             for sid in 0..sound_count {
    //                                 ui.horizontal(|ui| {
    //                                     let mut sounds = ac.sounds.lock().unwrap();
    //                                     if ui
    //                                         .add_sized(
    //                                             ui.available_size(),
    //                                             egui::SelectableLabel::new(self.view.audio.current_sound == sid as i32, format!("{}", sounds[sid].name)),
    //                                         )
    //                                         .clicked()
    //                                     {
    //                                         self.view.audio.current_sound = sid as i32;
    //                                         self.view.audio.current_source = -1;
    //                                     }
    //                                     mem::drop(sounds);
    //                                 });
    //                             }
    //                             if ui.add_sized(egui::Vec2::new(ui.available_width(), 0.0), egui::Button::new("+")).clicked() {
    //                                 ac.new_sound();
    //                             }
    //                         }
    //                     });
    //                 });
    //                 if self.view.audio.left_tab == 0 && self.view.audio.current_sound != -1 {
    //                     ui.group(|ui| {
    //                         ui.vertical(|ui| {
    //                             ui.set_width(130.0);
    //                             ui.horizontal(|ui| {
    //                                 let mut sounds = ac.sounds.lock().unwrap();
    //                                 let cs = self.view.audio.current_sound as usize;
    //                                 ui.text_edit_singleline(sounds[cs].name_mut());
    //                                 mem::drop(sounds);
    //                                 if ui.button("Play").clicked() {
    //                                     ac.play(cs);
    //                                 }
    //                             });
    //                             ui.horizontal(|ui| {
    //                                 let mut sounds = ac.sounds.lock().unwrap();
    //                                 let cs = self.view.audio.current_sound as usize;

    //                                 ui.label("Duration: ");

    //                                 ui.add_enabled(
    //                                     !sounds[cs].auto_duration,
    //                                     egui::DragValue::new(&mut sounds[cs].duration).clamp_range(1..=u64::MAX).speed(1).suffix("ms"),
    //                                 );
    //                                 ui.checkbox(&mut sounds[cs].auto_duration, "Auto");
    //                             });
    //                             ui.separator();

    //                             let mut sounds = ac.sounds.lock().unwrap();
    //                             let cs = self.view.audio.current_sound as usize;
    //                             let source_count = sounds[cs].sources.len();
    //                             for i in 0..source_count {
    //                                 let source_type = sounds[cs].sources[i].as_type_string();
    //                                 if ui
    //                                     .add_sized(
    //                                         egui::Vec2::new(ui.available_width(), 0.0),
    //                                         egui::SelectableLabel::new(self.view.audio.current_source == i as i32, format!("{}: {}", i, source_type)),
    //                                     )
    //                                     .clicked()
    //                                 {
    //                                     self.view.audio.current_source = i as i32;
    //                                 }
    //                             }
    //                             if ui.add_sized(egui::Vec2::new(ui.available_width(), 0.0), egui::Button::new("+")).clicked() {
    //                                 sounds[cs].new_source();
    //                             }
    //                             mem::drop(sounds);
    //                         });
    //                     });
    //                 }
    //                 if self.view.audio.left_tab == 0 && self.view.audio.current_sound != -1 && self.view.audio.current_source != -1 {
    //                     ui.group(|ui| {
    //                         ui.vertical(|ui| {
    //                             let mut sounds = ac.sounds.lock().unwrap();
    //                             sounds[self.view.audio.current_sound as usize].ui(ui, self.view.audio.current_source as i32, 0);
    //                         });
    //                     });
    //                 }
    //             });
    //         });
    //     }
    // }

    // fn timeline_menu(&mut self, ctx: &Context, ac: &mut AudioController) {
    //     if self.view.audio.timeline_menu {
    //         egui::Window::new("Timeline").collapsible(false).resizable(true).show(ctx, |ui| {
    //             let mut timeline = Timeline::new();
    //             timeline.regions.push((0.2, 0.4)); // Example region
    //             timeline.regions.push((0.5, 0.7)); // Another region
    //             timeline.playhead_position = 0.3; // Example playhead position

    //             ui.add(&mut timeline);
    //         });
    //     }
    // }

    // fn waveform_menu(&mut self, ctx: &Context, ac: &mut AudioController) {
    //     if self.view.audio.waveform {
    //         egui::Window::new("Waveform").collapsible(false).resizable(true).show(ctx, |ui| {
    //             ui.vertical(|ui| {
    //                 let ar = ac.ar.lock().unwrap();
    //                 let record = ar.get_record();
    //                 if self.view.audio.oscilloscope {
    //                     let mut oss_line_vec = vec![];
    //                     let life = self.view.audio.osc_life;
    //                     let record_duration = 1.0;
    //                     let start_index = record.0.len() - (record.0.len() as f32 * life) as usize;
    //                     for i in start_index..record.0.len() {
    //                         oss_line_vec.push([record.0[i] as f64, record.1[i] as f64]);
    //                     }
    //                     let oss_values = egui::plot::PlotPoints::from((oss_line_vec.to_owned()));
    //                     let oss_line = egui::plot::Line::new(oss_values);
    //                     let plot_oss = egui::plot::Plot::new("left channel plot")
    //                         .auto_bounds_x()
    //                         .auto_bounds_y()
    //                         .clamp_grid(true)
    //                         .height(800.0)
    //                         .width(800.0)
    //                         .allow_drag(false)
    //                         .show_axes([false, false])
    //                         .show(ui, |plot_ui| {
    //                             plot_ui.line(oss_line);
    //                         });
    //                 } else {
    //                     let l_values = egui::plot::PlotPoints::from_ys_f32(&record.0);
    //                     let l_line = egui::plot::Line::new(l_values);
    //                     let r_values = egui::plot::PlotPoints::from_ys_f32(&record.1);
    //                     let r_line = egui::plot::Line::new(r_values);
    //                     let plot_l = egui::plot::Plot::new("left channel plot")
    //                         .auto_bounds_x()
    //                         .auto_bounds_y()
    //                         .clamp_grid(true)
    //                         .height(300.0)
    //                         .allow_drag(false)
    //                         .show_axes([false, false])
    //                         .show(ui, |plot_ui| {
    //                             plot_ui.line(l_line);
    //                         });

    //                     let plot_r = egui::plot::Plot::new("right channel plot")
    //                         .auto_bounds_x()
    //                         .auto_bounds_y()
    //                         .clamp_grid(true)
    //                         .height(300.0)
    //                         .allow_drag(false)
    //                         .show_axes([false, false])
    //                         .show(ui, |plot_ui| {
    //                             plot_ui.line(r_line);
    //                         });
    //                 }
    //                 ui.horizontal(|ui| {
    //                     let mut selected_text = "Wave";
    //                     if self.view.audio.oscilloscope {
    //                         selected_text = "Oscilloscope";
    //                     }
    //                     egui::ComboBox::new("Waveform View", "").selected_text(format!("{}", selected_text)).show_ui(ui, |ui| {
    //                         ui.selectable_value(&mut self.view.audio.oscilloscope, false, "Wave");
    //                         ui.selectable_value(&mut self.view.audio.oscilloscope, true, "Oscilloscope");
    //                     });

    //                     if self.view.audio.oscilloscope {
    //                         ui.add(
    //                             egui::DragValue::new(&mut self.view.audio.osc_life)
    //                                 .clamp_range(0.001..=1.0)
    //                                 .prefix("Fade Time: ")
    //                                 .suffix(" s")
    //                                 .speed(0.001),
    //                         );
    //                     }
    //                 });
    //             });
    //         });
    //     }
    // }
}

#[derive(Debug, PartialEq)]
pub enum Structure {
    Grid,
    Random,
    Exp1,
    Exp2,
    Exp3,
    Exp4,
    Exp5,
    Exp6,
    Mats,
}

#[derive(Debug, PartialEq)]
pub enum BondType {
    Unbonded,
    Normal_Bonds,
    Linear_Contact_Bond,
    Parallel_Linear_Contact_Bond,
}

impl BondType {
    pub fn from_i32(num: i32) -> Self {
        return match num {
            i32::MIN..=0 => BondType::Unbonded,
            1 => BondType::Normal_Bonds,
            2 => BondType::Linear_Contact_Bond,
            3.. => BondType::Parallel_Linear_Contact_Bond,
        };
    }
    pub fn as_i32(&self) -> i32 {
        return match *self {
            BondType::Unbonded => 0,
            BondType::Normal_Bonds => 1,
            BondType::Linear_Contact_Bond => 2,
            BondType::Parallel_Linear_Contact_Bond => 3,
        };
    }
    pub fn to_string(&self) -> &str {
        return match *self {
            BondType::Unbonded => &"Unbonded",
            BondType::Normal_Bonds => &"Normal Bonds",
            BondType::Linear_Contact_Bond => &"Linear Contact Bonds",
            BondType::Parallel_Linear_Contact_Bond => &"Linear Parallel Bonds",
        };
    }
}

#[derive(Debug, PartialEq)]
pub enum Property {
    X_Position,
    Y_Position,
    X_Velocity,
    Y_Velocity,
    Rotation,
    Rotational_Velocity,
    Normal_Force,
    Shear_Force,
    Moment,
    FPS,
    Torn_Bonds,
}

#[derive(Serialize, Deserialize)]
struct Memory {
    pub current_dir: std::path::PathBuf,
}

impl Default for Memory {
    fn default() -> Self {
        return Memory {
            current_dir: env::current_dir().unwrap().join("saved_states"),
        };
    }
}

#[derive(Debug, PartialEq)]
pub enum ColorSource {
    None,
    Material,
    Random,
    Direction,
}

impl ColorSource {
    pub fn from_i32(num: i32) -> Self {
        return match num {
            i32::MIN..=0 => ColorSource::None,
            1 => ColorSource::Material,
            2 => ColorSource::Random,
            3.. => ColorSource::Direction,
        };
    }
    pub fn as_i32(&self) -> i32 {
        return match *self {
            ColorSource::None => 0,
            ColorSource::Material => 1,
            ColorSource::Random => 2,
            ColorSource::Direction => 3,
        };
    }
    pub fn to_string(&self) -> &str {
        return match *self {
            ColorSource::None => &"None",
            ColorSource::Material => &"Material",
            ColorSource::Random => &"Random",
            ColorSource::Direction => &"Direction",
        };
    }
}
