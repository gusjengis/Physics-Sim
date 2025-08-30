use egui::{Context, Ui};
use egui_plot::{Line, Plot, PlotPoints};

use crate::settings::{Property, Settings};

pub fn data_menu(settings: &mut Settings, ui: &mut Ui, ctx: &Context) {
    ui.menu_button("Data", |ui| {
        ui.style_mut().wrap = Some(false);

        if ui.selectable_label(settings.view.data_menu, "Data Panel").clicked() {
            settings.view.data_menu = !settings.view.data_menu;
        }

        ui.separator();
        ui.label("Recording");

        if ui.checkbox(&mut settings.timed_recording, "Timed").changed() {
            settings.start_time = settings.sim_time;
        }
        ui.add_enabled(settings.timed_recording, egui::DragValue::new(&mut settings.recording_duration).speed(0.001).suffix("s"));
        // });
        // ui.horizontal_centered(|ui| {
        if !(settings.recording || settings.gather_data) {
            if ui.button("Start").clicked() {
                if !settings.timed_recording {
                    settings.gather_data = true;
                } else {
                    settings.recording = true;
                }
                settings.start_time = settings.sim_time;
            }
        } else {
            if ui.button("Stop").clicked() {
                settings.recording = false;
                settings.gather_data = false;
                // settings.start_time = settings.sim_time;
            }
        }
        // });

        if ui.button("Export").clicked() {
            settings.save_data(None);
        }
    });
    if settings.view.data_menu {
        egui::TopBottomPanel::bottom("data_panel").resizable(true).default_height(300.0).show(ctx, |ui| {
            // if ui.checkbox(&mut settings.gather_data, "Gather Data").changed() {
            //     settings.start_time = settings.sim_time;
            // }
            let mut reset_button = None;
            egui::menu::bar(ui, |ui| {
                // });
                // ui.horizontal_centered(|ui|{

                egui::ComboBox::new("graph_property", "").selected_text(format!("{:?}", settings.plotted_prop)).show_ui(ui, |ui| {
                    ui.selectable_value(&mut settings.plotted_prop, Property::X_Position, "X Position");
                    ui.selectable_value(&mut settings.plotted_prop, Property::Y_Position, "Y Position");
                    ui.selectable_value(&mut settings.plotted_prop, Property::Rotation, "Rotation");
                    ui.selectable_value(&mut settings.plotted_prop, Property::X_Velocity, "X Velocity");
                    ui.selectable_value(&mut settings.plotted_prop, Property::Y_Velocity, "Y Velocity");
                    ui.selectable_value(&mut settings.plotted_prop, Property::Rotational_Velocity, "Rotational Velocity");
                    ui.selectable_value(&mut settings.plotted_prop, Property::Normal_Force, "Normal Force");
                    ui.selectable_value(&mut settings.plotted_prop, Property::Shear_Force, "Shear Force");
                    ui.selectable_value(&mut settings.plotted_prop, Property::Moment, "Moment");
                    // ui.selectable_value(&mut settings.plotted_prop, Property::Data_4, "Data 4");
                    ui.selectable_value(&mut settings.plotted_prop, Property::FPS, "FPS");
                    ui.selectable_value(&mut settings.plotted_prop, Property::Torn_Bonds, "Torn_Bonds");
                });
                reset_button = Some(ui.add(egui::Button::new("Reset View")));
            });
            let mut plot = Plot::new("physics plot").auto_bounds_x().auto_bounds_y().clamp_grid(true);
            if reset_button.unwrap().clicked() {
                plot = plot.reset()
            }
            plot.show(ui, |plot_ui| {
                match settings.plotted_prop {
                    Property::X_Position => {
                        plot_ui.line(Line::new("X_Position", PlotPoints::from(settings.data.x_pos_data.to_owned())));
                    }
                    Property::Y_Position => {
                        plot_ui.line(Line::new("Y_Position", PlotPoints::from(settings.data.y_pos_data.to_owned())));
                    }
                    Property::Rotation => {
                        plot_ui.line(Line::new("Rotation", PlotPoints::from(settings.data.rot_data.to_owned())));
                    }
                    Property::X_Velocity => {
                        plot_ui.line(Line::new("X_Velocity", PlotPoints::from(settings.data.x_vel_data.to_owned())));
                    }
                    Property::Y_Velocity => {
                        plot_ui.line(Line::new("Y_Velocity", PlotPoints::from(settings.data.y_vel_data.to_owned())));
                    }
                    Property::Rotational_Velocity => {
                        plot_ui.line(Line::new("Rotational_Velocity", PlotPoints::from(settings.data.rot_vel_data.to_owned())));
                    }
                    Property::Normal_Force => {
                        plot_ui.line(Line::new("Normal_Force", PlotPoints::from(settings.data.data1.to_owned())));
                    }
                    Property::Shear_Force => {
                        plot_ui.line(Line::new("Shear_Force", PlotPoints::from(settings.data.data2.to_owned())));
                    }
                    Property::Moment => {
                        plot_ui.line(Line::new("Moment", PlotPoints::from(settings.data.data3.to_owned())));
                    }
                    Property::FPS => {
                        plot_ui.line(Line::new("FPS", PlotPoints::from(settings.data.fps.to_owned())));
                    }
                    Property::Torn_Bonds => {
                        plot_ui.line(Line::new("Torn_Bonds", PlotPoints::from(settings.data.torn_bonds.to_owned())));
                    } // Property::Data_4 => {plot_ui.line(Line::new(PlotPoints::from(settings.data.data4.to_owned())));},
                }
            });
        });
    }
}
