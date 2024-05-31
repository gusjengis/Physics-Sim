use crate::window_init;
use crate::settings;
use crate::wgpu_structs::*;
use wgpu::util::DeviceExt;

pub struct WGPUConfig {
    #[allow(dead_code)]
    pub instance: wgpu::Instance,
    #[allow(dead_code)]
    pub adapter: wgpu::Adapter,
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub prog_settings: settings::Settings,
    pub surface_format: wgpu::TextureFormat
    // dim_uniform: Uniform,
    // cursor_uniform: Uniform,


}

impl WGPUConfig {
    // Creating some of the wgpu types requires async code
    
    pub async fn new(canvas: &window_init::Canvas) -> Self {
        
        let size = canvas.size;

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        
        // # Safety
        //
        // The surface needs to live as long as the canvas that created it.
        // State owns the canvas so this should be safe.
        let surface = unsafe { instance.create_surface(&canvas.window) }.unwrap();

        

        #[cfg(not(target_arch="wasm32"))] 
        let adapter = instance
        .enumerate_adapters(wgpu::Backends::all())
        .filter(|adapter| {
            // Check if this adapter supports our surface
            adapter.is_surface_supported(&surface)
        })
        .next()
        .unwrap();

        #[cfg(target_arch="wasm32")] 
        let adapter = instance
        .enumerate_adapters(wgpu::Backends::BROWSER_WEBGPU)
        .filter(|adapter| {
            // Check if this adapter supports our surface
            adapter.is_surface_supported(&surface)
        })
        .next()
        .unwrap();
        // let adapter = instance.request_adapter(
        //     &wgpu::RequestAdapterOptions {
        //         power_preference: wgpu::PowerPreference::default(),
        //         compatible_surface: Some(&surface),
        //         force_fallback_adapter: false,
        //     },
        // ).await.unwrap();

        // let descriptor = wgpu::DeviceDescriptor {
        //     features: wgpu::Features::empty(),
        //     limits: wgpu::Limits {
        //         max_compute_workgroups_per_dimension: 65535,
        //         ..Default::default()
        //     },
        //     label: None,
        // };
        let limits = wgpu::Limits { //downlevel_defaults()
            max_texture_dimension_1d: 2048,
            max_texture_dimension_2d: 8192,
            max_texture_dimension_3d: 256,
            max_texture_array_layers: 256,
            max_bind_groups: 8,
            max_bindings_per_bind_group: 640,
            max_dynamic_uniform_buffers_per_pipeline_layout: 8,
            max_dynamic_storage_buffers_per_pipeline_layout: 4,
            max_sampled_textures_per_shader_stage: 16,
            max_samplers_per_shader_stage: 16,
            max_storage_buffers_per_shader_stage: 16,
            max_storage_textures_per_shader_stage: 4,
            max_uniform_buffers_per_shader_stage: 12,
            max_uniform_buffer_binding_size: 16 << 10,
            max_storage_buffer_binding_size: 128 << 20,
            max_vertex_buffers: 8,
            max_vertex_attributes: 16,
            max_vertex_buffer_array_stride: 2048,
            max_push_constant_size: 0,
            min_uniform_buffer_offset_alignment: 256,
            min_storage_buffer_offset_alignment: 256,
            max_inter_stage_shader_components: 60,
            max_compute_workgroup_storage_size: 16352,
            max_compute_invocations_per_workgroup: 256,
            max_compute_workgroup_size_x: 256,
            max_compute_workgroup_size_y: 256,
            max_compute_workgroup_size_z: 64,
            max_compute_workgroups_per_dimension: 65535,
            max_buffer_size: 1 << 28,
        };

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::VERTEX_WRITABLE_STORAGE,// | wgpu::Features::SHADER_F64,
                limits: limits,
                label: None,
            },
            None, // Trace path
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps.formats.iter()
            .copied()
            .filter(|f| f.is_srgb())// this line is sus, changed f.describe().srgb to f.is_srgb(), describe was not a thing
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);
    
        let prog_settings = settings::Settings::new(canvas);
 

        
         
        Self {
            instance,
            adapter,
            surface,
            device,
            queue,
            config,
            size,
            prog_settings,
            surface_format
        }
    }
}



