use std::f32::consts::PI;

use rand::Rng;

use crate::settings::{*, self};
use crate::wgpu_config::*;

pub fn p_count(settings: &mut Settings) -> usize {
    match settings.structure {
        settings::Structure::Grid => {
            return settings.particles
        },
        settings::Structure::Mats => {
            return 4
        },
        settings::Structure::Random => {return settings.particles},
        _ => {
            return 2;
        }
    }
}


pub fn grid(settings: &mut Settings, pos: &mut Vec<f32>, vel: &mut Vec<f32>, rot: &mut Vec<f32>, rot_vel: &mut Vec<f32>, radii: &mut Vec<f32>, fixity: &mut Vec<i32>, forces: &mut Vec<f32>, material_pointers: &mut Vec<i32>) -> (Vec<i32>, Vec<i32>){
    settings.two_part = false;

    settings.materials.resize(settings.material_size*2, 0.0);

    // settings.materials[0] = 1.0; // Red
    // settings.materials[1] = 0.0; // Green
    // settings.materials[2] = 0.0; // Blue
    // settings.materials[3] = 1.0; // Density
    // settings.materials[4] = 10.0; // Normal Stiffness
    // settings.materials[5] = 0.25; // Shear Stiffness

    // // settings.materials[6] = 0.0; // Red
    // // settings.materials[7] = 0.0; // Green
    // // settings.materials[8] = 1.0; // Blue
    // settings.materials[9] = 1.0; // Density
    // settings.materials[10] = 10.0; // Normal Stiffness
    // settings.materials[11] = 0.25; // Shear Stiffness
    
    // settings.hor_bound = 3.0;    
    // settings.vert_bound = 2.0;
    // settings.scale = 0.5;
    let p_count = settings.particles;
    let mut rng = rand::thread_rng();
    let max_rad = settings.max_radius;
    let min_rad = settings.min_radius;
    let max_h_vel = settings.max_h_velocity;
    let min_h_vel = settings.min_h_velocity;
    let max_v_vel = settings.max_v_velocity;
    let min_v_vel = settings.min_v_velocity;
    let workgroups = settings.workgroups as f32;
    let max_pos_y = 20.0;
    let max_pos_x = 20.0;
    let mut distance = 0.0;
    for i in 0..p_count {
        let off = 0.0;//25;
        if i == 3 {
            distance = ((pos[0]-pos[2]).powf(2.0) + (pos[1]-pos[3]).powf(2.0)).powf(0.5)/2.0;
        }
        pos[i*2] = (i as f32%settings.grid_width)/max_pos_x - settings.grid_width/max_pos_x/2.0;
        if (i as f32/settings.grid_width%2.0).floor() == 1.0 {
            pos[i*2] += off;
        }
        pos[i*2+1] = ((i as f32)/settings.grid_width).floor()/max_pos_y - p_count as f32/settings.grid_width/max_pos_y/2.0;

        if min_h_vel < max_h_vel { vel[i*2] = rng.gen_range(min_h_vel..max_h_vel); } else { vel[i*2] = min_h_vel; }
        if min_v_vel < max_v_vel { vel[i*2+1] = rng.gen_range(min_v_vel..max_v_vel); } else { vel[i*2+1] = min_v_vel; }
        // if rng.gen_range(0.0..=1.0) > 0.999 {
        //     material_pointers[i] = 1;
        // }
    }
    for i in 0..radii.len() as usize {
        if settings.variable_rad && min_rad < max_rad {
            radii[i] = rng.gen_range(min_rad..max_rad)*(material_pointers[i] as f32*25.0 + 1.0);
        } else {
            radii[i] = max_rad;
        }
    }
    // Initialize Collision Sections
    // let vert_bound = 2.0;
    // let hor_bound = vert_bound*16.0/11.0;
    // let coll_grid_w = 30;
    // let coll_grid_h = 30;
    // let coll_section_size = 100;
    // let mut col_sec = vec![-1 as i32; coll_grid_w*coll_grid_h*coll_section_size];
    // Initialize Bonds
    let MAX_BONDS = settings.max_bonds;
    let mut bonds = vec![-1; p_count*MAX_BONDS*3];
    let mut bond_info = vec![-1; p_count*2];
    // let mut found_bonds = false;
    // for i in 0..p_count {
    //     let mut col_num = 0;
    //     for j in 0..p_count {
    //         if j != i {
    //             if ((pos[j*2] - pos[i*2]).powf(2.0) + (pos[j*2+1] - pos[i*2+1]).powf(2.0)).powf(0.5) < radii[i] + radii[j] {
    //                 if col_num < MAX_BONDS && bonds[(i*MAX_BONDS+col_num)*3] == -1 {
    //                     bonds[(i*MAX_BONDS+col_num)*3] = j as i32;
    //                     let delta = (pos[j*2] - pos[i*2], pos[j*2+1] - pos[i*2+1]);
    //                     let magnitude = (delta.0*delta.0 + delta.1*delta.1).powf(0.5);
    //                     let normalized_delta = (delta.0/magnitude, delta.1/magnitude);
    //                     let angle = normalized_delta.0.atan2(normalized_delta.1);
    //                     // println!("({}, {}) vs ({}, {})", normalized_delta.0, normalized_delta.1, angle.sin(), angle.cos());
    //                     bonds[(i*MAX_BONDS+col_num)*3+1] = (angle).to_bits() as i32;
    //                     bonds[(i*MAX_BONDS+col_num)*3+2] = (magnitude).to_bits() as i32;
    //                     // println!("{}, {}, {}", bonds[(i*MAX_BONDS+col_num)*3], angle, magnitude);
    //                     col_num += 1;
    //                     found_bonds = true;
    //                 } else if col_num == MAX_BONDS{
    //                     break;
    //                 }
    //             }
    //         }
    //     }
    // }
    // let mut index = 0;
    // for i in 0..p_count {
    //     let start = index;
    //     let mut length = 0;
    //     for j in 0..MAX_BONDS {
    //         if bonds[(i*MAX_BONDS+j)*3] != -1 {
    //             length += 1;
    //             index += 1;
    //         }
    //     }
    //     if length > 0 {
    //         bond_info[i*2] = start as i32;
    //         bond_info[i*2+1] = length as i32;
    //     } else {
    //         bond_info[i*2] = -1;
    //         bond_info[i*2+1] = -1;
    //     }
    // }
    // if found_bonds {
    //     bonds = (bonds).into_iter().filter(|num| *num != -1).collect();
    // }

    for i in 0..radii.len() {
        radii[i] *= distance/max_rad;// * 1.99;
    }
    return (bonds, bond_info);
}

