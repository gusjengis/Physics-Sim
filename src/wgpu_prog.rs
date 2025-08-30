// use core::slice::SlicePattern;
use bytemuck::bytes_of;
use image::EncodableLayout;
use naga::ShaderStage;
use naga::back::spv;
use rand::Rng;
use rfd::FileDialog;
use std::fmt::DebugTuple;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use wgpu::ColorTargetState;
use wgpu::Device;
use wgpu::PipelineLayout;
use wgpu::Queue;
use wgpu::RenderPipeline;
use wgpu::SurfaceTexture;
use wgpu::TextureFormat;
use wgpu::util::DeviceExt;

use crate::particle_def::Particle_Definition;
use crate::runtime;
use crate::scripts::ScriptManager;
use crate::settings;
use crate::settings::Structure;
use crate::settings::*;
use crate::setup;
use crate::setup::*;
use crate::shader_gen::*;
use crate::state::*;
use crate::wgpu_config::*;
use crate::wgpu_structs::*;

extern crate flatbuffers;

pub const VERTICES: &[Vertex] = &[
    Vertex { position: [1.0, 1.0, 0.0] },   // 0 - Top Right
    Vertex { position: [1.0, -1.0, 0.0] },  // 1 - Bottom Right
    Vertex { position: [-1.0, -1.0, 0.0] }, // 2 - Bottom Left
    Vertex { position: [-1.0, 1.0, 0.0] },  // 3 - Top Left
];

