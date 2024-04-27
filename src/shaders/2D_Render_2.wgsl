struct VertexIn {
    @location(0) position: vec2<f32>,
};

struct Dimensions {
    width: f32, time: f32,
    height: f32, temp: f32,
    xOff: f32, yOff: f32,
    scale: f32, dark: f32,
    x: i32, y: i32,
    rW: i32, rH: i32,
    pressed: i32
}

struct Camera {
    view_proj: mat4x4<f32>,
    eye: mat4x4<f32>,
    focus: mat4x4<f32>,
};

struct Material {
    red: f32,
    green: f32,
    blue: f32,
    density: f32,
    normal_stiffness: f32,
    shear_stiffness: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec4<f32>,
    // @location(1) color: vec3<f32>,
    @location(1) rot: f32,
    @location(2) rot_vel: f32,
    @location(3) id: u32,
    @location(4) w_h: vec2<i32>,
    @location(5) pixel: vec2<f32>,
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
    random_colors: i32
}

struct Bond {
    index: i32,
    angle: f32,
    length: f32
};

@group(0) @binding(0) var<uniform> dim: Dimensions;
@group(1) @binding(0) var<storage, read_write> pos_buf: array<vec2<f32>>;
@group(2) @binding(0) var<storage, read_write> radii_buf: array<f32>;
// @group(3) @binding(0) var<storage, read_write> color_buf: array<vec3<f32>>;
@group(3) @binding(2) var<storage, read_write> rot_buf: array<f32>;
@group(3) @binding(3) var<storage, read_write> rot_vel: array<f32>;
@group(4) @binding(0) var<storage, read_write> bonds: array<Bond>;
// @group(4) @binding(1) var<storage, read_write> bond_info: array<vec2<i32>>;
@group(4) @binding(3) var<storage, read_write> material_pointers: array<i32>;
@group(5) @binding(0) var<uniform> settings: Settings;
@group(6) @binding(0) var<storage, read_write> materials: array<Material>;
@group(7) @binding(0) var<storage, read_write> selections: array<i32>;


@vertex
fn vs_main(
    in: VertexIn,
    @builtin(instance_index) instance: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let aspect = dim.width/dim.height;
    let scale= dim.scale;
    let xy = 1.0/scale*vec2(in.position.x * aspect, in.position.y);
    let center = scale*vec2(pos_buf[instance].x / aspect, pos_buf[instance].y);
    let off = vec2(-dim.xOff, dim.yOff)/1000.0/scale;
    out.clip_position = vec4(in.position, 0.0, 1.0);
    out.position = vec4((xy + off), 0.0, 1.0);
    // out.color = color_buf[instance % u32(settings.colors)];
    // if material_pointers[instance] != -1 { out.color = vec3(materials[(material_pointers[instance])].red, materials[(material_pointers[instance])].green, materials[(material_pointers[instance])].blue); }
    out.rot = rot_buf[instance];
    out.rot_vel = rot_vel[instance];
    out.id = instance;
    out.w_h = vec2(i32(dim.width), i32(dim.height));
    out.pixel = out.clip_position.xy;
    return out;
}

const PI = 3.141592653589793238;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = vec4(0.0, 0.0, 0.0, 1.0); 
    if in.position.x >= settings.w || in.position.x <= -settings.w || in.position.y >= settings.h || in.position.y <= -settings.h {
        color = vec4(0.02, 0.02, 0.02, 1.0);
    }

    // if abs(in.position.x) % 1.0 < 0.003 || abs(in.position.x) % 1.0 > 0.997 || abs(in.position.y) % 1.0 < 0.003 || abs(in.position.y) % 1.0 > 0.997 {
    //     return vec4(0.04, 0.04, 0.04, 1.0);
    // }

    // if abs(in.position.x) % 0.5 < 0.002 || abs(in.position.x) % 0.5 > 0.998 || abs(in.position.y) % 0.5 < 0.002 || abs(in.position.y) % 0.5 > 0.998 {
    //     return vec4(0.04, 0.04, 0.04, 1.0);
    // }

    // if abs(in.position.x) % 0.25 < 0.001 || abs(in.position.x) % 0.25 > 0.999 || abs(in.position.y) % 0.25 < 0.001 || abs(in.position.y) % 0.25 > 0.999 {
    //     return vec4(0.04, 0.04, 0.04, 1.0);
    // }
    //done
    if dim.pressed == 1 {
        // let pos1 = in.scale*(in.position.xy);
        let pos = (vec2(in.pixel.x + 1.0, -in.pixel.y + 1.0))/2.0;
        let pixel = vec2(i32(pos.x*f32(in.w_h.x)),i32(pos.y*f32(in.w_h.y)));
        let lower_x = min(dim.x, dim.x + dim.rW);
        let upper_x = max(dim.x, dim.x + dim.rW);
        let lower_y = min(dim.y, dim.y + dim.rH);
        let upper_y = max(dim.y, dim.y + dim.rH);
        if pixel.x > lower_x && pixel.x < upper_x && pixel.y > lower_y && pixel.y < upper_y {
            if pixel.x == lower_x + 1 || pixel.x == upper_x - 1 || pixel.y == lower_y + 1 || pixel.y == upper_y - 1 {
                color = vec4(
                    srgb_to_linear(0.0/255.0),
                    srgb_to_linear(120.0/255.0),
                    srgb_to_linear(215.0/255.0),
                    0.0
                );
            } else {
                color = color + vec4(
                    srgb_to_linear(0.0/255.0),
                    srgb_to_linear(28.0/255.0),
                    srgb_to_linear(56.0/255.0),
                    0.0
                );
            }
        }
    }
    return color;
}

fn linear_to_srgb(value: f32) -> f32 {
    if (value <= 0.0031308) {
        return 12.92 * value;
    } else {
        return 1.055 * pow(value, 1.0 / 2.4) - 0.055;
    }
}

fn srgb_to_linear(value: f32) -> f32 {
    if value <= 0.04045 {
        return value / 12.92;
    } else {
        return pow((value + 0.055) / 1.055, 2.4);
    }
}