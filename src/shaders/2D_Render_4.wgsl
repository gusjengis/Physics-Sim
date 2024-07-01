struct VertexIn {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

struct Settings {
    circular_particles: i32,
    render_rot: i32,
    color_code_rot: i32,
    colors: i32,
    render_bonds: i32,
    w: f32,
    h: f32,
    stiffness: f32,
    render_grid: i32,
    round_bounds: i32,
    wall_radius: f32,
    render_outline: i32,
    use_part_color: i32,
    background_r: f32,
    background_g: f32,
    background_b: f32,
    outline_r: f32,
    outline_g: f32,
    outline_b: f32,
    dim_slow_particles: i32,
    max_brightness_vel: f32,
    crt_res: i32,
    grain: i32,
    grain_strength: f32,
    grain_size: i32,
    sobel: i32,
    invert: i32,
    chrom_ab: i32,
    abb_strength: f32
}

struct Dimensions {
    width: f32, time: f32,
    height: f32, temp: f32,
    xOff: f32, yOff: f32,
    scale: f32, dark: f32,
    x: i32, y: i32,
    rW: i32, rH: i32,
    pressed: i32,
    timestamp: i32
}

@group(0) @binding(0) var tex_view: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;
@group(1) @binding(0) var<uniform> settings: Settings;
@group(2) @binding(0) var<uniform> dim: Dimensions;

@vertex
fn vs_main(
    in: VertexIn,
    @builtin(instance_index) instance: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4(in.position, 0.0, 1.0);
    return out;
}

const PI = 3.141592653589793238;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dimensions = vec2(f32(textureDimensions(tex_view).x), f32(textureDimensions(tex_view).y));
    let pixel_coord = vec2(in.clip_position.x, in.clip_position.y);
    var color = get_pixel(dimensions, pixel_coord);

    if settings.sobel == 1 || settings.sobel == 3 {
        let texel_size = 1.0 / vec2<f32>(textureDimensions(tex_view));
        let top_left = get_pixel(dimensions, pixel_coord + vec2(-1.0, -1.0)).rgb;
        let top = get_pixel(dimensions, pixel_coord + vec2(0.0, -1.0)).rgb;
        let top_right = get_pixel(dimensions, pixel_coord + vec2(1.0, -1.0)).rgb;
        let left = get_pixel(dimensions, pixel_coord + vec2(-1.0, 0.0)).rgb;
        let right = get_pixel(dimensions, pixel_coord + vec2(1.0, 0.0)).rgb;
        let bottom_left = get_pixel(dimensions, pixel_coord + vec2(-1.0, 1.0)).rgb;
        let bottom = get_pixel(dimensions, pixel_coord + vec2(0.0, 1.0)).rgb;
        let bottom_right = get_pixel(dimensions, pixel_coord + vec2(1.0, 1.0)).rgb;
        let x = (top_right + 2.0*right + bottom_right) - (top_left + 2.0*left + bottom_left);
        let y = (bottom_left + 2.0*bottom + bottom_right) - (top_left + 2.0*top + top_right);
        let edge = sqrt(x*x + y*y);
        let edge_intensity = (edge.r + edge.g + edge.b) / 3.0;
        if settings.sobel == 3 {
            color = color * min(1.0, edge_intensity);
        } else {
            color = vec4(edge_intensity, edge_intensity, edge_intensity, 1.0);
        }
    }

    if settings.invert == 1 {
        color = vec4(1.0 - color.rgb, color.a);
    }

    if i32(in.clip_position.y) % settings.crt_res != 0 {
        color = vec4(0.0, 0.0, 0.0, 1.0);
    }

    if settings.grain == 1 {
        let noise = rand(u32((dim.timestamp * (i32(dimensions.x) * i32(dimensions.y))) % 2000000000 + i32(pixel_coord.x)/settings.grain_size + i32(pixel_coord.y)/settings.grain_size * i32(dimensions.x)), settings.grain_strength);
        color = vec4(color.rgb + noise, 1.0);
    }

    return color;
    
}

