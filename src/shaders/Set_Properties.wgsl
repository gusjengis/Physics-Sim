struct Input {
    set_x_pos: i32,
    set_y_pos: i32,
    set_rot: i32,
    set_x_vel: i32,
    set_y_vel: i32,
    set_rot_vel: i32,
    set_x_force: i32,
    set_y_force: i32,
    set_rot_force: i32,
    set_radius: i32,
    set_x_fixity: i32,
    set_y_fixity: i32,
    set_rot_fixity: i32,
    set_material: i32,
    x_pos: f32,
    y_pos: f32,
    rot: f32,
    x_vel: f32,
    y_vel: f32,
    rot_vel: f32,
    x_force: f32,
    y_force: f32,
    rot_force: f32,
    radius: f32,
    x_fixity: i32,
    y_fixity: i32,
    rot_fixity: i32,
    material: i32,
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

@group(0) @binding(0) var<storage, read_write> positions: array<vec2<f32>>;
@group(0) @binding(1) var<storage, read_write> radii: array<f32>;
@group(1) @binding(0) var<storage, read_write> velocities: array<vec2<f32>>;
@group(1) @binding(1) var<storage, read_write> velocities_buf: array<vec2<f32>>;
@group(1) @binding(2) var<storage, read_write> rot: array<f32>;
@group(1) @binding(3) var<storage, read_write> rot_vel: array<f32>;
@group(1) @binding(4) var<storage, read_write> rot_vel_buf: array<f32>;
@group(1) @binding(5) var<storage, read_write> acc: array<vec3<f32>>;
@group(1) @binding(6) var<storage, read_write> fixity: array<Particle_Settings>;
@group(1) @binding(7) var<storage, read_write> forces: array<Forces>;
@group(1) @binding(8) var<storage, read_write> del_pos: array<vec2<f32>>;
@group(1) @binding(9) var<storage, read_write> del_rot: array<f32>;
@group(2) @binding(3) var<storage, read_write> material_pointers: array<i32>;
@group(3) @binding(0) var<storage, read_write> selections: array<i32>;
@group(3) @binding(1) var<storage, read_write> groups: array<i32>;
@group(4) @binding(0) var<uniform> input: Input;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id: u32 = global_id.x;

    if selections[id] != 0 {
        if input.set_x_pos       == 1 { del_pos[id].x = input.x_pos - positions[id].x; positions[id].x = input.x_pos; } 
        if input.set_y_pos       == 1 { del_pos[id].y = input.y_pos - positions[id].y; positions[id].y = input.y_pos; } 
        if input.set_rot         == 1 { del_rot[id] = input.rot - rot[id]; rot[id] = input.rot; } 
        if input.set_x_vel       == 1 { velocities[id].x      = input.x_vel; } 
        if input.set_y_vel       == 1 { velocities[id].y      = input.y_vel; } 
        if input.set_rot_vel     == 1 { rot_vel[id]           = input.rot_vel; } 
        if input.set_radius      == 1 { radii[id]             = input.radius; } 
        if input.set_x_force     == 1 { forces[id].x          = input.x_force; } 
        if input.set_y_force     == 1 { forces[id].y          = input.y_force; } 
        if input.set_rot_force   == 1 { forces[id].rot        = input.rot_force; } 
        if input.set_x_fixity    == 1 { fixity[id].x_vel      = input.x_fixity; fixity[id].x_vel_2 = input.x_fixity; }
        if input.set_y_fixity    == 1 { fixity[id].y_vel      = input.y_fixity; fixity[id].y_vel_2 = input.y_fixity; }
        if input.set_rot_fixity  == 1 { fixity[id].rot_vel    = input.rot_fixity; fixity[id].rot_vel_2 = input.rot_fixity; }// rot_vel[id] = 0.0; } 
        if input.set_material    == 1 { material_pointers[id] = input.material; } 
        if input.set_x_fixity    == 1 ||
           input.set_y_fixity    == 1 ||
           input.set_rot_fixity  == 1 { selections[id]        = 2; }
    }
}