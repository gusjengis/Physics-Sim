use std::fmt::DebugTuple;

use bytemuck::bytes_of;
use image::EncodableLayout;
use naga::back::spv;
use naga::ShaderStage;
use rand::Rng;
use wgpu::ColorTargetState;
use wgpu::Device;
use wgpu::PipelineLayout;
use wgpu::Queue;
use wgpu::RenderPipeline;
use wgpu::TextureFormat;
use crate::settings;
use crate::settings::Structure;
use crate::setup;
use crate::wgpu_structs::*;
use crate::wgpu_config::*;
use crate::setup::*;
use crate::state::*;
use crate::settings::*;
use crate::shader_gen::*;

extern crate flatbuffers;
use wgpu::util::DeviceExt;

const p_mult: usize = 1;//5;

pub const VERTICES: &[Vertex] = &[
    Vertex { position: [1.0, 1.0, 0.0] }, // 0 - Top Right
    Vertex { position: [1.0, -1.0, 0.0] }, // 1 - Bottom Right
    Vertex { position: [-1.0, -1.0, 0.0] }, // 2 - Bottom Left
    Vertex { position: [-1.0, 1.0, 0.0] }, // 3 - Top Left
];

pub const INDICES: &[u16] = &[
    0, 3, 2,
    0, 2, 1
];

pub struct WGPUProg {
    pub render_input: Uniform,
    pub ren_set_uniform: Uniform,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub clear_color: wgpu::Color,
    pub shader_prog: WGPUComputeProg,
    pub depth_buffer: DepthBuffer,
    pub render_tex: Texture,
    shader: wgpu::ShaderModule,
    pub shader_strs: Vec<String>,
    pub pipeline_layouts: Vec<PipelineLayout>,
    pub tex_formats: Vec<TextureFormat>,
    pub render_pipelines: Vec<wgpu::RenderPipeline>,
    pub cam: Camera,
}

