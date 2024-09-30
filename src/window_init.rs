//Followed this guide to get started https://sotrh.github.io/learn-wgpu/#what-is-wgpu
use rand::prelude::*;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::WindowBuilder, dpi::PhysicalSize,
};

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

pub struct Canvas{
    pub window:  winit::window::Window,
    pub size: PhysicalSize<u32>
}

impl Canvas {
    pub fn new(window: winit::window::Window) -> Self {
        #[cfg(target_arch = "wasm32")]
        {  
            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.body()?;
                    let canvas = web_sys::Element::from(window.canvas());
                    canvas.set_id("winit");
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }

        #[cfg(target_arch = "wasm32")] {
            use winit::dpi::PhysicalSize;
            let w = web_sys::window().unwrap().inner_width().unwrap().as_f64().unwrap();
            let h = web_sys::window().unwrap().inner_height().unwrap().as_f64().unwrap();
            self.window.set_inner_size(PhysicalSize::new(w, h));
        }
    
        let size = window.inner_size();

        Self{
            window,
            size
        }
    }

    pub fn updateSize(&mut self, size: PhysicalSize<u32>){
        self.size = size;
        #[cfg(target_arch = "wasm32")] {
            use winit::dpi::PhysicalSize;
            let w = web_sys::window().unwrap().inner_width().unwrap().as_f64().unwrap();
            let h = web_sys::window().unwrap().inner_height().unwrap().as_f64().unwrap();
            self.window.set_inner_size(PhysicalSize::new(w, h));
        }
    }

}

use winit::window::Window;