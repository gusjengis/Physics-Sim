use egui::{Context, TextEdit};

use crate::{settings::Settings, wgpu_config::WGPUConfig, wgpu_prog::WGPUProg};

pub fn code_editor(settings: &mut Settings, ctx: &Context, prog: &mut WGPUProg, config: &mut WGPUConfig) {
    let mut panel_width = 200.0; // Store this as a field in your struct

    if settings.view.code_editor {
        egui::SidePanel::left("code_editor").resizable(true).show(ctx, |ui| {
            ui.set_min_width(ui.available_width());
            ui.set_max_width(ui.available_width());
            ui.horizontal(|ui| {
                if ui.selectable_label(settings.curr_shader == 1, "Background").clicked() {
                    settings.curr_shader = 1;
                }
                if ui.selectable_label(settings.curr_shader == 0, "Particles").clicked() {
                    settings.curr_shader = 0;
                }
                if ui.selectable_label(settings.curr_shader == 2, "Hit Detection").clicked() {
                    settings.curr_shader = 2;
                }
                if ui.selectable_label(settings.curr_shader == 3, "Post Processing").clicked() {
                    settings.curr_shader = 3;
                }
                if ui.selectable_label(settings.curr_shader == 4, "Laws of Motion").clicked() {
                    settings.curr_shader = 4;
                }
                if ui.selectable_label(settings.curr_shader == 5, "Simulation").clicked() {
                    settings.curr_shader = 5;
                }
            });
            if settings.curr_shader < 4 {
                egui::ScrollArea::show(egui::ScrollArea::new([true, true]), ui, |ui| {
                    let text_edit = TextEdit::multiline(&mut prog.shader_strs[settings.curr_shader])
                        .desired_width(ui.available_width())
                        .font(egui::TextStyle::Monospace)
                        .code_editor();
                    if ui.add(text_edit).changed() {
                        prog.rebuild_pipeline(config, &settings, settings.curr_shader);
                        if settings.curr_shader == 2 {
                            prog.shader_strs[4] = prog.shader_strs[2].clone();
                            prog.rebuild_pipeline(config, &settings, 4);
                        }
                    }
                });
            } else {
                egui::ScrollArea::show(egui::ScrollArea::new([true, true]), ui, |ui| {
                    let text_edit = TextEdit::multiline(&mut prog.shader_prog.shader_strs[settings.curr_shader - 4])
                        .desired_width(ui.available_width())
                        .font(egui::TextStyle::Monospace)
                        .code_editor();
                    if ui.add(text_edit).changed() {
                        prog.shader_prog.rebuild_pipeline(config, &settings, settings.curr_shader - 4);
                    }
                });
            }
        });
    }
}
