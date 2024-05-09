use std::fmt::Debug;

use egui::*;
use std::*;

use native_dialog::{FileDialog, MessageDialog, MessageType};

use crate::{state::State, wgpu_structs::Uniform, window_init::Canvas};

pub struct Menu {
    pub render_settings: bool,
    pub materials_menu: bool,
    pub setup_menu: bool,
    pub physics_menu: bool,
    pub walls_menu: bool,
    pub save_load_menu: bool,
    pub properties_menu: bool,
    pub data_menu: bool,
}

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
    pub current_file: std::path::PathBuf,
    pub save: bool
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
            current_file: std::path::PathBuf::new(),
            save: false
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

    pub fn save(&mut self) {
        let path = FileDialog::new()
            .set_location("~/OneDrive/Code/WASM/Engine Programs/Particle-Physics-Sim/data")
            .add_filter("CSV File", &["csv"])
            .show_save_single_file()
            .unwrap();

        match path {
            Some(path) => {
                self.current_file = path.clone();
                self.save = true;
            },
            None => {},
        };
    }
}

pub struct Settings {
    pub genPerFrame: i32,
    pub particles: usize,
    pub workgroups: usize,
    pub workgroup_size: usize,
    pub max_radius: f32,
    pub min_radius: f32,
    pub max_bonds: usize,
    pub max_contacts: usize,
    pub max_h_velocity: f32,
    pub min_h_velocity: f32,
    pub max_v_velocity: f32,
    pub min_v_velocity: f32,
    pub structure: Structure,
    pub grid_width: f32,
    pub variable_rad: bool,
    pub settings_menu: bool,
    pub holeyness: f32,
    pub maintain_ar: bool,
    pub hor_bound: f32,
    pub vert_bound: f32,
    pub gravity: bool,
    pub planet_mode: bool,
    pub gravity_acceleration: f32,
    pub bonds: i32,
    pub bondenum: BondType,
    pub bond_tearing: bool,
    pub bond_force_limit: f32,
    pub stiffness: f32,
    pub collisions: bool,
    pub friction: bool,
    pub friction_coefficient: f32,
    pub rotation: bool,
    pub linear_contact_bonds: bool,
    pub changed_collision_settings: bool,
    pub scale: f32,
    pub circular_particles: bool,
    pub render_rot: bool,
    pub color_code_rot: bool,
    pub colors: bool,
    pub random_colors: bool,
    pub render_bonds: bool,
    pub two_part: bool,
    pub materials: Vec<f32>,
    pub material_size: usize,
    pub materials_changed: bool,
    pub menu: Menu,
    pub current_file: std::path::PathBuf,
    pub load: bool,
    pub save: bool,
    pub regen_bonds: bool,
    pub properties: Properties,
    pub set_properties: bool,
    pub data: Data,
    pub gather_data: bool,
    pub auto_size_plot: bool,
    pub plotted_prop: Property,
    pub contact_damping: f32,
    pub bond_damping: f32,
    pub drag: f32,
    pub bond_shear_limit: f32,
    pub verlet: bool,
    pub timestep: f32,
    pub maxGenPerFrame: i32,
    pub hz: f32,
    pub fps: f32
}

