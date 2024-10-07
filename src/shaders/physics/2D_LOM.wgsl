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

struct GridInfo {
    cell_size: f32,
    cell_cap: i32,
    w: i32,
    h: i32,
}

struct Contact {
    b: i32,
    forces: vec2<f32>,
    moment: f32,
    s_force: vec2<f32>,
    theta_b:  f32,
    bond_type: i32,
    bond_length: f32,
    bond_angle: f32
};

struct Material {
    red: f32,
    green: f32,
    blue: f32,
    density: f32,
    normal_stiffness: f32,
    shear_stiffness: f32,
}

struct Settings {
    walls: i32,
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
    moment_contribution_factor: f32,
    collision_interval: i32,
    local_damping: i32,
    local_damping_alpha: f32,
    particles: i32,
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
@group(2) @binding(1) var<storage, read_write> contacts: array<Contact>;
@group(2) @binding(3) var<storage, read_write> material_pointers: array<i32>;
@group(2) @binding(4) var<storage, read_write> grid: array<atomic<i32>>;
@group(2) @binding(5) var<storage, read_write> grid_info_buffer: array<GridInfo>;
@group(2) @binding(6) var<storage, read_write> coll_cont: array<i32>;
@group(3) @binding(0) var<uniform> settings: Settings;
@group(4) @binding(0) var<storage, read_write> data: array<f32>;
@group(5) @binding(0) var<storage, read_write> materials: array<Material>; 


const PI = 3.141592653589793238;
const DATA_SIZE = 7u;
const MAX_CONTACTS = 14u;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    if radii[id] == 0.0 || id >= u32(settings.particles) { return; }
    let mat_id = material_pointers[id];
     
    if coll_cont[2] != 0 { sum_forces(id, mat_id); }

    let grid_info = grid_info_buffer[0];
    
    var int_vel = velocities[id];
    var int_rot_vel = rot_vel[id];

    if fixity[id].x_vel == 0 { int_vel.x += accelerations[id].x * settings.dT * 0.5; }
    if fixity[id].y_vel == 0 { int_vel.y += accelerations[id].y * settings.dT * 0.5; }
    if fixity[id].rot_vel == 0 { int_rot_vel += rot_acc      [id] * settings.dT * 0.5; }

    data[id * DATA_SIZE + 4u] = int_vel.x;
    data[id * DATA_SIZE + 5u] = int_vel.y;
    data[id * DATA_SIZE + 6u] = int_rot_vel;

    if coll_cont[3] == 0 {
        del_pos[id] = int_vel * settings.dT;
        del_rot[id] = int_rot_vel * settings.dT;
    }

    // Walls
    // if settings.walls == 1 {
    let new_pos = positions[id] + del_pos[id];
    if settings.round_bounds == 0 {
        let yH = settings.vert_bound / 2.0;
        let xW = settings.hor_bound / 2.0;

        if fixity[id].y_vel != 1 {
            if new_pos.y - radii[id] < -yH {
                int_vel.y = -int_vel.y * 0.5;
                positions[id].y += -yH - (new_pos.y - radii[id]);
                if fixity[id].rot_vel != 1 { rot_vel[id] = rot_vel[id] * 0.9; }
            } else if new_pos.y + radii[id] > yH {
                int_vel.y = -int_vel.y * 0.5;
                positions[id].y -= (new_pos.y + radii[id]) - yH;
                if fixity[id].rot_vel != 1 { rot_vel[id] = rot_vel[id] * 0.9; }
            }
        }
        if fixity[id].x_vel != 1 {
            if new_pos.x - radii[id] < -xW {
                int_vel.x = -int_vel.x * 0.5;
                positions[id].x += -xW - (new_pos.x - radii[id]);
                if fixity[id].rot_vel != 1 { rot_vel[id] = rot_vel[id] * 0.9; }
            } else if new_pos.x + radii[id] > xW {
                int_vel.x = -int_vel.x * 0.5;
                positions[id].x -= (new_pos.x + radii[id]) - xW;
                if fixity[id].rot_vel != 1 { rot_vel[id] = rot_vel[id] * 0.9; }
            }
        }
    } else if length(new_pos) + radii[id] > settings.bound_radius { // circular bounds
        let norm_pos = normalize(new_pos);
        let del_comp = dot(del_pos[id], norm_pos) * norm_pos;
        let comp_v_p = dot(int_vel, norm_pos) * norm_pos;
        if fixity[id].x_vel != 1 {
            positions[id].x = norm_pos.x * (settings.bound_radius - radii[id]) - del_comp.x;
            int_vel.x = -comp_v_p.x * 0.5;
        }
        if fixity[id].y_vel != 1 {
            positions[id].y = norm_pos.y * (settings.bound_radius - radii[id]) - del_comp.y;
            int_vel.y = -comp_v_p.y * 0.5;
        }
        if fixity[id].rot_vel != 1 {
            rot_vel[id] = rot_vel[id] * 0.9;
        }    
    }
    // }

