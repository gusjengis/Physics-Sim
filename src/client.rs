use crate::scripts::ScriptManager;
use crate::settings::Data;
use crate::wgpu_prog::WGPUComputeProg;
use crate::wgpu_structs::DepthBuffer;
use crate::wgpu_structs::Texture;
use crate::window_init;
use crate::wgpu_config::*;
use crate::wgpu_prog;
use crate::scripts;
use crate::settings::*;
use crate::wgpu_prog::WGPUProg;
use cgmath::Angle;
use egui::Rect;
use egui_demo_lib::DemoWindows;
use winit::window::Fullscreen;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::WindowBuilder, dpi::PhysicalSize,
};
use std::cmp::max;
use std::iter;
use std::path::PathBuf;
use cgmath::*;
use winit_fullscreen;
use winit_fullscreen::WindowFullScreen;

use egui_winit_platform::{Platform, PlatformDescriptor};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};

use chrono::prelude::*;

pub struct Client {
    pub canvas: window_init::Canvas,
    wgpu_config: WGPUConfig,
    wgpu_prog: WGPUProg,
    settings: Settings,
    script_manager: ScriptManager,
    last_draw: chrono::DateTime<Local>,
    log_framerate: bool,
    start_time: DateTime<Local>,
    bench_start_time: DateTime<Local>,
    generations: f32,
    temp: f32,
    prev_gen_time: DateTime<Local>,
    cursor_pos: (i32, i32),
    click_pos: (i32, i32),
    cursor_delta: (i32, i32),
    world_delta: (f32, f32),
    minimized: bool,
    fullscreen: bool,
    HL: bool,
    prevGen: i32,
    generation: i32,
    xOff: f32,
    yOff: f32,
    middle: bool,
    shift: bool,
    ctrl: bool,
    dark: f32,
    W: bool,
    A: bool,
    S: bool,
    D: bool,
    G: bool,
    V: bool,
    B: bool,
    N: bool,
    init: bool,
    pub platform: Platform,
    egui_rpass: RenderPass,
    data_length_backup: usize,
    available_rect: Rect,
    boot_time: i64
    // max_framerate: f32,
    // prev_framerate: f32
}

impl Client {
    pub async fn new() -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .build(&event_loop)
            .unwrap();
        window.set_title("Physics Simulator");

        let canvas = window_init::Canvas::new(window);
        let mut wgpu_config = WGPUConfig::new(&canvas).await;
        let mut settings = Settings::new(&canvas);
        settings.f64_support = wgpu_config.f64_support;
        let wgpu_prog = WGPUProg::new(&mut wgpu_config, &mut settings, (canvas.size.width as u32, canvas.size.height as u32));
        
        // UI Setup
        
