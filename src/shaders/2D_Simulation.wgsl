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

struct Contact {
    a: i32,
    b: i32,
    tangent_force: f32,
    bonded: i32
};

struct Bond {
    index: i32,
    angle: f32,
    length: f32
};

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
    friction: i32,
    friction_coefficient: f32,
    rotation: i32,
    linear_contact_bonds: i32,
    gravity_acc: f32,
    bond_tensile_strength: f32,
    bonds_tear: i32,
    bond_force_limit: f32,
    contact_damping: f32,
    bond_damping: f32,
    drag: f32,
    bond_shear_lim: f32,
    verlet: i32,
    dT: f32,
    bond_shear_strength: f32,
    bond_rotational_stiffness: f32,
    bond_rotational_strength: f32
}

struct Material {
    red: f32,
    green: f32,
    blue: f32,
    density: f32,
    normal_stiffness: f32,
    shear_stiffness: f32,
}

struct GridInfo {
    cell_size: f32,
    cell_cap: i32,
    w: i32,
    h: i32,
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
@group(2) @binding(0) var<storage, read_write> bonds: array<Bond>;
// @group(2) @binding(1) var<storage, read_write> bond_info: array<vec2<i32>>;
@group(2) @binding(1) var<storage, read_write> contacts: array<Contact>;
@group(2) @binding(4) var<storage, read_write> grid: array<i32>;
@group(2) @binding(5) var<storage, read_write> grid_info_buffer: array<GridInfo>;
// @group(2) @binding(3) var<storage, read_write> contact_pointers: array<i32>;
@group(2) @binding(3) var<storage, read_write> material_pointers: array<i32>;
@group(3) @binding(0) var<uniform> settings: Settings;
@group(4) @binding(0) var<storage, read_write> materials: array<Material>; 
@group(5) @binding(0) var<storage, read_write> data: array<f32>; 


// const settings.dT: f32 = 0.000005;//0.0000390625;
const PI = 3.141592653589793238;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id: u32 = global_id.x;
    let grid_info = grid_info_buffer[0];

    
    data[id*4u   ] = 0.0;
    data[id*4u+1u] = 0.0;
    data[id*4u+2u] = 0.0;
    data[id*4u+3u] = 0.0;

    if radii[id] == 0.0 { return; }
    let mat_id = material_pointers[id];
    // let contact_damping: f32 = 0.2; // contact_damping factor, can be adjusted

    var net_force = vec2(0.0, 0.0);
    var net_moment = 0.0;
    // var stress_tensor = vec3(0.0, 0.0, 0.0);