impl WGPUProg {
    pub fn new(config: &mut WGPUConfig, settings: &mut Settings, dimensions: (u32, u32)) -> Self {
        let mut shader_prog = WGPUComputeProg::new(config, settings, dimensions);
        let render_tex = Texture::new_from_dimensions(config, (1, 1), 0, config.config.format);
        let clear_color = wgpu::Color {
            r: 0.0,
            g: 0.0,//0.266,
            b: 0.0,//1.0,
            a: 1.0,
        };
        let indices = &[
            0, 2, 1,
            2, 4, 1,
            4, 3, 1,
            4, 5, 3,
        ];
        let cam = Camera::new(&config);

        let depth_buffer = DepthBuffer::new(&config.device, &config.config, "depth_texture");
        let shader1_str = include_str!("shaders/rendering/2D_Particles.wgsl");
        let shader = config.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(assemble_shader(shader1_str, settings).into()),
        });
        let shader2_str = include_str!("shaders/rendering/2D_Background.wgsl");
        let shader2 = config.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(assemble_shader(shader2_str, settings).into()),
        });
        let shader3_str = include_str!("shaders/rendering/2D_Hit_Tex.wgsl");
        let shader3 = config.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(assemble_shader(shader3_str, settings).into()),
        });
        let shader4_str = include_str!("shaders/rendering/2D_Post_Processing.wgsl");
        let shader4 = config.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(assemble_shader(shader4_str, settings).into()),
        });
        let dim_contents = &[config.size.width as f32, config.size.height as f32, config.size.width as f32, config.size.height as f32, 0 as f32, 0 as f32, 1 as f32, 0 as f32];
        let dim_uniform = Uniform::new(&config.device, bytemuck::cast_slice(dim_contents), String::from("dimensions"), 0);
        let ren_set_uniform = Uniform::new(&config.device, bytemuck::cast_slice(&settings.render_settings()), String::from("settings"), 0);

        let mut pipeline_layout1 =
        config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &dim_uniform.bind_group_layout,
                &shader_prog.buffers.pos_buffers.bind_group_layout,
                &shader_prog.buffers.mov_buffers.bind_group_layout,
                &shader_prog.buffers.contact_buffers.bind_group_layout,
                &ren_set_uniform.bind_group_layout,
                &shader_prog.buffers.material_buffer.bind_group_layout,
                &shader_prog.buffers.selection_buffers.bind_group_layout,
                &shader_prog.buffers.click_buffer.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let mut pipeline_layout2 =
        config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &dim_uniform.bind_group_layout,
                &shader_prog.buffers.pos_buffers.bind_group_layout,
                &shader_prog.buffers.mov_buffers.bind_group_layout,
                &shader_prog.buffers.contact_buffers.bind_group_layout,
                &ren_set_uniform.bind_group_layout,
                &shader_prog.buffers.material_buffer.bind_group_layout,
                &shader_prog.buffers.selection_buffers.bind_group_layout,
                &shader_prog.buffers.click_buffer.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let mut pipeline_layout3 =
        config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &dim_uniform.bind_group_layout,
                &shader_prog.buffers.pos_buffers.bind_group_layout,
                &shader_prog.buffers.mov_buffers.bind_group_layout,
                &shader_prog.buffers.contact_buffers.bind_group_layout,
                &ren_set_uniform.bind_group_layout,
                &shader_prog.buffers.material_buffer.bind_group_layout,
                &shader_prog.buffers.selection_buffers.bind_group_layout,
                &shader_prog.buffers.click_buffer.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let mut pipeline_layout4 =
        config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post Processing Render Pipeline Layout"),
            bind_group_layouts: &[
                &render_tex.bind_group_layout,
                &ren_set_uniform.bind_group_layout,
                &dim_uniform.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let mut pipeline_layout5 =
        config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &dim_uniform.bind_group_layout,
                &shader_prog.buffers.pos_buffers.bind_group_layout,
                &shader_prog.buffers.mov_buffers.bind_group_layout,
                &shader_prog.buffers.contact_buffers.bind_group_layout,
                &ren_set_uniform.bind_group_layout,
                &shader_prog.buffers.material_buffer.bind_group_layout,
                &shader_prog.buffers.selection_buffers.bind_group_layout,
                &shader_prog.buffers.click_buffer.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
        
        let render_pipeline = config.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout1),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", // 1.
                buffers: &[Vertex::desc()], // 2.
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: config.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                format: DepthBuffer::DEPTH_FORMAT,
                stencil: wgpu::StencilState::default(), // 2.
                bias: wgpu::DepthBiasState::default(),
              }), // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        let vertex_buffer = config.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = config.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let render_pipeline2 = config.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout2),
            vertex: wgpu::VertexState {
                module: &shader2,
                entry_point: "vs_main", // 1.
                buffers: &[Vertex::desc()], // 2.
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader2,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: config.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                format: DepthBuffer::DEPTH_FORMAT,
                stencil: wgpu::StencilState::default(), // 2.
                bias: wgpu::DepthBiasState::default(),
              }), // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        let render_pipeline3 = config.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout3),
            vertex: wgpu::VertexState {
                module: &shader3,
                entry_point: "vs_main", // 1.
                buffers: &[Vertex::desc()], // 2.
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader3,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                format: DepthBuffer::DEPTH_FORMAT,
                stencil: wgpu::StencilState::default(), // 2.
                bias: wgpu::DepthBiasState::default(),
            }), // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        let vertex_buffer = config.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = config.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let render_pipeline4 = config.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout4),
            vertex: wgpu::VertexState {
                module: &shader4,
                entry_point: "vs_main", // 1.
                buffers: &[Vertex::desc()], // 2.
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader4,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: config.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                format: DepthBuffer::DEPTH_FORMAT,
                stencil: wgpu::StencilState::default(), // 2.
                bias: wgpu::DepthBiasState::default(),
              }), // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        let vertex_buffer = config.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = config.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let render_pipeline5 = config.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout3),
            vertex: wgpu::VertexState {
                module: &shader3,
                entry_point: "vs_main", // 1.
                buffers: &[Vertex::desc()], // 2.
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader3,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: config.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                format: DepthBuffer::DEPTH_FORMAT,
                stencil: wgpu::StencilState::default(), // 2.
                bias: wgpu::DepthBiasState::default(),
              }), // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        let vertex_buffer = config.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = config.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        
        
        Self{
            render_input: dim_uniform,
            ren_set_uniform,
            vertex_buffer,
            index_buffer,
            clear_color,
            shader_prog,
            depth_buffer,
            shader,
            render_tex,
            shader_strs: vec![
                shader1_str.to_string(),
                shader2_str.to_string(),
                shader3_str.to_string().clone(),
                shader4_str.to_string(),
                shader3_str.to_string(),
            ],
            pipeline_layouts: vec![
                pipeline_layout1,
                pipeline_layout2,
                pipeline_layout3,
                pipeline_layout4,
                pipeline_layout5,
            ],
            tex_formats: vec![
                config.config.format.clone(),
                config.config.format.clone(),
                wgpu::TextureFormat::Bgra8Unorm,
                config.config.format.clone(),
                config.config.format.clone(),
            ],
            render_pipelines: vec![
                render_pipeline,
                render_pipeline2,
                render_pipeline3,
                render_pipeline4,
                render_pipeline5
            ],
            cam,
        }
    }

    pub fn resize(&mut self, config: &mut WGPUConfig, dimensions: (u32, u32)) {
        self.shader_prog.hit_tex = Texture::new_from_dimensions(config, dimensions, 0, wgpu::TextureFormat::Bgra8Unorm);
        self.render_tex = Texture::new_from_dimensions(config, dimensions, 0, config.config.format);
    }

    pub fn rebuild_shaders(&mut self, config: &mut WGPUConfig, settings: &Settings) {
        for i in 0..self.render_pipelines.len() {
            self.rebuild_pipeline(config, settings, i);
        }
        for i in 0..2 {
            self.shader_prog.rebuild_pipeline(config, settings, i);
        }
    }

    pub fn rebuild_pipeline(&mut self, config: &mut WGPUConfig, settings: &Settings, i: usize) {
        // Compile WGSL to SPIR-V
        let shader_str = self.shader_strs[i].as_str();
        return match naga::front::wgsl::parse_str(&assemble_shader(shader_str, settings)) {
            Ok(module) => {
                return match naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
                .validate(&module)
                .map_err(|e| format!("Shader validation error: {:?}", e)) {
                    Ok(info) => {
                        match naga::back::spv::write_vec(&module, &info, &spv::Options::default(), Some(&spv::PipelineOptions { shader_stage: ShaderStage::Vertex, entry_point: format!("vs_main") })) {
                            Ok(_) => {
                                let shader = config.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                                        label: Some("Shader"),
                                            source: wgpu::ShaderSource::Wgsl(assemble_shader(shader_str, settings).into()),
                                        });
                                        
                                        self.render_pipelines[i] = config.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                                            label: Some("Render Pipeline"),
                                            layout: Some(&self.pipeline_layouts[i]),
                                        vertex: wgpu::VertexState {
                                            module: &shader,
                                            entry_point: "vs_main", // 1.
                                            buffers: &[Vertex::desc()], // 2.
                                        },
                                        fragment: Some(wgpu::FragmentState { // 3.
                                            module: &shader,
                                            entry_point: "fs_main",
                                            targets: &[Some(wgpu::ColorTargetState { // 4.
                                                format: self.tex_formats[i],
                                                blend: Some(wgpu::BlendState::REPLACE),
                                                write_mask: wgpu::ColorWrites::ALL,
                                            })],
                                        }),
                                        primitive: wgpu::PrimitiveState {
                                            topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                                            strip_index_format: None,
                                            front_face: wgpu::FrontFace::Ccw, // 2.
                                            cull_mode: Some(wgpu::Face::Back),
                                            polygon_mode: wgpu::PolygonMode::Fill,
                                            unclipped_depth: false,
                                            conservative: false,
                                        },
                                        depth_stencil: Some(wgpu::DepthStencilState {
                                            depth_write_enabled: true,
                                            depth_compare: wgpu::CompareFunction::Less,
                                            format: DepthBuffer::DEPTH_FORMAT,
                                            stencil: wgpu::StencilState::default(), // 2.
                                            bias: wgpu::DepthBiasState::default(),
                                        }), // 1.
                                        multisample: wgpu::MultisampleState {
                                            count: 1, // 2.
                                            mask: !0, // 3.
                                            alpha_to_coverage_enabled: false, // 4.
                                        },
                                        multiview: None, // 5.
                                    });},
                            Err(e) => {},
                        }
                    },
                    Err(e) => {}
                };
            }
            Err(e) => {}
        };
    }
}

