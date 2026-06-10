#![allow(warnings)]
pub mod audio_controller;
pub mod client;
#[cfg(not(target_arch = "wasm32"))]
pub mod headless;
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

#[cfg(target_arch = "wasm32")]
use console_log::*;
use log::*;
use std::ptr::null;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::dpi::PhysicalSize;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn webmain() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
    let client = async_std::task::block_on(client::Client::new());
}

