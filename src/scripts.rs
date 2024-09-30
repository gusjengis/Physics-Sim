use crate::settings::{Data, Properties, Settings};
use crate::{wgpu_config::WGPUConfig, window_init::Canvas};
use chrono::Local;
use egui::Ui;
use native_dialog::FileDialog;
use serde::{self, Deserialize, Serialize};
use serde_json::*;
use std::fmt;
use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::{str::FromStr, vec};
use wgpu::{Device, Queue};
use winit::event::VirtualKeyCode;

use crate::{
    client::Client,
    wgpu_prog::{WGPUComputeProg, WGPUProg},
};

pub struct ScriptManager {
    pub scripts: Vec<Script>,
    pub threads: Vec<Thread>,
}

impl ScriptManager {
    pub fn new() -> Self {
        let mut script_manager = Self { scripts: vec![], threads: vec![] };
        script_manager.new_script("Script 1");
        return script_manager;
    }

    fn init_script(&mut self) {
        self.threads.push(Thread::new(self.scripts.len() - 1));
    }

    pub fn push_script(&mut self, script: Script) {
        self.scripts.push(script);
        self.init_script();
    }

    pub fn new_script(&mut self, name: &str) {
        self.scripts.push(Script::new(name));
        self.init_script();
    }

    pub fn delete_script(&mut self, script_index: usize) {
        for thread in &mut self.threads {
            if !thread.stack.is_empty() {
                for i in 0..thread.stack.len() {
                    if thread.stack[i].0 == script_index {
                        thread.executing = false;
                        thread.stack = vec![];
                    }
                }
            }
        }
        self.scripts.remove(script_index);
        self.threads.remove(script_index);
        if self.scripts.is_empty() {
            self.new_script("Script 1");
        } else {
            for script in &mut self.scripts {
                for i in 0..script.actions.len() {
                    match script.actions[i].name {
                        Command::Call_Script => {
                            if script.actions[i].parameters[0].as_i32() as usize == script_index {
                                script.actions[i].name = Command::None;
                                script.actions[i].init_parameters(0)
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn toggle_execution(&mut self, script_index: usize) {
        if !self.scripts[script_index].actions.is_empty() {
            self.threads[script_index].toggle_execution(script_index);
        }
    }

    pub fn auto_run(&mut self) {
        for i in 0..self.scripts.len() {
            if self.scripts[i].auto_run {
                self.toggle_execution(i);
            }
        }
    }

    pub fn push_action(&mut self, script_index: usize, action: Action) {
        self.scripts[script_index].push_action(action);
    }

    pub fn key_pressed(&mut self, key: VirtualKeyCode, prog: &mut WGPUProg, config: &mut WGPUConfig, settings: &mut Settings, canvas: &Canvas) {
        let k = Key::from_vck(key);
        for (i, script) in self.scripts.iter().enumerate() {
            match &script.script_trigger {
                Trigger::KeyPressed(sk) => {
                    if k == *sk {
                        if !self.threads[i].executing {
                            self.threads[i].toggle_execution(i);
                            self.threads[i].execute(&self.scripts, prog, config, settings, canvas);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn execute(&mut self, prog: &mut WGPUProg, config: &mut WGPUConfig, settings: &mut Settings, canvas: &Canvas) {
        for thread in &mut self.threads {
            thread.execute(&self.scripts, prog, config, settings, canvas);
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.scripts).unwrap()
    }

    pub fn from_json(&mut self, string: &str) {
        let scripts: Vec<Script> = serde_json::from_str(string).unwrap();
        self.scripts = vec![];
        self.threads = vec![];
        for script in scripts {
            self.push_script(script);
        }
        if self.scripts.len() == 0 {
            self.new_script("Script 1");
        }
    }

    pub fn export_scripts(&self) {
        if let Some(path) = FileDialog::new().set_location("").add_filter("JSON", &["json"]).show_save_single_file().unwrap() {
            let json_data = self.to_json(); // Export all scripts
            std::fs::write(path, json_data).expect("Unable to write to file");
        }
    }

    // Export a single script by index to a JSON file
    pub fn export_single_script(&self, script_index: usize) {
        if script_index < self.scripts.len() {
            if let Some(path) = FileDialog::new().set_location("").add_filter("JSON", &["json"]).show_save_single_file().unwrap() {
                let script_json = self.scripts[script_index].to_json(); // Serialize only one script
                std::fs::write(path, script_json).expect("Unable to write to file");
            }
        } else {
            eprintln!("Invalid script index: {}", script_index);
        }
    }

    pub fn import_scripts(&mut self) {
        if let Some(path) = FileDialog::new().set_location("").add_filter("JSON", &["json"]).show_open_single_file().unwrap() {
            let json_data = std::fs::read_to_string(path).expect("Unable to read file");

            // Try to deserialize as a list of scripts or a single script
            if let Ok(new_scripts) = serde_json::from_str::<Vec<Script>>(&json_data) {
                // Append the scripts if the JSON contains multiple scripts
                for script in new_scripts {
                    self.push_script(script);
                }
            } else if let Ok(single_script) = serde_json::from_str::<Script>(&json_data) {
                // Append the single script if the JSON contains only one script
                self.push_script(single_script);
            } else {
                eprintln!("Invalid JSON format for scripts.");
            }
        }
    }
}

pub struct Thread {
    pub stack: Vec<(usize, i64)>,
    pub executing: bool,
    pub wait_timestamp: i64,
}

impl Thread {
    pub fn new(script_index: usize) -> Self {
        Self {
            stack: vec![],
            executing: false,
            wait_timestamp: 0,
        }
    }

    pub fn toggle_execution(&mut self, script_index: usize) {
        self.executing = !self.executing;
        if self.stack.is_empty() {
            self.stack.push((script_index, -1));
        }
    }

    pub fn script(&self) -> usize {
        self.stack.last().unwrap().0
    }
    pub fn action(&self) -> i64 {
        self.stack.last().unwrap().1
    }
    pub fn set_action(&mut self, action: i64) {
        let stack_index = self.stack.len() - 1;
        self.stack[stack_index] = (self.stack[stack_index].0, action);
    }
    pub fn inc_action(&mut self, scripts: &Vec<Script>) -> (usize, i64) {
        let stack_index = self.stack.len() - 1;
        self.stack[stack_index] = (self.stack[stack_index].0, self.stack[stack_index].1 + 1);
        let res = (self.stack.last().unwrap()).clone();
        if res.1 as usize == scripts[res.0].actions.len() - 1 {
            self.stack.pop();
        }
        return res;
    }

    pub fn execute(&mut self, scripts: &Vec<Script>, prog: &mut WGPUProg, config: &mut WGPUConfig, settings: &mut Settings, canvas: &Canvas) {
        while !self.stack.is_empty() && self.executing && Local::now().timestamp_millis() > self.wait_timestamp {
            // print!("[");
            // for i in 0..self.stack.len() {
            //     print!("({}, {}) ", self.stack[i].0, self.stack[i].1);
            // }
            // println!("]");
            if self.action() < scripts[self.script()].actions.len() as i64 - 1 {
                let script_action = self.inc_action(scripts);
                self.execute_action(scripts, script_action.1 as usize, script_action.0, prog, config, settings, canvas);
                if self.stack.is_empty() {
                    self.executing = false;
                }
            }
        }
    }

    fn execute_action(&mut self, scripts: &Vec<Script>, action_index: usize, script_index: usize, prog: &mut WGPUProg, config: &mut WGPUConfig, settings: &mut Settings, canvas: &Canvas) {
        match scripts[script_index].actions[action_index].name {
            Command::None => {}
            Command::Wait => {
                self.wait(scripts[script_index].actions[action_index].parameters[0].to_string(), script_index);
            } //f64::from_str(action.parameters.as_str()).unwrap(), script_index); }
            Command::Select_All => {
                self.select_all(prog, config, canvas);
            }
            Command::Set_Properties => {
                let mut params = vec![];
                for i in 0..scripts[script_index].actions[action_index].parameters.len() {
                    params.push(scripts[script_index].actions[action_index].parameters[i].as_f32());
                }
                self.set_properties(prog, config, settings, params);
            }
            Command::Goto => {
                self.goto(script_index, scripts[script_index].actions[action_index].parameters[0].as_i32());
            }
            Command::Select => {
                self.select(scripts[script_index].actions[action_index].parameters[0].as_i32_vec(), &config, prog);
            }
            Command::Simulate => {
                self.set_simulation(scripts[script_index].actions[action_index].parameters[0].truthy(), settings);
            }
            Command::Backup => {
                self.backup(prog, config, settings);
            }
            Command::Restore => {
                self.restore(prog, config, settings);
            }
            Command::Call_Script => {
                self.call_script(scripts, action_index, script_index);
            }
            Command::Advance => {
                self.advance(scripts, action_index, script_index, settings);
            }
            Command::Export => {
                settings.save_data(Some(scripts[script_index].actions[action_index].parameters[0].as_path().clone()));
            }
            Command::Record => {
                self.record(scripts, action_index, script_index, settings);
            }
        }
    }

    fn wait(&mut self, duration_string: String, script_index: usize) {
        let duration = f64::from_str(duration_string.as_str()).unwrap();
        self.wait_timestamp = Local::now().timestamp_millis() + (duration * 1000.0) as i64;
    }

    fn select_all(&mut self, prog: &mut WGPUProg, config: &WGPUConfig, canvas: &Canvas) {
        prog.shader_prog.buffers.selectangle_input.updateUniform(
            &config.device,
            bytemuck::cast_slice(&[
                bytemuck::cast::<_, f32>(0 as i32),
                bytemuck::cast::<_, f32>(0 as i32),
                bytemuck::cast::<_, f32>(canvas.size.width as i32),
                bytemuck::cast::<_, f32>(canvas.size.height as i32),
            ]),
        );
        prog.shader_prog.selectangle(config, (canvas.size.width, canvas.size.height));
    }

    fn set_properties(&self, prog: &mut WGPUProg, config: &WGPUConfig, settings: &mut Settings, properties: Vec<f32>) {
        prog.shader_prog.buffers.set_prop_input.updateUniform(&config.device, bytemuck::cast_slice(&properties));
        prog.shader_prog.set_properties(config, settings);
    }

    fn goto(&mut self, script_index: usize, line: i32) {
        if self.stack.is_empty() {
            self.stack.push((script_index, (line - 2) as i64))
        } else {
            self.set_action((line - 2) as i64);
        }
    }

    fn select(&mut self, selections: Vec<i32>, config: &WGPUConfig, prog: &mut WGPUProg) {
        prog.shader_prog.buffers.selection_buffers.updateBuffer(&config.device, bytemuck::cast_slice(selections.as_slice()), 0);
    }

    fn set_simulation(&self, simulating: bool, settings: &mut Settings) {
        settings.simulating = simulating;
    }

    fn backup(&mut self, prog: &mut WGPUProg, config: &mut WGPUConfig, settings: &mut Settings) {
        prog.shader_prog.update_state(config, settings);
        prog.shader_prog.state.save(config, settings, None);
    }

    fn restore(&mut self, prog: &mut WGPUProg, config: &mut WGPUConfig, settings: &mut Settings) {
        prog.shader_prog.state.load(config, settings, None, false);
        prog.shader_prog.restore(config, settings);
        settings.data = Data::new();
    }

    fn call_script(&mut self, scripts: &Vec<Script>, action_index: usize, script_index: usize) {
        let called_script = scripts[script_index].actions[action_index].parameters[0].as_i32() as usize;
        self.stack.push((called_script, -1));
    }

    fn advance(&mut self, scripts: &Vec<Script>, action_index: usize, script_index: usize, settings: &mut Settings) {
        settings.simulation.advance_x_timesteps = true;
        settings.simulation.x_timesteps = scripts[script_index].actions[action_index].parameters[0].as_i32();
    }

    fn record(&mut self, scripts: &Vec<Script>, action_index: usize, script_index: usize, settings: &mut Settings) {
        let record = scripts[script_index].actions[action_index].parameters[0].truthy();
        settings.recording = record;
        settings.gather_data = record;
        if record {
            settings.start_time = settings.sim_time;
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Script {
    pub name: String,
    #[serde(default)]
    pub auto_run: bool,
    #[serde(default)]
    pub script_trigger: Trigger,
    pub actions: Vec<Action>,
}

impl Script {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from_str(name).unwrap(),
            auto_run: false,
            script_trigger: Trigger::None,
            actions: vec![],
        }
    }

    pub fn push_action(&mut self, action: Action) {
        self.actions.push(action);
    }

    pub fn from_json(json: &str) -> Self {
        return serde_json::from_str(json).unwrap();
    }
    pub fn to_json(&self) -> String {
        return serde_json::to_string(&self).unwrap();
    }
    pub fn to_string(&self) -> String {
        let mut string = format!("");
        for action in &self.actions {
            string.push_str(action.to_string().as_str());
            string.push('\n');
        }
        return string;
    }
    pub fn delete_action(&mut self, action_index: usize) {
        self.actions.remove(action_index);
        for i in action_index..self.actions.len() {
            match self.actions[i].name {
                Command::Goto => {
                    self.actions[i].parameters[0] = Parameter::Integer(format!("Line"), (self.actions[i].parameters[0].as_i32() - 1));
                }
                _ => {}
            }
        }
    }
    // pub fn from_string(&self) -> String {

    // }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
// #[derive(Clone)]
pub struct Action {
    pub name: Command,
    pub parameters: Vec<Parameter>,
}

impl Action {
    pub fn new(name: Command, parameters: Vec<Parameter>) -> Self {
        Self { name, parameters: parameters }
    }

    pub fn to_string(&self) -> String {
        let mut string = self.name.to_string();
        string.push('(');
        let mut index = 0;
        for param in &self.parameters {
            if index > 0 {
                string.push(',');
            }
            string.push_str(param.to_string().as_str());
            index += 1;
        }
        string.push(')');
        return string;
    }

    pub fn init_parameters(&mut self, particle_count: usize) {
        match self.name {
            Command::None => {
                self.parameters = vec![];
            }
            Command::Wait => {
                self.parameters = vec![Parameter::Float(format!("Duration"), 0.0)];
            }
            Command::Select_All => {
                self.parameters = vec![];
            }
            Command::Set_Properties => {
                self.parameters = vec![
                    Parameter::Boolean(format!("Set X Position"), false),
                    Parameter::Boolean(format!("Set Y Position"), false),
                    Parameter::Boolean(format!("Set Rotation"), false),
                    Parameter::Boolean(format!("Set X Velocity"), false),
                    Parameter::Boolean(format!("Set Y Velocity"), false),
                    Parameter::Boolean(format!("Set Rotational Velocity"), false),
                    Parameter::Boolean(format!("Set X Force"), false),
                    Parameter::Boolean(format!("Set Y Force"), false),
                    Parameter::Boolean(format!("Set Rotational Force"), false),
                    Parameter::Boolean(format!("Set Radius"), false),
                    Parameter::Boolean(format!("Set X Fixity"), false),
                    Parameter::Boolean(format!("Set Y Fixity"), false),
                    Parameter::Boolean(format!("Set Rotational Fixity"), false),
                    Parameter::Boolean(format!("Set Material"), false),
                    Parameter::Float(format!("X Position"), 0.0),
                    Parameter::Float(format!("Y Postition"), 0.0),
                    Parameter::Float(format!("Rotation"), 0.0),
                    Parameter::Float(format!("X Velocity"), 0.0),
                    Parameter::Float(format!("Y Velocity"), 0.0),
                    Parameter::Float(format!("Rotational Velocity"), 0.0),
                    Parameter::Float(format!("X Force"), 0.0),
                    Parameter::Float(format!("Y Force"), 0.0),
                    Parameter::Float(format!("Rotational Force"), 0.0),
                    Parameter::Float(format!("Radius"), 0.0),
                    Parameter::Boolean(format!("X Fixity"), false),
                    Parameter::Boolean(format!("Y Fixity"), false),
                    Parameter::Boolean(format!("Rotational Fixity"), false),
                    Parameter::Integer(format!("Material"), 0),
                ];
            }
            Command::Goto => {
                self.parameters = vec![Parameter::Integer(format!("Line"), 0)];
            }
            Command::Select => {
                self.parameters = vec![Parameter::List(format!("Particle ID's"), vec![0; particle_count])];
            }
            Command::Simulate => {
                self.parameters = vec![Parameter::Boolean(format!("Simulating"), false)];
            }
            Command::Backup => {
                self.parameters = vec![];
            }
            Command::Restore => {
                self.parameters = vec![];
            }
            Command::Call_Script => {
                self.parameters = vec![Parameter::Integer(format!("Script_Index"), 0)];
            }
            Command::Advance => {
                self.parameters = vec![Parameter::Integer(format!("Ticks"), 0)];
            }
            Command::Export => {
                self.parameters = vec![Parameter::Path(format!("Path"), PathBuf::new())];
            }
            Command::Record => {
                self.parameters = vec![Parameter::Boolean(format!("Record"), false)];
            }
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, id: String, mat_count: usize, action_count: usize, prog: &mut WGPUProg, device: &mut Device, queue: &mut Queue, script_names: Vec<String>) {
        match self.name {
            Command::None => {}
            Command::Wait => {
                ui.label("Duration: ");
                self.parameters[0].ui(ui, "s", true, Some(0.0..=f64::MAX));
            }
            Command::Select_All => {}
            Command::Set_Properties => {
                egui::CollapsingHeader::new("Properties").id_source(id).show(ui, |ui| {
                    ui.label("Position");
                    ui.horizontal(|ui| {
                        self.parameters[0].ui(ui, "", true, None);
                        let enabled = self.parameters[0].truthy();
                        self.parameters[14].ui(ui, "", enabled, Some(f32::MIN as f64..=f32::MAX as f64));
                        ui.label("X Position");
                    });
                    ui.horizontal(|ui| {
                        self.parameters[1].ui(ui, "", true, None);
                        let enabled = self.parameters[1].truthy();
                        self.parameters[15].ui(ui, "", enabled, Some(f32::MIN as f64..=f32::MAX as f64));
                        ui.label("Y Position");
                    });
                    ui.horizontal(|ui| {
                        self.parameters[2].ui(ui, "", true, None);
                        let enabled = self.parameters[2].truthy();
                        self.parameters[16].ui(ui, "", enabled, Some(f32::MIN as f64..=f32::MAX as f64));
                        ui.label("Rotation");
                    });
                    ui.label("Velocity");
                    ui.horizontal(|ui| {
                        self.parameters[3].ui(ui, "", true, None);
                        let enabled = self.parameters[3].truthy();
                        self.parameters[17].ui(ui, "", enabled, Some(f32::MIN as f64..=f32::MAX as f64));
                        ui.label("X Velocity");
                    });
                    ui.horizontal(|ui| {
                        self.parameters[4].ui(ui, "", true, None);
                        let enabled = self.parameters[4].truthy();
                        self.parameters[18].ui(ui, "", enabled, Some(f32::MIN as f64..=f32::MAX as f64));
                        ui.label("Y Velocity");
                    });
                    ui.horizontal(|ui| {
                        self.parameters[5].ui(ui, "", true, None);
                        let enabled = self.parameters[5].truthy();
                        self.parameters[19].ui(ui, "", enabled, Some(f32::MIN as f64..=f32::MAX as f64));
                        ui.label("Rotational Velocity");
                    });
                    ui.label("Forces");
                    ui.horizontal(|ui| {
                        self.parameters[6].ui(ui, "", true, None);
                        let enabled = self.parameters[6].truthy();
                        self.parameters[20].ui(ui, "", enabled, Some(f32::MIN as f64..=f32::MAX as f64));
                        ui.label("X Force");
                    });
                    ui.horizontal(|ui| {
                        self.parameters[7].ui(ui, "", true, None);
                        let enabled = self.parameters[7].truthy();
                        self.parameters[21].ui(ui, "", enabled, Some(f32::MIN as f64..=f32::MAX as f64));
                        ui.label("Y Force");
                    });
                    ui.horizontal(|ui| {
                        self.parameters[8].ui(ui, "", true, None);
                        let enabled = self.parameters[8].truthy();
                        self.parameters[22].ui(ui, "", enabled, Some(f32::MIN as f64..=f32::MAX as f64));
                        ui.label("Rotational Force");
                    });
                    ui.label("Radius");
                    ui.horizontal(|ui| {
                        self.parameters[9].ui(ui, "", true, None);
                        let enabled = self.parameters[9].truthy();
                        self.parameters[23].ui(ui, "", enabled, Some(0.0..=f32::MAX as f64));
                        ui.label("Radius");
                    });
                    ui.label("Fixity");
                    ui.horizontal(|ui| {
                        self.parameters[10].ui(ui, "", true, None);
                        let enabled = self.parameters[10].truthy();
                        self.parameters[24].ui(ui, "", enabled, None);
                        ui.label("X Fixity");
                    });
                    ui.horizontal(|ui| {
                        self.parameters[11].ui(ui, "", true, None);
                        let enabled = self.parameters[11].truthy();
                        self.parameters[25].ui(ui, "", enabled, None);
                        ui.label("Y Fixity");
                    });
                    ui.horizontal(|ui| {
                        self.parameters[12].ui(ui, "", true, None);
                        let enabled = self.parameters[12].truthy();
                        self.parameters[26].ui(ui, "", enabled, None);
                        ui.label("Rotational Fixity");
                    });
                    ui.label("Material");
                    ui.horizontal(|ui| {
                        self.parameters[13].ui(ui, "", true, None);
                        let enabled = self.parameters[13].truthy();
                        ui.add(egui::Slider::new(self.parameters[27].as_i32_ref().unwrap(), 0..=mat_count as i32 - 1))
                    });
                });
            }
            Command::Goto => {
                self.parameters[0].ui(ui, "", true, Some(1.0..=action_count as f64));
            }
            Command::Select => {
                if ui.button("Set").clicked() {
                    prog.shader_prog.update_selections(device, queue);
                    self.parameters[0].set_list(prog.shader_prog.state.selections.clone());
                    // self.parameters[0].set_list()
                }
                if ui.button("Restore").clicked() {
                    prog.shader_prog
                        .buffers
                        .selection_buffers
                        .updateBuffer(&device, bytemuck::cast_slice(self.parameters[0].as_i32_vec().as_slice()), 0);
                }
            }
            Command::Simulate => {
                self.parameters[0].ui(ui, "", true, None);
            }
            Command::Backup => {}
            Command::Restore => {}
            Command::Call_Script => {
                egui::ComboBox::new(id, "Script")
                    .selected_text(format!("{}", script_names[self.parameters[0].as_i32() as usize]))
                    .show_ui(ui, |ui| {
                        for i in 0..script_names.len() {
                            if ui.selectable_label(self.parameters[0].as_i32() == i as i32, script_names[i].clone()).clicked() {
                                self.parameters[0] = Parameter::Integer(format!("Script_Index"), i as i32);
                            }
                        }
                    });
            }
            Command::Advance => {
                ui.label("Ticks: ");
                self.parameters[0].ui(ui, "", true, Some(0.0..=f64::MAX));
            }
            Command::Export => {
                ui.label("Path: ");
                self.parameters[0].ui(ui, "", true, Some(0.0..=f64::MAX));
            }
            Command::Record => {
                self.parameters[0].ui(ui, "", true, Some(0.0..=f64::MAX));
            }
        }
    }

    // pub fn to_string(&self) -> String {
    //     return format!("{}:\n    {}", self.name.to_string(), self.parameters.to_);
    // }
}

// pub fn script_test(prog: &mut WGPUComputeProg) {
//     let mut manager = ScriptManager::new();
//     manager.new_script("test_script");
//     println!("{}", manager.scripts[0].to_json());
//     manager.push_action(0, Action::new(Command::Wait, vec![]));
//     println!("{}", manager.scripts[0].to_json());
// let deserialized_script = Script::from_json(manager.scripts[0].to_json().as_str());
// println!("{}", deserialized_script.actions[0].to_string());
// manager.toggle_execution(0);
//     manager.execute(prog);
// }

#[derive(Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum Trigger {
    #[default]
    None,
    Click,
    KeyDown(Key),
    KeyPressed(Key),
}

impl fmt::Display for Trigger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Trigger::None => write!(f, "None"),
            Trigger::Click => write!(f, "Click"),
            Trigger::KeyDown(_) => write!(f, "KeyDown"),
            Trigger::KeyPressed(_) => write!(f, "KeyPressed"),
        }
    }
}

impl Trigger {
    pub fn keycode(&self) -> Key {
        match self {
            Trigger::KeyDown(key) | Trigger::KeyPressed(key) => *key,
            _ => Key::Null,
        }
    }

    pub fn set_key(&mut self, key: Key) {
        match self {
            Trigger::KeyDown(k) | Trigger::KeyPressed(k) => {
                *k = key;
            }
            _ => {}
        }
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum Command {
    None,
    Wait,
    Select_All,
    Set_Properties,
    Goto,
    Select,
    Simulate,
    Backup,
    Restore,
    Call_Script,
    Advance,
    Export,
    Record,
    // Load_File,
    // Record
}

impl Command {
    pub fn to_string(&self) -> String {
        match self {
            Command::None => String::from_str("None").unwrap(),
            Command::Wait => String::from_str("Wait").unwrap(),
            Command::Select_All => String::from_str("Select All").unwrap(),
            Command::Set_Properties => String::from_str("Set Properties").unwrap(),
            Command::Goto => String::from_str("Goto").unwrap(),
            Command::Select => String::from_str("Select").unwrap(),
            Command::Simulate => String::from_str("Simulate").unwrap(),
            Command::Backup => String::from_str("Backup").unwrap(),
            Command::Restore => String::from_str("Restore").unwrap(),
            Command::Call_Script => String::from_str("Call Script").unwrap(),
            Command::Advance => String::from_str("Advance").unwrap(),
            Command::Export => String::from_str("Export").unwrap(),
            Command::Record => String::from_str("Record").unwrap(), // Command::Load_File => { String::from_str("Load File").unwrap() }
        }
    }
}

//Parameters
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum Parameter {
    Float(String, f32),
    Integer(String, i32),
    Boolean(String, bool),
    List(String, Vec<i32>),
    Path(String, PathBuf),
}

impl Parameter {
    fn to_string(&self) -> String {
        match self {
            Parameter::Float(_, value) => {
                return value.to_string();
            }
            Parameter::Integer(_, value) => {
                return value.to_string();
            }
            Parameter::Boolean(_, value) => {
                return value.to_string();
            }
            Parameter::List(_, value) => {
                let mut res = format!("");
                for num in value {
                    res.push_str(num.to_string().as_str());
                    res.push(',');
                }
                return res;
            }
            Parameter::Path(_, path) => {
                return String::from(match path.to_str() {
                    Some(str) => str,
                    None => "",
                });
            }
        }
    }

    fn ui(&mut self, ui: &mut Ui, label: &str, enabled: bool, range: Option<RangeInclusive<f64>>) {
        match self {
            Parameter::Float(_, value) => {
                ui.add_enabled(enabled, egui::DragValue::new(&mut *value).clamp_range(range.unwrap()).suffix(label));
            }
            Parameter::Integer(_, value) => {
                ui.add_enabled(enabled, egui::DragValue::new(&mut *value).clamp_range(range.unwrap()).suffix(label));
            }
            Parameter::Boolean(_, value) => {
                ui.add_enabled(enabled, egui::Checkbox::new(&mut *value, label));
            }
            Parameter::List(_, _) => {}
            Parameter::Path(_, path) => {
                ui.add_enabled_ui(enabled, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(String::from(match path.file_name() {
                            Some(str) => str.to_str().unwrap(),
                            None => "",
                        }));
                        if ui.button("Browse").clicked() {
                            let new_path = FileDialog::new().set_location("").add_filter("CSV", &["csv"]).show_open_single_file().unwrap();
                            match new_path {
                                Some(p) => {
                                    path.clear();
                                    path.push(p);
                                }
                                None => {}
                            }
                        }
                    });
                });
            }
        }
    }

    fn truthy(&self) -> bool {
        match self {
            Parameter::Float(_, value) => true,
            Parameter::Integer(_, value) => true,
            Parameter::Boolean(_, value) => *value,
            Parameter::List(_, value) => value.len() != 0,
            Parameter::Path(_, value) => value.to_str().unwrap().len() > 0,
        }
    }

    fn as_f32(&self) -> f32 {
        match self {
            Parameter::Float(_, value) => *value,
            Parameter::Integer(_, value) => bytemuck::cast(*value),
            Parameter::Boolean(_, value) => bytemuck::cast(*value as i32),
            Parameter::List(_, _) => -1.0,
            Parameter::Path(_, _) => -1.0,
        }
    }

    fn as_i32(&self) -> i32 {
        match self {
            Parameter::Float(_, value) => bytemuck::cast(*value),
            Parameter::Integer(_, value) => *value,
            Parameter::Boolean(_, value) => *value as i32,
            Parameter::List(_, _) => -1,
            Parameter::Path(_, _) => -1,
        }
    }

    fn as_i32_vec(&self) -> Vec<i32> {
        match self {
            Parameter::Float(_, value) => {
                vec![*value as i32; 1]
            }
            Parameter::Integer(_, value) => {
                vec![*value; 1]
            }
            Parameter::Boolean(_, value) => {
                vec![1; 1]
            }
            Parameter::List(_, value) => (*value.clone()).to_vec(),
            Parameter::Path(_, value) => {
                vec![1; 1]
            }
        }
    }

    fn set_list(&mut self, list: Vec<i32>) {
        match self {
            Parameter::Float(_, _) => {}
            Parameter::Integer(_, _) => {}
            Parameter::Boolean(_, _) => {}
            Parameter::List(_, value) => {
                *value = list;
            }
            Parameter::Path(_, _) => {}
        }
    }

    fn as_i32_ref(&mut self) -> Option<&mut i32> {
        match self {
            Parameter::Float(_, _) => None,
            Parameter::Integer(_, value) => Some(&mut *value),
            Parameter::Boolean(_, _) => None,
            Parameter::List(_, _) => None,
            Parameter::Path(_, _) => None,
        }
    }

    fn as_path(&self) -> PathBuf {
        match self {
            Parameter::Path(_, path) => path.clone(),
            _ => PathBuf::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug, Default)]
pub enum Key {
    #[default]
    Null,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Space,
}

impl Key {
    pub fn from_vck(key: VirtualKeyCode) -> Self {
        match key {
            VirtualKeyCode::A => Key::A,
            VirtualKeyCode::D => Key::D,
            VirtualKeyCode::S => Key::S,
            VirtualKeyCode::W => Key::W,
            VirtualKeyCode::Space => Key::Space,
            _ => Key::Null,
        }
    }
}

