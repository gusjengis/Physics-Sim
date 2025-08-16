# physics_structs;
# physics_settings;

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
@group(2) @binding(0) var<storage, read_write> bonds: array<Bond>;
@group(2) @binding(1) var<storage, read_write> contacts: array<Contact>;
@group(2) @binding(2) var<storage, read_write> contact_pointers: array<i32>;
@group(2) @binding(3) var<storage, read_write> material_pointers: array<i32>;
@group(2) @binding(4) var<storage, read_write> grid: array<i32>;
@group(2) @binding(5) var<storage, read_write> grid_info_buffer: array<GridInfo>;
@group(2) @binding(6) var<storage, read_write> coll_cont: array<i32>;
@group(3) @binding(0) var<uniform> settings: Settings;
@group(4) @binding(0) var<storage, read_write> materials: array<Material>; 
@group(5) @binding(0) var<storage, read_write> data: array<f32>; 
 
const PI = 3.141592653589793238;
const DATA_SIZE = 8u;
const MAX_CONTACTS = 14u;
 
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id: u32 = global_id.x;
    let grid_info = grid_info_buffer[0];

    data[id * DATA_SIZE   ] = 0.0;
    data[id * DATA_SIZE + 1u] = 0.0;
    data[id * DATA_SIZE + 2u] = 0.0;
    // data[id*DATA_SIZE+3u] = velocities[id].x;

    if id >= u32(settings.particles) || radii[id] == 0.0 { return; }

    var net_force = vec2(0.0, 0.0);
    var net_moment = 0.0;

    // OG O(n^2) Collisions
    if settings.collisions == 1 && coll_cont[0] == 1 {
        var collisions = array<i32, 14u>();
        var count = 0u;

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
    
        for (var cell_y = min_cell_y; cell_y <= max_cell_y && count < MAX_CONTACTS; cell_y++) {
            for (var cell_x = min_cell_x; cell_x <= max_cell_x && count < MAX_CONTACTS; cell_x++) {
                let cell_id = cell_y * grid_info.w + cell_x;  
                let base_index = cell_id * grid_info.cell_cap;  
                var neighbors = grid[base_index];  
 
                if neighbors > 1 { 
                    for (var i = 2; i < neighbors + 2; i++) {
                        let b = grid[base_index + i];
                        if u32(b) != id {
                            var already_found = false;
                            for (var j = 0u; j < count; j++) {
                                if collisions[j] == b {
                                    already_found = true;
                                }
                            }
                            if !already_found && length(positions[b] - positions[id]) < (radii[b] + radii[id]) {
                                collisions[count] = b; 
                                count += 1u;  
                                if count == MAX_CONTACTS {
                                    break; 
                                } 
                            } 
                        } 
                    }
                }
            }
        }
  
        // delete contacts that don't exist
        for (var j = id * MAX_CONTACTS; j < (id + 1u) * MAX_CONTACTS; j++) {
            if contacts[j].b == $ CSO {  0 } 
                                $ NCS { -1 } { 
                continue; 
            } 
            var b = contacts[j].b;
            
            $ CSO {
                if b < 0 {
                    b = i32(u32((-b - 1))/MAX_CONTACTS) + 1;
                }   
            }

            var found_collision = false;
            for (var i = 0u; i < count; i++) {
                if contacts[j].b == collisions[i] {
                    found_collision = true;
                }
            }
            if !found_collision && (contacts[j].bond_type < 0 || settings.bonds == 0) {
                // delete
                contacts[j].b = $ CSO {  0 } 
                                $ NCS { -1 };
            } else if !found_collision && contacts[j].bond_type > 0 && settings.bonds <= 1 {
                contacts[j].s_force.x = 0.0;
            }
        }   

        // create new contacts
        for (var i = 0u; i < count; i++) { 
            var existing_index = -1;  
            var empty_index = -1;  
            for (var j = id * MAX_CONTACTS; j < (id + 1u) * MAX_CONTACTS; j++) {
                var b = contacts[j].b;
                $ CSO {
                    if b < 0 {
                            b = i32(u32((-b - 1))/MAX_CONTACTS) + 1;
                    }
                }
                if b == collisions[i] {
                    existing_index = i32(j);
                    break; 
                } else if contacts[j].b == $ CSO { 0 } $ NCS { -1 } && empty_index == -1 {
                    empty_index = i32(j);
                }
            }

            if existing_index == -1 && empty_index == -1 {
                continue;
            } else if existing_index == -1 { //  initialize completely new contact
                let b = collisions[i];
                contacts[empty_index].b = b;
                contacts[empty_index].forces  = vec2(0.0);
                contacts[empty_index].moment  = 0.0;
                contacts[empty_index].s_force = vec2(0.0);
                contacts[empty_index].theta_b = 0.0;        
                contacts[empty_index].bond_angle = 0.0;        
            }
        } 
    }  
    if settings.update_contacts == 0 { 
        //compute forces at each contact
        for (var i = id * MAX_CONTACTS; i < (id + 1u) * MAX_CONTACTS; i++) {
            
            if contacts[i].b == $ CSO { 0 } $ NCS { -1 } $ DETERMINISTIC { || contacts[i].b < i32(id) $ CSO { + 1 } } { continue; }
            let a = i32(id); 
            let b = contacts[i].b $ CSO { - 1 } ; 
            var bonded = contacts[i].bond_type;
            $ NON-DETERMINISTIC {
                if bonded >= 0 && bonds[bonded].index < 0 {
                    contacts[i].bond_type = bonds[bonded].index;
                    bonded = bonds[bonded].index;
                }
            }
            var forces = vec3(0.0, 0.0, 0.0);
            if bonded < 0 || settings.bonds == 0 {
                forces = linear_model(a, b, i, -1);
            } else {
                if settings.bonds == 1 {
                    forces = normal_bonds(a, b, i, bonded);
                } else if settings.bonds == 2 {
                    forces = linear_contact_bonds(a, b, i, bonded);
                } else if settings.bonds == 3 {
                    forces = linear_parallel_bonds(a, b, i, bonded);
                }
            }

            $ DETERMINISTIC     { contacts[i].forces = forces.xy;
                                  contacts[i].moment = forces.z; }
        }

    }
    $ NON-DETERMINISTIC { store_forces(id, net_force, net_moment); }
    if id == 0u {
        coll_cont[2] += 1;
        coll_cont[3] = 0;
    }
}

