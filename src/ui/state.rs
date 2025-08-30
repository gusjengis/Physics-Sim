use egui::{Modifiers, Ui, Vec2};

use crate::settings::{Settings, Structure};

pub fn state_menu(settings: &mut Settings, ui: &mut Ui) {
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
            settings.backup = true;
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
            settings.restore = true;
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
            settings.reset = true;
            ui.close_menu();
        }
        ui.separator();
        ui.label("Setup");
        if ui
            .add(egui::Slider::new(&mut settings.setup.particles, 1..=settings.setup.workgroup_size * 200).text("Particles").step_by(1.0))
            .changed()
        {
            settings.setup.workgroups = (settings.setup.particles as f32 / settings.setup.workgroup_size as f32).ceil() as usize;
            settings.setup.grid_width = settings.setup.grid_width.min(settings.setup.particles as f32);
            settings.reset = true;
        };
        if settings.setup.structure == Structure::Grid {
            settings.reset |= ui
                .add(
                    egui::Slider::new(&mut settings.setup.grid_width, 1.0..=settings.setup.particles as f32)
                        .text("Grid Width")
                        .step_by(0.01)
                        .logarithmic(true),
                )
                .changed();
            settings.reset |= ui.checkbox(&mut settings.setup.hex_grid, "Hex Grid").changed();
        }

        settings.reset |= ui.checkbox(&mut settings.setup.variable_rad, "Random Radius").changed();

        if ui
            .add(egui::Slider::new(&mut settings.setup.max_radius, 0.000000001..=10.0).step_by(0.001).text("Max Radius"))
            .changed()
        {
            settings.setup.min_radius = settings.setup.max_radius / settings.setup.holeyness;
            settings.reset = true;
        }

        if settings.setup.variable_rad {
            match settings.setup.structure {
                Structure::Grid => {
                    if ui.add(egui::Slider::new(&mut settings.setup.holeyness, 1.0..=10.0).text("Holeyness")).changed() {
                        settings.setup.min_radius = settings.setup.max_radius / settings.setup.holeyness;
                        settings.reset = true;
                    };
                }
                _ => {
                    settings.reset |= ui.add(egui::Slider::new(&mut settings.setup.max_radius, 0.0001..=0.5).text("Max Radius")).changed();
                    settings.reset |= ui.add(egui::Slider::new(&mut settings.setup.min_radius, 0.0001..=0.5).text("Min Radius")).changed();
                }
            }
        }
        egui::CollapsingHeader::new("Initial Velocities").show(ui, |ui| {
            if ui.add(egui::Slider::new(&mut settings.setup.max_h_velocity, -10.0..=10.0).text("Max xV")).changed() {
                if settings.setup.max_h_velocity < settings.setup.min_h_velocity {
                    settings.setup.min_h_velocity = settings.setup.max_h_velocity;
                }
                settings.reset = true;
            };
            if ui.add(egui::Slider::new(&mut settings.setup.min_h_velocity, -10.0..=10.0).text("Min xV")).changed() {
                if settings.setup.max_h_velocity < settings.setup.min_h_velocity {
                    settings.setup.max_h_velocity = settings.setup.min_h_velocity;
                }
                settings.reset = true;
            };
            if ui.add(egui::Slider::new(&mut settings.setup.max_v_velocity, -10.0..=10.0).text("Max yV")).changed() {
                if settings.setup.max_v_velocity < settings.setup.min_v_velocity {
                    settings.setup.min_v_velocity = settings.setup.max_v_velocity;
                }
                settings.reset = true;
            };
            if ui.add(egui::Slider::new(&mut settings.setup.min_v_velocity, -10.0..=10.0).text("Min yV")).changed() {
                if settings.setup.max_v_velocity < settings.setup.min_v_velocity {
                    settings.setup.max_v_velocity = settings.setup.min_v_velocity;
                }
                settings.reset = true;
            };
        });
    });
}