    positions[id] += del_pos[id];
    rot[id] += del_rot[id];

    velocities[id] = int_vel;
    rot_vel[id] = int_rot_vel;

    // determine if simulation shader is going to search for new collisions
    if id == 0u {
        coll_cont[1] += 1;
        if coll_cont[1] >= settings.collision_interval {
            coll_cont[0] = 1;
            coll_cont[1] = 0;
        } else {
            coll_cont[0] = 0;
        }
    }

    // Broad Phase
    if coll_cont[0] == 1 {
        let base_x = -grid_info.cell_size * f32(grid_info.w) * 0.5;
        let base_y = grid_info.cell_size * f32(grid_info.h) * 0.5;

        let particle_left = positions[id].x - radii[id];
        let particle_right = positions[id].x + radii[id];
        let particle_bottom = positions[id].y - radii[id];
        let particle_top = positions[id].y + radii[id];

        let min_cell_x = max(i32((particle_left - base_x) / grid_info.cell_size), 0);
        let max_cell_x = min(i32((particle_right - base_x) / grid_info.cell_size), grid_info.w - 1);
        let min_cell_y = max(i32((base_y - particle_top) / grid_info.cell_size), 0);
        let max_cell_y = min(i32((base_y - particle_bottom) / grid_info.cell_size), grid_info.h - 1);

        for (var cell_y = min_cell_y; cell_y <= max_cell_y; cell_y++) {
            for (var cell_x = min_cell_x; cell_x <= max_cell_x; cell_x++) {
                let base_index = (cell_y * grid_info.w + cell_x) * grid_info.cell_cap;
                if atomicExchange(&grid[base_index + 1], coll_cont[2]) != coll_cont[2] { // store tick number in cell so we know when it was last updated and if we need to reset it
                    atomicStore(&grid[base_index + 0], 0);
                }
            }
        }

        storageBarrier();

        for (var cell_y = min_cell_y; cell_y <= max_cell_y; cell_y++) {
            for (var cell_x = min_cell_x; cell_x <= max_cell_x; cell_x++) {
                let base_index = (cell_y * grid_info.w + cell_x) * grid_info.cell_cap;
                let p_count = atomicAdd(&grid[base_index + 0], 1) + 1;
                if p_count < grid_info.cell_cap - 1 {
                    grid[base_index + 1 + p_count] = i32(id);
                }
            }
        }
    }
}

