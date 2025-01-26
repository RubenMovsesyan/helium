use std::time::Instant;

use cgmath::{Vector3, Zero};

pub struct Gravity {
    pub velocity: Vector3<f32>,
    acceleration: Vector3<f32>,
}

impl Gravity {
    pub fn new(gravitational_constant: Vector3<f32>) -> Self {
        Self {
            velocity: Vector3::zero(),
            acceleration: gravitational_constant,
        }
    }

    pub fn update_gravity(&mut self, delta_time: &Instant) -> &mut Self {
        self.velocity += self.acceleration * delta_time.elapsed().as_secs_f32();
        self
    }

    pub fn set_gravity(&mut self, gravitational_constant: Vector3<f32>) -> &mut Self {
        self.acceleration = gravitational_constant;
        self
    }

    pub fn kill_velocity(&mut self) -> &mut Self {
        self.velocity = Vector3::zero();
        self
    }
}