    // OG O(n^2) Collisions
    let max_contacts = 14u;
    if settings.collisions == 1 {
        var collisions = array<i32, 14u>();
        var count = 0u;
 
        // make a list of particles that we're colliding with

        // let base_x = -grid_info.cell_size * f32(grid_info.w) * 0.5;
        // let base_y = grid_info.cell_size * f32(grid_info.h) * 0.5;

        // let particle_left = positions[id].x - radii[id];
        // let particle_right = positions[id].x + radii[id];
        // let particle_bottom = positions[id].y - radii[id];
        // let particle_top = positions[id].y + radii[id];

        // let min_cell_x = max(i32((particle_left - base_x) / grid_info.cell_size), 0);
        // let max_cell_x = min(i32((particle_right - base_x) / grid_info.cell_size), grid_info.w - 1);
        // let min_cell_y = max(i32((base_y - particle_top) / grid_info.cell_size), 0);
        // let max_cell_y = min(i32((base_y - particle_bottom) / grid_info.cell_size), grid_info.h - 1);

        // for (var cell_y = min_cell_y; cell_y <= max_cell_y; cell_y++) {
        //     for (var cell_x = min_cell_x; cell_x <= max_cell_x; cell_x++) {
        //         let cell_id = cell_y * grid_info.w + cell_x;
        //         let base_index = cell_id * grid_info.cell_cap;
        //         var neighbors = grid[base_index];

        //         if neighbors > 1 {
        //             for (var i = 2; i < neighbors + 2; i++) {
        //                 let b = grid[base_index + i];
        //                 if u32(b) != id {
        //                     if length(positions[b] - positions[id]) < (radii[b] + radii[id]) {
        //                         collisions[count] = b;
        //                         count += 1u;
        //                         if count == max_contacts {
        //                             break;
        //                         }
        //                     }
        //                 }
        //             }
        //         }
        //         if count == max_contacts {
        //             break;
        //         }
        //     }
        //     if count == max_contacts {
        //         break;
        //     }
        // }

        for(var i = 0u; i<arrayLength(&radii); i++){
            if i != id {
                if length(positions[i] - positions[id]) < (radii[i] + radii[id]){
                    collisions[count] = i32(i);
                    count += 1u;
                    if count == max_contacts {
                        break;
                    }
                } 
            }
        }

        // delete contacts that don't exist
        for(var j = id*max_contacts; j<(id+1u)*max_contacts; j++){
            if contacts[j].b == -1 {
                continue;
            }
            var found_collision = false;
            var other_particle = -1;
            for(var i = 0u; i<count; i++){
                if contacts[j].b == collisions[i] {
                    found_collision = true;
                    other_particle = (contacts[j].b);
                }
            }
            if (!found_collision && contacts[j].bonded <= 0) {
                // delete
                contacts[j].a = -1;
                contacts[j].b = -1;
                // for(var k = u32(other_particle)*max_contacts; k<(u32(other_particle)+1u)*max_contacts; k++) {
                //     if contact_pointers[k] == i32(j) {
                //         contact_pointers[k] = -1;
                //         break;
                //     }
                // }
            } else if (!found_collision && contacts[j].bonded > 0 && settings.bonds <= 1) {
                contacts[j].tangent_force = 0.0;
            }
        }   

        // create new contacts
        for(var i = 0u; i<count; i++){
            var existing_index = -1;
            var empty_index = -1;
            for(var j = id*max_contacts; j<(id+1u)*max_contacts; j++){
                if contacts[j].b == collisions[i] {
                    existing_index = i32(j);
                    break;
                } else if contacts[j].b == -1 {
                    empty_index = i32(j);
                }
                
            }
            
            if existing_index == -1 && empty_index == -1 {
                continue;
            } else if existing_index == -1 { // initialize completely new contact
                let b = collisions[i];
                // for(var j = 0u; j<6u; j++){
                //     // if bonded_particles[j] == contacts[i].b {
                //     //     contacts[empty_index].bonded = 1;
                //     //     break;
                //     // }
                // }
                contacts[empty_index].a = i32(id);
                contacts[empty_index].b = b;
                contacts[empty_index].tangent_force = 0.0;
            }

        }
    }
    for(var i = id*max_contacts; i<(id+1u)*max_contacts; i++){
        // bonds[contacts[i].bonded].bond_type;
        // make function
        
        if contacts[i].b == -1 { continue; }
        let a = contacts[i].a;
        let b = contacts[i].b;
        var bonded = contacts[i].bonded;
        if bonded >= 0 && bonds[bonded].index < 0 {
            contacts[i].bonded = -1;
            bonded = -1;
        }
        var forces = vec3(0.0, 0.0, 0.0);
        if bonded < 0 || settings.bonds == 0 {
            forces = linear_model(a, b, i);
        } else{
            if settings.bonds == 1 {
                forces = normal_bonds(a, b, i, bonded);
            } else if settings.bonds == 2 {
                forces = linear_contact_bonds(a, b, i, bonded, id);
            } else if settings.bonds == 3 {
                forces = linear_parallel_bonds(a, b, i, bonded, id);
            }
        }
        
        net_force  += forces.xy;
        net_moment += forces.z;
    }

    // data[id*4] = stress_tensor.x;
    // data[id*4+1] = stress_tensor.y;
    // data[id*4+2] = stress_tensor.z;
    
    let wall_forces = walls(id);
    net_force -= wall_forces.xy;
    net_moment -= wall_forces.z;

    store_forces(id, mat_id, net_force, net_moment);
    
}

fn distance(a: i32, b: i32) -> f32 {
    return  length(positions[a] - positions[b]) - (radii[a] + radii[b]);
}

