use std::f32::consts::PI;

use rand::Rng;

use crate::settings::{*, self};
use crate::wgpu_config::*;

pub fn p_count(settings: &mut Settings) -> usize {
    match settings.setup.structure {
        settings::Structure::Grid => {
            return settings.setup.particles
        },
        settings::Structure::Mats => {
            return 4
        },
        settings::Structure::Random => {return settings.setup.particles},
        _ => {
            return 2;
        }
    }
}


pub fn grid(settings: &mut Settings, pos: &mut Vec<f32>, vel: &mut Vec<f32>, radii: &mut Vec<f32>, fixity: &mut Vec<i32>, forces: &mut Vec<f32>, material_pointers: &mut Vec<i32>) {
    let p_count = settings.setup.particles;
    let mut rng = rand::thread_rng();
    let max_rad = settings.setup.max_radius;
    let min_rad = settings.setup.min_radius;
    let max_h_vel = settings.setup.max_h_velocity;
    let min_h_vel = settings.setup.min_h_velocity;
    let max_v_vel = settings.setup.max_v_velocity;
    let min_v_vel = settings.setup.min_v_velocity;
    let workgroups = settings.setup.workgroups as f32;
    let max_pos_y = 20.0;
    let max_pos_x = 20.0;
    for i in 0..p_count {
        pos[i*2] = (i as f32%settings.setup.grid_width)*(max_rad*2.0) - settings.setup.grid_width*(max_rad*2.0)/2.0 + max_rad;
        pos[i*2+1] = ((i as f32)/settings.setup.grid_width).floor()*(max_rad*2.0) - p_count as f32/settings.setup.grid_width*(max_rad*2.0)/2.0 + max_rad;

        if min_h_vel < max_h_vel { vel[i*2] = rng.gen_range(min_h_vel..max_h_vel); } else { vel[i*2] = min_h_vel; }
        if min_v_vel < max_v_vel { vel[i*2+1] = rng.gen_range(min_v_vel..max_v_vel); } else { vel[i*2+1] = min_v_vel; }
    }
    for i in 0..radii.len() as usize {
        if settings.setup.variable_rad && min_rad < max_rad {
            radii[i] = rng.gen_range(min_rad..max_rad);
        } else {
            radii[i] = max_rad;
        }
    }
}