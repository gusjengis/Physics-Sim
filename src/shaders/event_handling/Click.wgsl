struct Input {
    x: i32,
    y: i32,
    release: i32,
    ctrl: i32
}

struct Particle_Settings {
    x_vel: i32,
    y_vel: i32,
    rot_vel: i32,
    x_vel_2: i32,
    y_vel_2: i32,
    rot_vel_2: i32,
}

struct Forces {
    x: f32,
    y: f32,
    rot: f32,
    delX: f32,
    delY: f32,
    delRot: f32,
}

@group(0) @binding(0) var<uniform> input: Input;
@group(1) @binding(0) var<storage, read_write> selections: array<i32>;
@group(1) @binding(1) var<storage, read_write> groups: array<i32>;
@group(2) @binding(0) var tex_sampler: texture_2d<f32>;
@group(0) @binding(1) var<storage, read_write> click_info: array<i32>;
@group(3) @binding(0) var<storage, read_write> velocities: array<vec2<f32>>;
@group(3) @binding(1) var<storage, read_write> velocities_buf: array<vec2<f32>>;
@group(3) @binding(2) var<storage, read_write> rot: array<f32>;
@group(3) @binding(3) var<storage, read_write> rot_vel: array<f32>;
@group(3) @binding(4) var<storage, read_write> rot_vel_buf: array<f32>;
@group(3) @binding(5) var<storage, read_write> acc: array<vec3<f32>>;
@group(3) @binding(6) var<storage, read_write> fixity: array<Particle_Settings>;
@group(3) @binding(7) var<storage, read_write> forces: array<Forces>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = i32(global_id.x);
    let pixel_coord = vec2(input.x, input.y);
    let pixel_color = textureLoad(tex_sampler, pixel_coord, 0);
    let clicked_particle = i32(u32((pixel_color.r)*255.0*255.0*255.0) + u32((pixel_color.g)*255.0*255.0) + u32((pixel_color.b)*255.0)) - 1;
    if input.ctrl == 0 && (clicked_particle < 0 || selections[clicked_particle] == 0) {
        selections[id] = 0;
    }
    if clicked_particle == id {
        click_info[0] = 1;
        select(clicked_particle);
    }// else if clicked_particle >= 0 && groups[id] >= 0 && groups[clicked_particle] == groups[id] {
    //    select(id);
    //}
}

fn select(id: i32) {
    if (fixity[id].x_vel == 1 && selections[id] == 0) || selections[id] == 2 {
            selections[id] = 2;
        } else {
            selections[id] = 1;
        }
        if  fixity[id].x_vel   == 1 ||
            fixity[id].y_vel   == 1 ||
            fixity[id].rot_vel == 1 
            { selections[id] = 2; }
        else 
            { selections[id] = 1; }
}
