#![allow(warnings)]
pub mod audio_controller;
pub mod client;
pub mod headless;
pub mod particle_def;
pub mod presets;
pub mod scenario;
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
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 && args[1] == "--headless" {
        let out = if args.len() >= 4 { args[3].clone() } else { "headless_out.csv".to_string() };
        headless::run(&args[2], &out);
        return;
    }
    let mut client = async_std::task::block_on(client::Client::new());
    client.resize(client.canvas.size);
}

