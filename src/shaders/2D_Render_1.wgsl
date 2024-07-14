struct VertexIn {
    @location(0) position: vec2<f32>,
};

struct Input {
    width: f32, time: f32,
    height: f32, temp: f32,
    xOff: f32, yOff: f32,
    ui_xOff: f32, ui_yOff: f32,
    scale: f32, dark: f32,
    x: i32, y: i32,
    rW: i32, rH: i32,
    pressed: i32,
    timestamp: i32
}

struct Contact {
    a: i32,
    b: i32,
    tangent_force: f32,
    bond_tangent_force: f32,
    theta_b: f32,
    bonded: i32
};
struct Material {
    red: f32,
    green: f32,
    blue: f32,
    density: f32,
    normal_stiffness: f32,
    shear_stiffness: f32,
}

struct Particle_Settings {
    x_vel: i32,
    y_vel: i32,
    rot_vel: i32,
    x_vel_2: i32,
    y_vel_2: i32,
    rot_vel_2: i32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec2<f32>,
    @location(1) color: vec3<f32>,
    @location(2) rot: f32,
    @location(3) rot_vel: f32,
    @location(4) id: u32,
    @location(5) selected: i32,
    @location(6) w_h: vec2<i32>,
    @location(7) pixel: vec2<f32>,
    @location(8) vel: vec2<f32>,
    @location(9) scale: f32,
};

struct Settings {
    circular_particles: i32,
    render_rot: i32,
    color_code_rot: i32,
    colors: i32,
    render_bonds: i32,
    walls: i32,
    w: f32,
    h: f32,
    stiffness: f32,
    render_grid: i32,
    round_bounds: i32,
    wall_radius: f32,
    render_outline: i32,
    use_part_color: i32,
    background_r: f32,
    background_g: f32,
    background_b: f32,
    outline_r: f32,
    outline_g: f32,
    outline_b: f32,
    dim_slow_particles: i32,
    max_brightness_vel: f32,
    crt_res: i32,
    grain: i32,
    grain_strength: f32,
    grain_size: i32,
    sobel: i32,
    invert: i32,
    chrom_ab: i32,
    abb_strength: f32
}

struct Bond {
    index: i32,
    angle: f32,
    length: f32
};

@group(0) @binding(0) var<uniform> input: Input;
@group(1) @binding(0) var<storage, read_write> pos_buf: array<vec2<f32>>;
@group(1) @binding(1) var<storage, read_write> radii_buf: array<f32>;
@group(2) @binding(0) var<storage, read_write> vel: array<vec2<f32>>;
@group(2) @binding(2) var<storage, read_write> rot_buf: array<f32>;
@group(2) @binding(3) var<storage, read_write> rot_vel: array<f32>;
@group(2) @binding(6) var<storage, read_write> fixity: array<Particle_Settings>;
@group(3) @binding(1) var<storage, read_write> contacts: array<Contact>;
@group(3) @binding(3) var<storage, read_write> material_pointers: array<i32>;
@group(4) @binding(0) var<uniform> settings: Settings;
@group(5) @binding(0) var<storage, read_write> materials: array<Material>;
@group(6) @binding(0) var<storage, read_write> selections: array<i32>;
@group(7) @binding(0) var<storage, read_write> click_info: array<i32>;

