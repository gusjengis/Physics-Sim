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
    b: i32,
    bond_index: i32,
    forces: vec2<f32>,
    moment: f32,
    s_force: vec2<f32>,
    theta_b:  f32,
    bond_type: i32,
    bond_length: f32,
    bond_angle: f32
};


struct Bond {
    index: i32,
    angle: f32,
    length: f32
};

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