fn distance(a: i32, b: i32) -> f32 {
    $ F64 {
        return  f32(length(vec2(f64(positions[a].x) - f64(positions[b].x), f64(positions[a].y) - f64(positions[b].y))) - (f64(radii[a]) + f64(radii[b])));
    }
    $ F32 {
        return  length(positions[a] - positions[b]) - (radii[a] + radii[b]);
    }
}

fn distance2(a: i32, b: i32, contact_index: u32) -> f32 {
    var length = radii[a] + radii[b];
    if contacts[contact_index].bond_type >= 0 {
        length = contacts[contact_index].bond_length;
    }
    $ F64 {
        return f32(length(vec2(f64(positions[a].x) - f64(positions[b].x), f64(positions[a].y) - f64(positions[b].y))) - f64(length));
    }
    $ F32 {
        return length(positions[a] - positions[b]) - length;
    }
}

fn linear_parallel_bonds(a: i32, b: i32, i: u32, bonded: i32) -> vec3<f32> { 
    let normal_stiffness = settings.bond_normal_stiffness;
    let shear_stiffness = settings.bond_shear_stiffness;

    let lambda = 0.5; // make parameter 
    let R = lambda * min(radii[a], radii[b]); // bond radius
    let t = 1.0; // "beam" thickness
    let A = 2.0 * R * t; // cross-sectional area
    let I = 2.0 / 3.0 * R * R * R; // moment of inertia
    let normal_displacement = distance2(a, b, i);
    var normal_force = -normal_stiffness * normal_displacement * A;// *1.30812284096;

    let normal = normalize(positions[a] - positions[b]);
    let tangent = vec2(-normal.y, normal.x);

    let del_pos_a = del_pos[a];
    let del_pos_b = del_pos[b];
    let del_rot_a = del_rot[a] * (radii[a]);
    let del_rot_b = del_rot[b] * (radii[b]);

    let rel_trans = del_pos_b - del_pos_a;
    let rel_rot = del_rot_b + del_rot_a;

    let rel_tangent = dot(rel_trans, tangent) + rel_rot;
    contacts[i].s_force.y += rel_tangent * shear_stiffness * A;

    contacts[i].theta_b += del_rot[a] - del_rot[b];

    var moment = -contacts[i].s_force.y;
    contacts[i].bond_angle =  -normal_stiffness * I * contacts[i].theta_b;
    var force = (normal * normal_force + tangent * contacts[i].s_force.y);

    // TEAR BOND
    var normal_limit = settings.bond_normal_strength;
    var shear_limit = settings.bond_shear_strength;
    let normal_and_moment = normal_force;
    if settings.bonds_tear == 1 && (normal_force / A - settings.moment_contribution_factor * (abs(-normal_stiffness * I * contacts[i].theta_b) * R) / I < -normal_limit || abs(contacts[i].s_force.y) / A > shear_limit) {
        let break_code = -10 + i32(sign(normal_displacement));
        $ DETERMINISTIC     { contacts[i].bond_type = break_code; }
        $ NON_DETERMINISTIC { bonds[contacts[i].bond_type] = break_code; }
        normal_force = 0.0;
        contacts[i].s_force.y = 0.0;
        force = vec2(0.0, 0.0);
        moment = 0.0;
        data[u32(a) * DATA_SIZE + 7u] += 1.0;
    }
    
    data[u32(a) * DATA_SIZE   ] += -normal_force;
    data[u32(a) * DATA_SIZE + 1u] += contacts[i].s_force.y; 
    data[u32(a) * DATA_SIZE + 2u] += moment * radii[a] + contacts[i].bond_angle;
    
    data[u32(b) * DATA_SIZE   ] += -normal_force;
    data[u32(b) * DATA_SIZE + 1u] += contacts[i].s_force.y; 
    data[u32(b) * DATA_SIZE + 2u] += moment * radii[b] + contacts[i].bond_angle;

    return  vec3(force, moment) + linear_model(a, b, i, bonded); 
}

