use crate::particle_def::Particle_Definition;
use crate::scripts::{self, Key, ScriptManager};
use crate::wgpu_config::WGPUConfig;
use crate::wgpu_prog::WGPUProg;
use egui::color_picker::Alpha;
use egui::*;
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

use native_dialog::{FileDialog, MessageDialog, MessageType};

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
        };
    }

    pub fn push(&mut self, timestamp: f64, datum: [f64; 10], fps: f64) {
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
    pub groups: i32,
    pub set_group: i32
    // pub paths: ReadDir,
}

impl Settings {
    pub fn new(canvas: &Canvas) -> Self {
        let particles = 256;
        let workgroup_size = 256;
        //particle settings
        let max_radius = 0.025;
        let holeyness = 1.7;
        let max_bonds = 6;
        let vert_bound = 2.0;
        let hor_bound = vert_bound * 1.333;
        let materials = vec![1.0, 1.0, 1.0, 0.01, 100.0, 50.0, 1.0, 0.0, 0.0, 0.01, 100.0, 50.0];
        let mut settings = Settings {
            view: ViewSettings {
                settings_menu: true,
                scale: 2.0 / vert_bound,
                rendering: true,
                circular_particles: true,
                render_rot: false,
                render_bonds: true,
                render_outline: true,
                render_bp_grid: false,
                color_code_rot: false,
                use_particle_color_outline: true,
                outline_color: [0.0, 0.0, 0.0],
                background_color: [0.0, 0.0, 0.0],
                color_source: ColorSource::Material,
                dim_slow_particles: false,
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
            },
            setup: SetupSettings {
                particles,
                workgroups: (particles as f32 / workgroup_size as f32).ceil() as usize,
                workgroup_size,
                max_radius,
                min_radius: max_radius / holeyness,
                variable_rad: false,
                holeyness,
                max_bonds,
                max_contacts: max_bonds + 8,
                max_h_velocity: 0.0,
                min_h_velocity: 0.0,
                max_v_velocity: 0.0,
                min_v_velocity: 0.0,
                structure: Structure::Grid,
                grid_width: 32.0,
                hex_grid: false,
            },
            simulation: SimulationSettings {
                timestep: 1.0 / 12600.0,
                round_timestep: true,
                gen_per_frame: 105,
                max_gen_per_frame: 213,
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
            simulating: false,
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
        };
        settings.load_memory();
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

    pub fn ui(&mut self, ctx: &Context, prog: &mut WGPUProg, script_manager: &mut ScriptManager, config: &mut WGPUConfig, window_size: (u32, u32)) -> bool {
        let mut reset = false;
        if !self.current_file.exists() && self.save {
            self.save();
        }
        if self.recording && self.start_time + self.recording_duration < self.sim_time {
            self.gather_data = false;
            self.recording = false;
        }
        if self.view.settings_menu {
            egui::TopBottomPanel::top("Settings Menu").show(ctx, |ui| {
                // ui.heading("Menu");
                egui::menu::bar(ui, |ui| {
                    ui.horizontal_centered(|ui| {
                        self.file_menu(ui);
                        self.edit_menu(ctx, ui);
                        self.view_menu(ui);
                        self.state_menu(ui);
                        self.sim_controls_menu(ui);
                        self.physics_menu(ui);
                        self.particle_menu(ui);
                        self.materials_menu(ui);
                        self.data_menu(ui, ctx);
                        self.script_menu(ui, ctx);
                        self.developer_menu(ui, ctx);
                    });
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        let max_perc = self.simulation.gen_per_frame as f32 / self.simulation.max_gen_per_frame as f32 * 100.0;
                        let mut fps_perc = max_perc * self.fps / self.hz;
                        if !self.simulating {
                            fps_perc = 0.0;
                        }
                        ui.add(egui::Label::new(format!("{:.0}/{:.0}%", fps_perc, max_perc))).on_hover_text("Actual/Target simulation speed.");
                    });
                });
            });
            if self.create.p_def_menu {
                self.p_def_menu(ctx);
            }
            self.script_panel(ctx, script_manager, prog, &mut config.device, &mut config.queue);
            self.code_editor(ctx, prog, config);
        }
        if self.simulation.auto_width {
            self.simulation.hor_bound = self.simulation.vert_bound * ctx.available_rect().width() as f32 / ctx.available_rect().height() as f32;
            self.changed_collision_settings = true;
        }

        return reset;
    }

