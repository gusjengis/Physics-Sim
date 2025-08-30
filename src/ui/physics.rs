use egui::Ui;

use crate::settings::{BondType, Settings};

pub fn physics_menu(settings: &mut Settings, ui: &mut Ui) {
    let min_y = 0.0;
    let mut min_x = match settings.physics.bondenum {
        BondType::Unbonded => 0.0,
        BondType::Normal_Bonds => 200.0,
        BondType::Linear_Contact_Bond => 200.0,
        BondType::Parallel_Linear_Contact_Bond => 320.0,
    };
    ui.menu_button("Physics", |ui| {
        ui.style_mut().wrap = Some(false);
        ui.set_min_width(min_x);
        settings.changed_collision_settings |= ui.checkbox(&mut settings.physics.gravity, "Gravity").changed();
        ui.add_enabled_ui(settings.physics.gravity, |ui| {
            settings.changed_collision_settings |= ui.checkbox(&mut settings.physics.planet_mode, "Planet Mode").changed();
            ui.add_enabled_ui(settings.physics.planet_mode, |ui| {
                settings.changed_collision_settings |= ui.checkbox(&mut settings.physics.mouse_gravity, "Mouse Gravity").changed();
            });
            ui.label("G Force");
            settings.changed_collision_settings |= ui.add(egui::Slider::new(&mut settings.physics.gravity_acceleration, -100.0..=100.0).step_by(0.1)).changed();
        });
        ui.separator();
        settings.changed_collision_settings |= ui.checkbox(&mut settings.physics.collisions, "Collisions").changed();
        ui.add_enabled_ui(settings.physics.collisions, |ui| {
            ui.label("Collision Interval");
            settings.changed_collision_settings |= ui.add(egui::Slider::new(&mut settings.physics.collision_interval, 1..=settings.simulation.max_gen_per_frame)).changed();
            ui.label("Friction Coefficient");
            settings.changed_collision_settings |= ui.add(egui::Slider::new(&mut settings.physics.friction_coefficient, 0.0..=1.0)).changed();
        });
        ui.separator();
        settings.changed_collision_settings |= ui.checkbox(&mut settings.physics.local_damping, "Local Damping").changed();
        ui.add_enabled_ui(settings.physics.local_damping, |ui| {
            settings.changed_collision_settings |= ui.add(egui::Slider::new(&mut settings.physics.local_damping_alpha, 0.0..=1.0)).changed();
        });
        ui.separator();
        let mut changed_bonds = false;

        ui.menu_button(format!("{}", settings.physics.bondenum.to_string()), |ui| {
            changed_bonds |= ui.selectable_value(&mut settings.physics.bondenum, BondType::Unbonded, "Unbonded").changed();
            changed_bonds |= ui.selectable_value(&mut settings.physics.bondenum, BondType::Normal_Bonds, "Normal Bonds").changed();
            changed_bonds |= ui.selectable_value(&mut settings.physics.bondenum, BondType::Linear_Contact_Bond, "Linear Contact Bonds").changed();
            changed_bonds |= ui
                .selectable_value(&mut settings.physics.bondenum, BondType::Parallel_Linear_Contact_Bond, "Linear Parallel Bonds")
                .changed();
        });

        if changed_bonds {
            settings.changed_collision_settings = true;
            settings.updateBonds();
        }

        if settings.physics.bonds != 0 {
            ui.separator();
            ui.label("Stiffness");
            settings.changed_collision_settings |= ui
                .add(egui::Slider::new(&mut settings.physics.bond_normal_stiffness, 0.001..=1000000000000.0).step_by(0.001).text("Normal"))
                .changed();
            if settings.physics.bonds > 1 {
                settings.changed_collision_settings |= ui
                    .add(egui::Slider::new(&mut settings.physics.bond_shear_stiffness, 0.001..=1000000000000.0).step_by(0.001).text("Shear"))
                    .changed();
            }
            settings.changed_collision_settings |= ui.checkbox(&mut settings.physics.bond_tearing, "Bond Tearing").changed();
            ui.add_enabled_ui(settings.physics.bond_tearing, |ui| {
                ui.separator();
                ui.label("Strength");
                settings.changed_collision_settings |= ui
                    .add(egui::Slider::new(&mut settings.physics.bond_normal_strength, 0.0..=1000000000.0).step_by(0.0001).text("Normal"))
                    .changed();
                if settings.physics.bonds > 1 {
                    settings.changed_collision_settings |= ui
                        .add(egui::Slider::new(&mut settings.physics.bond_shear_strength, 0.0..=1000000000.0).step_by(0.0001).text("Shear"))
                        .changed();
                }
                if settings.physics.bonds > 2 {
                    settings.changed_collision_settings |= ui
                        .add(
                            egui::Slider::new(&mut settings.physics.moment_contribution_factor, 0.0..=1.0)
                                .step_by(0.0001)
                                .text("Moment Contribution Factor"),
                        )
                        .changed();
                }
            });
        }
        settings.regen_bonds = ui.button("Regenerate Bonds").clicked();
    });
}