pub const INDICES: &[u16] = &[0, 3, 2, 0, 2, 1];

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
    pub fn new(config: &mut WGPUConfig, settings: &mut Settings, dimensions: (u32, u32), script_manager: &ScriptManager) -> Self {
        let mut shader_prog = WGPUComputeProg::new(config, settings, dimensions, script_manager);
        let render_tex = Texture::new_from_dimensions(config, (1, 1), 0, config.config.format);
        let clear_color = wgpu::Color {
            r: 0.0,
            g: 0.0, //0.266,
            b: 0.0, //1.0,
            a: 1.0,
        };
        let indices = &[0, 2, 1, 2, 4, 1, 4, 3, 1, 4, 5, 3];
        let cam = Camera::new(&config);

        let depth_buffer = DepthBuffer::new(&config.device, &config.config, "depth_texture");

        let shader1_str = include_str!("shaders/rendering/2D_Particles.wgsl");
        let shader = config.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle Shader"),
            source: wgpu::ShaderSource::Wgsl(assemble_shader(shader1_str, settings).into()),
        });
        let shader2_str = include_str!("shaders/rendering/2D_Background.wgsl");
        let shader2 = config.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Background Shader"),
            source: wgpu::ShaderSource::Wgsl(assemble_shader(shader2_str, settings).into()),
        });
        let shader3_str = include_str!("shaders/rendering/2D_Hit_Tex.wgsl");
        let shader3 = config.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Hit_Tex Shader"),
            source: wgpu::ShaderSource::Wgsl(assemble_shader(shader3_str, settings).into()),
        });
        let shader4_str = include_str!("shaders/rendering/2D_Post_Processing.wgsl");
        let shader4 = config.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Post Processing Shader"),
            source: wgpu::ShaderSource::Wgsl(assemble_shader(shader4_str, settings).into()),
        });
        let shader6_str = include_str!("shaders/rendering/2D_Creation.wgsl");
        let shader6 = config.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Creation Shader"),
            source: wgpu::ShaderSource::Wgsl(assemble_shader(shader6_str, settings).into()),
        });

        let dim_contents = &[
            config.size.width as f32,
            config.size.height as f32,
            config.size.width as f32,
            config.size.height as f32,
            0 as f32,
            0 as f32,
            1 as f32,
            0 as f32,
        ];
        let dim_uniform = Uniform::new(&config.device, bytemuck::cast_slice(dim_contents), String::from("dimensions"), 0);
        let ren_set_uniform = Uniform::new(&config.device, bytemuck::cast_slice(&settings.render_settings()), String::from("settings"), 0);

        let mut pipeline_layout1 = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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

        let mut pipeline_layout2 = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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

        let mut pipeline_layout3 = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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

        let mut pipeline_layout4 = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post Processing Render Pipeline Layout"),
            bind_group_layouts: &[&render_tex.bind_group_layout, &ren_set_uniform.bind_group_layout, &dim_uniform.bind_group_layout],
            push_constant_ranges: &[],
        });

        let mut pipeline_layout5 = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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

        let mut pipeline_layout6 = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &dim_uniform.bind_group_layout,
                &shader_prog.buffers.pos_buffers.bind_group_layout,
                &shader_prog.buffers.contact_buffers.bind_group_layout,
                &ren_set_uniform.bind_group_layout,
                &shader_prog.buffers.material_buffer.bind_group_layout,
                &shader_prog.buffers.selection_buffers.bind_group_layout,
                &shader_prog.buffers.click_buffer.bind_group_layout,
                &shader_prog.buffers.click_input.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let vertex_buffer = config.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = config.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let render_pipeline = config.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Particle Render Pipeline"),
            layout: Some(&pipeline_layout1),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // 1.
                buffers: &[Vertex::desc()],   // 2.
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[],
                    zero_initialize_workgroup_memory: true,
                },
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[],
                    zero_initialize_workgroup_memory: true,
                },
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
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
            cache: None,
        });

        let render_pipeline2 = config.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Background Render Pipeline"),
            layout: Some(&pipeline_layout2),
            vertex: wgpu::VertexState {
                module: &shader2,
                entry_point: Some("vs_main"), // 1.
                buffers: &[Vertex::desc()],   // 2.
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[],
                    zero_initialize_workgroup_memory: true,
                },
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader2,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[],
                    zero_initialize_workgroup_memory: true,
                },
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
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
            cache: None,
        });

        let render_pipeline3 = config.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Hit_Tex Render Pipeline"),
            layout: Some(&pipeline_layout3),
            vertex: wgpu::VertexState {
                module: &shader3,
                entry_point: Some("vs_main"), // 1.
                buffers: &[Vertex::desc()],   // 2.
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[],
                    zero_initialize_workgroup_memory: true,
                },
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader3,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[],
                    zero_initialize_workgroup_memory: true,
                },
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
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
            cache: None,
        });

        let render_pipeline4 = config.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Post Processing Render Pipeline"),
            layout: Some(&pipeline_layout4),
            vertex: wgpu::VertexState {
                module: &shader4,
                entry_point: Some("vs_main"), // 1.
                buffers: &[Vertex::desc()],   // 2.
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[],
                    zero_initialize_workgroup_memory: true,
                },
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader4,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[],
                    zero_initialize_workgroup_memory: true,
                },
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
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
            cache: None,
        });

        let render_pipeline5 = config.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Hit_Tex Render Pipeline (2)"),
            layout: Some(&pipeline_layout3),
            vertex: wgpu::VertexState {
                module: &shader3,
                entry_point: Some("vs_main"), // 1.
                buffers: &[Vertex::desc()],   // 2.
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[],
                    zero_initialize_workgroup_memory: true,
                },
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader3,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[],
                    zero_initialize_workgroup_memory: true,
                },
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
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
            cache: None,
        });

        let render_pipeline6 = config.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Creation Render Pipeline"),
            layout: Some(&pipeline_layout6),
            vertex: wgpu::VertexState {
                module: &shader6,
                entry_point: Some("vs_main"), // 1.
                buffers: &[Vertex::desc()],   // 2.
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[],
                    zero_initialize_workgroup_memory: true,
                },
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader6,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[],
                    zero_initialize_workgroup_memory: true,
                },
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
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
            cache: None,
        });

        Self {
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
                shader6_str.to_string(),
            ],
            pipeline_layouts: vec![pipeline_layout1, pipeline_layout2, pipeline_layout3, pipeline_layout4, pipeline_layout5, pipeline_layout6],
            tex_formats: vec![
                config.config.format.clone(),
                config.config.format.clone(),
                wgpu::TextureFormat::Bgra8Unorm,
                config.config.format.clone(),
                config.config.format.clone(),
                config.config.format.clone(),
            ],
            render_pipelines: vec![render_pipeline, render_pipeline2, render_pipeline3, render_pipeline4, render_pipeline5, render_pipeline6],
            cam,
        }
    }

    pub fn resize(&mut self, config: &mut WGPUConfig, dimensions: (u32, u32)) {
        self.shader_prog.hit_tex = Texture::new_from_dimensions(config, dimensions, 0, wgpu::TextureFormat::Bgra8Unorm);
        self.render_tex = Texture::new_from_dimensions(config, dimensions, 0, config.config.format);
    }

    pub fn rebuild_shaders(&mut self, config: &mut WGPUConfig, settings: &Settings) {
        runtime!("Rebuild Shaders", {
            for i in 0..self.render_pipelines.len() {
                self.rebuild_pipeline(config, settings, i);
            }
            for i in 0..2 {
                self.shader_prog.rebuild_pipeline(config, settings, i);
            }
        });
    }

    pub fn rebuild_pipeline(&mut self, config: &mut WGPUConfig, settings: &Settings, i: usize) {
        // Compile WGSL to SPIR-V
        let shader_str = self.shader_strs[i].as_str();
        return match naga::front::wgsl::parse_str(&assemble_shader(shader_str, settings)) {
            Ok(module) => {
                return match naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
                    .validate(&module)
                    .map_err(|e| format!("Shader validation error: {:?}", e))
                {
                    Ok(info) => {
                        match naga::back::spv::write_vec(
                            &module,
                            &info,
                            &spv::Options::default(),
                            Some(&spv::PipelineOptions {
                                shader_stage: ShaderStage::Vertex,
                                entry_point: format!("vs_main"),
                            }),
                        ) {
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
                                        entry_point: Some("vs_main"), // 1.
                                        buffers: &[Vertex::desc()],   // 2.
                                        compilation_options: wgpu::PipelineCompilationOptions {
                                            constants: &[],
                                            zero_initialize_workgroup_memory: true,
                                        },
                                    },
                                    fragment: Some(wgpu::FragmentState {
                                        // 3.
                                        module: &shader,
                                        entry_point: Some("fs_main"),
                                        targets: &[Some(wgpu::ColorTargetState {
                                            // 4.
                                            format: self.tex_formats[i],
                                            blend: Some(wgpu::BlendState::REPLACE),
                                            write_mask: wgpu::ColorWrites::ALL,
                                        })],
                                        compilation_options: wgpu::PipelineCompilationOptions {
                                            constants: &[],
                                            zero_initialize_workgroup_memory: true,
                                        },
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
                                        count: 1,                         // 2.
                                        mask: !0,                         // 3.
                                        alpha_to_coverage_enabled: false, // 4.
                                    },
                                    multiview: None, // 5.
                                    cache: None,
                                });
                            }
                            Err(e) => {}
                        }
                    }
                    Err(e) => {}
                };
            }
            Err(e) => {}
        };
    }

    pub fn export_screenshot(&mut self, config: &mut WGPUConfig, path_param: Option<PathBuf>, frame: &SurfaceTexture) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let path = match path_param {
                Some(p) => p,
                None => FileDialog::new().set_directory("~").add_filter("PNG File", &["png"]).pick_file().expect("No file selected"),
            };

            let mut encoder = config.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            let output_buffer_size = (config.size.width * config.size.height * 4) as wgpu::BufferAddress;
            let output_buffer_desc = wgpu::BufferDescriptor {
                size: output_buffer_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                label: None,
                mapped_at_creation: false,
            };
            let output_buffer = config.device.create_buffer(&output_buffer_desc);

            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: &frame.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &output_buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * config.size.width),
                        rows_per_image: Some(config.size.height),
                    },
                },
                wgpu::Extent3d {
                    width: config.size.width,
                    height: config.size.height,
                    depth_or_array_layers: 1,
                },
            );

            config.queue.submit(Some(encoder.finish()));

            let buffer_slice = output_buffer.slice(..);
            let (tx, rx) = std::sync::mpsc::channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                tx.send(result).unwrap();
            });
            config.device.poll(wgpu::PollType::Wait);

            if rx.recv().unwrap().is_ok() {
                let data = buffer_slice.get_mapped_range();
                let mut rgba_data: Vec<u8> = vec![0; data.len()];
                rgba_data.copy_from_slice(&data);
                drop(data);
                output_buffer.unmap();

                // Swap R and B channels
                for pixel in rgba_data.chunks_mut(4) {
                    pixel.swap(0, 2);
                }

                let image = image::RgbaImage::from_raw(config.size.width, config.size.height, rgba_data).unwrap();
                image.save(path).expect("Failed to save image");
            }
        }
    }
}

