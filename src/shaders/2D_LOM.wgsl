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
    round_bounds: i32,
    bound_radius: f32,
    wall_friction: f32,
    gravity: i32,
    planet_mode: i32,
    bonds: i32,
    collisions: i32,
    friction_coefficient: f32,
    gravity_acc: f32,
    bond_normal_stiffness: f32,
    bonds_tear: i32,
    bond_normal_strength: f32,
    contact_damping: f32,
    bond_damping: f32,
    drag: f32,
    bond_shear_strength: f32,
    dT: f32,
    bond_shear_stiffness: f32,
    gravity_x: f32,
    gravity_y: f32,
    mouse_gravity: i32,
    moment_contribution_factor: f32
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
@group(3) @binding(0) var<uniform> settings: Settings;


const PI = 3.141592653589793238;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    if radii[id] == 0.0 { return; }
    var int_vel     = velocities[id];
    var int_rot_vel = rot_vel[id];
    
    if fixity[id].x_vel   == 0 { int_vel.x   += accelerations[id].x * settings.dT * 0.5; }
    if fixity[id].y_vel   == 0 { int_vel.y   += accelerations[id].y * settings.dT * 0.5; }
    if fixity[id].rot_vel == 0 { int_rot_vel += rot_acc      [id]   * settings.dT * 0.5; }
    
    del_pos[id] = int_vel     * settings.dT; 
    del_rot[id] = int_rot_vel * settings.dT; 

    // Walls
    let new_pos = positions[id] + del_pos[id];
    if settings.round_bounds == 0 {
        let yH = settings.vert_bound/2.0;
        let xW = settings.hor_bound/2.0;

        if fixity[id].y_vel != 1 {
            if new_pos.y-radii[id] < -yH {
                int_vel.y = -int_vel.y * 0.5;
                positions[id].y += -yH - (new_pos.y-radii[id]);
                if fixity[id].rot_vel != 1 { rot_vel[id] = rot_vel[id]*0.9; }
            } else if new_pos.y+radii[id] > yH {
                int_vel.y = -int_vel.y * 0.5;
                positions[id].y -= (new_pos.y+radii[id]) - yH;
                if fixity[id].rot_vel != 1 { rot_vel[id] = rot_vel[id]*0.9; }
            }
        }
        if fixity[id].x_vel != 1 {
            if new_pos.x-radii[id] < -xW {
                int_vel.x = -int_vel.x * 0.5;
                positions[id].x += -xW - (new_pos.x-radii[id]);
                if fixity[id].rot_vel != 1 { rot_vel[id] = rot_vel[id]*0.9; }
            } else if new_pos.x+radii[id] > xW {
                int_vel.x = -int_vel.x * 0.5;
                positions[id].x -= (new_pos.x+radii[id]) - xW;
                if fixity[id].rot_vel != 1 { rot_vel[id] = rot_vel[id]*0.9; }
            }
        }
    } else if length(new_pos) + radii[id] > settings.bound_radius { // circular bounds
        let norm_pos = normalize(new_pos);
        let del_comp = dot(del_pos[id], norm_pos) * norm_pos;
        let comp_v_p = dot(int_vel, norm_pos) * norm_pos;
        if fixity[id].x_vel != 1 {
            positions[id].x = norm_pos.x * (settings.bound_radius - radii[id]) - del_comp.x;
            int_vel.x -= comp_v_p.x * 1.5;
        }
        if fixity[id].y_vel != 1 {
            positions[id].y = norm_pos.y * (settings.bound_radius - radii[id]) - del_comp.y;
            int_vel.y -= comp_v_p.y * 1.5;
        }
        if fixity[id].rot_vel != 1 {
            rot_vel[id] = rot_vel[id]*0.9;
        }    
    }

    positions[id] += del_pos[id];
    rot[id]       += del_rot[id];
    
    velocities[id] = int_vel;
    rot_vel[id] = int_rot_vel;
}