fn linear_model(a: i32, b: i32, i: u32, bonded: i32) -> vec3<f32> { //unbonded

    var normal_displacement = min(0.0, distance2(a, b, i));
    if bonded == -1 {
        normal_displacement = min(0.0, distance(a, b));
    }    
    if normal_displacement == 0.0 {  
        return vec3(0.0, 0.0, 0.0);
    }

    let normal_stiffness = 1.0 / (1.0 / materials[(material_pointers[a])].normal_stiffness + 1.0 / materials[(material_pointers[b])].normal_stiffness);
    let shear_stiffness = 1.0 / (1.0 / materials[(material_pointers[a])].shear_stiffness + 1.0 / materials[(material_pointers[b])].shear_stiffness);

    var normal_force = -normal_displacement * normal_stiffness;
    let normal = normalize(positions[a] - positions[b]);
    let tangent = vec2(-normal.y, normal.x);

    let del_pos_a = del_pos[a];
    let del_pos_b = del_pos[b];
    let del_rot_a = del_rot[a] * (radii[a]);
    let del_rot_b = del_rot[b] * (radii[b]);

    let rel_trans = del_pos_b - del_pos_a;
    let rel_rot = del_rot_b + del_rot_a;

    let rel_tangent = dot(rel_trans, tangent) + rel_rot;

    var friction_limit = abs(normal_force) * settings.friction_coefficient;
    contacts[i].s_force.x = clamp(contacts[i].s_force.x + rel_tangent * shear_stiffness, -friction_limit, friction_limit);
    var moment = -contacts[i].s_force.x;
    let force = (normal * normal_force + tangent * contacts[i].s_force.x);
    data[u32(a) * DATA_SIZE   ] += -normal_force;
    data[u32(a) * DATA_SIZE + 1u] += contacts[i].s_force.x; 
    data[u32(a) * DATA_SIZE + 2u] += moment * radii[a];
    
    data[u32(b) * DATA_SIZE   ] += -normal_force;
    data[u32(b) * DATA_SIZE + 1u] += contacts[i].s_force.x; 
    data[u32(b) * DATA_SIZE + 2u] += moment * radii[b];
    // data[u32(a) * DATA_SIZE + 3u] = normal.;


    return vec3(force, moment);
}

