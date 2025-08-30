use std::f32::consts::PI;

use egui::{Context, Modifiers, Ui, Vec2};

use crate::{particle_def::Particle_Definition, settings::Settings};

pub fn particle_menu(settings: &mut Settings, ui: &mut Ui) {
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
            settings.select_all = true;
            ui.close_menu();
        }

        // ui.menu_button("Groups", |ui| {
        //     for i in 0..settings.groups {
        //         ui.horizontal(|ui| {
        //             ui.label(format!("Group {}", i + 1));
        //             if ui.button("Set").clicked() {
        //                 settings.set_group = i;
        //             }
        //         });
        //         // ui.selectable_label(settings.set_group >= 0, text);
        //     }
        //     if ui.button("New Group").clicked() {
        //         settings.groups += 1;
        //     }
        // });

        ui.add_enabled_ui(false, |ui| {
            ui.add(egui::Button::new("Translate").min_size(Vec2::new(min_x, min_y)).shortcut_text("Click + Drag"));
        });

        if ui
            .add(egui::Button::new("Fix").min_size(Vec2::new(min_x, min_y)).shortcut_text("F"))
            .on_hover_text("Fix selected particles.")
            .clicked()
        {
            settings.fix = true;
            ui.close_menu();
        }

        if ui
            .add(egui::Button::new("Drop").min_size(Vec2::new(min_x, min_y)).shortcut_text("D"))
            .on_hover_text("Unfix selected particles.")
            .clicked()
        {
            settings.drop = true;
            ui.close_menu();
        }

        ui.separator();
        ui.label("Properties");

        ui.horizontal(|inner_ui| {
            inner_ui.vertical(|inner_ui2| {
                inner_ui2.label("Position");
                inner_ui2.horizontal(|inner_ui3| {
                    inner_ui3.checkbox(&mut settings.properties.set_x_pos, "");
                    inner_ui3.add_enabled_ui(settings.properties.set_x_pos, |inner_ui4| {
                        inner_ui4.add(egui::DragValue::new(&mut settings.properties.x_pos).speed(0.0000001).clamp_range(f32::MIN..=f32::MAX));
                    });
                    inner_ui3.label("X Position");
                });
                inner_ui2.horizontal(|inner_ui3| {
                    inner_ui3.checkbox(&mut settings.properties.set_y_pos, "");
                    inner_ui3.add_enabled_ui(settings.properties.set_y_pos, |inner_ui4| {
                        inner_ui4.add(egui::DragValue::new(&mut settings.properties.y_pos).speed(0.0000001).clamp_range(f32::MIN..=f32::MAX));
                    });
                    inner_ui3.label("Y Position");
                });
                inner_ui2.horizontal(|inner_ui3| {
                    inner_ui3.checkbox(&mut settings.properties.set_rot, "");
                    inner_ui3.add_enabled_ui(settings.properties.set_rot, |inner_ui4| {
                        inner_ui4.add(egui::DragValue::new(&mut settings.properties.rot).speed(0.0000001).clamp_range(0.0..=PI * 2.0));
                    });
                    inner_ui3.label("Rotation");
                });
                inner_ui2.label("Velocity");
                inner_ui2.horizontal(|inner_ui3| {
                    inner_ui3.checkbox(&mut settings.properties.set_x_vel, "");
                    inner_ui3.add_enabled_ui(settings.properties.set_x_vel, |inner_ui4| {
                        inner_ui4.add(egui::DragValue::new(&mut settings.properties.x_vel).speed(0.001).clamp_range(f32::MIN..=f32::MAX));
                    });
                    inner_ui3.label("X Velocity");
                });
                inner_ui2.horizontal(|inner_ui3| {
                    inner_ui3.checkbox(&mut settings.properties.set_y_vel, "");
                    inner_ui3.add_enabled_ui(settings.properties.set_y_vel, |inner_ui4| {
                        inner_ui4.add(egui::DragValue::new(&mut settings.properties.y_vel).speed(0.001).clamp_range(f32::MIN..=f32::MAX));
                    });
                    inner_ui3.label("Y Velocity");
                });
                inner_ui2.horizontal(|inner_ui3| {
                    inner_ui3.checkbox(&mut settings.properties.set_rot_vel, "");
                    inner_ui3.add_enabled_ui(settings.properties.set_rot_vel, |inner_ui4| {
                        inner_ui4.add(egui::DragValue::new(&mut settings.properties.rot_vel).speed(0.001).clamp_range(f32::MIN..=f32::MAX));
                    });
                    inner_ui3.label("Rotational Velocity");
                });
                inner_ui2.label("Forces");
                inner_ui2.horizontal(|inner_ui3| {
                    inner_ui3.checkbox(&mut settings.properties.set_x_force, "");
                    inner_ui3.add_enabled_ui(settings.properties.set_x_force, |inner_ui4| {
                        inner_ui4.add(egui::DragValue::new(&mut settings.properties.x_force).speed(0.01));
                    });
                    inner_ui3.label("X Force");
                });
                inner_ui2.horizontal(|inner_ui3| {
                    inner_ui3.checkbox(&mut settings.properties.set_y_force, "");
                    inner_ui3.add_enabled_ui(settings.properties.set_y_force, |inner_ui4| {
                        inner_ui4.add(egui::DragValue::new(&mut settings.properties.y_force).speed(0.01));
                    });
                    inner_ui3.label("Y Force");
                });
                inner_ui2.horizontal(|inner_ui3| {
                    inner_ui3.checkbox(&mut settings.properties.set_rot_force, "");
                    inner_ui3.add_enabled_ui(settings.properties.set_rot_force, |inner_ui4| {
                        inner_ui4.add(egui::DragValue::new(&mut settings.properties.rot_force).speed(0.01));
                    });
                    inner_ui3.label("Rotational Force");
                });
                inner_ui2.label("Radius");
                inner_ui2.horizontal(|inner_ui3| {
                    inner_ui3.checkbox(&mut settings.properties.set_radius, "");
                    inner_ui3.add_enabled_ui(settings.properties.set_radius, |inner_ui4| {
                        inner_ui4.add(egui::DragValue::new(&mut settings.properties.radius).speed(0.001).clamp_range(0.0..=f32::MAX));
                    });
                });
                inner_ui2.label("Fixity");
                inner_ui2.horizontal(|inner_ui3| {
                    inner_ui3.checkbox(&mut settings.properties.set_x_fixity, "");
                    inner_ui3.add_enabled_ui(settings.properties.set_x_fixity, |inner_ui4| {
                        if inner_ui4
                            .add(egui::SelectableLabel::new(
                                settings.properties.x_fixity,
                                match settings.properties.x_fixity {
                                    true => "True",
                                    false => "False",
                                },
                            ))
                            .clicked()
                        {
                            settings.properties.x_fixity = !settings.properties.x_fixity;
                        };
                    });
                    inner_ui3.label("X Fixity");
                });
                inner_ui2.horizontal(|inner_ui3| {
                    inner_ui3.checkbox(&mut settings.properties.set_y_fixity, "");
                    inner_ui3.add_enabled_ui(settings.properties.set_y_fixity, |inner_ui4| {
                        if inner_ui4
                            .add(egui::SelectableLabel::new(
                                settings.properties.y_fixity,
                                match settings.properties.y_fixity {
                                    true => "True",
                                    false => "False",
                                },
                            ))
                            .clicked()
                        {
                            settings.properties.y_fixity = !settings.properties.y_fixity;
                        };
                    });
                    inner_ui3.label("Y Fixity");
                });
                inner_ui2.horizontal(|inner_ui3| {
                    inner_ui3.checkbox(&mut settings.properties.set_rot_fixity, "");
                    inner_ui3.add_enabled_ui(settings.properties.set_rot_fixity, |inner_ui4| {
                        if inner_ui4
                            .add(egui::SelectableLabel::new(
                                settings.properties.rot_fixity,
                                match settings.properties.rot_fixity {
                                    true => "True",
                                    false => "False",
                                },
                            ))
                            .clicked()
                        {
                            settings.properties.rot_fixity = !settings.properties.rot_fixity;
                        };
                    });
                    inner_ui3.label("Rotational Fixity");
                });
                inner_ui2.label("Material");
                inner_ui2.horizontal(|inner_ui3| {
                    inner_ui3.checkbox(&mut settings.properties.set_material, "");
                    inner_ui3.add_enabled_ui(settings.properties.set_material, |inner_ui4| {
                        // inner_ui4.add(egui::DragValue::new(&mut settings.properties.material).clamp_range(0..=(settings.materials.len()/settings.material_size - 1)));
                        inner_ui4.add(egui::Slider::new(&mut settings.properties.material, 0..=(settings.materials.len() / settings.material_size - 1) as i32));
                    });
                });
                if inner_ui2
                    .add_enabled(
                        settings.properties.set_material
                            || settings.properties.set_x_pos
                            || settings.properties.set_y_pos
                            || settings.properties.set_rot
                            || settings.properties.set_x_vel
                            || settings.properties.set_y_vel
                            || settings.properties.set_rot_vel
                            || settings.properties.set_radius
                            || settings.properties.set_rot_fixity
                            || settings.properties.set_rot_force
                            || settings.properties.set_x_fixity
                            || settings.properties.set_x_force
                            || settings.properties.set_y_fixity
                            || settings.properties.set_y_force,
                        egui::Button::new("Set Properties"),
                    )
                    .clicked()
                {
                    settings.set_properties = !settings.set_properties;
                }
            });
        });
        //if ui.selectable_label(settings.menu.properties_menu, "Properties").clicked() { settings.menu.properties_menu = !settings.menu.properties_menu; }
    });
}