@vertex
fn vs_main(
    in: VertexIn,
    @builtin(instance_index) instance: u32,
) -> VertexOutput {
        var out: VertexOutput;
    let aspect = input.width/input.height;
    let scale= input.scale;
    let xy = 2.0*scale*vec2(in.position.x / aspect, in.position.y);
    let center = scale*vec2(pos_buf[instance].x / aspect, pos_buf[instance].y);
    let off = vec2((input.xOff + input.ui_xOff)/aspect, (input.yOff + input.ui_yOff))*(scale);
    out.clip_position = vec4(xy*radii_buf[instance] + center + off, 0.0, 1.0);
    out.position = in.position;
    out.rot = rot_buf[instance];
    out.rot_vel = rot_vel[instance];
    out.id = instance;
    out.selected = selections[instance];
    out.w_h = vec2(i32(input.width), i32(input.height));
    out.pixel = out.clip_position.xy;
    out.vel = vel[instance];
    out.scale = scale;

    if settings.colors == 0 {
        out.color = vec3(0.05, 0.05, 0.05);
    } else if settings.colors == 1 && material_pointers[instance] != -1 { 
        out.color = vec3(
            srgb_to_linear(materials[(material_pointers[instance])].red),
            srgb_to_linear(materials[(material_pointers[instance])].green),
            srgb_to_linear(materials[(material_pointers[instance])].blue)
        ); 
    } else if settings.colors == 2 {
        let seed1 = u32(rand(instance, 4294967296.0));
        let seed2 = u32(rand(seed1, 4294967296.0));
        let seed3 = u32(rand(seed2, 4294967296.0));
        out.color = vec3(
            rand(seed1, 1.0),
            rand(seed2, 1.0),
            rand(seed3, 1.0),
        );
    } else if settings.colors == 3 {
        let vel_norm = normalize(out.vel); 
        let angle = atan2(vel_norm.y, vel_norm.x) + PI;
        let r = (1.0 - abs(angle - 1.0000  * PI)/(2.0*PI/3.0)); 
        let g = (max(0.0, 1.0 - abs(angle - 1.6666  * PI)/(2.0*PI/3.0)) +  max(0.0, 1.0 - abs(angle + 0.3333  * PI)/(2.0*PI/3.0))); 
        let b = (max(0.0, 1.0 - abs(angle - 0.3333  * PI)/(2.0*PI/3.0)) +  max(0.0, 1.0 - abs(angle - 2.3333  * PI)/(2.0*PI/3.0)));
        
        out.color = vec3(r, g, b) * 1.0/max(max(r, b), g);
    } else { 
        out.color = vec3(1.0, 1.0, 1.0); 
    }

    if settings.dim_slow_particles == 1 {
        let vel_mag = length(out.vel) / settings.max_brightness_vel;
        let max_brightness = min(1.0, vel_mag*vel_mag*vel_mag);
        out.color *= max_brightness;
    }
    return out;
}

fn rand(seed: u32, max: f32) -> f32{
    //PCG Hash
    var res = seed;
    res = res * 747796405u + 2891336453u;
    res = ((res >> ((res >> 28u) + 4u)) ^ res) * 277803737u;
    res = (res >> 22u) ^ res;

    return max*f32(res)/4294967296.0;
}