fn noise(uv: vec2<f32>) -> f32 {
    let s = sin(dot(uv, vec2<f32>(12.9898, 78.233)));
    return fract(s * 43758.5453);
}

fn get_pixel(dimensions: vec2<f32>, coord: vec2<f32>) -> vec4<f32> {
    if settings.chrom_ab == 1 {
        let uv = coord / dimensions;
        let center = vec2(0.5, 0.5);
        var dist = length(uv - center);
        let noise_scale = 0.3; // Adjust this to control the amount of noise
        dist += (noise(uv * 10.0) - 0.5) * noise_scale;
        let offset = (uv - center) * dist * settings.abb_strength;
        
        let r = textureLoad(tex_view, vec2<i32>((uv + offset) * dimensions), 0).r;
        let g = textureLoad(tex_view, vec2<i32>(uv * dimensions), 0).g;
        let b = textureLoad(tex_view, vec2<i32>((uv - offset) * dimensions), 0).b;
        return vec4(r, g, b, 1.0);
    }
    return textureLoad(tex_view, vec2(i32(coord.x), i32(coord.y)), 0);
}

// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     // let pixel_coord = vec2<i32>(in.clip_position.xy);
//     // if i32(pixel_coord.y) % settings.crt_res != 0 {
//     //     return vec4(0.0, 0.0, 0.0, 1.0);
//     // }
//     // var color = textureLoad(tex_view, pixel_coord, 0);
//     // Sobel Filter
//     // let texel_size = 1.0 / vec2<f32>(textureDimensions(tex_view));

//     // let top_left = textureLoad(tex_view, pixel_coord + vec2<i32>(-1, -1), 0).rgb;
//     // let top = textureLoad(tex_view, pixel_coord + vec2<i32>(0, -1), 0).rgb;
//     // let top_right = textureLoad(tex_view, pixel_coord + vec2<i32>(1, -1), 0).rgb;
//     // let left = textureLoad(tex_view, pixel_coord + vec2<i32>(-1, 0), 0).rgb;
//     // let right = textureLoad(tex_view, pixel_coord + vec2<i32>(1, 0), 0).rgb;
//     // let bottom_left = textureLoad(tex_view, pixel_coord + vec2<i32>(-1, 1), 0).rgb;
//     // let bottom = textureLoad(tex_view, pixel_coord + vec2<i32>(0, 1), 0).rgb;
//     // let bottom_right = textureLoad(tex_view, pixel_coord + vec2<i32>(1, 1), 0).rgb;

//     // let x = (top_right + 2.0*right + bottom_right) - (top_left + 2.0*left + bottom_left);
//     // let y = (bottom_left + 2.0*bottom + bottom_right) - (top_left + 2.0*top + top_right);

//     // let edge = sqrt(x*x + y*y);
//     // let edge_intensity = (edge.r + edge.g + edge.b) / 3.0;
//     // color = color * min(1.0, edge_intensity);
//     // Chromatic Abberation
//     // let dimensions = vec2<f32>(textureDimensions(tex_view));
//     // let uv = in.clip_position.xy / dimensions;
//     // let center = vec2<f32>(0.5, 0.5);
//     // let offset = (uv - center) * 0.002; // Adjust the 0.02 to control the effect strength
    
//     // let r = textureLoad(tex_view, vec2<i32>((uv + offset) * dimensions), 0).r;
//     // let g = textureLoad(tex_view, vec2<i32>(uv * dimensions), 0).g;
//     // let b = textureLoad(tex_view, vec2<i32>((uv - offset) * dimensions), 0).b;
    