    fn file_menu(&mut self, ui: &mut Ui) {
        let load_shortcut = egui::KeyboardShortcut::new(Modifiers::CTRL, egui::Key::O);
        let save_shortcut = egui::KeyboardShortcut::new(Modifiers::CTRL, egui::Key::S);

        ui.menu_button("File", |ui| {
            ui.style_mut().wrap = Some(false);

            let min_x = 80.0;
            let min_y = 0.0;
            ui.menu_button("Load", |ui| {
                let mut paths = fs::read_dir(self.current_dir.clone());
                match paths {
                    Ok(_) => {
                        for path in paths.unwrap() {
                            let file = path.unwrap().path();
                            let mut extention = "";
                            match file.extension() {
                                Some(ext) => {
                                    extention = ext.to_str().unwrap();
                                }
                                None => {
                                    continue;
                                }
                            }

                            if extention.contains("bin") {
                                if ui.button(format!("{}", file.file_name().unwrap().to_str().unwrap())).clicked() {
                                    self.current_file = file;
                                    self.load = true;
                                };
                            }
                        }
                    }
                    Err(_) => {
                        ui.label("Invald Directory Path");
                    }
                }

                if ui.button("Select Folder").clicked() {
                    match FileDialog::new()
                        //.set_location(&self.current_dir)
                        .show_open_single_dir()
                        .unwrap()
                    {
                        Some(path) => {
                            self.current_dir = path.clone();
                            self.update_memory();
                        }
                        None => {}
                    };
                }
            });

            if ui
                .add(egui::Button::new("Save").min_size(Vec2::new(min_x, min_y)).shortcut_text(ui.ctx().format_shortcut(&save_shortcut)))
                .clicked()
            {
                self.save();
                ui.close_menu();
            }
        });
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

    fn p_def_menu(&mut self, ctx: &Context) {
        egui::Window::new("Particle Definitions").collapsible(false).resizable(false).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    if ui
                        .add(
                            egui::DragValue::new(&mut self.create.particle_defs[self.create.current_particle].radius)
                                .clamp_range(0.0..=10.0)
                                .prefix("Radius: ")
                                .suffix(" m")
                                .speed(0.001),
                        )
                        .changed()
                    {
                        self.create.particle_defs[self.create.current_particle].new_radius();
                        self.create.new_preview = true
                    }
                    if ui.checkbox(&mut self.create.particle_defs[self.create.current_particle].random_radius, "Random Radii").changed() {
                        self.create.particle_defs[self.create.current_particle].new_radius();
                        self.create.new_preview = true;
                    }
                    if self.create.particle_defs[self.create.current_particle].random_radius {
                        let min = self.create.particle_defs[self.create.current_particle].min_radius.clone();
                        let max = self.create.particle_defs[self.create.current_particle].max_radius.clone();
                        if ui.add(egui::Slider::new(&mut self.create.particle_defs[self.create.current_particle].min_radius, 0.0..=max)).changed() {
                            self.create.particle_defs[self.create.current_particle].new_radius();
                            self.create.new_preview = true;
                        }
                        if ui.add(egui::Slider::new(&mut self.create.particle_defs[self.create.current_particle].max_radius, min..=10.0)).changed() {
                            self.create.particle_defs[self.create.current_particle].new_radius();
                            self.create.new_preview = true;
                        }
                    }
                });
                ui.vertical(|ui| {
                    ui.label("Properties");

                    ui.horizontal(|inner_ui| {
                        inner_ui.vertical(|inner_ui2| {
                            inner_ui2.label("Velocity");
                            inner_ui2.horizontal(|inner_ui3| {
                                inner_ui3.add(
                                    egui::DragValue::new(&mut self.create.particle_defs[self.create.current_particle].x_vel)
                                        .speed(0.001)
                                        .clamp_range(f32::MIN..=f32::MAX),
                                );
                                inner_ui3.label("X Velocity");
                            });
                            inner_ui2.horizontal(|inner_ui3| {
                                inner_ui3.add(
                                    egui::DragValue::new(&mut self.create.particle_defs[self.create.current_particle].y_vel)
                                        .speed(0.001)
                                        .clamp_range(f32::MIN..=f32::MAX),
                                );
                                inner_ui3.label("Y Velocity");
                            });
                            inner_ui2.horizontal(|inner_ui3| {
                                inner_ui3.add(
                                    egui::DragValue::new(&mut self.create.particle_defs[self.create.current_particle].rot_vel)
                                        .speed(0.001)
                                        .clamp_range(f32::MIN..=f32::MAX),
                                );
                                inner_ui3.label("Rotational Velocity");
                            });
                            inner_ui2.label("Forces");
                            inner_ui2.horizontal(|inner_ui3| {
                                inner_ui3.add(egui::DragValue::new(&mut self.create.particle_defs[self.create.current_particle].x_force).speed(0.01));
                                inner_ui3.label("X Force");
                            });
                            inner_ui2.horizontal(|inner_ui3| {
                                inner_ui3.add(egui::DragValue::new(&mut self.create.particle_defs[self.create.current_particle].y_force).speed(0.01));
                                inner_ui3.label("Y Force");
                            });
                            inner_ui2.horizontal(|inner_ui3| {
                                inner_ui3.add(egui::DragValue::new(&mut self.create.particle_defs[self.create.current_particle].rot_force).speed(0.01));
                                inner_ui3.label("Rotational Force");
                            });
                            inner_ui2.label("Radius");
                            inner_ui2.horizontal(|inner_ui3| {
                                inner_ui3.add(
                                    egui::DragValue::new(&mut self.create.particle_defs[self.create.current_particle].radius)
                                        .speed(0.001)
                                        .clamp_range(0.0..=f32::MAX),
                                );
                            });
                            inner_ui2.label("Fixity");
                            inner_ui2.horizontal(|inner_ui3| {
                                if inner_ui3
                                    .add(egui::SelectableLabel::new(
                                        self.create.particle_defs[self.create.current_particle].x_fixity,
                                        match self.create.particle_defs[self.create.current_particle].x_fixity {
                                            true => "True",
                                            false => "False",
                                        },
                                    ))
                                    .clicked()
                                {
                                    self.create.particle_defs[self.create.current_particle].x_fixity = !self.create.particle_defs[self.create.current_particle].x_fixity;
                                }
                                inner_ui3.label("X Fixity");
                            });
                            inner_ui2.horizontal(|inner_ui3| {
                                if inner_ui3
                                    .add(egui::SelectableLabel::new(
                                        self.create.particle_defs[self.create.current_particle].y_fixity,
                                        match self.create.particle_defs[self.create.current_particle].y_fixity {
                                            true => "True",
                                            false => "False",
                                        },
                                    ))
                                    .clicked()
                                {
                                    self.create.particle_defs[self.create.current_particle].y_fixity = !self.create.particle_defs[self.create.current_particle].y_fixity;
                                };

                                inner_ui3.label("Y Fixity");
                            });
                            inner_ui2.horizontal(|inner_ui3| {
                                if inner_ui3
                                    .add(egui::SelectableLabel::new(
                                        self.create.particle_defs[self.create.current_particle].rot_fixity,
                                        match self.create.particle_defs[self.create.current_particle].rot_fixity {
                                            true => "True",
                                            false => "False",
                                        },
                                    ))
                                    .clicked()
                                {
                                    self.create.particle_defs[self.create.current_particle].rot_fixity = !self.create.particle_defs[self.create.current_particle].rot_fixity;
                                };
                                inner_ui3.label("Rotational Fixity");
                            });
                            inner_ui2.label("Material");
                            inner_ui2.horizontal(|inner_ui3| {
                                inner_ui3.add(egui::Slider::new(
                                    &mut self.create.particle_defs[self.create.current_particle].material,
                                    0..=(self.materials.len() / self.material_size - 1) as i32,
                                ));
                            });
                        });
                    });
                });
                ui.vertical_centered_justified(|ui| {
                    for (i, p_def) in self.create.particle_defs.iter().enumerate() {
                        if i == self.create.current_particle {
                            if ui.button(p_def.name.clone()).highlight().clicked() {
                                self.create.current_particle = i;
                            }
                        } else {
                            if ui.button(p_def.name.clone()).clicked() {
                                self.create.current_particle = i;
                            }
                        }
                    }
                    if ui.button("+").clicked() {
                        self.create.particle_defs.push(Particle_Definition::default());
                    }
                });
            });
        });
    }

    fn state_menu(&mut self, ui: &mut Ui) {
        let backup_shortcut = egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::B);
        let restore_shortcut = egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::R);
        let reset_shortcut = egui::KeyboardShortcut::new(Modifiers::SHIFT, egui::Key::R);

        ui.menu_button("State", |ui| {
            ui.style_mut().wrap = Some(false);

            ui.set_min_width(225.0);
            let min_x = 80.0;
            let min_y = 0.0;
            if ui
                .add(egui::Button::new("Backup").min_size(Vec2::new(min_x, min_y)).shortcut_text(ui.ctx().format_shortcut(&backup_shortcut)))
                .on_hover_text("Store current state.")
                .clicked()
            {
                self.backup = true;
                ui.close_menu();
            }

            if ui
                .add(
                    egui::Button::new("Restore")
                        .min_size(Vec2::new(min_x, min_y))
                        .shortcut_text(ui.ctx().format_shortcut(&restore_shortcut)),
                )
                .on_hover_text("Restore stored state.")
                .clicked()
            {
                self.restore = true;
                ui.close_menu();
            }

            if ui
                .add(
                    egui::Button::new("Hard Reset")
                        .min_size(Vec2::new(min_x, min_y))
                        .shortcut_text(ui.ctx().format_shortcut(&reset_shortcut)),
                )
                .on_hover_text("Rerun setup. Store generated state.")
                .clicked()
            {
                self.reset = true;
                ui.close_menu();
            }
            ui.separator();
            ui.label("Setup");
            if ui
                .add(egui::Slider::new(&mut self.setup.particles, 1..=self.setup.workgroup_size * 200).text("Particles").step_by(1.0))
                .changed()
            {
                self.setup.workgroups = (self.setup.particles as f32 / self.setup.workgroup_size as f32).ceil() as usize;
                self.setup.grid_width = self.setup.grid_width.min(self.setup.particles as f32);
                self.reset = true;
            };
            if self.setup.structure == Structure::Grid {
                self.reset |= ui
                    .add(
                        egui::Slider::new(&mut self.setup.grid_width, 1.0..=self.setup.particles as f32)
                            .text("Grid Width")
                            .step_by(0.01)
                            .logarithmic(true),
                    )
                    .changed();
                self.reset |= ui.checkbox(&mut self.setup.hex_grid, "Hex Grid").changed();
            }

            self.reset |= ui.checkbox(&mut self.setup.variable_rad, "Random Radius").changed();

            if ui.add(egui::Slider::new(&mut self.setup.max_radius, 0.000000001..=10.0).step_by(0.001).text("Max Radius")).changed() {
                self.setup.min_radius = self.setup.max_radius / self.setup.holeyness;
                self.reset = true;
            }

            if self.setup.variable_rad {
                match self.setup.structure {
                    Structure::Grid => {
                        if ui.add(egui::Slider::new(&mut self.setup.holeyness, 1.0..=10.0).text("Holeyness")).changed() {
                            self.setup.min_radius = self.setup.max_radius / self.setup.holeyness;
                            self.reset = true;
                        };
                    }
                    _ => {
                        self.reset |= ui.add(egui::Slider::new(&mut self.setup.max_radius, 0.0001..=0.5).text("Max Radius")).changed();
                        self.reset |= ui.add(egui::Slider::new(&mut self.setup.min_radius, 0.0001..=0.5).text("Min Radius")).changed();
                    }
                }
            }
            egui::CollapsingHeader::new("Initial Velocities").show(ui, |ui| {
                if ui.add(egui::Slider::new(&mut self.setup.max_h_velocity, -10.0..=10.0).text("Max xV")).changed() {
                    if self.setup.max_h_velocity < self.setup.min_h_velocity {
                        self.setup.min_h_velocity = self.setup.max_h_velocity;
                    }
                    self.reset = true;
                };
                if ui.add(egui::Slider::new(&mut self.setup.min_h_velocity, -10.0..=10.0).text("Min xV")).changed() {
                    if self.setup.max_h_velocity < self.setup.min_h_velocity {
                        self.setup.max_h_velocity = self.setup.min_h_velocity;
                    }
                    self.reset = true;
                };
                if ui.add(egui::Slider::new(&mut self.setup.max_v_velocity, -10.0..=10.0).text("Max yV")).changed() {
                    if self.setup.max_v_velocity < self.setup.min_v_velocity {
                        self.setup.min_v_velocity = self.setup.max_v_velocity;
                    }
                    self.reset = true;
                };
                if ui.add(egui::Slider::new(&mut self.setup.min_v_velocity, -10.0..=10.0).text("Min yV")).changed() {
                    if self.setup.max_v_velocity < self.setup.min_v_velocity {
                        self.setup.max_v_velocity = self.setup.min_v_velocity;
                    }
                    self.reset = true;
                };
            });
        });
    }

    fn view_menu(&mut self, ui: &mut Ui) {
        let zoom_in_shortcut = egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::PlusEquals);
        let zoom_out_shortcut = egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::Minus);
        // let pan_shortcut =
        //     egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::R);
        let home_shortcut = egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::H);

        ui.menu_button("View", |ui| {
            ui.style_mut().wrap = Some(false);
            let min_x = 100.0;
            ui.set_min_width(min_x);
            if ui
                .add(
                    egui::Button::new("Zoom In")
                        .min_size(Vec2::new(min_x, 0.0))
                        .shortcut_text(format!("{}/Wheel +", ui.ctx().format_shortcut(&zoom_in_shortcut))),
                )
                .on_hover_text("Zoom in 2x.")
                .clicked()
            {
                self.zoom_in = true;
                // ui.close_menu();
            }

            if ui
                .add(
                    egui::Button::new("Zoom Out")
                        .min_size(Vec2::new(min_x, 0.0))
                        .shortcut_text(format!("{}/Wheel -", ui.ctx().format_shortcut(&zoom_out_shortcut))),
                )
                .on_hover_text("Zoom out 2x.")
                .clicked()
            {
                self.zoom_out = true;
                // ui.close_menu();
            }

            if ui.button("Fit Bounds").clicked() {
                self.view.scale = 2.0 / self.simulation.vert_bound;
            }

            if ui
                .add(egui::Button::new("Home").min_size(Vec2::new(min_x, 0.0)).shortcut_text(ui.ctx().format_shortcut(&home_shortcut)))
                .on_hover_text("Centers the view on (0,0).")
                .clicked()
            {
                self.home = true;
                // ui.close_menu();
            }
            ui.add_enabled(false, egui::Button::new("Pan").min_size(Vec2::new(min_x, 0.0)).shortcut_text(format!("Shift + Drag")));
            ui.separator();
            ui.label("Rendering");
            // egui::Window::new("Render Settings").collapsible(false).auto_sized().show(ctx, |ui| {
            // ui.checkbox(&mut self.view.rendering, "Render Particles");
            self.rebuild_shaders |= ui.checkbox(&mut self.view.circular_particles, "Circular Particles").changed();
            ui.add_enabled(self.view.circular_particles, egui::Checkbox::new(&mut self.view.render_outline, "Render Outline"));
            self.rebuild_shaders |= ui.checkbox(&mut self.view.render_rot, "Render Rotation").changed();
            ui.checkbox(&mut self.view.render_unbonded_contacts, "Render Contacts");
            self.rebuild_shaders |= ui.checkbox(&mut self.view.render_bonds, "Render Bonds").changed();
            self.rebuild_shaders |= ui.checkbox(&mut self.view.lighting, "Lighting").changed();
            self.rebuild_shaders |= ui.checkbox(&mut self.simulation.d3, "3D").changed();
            ui.menu_button("Particle Color", |ui| {
                ui.label("Color Source:");
                ui.menu_button(format!("{}", self.view.color_source.to_string()), |ui| {
                    ui.selectable_value(&mut self.view.color_source, ColorSource::None, "None");
                    ui.selectable_value(&mut self.view.color_source, ColorSource::Material, "Material");
                    ui.selectable_value(&mut self.view.color_source, ColorSource::Direction, "Direction");
                    ui.selectable_value(&mut self.view.color_source, ColorSource::Random, "Random");
                });
                self.rebuild_shaders |= ui.checkbox(&mut self.view.color_code_rot, "Color Code Rotation").changed();
                ui.checkbox(&mut self.view.dim_slow_particles, "Dim Slow Particles");
                ui.add_enabled(
                    self.view.dim_slow_particles,
                    egui::DragValue::new(&mut self.view.max_brightness_vel)
                        .clamp_range(0.0001..=100.0)
                        .prefix("Dimming Threshold: ")
                        .suffix(" m/s")
                        .speed(0.01),
                );
            });
            ui.add_enabled_ui(self.view.render_outline && self.view.circular_particles, |ui| {
                ui.menu_button("Outline Color", |ui| {
                    ui.checkbox(&mut self.view.use_particle_color_outline, "Use Particle Color");
                    ui.add_enabled_ui(!self.view.use_particle_color_outline, |ui| {
                        let mut color = Color32::from_rgb(
                            (self.view.outline_color[0] * 255.0) as u8,
                            (self.view.outline_color[1] * 255.0) as u8,
                            (self.view.outline_color[2] * 255.0) as u8,
                        );
                        egui::color_picker::color_picker_color32(ui, &mut color, Alpha::Opaque);
                        let color_srgb = color.to_srgba_unmultiplied();
                        self.view.outline_color[0] = color_srgb[0] as f32 / 255.0;
                        self.view.outline_color[1] = color_srgb[1] as f32 / 255.0;
                        self.view.outline_color[2] = color_srgb[2] as f32 / 255.0;
                        ui.add(egui::Slider::new(&mut self.view.outline_color[0], 0.0..=1.0));
                        ui.add(egui::Slider::new(&mut self.view.outline_color[1], 0.0..=1.0));
                        ui.add(egui::Slider::new(&mut self.view.outline_color[2], 0.0..=1.0));
                    });
                });
            });
            ui.menu_button("Background Color", |ui| {
                let mut color = Color32::from_rgb(
                    (self.view.background_color[0] * 255.0) as u8,
                    (self.view.background_color[1] * 255.0) as u8,
                    (self.view.background_color[2] * 255.0) as u8,
                );
                egui::color_picker::color_picker_color32(ui, &mut color, Alpha::Opaque);
                // let color_srgb = color.to_srgba_unmultiplied();
                self.view.background_color[0] = (color[0] as f32 / 255.0);
                self.view.background_color[1] = (color[1] as f32 / 255.0);
                self.view.background_color[2] = (color[2] as f32 / 255.0);
                ui.add(egui::Slider::new(&mut self.view.background_color[0], 0.0..=1.0));
                ui.add(egui::Slider::new(&mut self.view.background_color[1], 0.0..=1.0));
                ui.add(egui::Slider::new(&mut self.view.background_color[2], 0.0..=1.0));
            });
            ui.menu_button("Post Processing", |ui| {
                // ui.label("CRT Effect");
                // ui.separator();
                ui.checkbox(&mut self.view.sobel, "Sobel Filter");
                ui.add_enabled(self.view.sobel, egui::Checkbox::new(&mut self.view.colored_sobel, "Colored Sobel"));
                ui.checkbox(&mut self.view.invert, "Invert Colors");
                // ui.horizontal(|ui|{
                //     ui.label("Render every");
                //     ui.add(DragValue::new(&mut self.view.crt_res).clamp_range(1..=16));
                //     ui.label("lines.");
                // });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.view.grain, "");
                    ui.add_enabled_ui(self.view.grain, |ui| {
                        ui.menu_button("Grain", |ui| {
                            ui.label("Size:");
                            ui.add(DragValue::new(&mut self.view.grain_size).suffix("px").clamp_range(1..=8));
                            ui.label("Strength:");
                            ui.add(DragValue::new(&mut self.view.grain_strength).clamp_range(0.0..=1.0).speed(0.001));
                        });
                    });
                });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.view.chrom_ab, "");
                    ui.add_enabled_ui(self.view.chrom_ab, |ui| {
                        ui.menu_button("Chromatic Aberation", |ui| {
                            ui.label("Offset Strength:");
                            ui.add(DragValue::new(&mut self.view.abb_strength).clamp_range(0.0..=0.25).speed(0.001));
                        });
                    });
                });
            });
            // });
        });
    }

    fn sim_controls_menu(&mut self, ui: &mut Ui) {
        let play_pause_shortcut = egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::Space);

        ui.menu_button("Simulation", |ui| {
            ui.style_mut().wrap = Some(false);
            ui.set_min_width(200.0);

            let min_x = 175.0;
            let min_y = 0.0;
            if ui
                .add(
                    egui::Button::new("Start/Stop")
                        .min_size(Vec2::new(min_x, min_y))
                        .shortcut_text(ui.ctx().format_shortcut(&play_pause_shortcut)),
                )
                .on_hover_text("Toggle simulation.")
                .clicked()
            {
                self.simulating = !self.simulating;
            }
            ui.separator();
            ui.label(format!("Speed | {} ticks/frame", self.simulation.gen_per_frame));
            // ui.menu_button("Speed", |ui| {
            let mut max_perc = self.simulation.gen_per_frame as f32 / self.simulation.max_gen_per_frame as f32 * 100.0;
            if self.speed_perc != max_perc {
                self.speed_perc = max_perc;
            }
            if ui
                .add(
                    egui::Slider::new(&mut self.speed_perc, 1.0 / self.simulation.max_gen_per_frame as f32..=100.0).custom_formatter(|n, _| {
                        let n = n as i32;
                        format!("{n}%")
                    }),
                )
                .changed()
            {
                self.simulation.gen_per_frame = 1.max((self.speed_perc / 100.0 * self.simulation.max_gen_per_frame as f32) as i32);
            }; //.logarithmic(true);//.text(format!("Ticks/Frame ({:.0}/{:.0}%)", fps_perc, max_perc)).text_color(Color32::from_rgb((255.0*(1.0 - (self.fps/self.hz).clamp(0.0, 1.0))) as u8, (255.0*(self.fps/self.hz).clamp(0.0, 1.0)) as u8, 0)));

            if ui
                .add(egui::Button::new("Speed Up").min_size(Vec2::new(min_x, 0.0)).shortcut_text("Right Arrow"))
                .on_hover_text("Increase ticks/frame.")
                .clicked()
            {
                self.simulation.gen_per_frame = self.simulation.max_gen_per_frame.min(self.simulation.gen_per_frame + 1);
            }

            if ui
                .add(egui::Button::new("Slow Down").min_size(Vec2::new(min_x, 0.0)).shortcut_text("Left Arrow"))
                .on_hover_text("Decrease ticks/frame.")
                .clicked()
            {
                self.simulation.gen_per_frame = 1.max(self.simulation.gen_per_frame - 1);
            }
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Advance").clicked() {
                    self.simulation.advance_x_timesteps = true;
                }
                ui.add(egui::DragValue::new(&mut self.simulation.x_timesteps).speed(1).clamp_range(1..=i32::MAX));
                ui.label("timesteps");
            });
            ui.separator();
            ui.label(format!("Quality | {} ticks/s", (1.0 / self.simulation.timestep).round() as i32));
            if ui.add(egui::Slider::new(&mut self.simulation.timestep, 0.0000000001..=1.0 / self.hz).logarithmic(true)).changed() {
                if self.simulation.round_timestep {
                    self.simulation.timestep = 1.0 / (((1.0 / self.simulation.timestep as f32) / 120.0).ceil() * 120.0);
                }
                self.changed_collision_settings = true;
            }
            if ui.checkbox(&mut self.simulation.round_timestep, "Round Timestep").changed() {
                if self.simulation.round_timestep {
                    self.simulation.timestep = 1.0 / (((1.0 / self.simulation.timestep as f32) / 120.0).ceil() * 120.0);
                    self.changed_collision_settings = true;
                }
            }
            ui.add_enabled_ui(self.f64_support, |ui| {
                if self.f64_support {
                    if ui
                        .checkbox(&mut self.simulation.use_f64, "64-bit precision")
                        .on_hover_text("Use f64s to calculate distance between particles.")
                        .clicked()
                    {
                        self.rebuild_shaders = true;
                    }
                } else {
                    ui.checkbox(&mut self.simulation.use_f64, "64-bit precision").on_hover_text("Not supported by your GPU.");
                }
            });
            ui.separator();
            ui.label("Walls");
            // ui.checkbox(&mut self.simulation.walls, "Walls");
            ui.add_enabled_ui(self.simulation.walls, |ui| {
                if ui.checkbox(&mut self.simulation.round_walls, "Circular Walls").changed() {
                    self.changed_collision_settings = true;
                }
                if self.simulation.round_walls {
                    if ui.add(egui::Slider::new(&mut self.simulation.wall_radius, 0.0..=64.0).text("Radius")).changed() {
                        self.changed_collision_settings = true;
                    }
                } else {
                    let ar = self.simulation.hor_bound / self.simulation.vert_bound;
                    ui.checkbox(&mut self.simulation.auto_width, "Auto Width");
                    ui.add_enabled(!self.simulation.auto_width, egui::Checkbox::new(&mut self.simulation.maintain_ar, "Maintain Aspect Ratio"));
                    if ui
                        .add_enabled(!self.simulation.auto_width, egui::Slider::new(&mut self.simulation.hor_bound, 0.0..=64.0).text("Width"))
                        .changed()
                    {
                        self.changed_collision_settings = true;
                        if self.simulation.maintain_ar || self.simulation.auto_width {
                            self.simulation.vert_bound = self.simulation.hor_bound * 1.0 / ar;
                        }
                    }
                    if ui.add(egui::Slider::new(&mut self.simulation.vert_bound, 0.0..=64.0).text("Height")).changed() {
                        self.changed_collision_settings = true;
                        if self.simulation.maintain_ar || self.simulation.auto_width {
                            self.simulation.hor_bound = self.simulation.vert_bound * ar;
                        }
                    }
                }
            });
        });
    }

    fn physics_menu(&mut self, ui: &mut Ui) {
        let min_y = 0.0;
        let mut min_x = match self.physics.bondenum {
            BondType::Unbonded => 0.0,
            BondType::Normal_Bonds => 200.0,
            BondType::Linear_Contact_Bond => 200.0,
            BondType::Parallel_Linear_Contact_Bond => 320.0,
        };
        ui.menu_button("Physics", |ui| {
            ui.style_mut().wrap = Some(false);
            ui.set_min_width(min_x);
            self.changed_collision_settings |= ui.checkbox(&mut self.physics.gravity, "Gravity").changed();
            ui.add_enabled_ui(self.physics.gravity, |ui| {
                self.changed_collision_settings |= ui.checkbox(&mut self.physics.planet_mode, "Planet Mode").changed();
                ui.add_enabled_ui(self.physics.planet_mode, |ui| {
                    self.changed_collision_settings |= ui.checkbox(&mut self.physics.mouse_gravity, "Mouse Gravity").changed();
                });
                ui.label("G Force");
                self.changed_collision_settings |= ui.add(egui::Slider::new(&mut self.physics.gravity_acceleration, -100.0..=100.0).step_by(0.1)).changed();
            });
            ui.separator();
            self.changed_collision_settings |= ui.checkbox(&mut self.physics.collisions, "Collisions").changed();
            ui.add_enabled_ui(self.physics.collisions, |ui| {
                ui.label("Collision Interval");
                self.changed_collision_settings |= ui.add(egui::Slider::new(&mut self.physics.collision_interval, 1..=self.simulation.max_gen_per_frame)).changed();
                ui.label("Friction Coefficient");
                self.changed_collision_settings |= ui.add(egui::Slider::new(&mut self.physics.friction_coefficient, 0.0..=1.0)).changed();
            });
            ui.separator();
            self.changed_collision_settings |= ui.checkbox(&mut self.physics.local_damping, "Local Damping").changed();
            ui.add_enabled_ui(self.physics.local_damping, |ui| {
                self.changed_collision_settings |= ui.add(egui::Slider::new(&mut self.physics.local_damping_alpha, 0.0..=1.0)).changed();
            });
            ui.separator();
            let mut changed_bonds = false;

            ui.menu_button(format!("{}", self.physics.bondenum.to_string()), |ui| {
                changed_bonds |= ui.selectable_value(&mut self.physics.bondenum, BondType::Unbonded, "Unbonded").changed();
                changed_bonds |= ui.selectable_value(&mut self.physics.bondenum, BondType::Normal_Bonds, "Normal Bonds").changed();
                changed_bonds |= ui.selectable_value(&mut self.physics.bondenum, BondType::Linear_Contact_Bond, "Linear Contact Bonds").changed();
                changed_bonds |= ui
                    .selectable_value(&mut self.physics.bondenum, BondType::Parallel_Linear_Contact_Bond, "Linear Parallel Bonds")
                    .changed();
            });

            if changed_bonds {
                self.changed_collision_settings = true;
                self.updateBonds();
            }

            if self.physics.bonds != 0 {
                ui.separator();
                ui.label("Stiffness");
                self.changed_collision_settings |= ui
                    .add(egui::Slider::new(&mut self.physics.bond_normal_stiffness, 0.001..=1000000000000.0).step_by(0.001).text("Normal"))
                    .changed();
                if self.physics.bonds > 1 {
                    self.changed_collision_settings |= ui
                        .add(egui::Slider::new(&mut self.physics.bond_shear_stiffness, 0.001..=1000000000000.0).step_by(0.001).text("Shear"))
                        .changed();
                }
                self.changed_collision_settings |= ui.checkbox(&mut self.physics.bond_tearing, "Bond Tearing").changed();
                ui.add_enabled_ui(self.physics.bond_tearing, |ui| {
                    ui.separator();
                    ui.label("Strength");
                    self.changed_collision_settings |= ui
                        .add(egui::Slider::new(&mut self.physics.bond_normal_strength, 0.0..=1000000000.0).step_by(0.0001).text("Normal"))
                        .changed();
                    if self.physics.bonds > 1 {
                        self.changed_collision_settings |= ui
                            .add(egui::Slider::new(&mut self.physics.bond_shear_strength, 0.0..=1000000000.0).step_by(0.0001).text("Shear"))
                            .changed();
                    }
                    if self.physics.bonds > 2 {
                        self.changed_collision_settings |= ui
                            .add(
                                egui::Slider::new(&mut self.physics.moment_contribution_factor, 0.0..=1.0)
                                    .step_by(0.0001)
                                    .text("Moment Contribution Factor"),
                            )
                            .changed();
                    }
                });
            }
            self.regen_bonds = ui.button("Regenerate Bonds").clicked();
        });
    }

    fn particle_menu(&mut self, ui: &mut Ui) {
        let select_all_shortcut = egui::KeyboardShortcut::new(Modifiers::CTRL, egui::Key::A);

        ui.menu_button("Particles", |ui| {
            ui.style_mut().wrap = Some(false);
            ui.set_min_width(180.0);

            let min_x = 80.0;
            let min_y = 0.0;
            ui.add_enabled_ui(false, |ui| {
                ui.add(egui::Button::new("Select").min_size(Vec2::new(min_x, min_y)).shortcut_text("Click"));
            });

            if ui
                .add(
                    egui::Button::new("Select All")
                        .min_size(Vec2::new(min_x, min_y))
                        .shortcut_text(ui.ctx().format_shortcut(&select_all_shortcut)),
                )
                .clicked()
            {
                self.select_all = true;
                ui.close_menu();
            }

            ui.menu_button("Groups", |ui|{
                for i in 0..self.groups {
                    ui.horizontal(|ui|{
                        ui.label(format!("Group {}", i+1));
                        if ui.button("Set").clicked() {
                            self.set_group = i;
                        }
                    });
                    // ui.selectable_label(self.set_group >= 0, text);
                }
               if ui.button("New Group").clicked(){
                    self.groups += 1;
                }
            });


            ui.add_enabled_ui(false, |ui| {
                ui.add(egui::Button::new("Translate").min_size(Vec2::new(min_x, min_y)).shortcut_text("Click + Drag"));
            });

            if ui
                .add(egui::Button::new("Fix").min_size(Vec2::new(min_x, min_y)).shortcut_text("F"))
                .on_hover_text("Fix selected particles.")
                .clicked()
            {
                self.fix = true;
                ui.close_menu();
            }

            if ui
                .add(egui::Button::new("Drop").min_size(Vec2::new(min_x, min_y)).shortcut_text("D"))
                .on_hover_text("Unfix selected particles.")
                .clicked()
            {
                self.drop = true;
                ui.close_menu();
            }

            ui.separator();
            ui.label("Properties");

            ui.horizontal(|inner_ui| {
                inner_ui.vertical(|inner_ui2| {
                    inner_ui2.label("Position");
                    inner_ui2.horizontal(|inner_ui3| {
                        inner_ui3.checkbox(&mut self.properties.set_x_pos, "");
                        inner_ui3.add_enabled_ui(self.properties.set_x_pos, |inner_ui4| {
                            inner_ui4.add(egui::DragValue::new(&mut self.properties.x_pos).speed(0.0000001).clamp_range(f32::MIN..=f32::MAX));
                        });
                        inner_ui3.label("X Position");
                    });
                    inner_ui2.horizontal(|inner_ui3| {
                        inner_ui3.checkbox(&mut self.properties.set_y_pos, "");
                        inner_ui3.add_enabled_ui(self.properties.set_y_pos, |inner_ui4| {
                            inner_ui4.add(egui::DragValue::new(&mut self.properties.y_pos).speed(0.0000001).clamp_range(f32::MIN..=f32::MAX));
                        });
                        inner_ui3.label("Y Position");
                    });
                    inner_ui2.horizontal(|inner_ui3| {
                        inner_ui3.checkbox(&mut self.properties.set_rot, "");
                        inner_ui3.add_enabled_ui(self.properties.set_rot, |inner_ui4| {
                            inner_ui4.add(egui::DragValue::new(&mut self.properties.rot).speed(0.0000001).clamp_range(0.0..=6.28318530718));
                        });
                        inner_ui3.label("Rotation");
                    });
                    inner_ui2.label("Velocity");
                    inner_ui2.horizontal(|inner_ui3| {
                        inner_ui3.checkbox(&mut self.properties.set_x_vel, "");
                        inner_ui3.add_enabled_ui(self.properties.set_x_vel, |inner_ui4| {
                            inner_ui4.add(egui::DragValue::new(&mut self.properties.x_vel).speed(0.001).clamp_range(f32::MIN..=f32::MAX));
                        });
                        inner_ui3.label("X Velocity");
                    });
                    inner_ui2.horizontal(|inner_ui3| {
                        inner_ui3.checkbox(&mut self.properties.set_y_vel, "");
                        inner_ui3.add_enabled_ui(self.properties.set_y_vel, |inner_ui4| {
                            inner_ui4.add(egui::DragValue::new(&mut self.properties.y_vel).speed(0.001).clamp_range(f32::MIN..=f32::MAX));
                        });
                        inner_ui3.label("Y Velocity");
                    });
                    inner_ui2.horizontal(|inner_ui3| {
                        inner_ui3.checkbox(&mut self.properties.set_rot_vel, "");
                        inner_ui3.add_enabled_ui(self.properties.set_rot_vel, |inner_ui4| {
                            inner_ui4.add(egui::DragValue::new(&mut self.properties.rot_vel).speed(0.001).clamp_range(f32::MIN..=f32::MAX));
                        });
                        inner_ui3.label("Rotational Velocity");
                    });
                    inner_ui2.label("Forces");
                    inner_ui2.horizontal(|inner_ui3| {
                        inner_ui3.checkbox(&mut self.properties.set_x_force, "");
                        inner_ui3.add_enabled_ui(self.properties.set_x_force, |inner_ui4| {
                            inner_ui4.add(egui::DragValue::new(&mut self.properties.x_force).speed(0.01));
                        });
                        inner_ui3.label("X Force");
                    });
                    inner_ui2.horizontal(|inner_ui3| {
                        inner_ui3.checkbox(&mut self.properties.set_y_force, "");
                        inner_ui3.add_enabled_ui(self.properties.set_y_force, |inner_ui4| {
                            inner_ui4.add(egui::DragValue::new(&mut self.properties.y_force).speed(0.01));
                        });
                        inner_ui3.label("Y Force");
                    });
                    inner_ui2.horizontal(|inner_ui3| {
                        inner_ui3.checkbox(&mut self.properties.set_rot_force, "");
                        inner_ui3.add_enabled_ui(self.properties.set_rot_force, |inner_ui4| {
                            inner_ui4.add(egui::DragValue::new(&mut self.properties.rot_force).speed(0.01));
                        });
                        inner_ui3.label("Rotational Force");
                    });
                    inner_ui2.label("Radius");
                    inner_ui2.horizontal(|inner_ui3| {
                        inner_ui3.checkbox(&mut self.properties.set_radius, "");
                        inner_ui3.add_enabled_ui(self.properties.set_radius, |inner_ui4| {
                            inner_ui4.add(egui::DragValue::new(&mut self.properties.radius).speed(0.001).clamp_range(0.0..=f32::MAX));
                        });
                    });
                    inner_ui2.label("Fixity");
                    inner_ui2.horizontal(|inner_ui3| {
                        inner_ui3.checkbox(&mut self.properties.set_x_fixity, "");
                        inner_ui3.add_enabled_ui(self.properties.set_x_fixity, |inner_ui4| {
                            if inner_ui4
                                .add(egui::SelectableLabel::new(
                                    self.properties.x_fixity,
                                    match self.properties.x_fixity {
                                        true => "True",
                                        false => "False",
                                    },
                                ))
                                .clicked()
                            {
                                self.properties.x_fixity = !self.properties.x_fixity;
                            };
                        });
                        inner_ui3.label("X Fixity");
                    });
                    inner_ui2.horizontal(|inner_ui3| {
                        inner_ui3.checkbox(&mut self.properties.set_y_fixity, "");
                        inner_ui3.add_enabled_ui(self.properties.set_y_fixity, |inner_ui4| {
                            if inner_ui4
                                .add(egui::SelectableLabel::new(
                                    self.properties.y_fixity,
                                    match self.properties.y_fixity {
                                        true => "True",
                                        false => "False",
                                    },
                                ))
                                .clicked()
                            {
                                self.properties.y_fixity = !self.properties.y_fixity;
                            };
                        });
                        inner_ui3.label("Y Fixity");
                    });
                    inner_ui2.horizontal(|inner_ui3| {
                        inner_ui3.checkbox(&mut self.properties.set_rot_fixity, "");
                        inner_ui3.add_enabled_ui(self.properties.set_rot_fixity, |inner_ui4| {
                            if inner_ui4
                                .add(egui::SelectableLabel::new(
                                    self.properties.rot_fixity,
                                    match self.properties.rot_fixity {
                                        true => "True",
                                        false => "False",
                                    },
                                ))
                                .clicked()
                            {
                                self.properties.rot_fixity = !self.properties.rot_fixity;
                            };
                        });
                        inner_ui3.label("Rotational Fixity");
                    });
                    inner_ui2.label("Material");
                    inner_ui2.horizontal(|inner_ui3| {
                        inner_ui3.checkbox(&mut self.properties.set_material, "");
                        inner_ui3.add_enabled_ui(self.properties.set_material, |inner_ui4| {
                            // inner_ui4.add(egui::DragValue::new(&mut self.properties.material).clamp_range(0..=(self.materials.len()/self.material_size - 1)));
                            inner_ui4.add(egui::Slider::new(&mut self.properties.material, 0..=(self.materials.len() / self.material_size - 1) as i32));
                        });
                    });
                    if inner_ui2
                        .add_enabled(
                            self.properties.set_material
                                || self.properties.set_x_pos
                                || self.properties.set_y_pos
                                || self.properties.set_rot
                                || self.properties.set_x_vel
                                || self.properties.set_y_vel
                                || self.properties.set_rot_vel
                                || self.properties.set_radius
                                || self.properties.set_rot_fixity
                                || self.properties.set_rot_force
                                || self.properties.set_x_fixity
                                || self.properties.set_x_force
                                || self.properties.set_y_fixity
                                || self.properties.set_y_force,
                            egui::Button::new("Set Properties"),
                        )
                        .clicked()
                    {
                        self.set_properties = !self.set_properties;
                    }
                });
            });
            //if ui.selectable_label(self.menu.properties_menu, "Properties").clicked() { self.menu.properties_menu = !self.menu.properties_menu; }
        });
    }

    fn materials_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("Materials", |ui| {
            ui.style_mut().wrap = Some(false);
            ui.set_max_width(83.0);

            let materials_count = self.materials.len() / self.material_size;
            for i in 0..materials_count {
                let mat_num = i;
                ui.menu_button(format!("Material {mat_num}"), |ui| {
                    ui.set_min_width(250.0);
                    ui.menu_button("Color", |ui| {
                        let mut color = Color32::from_rgb(
                            (self.materials[i * self.material_size + 0] * 255.0) as u8,
                            (self.materials[i * self.material_size + 1] * 255.0) as u8,
                            (self.materials[i * self.material_size + 2] * 255.0) as u8,
                        );
                        let color2 = color.clone();
                        egui::color_picker::color_picker_color32(ui, &mut color, Alpha::Opaque);
                        if color.r() != color2.r() || color.g() != color2.g() || color.b() != color2.b() {
                            self.materials_changed = true;
                        }
                        let color_srgb = color.to_srgba_unmultiplied();
                        self.materials[i * self.material_size + 0] = color_srgb[0] as f32 / 255.0;
                        self.materials[i * self.material_size + 1] = color_srgb[1] as f32 / 255.0;
                        self.materials[i * self.material_size + 2] = color_srgb[2] as f32 / 255.0;
                        if ui.add(egui::Slider::new(&mut self.materials[i * self.material_size + 0], 0.0..=1.0)).changed() {
                            self.materials_changed = true;
                        };
                        if ui.add(egui::Slider::new(&mut self.materials[i * self.material_size + 1], 0.0..=1.0)).changed() {
                            self.materials_changed = true;
                        };
                        if ui.add(egui::Slider::new(&mut self.materials[i * self.material_size + 2], 0.0..=1.0)).changed() {
                            self.materials_changed = true;
                        };
                    });
                    // if ui.add(egui::Slider::new(&mut self.materials[i*self.material_size + 0], 0.0..=1.0).text("Red")).changed() { self.materials_changed = true; };
                    // if ui.add(egui::Slider::new(&mut self.materials[i*self.material_size + 1], 0.0..=1.0).text("Green")).changed() { self.materials_changed = true; };
                    // if ui.add(egui::Slider::new(&mut self.materials[i*self.material_size + 2], 0.0..=1.0).text("Blue")).changed() { self.materials_changed = true; };
                    if ui
                        .add(egui::Slider::new(&mut self.materials[i * self.material_size + 3], -100000.0..=100000000000.0).text("Density"))
                        .changed()
                    {
                        self.materials_changed = true;
                    };
                    if ui
                        .add(egui::Slider::new(&mut self.materials[i * self.material_size + 4], -100000.0..=100000000000.0).text("Normal Stiffness"))
                        .changed()
                    {
                        self.materials_changed = true;
                    };
                    if ui
                        .add(egui::Slider::new(&mut self.materials[i * self.material_size + 5], -100000.0..=100000000000.0).text("Shear Stiffness"))
                        .changed()
                    {
                        self.materials_changed = true;
                    };
                });
            }
            if ui.button("Add Material").clicked() {
                self.materials.resize(self.material_size + self.materials.len(), 0.0);
                let base = self.materials.len() - 6;
                self.materials[base] = rand::random();
                self.materials[base + 1] = rand::random();
                self.materials[base + 2] = rand::random();
                self.materials[base + 3] = self.materials[3];
                self.materials[base + 4] = self.materials[4];
                self.materials[base + 5] = self.materials[5];
                self.materials_changed = true;
            }
        });
    }

    fn data_menu(&mut self, ui: &mut Ui, ctx: &Context) {
        ui.menu_button("Data", |ui| {
            ui.style_mut().wrap = Some(false);

            if ui.selectable_label(self.view.data_menu, "Data Panel").clicked() {
                self.view.data_menu = !self.view.data_menu;
            }

            ui.separator();
            ui.label("Recording");

            if ui.checkbox(&mut self.timed_recording, "Timed").changed() {
                self.start_time = self.sim_time;
            }
            ui.add_enabled(self.timed_recording, egui::DragValue::new(&mut self.recording_duration).speed(0.001).suffix("s"));
            // });
            // ui.horizontal_centered(|ui| {
            if !(self.recording || self.gather_data) {
                if ui.button("Start").clicked() {
                    if !self.timed_recording {
                        self.gather_data = true;
                    } else {
                        self.recording = true;
                    }
                    self.start_time = self.sim_time;
                }
            } else {
                if ui.button("Stop").clicked() {
                    self.recording = false;
                    self.gather_data = false;
                    // self.start_time = self.sim_time;
                }
            }
            // });

            if ui.button("Export").clicked() {
                self.save_data(None);
            }
        });
        if self.view.data_menu {
            egui::TopBottomPanel::bottom("data_panel").resizable(true).default_height(300.0).show(ctx, |ui| {
                // if ui.checkbox(&mut self.gather_data, "Gather Data").changed() {
                //     self.start_time = self.sim_time;
                // }
                let mut reset_button = None;
                egui::menu::bar(ui, |ui| {
                    // });
                    // ui.horizontal_centered(|ui|{

                    egui::ComboBox::new("graph_property", "").selected_text(format!("{:?}", self.plotted_prop)).show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.plotted_prop, Property::X_Position, "X Position");
                        ui.selectable_value(&mut self.plotted_prop, Property::Y_Position, "Y Position");
                        ui.selectable_value(&mut self.plotted_prop, Property::Rotation, "Rotation");
                        ui.selectable_value(&mut self.plotted_prop, Property::X_Velocity, "X Velocity");
                        ui.selectable_value(&mut self.plotted_prop, Property::Y_Velocity, "Y Velocity");
                        ui.selectable_value(&mut self.plotted_prop, Property::Rotational_Velocity, "Rotational Velocity");
                        ui.selectable_value(&mut self.plotted_prop, Property::Normal_Force, "Normal Force");
                        ui.selectable_value(&mut self.plotted_prop, Property::Shear_Force, "Shear Force");
                        ui.selectable_value(&mut self.plotted_prop, Property::Moment, "Moment");
                        // ui.selectable_value(&mut self.plotted_prop, Property::Data_4, "Data 4");
                        ui.selectable_value(&mut self.plotted_prop, Property::FPS, "FPS");
                    });
                    reset_button = Some(ui.add(egui::Button::new("Reset View")));
                });
                let mut plot = egui::plot::Plot::new("physics plot").auto_bounds_x().auto_bounds_y().clamp_grid(true);
                if reset_button.unwrap().clicked() {
                    plot = plot.reset()
                }
                plot.show(ui, |plot_ui| {
                    match self.plotted_prop {
                        Property::X_Position => {
                            plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(self.data.x_pos_data.to_owned())));
                        }
                        Property::Y_Position => {
                            plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(self.data.y_pos_data.to_owned())));
                        }
                        Property::Rotation => {
                            plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(self.data.rot_data.to_owned())));
                        }
                        Property::X_Velocity => {
                            plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(self.data.x_vel_data.to_owned())));
                        }
                        Property::Y_Velocity => {
                            plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(self.data.y_vel_data.to_owned())));
                        }
                        Property::Rotational_Velocity => {
                            plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(self.data.rot_vel_data.to_owned())));
                        }
                        Property::Normal_Force => {
                            plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(self.data.data1.to_owned())));
                        }
                        Property::Shear_Force => {
                            plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(self.data.data2.to_owned())));
                        }
                        Property::Moment => {
                            plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(self.data.data3.to_owned())));
                        }
                        // Property::Data_4 => {plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(self.data.data4.to_owned())));},
                        Property::FPS => {
                            plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(self.data.fps.to_owned())));
                        }
                    }
                });
            });
        }
    }

    fn script_menu(&mut self, ui: &mut Ui, ctx: &Context) {
        ui.menu_button("Scripts", |ui| {
            ui.style_mut().wrap = Some(false);
            if ui.selectable_label(self.view.script_menu, "Script Panel").clicked() {
                self.view.script_menu = !self.view.script_menu;
            }
        });
    }

    fn developer_menu(&mut self, ui: &mut Ui, ctx: &Context) {
        ui.menu_button("Developer", |ui| {
            ui.style_mut().wrap = Some(false);
            ui.label("Debug");
            ui.checkbox(&mut self.view.render_bp_grid, "Render Grid");
            ui.checkbox(&mut self.view.show_hit_tex, "Show Hit Texture");
            ui.separator();
            // ui.label("Experimental");

            if ui.selectable_label(self.view.code_editor, "Code Editor").clicked() {
                self.view.code_editor = !self.view.code_editor;
            }
        });
    }

    fn script_panel(&mut self, ctx: &Context, script_manager: &mut ScriptManager, prog: &mut WGPUProg, device: &mut Device, queue: &mut Queue) {
        if self.view.script_menu {
            egui::SidePanel::right("script_panel").resizable(true).show(ctx, |ui| {
                // egui::menu::bar(ui, |ui|{});
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut script_manager.scripts[self.current_script].name);
                    let delete_button = ui.button("Delete");
                    if delete_button.clicked() {
                        ui.memory_mut(|mem| mem.toggle_popup(format!("{}_delete", self.current_script).into()));
                    }
                    // if script_manager.delete_window[self.current_script] {
                    egui::popup_below_widget(ui, format!("{}_delete", self.current_script).into(), &delete_button, |ui2| {
                        // ui2.set_min_width(100.0);
                        // ui2.label(format!("Delete {}?", script_manager.scripts[self.current_script].name));
                        ui2.horizontal(|ui3| {
                            if ui3.button("Delete").clicked() {
                                script_manager.delete_script(self.current_script);
                                if self.current_script == script_manager.scripts.len() {
                                    self.current_script -= 1;
                                }
                            }
                            if ui3.button("Cancel").clicked() {
                                ui.memory_mut(|mem| mem.toggle_popup(format!("{}_delete", self.current_script).into()));
                            }
                        });
                    });
                    // }
                    if ui.selectable_label(script_manager.threads[self.current_script].executing, "Run").clicked() {
                        script_manager.toggle_execution(self.current_script);
                    }
                });
                ui.separator();
                ui.horizontal(|ui| {
                    for i in 0..script_manager.scripts.len() {
                        if ui.selectable_label(self.current_script == i, script_manager.scripts[i].name.as_str()).clicked() {
                            self.current_script = i;
                        }
                    }
                    if ui.button("+").clicked() {
                        script_manager.new_script(format!("Script {}", script_manager.scripts.len() + 1).as_str());
                        self.current_script = script_manager.scripts.len() - 1;
                    }
                });
                ui.separator();
                ui.heading("Actions");
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Trigger");
                    egui::ComboBox::new("Trigger", "")
                        .selected_text(format!("{}", script_manager.scripts[self.current_script].script_trigger.to_string()))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut script_manager.scripts[self.current_script].script_trigger, Trigger::None, "None");
                            ui.selectable_value(&mut script_manager.scripts[self.current_script].script_trigger, Trigger::Click, "Click");
                            ui.selectable_value(&mut script_manager.scripts[self.current_script].script_trigger, Trigger::KeyDown(Key::Null), "KeyDown");
                            ui.selectable_value(&mut script_manager.scripts[self.current_script].script_trigger, Trigger::KeyPressed(Key::Null), "KeyPressed");
                        });
                    match script_manager.scripts[self.current_script].script_trigger {
                        Trigger::KeyDown(key) | Trigger::KeyPressed(key) => {
                            let mut k = script_manager.scripts[self.current_script].script_trigger.keycode();
                            egui::ComboBox::new("Key", "").selected_text(format!("{:?}", k)).show_ui(ui, |ui| {
                                ui.selectable_value(&mut k, Key::Space, "Space");
                                ui.selectable_value(&mut k, Key::W, "W");
                                ui.selectable_value(&mut k, Key::A, "A");
                                ui.selectable_value(&mut k, Key::S, "S");
                                ui.selectable_value(&mut k, Key::D, "D");
                            });
                            script_manager.scripts[self.current_script].script_trigger.set_key(k);
                        }
                        _ => {}
                    }

                    ui.checkbox(&mut script_manager.scripts[self.current_script].auto_run, "Auto-Run")
                        .on_hover_text("Auto-run when script is loaded.");
                });
                ui.separator();
                egui::ScrollArea::new([false, true]).show(ui, |ui| {
                    if script_manager.scripts.len() > 0 {
                        let mut i = 0;
                        while i < script_manager.scripts[self.current_script].actions.len() {
                            ui.horizontal(|ui| {
                                let current_digits = ((i + 1) as f32).log(10.0) as i32;
                                let max_digits = (script_manager.scripts[self.current_script].actions.len() as f32).log(10.0) as i32;
                                let spaces = (max_digits - current_digits) * 2;
                                let mut space_string = format!("");
                                for j in 0..spaces {
                                    space_string.push(' ');
                                }
                                ui.label(format!("{space_string}{}", i + 1));
                                let mut changed_action = false;
                                let action_index = i;
                                egui::ComboBox::new(format!("{}", i).as_str(), "")
                                    .selected_text(format!("{}", script_manager.scripts[self.current_script].actions[action_index].name.to_string()))
                                    .show_ui(ui, |ui| {
                                        if ui.selectable_value(&mut script_manager.scripts[self.current_script].actions[i].name, Command::None, "None").clicked() {
                                            changed_action = true;
                                        }
                                        if ui.selectable_value(&mut script_manager.scripts[self.current_script].actions[i].name, Command::Wait, "Wait").clicked() {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[self.current_script].actions[i].name, Command::Simulate, "Simulate")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[self.current_script].actions[i].name, Command::Advance, "Advance")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[self.current_script].actions[i].name, Command::Select, "Select")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[self.current_script].actions[i].name, Command::Select_All, "Select All")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[self.current_script].actions[i].name, Command::Set_Properties, "Set Properties")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui.selectable_value(&mut script_manager.scripts[self.current_script].actions[i].name, Command::Goto, "Goto").clicked() {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[self.current_script].actions[i].name, Command::Backup, "Backup")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[self.current_script].actions[i].name, Command::Restore, "Restore")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[self.current_script].actions[i].name, Command::Call_Script, "Call Script")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[self.current_script].actions[i].name, Command::Record, "Record")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[self.current_script].actions[i].name, Command::Export, "Export")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                    });
                                if changed_action {
                                    script_manager.scripts[self.current_script].actions[i].init_parameters(self.setup.particles);
                                }
                                let mut script_names = vec![];
                                for script in &script_manager.scripts {
                                    script_names.push(script.name.clone());
                                }
                                let action_count = script_manager.scripts[self.current_script].actions.len();
                                script_manager.scripts[self.current_script].actions[action_index].ui(
                                    ui,
                                    format!("{}:{}", self.current_script, i),
                                    (self.materials.len() / self.material_size) as usize,
                                    action_count,
                                    prog,
                                    device,
                                    queue,
                                    script_names,
                                );
                                ui.with_layout(egui::Layout::right_to_left(Align::RIGHT), |ui| {
                                    if ui.button("X").clicked() {
                                        script_manager.scripts[self.current_script].delete_action(i);
                                    }
                                });
                            });
                            i += 1;
                        }
                    }
                    ui.separator();
                    if ui.button("Add Action").clicked() {
                        script_manager.push_action(self.current_script, Action::new(Command::None, vec![]));
                    }
                });
            });
        }
    }

    fn code_editor(&mut self, ctx: &Context, prog: &mut WGPUProg, config: &mut WGPUConfig) {
        let mut panel_width = 200.0; // Store this as a field in your struct

        if self.view.code_editor {
            egui::SidePanel::left("code_editor").resizable(true).show(ctx, |ui| {
                ui.set_min_width(ui.available_width());
                ui.set_max_width(ui.available_width());
                ui.horizontal(|ui| {
                    if ui.selectable_label(self.curr_shader == 1, "Background").clicked() {
                        self.curr_shader = 1;
                    }
                    if ui.selectable_label(self.curr_shader == 0, "Particles").clicked() {
                        self.curr_shader = 0;
                    }
                    if ui.selectable_label(self.curr_shader == 2, "Hit Detection").clicked() {
                        self.curr_shader = 2;
                    }
                    if ui.selectable_label(self.curr_shader == 3, "Post Processing").clicked() {
                        self.curr_shader = 3;
                    }
                    if ui.selectable_label(self.curr_shader == 4, "Laws of Motion").clicked() {
                        self.curr_shader = 4;
                    }
                    if ui.selectable_label(self.curr_shader == 5, "Simulation").clicked() {
                        self.curr_shader = 5;
                    }
                });
                if self.curr_shader < 4 {
                    egui::ScrollArea::show(egui::ScrollArea::new([true, true]), ui, |ui| {
                        let text_edit = TextEdit::multiline(&mut prog.shader_strs[self.curr_shader])
                            .desired_width(ui.available_width())
                            .font(egui::TextStyle::Monospace)
                            .code_editor();
                        if ui.add(text_edit).changed() {
                            prog.rebuild_pipeline(config, &self, self.curr_shader);
                            if self.curr_shader == 2 {
                                prog.shader_strs[4] = prog.shader_strs[2].clone();
                                prog.rebuild_pipeline(config, &self, 4);
                            }
                        }
                    });
                } else {
                    egui::ScrollArea::show(egui::ScrollArea::new([true, true]), ui, |ui| {
                        let text_edit = TextEdit::multiline(&mut prog.shader_prog.shader_strs[self.curr_shader - 4])
                            .desired_width(ui.available_width())
                            .font(egui::TextStyle::Monospace)
                            .code_editor();
                        if ui.add(text_edit).changed() {
                            prog.shader_prog.rebuild_pipeline(config, &self, self.curr_shader - 4);
                        }
                    });
                }
            });
        }
    }

    pub fn grid_info(&mut self) -> (usize, f32, i32, i32, i32) {
        let width  = self.simulation.hor_bound  * 2.0;
        let height = self.simulation.vert_bound * 2.0;
        let     max_rad = self.setup.max_radius * 2.0;
        let mut min_rad = self.setup.min_radius;
        if !self.setup.variable_rad { min_rad = self.setup.max_radius; }
        let w = (width/max_rad).ceil() as i32;
        let h = (height/max_rad).ceil() as i32;
        let cell_cap = ((max_rad/min_rad + 1.0).powf(2.0).ceil() as i32).min(self.setup.particles as i32) + 2;
        let total_size = w * h * cell_cap;
        println!("Cell Capacity:   {}", cell_cap);
        println!("Cell Dimensions: {} x {}", w, h);
        println!("Total Cells:     {}", w * h);
        println!("Total Capacity:  {}", total_size);
        println!("Bytes:           {}", total_size * 4);
    
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
        let path = FileDialog::new().set_location("").add_filter("Binary File", &["bin"]).show_open_single_file().unwrap();

        match path {
            Some(path) => {
                self.current_file = path.clone();
                self.load = true;
            }
            None => {}
        };
    }

    pub fn save(&mut self) {
        let path = FileDialog::new().add_filter("Binary File", &["bin"]).show_save_single_file().unwrap();

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

    pub fn save_data(&mut self, path_param: Option<PathBuf>) {
        let path = match path_param {
            Some(p) => Some(p),
            None => FileDialog::new().set_location("~").add_filter("CSV File", &["csv"]).show_save_single_file().unwrap(),
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
                let fps = self.data.fps[i][1];

                writeln!(
                    file,
                    "{},{},{},{},{},{},{},{},{},{},{},{}",
                    timestamp, x_pos, y_pos, x_vel, y_vel, rot, rot_vel, data1, data2, data3, data4, fps
                )
                .expect("Unable to write data row");
            }

            println!("Data saved to: {:?}", file_path);
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

    pub fn load_memory(&mut self) {
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
                println!("Err: Failed to read memory file.");
            }
        }
    }

    pub fn collision_settings(&mut self) -> Vec<f32> {
        self.changed_collision_settings = false;
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
    // Data_4,
    FPS,
}

#[derive(Serialize, Deserialize)]
struct Memory {
    pub current_dir: std::path::PathBuf,
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