fn linear_contact_bonds(a: i32, b: i32, i: u32, bonded: i32) -> vec3<f32> { //unbonded
    let normal_displacement = distance2(a, b, i);

    let normal_stiffness = 1.0 / (1.0 / settings.bond_normal_stiffness + 1.0 / settings.bond_normal_stiffness);
    let shear_stiffness = 1.0 / (1.0 / settings.bond_shear_stiffness + 1.0 / settings.bond_shear_stiffness);

    var normal_force = -normal_displacement * normal_stiffness;
    let normal = normalize(positions[a] - positions[b]);
    let tangent = vec2(-normal.y, normal.x);

    let del_pos_a = del_pos[a];
    let del_pos_b = del_pos[b];
    let del_rot_a = del_rot[a] * (radii[a]);
    let del_rot_b = del_rot[b] * (radii[b]);

    let rel_trans = del_pos_b - del_pos_a;
    let rel_rot = del_rot_b + del_rot_a;

    let rel_tangent = dot(rel_trans, tangent) + rel_rot;

    contacts[i].s_force.x += rel_tangent * shear_stiffness;
    var force = settings.contact_damping * (normal * normal_force + tangent * contacts[i].s_force.x);
    var moment = -contacts[i].s_force.x; 

    // TEAR BOND
    var shear_limit = settings.bond_shear_strength;
    var normal_limit = settings.bond_normal_strength;
    if settings.bonds_tear == 1 && (normal_force < -normal_limit || abs(contacts[i].s_force.x) > shear_limit) {
        let break_code = -10 + i32(sign(normal_displacement));
        $ DETERMINISTIC     { contacts[i].bond_type = break_code; }
        $ NON_DETERMINISTIC { bonds[contacts[i].bond_type] = break_code; }
        normal_force = 0.0;
        force = vec2(0.0, 0.0);
        moment = 0.0;
    }

    //data[u32(a) * DATA_SIZE   ] = -normal_force;
    //data[u32(a) * DATA_SIZE + 1u] = contacts[i].s_force.x; 
    //data[u32(a) * DATA_SIZE + 2u] = moment;

    return vec3(force, moment);
}

fn normal_bonds(a: i32, b: i32, i: u32, bonded: i32) -> vec3<f32> { //unbonded
    let displacement: f32 = distance2(i32(a), b, i);
    var force = vec2(0.0, 0.0);
    let spring_force = settings.bond_normal_stiffness * displacement;
    force += spring_force * normalize(positions[b] - positions[a]);// * settings.bond_damping;

    if settings.bonds_tear == 1 && spring_force >= settings.bond_normal_strength {
        let break_code = -10 + i32(sign(displacement));
        $ DETERMINISTIC     { contacts[i].bond_type = break_code; }
        $ NON_DETERMINISTIC { bonds[contacts[i].bond_type] = break_code; }
        force = vec2(0.0, 0.0);
    }
    return linear_model(a, b, i, bonded) + vec3(force, 0.0);
}

fn store_forces(id: u32, net_force: vec2<f32>, net_moment: f32) {
    // Apply sum of forces and gravity to velocities
    let mat_id = material_pointers[id];
    let density = materials[mat_id].density;
    let mass = density * PI * radii[id] * radii[id];
    let rot_inertia = 0.5 * mass * radii[id] * radii[id];
    var force = net_force + vec2(forces[id].x, forces[id].y);
    var moment = net_moment + forces[id].rot;
    
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

