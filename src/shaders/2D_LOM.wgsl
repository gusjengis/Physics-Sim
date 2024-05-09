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

struct Settings {
    hor_bound: f32,
    vert_bound: f32,
    gravity: i32,
    planet_mode: i32,
    bonds: i32,
    collisions: i32,
    friction: i32,
    friction_coefficient: f32,
    rotation: i32,
    linear_contact_bonds: i32,
    gravity_acc: f32,
    stiffness: f32,
    bonds_tear: i32,
    bond_force_limit: f32,
    contact_damping: f32,
    bond_damping: f32,
    drag: f32,
    bond_shear_lim: f32,
    verlet: i32,
    dT: f32
}

@group(0) @binding(0) var<storage, read_write> positions: array<vec2<f32>>;
@group(0) @binding(1) var<storage, read_write> radii: array<f32>;
@group(1) @binding(0) var<storage, read_write> velocities: array<vec2<f32>>;
@group(1) @binding(1) var<storage, read_write> accelerations: array<vec2<f32>>;
@group(1) @binding(2) var<storage, read_write> rot: array<f32>;
@group(1) @binding(3) var<storage, read_write> rot_vel: array<f32>;
@group(1) @binding(4) var<storage, read_write> rot_acc: array<f32>;
@group(1) @binding(5) var<storage, read_write> acc: array<vec3<f32>>;
@group(1) @binding(6) var<storage, read_write> fixity: array<Particle_Settings>;
@group(1) @binding(7) var<storage, read_write> forces: array<Forces>;
@group(1) @binding(8) var<storage, read_write> del_pos: array<vec2<f32>>;
@group(1) @binding(9) var<storage, read_write> del_rot: array<f32>;
@group(2) @binding(0) var<uniform> settings: Settings;


// const dT: f32 = 0.000005;//0.0000391236;
const PI = 3.141592653589793238;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id: u32 = global_id.x;
    if radii[id] == 0.0 { return; }
    var int_vel = velocities[id];
    var int_rot_vel = rot_vel[id];
    
    if fixity[id].x_vel   == 0 { int_vel.x   += accelerations[id].x * settings.dT * 0.5; }
    if fixity[id].y_vel   == 0 { int_vel.y   += accelerations[id].y * settings.dT * 0.5; }
    if fixity[id].rot_vel == 0 { int_rot_vel += rot_acc      [id]   * settings.dT * 0.5; }
    
    del_pos[id] = int_vel * settings.dT; 
    del_rot[id] = int_rot_vel * settings.dT; 

    positions[id] += del_pos[id];
    rot[id]       += del_rot[id];
    
    velocities[id] = int_vel;
    rot_vel[id] = int_rot_vel;
}