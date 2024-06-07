use crate::settings::Data;
use crate::wgpu_prog::WGPUComputeProg;
use crate::wgpu_structs::DepthBuffer;
use crate::wgpu_structs::Texture;
use crate::window_init;
use crate::wgpu_config::*;
use crate::wgpu_prog;

use crate::wgpu_prog::WGPUProg;
use cgmath::Angle;
use egui_demo_lib::DemoWindows;
use winit::window::Fullscreen;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::WindowBuilder, dpi::PhysicalSize,
};
use std::cmp::max;
use std::iter;
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
    minimized: bool,
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
        let wgpu_prog = WGPUProg::new(&mut wgpu_config, (canvas.size.width as u32, canvas.size.height as u32));
        
        // UI Setup

        let size = canvas.size;
        let platform = Platform::new(PlatformDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: canvas.window.scale_factor(),
            font_definitions: egui::FontDefinitions::default(),
            style: Default::default(),
        });
        platform.context().set_pixels_per_point(2.0);
        let mut egui_rpass = RenderPass::new(&wgpu_config.device, wgpu_config.surface_format, 1);
        let max_framerate = canvas.window.current_monitor().unwrap().refresh_rate_millihertz().unwrap() as f32/1000.0;

        let mut client = Client {
            canvas,
            wgpu_config,
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
            minimized: false,
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
            // max_framerate:  max_framerate,
            // prev_framerate: max_framerate
        };
        
        client.resize(client.canvas.size);
        
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
            let int_scale = self.wgpu_config.prog_settings.scale as f32;

            self.wgpu_prog.dim_uniform.updateUniform(&self.wgpu_config.device, bytemuck::cast_slice(
                &[self.wgpu_config.size.width as f32,
                  self.wgpu_config.size.width as f32, 
                  self.wgpu_config.size.height as f32,
                  self.wgpu_config.size.height as f32,
                  self.xOff as f32,
                  self.yOff as f32,
                  self.wgpu_config.prog_settings.scale as f32,
                  self.dark as f32]
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
                    self.wgpu_prog.shader_prog.click(&mut self.wgpu_config);
                        
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
                            2.0*(self.canvas.size.width/self.canvas.size.height) as f32 * (self.cursor_delta.0) as f32/self.canvas.size.width as f32 / self.wgpu_config.prog_settings.scale,
                            -2.0 as f32 * (self.cursor_delta.1) as f32/self.canvas.size.height as f32 / self.wgpu_config.prog_settings.scale,
                            bytemuck::cast::<_, f32>(self.wgpu_config.prog_settings.genPerFrame),
                            0.0 as f32,
                            0.0 as f32,
                            0.0 as f32
                            ]
                        ));
                    self.wgpu_prog.shader_prog.release(&mut self.wgpu_config);
                    
                }
                return true;
            }
            WindowEvent::MouseWheel { device_id, delta, phase, modifiers } => {
                let mut mY = 0.0;
                match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        mY = *y;
                        self.wgpu_config.prog_settings.scale = (self.wgpu_config.prog_settings.scale as f32*((2 as f32).powf(mY)));
                        self.xOff *= ((2 as f32).powf(mY));
                        self.yOff *= ((2 as f32).powf(mY));
                    }
                    _ => {}
                }
                return true;
                
            },
            WindowEvent::CursorMoved { position, .. } => {
                let delta = (position.x as i32 - self.cursor_pos.0, position.y as i32 - self.cursor_pos.1);
                self.cursor_delta = delta;
                self.cursor_pos = (position.x as i32, position.y as i32);
                if(self.middle && self.shift){
                    self.xOff += (delta.0 as f32) as f32;
                    self.yOff += (delta.1 as f32) as f32;
                } else if self.middle && !self.shift {
                    // println!("Drag");
                    self.wgpu_prog.shader_prog.buffers.drag_input.updateUniform(&self.wgpu_config.device, bytemuck::cast_slice(
                        &[
                            2.0*(self.canvas.size.width/self.canvas.size.height) as f32 * (delta.0) as f32/self.canvas.size.width as f32 / self.wgpu_config.prog_settings.scale,
                            -2.0 as f32 * (delta.1) as f32/self.canvas.size.height as f32 / self.wgpu_config.prog_settings.scale,
                            self.canvas.size.width as f32 / self.canvas.size.height as f32,
                            bytemuck::cast::<_, f32>(self.cursor_pos.1),
                        ]
                    ));
                    self.wgpu_prog.shader_prog.drag(&mut self.wgpu_config);
                    self.wgpu_prog.shader_prog.buffers.selectangle_input.updateUniform(&self.wgpu_config.device, bytemuck::cast_slice(
                        &[
                            bytemuck::cast::<_, f32>(self.click_pos.0),
                            bytemuck::cast::<_, f32>(self.click_pos.1),
                            bytemuck::cast::<_, f32>(self.cursor_pos.0 as i32 - self.click_pos.0 as i32),
                            bytemuck::cast::<_, f32>(self.cursor_pos.1 as i32 - self.click_pos.1 as i32),
                        ]
                    ));
                    self.wgpu_prog.shader_prog.selectangle(&mut self.wgpu_config, (self.canvas.size.width, self.canvas.size.height));
                }
                return true;
            },
            WindowEvent::KeyboardInput { input, .. } => {
                match input {
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::F11),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            self.canvas.window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                            return true;
                        },
                    KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Space),
                            state: ElementState::Pressed,
                            ..
                        } => {
                                // self.temp = 1.0;
                                // self.start_time = Local::now();
                                self.wgpu_config.prog_settings.simulating = !self.wgpu_config.prog_settings.simulating;
                                return true;
                            },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::B),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            self.backup();   
                            // self.data_length_backup = self.wgpu_config.prog_settings.data.len();
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
                            self.zoom_in();
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Minus),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            self.zoom_out();
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
                                self.wgpu_config.prog_settings.load();
                            }
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::M),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            self.wgpu_config.prog_settings.settings_menu = !self.wgpu_config.prog_settings.settings_menu; 
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
                                self.wgpu_config.prog_settings.save()
                            }
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::D),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            self.D = true;
                            self.wgpu_prog.shader_prog.drop(&mut self.wgpu_config);
                            return true;
                        },
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::F),
                        state: ElementState::Pressed,
                        ..
                    } => {
                            self.wgpu_prog.shader_prog.fix(&mut self.wgpu_config);
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
                            if(self.wgpu_config.prog_settings.genPerFrame > 1){
                                self.wgpu_config.prog_settings.genPerFrame -= 1;
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
                            if self.wgpu_config.prog_settings.genPerFrame < self.wgpu_config.prog_settings.maxGenPerFrame {
                                self.wgpu_config.prog_settings.genPerFrame += 1;
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
        self.wgpu_prog.shader_prog = WGPUComputeProg::new(&mut self.wgpu_config, (self.canvas.size.width as u32, self.canvas.size.height as u32));
        self.wgpu_config.prog_settings.simulating = false;
        self.generation = 0;
        self.wgpu_config.prog_settings.data = Data::new();
    }

    fn backup(&mut self){
        self.wgpu_prog.shader_prog.update_state(&mut self.wgpu_config);
        self.wgpu_prog.shader_prog.state.save(&mut self.wgpu_config);
    }

    fn restore(&mut self){
        self.wgpu_prog.shader_prog.state.load(&mut self.wgpu_config, false);
        self.wgpu_prog.shader_prog.restore(&mut self.wgpu_config);
        self.wgpu_config.prog_settings.data = Data::new();
    }

    fn zoom_in(&mut self){
        self.wgpu_config.prog_settings.scale = (self.wgpu_config.prog_settings.scale as f32*((2 as f32).powf(1.0)));
        self.xOff *= ((2 as f32).powf(1.0));
        self.yOff *= ((2 as f32).powf(1.0));
    }

    fn zoom_out(&mut self){
        self.wgpu_config.prog_settings.scale = (self.wgpu_config.prog_settings.scale as f32*((2 as f32).powf(-1.0)));
        self.xOff *= ((2 as f32).powf(-1.0));
        self.yOff *= ((2 as f32).powf(-1.0));
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
        self.wgpu_prog.shader_prog.selectangle(&mut self.wgpu_config, (self.canvas.size.width, self.canvas.size.height));
    }

    fn handle_events(&mut self){
        macro_rules! settings { () => { self.wgpu_config.prog_settings };}

        //Handle saving/loading
        if settings!().save && settings!().current_file.file_name().is_some() {
            settings!().save = false;
            self.wgpu_prog.shader_prog.update_state(&mut self.wgpu_config);
            self.wgpu_prog.shader_prog.state.save(&mut self.wgpu_config);
            self.wgpu_prog.shader_prog.state.save_to_file(settings!().current_file.clone());
        }
    
        if settings!().load && settings!().current_file.file_name().is_some() {
            settings!().load = false;
            self.wgpu_prog.shader_prog.state.load_from_file(settings!().current_file.clone());
            self.wgpu_prog.shader_prog.state.load(&mut self.wgpu_config, true);
            // self.wgpu_config.prog_settings.set_particles(self.wgpu_prog.shader_prog.state.p_count);
            // self.reset();
            self.wgpu_prog.shader_prog.restore(&mut self.wgpu_config);
        }

        //Bond Regen
        if settings!().regen_bonds {
            settings!().regen_bonds = false;
            self.wgpu_prog.shader_prog.update_state(&mut self.wgpu_config);
            self.wgpu_prog.shader_prog.state.regen_bonds(&mut self.wgpu_config);
            self.wgpu_prog.shader_prog.state.save(&mut self.wgpu_config);
            self.wgpu_prog.shader_prog.state.load(&mut self.wgpu_config, false);
            self.wgpu_prog.shader_prog.restore(&mut self.wgpu_config);
        }

        //Set Properties
        if settings!().set_properties {
            settings!().set_properties = false;
            self.wgpu_prog.shader_prog.buffers.set_prop_input.updateUniform(&self.wgpu_config.device, bytemuck::cast_slice(&settings!().properties()));
            self.wgpu_prog.shader_prog.set_properties(&mut self.wgpu_config);
        }

        if settings!().backup     { self.backup();                                          settings!().backup     = false }
        if settings!().restore    { self.restore();                                         settings!().restore    = false }
        if settings!().reset      { self.reset();                                           settings!().reset      = false }
        if settings!().zoom_in    { self.zoom_in();                                         settings!().zoom_in    = false }
        if settings!().zoom_out   { self.zoom_out();                                        settings!().zoom_out   = false }
        if settings!().home       { self.home();                                            settings!().home       = false }
        if settings!().select_all { self.select_all();                                      settings!().select_all = false }
        if settings!().fix        { self.wgpu_prog.shader_prog.fix(&mut self.wgpu_config);  settings!().fix        = false }
        if settings!().drop       { self.wgpu_prog.shader_prog.drop(&mut self.wgpu_config); settings!().drop       = false }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        macro_rules! settings { () => { self.wgpu_config.prog_settings };}
        let max_framerate = self.canvas.window.current_monitor().unwrap().refresh_rate_millihertz().unwrap() as f32/1000.0;
        settings!().maxGenPerFrame = ((1.0/settings!().timestep)/max_framerate).round() as i32;
        if settings!().maxGenPerFrame < settings!().genPerFrame {
            settings!().genPerFrame = settings!().maxGenPerFrame;
        }
        if max_framerate != settings!().hz {
            settings!().hz = max_framerate;
        }

        self.cursor_delta = (0, 0);

        // Compute

        if settings!().simulating {
            if settings!().changed_collision_settings {
                self.wgpu_prog.shader_prog.buffers.collision_settings.updateUniform(&self.wgpu_config.device, bytemuck::cast_slice(&settings!().collison_settings()));
            }
            // for i in 0..settings!().genPerFrame {
            self.wgpu_prog.shader_prog.compute(&mut self.wgpu_config);
            self.generation += settings!().genPerFrame;
            // }
        }

        // UI
        if !self.minimized {

            self.platform.update_time((Local::now().timestamp_millis() - self.start_time.timestamp_millis()) as f64 / 1000.0);
            
            let output_frame = self.wgpu_config.surface.get_current_texture().unwrap();
            let output_view = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

            self.handle_events();

            // Begin to draw the UI frame.
            self.platform.begin_frame();
            let needs_reset = settings!().ui(&self.platform.context(), &self.wgpu_prog.shader_prog.state);
            if needs_reset {
                self.reset();
            }
            
            if settings!().materials_changed {
                self.wgpu_prog.shader_prog.buffers.material_buffer.updateUniform(&self.wgpu_config.device, bytemuck::cast_slice(&settings!().materials));
            }

            self.wgpu_prog.dim_uniform.updateUniform(&self.wgpu_config.device, bytemuck::cast_slice(
                &[self.wgpu_config.size.width as f32,
                0.0 as f32, //time as f32, 
                self.wgpu_config.size.height as f32,
                self.temp,
                self.xOff as f32,
                self.yOff as f32,
                settings!().scale as f32,
                self.dark as f32,
                bytemuck::cast(self.click_pos.0),
                bytemuck::cast(self.click_pos.1),
                bytemuck::cast(self.cursor_pos.0 - self.click_pos.0),
                bytemuck::cast(self.cursor_pos.1 - self.click_pos.1),
                bytemuck::cast((self.middle && !self.shift) as i32),
                ]
            ));   
            
            let full_output = self.platform.end_frame(Some(&self.canvas.window));
            let paint_jobs = self.platform.context().tessellate(full_output.shapes);

            self.wgpu_prog.ren_set_uniform.updateUniform(&self.wgpu_config.device, bytemuck::cast_slice(&settings!().render_settings()));

            let mut encoder = self.wgpu_config.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });
            {
                let mut render_pass2 = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

                render_pass2.set_pipeline(&self.wgpu_prog.render_pipeline2);
                render_pass2.set_bind_group(0, &self.wgpu_prog.dim_uniform.bind_group, &[]);
                render_pass2.set_bind_group(1, &self.wgpu_prog.shader_prog.buffers.pos_buffers.bind_group, &[]);
                // render_pass2.set_bind_group(3, &self.wgpu_prog.shader_prog.color_buffer.bind_group, &[]);
                render_pass2.set_bind_group(2, &self.wgpu_prog.shader_prog.buffers.mov_buffers.bind_group, &[]);
                render_pass2.set_bind_group(3, &self.wgpu_prog.shader_prog.buffers.contact_buffers.bind_group, &[]);
                render_pass2.set_bind_group(4, &self.wgpu_prog.ren_set_uniform.bind_group, &[]);
                render_pass2.set_bind_group(5, &self.wgpu_prog.shader_prog.buffers.material_buffer.bind_group, &[]);
                render_pass2.set_bind_group(6, &self.wgpu_prog.shader_prog.buffers.selections.bind_group, &[]);
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

                render_pass3.set_pipeline(&self.wgpu_prog.render_pipeline3);
                render_pass3.set_bind_group(0, &self.wgpu_prog.dim_uniform.bind_group, &[]);
                render_pass3.set_bind_group(1, &self.wgpu_prog.shader_prog.buffers.pos_buffers.bind_group, &[]);
                render_pass3.set_bind_group(2, &self.wgpu_prog.shader_prog.buffers.mov_buffers.bind_group, &[]);
                render_pass3.set_bind_group(3, &self.wgpu_prog.shader_prog.buffers.contact_buffers.bind_group, &[]);
                render_pass3.set_bind_group(4, &self.wgpu_prog.ren_set_uniform.bind_group, &[]);
                render_pass3.set_bind_group(5, &self.wgpu_prog.shader_prog.buffers.material_buffer.bind_group, &[]);
                render_pass3.set_bind_group(6, &self.wgpu_prog.shader_prog.buffers.selections.bind_group, &[]);
                render_pass3.set_bind_group(7, &self.wgpu_prog.shader_prog.buffers.click_buffer.bind_group, &[]);
                render_pass3.set_vertex_buffer(0, self.wgpu_prog.vertex_buffer.slice(..));
                render_pass3.set_index_buffer(self.wgpu_prog.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                
                render_pass3.draw_indexed(0..6 as u32, 0, 0..settings!().particles as u32);
            }

            // {
            //     let mut render_pass4 = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            //         label: Some("Render Pass"),
            //         color_attachments: &[
            //             Some(wgpu::RenderPassColorAttachment {
            //                 view: &output_view,//self.wgpu_prog.shader_prog.hit_tex.view,
            //                 resolve_target: None,
            //                 ops: wgpu::Operations {
            //                     load: wgpu::LoadOp::Clear(self.wgpu_prog.clear_color),
            //                     store: true,
            //                 }
            //             })
            //             ],
            //         depth_stencil_attachment: None,
            //     });

            //     render_pass4.set_pipeline(&self.wgpu_prog.render_pipeline4);
            //     render_pass4.set_bind_group(0, &self.wgpu_prog.dim_uniform.bind_group, &[]);
            //     render_pass4.set_bind_group(1, &self.wgpu_prog.shader_prog.buffers.pos_buffers.bind_group, &[]);
            //     render_pass4.set_bind_group(2, &self.wgpu_prog.shader_prog.buffers.mov_buffers.bind_group, &[]);
            //     render_pass4.set_bind_group(3, &self.wgpu_prog.shader_prog.buffers.contact_buffers.bind_group, &[]);
            //     render_pass4.set_bind_group(4, &self.wgpu_prog.ren_set_uniform.bind_group, &[]);
            //     render_pass4.set_bind_group(5, &self.wgpu_prog.shader_prog.buffers.material_buffer.bind_group, &[]);
            //     render_pass4.set_bind_group(6, &self.wgpu_prog.shader_prog.buffers.selections.bind_group, &[]);
            //     render_pass4.set_bind_group(7, &self.wgpu_prog.shader_prog.buffers.click_buffer.bind_group, &[]);

            //     render_pass4.set_vertex_buffer(0, self.wgpu_prog.vertex_buffer.slice(..));
            //     render_pass4.set_index_buffer(self.wgpu_prog.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                
            //     render_pass4.draw_indexed(0..6 as u32, 0, 0..settings!().particles as u32);
            // }

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: &output_view,
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

                render_pass.set_pipeline(&self.wgpu_prog.render_pipeline);
                render_pass.set_bind_group(0, &self.wgpu_prog.dim_uniform.bind_group, &[]);
                render_pass.set_bind_group(1, &self.wgpu_prog.shader_prog.buffers.pos_buffers.bind_group, &[]);
                // render_pass.set_bind_group(3, &self.wgpu_prog.shader_prog.color_buffer.bind_group, &[]);
                render_pass.set_bind_group(2, &self.wgpu_prog.shader_prog.buffers.mov_buffers.bind_group, &[]);
                render_pass.set_bind_group(3, &self.wgpu_prog.shader_prog.buffers.contact_buffers.bind_group, &[]);
                render_pass.set_bind_group(4, &self.wgpu_prog.ren_set_uniform.bind_group, &[]);
                render_pass.set_bind_group(5, &self.wgpu_prog.shader_prog.buffers.material_buffer.bind_group, &[]);
                render_pass.set_bind_group(6, &self.wgpu_prog.shader_prog.buffers.selections.bind_group, &[]);
                render_pass.set_bind_group(7, &self.wgpu_prog.shader_prog.buffers.click_buffer.bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.wgpu_prog.vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.wgpu_prog.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..6 as u32, 0, 0..settings!().particles as u32);
                
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

    let now = Local::now();
    let sim_time_passed = settings!().timestep*self.generation as f32;    
    settings!().sim_time = sim_time_passed;

    if settings!().simulating && settings!().gather_data || settings!().recording {
        self.wgpu_prog.shader_prog.update_state(&mut self.wgpu_config);
        match self.wgpu_prog.shader_prog.state.get_datum(&settings!().plotted_prop) {
            Some(datum) => {settings!().data.push(sim_time_passed as f64, datum, self.wgpu_config.prog_settings.fps as f64);},
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
                let sim_speed = 100.0*genPerSec*settings!().timestep;
                let twsp = 100.0*20.0/sim_speed;
                println!("Generations/s: {}, Total Generations: {}", genPerSec, self.generation);
                println!("Elapsed Time: {} seconds", time_passed);
                println!("Elapsed Time(Sim): {} seconds, % Real Speed: {}", sim_time_passed, sim_speed);
                println!("20 Sec Proj: {}:{}:{}", (twsp/3600.0) as i32, ((twsp/60.0)%60.0) as i32, twsp%60.0);
                println!("Particles: {}", settings!().particles);
                println!("Generations/Frame: {}", settings!().genPerFrame as f32);
                println!("Scale: {}, (xOff, yOff): ({}, {})", settings!().scale as f32, self.xOff, self.yOff);
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