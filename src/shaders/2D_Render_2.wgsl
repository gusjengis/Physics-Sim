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
    pressed: i32,
    timestamp: i32
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

struct Bond {
    index: i32,
    angle: f32,
    length: f32
};

struct GridInfo {
    cell_size: f32,
    cell_cap: i32,
    w: i32,
    h: i32,
}

@group(0) @binding(0) var<uniform> dim: Dimensions;
@group(1) @binding(0) var<storage, read_write> pos_buf: array<vec2<f32>>;
@group(1) @binding(1) var<storage, read_write> radii_buf: array<f32>;
// @group(2) @binding(0) var<storage, read_write> color_buf: array<vec3<f32>>;
@group(2) @binding(2) var<storage, read_write> rot_buf: array<f32>;
@group(2) @binding(3) var<storage, read_write> rot_vel: array<f32>;
@group(3) @binding(0) var<storage, read_write> bonds: array<Bond>;
@group(3) @binding(4) var<storage, read_write> grid: array<i32>;
@group(3) @binding(5) var<storage, read_write> grid_info_buffer: array<GridInfo>;
// @group(3) @binding(1) var<storage, read_write> bond_info: array<vec2<i32>>;
@group(3) @binding(3) var<storage, read_write> material_pointers: array<i32>;
@group(4) @binding(0) var<uniform> settings: Settings;
@group(5) @binding(0) var<storage, read_write> materials: array<Material>;
@group(6) @binding(0) var<storage, read_write> selections: array<i32>;
@group(7) @binding(0) var<storage, read_write> click_info: array<i32>;

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
    let off = vec2(-dim.xOff, -dim.yOff);

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
    var color = vec4(vec3(
        srgb_to_linear(settings.background_r),
        srgb_to_linear(settings.background_g),
        srgb_to_linear(settings.background_b)
    ), 1.0); 
    if (settings.round_bounds == 1 && length(in.position.xy) > settings.wall_radius) || (settings.round_bounds == 0 && (in.position.x >= settings.w/2.0 || in.position.x <= -settings.w/2.0 || in.position.y >= settings.h/2.0 || in.position.y <= -settings.h/2.0)) {
        color = vec4(0.02, 0.02, 0.02, 1.0);
    } //else if settings.render_grid == 1 {
    //     let grid_info = grid_info_buffer[0];
    //     let base_x = -grid_info.cell_size*f32(grid_info.w)*0.5;
    //     let base_y =  grid_info.cell_size*f32(grid_info.h)*0.5;
    //     var cell_x = i32((in.position.x - base_x) / grid_info.cell_size);
    //     var cell_y = i32((base_y - in.position.y) / grid_info.cell_size);
    //     var cell_id = u32(cell_y) * u32(grid_info.w) + u32(cell_x);
    //     let left   = base_x + grid_info.cell_size * f32(cell_x    );
    //     let right  = base_x + grid_info.cell_size * f32(cell_x + 1);
    //     let top    = base_y - grid_info.cell_size * f32(cell_y    );
    //     let bottom = base_y - grid_info.cell_size * f32(cell_y + 1);
    //     let v1 = vec2(base_x, base_y);
    //     if cell_x >= 0 && cell_y >= 0 && base_x <= in.position.x && base_y >= in.position.y && cell_x < grid_info.w && cell_y < grid_info.h{
    //         let seed1 = u32(rand(cell_id, 4294967296.0));
    //         let seed2 = u32(rand(seed1,   4294967296.0));
    //         let seed3 = u32(rand(seed2,   4294967296.0));
    //         var brightness = f32(grid[i32(cell_id)*grid_info.cell_cap])/f32(grid_info.cell_cap);
    //         // var brightness = f32(grid[i32(cell_id)*grid_info.cell_cap + 1]);
    //         color = vec4(
    //             color.x + brightness * rand(seed1, 1.0) * 0.5,
    //             color.y + brightness * rand(seed2, 1.0) * 0.5,
    //             color.z + brightness * rand(seed3, 1.0) * 0.5,
    //             1.0
    //         );
    //     }
    //     if (in.position.x + settings.w < 0.005 || settings.w - in.position.x < 0.005 || in.position.y + settings.h < 0.005 || settings.h - in.position.y < 0.005 || in.position.x - left < 0.005 || right - in.position.x < 0.005 || in.position.y - bottom < 0.005 || top - in.position.y < 0.005) && cell_x < grid_info.w && cell_y < grid_info.h && cell_x >= 0 && cell_y >= 0  {
    //         color *= 4.0;
    //     }
    // }

    //done
    if dim.pressed == 1 && click_info[0] == 0 {
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

fn rand(seed: u32, max: f32) -> f32{
    //PCG Hash
    var res = seed;
    res = res * 747796405u + 2891336453u;
    res = ((res >> ((res >> 28u) + 4u)) ^ res) * 277803737u;
    res = (res >> 22u) ^ res;

    return max*f32(res)/4294967296.0;
}