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
@group(2) @binding(4) var<storage, read_write> grid: array<atomic<i32>>;
@group(2) @binding(5) var<uniform> grid_info_buffer: array<GridInfo, 1>;
@group(2) @binding(6) var<storage, read_write> coll_cont: array<i32>;
@group(3) @binding(0) var<uniform> settings: Settings;


const PI = 3.141592653589793238;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    let grid_info = grid_info_buffer[0];

    // Broad Phase
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
            // storageBarrier() calls removed: Tint rejects them in non-uniform
            // control flow, and this kernel is dead code (binning moved to
            // 2D_Grid_Insert.wgsl; the dispatch is commented out upstream).
            let p_count = atomicAdd(&grid[base_index + 0], 1) + 1;
            if p_count < grid_info.cell_cap - 1 {
                atomicStore(&grid[base_index + 1 + p_count], i32(id));
            }
        }
    }
}
