#![allow(warnings)]
pub mod audio_controller;
pub mod client;
pub mod particle_def;
pub mod scripts;
pub mod settings;
pub mod setup;
pub mod shader_gen;
pub mod state;
pub mod wgpu_config;
pub mod wgpu_prog;
pub mod wgpu_structs;
pub mod window_init;

pub fn main() {
    env_logger::init();
    let mut client = async_std::task::block_on(client::Client::new());
    client.resize(client.canvas.size);
}

