#![allow(warnings)]
pub mod window_init;
pub mod client;
pub mod wgpu_config;
pub mod wgpu_structs;
pub mod wgpu_prog;
pub mod settings;
pub mod setup;
pub mod state;
pub mod scripts;
pub mod shader_gen;

pub  fn main(){
    env_logger::init();
    let mut client = async_std::task::block_on(client::Client::new());
    client.resize(client.canvas.size);
}