const PI = 3.141592653589793238;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // discard corners to make circle
    let len = length(in.position);
    if settings.circular_particles == 1 {
        if len > 0.5 {
            discard;
        }
    }

    var color = vec4(in.color, 1.0);

    let border_width = 0.08;

    // cut out wedge for rotation
    let rot_point = vec2(cos(in.rot), sin(in.rot));
    let rot_dot = dot(rot_point, normalize(in.position));
    if settings.render_rot == 1 && !(len > 0.5-border_width && len < 0.5){
        if rot_dot > 0.9 {
            color = vec4(0.0, 0.0, 0.0, 1.0);
        }
    }
    
    // color code based on direction of rotation
    if settings.color_code_rot == 1 {
        if in.rot_vel > 0.0 {
            color = vec4(0.0, color.g, color.ba);
        } else if in.rot_vel < 0.0 {
            color = vec4(color.r, 0.0, color.ba);
        }
    }

    // bonds
    var border_pixel = false;
    for(var i = in.id*14u; i<(in.id+1u)*14u; i++){
        if contacts[i].a >= 0 {
            let delta = pos_buf[contacts[i].b] - pos_buf[in.id];
            let dir = normalize(delta);
            let tangent = normalize(vec2(-delta.y, delta.x));
            let midpoint = pos_buf[in.id] + delta * 0.5;
            // let tangent = vec2(delta.y, -delta.x);
            let pixel_world_pos = pos_buf[in.id] + radii_buf[in.id] * in.position * 2.0;
            let test_vec = midpoint - pixel_world_pos;
            let side = dot(delta, test_vec);
            border_pixel = border_pixel || dot(test_vec, dir) < border_width * 2.0 * radii_buf[in.id];
            if side < 0.0 {
                discard;
            }
            if settings.render_bonds == 1 {

                let displacement = ((radii_buf[in.id] + radii_buf[contacts[i].b]) - length(delta));
                let scaled_displacement = displacement * 255.0;
                if dot(dir, normalize(in.position)) > 0.99 {
                    color = vec4(1.0 - scaled_displacement, 1.0 + clamp(scaled_displacement*0.8, -0.8, 1.0) + 0.2*clamp(scaled_displacement, 0.0, 1.0), 1.0 - abs(scaled_displacement), 1.0);
                    if contacts[i].a < 0 {
                        color = vec4(1.0, 0.0, 0.0, 1.0);
                    }
                }
            }
        }
    }

    // add border/outline
    if settings.circular_particles == 1 && (settings.render_outline == 1 || in.selected != 0) {
        if len > 0.5-border_width && len < 0.5 || border_pixel {
            if in.selected != 0 {
                color = vec4(1.0, 0.8, 0.0, 1.0);
                if fixity[in.id].x_vel_2 != 0 || fixity[in.id].y_vel_2 != 0 || fixity[in.id].rot_vel_2 != 0 {
                    color = vec4(f32(fixity[in.id].x_vel_2), f32(fixity[in.id].y_vel_2), f32(fixity[in.id].rot_vel_2), 1.0);
                }
            } else {
                if settings.colors == 0 && settings.use_part_color == 1 { 
                    
                } else if settings.use_part_color == 0 {
                    color = vec4(
                        srgb_to_linear(settings.outline_r), 
                        srgb_to_linear(settings.outline_g), 
                        srgb_to_linear(settings.outline_b),
                        1.0
                    );
                } else {
                    color = vec4(color.rgb*0.5, color.a);
                }
            }
        }
    }
    
    //done
    if input.pressed == 1 && click_info[0] == 0 {
        let pos = (vec2(in.pixel.x + 1.0, -in.pixel.y + 1.0))/2.0;
        let pixel = vec2(i32(pos.x*f32(in.w_h.x)),i32(pos.y*f32(in.w_h.y)));
        let lower_x = min(input.x, input.x + input.rW);
        let upper_x = max(input.x, input.x + input.rW);
        let lower_y = min(input.y, input.y + input.rH);
        let upper_y = max(input.y, input.y + input.rH);
        if pixel.x > lower_x && pixel.x < upper_x && pixel.y > lower_y && pixel.y < upper_y {
            if pixel.x == lower_x + 1 || pixel.x == upper_x - 1 || pixel.y == lower_y + 1 || pixel.y == upper_y - 1 {
                color = vec4(
                    srgb_to_linear(0.0/255.0),
                    srgb_to_linear(120.0/255.0),
                    srgb_to_linear(215.0/255.0),
                    0.0
                );
            } else {
                color = color + vec4(
                    srgb_to_linear(0.0/255.0),
                    srgb_to_linear(28.0/255.0),
                    srgb_to_linear(56.0/255.0),
                    0.0
                );
            }
        }
    }
    // if settings.lighting == 1 {
    //     let surf_pos = vec3(in.position, cos(len))*radii_buf[in.id];
    //     let part_delta = vec3(pos_buf[0], 0.0) - vec3(pos_buf[in.id], 0.0);
    //     let delta = part_delta - surf_pos;
    //     return color * max(0.0, dot(normalize(part_delta), normalize(surf_pos))) / max(0.2, length(delta));
    // }
    return color;
}

fn linear_to_srgb(value: f32) -> f32 {
    if (value <= 0.0031308) {
        return 12.92 * value;
    } else {
        return 1.055 * pow(value, 1.0 / 2.4) - 0.055;
    }
}

fn srgb_to_linear(value: f32) -> f32 {
    if value <= 0.04045 {
        return value / 12.92;
    } else {
        return pow((value + 0.055) / 1.055, 2.4);
    }
}

fn cross(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x
    );
}

fn point_line_distance(point: vec2<f32>, line_point: vec2<f32>, line_vector: vec2<f32>) -> f32 {
    let v = normalize(line_vector);
    let pa = point - line_point;
    let projection = dot(pa, v) * v;
    let perpendicular = pa - projection;
    return length(perpendicular);
}