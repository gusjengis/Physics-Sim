// Grid insertion pass, split out of 2D_LOM.wgsl (AUTOPSY H8).
// storageBarrier() only synchronizes within a workgroup, so with more than
// 256 particles the LOM kernel's clear loop in one workgroup could race the
// insert loop of another, wiping already-inserted neighbor entries. Running
// the insert as its own dispatch puts an implicit barrier between clear and
// insert. Uses the same pipeline layout / bind groups as 2D_LOM.wgsl.

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
@group(2) @binding(4) var<storage, read_write> grid: array<atomic<i32>>;
@group(2) @binding(5) var<uniform> grid_info_buffer: array<GridInfo, 1>;
@group(2) @binding(6) var<storage, read_write> coll_cont: array<i32>;
@group(3) @binding(0) var<uniform> settings: Settings;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    if radii[id] == 0.0 || id >= u32(settings.particles) { return; }
    if coll_cont[0] != 1 { return; }
    let grid_info = grid_info_buffer[0];

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
            let p_count = atomicAdd(&grid[base_index + 0], 1) + 1;
            if p_count < grid_info.cell_cap - 1 {
                atomicStore(&grid[base_index + 1 + p_count], i32(id)); // plain assignment to atomic<i32> rejected by Tint
            }
        }
    }
}
