use std::time::Instant;

use cgmath::{InnerSpace, Point3, Vector3};

// Change this later
const CAMERA_SPEED: f32 = 100.0;

#[derive(Clone, Copy)]
pub struct Camera3d {
    pub eye: Point3<f32>,
    pub target: Vector3<f32>,
    pub up: Vector3<f32>,

    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera3d {
    pub fn new(
        eye: Point3<f32>,
        target: Vector3<f32>,
        up: Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn update_camera(
        &mut self,
        forward: bool,
        backward: bool,
        strafe_left: bool,
        strafe_right: bool,
        turn_left: bool,
        turn_right: bool,
        time: Instant,
    ) {
        let forward_norm = self.target.normalize();
        if forward {
            self.eye += self.target * time.elapsed().as_secs_f32() * CAMERA_SPEED;
        }

        if backward {
            self.eye -= self.target * time.elapsed().as_secs_f32() * CAMERA_SPEED;
        }

        let right = forward_norm.cross(self.up);

        if strafe_left {
            self.eye -= right * time.elapsed().as_secs_f32() * CAMERA_SPEED;
        }

        if strafe_right {
            self.eye += right * time.elapsed().as_secs_f32() * CAMERA_SPEED;
        }
    }
}