pub fn build_bonds(config: &mut WGPUConfig, pos: &mut Vec<f32>, radii: &mut Vec<f32>) -> (Vec<i32>, Vec<i32>){ 
    let mut p_count = p_count(&mut config.prog_settings);
    let MAX_BONDS = config.prog_settings.max_bonds;
    let mut bonds = vec![-1; p_count*MAX_BONDS*3];
    let mut bond_info = vec![-1; p_count*2];
    let mut found_bonds = true;
    for i in 0..p_count {
        let mut col_num = 0;
        for j in 0..p_count {
            if j != i {
                if ((pos[j*2] - pos[i*2]).powf(2.0) + (pos[j*2+1] - pos[i*2+1]).powf(2.0)).powf(0.5) < (radii[i] + radii[j])*1.02 {
                    if col_num < MAX_BONDS && bonds[(i*MAX_BONDS+col_num)*3] == -1 {
                        bonds[(i*MAX_BONDS+col_num)*3] = j as i32;
                        let delta = (pos[j*2] - pos[i*2], pos[j*2+1] - pos[i*2+1]);
                        let magnitude = (delta.0*delta.0 + delta.1*delta.1).powf(0.5);
                        let normalized_delta = (delta.0/magnitude, delta.1/magnitude);
                        let angle = normalized_delta.0.atan2(normalized_delta.1);
                        // println!("({}, {}) vs ({}, {})", normalized_delta.0, normalized_delta.1, angle.sin(), angle.cos());
                        bonds[(i*MAX_BONDS+col_num)*3+1] = (angle).to_bits() as i32;
                        bonds[(i*MAX_BONDS+col_num)*3+2] = (magnitude).to_bits() as i32;
                        // println!("{}, {}, {}", bonds[(i*MAX_BONDS+col_num)*3], angle, magnitude);
                        col_num += 1;
                        found_bonds = true;
                    } else if col_num == MAX_BONDS{
                        break;
                    }
                }
            }
        }
    }

    let mut index = 0;
    for i in 0..p_count {
        let start = index;
        let mut length = 0;
        for j in 0..MAX_BONDS {
            if bonds[(i*MAX_BONDS+j)*3] != -1 {
                length += 1;
                index += 1;
            }
        }
        if length > 0 {
            bond_info[i*2] = start as i32;
            bond_info[i*2+1] = length as i32;
        } else {
            bond_info[i*2] = -1;
            bond_info[i*2+1] = -1;
        }
    }
    if found_bonds {
        bonds = (bonds).into_iter().filter(|num| *num != -1).collect();
    }

    return (bonds, bond_info);
}

