use crate::wgpu_config::WGPUConfig;
use cgmath::{InnerSpace, Transform};
use wgpu::{util::DeviceExt, BindGroupLayout, BindGroupLayoutEntry, BindGroupEntry, Buffer};



#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ]
        }
    }
}
// unsafe impl bytemuck::Pod for Vertex {}
// unsafe impl bytemuck::Zeroable for Vertex {}



// struct Triangle{
//     vertices: [&Vertex; 3]
// }

// impl Triangle{
//     pub fn new() -> Self {

//     }
// }
// struct Model {
//     Vertices: Box<[Vertex]>,
//     Indices: [u16]
// }

// impl Model {
//     pub fn new(vertices: &[Vertex], indices: [u16] ){
//         let vertexBox = Box::new(vertices);
//     }
// }

pub struct Uniform{
    label: String,
    buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    binding: u32
}

impl Uniform {
    pub fn new(device: &wgpu::Device, contents: &[u8], label: String, binding: u32) -> Self {
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&label),
                contents: contents,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: binding,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some(&label),
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: binding,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: Some(&label),
        });

        Self { 
            label: label,
            buffer: buffer, 
            bind_group_layout: bind_group_layout, 
            bind_group: bind_group,
            binding: binding
        }
    }
    pub fn updateUniform(&mut self, device: &wgpu::Device, contents: &[u8]){
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&self.label),
                contents: contents,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: self.binding,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: Some(&self.label),
        });
    
        self.buffer = buffer;
        self.bind_group = bind_group;
    }
}

pub struct Texture{
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub diffuse_bind_group: wgpu::BindGroup,
    pub dimensions: (u32, u32),
    binding: u32
}

impl Texture {
    pub fn new(config: &WGPUConfig, bytes: &[u8], binding: u32) -> Self {
        let diffuse_image = image::load_from_memory(bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();

        use image::GenericImageView;
        let dimensions = diffuse_image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let diffuse_texture = config.device.create_texture(
            &wgpu::TextureDescriptor {
                // All textures are stored as 3D, we represent our 2D texture
                // by setting depth to 1.
                size: texture_size,
                mip_level_count: 1, // We'll talk about this a little later
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // Most images are stored using sRGB so we need to reflect that here.
                format: wgpu::TextureFormat::Rgba8Unorm,
                // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                // COPY_DST means that we want to copy data to this texture
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("diffuse_texture"),
                // This is the same as with the SurfaceConfig. It
                // specifies what texture formats can be used to
                // create TextureViews for this texture. The base
                // texture format (Rgba8UnormSrgb in this case) is
                // always supported. Note that using a different
                // texture format is not supported on the WebGL2
                // backend.
                view_formats: &[],
            }
        );

        config.queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &diffuse_rgba,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some((4 * dimensions.0) as u32),
                rows_per_image: Some((dimensions.1) as u32),
            },
            texture_size,
        );

        let diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_sampler = config.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            config.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: binding,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: binding+1,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = config.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: binding,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: binding+1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );

        Self { 
            texture: diffuse_texture, 
            view: diffuse_texture_view,
            sampler: diffuse_sampler,
            bind_group_layout: texture_bind_group_layout, 
            diffuse_bind_group: diffuse_bind_group,
            binding: binding,
            dimensions
        }
    }

    pub fn new_from_dimensions(config: &WGPUConfig, dimensions: (u32, u32), binding: u32, format: wgpu::TextureFormat) -> Self {

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let diffuse_texture = config.device.create_texture(
            &wgpu::TextureDescriptor {
                // All textures are stored as 3D, we represent our 2D texture
                // by setting depth to 1.
                size: texture_size,
                mip_level_count: 1, // We'll talk about this a little later
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // Most images are stored using sRGB so we need to reflect that here.
                format: format,
                // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                // COPY_DST means that we want to copy data to this texture
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
                label: Some("diffuse_texture"),
                // This is the same as with the SurfaceConfig. It
                // specifies what texture formats can be used to
                // create TextureViews for this texture. The base
                // texture format (Rgba8UnormSrgb in this case) is
                // always supported. Note that using a different
                // texture format is not supported on the WebGL2
                // backend.
                view_formats: &[],
            }
        );

        let diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_sampler = config.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            config.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: binding,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: binding+1,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = config.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: binding,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: binding+1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );

        Self { 
            texture: diffuse_texture, 
            view: diffuse_texture_view,
            sampler: diffuse_sampler,
            bind_group_layout: texture_bind_group_layout, 
            diffuse_bind_group: diffuse_bind_group,
            binding: binding,
            dimensions
        }
    }

    pub fn setBinding(&mut self, config: &WGPUConfig, binding: u32, storage: bool){
        if(!storage){
            let texture_bind_group_layout =
            config.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: binding,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: binding+1,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

            let bind_group = config.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    layout: &texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: binding,
                            resource: wgpu::BindingResource::TextureView(&self.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: binding+1,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        }
                    ],
                    label: Some("diffuse_bind_group"),
                }
            );
            self.bind_group_layout = texture_bind_group_layout;
            self.diffuse_bind_group = bind_group;
        } else {
            let texture_bind_group_layout =
            config.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("storage_texture_bind_group_layout"),
                entries: &[
        
                    wgpu::BindGroupLayoutEntry {
                        binding: binding,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            format: wgpu::TextureFormat::Rgba8Unorm, 
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            view_dimension: wgpu::TextureViewDimension::D2
                        },
                        count: None,
                    },
                ],
            });

            let bind_group = config.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    layout: &texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: binding,
                            resource: wgpu::BindingResource::TextureView(&self.view),
                        },
                    ],
                    label: Some("diffuse_bind_group"),
                }
            );
            self.bind_group_layout = texture_bind_group_layout;
            self.diffuse_bind_group = bind_group;
        }
        
    }
}



