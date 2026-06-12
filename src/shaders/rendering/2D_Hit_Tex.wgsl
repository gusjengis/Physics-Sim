struct VertexIn {
    @location(0) position: vec2<f32>,
};

struct Input {
    width: f32, time: f32,
    height: f32,
    xOff: f32, yOff: f32,
    ui_xOff: f32, ui_yOff: f32,
    ui_scale: f32, scale: f32,
    dark: f32,
    x: i32, y: i32,
    rW: i32, rH: i32,
    pressed: i32,
    timestamp: i32,
    p_count: i32,
    $ 3D { cam: Camera, }
}

struct Contact {
    a: i32,
    b: i32,
    tangent_force: f32,
    bond_tangent_force: f32,
    theta_b: f32,
    bonded: i32
};

$ 3D {
    struct Camera {
        pos: vec4<f32>,
        view_proj: mat4x4<f32>,
        eye: mat4x4<f32>,
        focus: mat4x4<f32>,
    };
}

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
    @location(0) position: vec2<f32>,
    // @location(1) color: vec3<f32>,
    @location(1) rot: f32,
    @location(2) rot_vel: f32,
    @location(3) @interpolate(flat) id: u32
};

struct Settings {
    circular_particles: i32,
    render_rot: i32,
    color_code_rot: i32,
    colors: i32,
    render_bonds: i32,
    walls: i32,
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
    abb_strength: f32,
    bond_highlight_strength: f32,
    render_unbonded_contacts: i32
}
struct Bond {
    index: i32,
    angle: f32,
    length: f32
};

@group(0) @binding(0) var<uniform> input: Input;
@group(1) @binding(0) var<storage, read> pos: array<vec2<f32>>;
@group(1) @binding(1) var<storage, read> radii: array<f32>;
@group(2) @binding(2) var<storage, read> rot: array<f32>;
@group(2) @binding(3) var<storage, read> rot_vel: array<f32>;
@group(3) @binding(0) var<storage, read> bonds: array<Bond>;
@group(3) @binding(1) var<storage, read> contacts: array<Contact>;
@group(3) @binding(3) var<storage, read> material_pointers: array<i32>;
@group(0) @binding(1) var<uniform> settings: Settings;
@group(0) @binding(2) var<storage, read> materials: array<Material>;
@group(0) @binding(3) var<storage, read> selections: array<i32>;
@group(0) @binding(4) var<storage, read> click_info: array<i32>;

@vertex
fn vs_main(
    in: VertexIn,
    @builtin(instance_index) instance: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let scale = input.scale;
    $ 2D {    
        let aspect = input.width/input.height;
        let xy = 2.0*scale*vec2(in.position.x / aspect, in.position.y);
        let center = scale*vec2(pos[instance].x / aspect, pos[instance].y);
        let off = vec2((input.xOff + input.ui_xOff) / aspect, (input.yOff + input.ui_yOff))*(scale);
        out.clip_position = vec4(xy*radii[instance] + center + off, 0.0, 1.0);
    }
    $ 3D {
        let cam_pos = input.cam.pos.xyz;
        let part_pos = vec3(pos[instance].x, pos[instance].y, 0.0);
        let dir = normalize(cam_pos - part_pos);

        // Create a rotation matrix to face the direction
        let up = vec3(0.0, 1.0, 0.0);
        let right = normalize(cross(up, dir));
        let rotated_up = cross(dir, right);
        let rotation_matrix = mat3x3(
            right,
            rotated_up,
            dir
        );
        
        let rotated_position = rotation_matrix * vec3(in.position, 0.0);
        
        let xy = 2.0 * input.ui_scale * rotated_position;
        let center = input.ui_scale * vec3(pos[instance].x, pos[instance].y, 0.0);
        let off = vec3(input.ui_xOff, input.ui_yOff, 0.0) * input.ui_scale;
    
        let final_pos = xy * radii[instance] + center + off;
        
        out.clip_position = input.cam.view_proj * vec4(final_pos, 1.0);
    }

    out.position = in.position;
    out.rot = rot[instance];
    out.rot_vel = rot_vel[instance];
    out.id = instance+1u;
    return out;
}

const PI = 3.141592653589793238;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let len = length(in.position);
    $ ROUND-PARTICLES {
        // discard corners to make circle
        if len > 0.5 {
            discard;
        }
    
        let id = in.id - 1u;
        for(var i = id*14u; i<(id+1u)*14u; i++){
            if contacts[i].a >= 0 {
                let delta = pos[contacts[i].b] - pos[id];
                let dir = normalize(delta);
                let displacement = ((radii[id] + radii[contacts[i].b]) - length(delta));
                let midpoint = pos[id] + dir * (radii[id] - displacement * 0.5);
                let pixel_world_pos = pos[id] + radii[id] * in.position * 2.0;
                let test_vec = midpoint - pixel_world_pos;
                if dot(delta, test_vec) < 0.0 {
                    discard;
                }
            }
        }
    }

    let red = f32(in.id / (255u * 255u)) / 255.0;
    let green = f32((in.id / 255u) % 255u) / 255.0;
    let blue = f32(in.id % 255u) / 255.0;

    return vec4(
        (red),
        (green),
        (blue),
        1.0
    );
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

fn cross(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x
    );
}