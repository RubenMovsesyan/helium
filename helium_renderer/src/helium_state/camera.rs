// cgmath imports
use cgmath::{perspective, Deg, InnerSpace, Matrix4, Point3, SquareMatrix, Vector3};

// wgpu imports
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, Device,
    ShaderStages,
};
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use super::resources::OPENGL_TO_WGPU_MATIX;

pub struct Camera {
    // Position and direction values
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,

    // Camera view values
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,

    // wgpu vars
    camera_uniform: CameraUniform,
    buffer: Buffer,
    layout: BindGroupLayout,
    bind_group: BindGroup,
}

impl Camera {
    pub fn get_bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn get_buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn new(
        device: &Device,
        eye: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj_with_matrix(Self::build_view_projection_matrix_parts(
            eye, target, up, aspect, fovy, znear, zfar,
        ));

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Camera bind group layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
            camera_uniform,
            buffer,
            layout,
            bind_group,
        }
    }

    pub fn get_uniform(&self) -> &CameraUniform {
        &self.camera_uniform
    }

    pub fn update_view_proj(&mut self) {
        self.camera_uniform
            .update_view_proj_with_matrix(Self::build_view_projection_matrix_parts(
                self.eye,
                self.target,
                self.up,
                self.aspect,
                self.fovy,
                self.znear,
                self.zfar,
            ));
    }

    fn build_view_projection_matrix_parts(
        eye: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(eye, target, up);
        let proj = perspective(Deg(fovy), aspect, znear, zfar);

        OPENGL_TO_WGPU_MATIX * proj * view
    }

    // fn build_view_projection_matrix(&self) -> Matrix4<f32> {
    //     let view = Matrix4::look_at_rh(self.eye, self.target, self.up);
    //     let proj = perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);

    //     OPENGL_TO_WGPU_MATIX * proj * view
    // }

    pub fn get_layout(&self) -> &BindGroupLayout {
        &self.layout
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Matrix4::identity().into(),
        }
    }

    // pub fn update_view_proj(&mut self, camera: &Camera) {
    //     self.view_proj = camera.build_view_projection_matrix().into();
    // }

    pub fn update_view_proj_with_matrix(&mut self, matrix: Matrix4<f32>) {
        self.view_proj = matrix.into();
    }
}

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_turn_left_pressed: bool,
    is_turn_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_turn_left_pressed: false,
            is_turn_right_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    KeyCode::KeyW => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyS => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyA => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyD => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    KeyCode::ArrowLeft => {
                        self.is_turn_left_pressed = is_pressed;
                        true
                    }
                    KeyCode::ArrowRight => {
                        self.is_turn_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }

        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);
        // let forward = camera.target - camera.eye;
        // let forward_mag = forward.magnitude();

        if self.is_left_pressed {
            camera.eye -= right * self.speed;
        }

        if self.is_right_pressed {
            camera.eye += right * self.speed;
        }
    }
}
