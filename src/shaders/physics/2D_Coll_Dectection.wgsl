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
    bond_tangent_force: f32,
    theta_b: f32,
    bonded: i32
};

struct Bond {
    index: i32,
    angle: f32,
    length: f32
};

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

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id: u32 = global_id.x;
    let grid_info = grid_info_buffer[0];

    if radii[id] == 0.0 { return; }


    let max_contacts = 14u;
    if settings.collisions == 1 {
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

        for (var cell_y = min_cell_y; cell_y <= max_cell_y && count < max_contacts; cell_y++) {
            for (var cell_x = min_cell_x; cell_x <= max_cell_x && count < max_contacts; cell_x++) {
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
                                if count == max_contacts {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        // OG O(n^2) Collisions
        // for(var i = 0u; i<arrayLength(&radii); i++){
        //     if i != id {
        //         if length(positions[i] - positions[id]) < (radii[i] + radii[id]){
        //             collisions[count] = i32(i);
        //             count += 1u;
        //             if count == max_contacts {
        //                 break;
        //             }
        //         } 
        //     }
        // }

        // delete contacts that don't exist
        for (var j = id * max_contacts; j < (id + 1u) * max_contacts; j++) {
            if contacts[j].b == -1 {
                continue;
            }
            var found_collision = false;
            var other_particle = -1;
            for (var i = 0u; i < count; i++) {
                if contacts[j].b == collisions[i] {
                    found_collision = true;
                    other_particle = (contacts[j].b);
                }
            }
            if !found_collision && (contacts[j].bonded < 0 || settings.bonds == 0) {
                // delete
                contacts[j].a = -1;
                contacts[j].b = -1;
            } else if !found_collision && contacts[j].bonded > 0 && settings.bonds <= 1 {
                contacts[j].tangent_force = 0.0;
            }
        }   

        // create new contacts
        for (var i = 0u; i < count; i++) {
            var existing_index = -1;
            var empty_index = -1;
            for (var j = id * max_contacts; j < (id + 1u) * max_contacts; j++) {
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
                contacts[empty_index].a = i32(id);
                contacts[empty_index].b = b;
                contacts[empty_index].tangent_force = 0.0;
            }
        }
    }
}