impl Settings {
    pub fn new(canvas: &Canvas) -> Self {
        let genPerFrame = 1;
        let particles = 256;
        let workgroup_size = 256;
        let workgroups = (particles as f32/workgroup_size as f32).ceil() as usize;
        //particle settings
        let max_radius = 0.025;
        let variable_rad = false;
        let holeyness = 1.7;
        let min_radius = max_radius/holeyness;
        let max_bonds = 6;
        let max_contacts = max_bonds + 8;
        let max_h_velocity = 0.0;
        let min_h_velocity = 0.0;
        let max_v_velocity = 0.0;
        let min_v_velocity = 0.0;
        let structure = Structure::Grid;
        let grid_width = 32.0;
        let settings_menu = true;
        let maintain_ar = true;
        let hor_bound = 1.333;
        let vert_bound = 1.0;
        let gravity = true;
        let gravity_acceleration = 1.0;
        let bonds = 0;
        let bondenum = BondType::Unbonded;
        let bond_tearing = false;
        let bond_force_limit = 0.5;
        let stiffness = 10.0;
        let collisions = true;
        let friction = true;
        let friction_coefficient = 0.5;
        let rotation = true;
        let linear_contact_bonds = true;
        let changed_collision_settings = false;
        let scale = 1.0/vert_bound;
        let circular_particles = true;
        let render_rot = false;
        let color_code_rot = false;
        let colors = true;
        let random_colors = false;
        let render_bonds = true;
        let two_part = false;
        let materials = vec![
            1.0,
            1.0,
            1.0,
            0.01,
            100.0,
            50.0
        ];
        let material_size = 6;
        let materials_changed = false; 
        let menu = Menu {
            render_settings: false,
            materials_menu: false,
            setup_menu: false,
            physics_menu: false,
            walls_menu: false,
            save_load_menu: false,
            properties_menu: false,
            data_menu: false,
        };

        let current_file = std::path::PathBuf::new();

        Self {
            genPerFrame,
            particles,
            workgroups,
            workgroup_size,
            max_radius,
            min_radius,
            max_bonds,
            max_contacts,
            max_h_velocity,
            min_h_velocity,
            max_v_velocity,
            min_v_velocity,
            structure,
            grid_width,
            variable_rad,
            settings_menu,
            holeyness,
            maintain_ar,
            hor_bound,
            vert_bound,
            gravity,
            planet_mode: false,
            gravity_acceleration,
            bonds,
            bondenum,
            bond_tearing,
            bond_force_limit,
            stiffness,
            collisions,
            friction,
            friction_coefficient,
            rotation,
            linear_contact_bonds,
            changed_collision_settings,
            scale,
            circular_particles,
            render_rot,
            color_code_rot,
            colors,
            random_colors,
            render_bonds,
            two_part,
            materials,
            material_size,
            materials_changed,
            menu,
            current_file,
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
            contact_damping: 0.2,
            bond_damping: 0.2,
            drag: 1.0,
            bond_shear_limit: 0.5,
            verlet: true,
            timestep: 0.0000390625,
            maxGenPerFrame: 213,
            hz: 120.0,
            fps: 120.0
        }
    }

    pub fn set_particles(&mut self, particles: usize) {
        self.particles = particles;
        self.workgroups = (self.particles as f32/self.workgroup_size as f32).ceil() as usize;
    }

