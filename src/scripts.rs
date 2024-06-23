use std::ops::RangeInclusive;
use std::{str::FromStr, vec};
use crate::{wgpu_config::WGPUConfig, window_init::Canvas};
use crate::settings::Properties;
use chrono::Local;
use egui::Ui;
use serde_json::*;
use serde::{self, Serialize, Deserialize};
use wgpu::{Device, Queue};

use crate::{client::Client, wgpu_prog::{WGPUProg, WGPUComputeProg}};

static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
// #[derive(Serialize, Deserialize)]
pub struct Script {
    pub name: String,
    pub actions: Vec<Action>
}

impl Script {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from_str(name).unwrap(),
            actions: vec![]
        }
    }

    pub fn push_action(&mut self, action: Action){
        self.actions.push(action);
    }

    // pub fn get_action(&self, index: usize) -> Action {
    //     return self.actions[index].clone();
    // }
    // pub fn from_json(json: &str) -> Self {
    //     return serde_json::from_str(json).unwrap();
    // }

    // pub fn to_json(&self) -> String {
    //     return serde_json::to_string(&self).unwrap();
    // }   
}

pub struct ScriptManager {
    pub scripts: Vec<Script>,
    pub action_indices: Vec<i64>,
    pub executing: Vec<bool>,
    pub wait_timestamps: Vec<i64>,
}

impl ScriptManager {
    pub fn new() -> Self {
        Self {
            scripts: vec![],
            action_indices: vec![],
            executing: vec![],
            wait_timestamps: vec![]
        }
    }

    fn init_script(&mut self){
        self.action_indices.push(-1);
        self.executing.push(false);
        self.wait_timestamps.push(0);
    }
    pub fn push_script(&mut self, script: Script) {
        self.scripts.push(script);
        self.init_script();
    }

    pub fn new_script(&mut self, name: &str) {
        self.scripts.push(
            Script::new(name)
        );
        self.init_script();
    }

    pub fn toggle_execution(&mut self, script_index: usize){
        self.executing[script_index] = !self.executing[script_index];
    }

    pub fn push_action(&mut self, script_index: usize, action: Action){
        self.scripts[script_index].push_action(action);
    }

    pub fn execute(&mut self, prog: &mut WGPUProg, config: &WGPUConfig, canvas: &Canvas) {
        for i in 0..self.scripts.len() {
            while self.executing[i] && Local::now().timestamp_millis() > self.wait_timestamps[i] {
                if self.action_indices[i] < self.scripts[i].actions.len() as i64 - 1 {
                    self.action_indices[i] += 1;
                    self.execute_action(self.action_indices[i] as usize, i, prog, config, canvas);
                } else {
                    self.executing[i] = false;
                    self.action_indices[i] = -1;
                
                }
            }
        }
    }

    fn execute_action(&mut self, action_index: usize, script_index: usize, prog: &mut WGPUProg, config: &WGPUConfig, canvas: &Canvas){
        match self.scripts[script_index].actions[action_index].name {
            Command::None => {},
            Command::Wait => { self.wait(self.scripts[script_index].actions[action_index].parameters[0].to_string(), script_index); }//f64::from_str(action.parameters.as_str()).unwrap(), script_index); }
            Command::Select_All => { self.select_all(prog, config, canvas); }
            Command::Set_Properties => { let mut params = vec![]; for i in 0..self.scripts[script_index].actions[action_index].parameters.len() { params.push(self.scripts[script_index].actions[action_index].parameters[i].as_f32()); } self.set_properties(prog, config, params); }
            Command::Goto => { self.goto(script_index, self.scripts[script_index].actions[action_index].parameters[0].as_i32());}
        }
    }

    fn wait(&mut self, duration_string: String, script_index: usize) {
        let duration = f64::from_str(duration_string.as_str()).unwrap();
        self.wait_timestamps[script_index] = Local::now().timestamp_millis() + (duration * 1000.0) as i64;
    }

    fn select_all(&mut self, prog: &mut WGPUProg, config: &WGPUConfig, canvas: &Canvas) {
        prog.shader_prog.buffers.selectangle_input.updateUniform(&config.device, bytemuck::cast_slice(
            &[
                bytemuck::cast::<_, f32>(0 as i32),
                bytemuck::cast::<_, f32>(0 as i32),
                bytemuck::cast::<_, f32>(canvas.size.width as i32),
                bytemuck::cast::<_, f32>(canvas.size.height as i32),
            ]
        ));
        prog.shader_prog.selectangle(config, (canvas.size.width, canvas.size.height));
    }

    fn set_properties(&self, prog: &mut WGPUProg, config: &WGPUConfig, properties: Vec<f32>) {
        prog.shader_prog.buffers.set_prop_input.updateUniform(&config.device, bytemuck::cast_slice(&properties));
        prog.shader_prog.set_properties(config);
    }

    fn goto(&mut self, script_index: usize, line: i32){
        self.action_indices[script_index] = (line - 2) as i64;
    }

