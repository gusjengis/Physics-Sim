mod data;
mod developer;
mod editor;
mod file;
mod materials;
mod particles;
mod physics;
mod script;
mod simulation;
mod state;
mod view;

use egui::{Align, Context, containers::menu::MenuConfig};

use crate::{
    scripts::ScriptManager,
    settings::Settings,
    ui::{
        data::data_menu,
        developer::developer_menu,
        editor::code_editor,
        file::file_menu,
        materials::materials_menu,
        particles::{p_def_menu, particle_menu},
        physics::physics_menu,
        script::{script_menu, script_panel},
        simulation::simulation_menu,
        state::state_menu,
        view::view_menu,
    },
    wgpu_config::WGPUConfig,
    wgpu_prog::WGPUProg,
};

pub fn main(settings: &mut Settings, ctx: &Context, prog: &mut WGPUProg, script_manager: &mut ScriptManager, config: &mut WGPUConfig, window_size: (u32, u32)) -> bool {
    //, ac: &mut AudioController) -> bool {
    let mut reset = false;
    if !settings.current_file.exists() && settings.save {
        settings.save();
    }
    if settings.recording && settings.start_time + settings.recording_duration < settings.sim_time {
        settings.gather_data = false;
        settings.recording = false;
    }
    if settings.view.settings_menu && !settings.export_screenshot {
        egui::TopBottomPanel::top("Settings Menu").show(ctx, |ui| {
            // ui.heading("Menu");
            egui::MenuBar::new()
                .config(MenuConfig::new().close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside))
                .ui(ui, |ui| {
                    ui.horizontal_centered(|ui| {
                        file_menu(settings, ui);
                        // settings.edit_menu(ctx, ui);
                        view_menu(settings, ui);
                        state_menu(settings, ui);
                        simulation_menu(settings, ui);
                        physics_menu(settings, ui);
                        particle_menu(settings, ui);
                        materials_menu(settings, ui);
                        data_menu(settings, ui, ctx);
                        script_menu(settings, ui, ctx);
                        developer_menu(settings, ui, ctx);
                    });
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        let max_perc = settings.simulation.gen_per_frame as f32 / settings.simulation.max_gen_per_frame as f32 * 100.0;
                        let mut fps_perc = max_perc * settings.fps / settings.hz;
                        if !settings.simulating {
                            fps_perc = 0.0;
                        }
                        ui.add(egui::Label::new(format!("{:.0}/{:.0}%", fps_perc, max_perc))).on_hover_text("Actual/Target simulation speed.");
                    });
                });
        });
        if settings.create.p_def_menu {
            p_def_menu(settings, ctx);
        }
        script_panel(settings, ctx, script_manager, prog, &mut config.device, &mut config.queue);
        code_editor(settings, ctx, prog, config);

        // settings.audio_menu(ctx, ac);
        // settings.waveform_menu(ctx, ac);
        // settings.timeline_menu(ctx, ac);
    }
    if settings.simulation.auto_width && !settings.export_screenshot {
        settings.simulation.hor_bound = settings.simulation.vert_bound * ctx.available_rect().width() as f32 / ctx.available_rect().height() as f32;
        settings.changed_collision_settings = true;
    }

    return reset;
}