fn linear_parallel_bonds(a: i32, b: i32, i: u32, bonded: i32, id: u32) -> vec3<f32> { //unbonded
    // let normal_displacement = (length((positions[a] - positions[b])) - (radii[a] + radii[b]));
    let normal_displacement = f32(length(vec2(f64(positions[a].x) - f64(positions[b].x), f64(positions[a].y) - f64(positions[b].y))) - (f64(radii[a]) + f64(radii[b])));
    let angular_displacement = rot[b] - rot[a];
    // let normal_stiffness = 1.0/(1.0/materials[(material_pointers[a])].normal_stiffness + 1.0/materials[(material_pointers[b])].normal_stiffness);
    // let shear_stiffness  = 1.0/(1.0/materials[(material_pointers[a])].shear_stiffness  + 1.0/materials[(material_pointers[b])].shear_stiffness);
    let normal_stiffness      = 1.0/(1.0/settings.bond_tensile_strength     + 1.0/settings.bond_tensile_strength    );
    let shear_stiffness       = 1.0/(1.0/settings.bond_shear_strength       + 1.0/settings.bond_shear_strength      );
    let rotational_stiffness  = 1.0/(1.0/settings.bond_rotational_stiffness + 1.0/settings.bond_rotational_stiffness);

    let normal_force = -normal_displacement*normal_stiffness;
    let normal = normalize(positions[a] - positions[b]); 
    let tangent = vec2(-normal.y, normal.x);

    let del_pos_a = del_pos[a];
    let del_pos_b = del_pos[b];
    let del_rot_a = del_rot[a]*(radii[a]);
    let del_rot_b = del_rot[b]*(radii[b]);

    let rel_trans = del_pos_b - del_pos_a;  
    let rel_rot = del_rot_b + del_rot_a;

    let rel_tangent = dot(rel_trans, tangent) + rel_rot;

    contacts[i].tangent_force += rel_tangent*shear_stiffness;
    let angular_force = angular_displacement*rotational_stiffness;
    let force  = settings.contact_damping * (normal*normal_force + tangent*contacts[i].tangent_force);
    let moment = -(radii[a])*contacts[i].tangent_force + angular_force;
    data[u32(id)*4u   ] = normal_force;
    data[u32(id)*4u+1u] = contacts[i].tangent_force;
    data[u32(id)*4u+2u] = moment;
    data[u32(id)*4u+3u] = normal_displacement;//atan2(normal.y, normal.x);

    // TEAR BOND
    var shear_limit  = settings.bond_shear_lim;
    var normal_limit = settings.bond_force_limit;
    var rotational_strength = settings.bond_rotational_strength;
    if settings.bonds_tear == 1 && (normal_force < -normal_limit || abs(contacts[i].tangent_force) > shear_limit || abs(angular_force) > rotational_strength){
        bonds[bonded].index = -bonds[bonded].index;
        contacts[i].bonded = -1;
    }

    return vec3(force, moment);
}