fn sum_forces(id: u32, mat_id: i32) {
    var net_force = vec3(0.0);
    var contact_indices = array<u32, MAX_CONTACTS>();
    var contact_count = 0u;

    // Collect valid contact indices
    for (var i = id * MAX_CONTACTS; i < (id + 1u) * MAX_CONTACTS; i++) {
        if contacts[i].b >= 0 {
            contact_indices[contact_count] = i;
            contact_count += 1u;
        }
    }
    if (contact_count > 1u) {
        // Sort the contact indices based on the other particle's ID using a simple sorting algorithm
        for (var i = 0u; i < contact_count - 1u; i++) {
            for (var j = i + 1u; j < contact_count; j++) {
                let a_id = contacts[(contact_indices[i])].b;
                let b_id = contacts[(contact_indices[j])].b;
                if a_id > b_id {
                    // Swap indices to sort in ascending order
                    let temp = contact_indices[i];
                    contact_indices[i] = contact_indices[j];
                    contact_indices[j] = temp;
                }
            }
        }
    }

    // Sum forces in the sorted order
    for (var idx = 0u; idx < contact_count; idx++) {
        let i = contact_indices[idx];
        if contacts[i].b > i32(id) {
            net_force += vec3(contacts[i].forces, contacts[i].moment);
            if settings.bonds == 3 { net_force.z += contacts[i].bond_angle; }
        } else if contacts[i].b >= 0 {  
            // For contacts where the other particle has a lower ID, ensure consistent force retrieval
            for (var j = u32(contacts[i].b) * MAX_CONTACTS; j < (u32(contacts[i].b) + 1u) * MAX_CONTACTS; j++) {
                if u32(contacts[j].b) == id {
                    net_force += vec3(-contacts[j].forces, contacts[j].moment);
                    if settings.bonds == 3 { net_force.z += contacts[i].bond_angle; }
                    break; 
                }   
            }   
        }       
    }    
    //for (var i = id * MAX_CONTACTS; i < (id + 1u) * MAX_CONTACTS; i++) {
    //    if contacts[i].b > i32(id) {
    //       net_force += vec3(contacts[i].forces, contacts[i].moment);
    //    } else if contacts[i].b >= 0 {  
    //        for (var j = u32(contacts[i].b) * MAX_CONTACTS; j < (u32(contacts[i].b) + 1u) * MAX_CONTACTS; j++) {
    //            if u32(contacts[j].b) == id {
    //                net_force += vec3(-contacts[j].forces, contacts[j].moment);
    //                break; 
    //            }   
    //        }   
    //    }     
    //}    
    //data[id * DATA_SIZE + 0u] = net_force.x;
    //data[id * DATA_SIZE + 1u] = net_force.y;
    //data[id * DATA_SIZE + 2u] = net_force.z;
    // Apply sum of forces and gravity to velocities
    let density = materials[mat_id].density;
    let mass = density * PI * radii[id] * radii[id];
    let rot_inertia = 0.5 * mass * radii[id] * radii[id];
  
    var force = net_force.xy + vec2(forces[id].x, forces[id].y);
    var moment = net_force.z * radii[id] + forces[id].rot;
    
    // gravity
    var gravity = vec2(0.0, 0.0);
    if settings.gravity == 1 && settings.planet_mode == 1 {
        var center_of_gravity = vec2(0.0, 0.0);
        if settings.mouse_gravity == 1 {
            center_of_gravity = vec2(settings.gravity_x, settings.gravity_y);
        }
        let delta = (center_of_gravity - positions[id]);
        if delta.x != 0.0 || delta.y != 0.0 {
            force += (delta / length(delta) * 9.81 * settings.gravity_acc) * mass;
        }
    } else if settings.gravity == 1 {
        let gravity_acc = 9.81 * settings.gravity_acc;
        force += vec2(0.0, -gravity_acc * mass);
    }

    //damping
    // if settings.local_damping == 1 {
    //     let alpha = settings.local_damping_alpha;
    //     force  += vec2(abs(force.x), abs(force.y))  * alpha * -vec2(sign(velocities[id].x), sign(velocities[id].y));
    //     moment += moment         * alpha * -sign(rot_vel[id]);
    // }

    // natural accelerations
    accelerations[id] = (force) / mass;
    rot_acc[id] = (moment) / rot_inertia;


    if fixity[id].x_vel == 0 { velocities[id].x += 0.5 * accelerations[id].x * settings.dT; }
    if fixity[id].y_vel == 0 { velocities[id].y += 0.5 * accelerations[id].y * settings.dT; }
    if fixity[id].rot_vel == 0 { rot_vel[id] += 0.5 * rot_acc[id] * settings.dT; }

    // // artifical accelerations
    // velocities[id] += 0.5 * vec2(forces[id].x, forces[id].y) * settings.dT;
    // rot_vel[id] += forces[id].rot*settings.dT;
}
