# physics_structs;

@group(0) @binding(0) var<storage, read_write> selections: array<i32>;
@group(1) @binding(0) var<storage, read_write> velocities: array<vec2<f32>>;
@group(1) @binding(1) var<storage, read_write> velocities_buf: array<vec2<f32>>;
@group(1) @binding(2) var<storage, read_write> rot: array<f32>;
@group(1) @binding(3) var<storage, read_write> rot_vel: array<f32>;
@group(1) @binding(4) var<storage, read_write> rot_vel_buf: array<f32>;
@group(1) @binding(5) var<storage, read_write> acc: array<vec3<f32>>;
@group(1) @binding(6) var<storage, read_write> fixity: array<Particle_Settings>;
@group(1) @binding(7) var<storage, read_write> forces: array<Forces>;
@group(2) @binding(0) var<storage, read_write> click_info: array<i32>;


const deltaTime: f32 = 0.0000390625;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id: u32 = global_id.x;
    if selections[id] == 1 {
        selections[id] = 2; 
        rot_vel[id] = 0.0;
        fixity[id] = Particle_Settings(
            1,
            1,
            1,
            1,
            1,
            1,
        );
        velocities[id] = vec2(0.0, 0.0);
    }
}