pub struct BufferUniform{
    label: String,
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    binding: u32
}

impl BufferUniform {
    pub fn new(device: &wgpu::Device, contents: &[u8], label: String, binding: u32) -> Self {
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&label),
                contents: contents,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::UNIFORM |wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            }
        );
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: binding,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage {read_only: false},
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some(&label),
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: binding,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: Some(&label),
        });

        Self { 
            label: label,
            buffer: buffer, 
            bind_group_layout: bind_group_layout, 
            bind_group: bind_group,
            binding: binding
        }
    }

    pub fn updateUniform(&mut self, device: &wgpu::Device, contents: &[u8]){
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&self.label),
                contents: contents,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::UNIFORM |wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            }
        );
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: self.binding,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: Some(&self.label),
        });
    
        self.buffer = buffer;
        self.bind_group = bind_group;
    }

    pub fn setBinding(&mut self, config: &WGPUConfig, binding: u32, storage: bool){
        let mut visibility = wgpu::ShaderStages::COMPUTE;
        let mut ty = wgpu::BufferBindingType::Storage {read_only: false};
        if(!storage){
            visibility = wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE;;
            ty = wgpu::BufferBindingType::Uniform
        }
        let bind_group_layout = config.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: binding,
                    visibility: visibility,
                    ty: wgpu::BindingType::Buffer {
                        ty: ty,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some(&self.label),
        });
        let bind_group = config.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: binding,
                    resource: self.buffer.as_entire_binding(),
                }
            ],
            label: Some(&self.label),
        });
        self.bind_group_layout = bind_group_layout;
        self.bind_group = bind_group;
        self.binding = binding;
    }
        
}

pub struct BufferGroup{
    label: String,
    layout_entries: Vec<BindGroupLayoutEntry>,
    // entries: Vec<BindGroupEntry>,
    pub buffers: Vec<Buffer>,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl BufferGroup {
    pub fn new(device: &wgpu::Device, contents: Vec<&[u8]>, label: String,) -> Self {
        let mut buffers = vec![];
        for i in 0..contents.len() {
            buffers.push(device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some(&label),
                    contents: contents[i],
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
                }
            ));
        }
        let mut layout_entries = vec![];
        let mut entries = vec![];
        for i in 0..contents.len() {
            layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage {read_only: false},
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
            entries.push(wgpu::BindGroupEntry {
                binding: i as u32,
                resource: buffers[i].as_entire_binding(),
            });
        }
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &layout_entries,
            label: Some(&label),
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &entries,
            label: Some(&label),
        });

        Self {
            label,
            layout_entries,
            buffers,
            bind_group_layout, 
            bind_group,
        }
    }

    pub fn updateBuffer(&mut self, device: &wgpu::Device, contents: &[u8], index: usize){
        self.buffers[index] = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&self.label),
                contents: contents,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::UNIFORM |wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            }
        );
        // println!("{}: {}", self.label, self.buffers[index].size());
        let mut layout_entries = vec![];
        let mut entries = vec![];
        for i in 0..self.buffers.len() {
            layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage {read_only: false},
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
            entries.push(wgpu::BindGroupEntry {
                binding: i as u32,
                resource: self.buffers[i].as_entire_binding(),
            });
        }
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &layout_entries,
            label: Some(&self.label),
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &entries,
            label: Some(&self.label),
        });
        self.bind_group_layout = bind_group_layout;
        self.bind_group = bind_group;
    }

    // pub fn setReadOnly(&mut self, bufferID: usize, readonly: bool){
    //     self.layout_entries[bufferID] = wgpu::BindGroupLayoutEntry {
    //         binding: bufferID as u32,
    //         visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX_FRAGMENT,
    //         ty: wgpu::BindingType::Buffer {
    //             ty: wgpu::BufferBindingType::Storage {read_only: readonly},
    //             has_dynamic_offset: false,
    //             min_binding_size: None,
    //         },
    //         count: None,
    //     };
    // }
    
    // pub fn updateBindGroup(&mut self, device: &wgpu::Device){
    //     let mut entries = vec![];
    //     for i in 0..self.layout_entries.len() {
    //         entries.push(wgpu::BindGroupEntry {
    //             binding: i as u32,
    //             resource: self.buffers[i].as_entire_binding(),
    //         });
    //     }
    //     self.bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    //         entries: &self.layout_entries,
    //         label: Some(&self.label),
    //     });
    //     self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
    //         layout: &self.bind_group_layout,
    //         entries: &entries,
    //         label: Some(&self.label),
    //     });
    // }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub speed: f32,
    pub view_proj: [[f32; 4]; 4],
}

