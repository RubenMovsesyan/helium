use std::time::Instant;

use cgmath::{InnerSpace, Point3, Quaternion, Rotation, Vector3};
use log::info;
use winit::{
    event::{DeviceEvent, ElementState, KeyEvent, RawKeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

// Change this later
const CAMERA_SPEED: f32 = 50.0;
const ANGLE_SPEED: f32 = 0.01;

#[derive(Clone, Copy)]
pub struct Camera3d {
    pub eye: Point3<f32>,
    pub target: Vector3<f32>,
    pub up: Vector3<f32>,

    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,

    update_flag: bool,
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
            update_flag: true,
        }
    }

    /// Rotates the camera pitch by the specified angle
    ///
    /// # Arguments
    ///
    /// `angle` - Angle to change the pitch of the camera by
    pub fn add_pitch(&mut self, angle: f32) {
        let forward_norm = self.target.normalize();
        let right = forward_norm.cross(self.up);

        let rotation = Quaternion {
            v: right * f32::sin(angle * ANGLE_SPEED / 2.0),
            s: f32::cos(angle * ANGLE_SPEED / 2.0),
        }
        .normalize();

        self.target = rotation.rotate_vector(self.target);
        self.update_flag = true;
    }

    /// Rotates the camera yaw by the specified angle
    ///
    /// # Arguments
    ///
    /// `angle` - Angle to change the yaw of the camera by
    pub fn add_yaw(&mut self, angle: f32) {
        let up_norm = self.up.normalize();
        let rotation = Quaternion {
            v: up_norm * f32::sin(angle * ANGLE_SPEED / 2.0),
            s: f32::cos(angle * ANGLE_SPEED / 2.0),
        }
        .normalize();

        self.target = rotation.rotate_vector(self.target);
        self.update_flag = true;
    }

    pub fn update_camera(
        &mut self,
        forward: bool,
        backward: bool,
        strafe_left: bool,
        strafe_right: bool,
        turn_left: bool,
        turn_right: bool,
        delta_time: &Instant,
    ) {
        let forward_norm = self.target.normalize();
        if forward {
            self.eye += forward_norm * delta_time.elapsed().as_secs_f32() * CAMERA_SPEED;
            self.update_flag = true;
        }

        if backward {
            self.eye -= forward_norm * delta_time.elapsed().as_secs_f32() * CAMERA_SPEED;
            self.update_flag = true;
        }

        let right = forward_norm.cross(self.up);

        if strafe_left {
            self.eye -= right * delta_time.elapsed().as_secs_f32() * CAMERA_SPEED;
            self.update_flag = true;
        }

        if strafe_right {
            self.eye += right * delta_time.elapsed().as_secs_f32() * CAMERA_SPEED;
            self.update_flag = true;
        }
    }

    pub fn get_update_flag(&self) -> bool {
        self.update_flag
    }
}

pub struct CameraController {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub delta: (f32, f32),
}

impl CameraController {
    pub fn process_events(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::Key(RawKeyEvent {
                physical_key: PhysicalKey::Code(keycode),
                state,
            }) => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    KeyCode::KeyW => {
                        self.forward = is_pressed;
                    }
                    KeyCode::KeyS => {
                        self.backward = is_pressed;
                    }
                    KeyCode::KeyA => {
                        self.left = is_pressed;
                    }
                    KeyCode::KeyD => {
                        self.right = is_pressed;
                    }
                    _ => {}
                }
            }
            DeviceEvent::MouseMotion { delta } => {
                self.delta = (delta.0 as f32, delta.1 as f32);
            }
            _ => {} // WindowEvent::KeyboardInput {
                    //     event:
                    //         KeyEvent {
                    //             state,
                    //             physical_key: PhysicalKey::Code(keycode),
                    //             ..
                    //         },
                    //     ..
                    // } => {
                    //     let is_pressed = *state == ElementState::Pressed;
                    //     match keycode {
                    //         KeyCode::KeyW => {
                    //             self.forward = is_pressed;
                    //         }
                    //         KeyCode::KeyS => {
                    //             self.backward = is_pressed;
                    //         }
                    //         KeyCode::KeyA => {
                    //             self.left = is_pressed;
                    //         }
                    //         KeyCode::KeyD => {
                    //             self.right = is_pressed;
                    //         }
                    //         _ => {}
                    //     }
                    // }
                    // _ => {}
        }
    }
}