pub struct BufferContainer {
    pub pos_buffers: BufferGroup,
    pub mov_buffers: BufferGroup,
    pub contact_buffers: BufferGroup,
    pub collision_settings: Uniform,
    pub click_input: Uniform,
    pub click_buffer: BufferUniform,
    pub selectangle_input: Uniform,
    pub release_input: Uniform,
    pub drag_input: Uniform,
    pub set_prop_input: Uniform,
    pub selection_buffers: BufferGroup,
    pub data_buffer: BufferUniform,
    pub material_buffer: BufferUniform,
}

impl BufferContainer {
    pub fn new(
        pos_buffers: BufferGroup,
        mov_buffers: BufferGroup,
        contact_buffers: BufferGroup,
        collision_settings: Uniform,
        click_input: Uniform,
        click_buffer: BufferUniform,
        selectangle_input: Uniform,
        release_input: Uniform,
        drag_input: Uniform,
        set_prop_input: Uniform,
        selection_buffers: BufferGroup,
        data_buffer: BufferUniform,
        material_buffer: BufferUniform,
        ) -> Self {
        
        Self {
            pos_buffers,
            mov_buffers,
            contact_buffers,
            collision_settings,
            click_input,
            click_buffer,
            selectangle_input,
            release_input,
            drag_input,
            set_prop_input,
            selection_buffers,
            data_buffer,
            material_buffer,
        }

        
    }
}

pub struct GridInfo {
    pub total_cells: usize,
    pub cell_size: f32,
    pub cell_cap: i32,
    pub w: i32,
    pub h: i32,
}

impl GridInfo {
    pub fn new(
        total_cells: usize,
        cell_size: f32,
        cell_cap: i32,
        w: i32,
        h: i32,
        ) -> Self {
        
        Self {
            total_cells,
            cell_size,
            cell_cap,
            w,
            h,
        }
    }

    pub fn as_vec(&self) -> Vec<f32> {
        return vec![
            self.cell_size,
            bytemuck::cast(self.cell_cap),
            bytemuck::cast(self.w),
            bytemuck::cast(self.h),
        ];
    }
}

pub struct WGPUComputeProg {
    pub state: State,
    pub buffers: BufferContainer,
    pub broad_phase_compute_pipeline: wgpu::ComputePipeline,
    pub click_compute_shader: wgpu::ShaderModule,
    pub click_compute_pipeline: wgpu::ComputePipeline,
    pub selectangle_compute_shader: wgpu::ShaderModule,
    pub selectangle_compute_pipeline: wgpu::ComputePipeline,
    pub release_compute_pipeline: wgpu::ComputePipeline,
    pub drag_compute_pipeline: wgpu::ComputePipeline,
    pub fix_compute_pipeline: wgpu::ComputePipeline,
    pub drop_compute_pipeline: wgpu::ComputePipeline,
    pub set_prop_compute_pipeline: wgpu::ComputePipeline,
    pub hit_tex: Texture,
    pub grid_info: GridInfo,
    pub shader_strs: Vec<String>,
    pub pipeline_layouts: Vec<PipelineLayout>,
    pub compute_pipelines: Vec<wgpu::ComputePipeline>,
}

pub fn grid_capacity(settings: &crate::settings::Settings) -> (usize, f32, i32, i32, i32) {
    let width  = settings.simulation.hor_bound  * 2.0;
    let height = settings.simulation.vert_bound * 2.0;
    let     max_rad = settings.setup.max_radius * 2.0;
    let mut min_rad = settings.setup.min_radius;
    if !settings.setup.variable_rad { min_rad = settings.setup.max_radius; }
    let w = (width/max_rad).ceil() as i32;
    let h = (height/max_rad).ceil() as i32;
    let cell_cap = ((max_rad/min_rad + 1.0).powf(2.0).ceil() as i32).min(settings.setup.particles as i32) + 2;
    let total_size = w * h * cell_cap;
    println!("Cell Capacity:   {}", cell_cap);
    println!("Cell Dimensions: {} x {}", w, h);
    println!("Total Cells:     {}", w * h);
    println!("Total Capacity:  {}", total_size);
    println!("Bytes:           {}", total_size * 4);

    return ((w * h) as usize, max_rad, cell_cap, w, h);
}