    pub fn ui(&mut self, ctx: &Context, state: &State) -> bool {
        let mut reset = false;
        if !self.current_file.exists() && self.save {
            self.save();
        }
        if self.settings_menu {
            egui::TopBottomPanel::top("Settings Menu").show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    // ui.heading("Menu");
                    if ui.selectable_label(self.menu.setup_menu, "Setup").clicked() { self.menu.setup_menu = !self.menu.setup_menu; }
                    if ui.selectable_label(self.menu.physics_menu, "Physics Settings").clicked() { self.menu.physics_menu = !self.menu.physics_menu; }
                    if ui.selectable_label(self.menu.materials_menu, "Materials").clicked() { self.menu.materials_menu = !self.menu.materials_menu; }
                    if ui.selectable_label(self.menu.properties_menu, "Properties").clicked() { self.menu.properties_menu = !self.menu.properties_menu; }
                    if ui.selectable_label(self.menu.render_settings, "Render Settings").clicked() { self.menu.render_settings = !self.menu.render_settings; }
                    if ui.selectable_label(self.menu.walls_menu, "Walls").clicked() { self.menu.walls_menu = !self.menu.walls_menu; }
                    if ui.selectable_label(self.menu.data_menu, "Data").clicked() { self.menu.data_menu = !self.menu.data_menu; }
                    if ui.selectable_label(self.menu.save_load_menu, "Save/Load").clicked() { self.menu.save_load_menu = !self.menu.save_load_menu; }
                });
            });
            if self.menu.render_settings {
                egui::Window::new("Render Settings").collapsible(false).auto_sized().show(ctx, |ui| {
                    ui.checkbox(&mut self.circular_particles, "Circular Particles");
                    ui.checkbox(&mut self.render_rot, "Render Rotation");
                    ui.checkbox(&mut self.render_bonds, "Render Bonds");
                    ui.checkbox(&mut self.colors, "Colors");
                    ui.checkbox(&mut self.random_colors, "Random Colors");
                    ui.checkbox(&mut self.color_code_rot, "Color Code Rotation"); 
                });
            }
            if self.menu.properties_menu {
                egui::Window::new("Properties").collapsible(false).auto_sized().show(ctx, |ui| {
                    ui.horizontal(|inner_ui| {
                        inner_ui.vertical(|inner_ui2| {
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
                                    inner_ui4.add(egui::DragValue::new(&mut self.properties.rot_vel).speed(0.001).clamp_range(0.0..=6.28318530718));
                                });
                                inner_ui3.label("Rotational Velocity");
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
                                    if inner_ui4.add(egui::SelectableLabel::new(self.properties.x_fixity, match self.properties.x_fixity {true => {"True"}, false => {"False"}})).clicked() { self.properties.x_fixity = ! self.properties.x_fixity; };
                                });
                                inner_ui3.label("X Fixity");
                            });
                            inner_ui2.horizontal(|inner_ui3| {
                                inner_ui3.checkbox(&mut self.properties.set_y_fixity, "");
                                inner_ui3.add_enabled_ui(self.properties.set_y_fixity, |inner_ui4| {
                                    if inner_ui4.add(egui::SelectableLabel::new(self.properties.y_fixity, match self.properties.y_fixity {true => {"True"}, false => {"False"}})).clicked() { self.properties.y_fixity = ! self.properties.y_fixity; };
                                });
                                inner_ui3.label("Y Fixity");
                            });
                            inner_ui2.horizontal(|inner_ui3| {
                                inner_ui3.checkbox(&mut self.properties.set_rot_fixity, "");
                                inner_ui3.add_enabled_ui(self.properties.set_rot_fixity, |inner_ui4| {
                                    if inner_ui4.add(egui::SelectableLabel::new(self.properties.rot_fixity, match self.properties.rot_fixity {true => {"True"}, false => {"False"}})).clicked() { self.properties.rot_fixity = ! self.properties.rot_fixity; };
                                });
                                inner_ui3.label("Rotational Fixity");
                            });
                            inner_ui2.label("Material");
                            inner_ui2.horizontal(|inner_ui3| {
                                inner_ui3.checkbox(&mut self.properties.set_material, "");
                                inner_ui3.add_enabled_ui(self.properties.set_material, |inner_ui4| {
                                    // inner_ui4.add(egui::DragValue::new(&mut self.properties.material).clamp_range(0..=(self.materials.len()/self.material_size - 1)));
                                    inner_ui4.add(egui::Slider::new(&mut self.properties.material, 0..=(self.materials.len()/self.material_size - 1) as i32));
                                });
                            });
                            if inner_ui2.add_enabled(
                                self.properties.set_material || self.properties.set_x_pos || self.properties.set_y_pos || self.properties.set_rot || self.properties.set_x_vel || self.properties.set_y_vel || self.properties.set_rot_vel || self.properties.set_radius || self.properties.set_rot_fixity || self.properties.set_rot_force || self.properties.set_x_fixity || self.properties.set_x_force || self.properties.set_y_fixity || self.properties.set_y_force,
                                egui::Button::new("Set Properties")).
                                clicked() 
                            {
                                self.set_properties = !self.set_properties;
                            }
                        });
                    });
                });
            }
            if self.menu.setup_menu {
                egui::Window::new("Setup").collapsible(false).auto_sized().show(ctx, |ui| {
                    if !self.two_part { if ui.add(egui::Slider::new(&mut self.particles, 1..=self.workgroup_size*200).
                        text("Particles").
                        step_by(1.0)).changed() {
                            self.workgroups = (self.particles as f32/self.workgroup_size as f32).ceil() as usize;
                            self.grid_width = self.grid_width.min(self.particles as f32);
                            reset = true;
                        };}

                        egui::ComboBox::from_label("Structures")
                            .selected_text(format!("{:?}", self.structure))
                            .show_ui(ui, |ui| {
                                // reset = ui.selectable_value(&mut self.structure, Structure::Random, "Random").changed();
                                reset = reset || ui.selectable_value(&mut self.structure, Structure::Grid, "Grid").changed();
                                reset = reset || ui.selectable_value(&mut self.structure, Structure::Exp1, "Experiment 1").changed();
                                reset = reset || ui.selectable_value(&mut self.structure, Structure::Exp2, "Experiment 2").changed();
                                reset = reset || ui.selectable_value(&mut self.structure, Structure::Exp3, "Experiment 3").changed();
                                reset = reset || ui.selectable_value(&mut self.structure, Structure::Exp4, "Experiment 4").changed();
                                reset = reset || ui.selectable_value(&mut self.structure, Structure::Exp5, "Experiment 5").changed();
                                reset = reset || ui.selectable_value(&mut self.structure, Structure::Exp6, "Experiment 6").changed();
                                reset = reset || ui.selectable_value(&mut self.structure, Structure::Mats, "Mats").changed();
                            });
                        if !self.two_part { if self.structure == Structure::Grid {
                            if ui.add(egui::Slider::new(&mut self.grid_width, 1.0..=self.particles as f32).
                            text("Grid Width").step_by(0.01)
                            .logarithmic(true)).changed() {
                                reset = true;
                            };
                        }
                        if ui.checkbox(&mut self.variable_rad, "Random Radius").changed() {
                            reset = true;
                        }
                        if self.variable_rad {
                            match self.structure {
                                Structure::Grid => {
                                    if ui.add(egui::Slider::new(&mut self.holeyness, 1.0..=10.0).
                                    text("Holeyness")).changed() {
                                        self.min_radius = self.max_radius/self.holeyness;
                                        reset = true;
                                    };
                                },
                                _ => {
                                    if ui.add(egui::Slider::new(&mut self.max_radius, 0.0001..=0.5).
                                    text("Max Radius")).changed() {
                                        reset = true;
                                    };
                                    if ui.add(egui::Slider::new(&mut self.min_radius, 0.0001..=0.5).
                                    text("Min Radius")).changed() {
                                        reset = true;
                                    };
                                }
                            }
                        }
                        egui::CollapsingHeader::new("Initial Velocities").show(ui, |ui| {
                            if ui.add(egui::Slider::new(&mut self.max_h_velocity, -10.0..=10.0).
                            text("Max xV")).changed() {
                                if self.max_h_velocity < self.min_h_velocity {
                                    self.min_h_velocity = self.max_h_velocity;
                                }
                                reset = true;
                            };
                            if ui.add(egui::Slider::new(&mut self.min_h_velocity, -10.0..=10.0).
                            text("Min xV")).changed() {
                                if self.max_h_velocity < self.min_h_velocity {
                                    self.max_h_velocity = self.min_h_velocity;
                                }
                                reset = true;
                            };
                            if ui.add(egui::Slider::new(&mut self.max_v_velocity, -10.0..=10.0).
                            text("Max yV")).changed() {
                                if self.max_v_velocity < self.min_v_velocity {
                                    self.min_v_velocity = self.max_v_velocity;
                                }
                                reset = true;
                            };
                            if ui.add(egui::Slider::new(&mut self.min_v_velocity, -10.0..=10.0).
                            text("Min yV")).changed() {
                                if self.max_v_velocity < self.min_v_velocity {
                                    self.max_v_velocity = self.min_v_velocity;
                                }
                                reset = true;
                            };
                        });}
                        if ui.button("Regenerate Bonds").clicked() {
                            self.regen_bonds = true;                            
                        }
                    });
                }
            if self.menu.physics_menu {
                egui::Window::new("Physics").collapsible(false).auto_sized().show(ctx, |ui| {
                    if ui.add(egui::Slider::new(&mut self.timestep, 0.0..=1.0/self.hz).logarithmic(true).text("Sec/Tick")).changed() {
                        self.changed_collision_settings = true;
                    }
                    let fps_perc = 100.0*self.fps/self.hz; 
                    ui.add(egui::Slider::new(&mut self.genPerFrame, 1..=self.maxGenPerFrame).logarithmic(true).text(format!("Ticks/Frame ({:.0}/{:.0})", self.fps, self.hz)).text_color(Color32::from_rgb((255.0*(1.0 - (self.fps/self.hz).clamp(0.0, 1.0))) as u8, (255.0*(self.fps/self.hz).clamp(0.0, 1.0)) as u8, 0)));

                    if ui.checkbox(&mut self.gravity, "Gravity").changed() {
                        self.changed_collision_settings = true;
                    }
                    if ui.checkbox(&mut self.verlet, "Verlet Velocity Integration").changed() {
                        self.changed_collision_settings = true;
                    }
                    if self.gravity {
                        if ui.checkbox(&mut self.planet_mode, "Planet Mode").changed() {
                            self.changed_collision_settings = true;
                        }
                        if ui.add(egui::Slider::new(&mut self.gravity_acceleration, -100.0..=100.0).step_by(0.1).
                        text("G Force")).changed() {
                            // println!("{}", self.gravity_acceleration);
                            self.changed_collision_settings = true;
                        };
                    }
                    if ui.add(egui::Slider::new(&mut self.contact_damping, 0.0..=10.0).
                    text("contact_damping")).changed() {
                        self.changed_collision_settings = true;
                    };
                    let mut changed_bonds = false;
                    egui::ComboBox::from_label("Bonds")
                    .selected_text(format!("{:?}", self.bondenum))
                    .show_ui(ui, |ui| {
                        changed_bonds = changed_bonds || ui.selectable_value(&mut self.bondenum, BondType::Unbonded, "Unbonded").changed();
                        changed_bonds = changed_bonds || ui.selectable_value(&mut self.bondenum, BondType::Normal_Bonds, "Normal Bonds").changed();
                        changed_bonds = changed_bonds || ui.selectable_value(&mut self.bondenum, BondType::Linear_Contact_Bond, "Linear Contact Bonds").changed();
                        changed_bonds = changed_bonds || ui.selectable_value(&mut self.bondenum, BondType::Parallel_Linear_Contact_Bond, "Parallel Linear Contact Bonds").changed();
                    });
                    if changed_bonds { 
                        self.changed_collision_settings = true;
                        self.updateBonds();
                    }
                    // if ui.checkbox(&mut self.bonds, "Bonds").changed() {
                        //     self.changed_collision_settings = true;
                        // }
                    if self.bonds != 0 {
                        if ui.add(egui::Slider::new(&mut self.stiffness, 0.001..=1000000.0).step_by(0.001).
                        text("Stiffness")).changed() {
                            self.changed_collision_settings = true;
                        };
                        // if ui.add(egui::Slider::new(&mut self.bond_shear_limit, 0.0..=10.0).
                        // text("Bond Shear Limit")).changed() {
                        //     self.changed_collision_settings = true;
                        // };
                        if ui.checkbox(&mut self.bond_tearing, "Bond Tearing").changed() {
                            self.changed_collision_settings = true;
                        }
                        if self.bond_tearing {
                            if ui.add(egui::Slider::new(&mut self.bond_force_limit, 0.0..=5.0).step_by(0.0001).
                            text("Tear Limit")).changed() {
                                self.changed_collision_settings = true;
                            };
                        }
                    }
                    if ui.checkbox(&mut self.collisions, "Collisions").changed() {
                        self.changed_collision_settings = true;
                    }
                    if self.collisions {
                        if ui.add(egui::Slider::new(&mut self.friction_coefficient, 0.0..=1.0).
                        text("Friction Coef.")).changed() {
                            self.changed_collision_settings = true;
                        };
                    }
                });
            }          
            if self.menu.walls_menu {
                egui::Window::new("Walls").collapsible(false).auto_sized().show(ctx, |ui| {
                    ui.checkbox(&mut self.maintain_ar, "Maintain Aspect Ratio");
                        let ar = self.hor_bound/self.vert_bound;
                        if ui.add(egui::Slider::new(&mut self.hor_bound, 0.0..=64.0).
                            text("Width")).changed() {
                                self.changed_collision_settings = true;
                                if self.maintain_ar {
                                    self.vert_bound = self.hor_bound*1.0/ar;
                                }
                            };
                        if ui.add(egui::Slider::new(&mut self.vert_bound, 0.0..=64.0).
                            text("Height")).changed() {
                                self.changed_collision_settings = true;
                                if self.maintain_ar {
                                    self.hor_bound = self.vert_bound*ar;
                                }
                            };
                });
            }
            if self.menu.materials_menu { egui::Window::new("Materials").collapsible(false).auto_sized().show(ctx, |ui| {
                let materials_count = self.materials.len()/self.material_size;
                for i in 0..materials_count {
                    let mat_num = i+1;
                    egui::CollapsingHeader::new(format!("Material {mat_num}")).show(ui, |ui| {
                        if ui.add(egui::Slider::new(&mut self.materials[i*self.material_size + 0], 0.0..=1.0).text("Red")).changed() { self.materials_changed = true; };
                        if ui.add(egui::Slider::new(&mut self.materials[i*self.material_size + 1], 0.0..=1.0).text("Green")).changed() { self.materials_changed = true; };
                        if ui.add(egui::Slider::new(&mut self.materials[i*self.material_size + 2], 0.0..=1.0).text("Blue")).changed() { self.materials_changed = true; };
                        if ui.add(egui::Slider::new(&mut self.materials[i*self.material_size + 3], 0.001..=1000000.0).text("Density")).changed() { self.materials_changed = true; };
                        if ui.add(egui::Slider::new(&mut self.materials[i*self.material_size + 4], 0.001..=1000000.0).text("Normal Stiffness")).changed() { self.materials_changed = true; };
                        if ui.add(egui::Slider::new(&mut self.materials[i*self.material_size + 5], 0.001..=1000000.0).text("Shear Stiffness")).changed() { self.materials_changed = true; };
                    });
                }
            });}
            if self.menu.data_menu {
                egui::Window::new("Data").collapsible(false).resizable(true).show(ctx, |ui| {
                    ui.checkbox(&mut self.gather_data, "Gather Data");
                    let mut plot = egui::plot::Plot::new("physics plot").auto_bounds_x().auto_bounds_y().clamp_grid(true);
                    let button = egui::Button::new("Reset View");
                    egui::ComboBox::from_label("Property")
                            .selected_text(format!("{:?}", self.plotted_prop))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.plotted_prop, Property::X_Position, "X Position");
                                ui.selectable_value(&mut self.plotted_prop, Property::Y_Position, "Y Position");
                                ui.selectable_value(&mut self.plotted_prop, Property::Rotation, "Rotation");
                                ui.selectable_value(&mut self.plotted_prop, Property::X_Velocity, "X Velocity");
                                ui.selectable_value(&mut self.plotted_prop, Property::Y_Velocity, "Y Velocity");
                                ui.selectable_value(&mut self.plotted_prop, Property::Rotational_Velocity, "Rotational Velocity");
                                ui.selectable_value(&mut self.plotted_prop, Property::Data_1, "Data 1");
                                ui.selectable_value(&mut self.plotted_prop, Property::Data_2, "Data 2");
                                ui.selectable_value(&mut self.plotted_prop, Property::Data_3, "Data 3");
                                ui.selectable_value(&mut self.plotted_prop, Property::Data_4, "Data 4");
                                ui.selectable_value(&mut self.plotted_prop, Property::FPS, "FPS");
                            });
                    if ui.add(button).clicked() { plot = plot.reset() }
                    plot.show(ui, |plot_ui| {
                        match self.plotted_prop {
                            Property::X_Position => {plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(self.data.x_pos_data.to_owned())));},
                            Property::Y_Position => {plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(self.data.y_pos_data.to_owned())));},
                            Property::Rotation => {plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(self.data.rot_data.to_owned())));},
                            Property::X_Velocity => {plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(self.data.x_vel_data.to_owned())));},
                            Property::Y_Velocity => {plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(self.data.y_vel_data.to_owned())));},
                            Property::Rotational_Velocity => {plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(self.data.rot_vel_data.to_owned())));},
                            Property::Data_1 => {plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(self.data.data1.to_owned())));},
                            Property::Data_2 => {plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(self.data.data2.to_owned())));},
                            Property::Data_3 => {plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(self.data.data3.to_owned())));},
                            Property::Data_4 => {plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(self.data.data4.to_owned())));},
                            Property::FPS => {plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(self.data.fps.to_owned())));},
                        }
                    });
                });
            }
            if self.menu.save_load_menu {
                egui::Window::new("Save/Load").collapsible(false).auto_sized().show(ctx, |ui| {
                    if (ui.button("Load")).clicked() { self.load(); }
                    if (ui.button("Save")).clicked() { self.save(); }
                });
            }
        }
        return reset;   
    }

    pub fn updateBonds(&mut self) {
        self.bonds = match self.bondenum {
            BondType::Unbonded => { 0 },
            BondType::Normal_Bonds => { 1 },
            BondType::Linear_Contact_Bond => { 2 },
            BondType::Parallel_Linear_Contact_Bond => { 3 },
        }
    }

    pub fn load(&mut self) {
        let path = FileDialog::new()
            .set_location("~/OneDrive/Code/WASM/Engine Programs/Particle-Physics-Sim/saved_states")
            .add_filter("Binary File", &["bin"])
            .show_open_single_file()
            .unwrap();

        match path {
            Some(path) => {
                self.current_file = path.clone();
                self.load = true;
            },
            None => {},
        };
        
        // if !self.current_file.exists() {
        //     self.load = false;
        // }
    }
    
    pub fn save(&mut self) {
        let path = FileDialog::new()
            .set_location("~/OneDrive/Code/WASM/Engine Programs/Particle-Physics-Sim/saved_states")
            .add_filter("Binary File", &["bin"])
            .show_save_single_file()
            .unwrap();

        match path {
            Some(path) => {
                self.current_file = path.clone();
                self.save = true;
            },
            None => {},
        };

        // if !self.current_file.file_name().is_none() {
        //     self.save = false;
        // }
    }

    pub fn collison_settings(&mut self) -> Vec<f32> {
        self.changed_collision_settings = false;
        return vec![
            self.hor_bound,
            self.vert_bound,
            bytemuck::cast(self.gravity as i32),
            bytemuck::cast(self.planet_mode as i32),
            bytemuck::cast(self.bonds),
            bytemuck::cast(self.collisions as i32),
            bytemuck::cast(self.friction as i32),
            self.friction_coefficient,
            bytemuck::cast(self.rotation as i32),
            bytemuck::cast(self.linear_contact_bonds as i32),
            self.gravity_acceleration,
            self.stiffness,
            bytemuck::cast(self.bond_tearing as i32),
            self.bond_force_limit,
            self.contact_damping,
            self.bond_damping,
            self.drag,
            self.bond_shear_limit,
            bytemuck::cast(self.verlet as i32),
            self.timestep
        ];
    }

    pub fn render_settings(&mut self) -> Vec<i32> {
        return vec![
            self.circular_particles as i32,
            self.render_rot as i32,
            self.color_code_rot as i32,
            self.colors as i32,
            (self.bonds != 0 && self.render_bonds) as i32,
            self.hor_bound.to_bits() as i32,
            self.vert_bound.to_bits() as i32,
            self.stiffness.to_bits() as i32,
            self.random_colors as i32,
        ];
    }

    pub fn properties(&mut self) -> Vec<f32> {
        return vec![
            bytemuck::cast(self.properties.set_x_force as i32),
            bytemuck::cast(self.properties.set_y_force as i32),
            bytemuck::cast(self.properties.set_rot_force as i32),
            bytemuck::cast(self.properties.set_material as i32),
            bytemuck::cast(self.properties.set_x_fixity as i32),
            bytemuck::cast(self.properties.set_y_fixity as i32),
            bytemuck::cast(self.properties.set_rot_fixity as i32),
            bytemuck::cast(self.properties.set_x_pos as i32),
            bytemuck::cast(self.properties.set_y_pos as i32),
            bytemuck::cast(self.properties.set_rot as i32),
            bytemuck::cast(self.properties.set_x_vel as i32),
            bytemuck::cast(self.properties.set_y_vel as i32),
            bytemuck::cast(self.properties.set_rot_vel as i32),
            bytemuck::cast(self.properties.set_radius as i32),
            self.properties.x_force,
            self.properties.y_force,
            self.properties.rot_force,
            bytemuck::cast(self.properties.material as i32),
            bytemuck::cast(self.properties.x_fixity as i32),
            bytemuck::cast(self.properties.y_fixity as i32),
            bytemuck::cast(self.properties.rot_fixity as i32),
            self.properties.x_pos,
            self.properties.y_pos,
            self.properties.rot,
            self.properties.x_vel,
            self.properties.y_vel,
            self.properties.rot_vel,
            self.properties.radius,
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
    Mats
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
            1            => BondType::Normal_Bonds,
            2            => BondType::Linear_Contact_Bond,
            3..          => BondType::Parallel_Linear_Contact_Bond,
        }
    }
    pub fn as_i32(&self) -> i32 {
        return match *self {
            BondType::Unbonded                     => 0,
            BondType::Normal_Bonds                 => 1,
            BondType::Linear_Contact_Bond          => 2,
            BondType::Parallel_Linear_Contact_Bond => 3
        }
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
    Data_1,
    Data_2,
    Data_3,
    Data_4,
    FPS,
}