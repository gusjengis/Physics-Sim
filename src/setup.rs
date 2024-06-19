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


pub fn grid(settings: &mut Settings, pos: &mut Vec<f32>, vel: &mut Vec<f32>, radii: &mut Vec<f32>, fixity: &mut Vec<i32>, forces: &mut Vec<f32>, material_pointers: &mut Vec<i32>) {
    settings.two_part = false;
    // println!("{}", settings.materials.len());
    // settings.materials.resize(settings.material_size*2, 0.0);

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
    for i in 0..p_count {
        pos[i*2] = (i as f32%settings.grid_width)/max_pos_x - settings.grid_width/max_pos_x/2.0;
        pos[i*2+1] = ((i as f32)/settings.grid_width).floor()/max_pos_y - p_count as f32/settings.grid_width/max_pos_y/2.0;

        if min_h_vel < max_h_vel { vel[i*2] = rng.gen_range(min_h_vel..max_h_vel); } else { vel[i*2] = min_h_vel; }
        if min_v_vel < max_v_vel { vel[i*2+1] = rng.gen_range(min_v_vel..max_v_vel); } else { vel[i*2+1] = min_v_vel; }
    }
    for i in 0..radii.len() as usize {
        if settings.variable_rad && min_rad < max_rad {
            radii[i] = rng.gen_range(min_rad..max_rad);//*(material_pointers[i] as f32*25.0 + 1.0);
        } else {
            radii[i] = max_rad;
        }
    }
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