impl WGPUComputeProg {
    pub fn new(config: &mut WGPUConfig, settings: &mut Settings, dimensions: (u32, u32)) -> Self {
        // Create empty arrays for particle data_buffer

        let state = State::new(config, settings);

        let p_count = state.p_count;
        // let mut contacts = vec![bytemuck::cast::<i32, f32>(-1); 4*settings.max_contacts*p_count];
        let grid_info_return = grid_capacity(&settings);
        let mut bp_grid = vec![0; grid_info_return.0 * grid_info_return.2 as usize];
        let mut cilck_info = vec![0; 4];
        let grid_info = GridInfo::new(
            grid_info_return.0,
            grid_info_return.1,
            grid_info_return.2,
            grid_info_return.3,
            grid_info_return.4,
        );

        // Convert arrays to GPU buffers
        let pos_buffers = BufferGroup::new(&config.device, vec![
            bytemuck::cast_slice(&state.pos),
            bytemuck::cast_slice(&state.radii)
        ], "Position Buffers".to_string());
        let mut mov_buffers = BufferGroup::new(&config.device, vec![
            bytemuck::cast_slice(&state.vel),
            bytemuck::cast_slice(&state.vel),
            bytemuck::cast_slice(&state.rot),
            bytemuck::cast_slice(&state.rot_vel),
            bytemuck::cast_slice(&state.rot_vel),
            bytemuck::cast_slice(&state.acc),
            bytemuck::cast_slice(&state.fixity),
            bytemuck::cast_slice(&state.forces),
            bytemuck::cast_slice(&state.vel),
            bytemuck::cast_slice(&state.rot_vel),
        ], "Movement Buffer".to_string() );
        let mut contact_buffers = BufferGroup::new(&config.device, vec![
            bytemuck::cast_slice(&state.bonds),
            // bytemuck::cast_slice(&state.bond_info),
            bytemuck::cast_slice(&state.contacts),
            bytemuck::cast_slice(&state.contact_pointers),
            bytemuck::cast_slice(&state.material_pointers),
            bytemuck::cast_slice(&bp_grid),
            bytemuck::cast_slice(&grid_info.as_vec()),
            bytemuck::cast_slice(&vec![0 as i32; 3]) //shared mem, for controlling intermittent collision detection
            ], "Contact Buffers".to_string() );
        // let contact_buffer = BufferUniform::new(&config.device, bytemuck::cast_slice(&contacts), "Contact Buffer".to_string(), 0);
        // let bond_buffer = BufferUniform::new(&config.device, bytemuck::cast_slice(&bonds), "Bond Buffer".to_string(), 0);
        // let bond_info_buffer = BufferUniform::new(&config.device, bytemuck::cast_slice(&state.bond_info), "Bond Info Buffer".to_string(), 0);
        let material_buffer = BufferUniform::new(&config.device, bytemuck::cast_slice(&settings.materials), "Materials".to_string(), 0);
        let collision_settings = Uniform::new(&config.device, bytemuck::cast_slice(&settings.collision_settings()), "Collision Settings".to_string(), 0);
        
        let click_buffer = BufferUniform::new(&config.device, bytemuck::cast_slice(&cilck_info), "Color Buffer".to_string(), 0);
        
        let click_input = Uniform::new(&config.device, bytemuck::cast_slice(&[0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32]), "Click Data".to_string(), 0);
        let selectangle_input = Uniform::new(&config.device, bytemuck::cast_slice(&[0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32]), "Selectangle Data".to_string(), 0);
        let release_input = Uniform::new(&config.device, bytemuck::cast_slice(&[0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32]), "Release Data".to_string(), 0);
        let drag_input = Uniform::new(&config.device, bytemuck::cast_slice(&[0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32]), "Drag Data".to_string(), 0);
        let set_prop_input = Uniform::new(&config.device, bytemuck::cast_slice(&[0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32]), "Drag Data".to_string(), 0);
        let selection_buffers = BufferGroup::new(&config.device, vec![
            bytemuck::cast_slice(&state.selections),
            bytemuck::cast_slice(&state.groups),
            ], "Selection Buffers".to_string() );
        let data_buffer = BufferUniform::new(&config.device, bytemuck::cast_slice(&state.data), "Selection Buffer".to_string(), 0);
        let hit_tex = Texture::new_from_dimensions(&config, dimensions, 0, wgpu::TextureFormat::Bgra8Unorm);
        
        let buffers = BufferContainer::new(
            pos_buffers,
            mov_buffers,
            contact_buffers,
            collision_settings,
            click_input,
            click_buffer,
            selectangle_input,
            release_input,
            drag_input,
            set_prop_input,
            selection_buffers,
            data_buffer,
            material_buffer
        );
        // let col_buffer = BufferUniform::new(&config.device, bytemuck::cast_slice(&col_sec), "Collision Buffer".to_string(), 0);

        // let time_uniform = Uniform::new(&config.device, bytemuck::cast_slice(&[0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32]), "Timestamp_Uniform".to_string(), 1);
        
        //create shaders
        // println!("1");
        let lom_shader = include_str!("./shaders/physics/2D_LOM.wgsl");
        let compute_shader = config.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(assemble_shader(lom_shader, settings).into()),
        });
        // println!("2");
        let broad_phase_compute_shader = config.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(assemble_shader(include_str!("./shaders/physics/2D_Broad_Phase.wgsl"), settings).into()),
        });
        // println!("3");
        let sim_shader = include_str!("./shaders/physics/2D_Simulation.wgsl");
        let mut compute_shader2 = config.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(assemble_shader(sim_shader, settings).into()),
        });
        
        // println!("4");
        let selectangle_compute_shader = config.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(assemble_shader(include_str!("./shaders/event_handling/Selectangle.wgsl"), settings).into()),
        });
        // println!("5");
        let release_compute_shader = config.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(assemble_shader(include_str!("./shaders/event_handling/Release.wgsl"), settings).into()),
        });
        // println!("6");
        let click_compute_shader = config.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(assemble_shader(include_str!("./shaders/event_handling/Click.wgsl"), settings).into()),
        });
        // println!("7");
        let drag_compute_shader = config.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(assemble_shader(include_str!("./shaders/event_handling/Translate.wgsl"), settings).into()),
        });
        // println!("8");
        let fix_compute_shader = config.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(assemble_shader(include_str!("./shaders/event_handling/Fix.wgsl"), settings).into()),
        });
        // println!("9");
        let drop_compute_shader = config.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(assemble_shader(include_str!("./shaders/event_handling/Drop.wgsl"), settings).into()),
        });
        // println!("10");
        let set_prop_compute_shader = config.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(assemble_shader(include_str!("./shaders/event_handling/Set_Properties.wgsl"), settings).into()),
        });
        //create pipeline layout
        let compute_pipeline_layout = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("LOM compute"),
            bind_group_layouts: &[&buffers.pos_buffers.bind_group_layout, &buffers.mov_buffers.bind_group_layout, &buffers.contact_buffers.bind_group_layout, &buffers.collision_settings.bind_group_layout],
            push_constant_ranges: &[]
        });

        let broad_phase_compute_pipeline_layout = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Broad phase compute"),
            bind_group_layouts: &[&buffers.pos_buffers.bind_group_layout, &buffers.mov_buffers.bind_group_layout, &buffers.contact_buffers.bind_group_layout],
            push_constant_ranges: &[]
        });

        let compute_pipeline_layout2 = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Collision compute"),
            bind_group_layouts: &[&buffers.pos_buffers.bind_group_layout, &buffers.mov_buffers.bind_group_layout, &buffers.contact_buffers.bind_group_layout, &buffers.collision_settings.bind_group_layout, &buffers.material_buffer.bind_group_layout, &buffers.data_buffer.bind_group_layout],
            push_constant_ranges: &[]
        });
        
        let drag_compute_pipeline_layout = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Translate compute"),
            bind_group_layouts: &[&buffers.drag_input.bind_group_layout, &buffers.selection_buffers.bind_group_layout, &buffers.pos_buffers.bind_group_layout, &buffers.mov_buffers.bind_group_layout, &buffers.click_buffer.bind_group_layout],
            push_constant_ranges: &[]
        });

        let selectangle_compute_pipeline_layout = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Selectangle compute"),
            bind_group_layouts: &[&buffers.selectangle_input.bind_group_layout, &buffers.selection_buffers.bind_group_layout, &hit_tex.bind_group_layout, &buffers.click_buffer.bind_group_layout, &buffers.mov_buffers.bind_group_layout],
            push_constant_ranges: &[]
        });

        let click_compute_pipeline_layout = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Click compute"),
            bind_group_layouts: &[&buffers.click_input.bind_group_layout, &buffers.selection_buffers.bind_group_layout, &hit_tex.bind_group_layout, &buffers.click_buffer.bind_group_layout, &buffers.mov_buffers.bind_group_layout],
            push_constant_ranges: &[]
        });

        let release_compute_pipeline_layout = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Release compute"),
            bind_group_layouts: &[&buffers.release_input.bind_group_layout, &buffers.selection_buffers.bind_group_layout, &buffers.mov_buffers.bind_group_layout, &buffers.click_buffer.bind_group_layout, &buffers.collision_settings.bind_group_layout],
            push_constant_ranges: &[]
        });

        let fix_compute_pipeline_layout = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Fix compute"),
            bind_group_layouts: &[&buffers.selection_buffers.bind_group_layout, &buffers.mov_buffers.bind_group_layout, &buffers.click_buffer.bind_group_layout],
            push_constant_ranges: &[]
        });

        let drop_compute_pipeline_layout = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Drop compute"),
            bind_group_layouts: &[&buffers.selection_buffers.bind_group_layout, &buffers.mov_buffers.bind_group_layout, &buffers.click_buffer.bind_group_layout],
            push_constant_ranges: &[]
        });

        let set_prop_compute_pipeline_layout = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Collision compute"),
            bind_group_layouts: &[&buffers.pos_buffers.bind_group_layout, &buffers.mov_buffers.bind_group_layout, &buffers.contact_buffers.bind_group_layout, &buffers.selection_buffers.bind_group_layout, &buffers.set_prop_input.bind_group_layout],
            push_constant_ranges: &[]
        });

        //create pipeline
        // println!("1");
        let compute_pipeline = config.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main",
        });
        // println!("2");
        let broad_phase_compute_pipeline = config.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&broad_phase_compute_pipeline_layout),
            module: &broad_phase_compute_shader,
            entry_point: "main",
        });
        // println!("3");
        let compute_pipeline2 = config.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&compute_pipeline_layout2),
            module: &compute_shader2,
            entry_point: "main",
        });
        // println!("4");
        let drag_compute_pipeline = config.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&drag_compute_pipeline_layout),
            module: &drag_compute_shader,
            entry_point: "main",
        });
        // println!("5");
        let click_compute_pipeline = config.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&click_compute_pipeline_layout),
            module: &click_compute_shader,
            entry_point: "main",
        });
        // println!("6");
        let selectangle_compute_pipeline = config.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&selectangle_compute_pipeline_layout),
            module: &selectangle_compute_shader,
            entry_point: "main",
        });
        // println!("7");
        let release_compute_pipeline = config.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&release_compute_pipeline_layout),
            module: &release_compute_shader,
            entry_point: "main",
        });
        // println!("8");
        let fix_compute_pipeline = config.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&fix_compute_pipeline_layout),
            module: &fix_compute_shader,
            entry_point: "main",
        });
        // println!("9");
        let drop_compute_pipeline = config.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&drop_compute_pipeline_layout),
            module: &drop_compute_shader,
            entry_point: "main",
        });
        // println!("10");
        let set_prop_compute_pipeline = config.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&set_prop_compute_pipeline_layout),
            module: &set_prop_compute_shader,
            entry_point: "main",
        });

        Self {
            state,
            buffers,
            broad_phase_compute_pipeline,
            click_compute_shader,
            click_compute_pipeline,
            selectangle_compute_shader,
            selectangle_compute_pipeline,
            release_compute_pipeline,
            drag_compute_pipeline,
            fix_compute_pipeline,
            drop_compute_pipeline,
            set_prop_compute_pipeline,
            hit_tex,
            grid_info,
            shader_strs: vec![
                lom_shader.to_string(),
                sim_shader.to_string()
            ],
            pipeline_layouts: vec![
                compute_pipeline_layout,
                compute_pipeline_layout2,
            ],
            compute_pipelines: vec![
                compute_pipeline,
                compute_pipeline2,
            ]
        }
    }

    pub fn rebuild_pipeline(&mut self, config: &mut WGPUConfig, settings: &Settings, i: usize) {
        let shader_str = self.shader_strs[i].as_str();
        return match naga::front::wgsl::parse_str(&assemble_shader(shader_str, settings)) {
            Ok(module) => {
                return match naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
                .validate(&module)
                .map_err(|e| format!("Shader validation error: {:?}", e)) {
                    Ok(info) => {
                        match naga::back::spv::write_vec(&module, &info, &spv::Options::default(), Some(&spv::PipelineOptions { shader_stage: ShaderStage::Compute, entry_point: format!("main") })) {
                            Ok(_) => {
                                let shader = config.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                                    label: Some("Shader"),
                                    source: wgpu::ShaderSource::Wgsl(assemble_shader(shader_str, settings).into()),
                                });
                                        
                                self.compute_pipelines[i] = config.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                                    label: None,
                                    layout: Some(&self.pipeline_layouts[i]),
                                    module: &shader,
                                    entry_point: "main",
                                });
                            },
                            Err(e) => {},
                        }
                    },
                    Err(e) => {}
                };
            }
            Err(e) => {}
        };
    }

    pub fn update_state(&mut self, config: &mut WGPUConfig, settings: &Settings) {
        
        self.state.update_state(config, settings, &mut self.buffers);
    }

    pub fn update_selections(&mut self, device: &mut Device, queue: &mut Queue) {
        self.state.update_selections(device, queue, &mut self.buffers);
    }

    pub fn restore(&mut self, config: &mut WGPUConfig, settings: &mut Settings) {

        settings.set_particles(self.state.p_count);
        self.buffers.pos_buffers.updateBuffer(&config.device, self.state.pos.as_bytes(), 0);
        self.buffers.pos_buffers.updateBuffer(&config.device, self.state.radii.as_bytes(), 1);
        self.buffers.mov_buffers.updateBuffer(&config.device, self.state.vel.as_bytes(), 0);
        self.buffers.mov_buffers.updateBuffer(&config.device, self.state.acc.as_bytes(), 1);
        self.buffers.mov_buffers.updateBuffer(&config.device, self.state.rot.as_bytes(), 2);
        self.buffers.mov_buffers.updateBuffer(&config.device, self.state.rot_vel.as_bytes(), 3);
        self.buffers.mov_buffers.updateBuffer(&config.device, self.state.rot_acc.as_bytes(), 4);
        // self.buffers.mov_buffers.updateBuffer(&config.device, self.state.acc.as_bytes(), 5);
        self.buffers.mov_buffers.updateBuffer(&config.device, bytemuck::cast_slice(self.state.fixity.as_slice()), 6);
        self.buffers.mov_buffers.updateBuffer(&config.device, self.state.forces.as_bytes(), 7);
        self.buffers.mov_buffers.updateBuffer(&config.device, self.state.vel.as_bytes(), 8);
        self.buffers.mov_buffers.updateBuffer(&config.device, self.state.rot_vel.as_bytes(), 9);
        self.buffers.contact_buffers.updateBuffer(&config.device, bytemuck::cast_slice(self.state.bonds.as_slice()), 0);
        // self.buffers.contact_buffers.updateBuffer(&config.device, bytemuck::cast_slice(self.state.bond_info.as_slice()), 1);
        self.buffers.contact_buffers.updateBuffer(&config.device, bytemuck::cast_slice(self.state.contacts.as_slice()), 1);
        self.buffers.contact_buffers.updateBuffer(&config.device, bytemuck::cast_slice(self.state.material_pointers.as_slice()), 3);
        self.buffers.contact_buffers.updateBuffer(&config.device, bytemuck::cast_slice(self.state.grid.as_slice()), 4);
        self.buffers.contact_buffers.updateBuffer(&config.device, bytemuck::cast_slice(self.state.grid_info.as_vec().as_slice()), 5);
        self.buffers.data_buffer.updateUniform(&config.device, bytemuck::cast_slice(self.state.data.as_slice()));
        self.buffers.selection_buffers.updateBuffer(&config.device, bytemuck::cast_slice(self.state.selections.as_slice()), 0);
    }

    // fn save_state(&self , state: &State) {
    //     // let mut builder = flatbuffers::FlatBufferBuilder::new();

    //     // builder.finish(self.state, None);
    // }

    pub fn click(&mut self, config: &mut WGPUConfig, settings: &Settings) {
        let mut encoder = config.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut compute_pass_descriptor = wgpu::ComputePassDescriptor::default();

        {
            let mut compute_pass = encoder.begin_compute_pass(&compute_pass_descriptor);

            compute_pass.set_pipeline(&self.click_compute_pipeline);
            
            compute_pass.set_bind_group(0, &self.buffers.click_input.bind_group, &[]);
            compute_pass.set_bind_group(1, &self.buffers.selection_buffers.bind_group, &[]);   
            compute_pass.set_bind_group(2, &self.hit_tex.diffuse_bind_group, &[]);   
            compute_pass.set_bind_group(3, &self.buffers.click_buffer.bind_group, &[]);   
            compute_pass.set_bind_group(4, &self.buffers.mov_buffers.bind_group, &[]);  

            compute_pass.dispatch_workgroups(settings.setup.workgroups as u32, 1, 1);
            
        }

        config.queue.submit(Some(encoder.finish()));
    }

    pub fn selectangle(&mut self, config: &WGPUConfig, dimensions: (u32, u32)) {
        let mut encoder = config.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut compute_pass_descriptor = wgpu::ComputePassDescriptor::default();

        {
            let mut compute_pass = encoder.begin_compute_pass(&compute_pass_descriptor);

            compute_pass.set_pipeline(&self.selectangle_compute_pipeline);
            
            compute_pass.set_bind_group(0, &self.buffers.selectangle_input.bind_group, &[]);
            compute_pass.set_bind_group(1, &self.buffers.selection_buffers.bind_group, &[]);   
            compute_pass.set_bind_group(2, &self.hit_tex.diffuse_bind_group, &[]);   
            compute_pass.set_bind_group(3, &self.buffers.click_buffer.bind_group, &[]);   
            compute_pass.set_bind_group(4, &self.buffers.mov_buffers.bind_group, &[]);  

            compute_pass.dispatch_workgroups(((dimensions.0*dimensions.1) as f32/256.0).ceil() as u32, 1, 1);
            
        }

        config.queue.submit(Some(encoder.finish()));
    }

    pub fn release(&mut self, config: &mut WGPUConfig, settings: &Settings) {
        let mut encoder = config.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut compute_pass_descriptor = wgpu::ComputePassDescriptor::default();

        {
            let mut compute_pass = encoder.begin_compute_pass(&compute_pass_descriptor);

            compute_pass.set_pipeline(&self.release_compute_pipeline);
            
            compute_pass.set_bind_group(0, &self.buffers.release_input.bind_group, &[]);
            compute_pass.set_bind_group(1, &self.buffers.selection_buffers.bind_group, &[]);   
            compute_pass.set_bind_group(2, &self.buffers.mov_buffers.bind_group, &[]);  
            compute_pass.set_bind_group(3, &self.buffers.click_buffer.bind_group, &[]); 
            compute_pass.set_bind_group(4, &self.buffers.collision_settings.bind_group, &[]);   


            compute_pass.dispatch_workgroups(settings.setup.workgroups as u32, 1, 1);
            
        }

        config.queue.submit(Some(encoder.finish()));
    }

    pub fn drag(&mut self, config: &mut WGPUConfig, settings: &Settings) {
        let mut encoder = config.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut compute_pass_descriptor = wgpu::ComputePassDescriptor::default();

        {
            let mut compute_pass = encoder.begin_compute_pass(&compute_pass_descriptor);

            compute_pass.set_pipeline(&self.drag_compute_pipeline);
            
            compute_pass.set_bind_group(0, &self.buffers.drag_input.bind_group, &[]);
            compute_pass.set_bind_group(1, &self.buffers.selection_buffers.bind_group, &[]);   
            compute_pass.set_bind_group(2, &self.buffers.pos_buffers.bind_group, &[]);   
            compute_pass.set_bind_group(3, &self.buffers.mov_buffers.bind_group, &[]);     
            compute_pass.set_bind_group(4, &self.buffers.click_buffer.bind_group, &[]);   

            compute_pass.dispatch_workgroups(settings.setup.workgroups as u32, 1, 1);
            
        }

        config.queue.submit(Some(encoder.finish()));
    }

    pub fn fix(&mut self, config: &mut WGPUConfig, settings: &Settings) {
        let mut encoder = config.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut compute_pass_descriptor = wgpu::ComputePassDescriptor::default();

        {
            let mut compute_pass = encoder.begin_compute_pass(&compute_pass_descriptor);

            compute_pass.set_pipeline(&self.fix_compute_pipeline);
            
            compute_pass.set_bind_group(0, &self.buffers.selection_buffers.bind_group, &[]);    
            compute_pass.set_bind_group(1, &self.buffers.mov_buffers.bind_group, &[]);     
            compute_pass.set_bind_group(2, &self.buffers.click_buffer.bind_group, &[]);   

            compute_pass.dispatch_workgroups(settings.setup.workgroups as u32, 1, 1);
            
        }

        config.queue.submit(Some(encoder.finish()));
    }

    pub fn drop(&mut self, config: &mut WGPUConfig, settings: &Settings) {
        let mut encoder = config.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut compute_pass_descriptor = wgpu::ComputePassDescriptor::default();

        {
            let mut compute_pass = encoder.begin_compute_pass(&compute_pass_descriptor);

            compute_pass.set_pipeline(&self.drop_compute_pipeline);
            
            compute_pass.set_bind_group(0, &self.buffers.selection_buffers.bind_group, &[]);    
            compute_pass.set_bind_group(1, &self.buffers.mov_buffers.bind_group, &[]);     
            compute_pass.set_bind_group(2, &self.buffers.click_buffer.bind_group, &[]);   

            compute_pass.dispatch_workgroups(settings.setup.workgroups as u32, 1, 1);
            
        }

        config.queue.submit(Some(encoder.finish()));
    }

    pub fn set_properties(&mut self, config: &WGPUConfig, settings: &Settings) {
        let mut encoder = config.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        
        let mut compute_pass_descriptor = wgpu::ComputePassDescriptor::default();

        {
            let mut compute_pass = encoder.begin_compute_pass(&compute_pass_descriptor);

            compute_pass.set_pipeline(&self.set_prop_compute_pipeline);
            
            compute_pass.set_bind_group(0, &self.buffers.pos_buffers.bind_group, &[]);    
            compute_pass.set_bind_group(1, &self.buffers.mov_buffers.bind_group, &[]);     
            compute_pass.set_bind_group(2, &self.buffers.contact_buffers.bind_group, &[]);   
            compute_pass.set_bind_group(3, &self.buffers.selection_buffers.bind_group, &[]);   
            compute_pass.set_bind_group(4, &self.buffers.set_prop_input.bind_group, &[]);   

            compute_pass.dispatch_workgroups(settings.setup.workgroups as u32, 1, 1);
            
        }

        config.queue.submit(Some(encoder.finish()));
    }

    pub fn compute(&mut self, config: &mut WGPUConfig, settings: &Settings){
        

        let mut encoder = config.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut compute_pass_descriptor = wgpu::ComputePassDescriptor::default();

        for i in 0..settings.simulation.genPerFrame {
            // LAWS OF MOTION
            {
                let mut compute_pass = encoder.begin_compute_pass(&compute_pass_descriptor);
                
                compute_pass.set_pipeline(&self.compute_pipelines[0]);
                
                compute_pass.set_bind_group(0, &self.buffers.pos_buffers.bind_group, &[]);
                compute_pass.set_bind_group(1, &self.buffers.mov_buffers.bind_group, &[]);      
                compute_pass.set_bind_group(2, &self.buffers.contact_buffers.bind_group, &[]);         
                compute_pass.set_bind_group(3, &self.buffers.collision_settings.bind_group, &[]);   
                
                compute_pass.dispatch_workgroups(settings.setup.workgroups as u32, 1, 1);
                
            }
            
            // // BROAD PHASE, now handeled in LOM
            // {
            //     let mut compute_pass = encoder.begin_compute_pass(&compute_pass_descriptor);

            //     compute_pass.set_pipeline(&self.broad_phase_compute_pipeline);
                
            //     compute_pass.set_bind_group(0, &self.buffers.pos_buffers.bind_group, &[]);
            //     compute_pass.set_bind_group(1, &self.buffers.mov_buffers.bind_group, &[]);         
            //     compute_pass.set_bind_group(2, &self.buffers.contact_buffers.bind_group, &[]);         

            //     compute_pass.dispatch_workgroups(1, 1, 1);//(self.grid_info.total_cells as f32 / 256.0).ceil() as u32, 1, 1);
            // }

            // SIMULATION/COLLISIONS/BONDS

            {
                let mut compute_pass = encoder.begin_compute_pass(&compute_pass_descriptor);

                compute_pass.set_pipeline(&self.compute_pipelines[1]);
                
                compute_pass.set_bind_group(0, &self.buffers.pos_buffers.bind_group, &[]);
                compute_pass.set_bind_group(1, &self.buffers.mov_buffers.bind_group, &[]);      
                compute_pass.set_bind_group(2, &self.buffers.contact_buffers.bind_group, &[]);         
                compute_pass.set_bind_group(3, &self.buffers.collision_settings.bind_group, &[]);  
                compute_pass.set_bind_group(4, &self.buffers.material_buffer.bind_group, &[]);
                compute_pass.set_bind_group(5, &self.buffers.data_buffer.bind_group, &[]);

                compute_pass.dispatch_workgroups(settings.setup.workgroups as u32, 1, 1);

            }

            
        }

        config.queue.submit(Some(encoder.finish()));

    }
    
    fn print_particle(i: usize, pos: &[f32], vel: &[f32], radii: &[f32], color: &[f32]) {
        println!("\nParticle [\n
                        pos:   {}, {}\n
                        vel:   {}, {}\n    
                        rad:   {}\n    
                        color: {}, {}, {}\n
                    ]",
                        pos[i*2], pos[i*2+1], vel[i*2], vel[i*2+1], radii[i], 255.0*color[i*3], 255.0*color[i*3+1], 255.0*color[i*3+2]);
    }
}