//     // return vec4<f32>(r, g, b, 1.0);
//     // Sepia    
//     // let sepia = vec3<f32>(
//     //     dot(color.rgb, vec3<f32>(0.393, 0.769, 0.189)),
//     //     dot(color.rgb, vec3<f32>(0.349, 0.686, 0.168)),
//     //     dot(color.rgb, vec3<f32>(0.272, 0.534, 0.131))
//     // );
    
//     // return vec4<f32>(sepia, color.a);
//     //grain
//     // let dimensions = vec2(i32(textureDimensions(tex_view).x), i32(textureDimensions(tex_view).y));
//     // let noise_strength = 0.002;
//     // let grain_size = 4;
//     // let noise = rand(u32((dim.timestamp * (dimensions.x * dimensions.y)) % 2000000000 + pixel_coord.x/grain_size + pixel_coord.y/grain_size * dimensions.x), noise_strength);
    
//     // return vec4(color.r + noise, color.g + noise, color.b + noise, 1.0);

//     //Cartoon    
//     // let pixel_coord = vec2<i32>(in.clip_position.xy);
//     // let color = textureLoad(tex_view, pixel_coord, 0);
    
//     // // Edge detection
//     // let left = textureLoad(tex_view, pixel_coord + vec2<i32>(-1, 0), 0).rgb;
//     // let right = textureLoad(tex_view, pixel_coord + vec2<i32>(1, 0), 0).rgb;
//     // let up = textureLoad(tex_view, pixel_coord + vec2<i32>(0, -1), 0).rgb;
//     // let down = textureLoad(tex_view, pixel_coord + vec2<i32>(0, 1), 0).rgb;
    
//     // let edge = (abs(left - right) + abs(up - down)) * 2.0;
//     // let edge_intensity = (edge.r + edge.g + edge.b) / 3.0;
    
//     // // Color quantization
//     // let levels = 5.0;
//     // let quantized_color = floor(color.rgb * levels) / levels;
    
//     // // Combine edge detection and quantized colors
//     // let cartoon = mix(color.rgb, vec3<f32>(0.0), step(0.8, edge_intensity));
    
//     // return vec4<f32>(cartoon, color.a);
// }

fn rand(seed: u32, max: f32) -> f32{
    //PCG Hash
    var res = seed;
    res = res * 747796405u + 2891336453u;
    res = ((res >> ((res >> 28u) + 4u)) ^ res) * 277803737u;
    res = (res >> 22u) ^ res;

    return max*f32(res)/4294967296.0;
}

// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     let dimensions = vec2<f32>(textureDimensions(tex_view));
//     let pixel_size = 8.0; // Adjust for larger/smaller pixels
//     let uv = floor(in.clip_position.xy / pixel_size) * pixel_size;
//     let pixel_coord = vec2<i32>(uv);
    
//     return textureLoad(tex_view, pixel_coord, 0);
// }
// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     var sum = vec4(0.0, 0.0, 0.0, 1.0);
//     let pixel_coord = vec2(i32(in.clip_position.x), i32(in.clip_position.y));
//     let kernel_dim = vec2(9, 9);
//     let x_offset = (kernel_dim.x - 1)/2;
//     let y_offset = (kernel_dim.y - 1)/2;
//     for(var x = -x_offset; x < x_offset + 1; x++){
//         for(var y = -y_offset; y < y_offset + 1; y++){
//             let weight = 1.0;//max(length(vec2(f32(x), f32(y))), 1.0);
//             // if weight > 25.0 {
//             //     continue;
//             // }
//             sum += textureLoad(tex_view, vec2(pixel_coord.x + x, pixel_coord.y + y), 0)/weight;
//         }
//     }
//     let pixel_color = textureLoad(tex_view, pixel_coord, 0);
//     let avg = sum/f32(kernel_dim.x * kernel_dim.y);
//     // let color = vec4(pixel_color.r, 0.0, 0.0, 0.0);
//     return vec4(max(avg.r, pixel_color.r), max(avg.g, pixel_color.g), max(avg.b, pixel_color.b), 1.0);
// }