pub struct BufferContainer {
    pub pos_buffers: BufferGroup,
    pub mov_buffers: BufferGroup,
    pub contact_buffers: BufferGroup,
    pub collision_settings: Uniform,
    pub create_input: Uniform,
    pub click_input: Uniform,
    pub click_buffer: BufferUniform,
    pub selectangle_input: Uniform,
    pub release_input: Uniform,
    pub drag_input: Uniform,
    pub set_prop_input: Uniform,
    // pub set_group_input: Uniform,
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
        create_input: Uniform,
        click_input: Uniform,
        click_buffer: BufferUniform,
        selectangle_input: Uniform,
        release_input: Uniform,
        drag_input: Uniform,
        set_prop_input: Uniform,
        // set_group_input: Uniform,
        selection_buffers: BufferGroup,
        data_buffer: BufferUniform,
        material_buffer: BufferUniform,
    ) -> Self {
        Self {
            pos_buffers,
            mov_buffers,
            contact_buffers,
            collision_settings,
            create_input,
            click_input,
            click_buffer,
            selectangle_input,
            release_input,
            drag_input,
            set_prop_input,
            // set_group_input,
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
    pub fn new(total_cells: usize, cell_size: f32, cell_cap: i32, w: i32, h: i32) -> Self {
        Self {
            total_cells,
            cell_size,
            cell_cap,
            w,
            h,
        }
    }

    pub fn as_vec(&self) -> Vec<f32> {
        return vec![self.cell_size, bytemuck::cast(self.cell_cap), bytemuck::cast(self.w), bytemuck::cast(self.h)];
    }
}

pub struct WGPUComputeProg {
    pub state: State,
    pub buffers: BufferContainer,
    pub click_compute_shader: wgpu::ShaderModule,
    pub click_compute_pipeline: wgpu::ComputePipeline,
    pub selectangle_compute_shader: wgpu::ShaderModule,
    pub selectangle_compute_pipeline: wgpu::ComputePipeline,
    pub release_compute_pipeline: wgpu::ComputePipeline,
    pub drag_compute_pipeline: wgpu::ComputePipeline,
    pub fix_compute_pipeline: wgpu::ComputePipeline,
    pub drop_compute_pipeline: wgpu::ComputePipeline,
    pub set_prop_compute_pipeline: wgpu::ComputePipeline,
    // pub set_group_compute_pipeline: wgpu::ComputePipeline,
    pub hit_tex: Texture,
    pub shader_strs: Vec<String>,
    pub pipeline_layouts: Vec<PipelineLayout>,
    pub compute_pipelines: Vec<wgpu::ComputePipeline>,
}

impl WGPUComputeProg {
    pub fn new(config: &mut WGPUConfig, settings: &mut Settings, dimensions: (u32, u32), script_manager: &ScriptManager) -> Self {
        // Create empty arrays for particle data_buffer

        let state = State::new(config, settings, script_manager);

        let p_count = state.p_count;
        // let mut contacts = vec![bytemuck::cast::<i32, f32>(-1); 4*settings.max_contacts*p_count];

        // Convert arrays to GPU buffers
        let pos_buffers = BufferGroup::new(
            &config.device,
            vec![bytemuck::cast_slice(&state.pos), bytemuck::cast_slice(&state.radii)],
            "Position Buffers".to_string(),
        );
        let mut mov_buffers = BufferGroup::new(
            &config.device,
            vec![
                bytemuck::cast_slice(&state.vel),
                bytemuck::cast_slice(&state.vel),
                bytemuck::cast_slice(&state.rot),
                bytemuck::cast_slice(&state.rot_vel),
                bytemuck::cast_slice(&state.rot_vel),
                bytemuck::cast_slice(&state.acc),
                bytemuck::cast_slice(&state.fixity),
                bytemuck::cast_slice(&state.forces),
                bytemuck::cast_slice(&state.del_pos),
                bytemuck::cast_slice(&state.del_rot),
            ],
            "Movement Buffer".to_string(),
        );
        let mut contact_buffers = BufferGroup::new(
            &config.device,
            vec![
                bytemuck::cast_slice(&state.bonds),
                // bytemuck::cast_slice(&state.bond_info),
                bytemuck::cast_slice(&state.contacts),
                bytemuck::cast_slice(&state.contact_pointers),
                bytemuck::cast_slice(&state.material_pointers),
                bytemuck::cast_slice(&state.grid),
                bytemuck::cast_slice(&state.grid_info.as_vec()),
                bytemuck::cast_slice(&vec![0 as i32; 4]), //shared mem, for controlling intermittent collision detection
            ],
            "Contact Buffers".to_string(),
        );
        // let contact_buffer = BufferUniform::new(&config.device, bytemuck::cast_slice(&contacts), "Contact Buffer".to_string(), 0);
        // let bond_buffer = BufferUniform::new(&config.device, bytemuck::cast_slice(&bonds), "Bond Buffer".to_string(), 0);
        // let bond_info_buffer = BufferUniform::new(&config.device, bytemuck::cast_slice(&state.bond_info), "Bond Info Buffer".to_string(), 0);
        let material_buffer = BufferUniform::new(&config.device, bytemuck::cast_slice(&settings.materials), "Materials".to_string(), 0);
        let collision_settings = Uniform::new(&config.device, bytemuck::cast_slice(&settings.collision_settings()), "Collision Settings".to_string(), 0);

        let mut cilck_info = vec![0; 4];
        let click_buffer = BufferUniform::new(&config.device, bytemuck::cast_slice(&cilck_info), "Color Buffer".to_string(), 0);

        let create_input = Uniform::new(
            &config.device,
            bytemuck::cast_slice(&[0.0 as f32, 0.0 as f32, bytemuck::cast(state.p_count as i32)]),
            "Click Data".to_string(),
            0,
        );
        let click_input = Uniform::new(&config.device, bytemuck::cast_slice(&[0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32]), "Click Data".to_string(), 0);
        let selectangle_input = Uniform::new(
            &config.device,
            bytemuck::cast_slice(&[0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32]),
            "Selectangle Data".to_string(),
            0,
        );
        let release_input = Uniform::new(
            &config.device,
            bytemuck::cast_slice(&[0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32]),
            "Release Data".to_string(),
            0,
        );
        let drag_input = Uniform::new(
            &config.device,
            bytemuck::cast_slice(&[0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32]),
            "Drag Data".to_string(),
            0,
        );
        let set_prop_input = Uniform::new(
            &config.device,
            bytemuck::cast_slice(&[
                0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32,
                0.0 as f32,
            ]),
            "Drag Data".to_string(),
            0,
        );
        let selection_buffers = BufferGroup::new(
            &config.device,
            vec![bytemuck::cast_slice(&state.selections), bytemuck::cast_slice(&state.groups)],
            "Selection Buffers".to_string(),
        );
        let data_buffer = BufferUniform::new(&config.device, bytemuck::cast_slice(&state.data), "Selection Buffer".to_string(), 0);
        let hit_tex = Texture::new_from_dimensions(&config, dimensions, 0, wgpu::TextureFormat::Bgra8Unorm);

        let buffers = BufferContainer::new(
            pos_buffers,
            mov_buffers,
            contact_buffers,
            collision_settings,
            create_input,
            click_input,
            click_buffer,
            selectangle_input,
            release_input,
            drag_input,
            set_prop_input,
            // set_group_input,
            selection_buffers,
            data_buffer,
            material_buffer,
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
        // println!("11");
        let set_group_compute_shader = config.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(assemble_shader(include_str!("./shaders/event_handling/Set_Group.wgsl"), settings).into()),
        });
        ////create pipeline layout
        let compute_pipeline_layout = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("LOM compute"),
            bind_group_layouts: &[
                &buffers.pos_buffers.bind_group_layout,
                &buffers.mov_buffers.bind_group_layout,
                &buffers.contact_buffers.bind_group_layout,
                &buffers.collision_settings.bind_group_layout,
                &buffers.data_buffer.bind_group_layout,
                &buffers.material_buffer.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let compute_pipeline_layout2 = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Collision compute"),
            bind_group_layouts: &[
                &buffers.pos_buffers.bind_group_layout,
                &buffers.mov_buffers.bind_group_layout,
                &buffers.contact_buffers.bind_group_layout,
                &buffers.collision_settings.bind_group_layout,
                &buffers.material_buffer.bind_group_layout,
                &buffers.data_buffer.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let drag_compute_pipeline_layout = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Translate compute"),
            bind_group_layouts: &[
                &buffers.drag_input.bind_group_layout,
                &buffers.selection_buffers.bind_group_layout,
                &buffers.pos_buffers.bind_group_layout,
                &buffers.mov_buffers.bind_group_layout,
                &buffers.click_buffer.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let selectangle_compute_pipeline_layout = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Selectangle compute"),
            bind_group_layouts: &[
                &buffers.selectangle_input.bind_group_layout,
                &buffers.selection_buffers.bind_group_layout,
                &hit_tex.bind_group_layout,
                &buffers.click_buffer.bind_group_layout,
                &buffers.mov_buffers.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let click_compute_pipeline_layout = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Click compute"),
            bind_group_layouts: &[
                &buffers.click_input.bind_group_layout,
                &buffers.selection_buffers.bind_group_layout,
                &hit_tex.bind_group_layout,
                &buffers.click_buffer.bind_group_layout,
                &buffers.mov_buffers.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let release_compute_pipeline_layout = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Release compute"),
            bind_group_layouts: &[
                &buffers.release_input.bind_group_layout,
                &buffers.selection_buffers.bind_group_layout,
                &buffers.mov_buffers.bind_group_layout,
                &buffers.click_buffer.bind_group_layout,
                &buffers.collision_settings.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let fix_compute_pipeline_layout = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Fix compute"),
            bind_group_layouts: &[
                &buffers.selection_buffers.bind_group_layout,
                &buffers.mov_buffers.bind_group_layout,
                &buffers.click_buffer.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let drop_compute_pipeline_layout = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Drop compute"),
            bind_group_layouts: &[
                &buffers.selection_buffers.bind_group_layout,
                &buffers.mov_buffers.bind_group_layout,
                &buffers.click_buffer.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let set_prop_compute_pipeline_layout = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Collision compute"),
            bind_group_layouts: &[
                &buffers.pos_buffers.bind_group_layout,
                &buffers.mov_buffers.bind_group_layout,
                &buffers.contact_buffers.bind_group_layout,
                &buffers.selection_buffers.bind_group_layout,
                &buffers.set_prop_input.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        // let set_group_compute_pipeline_layout = config.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //     label: Some("Collision compute"),
        //     bind_group_layouts: &[
        //         &buffers.pos_buffers.bind_group_layout,
        //         &buffers.mov_buffers.bind_group_layout,
        //         &buffers.contact_buffers.bind_group_layout,
        //         &buffers.selection_buffers.bind_group_layout,
        //         // &buffers.set_group_input.bind_group_layout,
        //     ],
        //     push_constant_ranges: &[],
        // });

        //create pipeline
        // println!("1");
        let compute_pipeline = config.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: Some("main"),
            // TODO: Figure out what this stuff means
            cache: None,
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &[],
                zero_initialize_workgroup_memory: true,
            },
        });
        // println!("2");
        let compute_pipeline2 = config.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&compute_pipeline_layout2),
            module: &compute_shader2,
            entry_point: Some("main"),
            cache: None,
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &[],
                zero_initialize_workgroup_memory: true,
            },
        });
        // println!("4");
        let drag_compute_pipeline = config.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&drag_compute_pipeline_layout),
            module: &drag_compute_shader,
            entry_point: Some("main"),
            cache: None,
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &[],
                zero_initialize_workgroup_memory: true,
            },
        });
        // println!("5");
        let click_compute_pipeline = config.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&click_compute_pipeline_layout),
            module: &click_compute_shader,
            entry_point: Some("main"),
            cache: None,
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &[],
                zero_initialize_workgroup_memory: true,
            },
        });
        // println!("6");
        let selectangle_compute_pipeline = config.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&selectangle_compute_pipeline_layout),
            module: &selectangle_compute_shader,
            entry_point: Some("main"),
            cache: None,
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &[],
                zero_initialize_workgroup_memory: true,
            },
        });
        // println!("7");
        let release_compute_pipeline = config.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&release_compute_pipeline_layout),
            module: &release_compute_shader,
            entry_point: Some("main"),
            cache: None,
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &[],
                zero_initialize_workgroup_memory: true,
            },
        });
        // println!("8");
        let fix_compute_pipeline = config.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&fix_compute_pipeline_layout),
            module: &fix_compute_shader,
            entry_point: Some("main"),
            cache: None,
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &[],
                zero_initialize_workgroup_memory: true,
            },
        });
        // println!("9");
        let drop_compute_pipeline = config.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&drop_compute_pipeline_layout),
            module: &drop_compute_shader,
            entry_point: Some("main"),
            cache: None,
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &[],
                zero_initialize_workgroup_memory: true,
            },
        });
        // println!("10");
        let set_prop_compute_pipeline = config.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&set_prop_compute_pipeline_layout),
            module: &set_prop_compute_shader,
            entry_point: Some("main"),
            cache: None,
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &[],
                zero_initialize_workgroup_memory: true,
            },
        });
        Self {
            state,
            buffers,
            click_compute_shader,
            click_compute_pipeline,
            selectangle_compute_shader,
            selectangle_compute_pipeline,
            release_compute_pipeline,
            drag_compute_pipeline,
            fix_compute_pipeline,
            drop_compute_pipeline,
            set_prop_compute_pipeline,
            // set_group_compute_pipeline,
            hit_tex,
            shader_strs: vec![lom_shader.to_string(), sim_shader.to_string()],
            pipeline_layouts: vec![compute_pipeline_layout, compute_pipeline_layout2],
            compute_pipelines: vec![compute_pipeline, compute_pipeline2],
        }
    }

    pub fn reset(&mut self, config: &mut WGPUConfig, settings: &mut Settings, dimensions: (u32, u32), script_manager: &ScriptManager) {
        self.state = State::new(config, settings, script_manager);
        self.restore(config, settings);
    }

    pub fn rebuild_pipeline(&mut self, config: &mut WGPUConfig, settings: &Settings, i: usize) {
        let shader_str = self.shader_strs[i].as_str();
        return match naga::front::wgsl::parse_str(&assemble_shader(shader_str, settings)) {
            Ok(module) => {
                return match naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
                    .validate(&module)
                    .map_err(|e| format!("Shader validation error: {:?}", e))
                {
                    Ok(info) => match naga::back::spv::write_vec(
                        &module,
                        &info,
                        &spv::Options::default(),
                        Some(&spv::PipelineOptions {
                            shader_stage: ShaderStage::Compute,
                            entry_point: format!("main"),
                        }),
                    ) {
                        Ok(_) => {
                            let shader = config.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                                label: Some("Shader"),
                                source: wgpu::ShaderSource::Wgsl(assemble_shader(shader_str, settings).into()),
                            });

                            self.compute_pipelines[i] = config.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                                label: None,
                                layout: Some(&self.pipeline_layouts[i]),
                                module: &shader,
                                entry_point: Some("main"),
                                cache: None,
                                compilation_options: wgpu::PipelineCompilationOptions {
                                    constants: &[],
                                    zero_initialize_workgroup_memory: true,
                                },
                            });
                        }
                        Err(e) => {}
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

    pub fn update_preview(&mut self, config: &mut WGPUConfig, settings: &mut Settings, reallocating: bool) {
        self.state.store_particle(self.state.radii.len() - 1, 0.0, 0.0, settings.create.current_particle, settings);
        self.store_particle(self.state.radii.len() - 1, 0.0, 0.0, config, settings);
    }

    pub fn spawn_particle(&mut self, x: f32, y: f32, config: &mut WGPUConfig, settings: &mut Settings) {
        let p_def = settings.create.current_particle;
        let reallocating = self.state.rot.len() - settings.create.quantity as usize <= self.state.p_count;
        if reallocating {
            self.realloc(config, settings);
            self.state.spawn_particle(x, y, p_def, settings);
            self.resize_buffers(config, settings);
        } else {
            self.state.spawn_particle(x, y, p_def, settings);
            self.store_particle(self.state.p_count - 1, x, y, config, settings);
        }
        self.update_preview(config, settings, reallocating);
    }

    pub fn store_particle(&mut self, index: usize, x: f32, y: f32, config: &mut WGPUConfig, settings: &mut Settings) {
        self.buffers.pos_buffers.write_data(config, 1 * 4 * index, self.state.radii[index..index + 1].as_bytes(), 1); //radius
        self.buffers.pos_buffers.write_data(config, 2 * 4 * index, self.state.pos[index * 2..index * 2 + 2].as_bytes(), 0); //x,y
        self.buffers.mov_buffers.write_data(config, 1 * 4 * index, self.state.rot[index..index + 1].as_bytes(), 2); //rot
        self.buffers.mov_buffers.write_data(config, 2 * 4 * index, self.state.vel[2 * index..2 * index + 2].as_bytes(), 0); //vel: x,y
        self.buffers.mov_buffers.write_data(config, 1 * 4 * index, self.state.rot_vel[index..index + 1].as_bytes(), 3); //vel: rot
        self.buffers
            .mov_buffers
            .write_data(config, 6 * 4 * index, bytemuck::cast_slice::<i32, f32>(&self.state.fixity[6 * index..6 * index + 6]).as_bytes(), 6); //fixity
        self.buffers.mov_buffers.write_data(config, 6 * 4 * index, self.state.forces[6 * index..6 * index + 6].as_bytes(), 7); //forces
        self.buffers
            .contact_buffers
            .write_data(config, 1 * 4 * index, bytemuck::cast_slice::<i32, f32>(&self.state.material_pointers[index..index + 1]).as_bytes(), 3);
        //material
    }

    pub fn realloc(&mut self, config: &mut WGPUConfig, settings: &mut Settings) {
        self.update_state(config, settings);
        self.state.realloc(settings);
    }

    pub fn resize_buffers(&mut self, config: &mut WGPUConfig, settings: &mut Settings) {
        self.buffers.pos_buffers.updateBuffer(&config.device, self.state.pos.as_bytes(), 0);
        self.buffers.pos_buffers.updateBuffer(&config.device, self.state.radii.as_bytes(), 1);
        self.buffers.mov_buffers.updateBuffer(&config.device, self.state.vel.as_bytes(), 0);
        self.buffers.mov_buffers.updateBuffer(&config.device, self.state.acc.as_bytes(), 1);
        self.buffers.mov_buffers.updateBuffer(&config.device, self.state.rot.as_bytes(), 2);
        self.buffers.mov_buffers.updateBuffer(&config.device, self.state.rot_vel.as_bytes(), 3);
        self.buffers.mov_buffers.updateBuffer(&config.device, self.state.rot_acc.as_bytes(), 4);
        self.buffers.mov_buffers.updateBuffer(&config.device, bytemuck::cast_slice(self.state.fixity.as_slice()), 6);
        self.buffers.mov_buffers.updateBuffer(&config.device, self.state.forces.as_bytes(), 7);
        self.buffers.mov_buffers.updateBuffer(&config.device, self.state.del_pos.as_bytes(), 8);
        self.buffers.mov_buffers.updateBuffer(&config.device, self.state.del_rot.as_bytes(), 9);
        self.buffers.contact_buffers.updateBuffer(&config.device, bytemuck::cast_slice(self.state.contacts.as_slice()), 1);
        self.buffers
            .contact_buffers
            .updateBuffer(&config.device, bytemuck::cast_slice(self.state.material_pointers.as_slice()), 3);
        self.buffers.data_buffer.updateUniform(&config.device, bytemuck::cast_slice(self.state.data.as_slice()));
        self.buffers.selection_buffers.updateBuffer(&config.device, bytemuck::cast_slice(self.state.selections.as_slice()), 0);
        self.buffers.selection_buffers.updateBuffer(&config.device, bytemuck::cast_slice(self.state.groups.as_slice()), 1);
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
        self.buffers.mov_buffers.updateBuffer(&config.device, self.state.acc.as_bytes(), 5);
        self.buffers.mov_buffers.updateBuffer(&config.device, bytemuck::cast_slice(self.state.fixity.as_slice()), 6);
        self.buffers.mov_buffers.updateBuffer(&config.device, self.state.forces.as_bytes(), 7);
        self.buffers.mov_buffers.updateBuffer(&config.device, self.state.vel.as_bytes(), 8);
        self.buffers.mov_buffers.updateBuffer(&config.device, self.state.rot_vel.as_bytes(), 9);
        self.buffers.contact_buffers.updateBuffer(&config.device, bytemuck::cast_slice(self.state.bonds.as_slice()), 0);
        // self.buffers.contact_buffers.updateBuffer(&config.device, bytemuck::cast_slice(self.state.bond_info.as_slice()), 1);
        self.buffers.contact_buffers.updateBuffer(&config.device, bytemuck::cast_slice(self.state.contacts.as_slice()), 1);
        self.buffers
            .contact_buffers
            .updateBuffer(&config.device, bytemuck::cast_slice(self.state.material_pointers.as_slice()), 3);
        self.buffers.contact_buffers.updateBuffer(&config.device, bytemuck::cast_slice(self.state.grid.as_slice()), 4);
        self.buffers
            .contact_buffers
            .updateBuffer(&config.device, bytemuck::cast_slice(self.state.grid_info.as_vec().as_slice()), 5);
        self.buffers.data_buffer.updateUniform(&config.device, bytemuck::cast_slice(self.state.data.as_slice()));
        self.buffers.selection_buffers.updateBuffer(&config.device, bytemuck::cast_slice(self.state.selections.as_slice()), 0);
        self.buffers.selection_buffers.updateBuffer(&config.device, bytemuck::cast_slice(self.state.groups.as_slice()), 1);
        self.state.up_to_date = true;
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

            compute_pass.dispatch_workgroups(((dimensions.0 * dimensions.1) as f32 / 256.0).ceil() as u32, 1, 1);
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

    pub fn compute(&mut self, config: &mut WGPUConfig, settings: &Settings, ticks: usize) {
        let mut encoder = config.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut compute_pass_descriptor = wgpu::ComputePassDescriptor::default();

        for i in 0..ticks {
            // LAWS OF MOTION
            {
                let mut compute_pass = encoder.begin_compute_pass(&compute_pass_descriptor);

                compute_pass.set_pipeline(&self.compute_pipelines[0]);

                compute_pass.set_bind_group(0, &self.buffers.pos_buffers.bind_group, &[]);
                compute_pass.set_bind_group(1, &self.buffers.mov_buffers.bind_group, &[]);
                compute_pass.set_bind_group(2, &self.buffers.contact_buffers.bind_group, &[]);
                compute_pass.set_bind_group(3, &self.buffers.collision_settings.bind_group, &[]);
                compute_pass.set_bind_group(4, &self.buffers.data_buffer.bind_group, &[]);
                compute_pass.set_bind_group(5, &self.buffers.material_buffer.bind_group, &[]);

                compute_pass.dispatch_workgroups(settings.setup.workgroups as u32, 1, 1);
            }

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
        self.state.up_to_date = false;
    }

    fn print_particle(i: usize, pos: &[f32], vel: &[f32], radii: &[f32], color: &[f32]) {
        println!(
            "\nParticle [\n
                        pos:   {}, {}\n
                        vel:   {}, {}\n    
                        rad:   {}\n    
                        color: {}, {}, {}\n
                    ]",
            pos[i * 2],
            pos[i * 2 + 1],
            vel[i * 2],
            vel[i * 2 + 1],
            radii[i],
            255.0 * color[i * 3],
            255.0 * color[i * 3 + 1],
            255.0 * color[i * 3 + 2]
        );
    }
}
