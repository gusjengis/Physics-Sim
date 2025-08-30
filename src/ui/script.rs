use egui::{Align, Context, Ui};
use wgpu::{Device, Queue};

use crate::{
    scripts::{Action, Command, Key, ScriptManager, Trigger},
    settings::Settings,
    wgpu_prog::WGPUProg,
};

pub fn script_menu(settings: &mut Settings, ui: &mut Ui, ctx: &Context) {
    ui.menu_button("Scripts", |ui| {
        ui.style_mut().wrap = Some(false);
        if ui.selectable_label(settings.view.script_menu, "Script Panel").clicked() {
            settings.view.script_menu = !settings.view.script_menu;
        }
    });
}

pub fn script_panel(settings: &mut Settings, ctx: &Context, script_manager: &mut ScriptManager, prog: &mut WGPUProg, device: &mut Device, queue: &mut Queue) {
    if settings.view.script_menu {
        egui::SidePanel::right("script_panel").resizable(true).show(ctx, |ui| {
            // egui::menu::bar(ui, |ui|{});
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut script_manager.scripts[settings.current_script].name);
                let delete_button = ui.button("Delete");
                if delete_button.clicked() {
                    ui.memory_mut(|mem| mem.toggle_popup(format!("{}_delete", settings.current_script).into()));
                }
                // if script_manager.delete_window[settings.current_script] {
                egui::popup_below_widget(
                    ui,
                    format!("{}_delete", settings.current_script).into(),
                    &delete_button,
                    egui::PopupCloseBehavior::CloseOnClickOutside,
                    |ui2: &mut Ui| {
                        // ui2.set_min_width(100.0);
                        // ui2.label(format!("Delete {}?", script_manager.scripts[settings.current_script].name));
                        ui2.horizontal(|ui3| {
                            if ui3.button("Delete").clicked() {
                                script_manager.delete_script(settings.current_script);
                                if settings.current_script == script_manager.scripts.len() {
                                    settings.current_script -= 1;
                                }
                            }
                            if ui3.button("Cancel").clicked() {
                                ui.memory_mut(|mem| mem.toggle_popup(format!("{}_delete", settings.current_script).into()));
                            }
                        });
                    },
                );
                // }
                if ui.selectable_label(script_manager.threads[settings.current_script].executing, "Run").clicked() {
                    script_manager.toggle_execution(settings.current_script);
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                for i in 0..script_manager.scripts.len() {
                    if ui.selectable_label(settings.current_script == i, script_manager.scripts[i].name.as_str()).clicked() {
                        settings.current_script = i;
                    }
                }
                if ui.button("+").clicked() {
                    script_manager.new_script(format!("Script {}", script_manager.scripts.len() + 1).as_str());
                    settings.current_script = script_manager.scripts.len() - 1;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Import").clicked() {
                        script_manager.import_scripts();
                    }
                    if ui.button("Export All").clicked() {
                        script_manager.export_scripts();
                    }
                });
            });
            ui.separator();
            ui.horizontal(|ui| {
                if ui.selectable_label(settings.json_scripts, "JSON").clicked() {
                    settings.json_scripts = !settings.json_scripts;
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.heading("Actions");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Export All").clicked() {
                        script_manager.export_scripts();
                    }
                });
            });
            ui.separator();
            if !settings.json_scripts {
                ui.horizontal(|ui| {
                    ui.label("Trigger");
                    egui::ComboBox::new("Trigger", "")
                        .selected_text(format!("{}", script_manager.scripts[settings.current_script].script_trigger.to_string()))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut script_manager.scripts[settings.current_script].script_trigger, Trigger::None, "None");
                            ui.selectable_value(&mut script_manager.scripts[settings.current_script].script_trigger, Trigger::Click, "Click");
                            ui.selectable_value(&mut script_manager.scripts[settings.current_script].script_trigger, Trigger::KeyDown(Key::Null), "KeyDown");
                            ui.selectable_value(&mut script_manager.scripts[settings.current_script].script_trigger, Trigger::KeyPressed(Key::Null), "KeyPressed");
                        });
                    match script_manager.scripts[settings.current_script].script_trigger {
                        Trigger::KeyDown(key) | Trigger::KeyPressed(key) => {
                            let mut k = script_manager.scripts[settings.current_script].script_trigger.keycode();
                            egui::ComboBox::new("Key", "").selected_text(format!("{:?}", k)).show_ui(ui, |ui| {
                                ui.selectable_value(&mut k, Key::Space, "Space");
                                ui.selectable_value(&mut k, Key::W, "W");
                                ui.selectable_value(&mut k, Key::A, "A");
                                ui.selectable_value(&mut k, Key::S, "S");
                                ui.selectable_value(&mut k, Key::D, "D");
                            });
                            script_manager.scripts[settings.current_script].script_trigger.set_key(k);
                        }
                        _ => {}
                    }

                    ui.checkbox(&mut script_manager.scripts[settings.current_script].auto_run, "Auto-Run")
                        .on_hover_text("Auto-run when script is loaded.");
                });
                ui.separator();
                egui::ScrollArea::new([false, true]).show(ui, |ui| {
                    if script_manager.scripts.len() > 0 {
                        let mut i = 0;
                        while i < script_manager.scripts[settings.current_script].actions.len() {
                            ui.horizontal(|ui| {
                                let current_digits = ((i + 1) as f32).log(10.0) as i32;
                                let max_digits = (script_manager.scripts[settings.current_script].actions.len() as f32).log(10.0) as i32;
                                let spaces = (max_digits - current_digits) * 2;
                                let mut space_string = format!("");
                                for j in 0..spaces {
                                    space_string.push(' ');
                                }
                                ui.label(format!("{space_string}{}", i + 1));
                                let mut changed_action = false;
                                let action_index = i;
                                egui::ComboBox::new(format!("{}", i).as_str(), "")
                                    .selected_text(format!("{}", script_manager.scripts[settings.current_script].actions[action_index].name.to_string()))
                                    .show_ui(ui, |ui| {
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[settings.current_script].actions[i].name, Command::None, "None")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[settings.current_script].actions[i].name, Command::Wait, "Wait")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[settings.current_script].actions[i].name, Command::Simulate, "Simulate")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[settings.current_script].actions[i].name, Command::Advance, "Advance")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[settings.current_script].actions[i].name, Command::Select, "Select")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[settings.current_script].actions[i].name, Command::SelectAll, "Select All")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[settings.current_script].actions[i].name, Command::Set_Properties, "Set Properties")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[settings.current_script].actions[i].name, Command::Set_Physics, "Set Physics")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[settings.current_script].actions[i].name, Command::Set_Material, "Set Material")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[settings.current_script].actions[i].name, Command::Set_Bonds, "Set Bonds")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[settings.current_script].actions[i].name, Command::Goto, "Goto")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[settings.current_script].actions[i].name, Command::Backup, "Backup")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[settings.current_script].actions[i].name, Command::Restore, "Restore")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[settings.current_script].actions[i].name, Command::Call_Script, "Call Script")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[settings.current_script].actions[i].name, Command::Record, "Record")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[settings.current_script].actions[i].name, Command::Export, "Export")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                        if ui
                                            .selectable_value(&mut script_manager.scripts[settings.current_script].actions[i].name, Command::Export_Screenshot, "Export Screenshot")
                                            .clicked()
                                        {
                                            changed_action = true;
                                        }
                                    });
                                if changed_action {
                                    script_manager.scripts[settings.current_script].actions[i].init_parameters(settings.setup.particles);
                                }
                                let mut script_names = vec![];
                                for script in &script_manager.scripts {
                                    script_names.push(script.name.clone());
                                }
                                let action_count = script_manager.scripts[settings.current_script].actions.len();
                                script_manager.scripts[settings.current_script].actions[action_index].ui(
                                    ui,
                                    format!("{}:{}", settings.current_script, i),
                                    (settings.materials.len() / settings.material_size) as usize,
                                    action_count,
                                    prog,
                                    device,
                                    queue,
                                    script_names,
                                );
                                ui.with_layout(egui::Layout::right_to_left(Align::RIGHT), |ui| {
                                    if ui.button("X").clicked() {
                                        script_manager.scripts[settings.current_script].delete_action(i);
                                    }
                                });
                            });
                            i += 1;
                        }
                    }
                    ui.separator();
                    if ui.add_sized(egui::Vec2::new(ui.available_width(), 0.0), egui::Button::new("+")).clicked() {
                        script_manager.push_action(settings.current_script, Action::new(Command::None, vec![]));
                    }
                });
            } else {
                ui.text_edit_multiline(&mut script_manager.scripts[settings.current_script].to_json());
            }
        });
    }
}
