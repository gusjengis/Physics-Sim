use egui::{Color32, Ui, color_picker::Alpha};

use crate::settings::Settings;

pub fn materials_menu(settings: &mut Settings, ui: &mut Ui) {
    ui.menu_button("Materials", |ui| {
        ui.style_mut().wrap = Some(false);
        ui.set_max_width(83.0);

        let materials_count = settings.materials.len() / settings.material_size;
        for i in 0..materials_count {
            let mat_num = i;
            ui.menu_button(format!("Material {mat_num}"), |ui| {
                ui.set_min_width(250.0);
                ui.menu_button("Color", |ui| {
                    let mut color = Color32::from_rgb(
                        (settings.materials[i * settings.material_size + 0] * 255.0) as u8,
                        (settings.materials[i * settings.material_size + 1] * 255.0) as u8,
                        (settings.materials[i * settings.material_size + 2] * 255.0) as u8,
                    );
                    let color2 = color.clone();
                    egui::color_picker::color_picker_color32(ui, &mut color, Alpha::Opaque);
                    if color.r() != color2.r() || color.g() != color2.g() || color.b() != color2.b() {
                        settings.materials_changed = true;
                    }
                    let color_srgb = color.to_srgba_unmultiplied();
                    settings.materials[i * settings.material_size + 0] = color_srgb[0] as f32 / 255.0;
                    settings.materials[i * settings.material_size + 1] = color_srgb[1] as f32 / 255.0;
                    settings.materials[i * settings.material_size + 2] = color_srgb[2] as f32 / 255.0;
                    if ui.add(egui::Slider::new(&mut settings.materials[i * settings.material_size + 0], 0.0..=1.0)).changed() {
                        settings.materials_changed = true;
                    };
                    if ui.add(egui::Slider::new(&mut settings.materials[i * settings.material_size + 1], 0.0..=1.0)).changed() {
                        settings.materials_changed = true;
                    };
                    if ui.add(egui::Slider::new(&mut settings.materials[i * settings.material_size + 2], 0.0..=1.0)).changed() {
                        settings.materials_changed = true;
                    };
                });
                // if ui.add(egui::Slider::new(&mut settings.materials[i*settings.material_size + 0], 0.0..=1.0).text("Red")).changed() { settings.materials_changed = true; };
                // if ui.add(egui::Slider::new(&mut settings.materials[i*settings.material_size + 1], 0.0..=1.0).text("Green")).changed() { settings.materials_changed = true; };
                // if ui.add(egui::Slider::new(&mut settings.materials[i*settings.material_size + 2], 0.0..=1.0).text("Blue")).changed() { settings.materials_changed = true; };
                if ui
                    .add(egui::Slider::new(&mut settings.materials[i * settings.material_size + 3], -100000.0..=100000000000.0).text("Density"))
                    .changed()
                {
                    settings.materials_changed = true;
                };
                if ui
                    .add(egui::Slider::new(&mut settings.materials[i * settings.material_size + 4], -100000.0..=100000000000.0).text("Normal Stiffness"))
                    .changed()
                {
                    settings.materials_changed = true;
                };
                if ui
                    .add(egui::Slider::new(&mut settings.materials[i * settings.material_size + 5], -100000.0..=100000000000.0).text("Shear Stiffness"))
                    .changed()
                {
                    settings.materials_changed = true;
                };
            });
        }
        if ui.button("Add Material").clicked() {
            settings.materials.resize(settings.material_size + settings.materials.len(), 0.0);
            let base = settings.materials.len() - 6;
            settings.materials[base] = rand::random();
            settings.materials[base + 1] = rand::random();
            settings.materials[base + 2] = rand::random();
            settings.materials[base + 3] = settings.materials[3];
            settings.materials[base + 4] = settings.materials[4];
            settings.materials[base + 5] = settings.materials[5];
            settings.materials_changed = true;
        }
    });
}