// Two-Particle Experiments
pub fn exp1(settings: &mut Settings, pos: &mut Vec<f32>, vel: &mut Vec<f32>, rot: &mut Vec<f32>, rot_vel: &mut Vec<f32>, radii: &mut Vec<f32>, fixity: &mut Vec<i32>, forces: &mut Vec<f32>, material_pointers: &mut Vec<i32>) -> (Vec<i32>, Vec<i32>){
    settings.colors = true;
    settings.gravity = true;
    settings.friction = true;
    // settings.bonds = 2;
    // settings.hor_bound = 2.666;    
    // settings.vert_bound = 2.0;
    // settings.scale = 0.5;
    settings.render_rot = true;
    settings.color_code_rot = false;
    // settings.two_part = true;
    //            A                  B
    pos[0]     = -0.199; pos[2]     =  0.2; // X
    pos[1]     =  0.0; pos[3]     =  0.0; // Y
    rot[0]     =  0.0; rot[1]     =  0.0; // Angle
    vel[0]     =  0.0; vel[2]     =  0.0; // X Velocity
    vel[1]     = -0.2; vel[3]     =  0.2; // Y Velocity
    rot_vel[0] =  0.0; rot_vel[1] =  0.0; // Angular Velocity
    radii[0]   =  0.2; radii[1]   =  0.2; // Radius


    let delta = (pos[2] - pos[0], pos[3] - pos[1]);
    let magnitude = (delta.0*delta.0 + delta.1*delta.1).powf(0.5);
    let normalized_delta = (delta.0/magnitude, delta.1/magnitude);
    let angle = normalized_delta.0.atan2(normalized_delta.1);
    return (vec![1, (angle).to_bits() as i32, (magnitude).to_bits() as i32,
                 0, (angle+PI).to_bits() as i32, (magnitude).to_bits() as i32], // bonds
            vec![0, 1, 
                 1, 1]); // bond_info
}

pub fn exp2(settings: &mut Settings, pos: &mut Vec<f32>, vel: &mut Vec<f32>, rot: &mut Vec<f32>, rot_vel: &mut Vec<f32>, radii: &mut Vec<f32>, fixity: &mut Vec<i32>, forces: &mut Vec<f32>, material_pointers: &mut Vec<i32>) -> (Vec<i32>, Vec<i32>){
    settings.colors = true;
    let mut rng = rand::thread_rng();

    settings.gravity = false;
    // settings.hor_bound = 3.0;    
    // settings.vert_bound = 2.0;
    // settings.scale = 0.5;
    settings.render_rot = true;
    settings.color_code_rot = true;
    settings.two_part = true;
    //            A                  B
    pos[0]     = -0.5; pos[2]     =  0.5; // X
    pos[1]     =  0.0; pos[3]     =  0.0; // Y
    rot[0]     =  0.0; rot[1]     =  0.0; // Angle
    vel[0]     =  5.0; vel[2]     =  0.0; // X Velocity //rng.gen_range(1.0..10.0)
    vel[1]     =  0.0; vel[3]     =  0.0; // Y Velocity
    rot_vel[0] =  0.0; rot_vel[1] =  -100.0; // Angular Velocity
    radii[0]   =  0.2; radii[1]   =  0.2; // Radius

    //Fixity
    //          A              B
    fixity[0] = 0; fixity[3] = 1; // X-Velocity
    fixity[1] = 0; fixity[4] = 1; // Y-Velocity
    fixity[2] = 0; fixity[5] = 1; // Angular-Velocity

    //Forces
    //            A                  B
    forces[0] =  0.0; forces[6]  =  0.0; // X-Force
    forces[1] =  0.0; forces[7]  =  0.0; // Y-Force
    forces[2] =  0.0; forces[8]  =  0.0; // Moment
    forces[3] =  0.0; forces[9]  =  0.0; // X-Force Vel
    forces[4] =  0.0; forces[10] =  0.0; // Y-Force Vel
    forces[5] =  0.0; forces[11] =  0.0; // Moment Vel

    return (vec![-1; 2*2], vec![-1; 2*settings.max_bonds]);

}

