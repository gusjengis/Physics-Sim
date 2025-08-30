use egui::{Color32, DragValue, Modifiers, Ui, Vec2, color_picker::Alpha};

use crate::settings::{ColorSource, Settings};

pub fn view_menu(settings: &mut Settings, ui: &mut Ui) {
    let zoom_in_shortcut = egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::Plus);
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
            settings.zoom_in = true;
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
            settings.zoom_out = true;
            // ui.close_menu();
        }

        if ui.button("Fit Bounds").clicked() {
            settings.view.scale = 2.0 / settings.simulation.vert_bound;
        }

        if ui
            .add(egui::Button::new("Home").min_size(Vec2::new(min_x, 0.0)).shortcut_text(ui.ctx().format_shortcut(&home_shortcut)))
            .on_hover_text("Centers the view on (0,0).")
            .clicked()
        {
            settings.home = true;
            // ui.close_menu();
        }
        ui.add_enabled(false, egui::Button::new("Pan").min_size(Vec2::new(min_x, 0.0)).shortcut_text(format!("Shift + Drag")));
        ui.separator();
        ui.label("Rendering");
        // egui::Window::new("Render Settings").collapsible(false).auto_sized().show(ctx, |ui| {
        // ui.checkbox(&mut settings.view.rendering, "Render Particles");
        settings.rebuild_shaders |= ui.checkbox(&mut settings.view.circular_particles, "Circular Particles").changed();
        ui.add_enabled(settings.view.circular_particles, egui::Checkbox::new(&mut settings.view.render_outline, "Render Outline"));
        settings.rebuild_shaders |= ui.checkbox(&mut settings.view.render_rot, "Render Rotation").changed();
        ui.checkbox(&mut settings.view.render_unbonded_contacts, "Render Contacts");
        settings.rebuild_shaders |= ui.checkbox(&mut settings.view.render_bonds, "Render Bonds").changed();
        settings.rebuild_shaders |= ui.checkbox(&mut settings.view.lighting, "Lighting").changed();
        settings.rebuild_shaders |= ui.checkbox(&mut settings.simulation.d3, "3D").changed();
        ui.menu_button("Particle Color", |ui| {
            ui.label("Color Source:");
            ui.menu_button(format!("{}", settings.view.color_source.to_string()), |ui| {
                ui.selectable_value(&mut settings.view.color_source, ColorSource::None, "None");
                ui.selectable_value(&mut settings.view.color_source, ColorSource::Material, "Material");
                ui.selectable_value(&mut settings.view.color_source, ColorSource::Direction, "Direction");
                ui.selectable_value(&mut settings.view.color_source, ColorSource::Random, "Random");
            });
            settings.rebuild_shaders |= ui.checkbox(&mut settings.view.color_code_rot, "Color Code Rotation").changed();
            ui.checkbox(&mut settings.view.dim_slow_particles, "Dim Slow Particles");
            ui.add_enabled(
                settings.view.dim_slow_particles,
                egui::DragValue::new(&mut settings.view.max_brightness_vel)
                    .clamp_range(0.0001..=100.0)
                    .prefix("Dimming Threshold: ")
                    .suffix(" m/s")
                    .speed(0.01),
            );
        });
        ui.add_enabled_ui(settings.view.render_outline && settings.view.circular_particles, |ui| {
            ui.menu_button("Outline Color", |ui| {
                ui.checkbox(&mut settings.view.use_particle_color_outline, "Use Particle Color");
                ui.add_enabled_ui(!settings.view.use_particle_color_outline, |ui| {
                    let mut color = Color32::from_rgb(
                        (settings.view.outline_color[0] * 255.0) as u8,
                        (settings.view.outline_color[1] * 255.0) as u8,
                        (settings.view.outline_color[2] * 255.0) as u8,
                    );
                    egui::color_picker::color_picker_color32(ui, &mut color, Alpha::Opaque);
                    let color_srgb = color.to_srgba_unmultiplied();
                    settings.view.outline_color[0] = color_srgb[0] as f32 / 255.0;
                    settings.view.outline_color[1] = color_srgb[1] as f32 / 255.0;
                    settings.view.outline_color[2] = color_srgb[2] as f32 / 255.0;
                    ui.add(egui::Slider::new(&mut settings.view.outline_color[0], 0.0..=1.0));
                    ui.add(egui::Slider::new(&mut settings.view.outline_color[1], 0.0..=1.0));
                    ui.add(egui::Slider::new(&mut settings.view.outline_color[2], 0.0..=1.0));
                });
            });
        });
        ui.menu_button("Background Color", |ui| {
            let mut color = Color32::from_rgb(
                (settings.view.background_color[0] * 255.0) as u8,
                (settings.view.background_color[1] * 255.0) as u8,
                (settings.view.background_color[2] * 255.0) as u8,
            );
            egui::color_picker::color_picker_color32(ui, &mut color, Alpha::Opaque);
            // let color_srgb = color.to_srgba_unmultiplied();
            settings.view.background_color[0] = (color[0] as f32 / 255.0);
            settings.view.background_color[1] = (color[1] as f32 / 255.0);
            settings.view.background_color[2] = (color[2] as f32 / 255.0);
            ui.add(egui::Slider::new(&mut settings.view.background_color[0], 0.0..=1.0));
            ui.add(egui::Slider::new(&mut settings.view.background_color[1], 0.0..=1.0));
            ui.add(egui::Slider::new(&mut settings.view.background_color[2], 0.0..=1.0));
        });
        ui.menu_button("Post Processing", |ui| {
            // ui.label("CRT Effect");
            // ui.separator();
            ui.checkbox(&mut settings.view.sobel, "Sobel Filter");
            ui.add_enabled(settings.view.sobel, egui::Checkbox::new(&mut settings.view.colored_sobel, "Colored Sobel"));
            ui.checkbox(&mut settings.view.invert, "Invert Colors");
            // ui.horizontal(|ui|{
            //     ui.label("Render every");
            //     ui.add(DragValue::new(&mut settings.view.crt_res).clamp_range(1..=16));
            //     ui.label("lines.");
            // });
            ui.horizontal(|ui| {
                ui.checkbox(&mut settings.view.grain, "");
                ui.add_enabled_ui(settings.view.grain, |ui| {
                    ui.menu_button("Grain", |ui| {
                        ui.label("Size:");
                        ui.add(DragValue::new(&mut settings.view.grain_size).suffix("px").clamp_range(1..=8));
                        ui.label("Strength:");
                        ui.add(DragValue::new(&mut settings.view.grain_strength).clamp_range(0.0..=1.0).speed(0.001));
                    });
                });
            });
            ui.horizontal(|ui| {
                ui.checkbox(&mut settings.view.chrom_ab, "");
                ui.add_enabled_ui(settings.view.chrom_ab, |ui| {
                    ui.menu_button("Chromatic Aberation", |ui| {
                        ui.label("Offset Strength:");
                        ui.add(DragValue::new(&mut settings.view.abb_strength).clamp_range(0.0..=0.25).speed(0.001));
                    });
                });
            });
        });
        // });
    });
}
