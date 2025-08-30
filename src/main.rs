#![allow(warnings)]

use std::env;
pub mod audio_controller;
pub mod client;
pub mod macros;
pub mod particle_def;
pub mod scripts;
pub mod settings;
pub mod setup;
pub mod shader_gen;
pub mod sound;
pub mod state;
pub mod timeline_widget;
pub mod ui;
pub mod wgpu_config;
pub mod wgpu_prog;
pub mod wgpu_structs;
pub mod window_init;

pub fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    dbg!(args);
    let mut client = async_std::task::block_on(client::Client::new());
    client.start_event_loop();
    client.resize(client.canvas.size);
}