pub fn exp3(settings: &mut Settings, pos: &mut Vec<f32>, vel: &mut Vec<f32>, rot: &mut Vec<f32>, rot_vel: &mut Vec<f32>, radii: &mut Vec<f32>, fixity: &mut Vec<i32>, forces: &mut Vec<f32>, material_pointers: &mut Vec<i32>) -> (Vec<i32>, Vec<i32>){
    settings.colors = true;
    settings.gravity = false;
    // settings.hor_bound = 1.5;    
    // settings.vert_bound = 1.0;
    // settings.scale = 1.0;
    settings.render_rot = true;
    settings.color_code_rot = true;
    settings.two_part = true;
    //             A                  B
    pos[0]     =  0.0; pos[2]     =  0.020; // X
    pos[1]     =  0.0; pos[3]     =  0.0; // Y
    rot[0]     =  0.0; rot[1]     =  0.0; // Angle
    vel[0]     =  0.0; vel[2]     =  0.0; // X Velocity
    vel[1]     =  0.0; vel[3]     =  0.0; // Y Velocity
    rot_vel[0] =  0.0; rot_vel[1] =  0.0; // Angular Velocity
    radii[0]   =  0.01; radii[1]   = 0.01; // Radius

    //Fixity
    //          A              B
    fixity[0] = 1; fixity[3] = 0; // X-Velocity
    fixity[1] = 1; fixity[4] = 1; // Y-Velocity
    fixity[2] = 0; fixity[5] = 1; // Angular-Velocity

    //Forces
    //            A                  B
    forces[0] =  0.0; forces[6]  =  0.0; // X-Force
    forces[1] =  0.0; forces[7]  =  0.0; // Y-Force
    forces[2] =  5.0; forces[8]  =  0.0; // Moment
    forces[3] =  0.0; forces[9]  = -100.0; // X-Force Vel
    forces[4] =  0.0; forces[10] =  0.0; // Y-Force Vel
    forces[5] =  0.0; forces[11] =  0.0; // Moment Vel

    return (vec![-1; 2*2], vec![-1; 2*settings.max_bonds]);

}

pub fn exp4(settings: &mut Settings, pos: &mut Vec<f32>, vel: &mut Vec<f32>, rot: &mut Vec<f32>, rot_vel: &mut Vec<f32>, radii: &mut Vec<f32>, fixity: &mut Vec<i32>, forces: &mut Vec<f32>, material_pointers: &mut Vec<i32>) -> (Vec<i32>, Vec<i32>){
    settings.colors = true;
    settings.gravity = false;
    // settings.hor_bound = 1.5;    
    // settings.vert_bound = 1.0;
    // settings.scale = 1.0;
    settings.render_rot = true;
    settings.color_code_rot = true;
    settings.two_part = true;

    //            A                  B
    pos[0]     =  -0.2; pos[2]     = 0.2; // X
    pos[1]     =  0.0; pos[3]     =  0.0; // Y
    rot[0]     =  0.0; rot[1]     =  0.0; // Angle
    vel[0]     =  0.0; vel[2]     =  0.0; // X Velocity
    vel[1]     =  0.0; vel[3]     =  0.0; // Y Velocity
    rot_vel[0] =  0.0; rot_vel[1] =  10.0; // Angular Velocity
    radii[0]   =  0.2; radii[1]   =  0.2; // Radius

    //Fixity
    //          A              B
    fixity[0] = 0; fixity[3] = 1; // X-Velocity
    fixity[1] = 1; fixity[4] = 1; // Y-Velocity
    fixity[2] = 0; fixity[5] = 0; // Angular-Velocity
    
    //Forces
    //            A                  B
    forces[0] =  0.0; forces[6]  =  0.0; // X-Force
    forces[1] =  0.0; forces[7]  =  0.0; // Y-Force
    forces[2] =  0.0; forces[8]  =  0.0; // Moment
    forces[3] =  500.0; forces[9]  =  0.0; // X-Force Vel
    forces[4] =  0.0; forces[10] =  0.0; // Y-Force Vel
    forces[5] =  0.0; forces[11] =  0.0; // Moment Vel
    return (vec![-1; 2*2], vec![-1; 2*settings.max_bonds]);

}

