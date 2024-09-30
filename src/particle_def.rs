use std::{
    sync::atomic::{AtomicUsize, Ordering},
    usize,
};

use rand::random;

static INSTANCE_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
pub struct Particle_Definition {
    pub name: String,
    pub x_vel: f32,
    pub y_vel: f32,
    pub rot_vel: f32,
    pub x_force: f32,
    pub y_force: f32,
    pub rot_force: f32,
    pub radius: f32,
    pub random_radius: bool,
    pub min_radius: f32,
    pub max_radius: f32,
    pub next_radius: f32,
    pub x_fixity: bool,
    pub y_fixity: bool,
    pub rot_fixity: bool,
    pub material: i32,
}

impl Particle_Definition {
    pub fn default() -> Self {
        let id = INSTANCE_COUNT.fetch_add(1, Ordering::SeqCst);
        let mut p_def = Particle_Definition {
            name: Particle_Definition::name_from_id(id),
            x_vel: 0.0,
            y_vel: 0.0,
            rot_vel: 0.0,
            x_force: 0.0,
            y_force: 0.0,
            rot_force: 0.0,
            radius: 0.025,
            random_radius: false,
            min_radius: 0.020,
            max_radius: 0.030,
            next_radius: 1.0,
            x_fixity: false,
            y_fixity: false,
            rot_fixity: false,
            material: 0,
        };

        p_def.next_radius = p_def.new_radius();

        return p_def;
    }

    fn name_from_id(id: usize) -> String {
        match id {
            0 => format!("Default"),
            _ => format!("Definiton {}", id),
        }
    }

    fn delete() {
        INSTANCE_COUNT.fetch_sub(1, Ordering::SeqCst);
    }

    pub fn new_radius(&mut self) -> f32 {
        if self.random_radius {
            self.next_radius = random::<f32>() * (self.max_radius - self.min_radius) + self.min_radius;
        } else {
            self.next_radius = self.radius;
        }
        self.next_radius
    }

    pub fn spawned(&mut self) {}
}