pub fn p_def_menu(settings: &mut Settings, ctx: &Context) {
    egui::Window::new("Particle Definitions").collapsible(false).resizable(false).show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                if ui
                    .add(
                        egui::DragValue::new(&mut settings.create.particle_defs[settings.create.current_particle].radius)
                            .clamp_range(0.0..=10.0)
                            .prefix("Radius: ")
                            .suffix(" m")
                            .speed(0.001),
                    )
                    .changed()
                {
                    settings.create.particle_defs[settings.create.current_particle].new_radius();
                    settings.create.new_preview = true
                }
                if ui
                    .checkbox(&mut settings.create.particle_defs[settings.create.current_particle].random_radius, "Random Radii")
                    .changed()
                {
                    settings.create.particle_defs[settings.create.current_particle].new_radius();
                    settings.create.new_preview = true;
                }
                if settings.create.particle_defs[settings.create.current_particle].random_radius {
                    let min = settings.create.particle_defs[settings.create.current_particle].min_radius.clone();
                    let max = settings.create.particle_defs[settings.create.current_particle].max_radius.clone();
                    if ui
                        .add(egui::Slider::new(&mut settings.create.particle_defs[settings.create.current_particle].min_radius, 0.0..=max))
                        .changed()
                    {
                        settings.create.particle_defs[settings.create.current_particle].new_radius();
                        settings.create.new_preview = true;
                    }
                    if ui
                        .add(egui::Slider::new(&mut settings.create.particle_defs[settings.create.current_particle].max_radius, min..=10.0))
                        .changed()
                    {
                        settings.create.particle_defs[settings.create.current_particle].new_radius();
                        settings.create.new_preview = true;
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
                                egui::DragValue::new(&mut settings.create.particle_defs[settings.create.current_particle].x_vel)
                                    .speed(0.001)
                                    .clamp_range(f32::MIN..=f32::MAX),
                            );
                            inner_ui3.label("X Velocity");
                        });
                        inner_ui2.horizontal(|inner_ui3| {
                            inner_ui3.add(
                                egui::DragValue::new(&mut settings.create.particle_defs[settings.create.current_particle].y_vel)
                                    .speed(0.001)
                                    .clamp_range(f32::MIN..=f32::MAX),
                            );
                            inner_ui3.label("Y Velocity");
                        });
                        inner_ui2.horizontal(|inner_ui3| {
                            inner_ui3.add(
                                egui::DragValue::new(&mut settings.create.particle_defs[settings.create.current_particle].rot_vel)
                                    .speed(0.001)
                                    .clamp_range(f32::MIN..=f32::MAX),
                            );
                            inner_ui3.label("Rotational Velocity");
                        });
                        inner_ui2.label("Forces");
                        inner_ui2.horizontal(|inner_ui3| {
                            inner_ui3.add(egui::DragValue::new(&mut settings.create.particle_defs[settings.create.current_particle].x_force).speed(0.01));
                            inner_ui3.label("X Force");
                        });
                        inner_ui2.horizontal(|inner_ui3| {
                            inner_ui3.add(egui::DragValue::new(&mut settings.create.particle_defs[settings.create.current_particle].y_force).speed(0.01));
                            inner_ui3.label("Y Force");
                        });
                        inner_ui2.horizontal(|inner_ui3| {
                            inner_ui3.add(egui::DragValue::new(&mut settings.create.particle_defs[settings.create.current_particle].rot_force).speed(0.01));
                            inner_ui3.label("Rotational Force");
                        });
                        inner_ui2.label("Radius");
                        inner_ui2.horizontal(|inner_ui3| {
                            inner_ui3.add(
                                egui::DragValue::new(&mut settings.create.particle_defs[settings.create.current_particle].radius)
                                    .speed(0.001)
                                    .clamp_range(0.0..=f32::MAX),
                            );
                        });
                        inner_ui2.label("Fixity");
                        inner_ui2.horizontal(|inner_ui3| {
                            if inner_ui3
                                .add(egui::SelectableLabel::new(
                                    settings.create.particle_defs[settings.create.current_particle].x_fixity,
                                    match settings.create.particle_defs[settings.create.current_particle].x_fixity {
                                        true => "True",
                                        false => "False",
                                    },
                                ))
                                .clicked()
                            {
                                settings.create.particle_defs[settings.create.current_particle].x_fixity = !settings.create.particle_defs[settings.create.current_particle].x_fixity;
                            }
                            inner_ui3.label("X Fixity");
                        });
                        inner_ui2.horizontal(|inner_ui3| {
                            if inner_ui3
                                .add(egui::SelectableLabel::new(
                                    settings.create.particle_defs[settings.create.current_particle].y_fixity,
                                    match settings.create.particle_defs[settings.create.current_particle].y_fixity {
                                        true => "True",
                                        false => "False",
                                    },
                                ))
                                .clicked()
                            {
                                settings.create.particle_defs[settings.create.current_particle].y_fixity = !settings.create.particle_defs[settings.create.current_particle].y_fixity;
                            };

                            inner_ui3.label("Y Fixity");
                        });
                        inner_ui2.horizontal(|inner_ui3| {
                            if inner_ui3
                                .add(egui::SelectableLabel::new(
                                    settings.create.particle_defs[settings.create.current_particle].rot_fixity,
                                    match settings.create.particle_defs[settings.create.current_particle].rot_fixity {
                                        true => "True",
                                        false => "False",
                                    },
                                ))
                                .clicked()
                            {
                                settings.create.particle_defs[settings.create.current_particle].rot_fixity = !settings.create.particle_defs[settings.create.current_particle].rot_fixity;
                            };
                            inner_ui3.label("Rotational Fixity");
                        });
                        inner_ui2.label("Material");
                        inner_ui2.horizontal(|inner_ui3| {
                            inner_ui3.add(egui::Slider::new(
                                &mut settings.create.particle_defs[settings.create.current_particle].material,
                                0..=(settings.materials.len() / settings.material_size - 1) as i32,
                            ));
                        });
                    });
                });
            });
            ui.vertical_centered_justified(|ui| {
                for (i, p_def) in settings.create.particle_defs.iter().enumerate() {
                    if i == settings.create.current_particle {
                        if ui.button(p_def.name.clone()).highlight().clicked() {
                            settings.create.current_particle = i;
                        }
                    } else {
                        if ui.button(p_def.name.clone()).clicked() {
                            settings.create.current_particle = i;
                        }
                    }
                }
                if ui.button("+").clicked() {
                    settings.create.particle_defs.push(Particle_Definition::default());
                }
            });
        });
    });
}