pub fn exp5(settings: &mut Settings, pos: &mut Vec<f32>, vel: &mut Vec<f32>, rot: &mut Vec<f32>, rot_vel: &mut Vec<f32>, radii: &mut Vec<f32>, fixity: &mut Vec<i32>, forces: &mut Vec<f32>, material_pointers: &mut Vec<i32>) -> (Vec<i32>, Vec<i32>){
    settings.colors = true;
    settings.gravity = false;
    // settings.hor_bound = 2.666;    
    // settings.vert_bound = 2.0;
    // settings.scale = 0.5;
    settings.render_rot = true;
    settings.color_code_rot = true;
    settings.two_part = true;
    //            A                  B
    pos[0]     = -0.8; pos[2]     =  0.5; // X
    pos[1]     =  0.0; pos[3]     =  0.399; // Y
    rot[0]     =  0.0; rot[1]     =  0.0; // Angle
    vel[0]     =  0.0; vel[2]     = -10.0; // X Velocity
    vel[1]     =  0.0; vel[3]     =  0.0; // Y Velocity
    rot_vel[0] =  0.0; rot_vel[1] =  0.0; // Angular Velocity
    radii[0]   =  0.2; radii[1]   =  0.2; // Radius

    //Fixity
    //          A              B
    fixity[0] = 0; fixity[3] = 0; // X-Velocity
    fixity[1] = 0; fixity[4] = 0; // Y-Velocity
    fixity[2] = 0; fixity[5] = 0; // Angular-Velocity
    
    //Forces
    //            A                  B
    forces[0] =  0.0; forces[6]  =  0.0; // X-Force
    forces[1] =  0.0; forces[7]  =  0.0; // Y-Force
    forces[2] =  0.0; forces[8]  =  0.0; // Moment
    forces[3] =  0.0; forces[9]  =  0.0; // X-Force Vel
    forces[4] =  0.0; forces[10] =  0.0; // Y-Force Vel
    forces[5] =  0.0; forces[11] =  0.0; // Moment Vel

    return (vec![-1; 2*2], vec![-1; 2*settings.max_bonds]);

}

pub fn exp6(settings: &mut Settings, pos: &mut Vec<f32>, vel: &mut Vec<f32>, rot: &mut Vec<f32>, rot_vel: &mut Vec<f32>, radii: &mut Vec<f32>, fixity: &mut Vec<i32>, forces: &mut Vec<f32>, material_pointers: &mut Vec<i32>) -> (Vec<i32>, Vec<i32>){
    settings.colors = true;
    settings.gravity = false;
    settings.linear_contact_bonds = false;
    settings.friction = false;
    // settings.hor_bound = 2.666;    
    // settings.vert_bound = 2.0;
    // settings.scale = 0.5;
    settings.render_rot = true;
    settings.color_code_rot = true;
    settings.two_part = true;
    //            A                  B
    pos[0]     = -0.5; pos[2]     =  0.5; // X
    pos[1]     =  0.0; pos[3]     =  0.0; // Y
    rot[0]     =  0.0; rot[1]     =  0.0; // Angle
    vel[0]     =  0.0; vel[2]     =  0.0; // X Velocity
    vel[1]     =  0.0; vel[3]     =  0.0; // Y Velocity
    rot_vel[0] =  0.0; rot_vel[1] =  100.0; // Angular Velocity
    radii[0]   =  0.5; radii[1]   =  0.5; // Radius

    //Fixity
    //          A              B
    fixity[0] = 0; fixity[3] = 1; // X-Velocity
    fixity[1] = 0; fixity[4] = 1; // Y-Velocity
    fixity[2] = 0; fixity[5] = 1; // Angular-Velocity
    
    //Forces
    //            A                  B
    forces[0] =  0.0; forces[6]  =  0.0; // X-Force
    forces[1] =  0.0; forces[7]  =  0.0; // Y-Force
    forces[2] =  0.0; forces[8]  =  0.0; // Moment
    forces[3] =  0.0; forces[9]  =  0.0; // X-Force Vel
    forces[4] =  0.0; forces[10] =  0.0; // Y-Force Vel
    forces[5] =  0.0; forces[11] =  0.0; // Moment Vel

    let delta = (pos[2] - pos[0], pos[3] - pos[1]);
    let magnitude = (delta.0*delta.0 + delta.1*delta.1).powf(0.5);
    let normalized_delta = (delta.0/magnitude, delta.1/magnitude);
    let angle = normalized_delta.0.atan2(normalized_delta.1);
    return (vec![-1; 2*2], vec![-1; 2*settings.max_bonds]);
}

