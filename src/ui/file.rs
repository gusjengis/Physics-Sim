use egui::{Modifiers, Ui, Vec2};
use rfd::FileDialog;
use std::fs;

use crate::settings::Settings;

pub fn file_menu(settings: &mut Settings, ui: &mut Ui) {
    let load_shortcut = egui::KeyboardShortcut::new(Modifiers::CTRL, egui::Key::O);
    let save_shortcut = egui::KeyboardShortcut::new(Modifiers::CTRL, egui::Key::S);

    ui.menu_button("File", |ui| {
        ui.style_mut().wrap = Some(false);

        let min_x = 80.0;
        let min_y = 0.0;

        ui.menu_button("Load", |ui| {
            ui.style_mut().wrap = Some(false);

            let mut entries: Vec<(std::path::PathBuf, String)> = Vec::new();
            if let Ok(read_dir) = fs::read_dir(&settings.current_dir) {
                for entry in read_dir.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("bin") {
                        if let Some(stem) = path.clone().file_stem().and_then(|s| s.to_str()) {
                            entries.push((path, stem.to_string()));
                        }
                    }
                }
            } else {
                ui.label("Invalid Directory Path");
                return;
            }

            let font_id = egui::TextStyle::Button.resolve(ui.style());
            let max_text_px = ui.fonts(|fonts| {
                entries
                    .iter()
                    .map(|(_, name)| fonts.layout_no_wrap(name.clone(), font_id.clone(), egui::Color32::WHITE).size().x)
                    .fold(0.0, f32::max)
            });

            let padding = 24.0;
            ui.set_min_width(max_text_px + padding);

            egui::ScrollArea::vertical().auto_shrink([true, true]).show(ui, |ui| {
                for (path, display) in entries {
                    if ui.button(&display).clicked() {
                        settings.current_file = path;
                        settings.load = true;
                    }
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                    ui.separator();
                    if ui.button("Select Folder…").clicked() {
                        if let Some(path) = FileDialog::new().pick_folder() {
                            settings.current_dir = path;
                            settings.update_memory();
                        }
                    }
                }
            });
        });
        if ui
            .add(egui::Button::new("Save").min_size(Vec2::new(min_x, min_y)).shortcut_text(ui.ctx().format_shortcut(&save_shortcut)))
            .clicked()
        {
            settings.save();
            ui.close_menu();
        }
    });
}
