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

struct GridInfo {
    cell_size: f32,
    cell_cap: i32,
    w: i32,
    h: i32,
}

@group(0) @binding(0) var<storage, read_write> positions: array<vec2<f32>>;
@group(0) @binding(1) var<storage, read_write> radii: array<f32>;
@group(1) @binding(4) var<storage, read_write> grid: array<i32>;
@group(1) @binding(5) var<storage, read_write> grid_info_buffer: array<GridInfo>;
@group(2) @binding(0) var<uniform> settings: Settings;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = i32(global_id.x);
    let grid_info = grid_info_buffer[0];
    
    let base_index = id * grid_info.cell_cap;
    if u32(base_index) >= arrayLength(&grid) {
        return;
    }

    let base_x = -grid_info.cell_size*f32(grid_info.w)*0.5;
    let base_y =  grid_info.cell_size*f32(grid_info.h)*0.5;

    let cell_index  = vec2(
        id % grid_info.w,
        id / grid_info.w
    );

    let left   = base_x + grid_info.cell_size * f32(cell_index.x    );
    let right  = base_x + grid_info.cell_size * f32(cell_index.x + 1);
    let top    = base_y - grid_info.cell_size * f32(cell_index.y    );
    let bottom = base_y - grid_info.cell_size * f32(cell_index.y + 1);

    var index = 0;
    
    for(var i = 0u; i<arrayLength(&radii); i++){
        let closest_point = vec2(
            clamp(positions[i].x, left,   right),
            clamp(positions[i].y, bottom, top  )
        );
        if length(closest_point - positions[i]) < radii[i] {
            grid[base_index + index + 1] = i32(i);
            index += 1;
        }
    }

    grid[base_index] = index;
}