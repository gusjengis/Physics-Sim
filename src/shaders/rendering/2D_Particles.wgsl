struct VertexIn {
    @location(0) position: vec2<f32>,
};

struct Input {
    width: f32, time: f32,
    height: f32,
    xOff: f32, yOff: f32,
    ui_xOff: f32, ui_yOff: f32,
    ui_scale: f32, scale: f32,
    dark: f32,
    x: i32, y: i32,
    rW: i32, rH: i32,
    pressed: i32,
    timestamp: i32,
    p_count: i32,
    $ 3D { cam: Camera, }
}

struct Contact {
    b: i32,
    forces: vec2<f32>,
    moment: f32,
    s_force: vec2<f32>,
    theta_b:  f32,
    bond_type: i32,
    bond_length: f32,
    bond_angle: f32
};

$ 3D {
struct Camera {
        pos: vec4<f32>,
        view_proj: mat4x4<f32>,
        eye: mat4x4<f32>,
        focus: mat4x4<f32>,
    };
}

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
    $ 3D {@location(10) dir: vec3<f32>,}
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
    abb_strength: f32,
    bond_highlight_strength: f32,
    render_unbonded_contacts: i32
}

struct Bond {
    index: i32,
    angle: f32,
    length: f32
};

@group(0) @binding(0) var<uniform> input: Input;
@group(1) @binding(0) var<storage, read_write> pos: array<vec2<f32>>;
@group(1) @binding(1) var<storage, read_write> radii: array<f32>;
@group(2) @binding(0) var<storage, read_write> vel: array<vec2<f32>>;
@group(2) @binding(2) var<storage, read_write> rot: array<f32>;
@group(2) @binding(3) var<storage, read_write> rot_vel: array<f32>;
@group(2) @binding(6) var<storage, read_write> fixity: array<Particle_Settings>;
@group(3) @binding(1) var<storage, read_write> contacts: array<Contact>;
@group(3) @binding(3) var<storage, read_write> material_pointers: array<i32>;
@group(4) @binding(0) var<uniform> settings: Settings;
@group(5) @binding(0) var<storage, read_write> materials: array<Material>;
@group(6) @binding(0) var<storage, read_write> selections: array<i32>;
@group(7) @binding(0) var<storage, read_write> click_info: array<i32>;

const PI = 3.141592653589793238;