    // fn backup(&mut self, prog: &mut WGPUProg){
    //    prog.shader_prog.update_state(&mut self.wgpu_config);
    //    prog.shader_prog.state.save(&mut self.wgpu_config);
    // }

    // fn restore(&mut self, prog: &mut WGPUProg,){
    //    prog.shader_prog.state.load(&mut self.wgpu_config, false);
    //    prog.shader_prog.restore(&mut self.wgpu_config);
    //     self.wgpu_config.prog_settings.data = Data::new();
    // }
}

// #[derive(Clone, Serialize, Deserialize, PartialEq)]
// #[derive(Clone)]
pub struct Action {
    pub name: Command,
    pub parameters: Vec<Box<dyn Parameter>>,
}

impl Action {
    pub fn new(name: Command, parameters: Vec<Box<dyn Parameter>>) -> Self {
        Self {
            name,
            parameters: parameters
        }
    }

    pub fn init_parameters(&mut self) {
        match self.name {
            Command::None => { self.parameters = vec![]; },
            Command::Wait => { self.parameters = vec![
                Box::new(Float::new(0.0))
            ]; },
            Command::Select_All => { self.parameters = vec![]; },
            Command::Set_Properties => { self.parameters = vec![
                Box::new(Boolean::new(false)),
                Box::new(Boolean::new(false)),
                Box::new(Boolean::new(false)),
                Box::new(Boolean::new(false)),
                Box::new(Boolean::new(false)),
                Box::new(Boolean::new(false)),
                Box::new(Boolean::new(false)),
                Box::new(Boolean::new(false)),
                Box::new(Boolean::new(false)),
                Box::new(Boolean::new(false)),
                Box::new(Boolean::new(false)),
                Box::new(Boolean::new(false)),
                Box::new(Boolean::new(false)),
                Box::new(Boolean::new(false)),
                Box::new(Float::new(0.0)),
                Box::new(Float::new(0.0)),
                Box::new(Float::new(0.0)),
                Box::new(Float::new(0.0)),
                Box::new(Float::new(0.0)),
                Box::new(Float::new(0.0)),
                Box::new(Float::new(0.0)),
                Box::new(Float::new(0.0)),
                Box::new(Float::new(0.0)),
                Box::new(Float::new(0.0)),
                Box::new(Boolean::new(false)),
                Box::new(Boolean::new(false)),
                Box::new(Boolean::new(false)),
                Box::new(Integer::new(0)),
            ]; },
            Command::Goto => {self.parameters = vec![
                Box::new(Integer::new(0))
            ];},
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, id: String, mat_count: usize, action_count: usize) {
        match self.name {
            Command::None => {},
            Command::Wait => {ui.label("Duration: "); self.parameters[0].ui(ui, "s", true, Some(0.0..=f64::MAX));}
            Command::Select_All => {},
            Command::Set_Properties => {
                egui::CollapsingHeader::new("Properties").id_source(id).show(ui, |ui| {
                    ui.label("Position");
                    ui.horizontal(|ui| {self.parameters[ 0].ui(ui, "", true, None); let enabled = self.parameters[ 0].truthy(); self.parameters[14].ui(ui, "", enabled, Some(f32::MIN as f64..=f32::MAX as f64)); ui.label("X Position"); });
                    ui.horizontal(|ui| {self.parameters[ 1].ui(ui, "", true, None); let enabled = self.parameters[ 1].truthy(); self.parameters[15].ui(ui, "", enabled, Some(f32::MIN as f64..=f32::MAX as f64)); ui.label("Y Position"); });
                    ui.horizontal(|ui| {self.parameters[ 2].ui(ui, "", true, None); let enabled = self.parameters[ 2].truthy(); self.parameters[16].ui(ui, "", enabled, Some(f32::MIN as f64..=f32::MAX as f64)); ui.label("Rotation"); });
                    ui.label("Velocity");
                    ui.horizontal(|ui| {self.parameters[ 3].ui(ui, "", true, None); let enabled = self.parameters[ 3].truthy(); self.parameters[17].ui(ui, "", enabled, Some(f32::MIN as f64..=f32::MAX as f64)); ui.label("X Velocity"); });
                    ui.horizontal(|ui| {self.parameters[ 4].ui(ui, "", true, None); let enabled = self.parameters[ 4].truthy(); self.parameters[18].ui(ui, "", enabled, Some(f32::MIN as f64..=f32::MAX as f64)); ui.label("Y Velocity"); });
                    ui.horizontal(|ui| {self.parameters[ 5].ui(ui, "", true, None); let enabled = self.parameters[ 5].truthy(); self.parameters[19].ui(ui, "", enabled, Some(f32::MIN as f64..=f32::MAX as f64)); ui.label("Rotational Velocity"); });
                    ui.label("Forces");
                    ui.horizontal(|ui| {self.parameters[ 6].ui(ui, "", true, None); let enabled = self.parameters[ 6].truthy(); self.parameters[20].ui(ui, "", enabled, Some(f32::MIN as f64..=f32::MAX as f64)); ui.label("X Force"); });
                    ui.horizontal(|ui| {self.parameters[ 7].ui(ui, "", true, None); let enabled = self.parameters[ 7].truthy(); self.parameters[21].ui(ui, "", enabled, Some(f32::MIN as f64..=f32::MAX as f64)); ui.label("Y Force"); });
                    ui.horizontal(|ui| {self.parameters[ 8].ui(ui, "", true, None); let enabled = self.parameters[ 8].truthy(); self.parameters[22].ui(ui, "", enabled, Some(f32::MIN as f64..=f32::MAX as f64)); ui.label("Rotational Force"); });
                    ui.label("Radius");
                    ui.horizontal(|ui| {self.parameters[ 9].ui(ui, "", true, None); let enabled = self.parameters[ 9].truthy(); self.parameters[23].ui(ui, "", enabled, Some(0.0..=f32::MAX as f64)); ui.label("Radius"); });
                    ui.label("Fixity");
                    ui.horizontal(|ui| {self.parameters[10].ui(ui, "", true, None); let enabled = self.parameters[10].truthy(); self.parameters[24].ui(ui, "", enabled, None); ui.label("X Fixity"); });
                    ui.horizontal(|ui| {self.parameters[11].ui(ui, "", true, None); let enabled = self.parameters[11].truthy(); self.parameters[25].ui(ui, "", enabled, None); ui.label("Y Fixity"); });
                    ui.horizontal(|ui| {self.parameters[12].ui(ui, "", true, None); let enabled = self.parameters[12].truthy(); self.parameters[26].ui(ui, "", enabled, None); ui.label("Rotational Fixity"); });
                    ui.label("Material");
                    ui.horizontal(|ui| {self.parameters[13].ui(ui, "", true, None); let enabled = self.parameters[13].truthy(); self.parameters[27].ui(ui, "", enabled, Some(0.0..=(mat_count as f64 - 1.0) )); ui.label("Material"); });
                });
            },
            Command::Goto => {
                self.parameters[0].ui(ui, "", true, Some(1.0..=action_count as f64));
            },
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

// #[derive(Clone, Serialize, Deserialize, PartialEq)]
#[derive(Clone, PartialEq)]
pub enum Command {
    None,
    Wait,
    Select_All,
    Set_Properties,
    Goto
}

impl Command {
    pub fn to_string(&self) -> String {
        match self {
            Command::None => { String::from_str("None").unwrap() }
            Command::Wait => { String::from_str("Wait").unwrap() }
            Command::Select_All => { String::from_str("Select All").unwrap() }
            Command::Set_Properties => { String::from_str("Set Properties").unwrap() }
            Command::Goto => { String::from_str("Goto").unwrap() }
        }
    }
}

//Parameters
pub trait Parameter {
    fn ui(&mut self, ui: &mut Ui, label: &str, enabled: bool, range: Option<RangeInclusive<f64>>);
    fn to_string(&self) -> String;
    fn truthy(&self) -> bool;
    fn as_f32(&self) -> f32;
    fn as_i32(&self) -> i32;
}

// #[derive(Clone)]
pub struct Float {
    pub value: f32
}
pub struct Integer {
    pub value: i32
}
pub struct Boolean {
    pub value: bool
}

impl Float { 
    pub fn new(value: f32) -> Self {
        Self {
            value
        }
    }
}

impl Integer { 
    pub fn new(value: i32) -> Self {
        Self {
            value
        }
    }
}

impl Boolean { 
    pub fn new(value: bool) -> Self {
        Self {
            value
        }
    }
}

impl Parameter for Float {
    fn to_string(&self) -> String {
        return self.value.to_string();
    }

    fn ui(&mut self, ui: &mut Ui, label: &str, enabled: bool, range: Option<RangeInclusive<f64>>) {
        ui.add_enabled(enabled, egui::DragValue::new(&mut self.value).clamp_range(range.unwrap()).suffix(label));
    }

    fn truthy(&self) -> bool { true }
    fn as_f32(&self) -> f32 { self.value }
    fn as_i32(&self) -> i32 { bytemuck::cast(self.value) }
}

impl Parameter for Integer {
    fn to_string(&self) -> String {
        return self.value.to_string();
    }

    fn ui(&mut self, ui: &mut Ui, label: &str, enabled: bool, range: Option<RangeInclusive<f64>>) {
        ui.add_enabled(enabled, egui::DragValue::new(&mut self.value).clamp_range(range.unwrap()).suffix(label));
    }

    fn truthy(&self) -> bool { true }
    fn as_f32(&self) -> f32 { bytemuck::cast(self.value) }
    fn as_i32(&self) -> i32 { self.value }
}

impl Parameter for Boolean {
    fn to_string(&self) -> String {
        return self.value.to_string();
    }

    fn ui(&mut self, ui: &mut Ui, label: &str, enabled: bool, range: Option<RangeInclusive<f64>>) {
        ui.add_enabled(enabled, egui::Checkbox::new(&mut self.value, label));
    }

    fn truthy(&self) -> bool { self.value }
    fn as_f32(&self) -> f32 { bytemuck::cast(self.value as i32) }
    fn as_i32(&self) -> i32 { self.value as i32 }
}