pub fn mats(settings: &mut Settings, pos: &mut Vec<f32>, vel: &mut Vec<f32>, rot: &mut Vec<f32>, rot_vel: &mut Vec<f32>, radii: &mut Vec<f32>, fixity: &mut Vec<i32>, forces: &mut Vec<f32>, material_pointers: &mut Vec<i32>) -> (Vec<i32>, Vec<i32>){
    settings.colors = true;
    settings.gravity = true;
    settings.friction = true;
    // settings.bonds = false;
    // settings.hor_bound = 2.666;    
    // settings.vert_bound = 2.0;
    // settings.scale = 0.5;
    settings.render_rot = true;
    settings.color_code_rot = false;
    // settings.two_part = true;
    //            A                  B
    pos[0]     = -0.5; pos[2]     =  0.5; // X
    pos[1]     =  0.0; pos[3]     =  0.0; // Y
    rot[0]     =  0.0; rot[1]     =  0.0; // Angle
    vel[0]     =  10.0; vel[2]     = -10.0; // X Velocity
    vel[1]     =  0.0; vel[3]     =  0.0; // Y Velocity
    rot_vel[0] =  0.0; rot_vel[1] =  0.0; // Angular Velocity
    radii[0]   =  0.2; radii[1]   =  0.2; // Radius

    //            A                  B
    pos[4+0]     = -0.5; pos[4+2]     =  0.5; // X
    pos[4+1]     = -0.5; pos[4+3]     =  0.5; // Y
    rot[2+0]     =  0.0; rot[2+1]     =  0.0; // Angle
    vel[4+0]     =  10.0; vel[4+2]     = -10.0; // X Velocity
    vel[4+1]     =  0.0; vel[4+3]     =  0.0; // Y Velocity
    rot_vel[2+0] =  0.0; rot_vel[2+1] =  0.0; // Angular Velocity
    radii[2+0]   =  0.2; radii[2+1]   =  0.2; // Radius

    settings.materials.resize(settings.material_size*2, 0.0);

    settings.materials[0] = 1.0; // Red
    settings.materials[1] = 0.0; // Green
    settings.materials[2] = 0.0; // Blue
    settings.materials[3] = 1.0; // Density
    settings.materials[4] = 10.0; // Normal Stiffness
    settings.materials[5] = 0.25; // Shear Stiffness

    settings.materials[6] = 0.0; // Red
    settings.materials[7] = 0.0; // Green
    settings.materials[8] = 1.0; // Blue
    settings.materials[9] = 1.0; // Density
    settings.materials[10] = 10.0; // Normal Stiffness
    settings.materials[11] = 0.25; // Shear Stiffness

    material_pointers[0] = 0;
    material_pointers[1] = 1;
    material_pointers[2] = 0;
    material_pointers[3] = 1;

    let delta = (pos[2] - pos[0], pos[3] - pos[1]);
    let magnitude = (delta.0*delta.0 + delta.1*delta.1).powf(0.5);
    let normalized_delta = (delta.0/magnitude, delta.1/magnitude);
    let angle = normalized_delta.0.atan2(normalized_delta.1);
    return (vec![1, (angle).to_bits() as i32, (magnitude).to_bits() as i32,
                 0, (angle+PI).to_bits() as i32, (magnitude).to_bits() as i32], // bonds
            vec![0, 1, 
                 1, 1]); // bond_info
}