        let size = canvas.size;
        let platform = Platform::new(PlatformDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: canvas.window.scale_factor(),
            font_definitions: egui::FontDefinitions::default(),
            style: Default::default(),
        });
        let available_rect = platform.context().available_rect();
        platform.context().set_pixels_per_point(2.0);
        let mut egui_rpass = RenderPass::new(&wgpu_config.device, wgpu_config.surface_format, 1);
        let max_framerate = canvas.window.current_monitor().unwrap().refresh_rate_millihertz().unwrap() as f32/1000.0;

        let mut client = Client {
            canvas,
            wgpu_config,
            settings,
            script_manager: ScriptManager::new(),
            last_draw: Local::now(),
            log_framerate: false,
            wgpu_prog,
            start_time: Local::now(),
            bench_start_time: Local::now(),
            temp: 34.0,
            prevGen: 0,
            generations: 100.0,
            prev_gen_time: Local::now(),
            cursor_pos: (0, 0),
            click_pos: (0, 0),
            cursor_delta: (0, 0),
            world_delta: (0.0, 0.0),
            minimized: false,
            fullscreen: false,
            HL: false,
            generation: 0,
            xOff: 0.0,
            yOff: 0.0,
            middle: false,
            shift: false,
            ctrl: false,
            dark: 0.0,
            W: false,
            A: false,
            S: false,
            D: false,
            G: false,
            V: false,
            B: false,
            N: false,
            init: false,
            platform,
            egui_rpass,
            data_length_backup: 1,
            available_rect: available_rect,
            boot_time: Local::now().timestamp_millis()
            // max_framerate:  max_framerate,
            // prev_framerate: max_framerate
        };
        client.resize(client.canvas.size);
        client.platform.handle_event(&Event::WindowEvent {
            window_id: client.canvas.window.id(),
            event: WindowEvent::Resized(client.canvas.size),
            // The generic type is provided here
        } as &Event<()>); 

        
        // client.wgpu_prog =  WGPUProg::new(&mut client.wgpu_config, (client.canvas.size.width as u32, client.canvas.size.height as u32));
        event_loop.run(move |event, _, control_flow| {
            client.platform.handle_event(&event);
            
            if !client.platform.captures_event(&event) {

                match event { 
                    Event::WindowEvent {
                        ref event,
                        window_id,
                    }
                    if window_id == client.canvas.window.id() => {
                        if !client.input(event) {
                    match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        client.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &&mut so we have to dereference it twice
                        client.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }}
            Event::RedrawRequested(window_id) if window_id == client.canvas.window.id() => {
                match client.render() {
                    Ok(_) => {}
                        
                        Err(wgpu::SurfaceError::Lost) => {client.resize(client.canvas.size.clone());} ,
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                client.canvas.window.request_redraw();
            },
            _ => {}
        }}});
        return client;
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        // println!("{}, {}", new_size.width, new_size.height);
        if new_size.width > 0 && new_size.height > 0 {
            self.minimized = false;
            self.canvas.updateSize(new_size);
            self.wgpu_config.config.width = new_size.width;
            self.wgpu_config.config.height = new_size.height;
            self.wgpu_config.size = new_size;
            
            self.wgpu_prog.resize(&mut self.wgpu_config, (self.canvas.size.width as u32, self.canvas.size.height as u32));
            self.wgpu_config.surface.configure(&self.wgpu_config.device, &self.wgpu_config.config);

            let windowDim = self.wgpu_config.size;
            let int_scale = self.settings.view.scale as f32;

            self.wgpu_prog.dim_uniform.updateUniform(&self.wgpu_config.device, bytemuck::cast_slice(
                &[  self.wgpu_config.size.width as f32,
                    0.0 as f32, //time as f32, 
                    self.wgpu_config.size.height as f32,
                    self.temp,
                    self.xOff as f32,
                    self.yOff as f32,
                    self.post_ui_x_off(),
                    self.post_ui_y_off(),
                    self.post_ui_scale(),
                    self.dark as f32,
                    bytemuck::cast(self.click_pos.0),
                    bytemuck::cast(self.click_pos.1),
                    bytemuck::cast(self.cursor_pos.0 - self.click_pos.0),
                    bytemuck::cast(self.cursor_pos.1 - self.click_pos.1),
                    bytemuck::cast((self.middle && !self.shift) as i32),
                    bytemuck::cast((Local::now().timestamp_millis() - self.boot_time) as i32)
                ]
            ));

            self.wgpu_prog.depth_buffer = DepthBuffer::new(&self.wgpu_config.device, &self.wgpu_config.config, "depth_texture");

            
        } else {
            self.minimized = true;
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                self.click_pos = self.cursor_pos;
                self.middle = true;
                if !self.shift {
                    // println!("Click");
                    self.wgpu_prog.shader_prog.buffers.click_input.updateUniform(&self.wgpu_config.device, bytemuck::cast_slice(
                        &[
                            bytemuck::cast::<_, f32>(self.cursor_pos.0),
                            bytemuck::cast::<_, f32>(self.cursor_pos.1),
                            bytemuck::cast::<_, f32>(0), 
                            bytemuck::cast::<_, f32>(self.ctrl as i32)
                            ]
                        ));
                    self.wgpu_prog.shader_prog.click(&mut self.wgpu_config, &self.settings);
                        
                }
                return true;
            },
            WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, .. } => {
                self.middle = false;

                if !self.shift {
                    // println!("Release");
                    self.wgpu_prog.shader_prog.buffers.release_input.updateUniform(&self.wgpu_config.device, bytemuck::cast_slice(
                        &[
                            bytemuck::cast::<_, f32>(self.cursor_pos.0),
                            bytemuck::cast::<_, f32>(self.cursor_pos.1),
                            2.0*(self.canvas.size.width/self.canvas.size.height) as f32 * (self.cursor_delta.0) as f32/self.canvas.size.width as f32 / self.settings.view.scale,
                            -2.0 as f32 * (self.cursor_delta.1) as f32/self.canvas.size.height as f32 / self.settings.view.scale,
                            bytemuck::cast::<_, f32>(self.settings.simulation.genPerFrame),
                            0.0 as f32,
                            0.0 as f32,
                            0.0 as f32
                            ]
                        ));
                    self.wgpu_prog.shader_prog.release(&mut self.wgpu_config, &self.settings);
                    
                }
                return true;
            }
            WindowEvent::MouseWheel { device_id, delta, phase, modifiers } => {
                let mut mY = 0.0;
                match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        self.zoom(*y);
                    }
                    _ => {}
                }
                return true;
                
            },
            WindowEvent::CursorMoved { position, .. } => {
                let delta = (position.x as i32 - self.cursor_pos.0, position.y as i32 - self.cursor_pos.1);
                let world_delta = self.cursor_del_to_world_delta(delta);
                self.cursor_delta = delta;
                self.cursor_pos = (position.x as i32, position.y as i32);
                self.settings.update_world_pos(self.cursor_to_world_pos(self.cursor_pos), (self.post_ui_x_off(), self.post_ui_y_off()));
                if(self.middle && self.shift){
                    self.xOff += (world_delta.0 as f32);
                    self.yOff += (world_delta.1 as f32);
                }

                let ar = self.canvas.size.width as f32/self.canvas.size.height as f32;
                self.world_delta = (self.world_delta.0 + world_delta.0 / ar, self.world_delta.1 + world_delta.1);
                return true;
            },
            WindowEvent::KeyboardInput { input, .. } => {
                match input {
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::F11),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            self.fullscreen = !self.fullscreen;
                            if self.fullscreen {
                                self.canvas.window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                            } else {
                                self.canvas.window.set_fullscreen(None);
                            }
                            return true;
                        },
                    KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Space),
                            state: ElementState::Pressed,
                            ..
                        } => {
                                // self.temp = 1.0;
                                // self.start_time = Local::now();
                                self.settings.simulating = !self.settings.simulating;
                                return true;
                            },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::B),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            self.backup();   
                            // self.data_length_backup = self.settings.data.len();
                            return true;
                        },
    

                    //SHIFT        
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::LShift),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            self.shift = true;
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::LShift),
                        state: ElementState::Released,
                        ..
                    } => {
                            self.shift = false;
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::RShift),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            self.shift = true;
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::RShift),
                        state: ElementState::Released,
                        ..
                    } => {
                            self.shift = false;
                            return true;
                        },

                    //CTRL    
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::LControl),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            self.ctrl = true;
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::LControl),
                        state: ElementState::Released,
                        ..
                    } => {
                            self.ctrl = false;
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::RControl),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            self.ctrl = true;
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::RControl),
                        state: ElementState::Released,
                        ..
                    } => {
                            self.ctrl = false;
                            return true;
                        },

                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::R),
                        state: ElementState::Pressed,
                        ..
                    } => {
                        if self.shift {
                            self.reset();
                        } else {
                            self.restore();
                        }
                        return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Equals),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            self.zoom(1.0);
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Minus),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            self.zoom(-1.0);
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::H),
                        state: ElementState::Pressed,
                        ..
                    } => {
                           self.home();
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::O),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            if self.ctrl {
                                self.settings.load();
                            }
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::M),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            self.settings.view.settings_menu = !self.settings.view.settings_menu; 
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::L),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            self.HL = !self.HL;
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::W),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            self.W = true;
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::A),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            self.A = true;
                            if self.ctrl {
                                // println!("{}, {}", self.canvas.size.width as i32, self.canvas.size.height as i32);
                                // println!("{}, {}", self.wgpu_prog.shader_prog.hit_tex.dimensions.0 as i32, self.wgpu_prog.shader_prog.hit_tex.dimensions.1 as i32);
                                self.select_all();
                            }
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::S),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            self.S = true;
                            if self.ctrl {
                                self.settings.save()
                            }
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::D),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            self.D = true;
                            self.wgpu_prog.shader_prog.drop(&mut self.wgpu_config, &self.settings);
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::F),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            self.wgpu_prog.shader_prog.fix(&mut self.wgpu_config, &self.settings);
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::T),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            // crate::scripts::script_test(&mut self.wgpu_prog.shader_prog);
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::W),
                        state: ElementState::Released,
                        ..
                    } => {
                            self.W = false;
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::A),
                        state: ElementState::Released,
                        ..
                    } => {
                            self.A = false;
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::S),
                        state: ElementState::Released,
                        ..
                    } => {
                            self.S = false;
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::D),
                        state: ElementState::Released,
                        ..
                    } => {
                            self.D = false;
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Down),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            self.temp -= 1.0;
                            if(self.temp < 0.0){
                                self.temp = 0.0;
                            }
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Left),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            if(self.settings.simulation.genPerFrame > 1){
                                self.settings.simulation.genPerFrame -= 1;
                            } else {
                                self.generations += 10.0;
                            }
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Right),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            if self.settings.simulation.genPerFrame < self.settings.simulation.maxGenPerFrame {
                                self.settings.simulation.genPerFrame += 1;
                            }
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::F3),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            self.log_framerate = !self.log_framerate;
                            Client::clearConsole();
                            return true;
                        },
                    _ => false
                }
            },
            _ => false,
        }
    }

    fn reset(&mut self){
        // self.start_time = Local::now();
        self.wgpu_prog.shader_prog = WGPUComputeProg::new(&mut self.wgpu_config, &mut self.settings, (self.canvas.size.width as u32, self.canvas.size.height as u32));
        self.settings.simulating = false;
        self.generation = 0;
        self.settings.data = Data::new();
    }

    fn backup(&mut self){
        self.wgpu_prog.shader_prog.update_state(&mut self.wgpu_config, &self.settings);
        self.wgpu_prog.shader_prog.state.save(&mut self.wgpu_config, &self.settings);
    }

    fn restore(&mut self){
        self.wgpu_prog.shader_prog.state.load(&mut self.wgpu_config, &mut self.settings, false);
        self.wgpu_prog.shader_prog.restore(&mut self.wgpu_config, &mut self.settings);
        self.settings.data = Data::new();
    }

    fn zoom(&mut self, change: f32){
        
        self.settings.view.scale = (self.settings.view.scale as f32*((2 as f32).powf(change)));
        // self.xOff *= ((2 as f32).powf(change));
        // self.yOff *= ((2 as f32).powf(change));
    }

    fn home(&mut self){
        self.xOff = 0.0;
        self.yOff = 0.0;
    }

    fn select_all(&mut self){
        self.wgpu_prog.shader_prog.buffers.selectangle_input.updateUniform(&self.wgpu_config.device, bytemuck::cast_slice(
            &[
                bytemuck::cast::<_, f32>(0 as i32),
                bytemuck::cast::<_, f32>(0 as i32),
                bytemuck::cast::<_, f32>(self.canvas.size.width as i32),
                bytemuck::cast::<_, f32>(self.canvas.size.height as i32),
            ]
        ));
        self.wgpu_prog.shader_prog.selectangle(&self.wgpu_config, (self.canvas.size.width, self.canvas.size.height));
    }

    fn drag_and_selectangle(&mut self) {
        if self.middle && !self.shift {
            // println!("Drag");
            self.wgpu_prog.shader_prog.buffers.drag_input.updateUniform(&self.wgpu_config.device, bytemuck::cast_slice(
                &[
                    self.world_delta.0,
                    self.world_delta.1,
                    self.canvas.size.width as f32 / self.canvas.size.height as f32,
                    bytemuck::cast::<_, f32>(self.cursor_pos.1),
                    self.settings.simulation.timestep,
                    self.settings.simulation.genPerFrame as f32,
                    bytemuck::cast::<_, f32>(self.settings.simulating as i32)
                ]
            ));
            self.wgpu_prog.shader_prog.drag(&mut self.wgpu_config, &self.settings);
            self.wgpu_prog.shader_prog.buffers.selectangle_input.updateUniform(&self.wgpu_config.device, bytemuck::cast_slice(
                &[
                    bytemuck::cast::<_, f32>(self.click_pos.0),
                    bytemuck::cast::<_, f32>(self.click_pos.1),
                    bytemuck::cast::<_, f32>(self.cursor_pos.0 as i32 - self.click_pos.0 as i32),
                    bytemuck::cast::<_, f32>(self.cursor_pos.1 as i32 - self.click_pos.1 as i32),
                ]
            ));
            self.wgpu_prog.shader_prog.selectangle(&self.wgpu_config, (self.canvas.size.width, self.canvas.size.height));
        }
    }

    fn cursor_del_to_world_delta(&self, cursor_del: (i32, i32)) -> (f32, f32) {
        let scale = self.post_ui_scale();
        let xOff = self.xOff;
        let yOff = self.yOff;
        let w = self.canvas.size.width as f32;
        let h = self.canvas.size.height as f32;

        let viewport_pos = (2.0 * (cursor_del.0 as f32)/w, -2.0 * (cursor_del.1 as f32)/h);
        let ar_corrected = (viewport_pos.0 * (w/h), viewport_pos.1);
        let scaled = (ar_corrected.0/scale, ar_corrected.1/scale);

        return scaled;
    }

    fn cursor_to_world_pos(&self, pos: (i32, i32)) -> (f32, f32) {
        let scale = self.post_ui_scale();
        let xOff = self.xOff;
        let yOff = self.yOff;
        let scale_factor = self.canvas.window.scale_factor() as f32;
        let w = self.canvas.size.width as f32 / scale_factor;
        let h = self.canvas.size.height as f32 / scale_factor;

        let viewport_pos = (2.0 * (pos.0 as f32 - w/2.0)/w, -2.0 * (pos.1 as f32 - h/2.0)/h);
        let ar_corrected = (viewport_pos.0 * (w/h), viewport_pos.1);
        let scaled = (ar_corrected.0/scale, ar_corrected.1/scale);
        let translated = (scaled.0 - self.xOff, scaled.1 - self.yOff);

        return translated;
    }

    fn handle_events(&mut self){ macro_rules! settings { () => { self.settings };}

        //Bond Regen
        if settings!().regen_bonds {
            settings!().regen_bonds = false;
            self.wgpu_prog.shader_prog.update_state(&mut self.wgpu_config, &self.settings);
            self.wgpu_prog.shader_prog.state.regen_bonds(&mut self.wgpu_config, &self.settings);
            self.wgpu_prog.shader_prog.state.save(&mut self.wgpu_config, &self.settings);
            self.wgpu_prog.shader_prog.state.load(&mut self.wgpu_config, &mut self.settings, false);
            self.wgpu_prog.shader_prog.restore(&mut self.wgpu_config, &mut self.settings);
        }

        //Set Properties
        if settings!().set_properties {
            settings!().set_properties = false;
            self.wgpu_prog.shader_prog.buffers.set_prop_input.updateUniform(&self.wgpu_config.device, bytemuck::cast_slice(&settings!().properties()));
            self.wgpu_prog.shader_prog.set_properties(&self.wgpu_config, &self.settings);
        }

        if settings!().backup         { self.backup();                                                                    settings!().backup         = false }
        if settings!().reset          { self.reset();                                                                     settings!().reset          = false }
        if settings!().zoom_in        { self.zoom(1.0);                                                                   settings!().zoom_in        = false }
        if settings!().zoom_out       { self.zoom(-1.0);                                                                  settings!().zoom_out       = false }
        if settings!().home           { self.home();                                                                      settings!().home           = false }
        if settings!().select_all     { self.select_all();                                                                settings!().select_all     = false }
        if settings!().update_shaders { self.wgpu_prog.shader_prog.update_shaders(&mut self.wgpu_config, &self.settings); settings!().update_shaders = false }
        if settings!().fix            { self.wgpu_prog.shader_prog.fix(&mut self.wgpu_config, &self.settings);            settings!().fix            = false }
        if settings!().drop           { self.wgpu_prog.shader_prog.drop(&mut self.wgpu_config, &self.settings);           settings!().drop           = false }

        self.drag_and_selectangle();
    }

    fn post_ui_x_off(&self) -> f32 {
        let scale_factor = self.canvas.window.scale_factor() as f32;
        let center = self.canvas.size.width as f32 / (2.0);
        let left = self.available_rect.left() * scale_factor;
        let right = self.available_rect.right() * scale_factor;
        let new_center = (right - left)/2.0 + left;
        let world_offset = self.cursor_del_to_world_delta(((new_center - center) as i32, 0));
        return world_offset.0 as f32;
    }

    fn post_ui_y_off(&self) -> f32 {
        let scale_factor = self.canvas.window.scale_factor() as f32;
        let center = self.canvas.size.height as f32 / (2.0);
        let top = self.available_rect.top() * scale_factor;
        let bottom = self.available_rect.bottom() * scale_factor;
        let new_center = ((bottom - top)/2.0 + top);
        let world_offset = self.cursor_del_to_world_delta((0, (new_center - center) as i32));
        return world_offset.1 as f32;
    }

    fn post_ui_scale(&self) -> f32 {
        return self.canvas.window.scale_factor() as f32 * self.settings.view.scale * self.available_rect.height()/self.canvas.size.height as f32;
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> { macro_rules! settings { () => { self.settings };}
        self.handle_events();
        self.script_manager.execute(&mut self.wgpu_prog, &mut self.wgpu_config, &mut self.settings, &self.canvas);
        let max_framerate = self.canvas.window.current_monitor().unwrap().refresh_rate_millihertz().unwrap() as f32/1000.0;
        settings!().simulation.maxGenPerFrame = ((1.0/settings!().simulation.timestep)/max_framerate).round() as i32;
        if settings!().simulation.maxGenPerFrame < settings!().simulation.genPerFrame {
            settings!().simulation.genPerFrame = settings!().simulation.maxGenPerFrame;
        }
        if max_framerate != settings!().hz {
            settings!().hz = max_framerate;
        }

        self.cursor_delta = ( 0 ,  0 );
        self.world_delta  = (0.0, 0.0);
        // Compute
        
        if settings!().simulating {
            if settings!().changed_collision_settings {
                self.wgpu_prog.shader_prog.buffers.collision_settings.updateUniform(&self.wgpu_config.device, bytemuck::cast_slice(&settings!().collision_settings()));
            }
            // for i in 0..settings!().simulation.genPerFrame {
            self.wgpu_prog.shader_prog.compute(&mut self.wgpu_config, &self.settings);
            self.generation += settings!().simulation.genPerFrame;
            // }
        }

        //Handle saving/loading
        if settings!().save && settings!().current_file.file_name().is_some() {
            settings!().save = false;
            self.wgpu_prog.shader_prog.update_state(&mut self.wgpu_config, &self.settings);
            self.wgpu_prog.shader_prog.state.save(&mut self.wgpu_config, &self.settings);
            self.wgpu_prog.shader_prog.state.save_to_file(settings!().current_file.clone());
        }
    
        if settings!().load && settings!().current_file.file_name().is_some() {
            self.wgpu_prog.shader_prog.state.load_from_file(settings!().current_file.clone());
            self.wgpu_prog.shader_prog.state.load(&mut self.wgpu_config, &mut self.settings, true);
            self.wgpu_prog.shader_prog.restore(&mut self.wgpu_config, &mut self.settings);
            settings!().load = false;
        }

        if settings!().restore    { self.restore();                                         settings!().restore    = false }


        // UI
        if !self.minimized {
            
            self.platform.update_time((Local::now().timestamp_millis() - self.start_time.timestamp_millis()) as f64 / 1000.0);
            
            let output_frame = self.wgpu_config.surface.get_current_texture().unwrap();
            let output_view = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        
        // Begin to draw the UI frame.
        // if self.platform
        let scale_factor = self.canvas.window.scale_factor() as f32;
        self.platform.raw_input_mut().screen_rect = Some(
            egui::Rect::from_min_size(
                egui::Pos2::new(0.0, 0.0), 
                egui::Vec2::new(
                    self.canvas.size.width as f32 / scale_factor, 
                    self.canvas.size.height as f32 / scale_factor)
                )
        );
        self.platform.begin_frame();
        let needs_reset = settings!().ui(&self.platform.context(), &mut self.wgpu_prog, &mut self.script_manager, &mut self.wgpu_config, (self.canvas.size.width, self.canvas.size.height));
        self.available_rect = self.platform.context().available_rect();
        if needs_reset {
            self.reset();
        }
        
        if settings!().materials_changed {
            self.wgpu_prog.shader_prog.buffers.material_buffer.updateUniform(&self.wgpu_config.device, bytemuck::cast_slice(&settings!().materials));
        }
        
            self.wgpu_prog.dim_uniform.updateUniform(&self.wgpu_config.device, bytemuck::cast_slice(
                &[  self.wgpu_config.size.width as f32,
                    0.0 as f32, //time as f32, 
                    self.wgpu_config.size.height as f32,
                    self.temp,
                    self.xOff as f32,
                    self.yOff as f32,
                    self.post_ui_x_off(),
                    self.post_ui_y_off(),
                    self.post_ui_scale(),
                    self.dark as f32,
                    bytemuck::cast(self.click_pos.0),
                    bytemuck::cast(self.click_pos.1),
                    bytemuck::cast(self.cursor_pos.0 - self.click_pos.0),
                    bytemuck::cast(self.cursor_pos.1 - self.click_pos.1),
                    bytemuck::cast((self.middle && !self.shift) as i32),
                    bytemuck::cast((Local::now().timestamp_millis() - self.boot_time) as i32)
                ]
            ));   
            
            let full_output = self.platform.end_frame(Some(&self.canvas.window));
            let paint_jobs = self.platform.context().tessellate(full_output.shapes);

            self.wgpu_prog.ren_set_uniform.updateUniform(&self.wgpu_config.device, bytemuck::cast_slice(&settings!().render_settings()));

            let mut encoder = self.wgpu_config.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });
            if settings!().view.rendering {
                {
                    let mut render_pass2 = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[
                            Some(wgpu::RenderPassColorAttachment {
                                view: &self.wgpu_prog.render_tex.view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(self.wgpu_prog.clear_color),
                                    store: true,
                                }
                            })
                            ],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &self.wgpu_prog.depth_buffer.view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: true,
                            }),
                            stencil_ops: None,
                        }),
                    });

                    render_pass2.set_pipeline(&self.wgpu_prog.render_pipelines[1]);
                    render_pass2.set_bind_group(0, &self.wgpu_prog.dim_uniform.bind_group, &[]);
                    render_pass2.set_bind_group(1, &self.wgpu_prog.shader_prog.buffers.pos_buffers.bind_group, &[]);
                    // render_pass2.set_bind_group(3, &self.wgpu_prog.shader_prog.color_buffer.bind_group, &[]);
                    render_pass2.set_bind_group(2, &self.wgpu_prog.shader_prog.buffers.mov_buffers.bind_group, &[]);
                    render_pass2.set_bind_group(3, &self.wgpu_prog.shader_prog.buffers.contact_buffers.bind_group, &[]);
                    render_pass2.set_bind_group(4, &self.wgpu_prog.ren_set_uniform.bind_group, &[]);
                    render_pass2.set_bind_group(5, &self.wgpu_prog.shader_prog.buffers.material_buffer.bind_group, &[]);
                    render_pass2.set_bind_group(6, &self.wgpu_prog.shader_prog.buffers.selection_buffers.bind_group, &[]);
                    render_pass2.set_bind_group(7, &self.wgpu_prog.shader_prog.buffers.click_buffer.bind_group, &[]);
                    render_pass2.set_vertex_buffer(0, self.wgpu_prog.vertex_buffer.slice(..));
                    render_pass2.set_index_buffer(self.wgpu_prog.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass2.draw_indexed(0..6 as u32, 0, 0..1);
                }

                {
                    let mut render_pass3 = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[
                            Some(wgpu::RenderPassColorAttachment {
                                view: &self.wgpu_prog.shader_prog.hit_tex.view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(self.wgpu_prog.clear_color),
                                    store: true,
                                }
                            })
                            ],
                        depth_stencil_attachment: None,
                    });

                    render_pass3.set_pipeline(&self.wgpu_prog.render_pipelines[2]);
                    render_pass3.set_bind_group(0, &self.wgpu_prog.dim_uniform.bind_group, &[]);
                    render_pass3.set_bind_group(1, &self.wgpu_prog.shader_prog.buffers.pos_buffers.bind_group, &[]);
                    render_pass3.set_bind_group(2, &self.wgpu_prog.shader_prog.buffers.mov_buffers.bind_group, &[]);
                    render_pass3.set_bind_group(3, &self.wgpu_prog.shader_prog.buffers.contact_buffers.bind_group, &[]);
                    render_pass3.set_bind_group(4, &self.wgpu_prog.ren_set_uniform.bind_group, &[]);
                    render_pass3.set_bind_group(5, &self.wgpu_prog.shader_prog.buffers.material_buffer.bind_group, &[]);
                    render_pass3.set_bind_group(6, &self.wgpu_prog.shader_prog.buffers.selection_buffers.bind_group, &[]);
                    render_pass3.set_bind_group(7, &self.wgpu_prog.shader_prog.buffers.click_buffer.bind_group, &[]);
                    render_pass3.set_vertex_buffer(0, self.wgpu_prog.vertex_buffer.slice(..));
                    render_pass3.set_index_buffer(self.wgpu_prog.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    
                    render_pass3.draw_indexed(0..6 as u32, 0, 0..settings!().setup.particles as u32);
                }

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[
                            Some(wgpu::RenderPassColorAttachment {
                                view: &self.wgpu_prog.render_tex.view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: true,
                                }
                            })
                        ],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &self.wgpu_prog.depth_buffer.view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: true,
                            }),
                            stencil_ops: None,
                        }),
                    });

                    render_pass.set_pipeline(&self.wgpu_prog.render_pipelines[0]);
                    render_pass.set_bind_group(0, &self.wgpu_prog.dim_uniform.bind_group, &[]);
                    render_pass.set_bind_group(1, &self.wgpu_prog.shader_prog.buffers.pos_buffers.bind_group, &[]);
                    // render_pass.set_bind_group(3, &self.wgpu_prog.shader_prog.color_buffer.bind_group, &[]);
                    render_pass.set_bind_group(2, &self.wgpu_prog.shader_prog.buffers.mov_buffers.bind_group, &[]);
                    render_pass.set_bind_group(3, &self.wgpu_prog.shader_prog.buffers.contact_buffers.bind_group, &[]);
                    render_pass.set_bind_group(4, &self.wgpu_prog.ren_set_uniform.bind_group, &[]);
                    render_pass.set_bind_group(5, &self.wgpu_prog.shader_prog.buffers.material_buffer.bind_group, &[]);
                    render_pass.set_bind_group(6, &self.wgpu_prog.shader_prog.buffers.selection_buffers.bind_group, &[]);
                    render_pass.set_bind_group(7, &self.wgpu_prog.shader_prog.buffers.click_buffer.bind_group, &[]);
                    render_pass.set_vertex_buffer(0, self.wgpu_prog.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(self.wgpu_prog.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..6 as u32, 0, 0..settings!().setup.particles as u32);
                    
                }

                {
                    let mut render_pass4 = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[
                            Some(wgpu::RenderPassColorAttachment {
                                view: &output_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(self.wgpu_prog.clear_color),
                                    store: true,
                                }
                            })
                            ],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &self.wgpu_prog.depth_buffer.view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: true,
                            }),
                            stencil_ops: None,
                        }),
                    });

                    render_pass4.set_pipeline(&self.wgpu_prog.render_pipelines[3]);
                    render_pass4.set_bind_group(0, &self.wgpu_prog.render_tex.diffuse_bind_group, &[]);
                    render_pass4.set_bind_group(1, &self.wgpu_prog.ren_set_uniform.bind_group, &[]);
                    render_pass4.set_bind_group(2, &self.wgpu_prog.dim_uniform.bind_group, &[]);
                    render_pass4.set_vertex_buffer(0, self.wgpu_prog.vertex_buffer.slice(..));
                    render_pass4.set_index_buffer(self.wgpu_prog.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass4.draw_indexed(0..6 as u32, 0, 0..1);
                }
            }

            // Upload all resources for the GPU.
            let screen_descriptor = ScreenDescriptor {
                physical_width: self.canvas.size.width,
                physical_height: self.canvas.size.height,
                scale_factor: self.canvas.window.scale_factor() as f32,
            };
            let tdelta: egui::TexturesDelta = full_output.textures_delta;
            self.egui_rpass
                .add_textures(&self.wgpu_config.device, &self.wgpu_config.queue, &tdelta)
                .expect("add texture ok");
            self.egui_rpass.update_buffers(&self.wgpu_config.device, &self.wgpu_config.queue, &paint_jobs, &screen_descriptor);
            
            self.egui_rpass
            .execute(
                &mut encoder,
                &output_view,
                &paint_jobs,
                &screen_descriptor,
                None,
            )
            .unwrap();
        
        self.wgpu_config.queue.submit(iter::once(encoder.finish()));
        
        output_frame.present();
        
        self.egui_rpass
        .remove_textures(tdelta)
        .expect("remove texture ok");
    }

    // // println!("{}", self.platform.context().pixels_per_point());
    // if self.platform.context().pixels_per_point() < 2.0 {
    //     self.platform.context().set_pixels_per_point(self.platform.context().pixels_per_point() + 0.1);
    //     self.platform.context().request_repaint();
    // //     self.resize(self.canvas.size);
    // }

    let now = Local::now();
    let sim_time_passed = settings!().simulation.timestep*self.generation as f32;    
    settings!().sim_time = sim_time_passed;

    if settings!().simulating && settings!().gather_data || settings!().recording {
        self.wgpu_prog.shader_prog.update_state(&mut self.wgpu_config, &self.settings);
        match self.wgpu_prog.shader_prog.state.get_datum(&settings!().plotted_prop) {
            Some(datum) => {settings!().data.push(sim_time_passed as f64, datum, self.settings.fps as f64);},
            None => {settings!().data = Data::new();}
        }
    }

    settings!().fps = 1000000.0/(now.timestamp_micros() - self.last_draw.timestamp_micros()) as f32;

    if(self.log_framerate){
        
    let time_since = (now.timestamp_millis() - self.bench_start_time.timestamp_millis()) as f32/1000.0;
    
    if(time_since >= 0.25){
        Client::clearConsole();
        #[cfg(not(target_arch = "wasm32"))] {
            println!("FPS: {}", settings!().fps);
        }
        #[cfg(target_arch = "wasm32")] {
            log::warn!("FPS: {}", settings!().fps);
        }
        let mut time_passed = (Local::now().timestamp_millis() - self.start_time.timestamp_millis()) as f32/1000.0;
        if !settings!().simulating { time_passed = 0.0; }
        let genPerSec = (self.generation - self.prevGen) as f32/time_since;
                let sim_speed = 100.0*genPerSec*settings!().simulation.timestep;
                let twsp = 100.0*20.0/sim_speed;
                println!("Generations/s: {}, Total Generations: {}", genPerSec, self.generation);
                println!("Elapsed Time: {} seconds", time_passed);
                println!("Elapsed Time(Sim): {} seconds, % Real Speed: {}", sim_time_passed, sim_speed);
                println!("20 Sec Proj: {}:{}:{}", (twsp/3600.0) as i32, ((twsp/60.0)%60.0) as i32, twsp%60.0);
                println!("Particles: {}", settings!().setup.particles);
                println!("Generations/Frame: {}", settings!().simulation.genPerFrame as f32);
                println!("Scale: {}, (xOff, yOff): ({}, {})", settings!().view.scale as f32, self.xOff, self.yOff);
                self.prevGen = self.generation;
                self.bench_start_time = Local::now();
                
                // self.wgpu_prog.shader_prog.state.print_state();
            }
            
        }
        
        self.last_draw = now;

        Ok(())
    }

    fn clearConsole(){
        #[cfg(not(target_arch = "wasm32"))] {
            print!("\x1B[2J\x1B[1;1H");
        }
        #[cfg(target_arch = "wasm32")] {
            web_sys::console::clear();
        }
    }
}