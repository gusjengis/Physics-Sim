# physics_structs;
# rendering_structs;
# rendering_settings;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@group(0) @binding(0) var tex_view: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;
@group(1) @binding(0) var<uniform> settings: Settings;
@group(2) @binding(0) var<uniform> input: Input;

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
        let noise = rand(u32((input.timestamp * (i32(dimensions.x) * i32(dimensions.y))) % 2000000000 + i32(pixel_coord.x)/settings.grain_size + i32(pixel_coord.y)/settings.grain_size * i32(dimensions.x)), settings.grain_strength);
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
        let ui_offset = vec2(input.ui_xOff/(input.width/input.height), -input.ui_yOff)/2.0 * input.scale;
        let center = vec2(0.5, 0.5) + ui_offset;
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

fn rand(seed: u32, max: f32) -> f32{
    //PCG Hash
    var res = seed;
    res = res * 747796405u + 2891336453u;
    res = ((res >> ((res >> 28u) + 4u)) ^ res) * 277803737u;
    res = (res >> 22u) ^ res;

    return max*f32(res)/4294967296.0;
}
