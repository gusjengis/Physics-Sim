use egui::{Context, Ui};

use crate::settings::Settings;

pub fn developer_menu(settings: &mut Settings, ui: &mut Ui, ctx: &Context) {
    ui.menu_button("Developer", |ui| {
        ui.style_mut().wrap = Some(false);
        ui.label("Debug");
        ui.checkbox(&mut settings.view.render_bp_grid, "Render Grid");
        ui.checkbox(&mut settings.view.show_hit_tex, "Show Hit Texture");
        ui.separator();
        ui.label("Experimental");
        if settings.create.create_mode {
            if ui.button("Create Mode").highlight().clicked() {
                settings.toggle_create();
            }
        } else {
            if ui.button("Create Mode").clicked() {
                settings.toggle_create();
            }
        }
        if ui.button("Particle Definitions").clicked() {
            settings.create.p_def_menu = !settings.create.p_def_menu;
        }
        ui.separator();
        // ui.label("Experimental");

        if ui.selectable_label(settings.view.code_editor, "Code Editor").clicked() {
            settings.view.code_editor = !settings.view.code_editor;
        }
        ui.separator();
        ui.label("Performance");
        if ui.checkbox(&mut settings.contact_search_optimization, "CSO").changed() {
            settings.rebuild_shaders = true;
        }
        // ui.separator();
        // ui.label("Audio");
        // if ui.selectable_label(settings.view.audio.menu, "Sounds").clicked() {
        //     settings.view.audio.menu = !settings.view.audio.menu;
        // }
        // if ui.selectable_label(settings.view.audio.waveform, "Waveform").clicked() {
        //     settings.view.audio.waveform = !settings.view.audio.waveform;
        // }
        // if ui.selectable_label(settings.view.audio.timeline_menu, "Timeline").clicked() {
        //     settings.view.audio.timeline_menu = !settings.view.audio.timeline_menu;
        // }
    });
}
