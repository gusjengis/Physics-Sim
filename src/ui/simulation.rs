use egui::{Modifiers, Ui, Vec2};

use crate::settings::Settings;

pub fn simulation_menu(settings: &mut Settings, ui: &mut Ui) {
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
            settings.simulating = !settings.simulating;
        }
        ui.separator();
        ui.label(format!("Speed | {} ticks/frame", settings.simulation.gen_per_frame));
        // ui.menu_button("Speed", |ui| {
        let mut max_perc = settings.simulation.gen_per_frame as f32 / settings.simulation.max_gen_per_frame as f32 * 100.0;
        if settings.speed_perc != max_perc {
            settings.speed_perc = max_perc;
        }
        if ui
            .add(
                egui::Slider::new(&mut settings.speed_perc, 1.0 / settings.simulation.max_gen_per_frame as f32..=100.0).custom_formatter(|n, _| {
                    let n = n as i32;
                    format!("{n}%")
                }),
            )
            .changed()
        {
            settings.simulation.gen_per_frame = 1.max((settings.speed_perc / 100.0 * settings.simulation.max_gen_per_frame as f32) as i32);
        }; //.logarithmic(true);//.text(format!("Ticks/Frame ({:.0}/{:.0}%)", fps_perc, max_perc)).text_color(Color32::from_rgb((255.0*(1.0 - (settings.fps/settings.hz).clamp(0.0, 1.0))) as u8, (255.0*(settings.fps/settings.hz).clamp(0.0, 1.0)) as u8, 0)));

        if ui
            .add(egui::Button::new("Speed Up").min_size(Vec2::new(min_x, 0.0)).shortcut_text("Right Arrow"))
            .on_hover_text("Increase ticks/frame.")
            .clicked()
        {
            settings.simulation.gen_per_frame = settings.simulation.max_gen_per_frame.min(settings.simulation.gen_per_frame + 1);
        }

        if ui
            .add(egui::Button::new("Slow Down").min_size(Vec2::new(min_x, 0.0)).shortcut_text("Left Arrow"))
            .on_hover_text("Decrease ticks/frame.")
            .clicked()
        {
            settings.simulation.gen_per_frame = 1.max(settings.simulation.gen_per_frame - 1);
        }
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Advance").clicked() {
                settings.simulation.advance_x_timesteps = true;
            }
            ui.add(egui::DragValue::new(&mut settings.simulation.x_timesteps).speed(1).clamp_range(1..=i32::MAX));
            ui.label("timesteps");
        });
        ui.separator();
        if ui.checkbox(&mut settings.simulation.auto_timestep, "Auto Timestep").changed() {
            settings.update_critical_timestep = true;
        }
        ui.add_enabled_ui(!settings.simulation.auto_timestep, |ui| {
            ui.label(format!("Quality | {} ticks/s", (1.0 / settings.simulation.timestep).round() as i32));
            if ui
                .add(egui::Slider::new(&mut settings.simulation.timestep, 0.0000000001..=1.0 / settings.hz).logarithmic(true))
                .changed()
            {
                if settings.simulation.round_timestep {
                    settings.simulation.timestep = 1.0 / (((1.0 / settings.simulation.timestep as f32) / 120.0).ceil() * 120.0);
                }
                settings.changed_collision_settings = true;
            }
        });
        if ui.checkbox(&mut settings.simulation.round_timestep, "Round Timestep").changed() {
            if settings.simulation.round_timestep {
                settings.simulation.timestep = 1.0 / (((1.0 / settings.simulation.timestep as f32) / 120.0).ceil() * 120.0);
                settings.changed_collision_settings = true;
            }
        }
        if ui
            .checkbox(&mut settings.simulation.deterministic, "Deterministic")
            .on_hover_text("Provides consistent results, impacts performance.")
            .changed()
        {
            settings.rebuild_shaders = true;
        }
        ui.add_enabled_ui(settings.f64_support, |ui| {
            if settings.f64_support {
                if ui
                    .checkbox(&mut settings.simulation.use_f64, "64-bit precision")
                    .on_hover_text("Use f64s to calculate distance between particles.")
                    .clicked()
                {
                    settings.rebuild_shaders = true;
                }
            } else {
                ui.checkbox(&mut settings.simulation.use_f64, "64-bit precision").on_hover_text("Not supported by your GPU.");
            }
        });
        ui.separator();
        ui.label("Walls");
        // ui.checkbox(&mut settings.simulation.walls, "Walls");
        ui.add_enabled_ui(settings.simulation.walls, |ui| {
            if ui.checkbox(&mut settings.simulation.round_walls, "Circular Walls").changed() {
                settings.changed_collision_settings = true;
            }
            if settings.simulation.round_walls {
                if ui.add(egui::Slider::new(&mut settings.simulation.wall_radius, 0.0..=64.0).text("Radius")).changed() {
                    settings.changed_collision_settings = true;
                }
            } else {
                let ar = settings.simulation.hor_bound / settings.simulation.vert_bound;
                ui.checkbox(&mut settings.simulation.auto_width, "Auto Width");
                ui.add_enabled(!settings.simulation.auto_width, egui::Checkbox::new(&mut settings.simulation.maintain_ar, "Maintain Aspect Ratio"));
                if ui
                    .add_enabled(!settings.simulation.auto_width, egui::Slider::new(&mut settings.simulation.hor_bound, 0.0..=64.0).text("Width"))
                    .changed()
                {
                    settings.changed_collision_settings = true;
                    if settings.simulation.maintain_ar || settings.simulation.auto_width {
                        settings.simulation.vert_bound = settings.simulation.hor_bound * 1.0 / ar;
                    }
                }
                if ui.add(egui::Slider::new(&mut settings.simulation.vert_bound, 0.0..=64.0).text("Height")).changed() {
                    settings.changed_collision_settings = true;
                    if settings.simulation.maintain_ar || settings.simulation.auto_width {
                        settings.simulation.hor_bound = settings.simulation.vert_bound * ar;
                    }
                }
            }
        });
    });
}
