struct GridInfo {
    cell_size: f32,
    cell_cap: i32,
    w: i32,
    h: i32,
}

@group(0) @binding(0) var<storage, read_write> positions: array<vec2<f32>>;
@group(0) @binding(1) var<storage, read_write> radii: array<f32>;
@group(2) @binding(4) var<storage, read_write> grid: array<i32>;
@group(2) @binding(5) var<storage, read_write> grid_info_buffer: array<GridInfo>;
@group(2) @binding(6) var<storage, read_write> coll_cont: array<atomic<i32>>;


@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if coll_cont[0] == 1 {
        let grid_info = grid_info_buffer[0];
        let cell_count = grid_info.w * grid_info.h;
        for(var cell_id = 0; cell_id<cell_count; cell_id++) {
            grid[cell_id*grid_info.cell_cap] = 0;
        }
        let base_x = -grid_info.cell_size * f32(grid_info.w) * 0.5;
        let base_y = grid_info.cell_size * f32(grid_info.h) * 0.5;
        let p_count = i32(arrayLength(&radii));
        for(var id = 0; id<p_count; id++) {

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
                    // if grid[base_index + 1] == 0 { // if not initialized
                    //     grid[base_index] = 0; // part_count = 0
                    //     grid[base_index + 1] = 1; // initialized = true
                    // } 
                    grid[base_index] += 1;
                    grid[base_index + 1 + (grid[base_index])] = id;
                } 
            }
        }
    }
}
//             let base_index = id * grid_info.cell_cap;
//             var index = 0;
//             if grid[base_index + 1] == 1 {
//                 if u32(base_index) >= arrayLength(&grid) {
//                     return;
//                 }

//                 let base_x = -grid_info.cell_size*f32(grid_info.w)*0.5;
//                 let base_y =  grid_info.cell_size*f32(grid_info.h)*0.5;

//                 let cell_index  = vec2(
//                     id % grid_info.w,
//                     id / grid_info.w
//                 );

//                 let left   = base_x + grid_info.cell_size * f32(cell_index.x    );
//                 let right  = base_x + grid_info.cell_size * f32(cell_index.x + 1);
//                 let top    = base_y - grid_info.cell_size * f32(cell_index.y    );
//                 let bottom = base_y - grid_info.cell_size * f32(cell_index.y + 1);

//                 for(var i = 0u; i<arrayLength(&radii); i++){
//                     let closest_point = vec2(
//                         clamp(positions[i].x, left,   right),
//                         clamp(positions[i].y, bottom, top  )
//                     );
//                     if length(closest_point - positions[i]) < radii[i] {
//                         grid[base_index + index + 2] = i32(i);
//                         index += 1;
//                         if index > grid_info.cell_cap - 2 {
//                             break;
//                         }
//                     }
//                 }
//             }

//             grid[base_index]     = index;
//             grid[base_index + 1] = 0;
//         }
//     }    
// }