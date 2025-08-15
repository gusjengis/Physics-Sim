use crate::settings;
use crate::wgpu_structs::*;
use crate::window_init;
use wgpu::util::DeviceExt;
use wgpu::BackendOptions;
use wgpu::Backends;
use wgpu::RequestAdapterOptions;
use wgpu::Trace;

pub struct WGPUConfig {
    #[allow(dead_code)]
    pub instance: wgpu::Instance,
    #[allow(dead_code)]
    pub adapter: wgpu::Adapter,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub surface_format: wgpu::TextureFormat,
    pub f64_support: bool,
    // dim_uniform: Uniform,
    // cursor_uniform: Uniform,
}

impl WGPUConfig {
    // Creating some of the wgpu types requires async code

    pub async fn new(canvas: &window_init::Canvas) -> Self {
        let size = canvas.size;

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // # Safety
        //
        // The surface needs to live as long as the canvas that created it.
        // State owns the canvas so this should be safe.
        let surface = unsafe {
            instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(&canvas.window).expect("raw window/display handles"))
                .expect("create surface")
        };

        #[cfg(not(target_arch = "wasm32"))]
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("No suitable GPU adapters found.");

        #[cfg(target_arch = "wasm32")]
        // let adapter = instance
        //     .enumerate_adapters(wgpu::Backends::BROWSER_WEBGPU)
        //     .filter(|adapter| {
        //         // Check if this adapter supports our surface
        //         adapter.is_surface_supported(&surface)
        //     })
        //     .next()
        //     .unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // let descriptor = wgpu::DeviceDescriptor {
        //     features: wgpu::Features::empty(),
        //     limits: wgpu::Limits {
        //         max_compute_workgroups_per_dimension: 65535,
        //         ..Default::default()
        //     },
        //     label: None,
        // };
        let limits = wgpu::Limits {
            max_texture_dimension_1d: 8192,
            max_texture_dimension_2d: 8192,
            max_texture_dimension_3d: 2048,
            max_texture_array_layers: 256,
            max_bind_groups: 8, // changed
            max_bindings_per_bind_group: 1000,
            max_dynamic_uniform_buffers_per_pipeline_layout: 8,
            max_dynamic_storage_buffers_per_pipeline_layout: 4,
            max_sampled_textures_per_shader_stage: 16,
            max_samplers_per_shader_stage: 16,
            max_storage_buffers_per_shader_stage: 16, // changed
            max_storage_textures_per_shader_stage: 4,
            max_uniform_buffers_per_shader_stage: 12,
            max_binding_array_elements_per_shader_stage: 0,
            max_binding_array_sampler_elements_per_shader_stage: 0,
            max_uniform_buffer_binding_size: 64 << 10,  // (64 KiB)
            max_storage_buffer_binding_size: 128 << 20, // (128 MiB)
            max_vertex_buffers: 16,
            max_buffer_size: 256 << 20, // (256 MiB)
            max_vertex_attributes: 16,
            max_vertex_buffer_array_stride: 2048,
            min_uniform_buffer_offset_alignment: 256,
            min_storage_buffer_offset_alignment: 256,
            max_inter_stage_shader_components: 60,
            max_color_attachments: 8,
            max_color_attachment_bytes_per_sample: 32,
            max_compute_workgroup_storage_size: 16384,
            max_compute_invocations_per_workgroup: 256,
            max_compute_workgroup_size_x: 256,
            max_compute_workgroup_size_y: 256,
            max_compute_workgroup_size_z: 64,
            max_compute_workgroups_per_dimension: 65535,
            min_subgroup_size: 0,
            max_subgroup_size: 0,
            max_push_constant_size: 0,
            max_non_sampler_bindings: 1_000_000,
        };

        let mut dev_temp = None;
        let mut que_temp = None;
        let mut f64_support = adapter.features().contains(wgpu::Features::SHADER_F64);
        let mut features = wgpu::Features::VERTEX_WRITABLE_STORAGE;
        if f64_support {
            features |= wgpu::Features::SHADER_F64;
        }

        let desc = wgpu::DeviceDescriptor {
            label: None,
            required_features: features,
            required_limits: limits.clone(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: Trace::default(),
        };
        match adapter.request_device(&desc).await {
            Ok((dev, que)) => {
                dev_temp = Some(dev);
                que_temp = Some(que)
            }
            Err(_) => {
                println!("Warning: GPU doesn't support f64 compute, this feature will be unavailable.");
                let temp = adapter.request_device(&desc).await.unwrap();
                dev_temp = Some(temp.0);
                que_temp = Some(temp.1);
                f64_support = false;
            }
        }

        let (device, queue) = (dev_temp.unwrap(), que_temp.unwrap());

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb()) // this line is sus, changed f.describe().srgb to f.is_srgb(), describe was not a thing
            .next()
            .unwrap_or(surface_caps.formats[0]);
        // println!("{:?}", );
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT, // | wgpu::TextureUsages::COPY_SRC,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Self {
            instance,
            adapter,
            surface,
            device,
            queue,
            config,
            size,
            surface_format,
            f64_support,
        }
    }
}