impl Camera {
    pub fn new(config: &WGPUConfig) -> Self {
        use cgmath::SquareMatrix;
        let mut returnVal =  Self {
            eye: (0.0, 0.0, 1.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: config.size.width as f32 / config.size.height as f32,
            fovy: 90.0,
            znear: 0.05,
            zfar: 1141.0,
            speed: 10.2,
            view_proj: cgmath::Matrix4::identity().into(),
        };

        returnVal.update_view_proj(config);

        return returnVal;
        
    }
    pub fn build_view_projection_matrix(&mut self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        // let view = cgmath::Matrix3::look_at_rh(self.eye, self.up);
        
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }

    pub fn process_mouse_movement(&mut self, delta_x: f32, delta_y: f32, sensitivity: f32) {
        use cgmath::{Matrix4, Rad, Vector3};

        // Adjust mouse delta by sensitivity
        let delta_x = delta_x * sensitivity;
        let delta_y = delta_y * sensitivity;

        // Calculate new rotation angles
        let horizontal_angle = Rad(delta_x);
        let vertical_angle = Rad(delta_y);

        // Get current direction vector from eye to target
        let mut direction = self.target - self.eye;

        // Rotate around the up axis for horizontal movement
        let horizontal_rotation = Matrix4::from_axis_angle(self.up, horizontal_angle);
        direction = horizontal_rotation.transform_vector(direction);

        // Rotate around the right axis (perpendicular to up and direction) for vertical movement
        let right = direction.cross(self.up).normalize();
        let vertical_rotation = Matrix4::from_axis_angle(right, vertical_angle);
        direction = vertical_rotation.transform_vector(direction);

        // Update target position
        self.target = self.eye + direction;
        self.view_proj = self.build_view_projection_matrix().into();
    }

    pub fn eye(&self) -> [[f32; 4]; 4] {
        return [
            [self.eye.x, self.eye.y, self.eye.z, 0.0],
            [self.eye.x, self.eye.y, self.eye.z, 0.0],
            [self.eye.x, self.eye.y, self.eye.z, 0.0],
            [self.eye.x, self.eye.y, self.eye.z, 0.0],
        ];
    }

    pub fn target(&self) -> [[f32; 4]; 4] {
        return [
            [self.target.x, self.target.y, self.target.z, 0.0],
            [self.target.x, self.target.y, self.target.z, 0.0],
            [self.target.x, self.target.y, self.target.z, 0.0],
            [self.target.x, self.target.y, self.target.z, 0.0],
        ];
    }

    pub fn update_view_proj(&mut self, config: &WGPUConfig) {
        self.aspect = config.size.width as f32 / config.size.height as f32;
        self.view_proj = self.build_view_projection_matrix().into();
        // self.rot_mat = 
    }

    pub fn zoom(&mut self, zoom: f32){
        let forward = self.target - self.eye;
        let forward_norm = forward.normalize();
        let mag = (self.target - self.eye).magnitude();
        self.eye += forward_norm * mag/(zoom);
    }
}
 
pub struct DepthBuffer {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler
}

impl DepthBuffer {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.
    
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, label: &str) -> Self {
        let size = wgpu::Extent3d { // 2.
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor { // 4.
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual), // 5.
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );

        Self { texture, view, sampler }
    }
}
 