@vertex
fn vs_main(
    in: VertexIn,
    @builtin(instance_index) instance: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let scale = input.scale;
    $ 2D {
        let aspect = input.width / input.height;
        let xy = 2.0 * scale * vec2(in.position.x / aspect, in.position.y);
        let center = scale * vec2(pos[instance].x / aspect, pos[instance].y);
        let off = vec2((input.xOff + input.ui_xOff) / aspect, (input.yOff + input.ui_yOff)) * (scale);
        out.clip_position = vec4(xy * radii[instance] + center + off, 0.0, 1.0);
    }
    $ 3D {
        let cam_pos = input.cam.pos.xyz;
        let part_pos = vec3(pos[instance].x, pos[instance].y, 0.0);
        let dir = normalize(cam_pos - part_pos);

        // Create a rotation matrix to face the direction
        let up = vec3(0.0, 1.0, 0.0);
        let right = normalize(cross(up, dir));
        let rotated_up = cross(dir, right);
        let rotation_matrix = mat3x3(
            right,
            rotated_up,
            dir
        );

        let rotated_position = rotation_matrix * vec3(in.position, 0.0);

        let xy = 2.0 * input.ui_scale * rotated_position;
        let center = input.ui_scale * vec3(pos[instance].x, pos[instance].y, 0.0);
        let off = vec3(input.ui_xOff, input.ui_yOff, 0.0) * input.ui_scale;

        let final_pos = xy * radii[instance] + center + off;

        out.clip_position = input.cam.view_proj * vec4(final_pos, 1.0);
        out.dir = dir;
    }
    out.position = in.position;
    out.rot = rot[instance];
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
        let r = (1.0 - abs(angle - 1.0000 * PI) / (2.0 * PI / 3.0));
        let g = (max(0.0, 1.0 - abs(angle - 1.6666 * PI) / (2.0 * PI / 3.0)) + max(0.0, 1.0 - abs(angle + 0.3333 * PI) / (2.0 * PI / 3.0)));
        let b = (max(0.0, 1.0 - abs(angle - 0.3333 * PI) / (2.0 * PI / 3.0)) + max(0.0, 1.0 - abs(angle - 2.3333 * PI) / (2.0 * PI / 3.0)));

        out.color = vec3(r, g, b) * 1.0 / max(max(r, b), g);
    } else {
        out.color = vec3(1.0, 1.0, 1.0);
    }

    if settings.dim_slow_particles == 1 {
        let vel_mag = length(out.vel) / settings.max_brightness_vel;
        let max_brightness = min(1.0, vel_mag * vel_mag * vel_mag);
        out.color *= max_brightness;
    }
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let len = length(in.position);
    $ ROUND-PARTICLES {
        // discard corners to make circle
        if len > 0.5 {
            discard;
        }
    }

    var color = vec4(in.color, 1.0);

    let border_width = 0.08;

    $ ROTATION {
        // cut out wedge for rotation
        let rot_point = vec2(cos(in.rot), sin(in.rot));
        let rot_dot = dot(rot_point, normalize(in.position));
        if !(len > 0.5 - border_width && len < 0.5) {
            if rot_dot > 0.9 {
                color = vec4(0.0, 0.0, 0.0, 1.0);
            }
        }
    }

    $ COLOR-ROTATION {
        // color code based on direction of rotation
        if settings.color_code_rot == 1 {
            if in.rot_vel > 0.0 {
                color = vec4(0.0, color.g, color.ba);
            } else if in.rot_vel < 0.0 {
                color = vec4(color.r, 0.0, color.ba);
            }
        }
    }

    
    // bonds
    var border_pixel = false;
    for (var i = in.id * 14u; i < (in.id + 1u) * 14u; i++) {
        if contacts[i].b >= 0 {
            let delta = pos[contacts[i].b] - pos[in.id];  
            let dir = normalize(delta);
            let displacement = ((radii[in.id] + radii[contacts[i].b]) - length(delta));
            $ ROUND-PARTICLES {
                let midpoint = pos[in.id] + dir * (radii[in.id] - displacement * 0.5);
                let pixel_world_pos = pos[in.id] + radii[in.id] * in.position * 2.0;
                let test_vec = midpoint - pixel_world_pos;
                border_pixel = border_pixel || dot(test_vec, dir) < border_width * 2.0 * radii[in.id];
                if dot(delta, test_vec) < 0.0 {
                    discard;
                }
            }
            $ BONDS {
                if (contacts[i].bond_type > -1 || settings.render_unbonded_contacts == 1) {
                    let scaled_displacement = displacement / radii[in.id] * settings.bond_highlight_strength;
                    if dot(dir, normalize(in.position)) > 0.99 {
                        color = vec4(1.0 - scaled_displacement, 1.0 + clamp(scaled_displacement * 0.8, -0.8, 1.0) + 0.2 * clamp(scaled_displacement, 0.0, 1.0), 1.0 - abs(scaled_displacement), 1.0);
                        //color = vec4(0.0 + contacts[i].forces.x, 0.0 + contacts[i].forces.y, 0.0 + contacts[i].moment, 1.0);
                        if contacts[i].bond_type == -1 {
                            color = vec4(0.0, 1.0, 1.0, 1.0);
                        }// else if contacts[i].bond_type == -9 {
                        //    color = vec4(1.0, 0.0, 0.0, 1.0);
                        //} else if contacts[i].bond_type == -11 {
                        //    color = vec4(0.0, 0.0, 1.0, 1.0);
                        //}
                    }
                }
            }
        }
    }

    $ LIGHTING {
        $ 2D {
            let axes = build_orthonormal_basis(vec3(0.0, 0.0, 1.0));
            let x = axes[0];
            let y = axes[1];
            let z = axes[2];
            var surface_normal = normalize(
                sin(in.position.x * PI) * x + sin(in.position.y * PI) * y + cos(len * PI) * z
            );
            let center = vec3(pos[in.id], 0.0);
            let surface_pos = center + surface_normal * radii[in.id];
            let light_source = vec3(0.0, 0.0, 1.0);
            // let source_dist = light_source - surface_pos;
            let source_dir = vec3(0.0, 0.0, 1.0);
            color = color * min(1.0, max(0.015, dot(source_dir, surface_normal)));/// max(1.0, length(source_dist));
        }

        $ 3D {
            let axes = build_orthonormal_basis(in.dir);
            let x = axes[0];
            let y = axes[1];
            let z = axes[2];
            var surface_normal = normalize(
                sin(in.position.x * PI) * x + sin(in.position.y * PI) * y + cos(len * PI) * z
            );
            let center = vec3(pos[in.id], 0.0);
            let surface_pos = center + surface_normal * radii[in.id];
            let light_source = vec3(0.0, 0.0, 1.0);
            // let source_dist = light_source - surface_pos;
            let source_dir = vec3(0.0, 0.0, 1.0);
            color = color * min(1.0, max(0.015, dot(source_dir, surface_normal)));/// max(1.0, length(source_dist));
        }
    }

    $ ROUND-PARTICLES {
        // add border/outline
        if settings.render_outline == 1 || in.selected != 0 {
            if len > 0.5 - border_width && len < 0.5 || border_pixel {
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
                        color = vec4(color.rgb * 0.5, color.a);
                    }
                }
            }
        }
    }
    
    //done
    if input.pressed == 1 && click_info[0] == 0 {
        let pos = (vec2(in.pixel.x + 1.0, -in.pixel.y + 1.0)) / 2.0;
        let pixel = vec2(i32(pos.x * f32(in.w_h.x)), i32(pos.y * f32(in.w_h.y)));
        let lower_x = min(input.x, input.x + input.rW);
        let upper_x = max(input.x, input.x + input.rW);
        let lower_y = min(input.y, input.y + input.rH);
        let upper_y = max(input.y, input.y + input.rH);
        if pixel.x > lower_x && pixel.x < upper_x && pixel.y > lower_y && pixel.y < upper_y {
            if pixel.x == lower_x + 1 || pixel.x == upper_x - 1 || pixel.y == lower_y + 1 || pixel.y == upper_y - 1 {
                color = vec4(
                    srgb_to_linear(0.0 / 255.0),
                    srgb_to_linear(120.0 / 255.0),
                    srgb_to_linear(215.0 / 255.0),
                    0.0
                );
            } else {
                color = color + vec4(
                    srgb_to_linear(0.0 / 255.0),
                    srgb_to_linear(28.0 / 255.0),
                    srgb_to_linear(56.0 / 255.0),
                    0.0
                );
            }
        }
    }
    return color;
}

fn rand(seed: u32, max: f32) -> f32 {
    //PCG Hash
    var res = seed;
    res = res * 747796405u + 2891336453u;
    res = ((res >> ((res >> 28u) + 4u)) ^ res) * 277803737u;
    res = (res >> 22u) ^ res;

    return max * f32(res) / 4294967296.0;
}

fn linear_to_srgb(value: f32) -> f32 {
    if value <= 0.0031308 {
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

fn build_orthonormal_basis(z: vec3<f32>) -> mat3x3<f32> {
    let z_norm = normalize(z);
    
    // Choose an arbitrary vector not parallel to z
    var arbitrary = vec3<f32>(0.0, 1.0, 0.0);  // world up
    if abs(dot(z_norm, arbitrary)) > 0.99 {
        arbitrary = vec3<f32>(1.0, 0.0, 0.0);  // world right
    }
    
    // Compute x axis
    let x = normalize(cross(arbitrary, z_norm));
    
    // Compute y axis
    let y = cross(z_norm, x);
    
    // Return the orthonormal basis as a 3x3 matrix
    return mat3x3<f32>(
        x,
        y,
        z_norm
    );
}
