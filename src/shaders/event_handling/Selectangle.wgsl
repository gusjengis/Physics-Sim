# physics_structs;

struct Input {
    x: i32,
    y: i32,
    w: i32,
    h: i32
}

@group(0) @binding(0) var<uniform> input: Input;
@group(1) @binding(0) var<storage, read_write> selections: array<i32>;
@group(2) @binding(0) var tex_sampler: texture_2d<f32>;
@group(3) @binding(0) var<storage, read_write> click_info: array<i32>;
@group(4) @binding(0) var<storage, read_write> velocities: array<vec2<f32>>;
@group(4) @binding(1) var<storage, read_write> velocities_buf: array<vec2<f32>>;
@group(4) @binding(2) var<storage, read_write> rot: array<f32>;
@group(4) @binding(3) var<storage, read_write> rot_vel: array<f32>;
@group(4) @binding(4) var<storage, read_write> rot_vel_buf: array<f32>;
@group(4) @binding(5) var<storage, read_write> acc: array<vec3<f32>>;
@group(4) @binding(6) var<storage, read_write> fixity: array<Particle_Settings>;
@group(4) @binding(7) var<storage, read_write> forces: array<Forces>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id: u32 = global_id.x;
        if click_info[0] == 0 && id < u32(abs(input.w*input.h)) {
            let x_sign = i32(sign(f32(input.w)));
            let y_sign = i32(sign(f32(input.h)));
            let pixel_coord = vec2(i32(input.x) + x_sign * i32(i32(id) % abs(input.w)), i32(input.y) + y_sign * i32(i32(id) / abs(input.w)));
            let pixel_color = textureLoad(tex_sampler, pixel_coord, 0);
            let clicked_particle = u32((pixel_color.r)*255.0*255.0*255.0) + u32((pixel_color.g)*255.0*255.0) + u32((pixel_color.b)*255.0) - 1u;
            // if fixity[clicked_particle].x_vel == 1 && selections[clicked_particle] == 0 || selections[clicked_particle] == 2 {
            //     selections[clicked_particle] = 2;
            // } else {
            //     selections[clicked_particle] = 1;
            // }

            if  fixity[clicked_particle].x_vel   == 1 ||
                fixity[clicked_particle].y_vel   == 1 ||
                fixity[clicked_particle].rot_vel == 1 
                { selections[clicked_particle] = 2; }
            else 
                { selections[clicked_particle] = 1; }
        }
}