fn linear_contact_bonds(a: i32, b: i32, i: u32, bonded: i32, id: u32) -> vec3<f32> { //unbonded
    // let normal_displacement = (length((positions[a] - positions[b])) - (radii[a] + radii[b]));
    let normal_displacement = f32(length(vec2(f64(positions[a].x) - f64(positions[b].x), f64(positions[a].y) - f64(positions[b].y))) - (f64(radii[a]) + f64(radii[b])));

    // let normal_stiffness = 1.0/(1.0/materials[(material_pointers[a])].normal_stiffness + 1.0/materials[(material_pointers[b])].normal_stiffness);
    // let shear_stiffness  = 1.0/(1.0/materials[(material_pointers[a])].shear_stiffness  + 1.0/materials[(material_pointers[b])].shear_stiffness);
    let normal_stiffness = 1.0/(1.0/settings.bond_tensile_strength + 1.0/settings.bond_tensile_strength);
    let shear_stiffness  = 1.0/(1.0/settings.bond_shear_strength   + 1.0/settings.bond_shear_strength);

    let normal_force = -normal_displacement*normal_stiffness;
    let normal = normalize(positions[a] - positions[b]); 
    let tangent = vec2(-normal.y, normal.x);

    let del_pos_a = del_pos[a];
    let del_pos_b = del_pos[b];
    let del_rot_a = del_rot[a]*(radii[a]);
    let del_rot_b = del_rot[b]*(radii[b]);

    let rel_trans = del_pos_b - del_pos_a;  
    let rel_rot = del_rot_b + del_rot_a;

    let rel_tangent = dot(rel_trans, tangent) + rel_rot;

    contacts[i].tangent_force += rel_tangent*shear_stiffness;
    let force  = settings.contact_damping * (normal*normal_force + tangent*contacts[i].tangent_force);
    let moment = -(radii[a])*contacts[i].tangent_force;
    data[u32(id)*4u   ] = normal_force;
    data[u32(id)*4u+1u] = contacts[i].tangent_force;
    data[u32(id)*4u+2u] = moment;
    data[u32(id)*4u+3u] = normal_displacement;//atan2(normal.y, normal.x);

    // TEAR BOND
    var shear_limit  = settings.bond_shear_lim;
    var normal_limit = settings.bond_force_limit;
    if settings.bonds_tear == 1 && (normal_force < -normal_limit || abs(contacts[i].tangent_force) > shear_limit){
        bonds[bonded].index = -bonds[bonded].index;
        contacts[i].bonded = -1;
    }

    return vec3(force, moment);
}

fn normal_bonds(a: i32, b: i32, i: u32, bonded: i32) -> vec3<f32> { //unbonded
    let displacement: f32 = -distance(i32(a), b);
    var force = vec2(0.0, 0.0);
    let spring_force = settings.bond_tensile_strength * displacement;// * abs(displacement);
    force -= spring_force * normalize(positions[b] - positions[a]) * settings.bond_damping;

    if settings.bonds_tear == 1 && spring_force < -settings.bond_force_limit {
        bonds[bonded].index = -bonds[bonded].index;
        contacts[i].bonded = -1;
    }
    return linear_model(a, b, i) + vec3(force, 0.0);
}

fn linear_model(a: i32, b: i32, i: u32) -> vec3<f32> { //unbonded
    
    let normal_displacement = min(0.0, f32(length(vec2(f64(positions[a].x) - f64(positions[b].x), f64(positions[a].y) - f64(positions[b].y))) - (f64(radii[a]) + f64(radii[b]))));
    if normal_displacement == 0.0 {
        return vec3(0.0, 0.0, 0.0);
    }
    
    let normal_stiffness = 1.0/(1.0/materials[(material_pointers[a])].normal_stiffness + 1.0/materials[(material_pointers[b])].normal_stiffness);
    let shear_stiffness  = 1.0/(1.0/materials[(material_pointers[a])].shear_stiffness  + 1.0/materials[(material_pointers[b])].shear_stiffness);

    var normal_force = -normal_displacement*normal_stiffness;
    let normal = normalize(positions[a] - positions[b]); 
    let tangent = vec2(-normal.y, normal.x);

    let del_pos_a = del_pos[a];
    let del_pos_b = del_pos[b];
    let del_rot_a = del_rot[a]*(radii[a]);
    let del_rot_b = del_rot[b]*(radii[b]);
    
    let rel_trans = del_pos_b - del_pos_a;  
    let rel_rot = del_rot_b + del_rot_a;
    
    let rel_tangent = dot(rel_trans, tangent) + rel_rot;
    
    var friction_limit = abs(normal_force)*settings.friction_coefficient;
    contacts[i].tangent_force = clamp(contacts[i].tangent_force + rel_tangent*shear_stiffness, -friction_limit, friction_limit);
    let force  = settings.contact_damping * (normal*normal_force + tangent*contacts[i].tangent_force);
    let moment = -(radii[a])*contacts[i].tangent_force;
    // data[u32(a)*4u+3u] = contacts[i].tangent_force;

    return vec3(force, moment);
}

