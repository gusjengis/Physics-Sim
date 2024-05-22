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


@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = i32(global_id.x);
    let grid_info = grid_info_buffer[0];
    
    let base_index = id * grid_info.cell_cap;
    var index = 0;
    if grid[base_index + 1] == 1 {
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

        for(var i = 0u; i<arrayLength(&radii); i++){
            let closest_point = vec2(
                clamp(positions[i].x, left,   right),
                clamp(positions[i].y, bottom, top  )
            );
            if length(closest_point - positions[i]) < radii[i] {
                grid[base_index + index + 2] = i32(i);
                index += 1;
                if index > grid_info.cell_cap - 2 {
                    break;
                }
            }
        }
    }

    grid[base_index]     = index;
    grid[base_index + 1] = 0;
}