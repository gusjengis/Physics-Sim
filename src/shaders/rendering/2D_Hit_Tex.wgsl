# physics_structs;
# rendering_structs;
# rendering_settings;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec2<f32>,
    @location(1) rot: f32,
    @location(2) rot_vel: f32,
    @location(3) id: u32
};

@group(0) @binding(0) var<uniform> input: Input;
@group(1) @binding(0) var<storage, read_write> pos: array<vec2<f32>>;
@group(1) @binding(1) var<storage, read_write> radii: array<f32>;
@group(2) @binding(2) var<storage, read_write> rot: array<f32>;
@group(2) @binding(3) var<storage, read_write> rot_vel: array<f32>;
@group(3) @binding(0) var<storage, read_write> bonds: array<Bond>;
@group(3) @binding(1) var<storage, read_write> contacts: array<Contact>;
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
            if contacts[i].b >= 0 {
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