fn store_forces(id: u32, mat_id: i32, net_force: vec2<f32>, net_moment: f32) {
    // Apply sum of forces and gravity to velocities
    let density = materials[mat_id].density;
    let mass = density * PI * radii[id] * radii[id];
    let rot_inertia = 0.5*mass*radii[id]*radii[id];
    // natural accelerations
    accelerations[id] = net_force/mass;
    rot_acc[id] = net_moment/rot_inertia;

    // data[id*4u]    = net_force.x;
    // data[id*4u+1u] = net_force.y;
    // data[id*4u+2u] = net_moment;
    // gravity
    if settings.gravity == 1 && settings.planet_mode == 1  {
        let delta = (vec2(0.0, 0.0) - positions[id]);
        accelerations[id] += delta/length(delta) * 9.81 * settings.gravity_acc;
    } else if settings.gravity == 1 {
        let gravity = 9.81 * settings.gravity_acc;
        accelerations[id] += vec2(0.0, -gravity);
    }

    if fixity[id].x_vel   == 0 { velocities[id].x += 0.5 * accelerations[id].x * settings.dT; }
    if fixity[id].y_vel   == 0 { velocities[id].y += 0.5 * accelerations[id].y * settings.dT; }
    if fixity[id].rot_vel == 0 { rot_vel[id]      += 0.5 * rot_acc[id]         * settings.dT; }
    // artifical accelerations
    velocities[id] += 0.5 * vec2(forces[id].x, forces[id].y) * settings.dT;
    rot_vel[id] += forces[id].rot*settings.dT;
}

fn walls(id: u32) -> vec3<f32> {
    // BS Walls
    let pos = positions[id];
    let rad = radii[id];
    let elasticity = 0.5;
    let anti_stick_coating = 0.01;
    let yH = settings.vert_bound;
    let xW = settings.hor_bound;
    var forces = vec3(0.0, 0.0, 0.0);

    if settings.round_bounds == 0 {

        if pos.x+rad > xW {
            velocities[id] = vec2(-velocities[id].x, velocities[id].y)*elasticity;
            rot_vel[id] = rot_vel[id]*0.9;
            positions[id] = vec2(xW-rad, pos.y);
        } else if pos.x-rad < -xW {
            velocities[id] = vec2(-velocities[id].x, velocities[id].y)*elasticity;
            rot_vel[id] = rot_vel[id]*0.9;
            positions[id] = vec2(-xW+rad, pos.y);
        }
        if pos.y+rad > yH {
            velocities[id] = vec2(velocities[id].x, -velocities[id].y)*elasticity;
            rot_vel[id] = rot_vel[id]*0.9;
            positions[id] = vec2(pos.x, yH-rad - anti_stick_coating);
        } else if pos.y-rad < -yH {
            velocities[id] = vec2(velocities[id].x, -velocities[id].y)*elasticity;
            rot_vel[id] = rot_vel[id]*0.9;
            positions[id] = vec2(pos.x, -yH+rad);
        }

    } else if length(pos) + rad > settings.bound_radius { // circular bounds
        let norm_pos = normalize(pos);
        let vel = velocities[id];
        positions[id] = norm_pos * (settings.bound_radius - rad);
        let tangent = vec2(-norm_pos.y, norm_pos.x);
        let comp_v_p = dot(vel, norm_pos) * norm_pos;
        let comp_v_p_t = dot(vel, tangent) * tangent;
        velocities[id] -= comp_v_p * 1.6;// + friction;

        // Doesn't work
        // let del_pos = del_pos[id];
        // let del_rot = del_rot[id]*(radii[id]);
        // let rel_trans = dot(del_pos, tangent) * tangent;  
        // let rel_rot = del_rot;
        // let rel_tangent = dot(rel_trans, tangent) + rel_rot;

        // let friction = rel_tangent*tangent * settings.wall_friction;
        // let moment = rel_tangent * settings.wall_friction * radii[id];
        // rot_vel[id] = moment;
        // forces = vec3(friction, moment);
        // data[id*4u + 3u] = moment;
    }
    return forces;
}

fn stress_tensor(id: u32, force: vec2<f32>, delta: vec2<f32>) -> vec3<f32> {
    var tensor = vec3(0.0, 0.0, 0.0);
    // let 
    return tensor;
}