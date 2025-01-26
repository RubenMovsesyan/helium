// cgmath imports
use cgmath::{perspective, Deg, Matrix4, Point3, SquareMatrix, Vector3};

// wgpu imports
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, Device,
    ShaderStages,
};

use super::resources::OPENGL_TO_WGPU_MATIX;

pub struct Camera {
    // Position and direction values
    pub eye: Point3<f32>,
    pub target: Vector3<f32>,
    pub up: Vector3<f32>,

    // Camera view values
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,

    // wgpu vars
    pub camera_uniform: CameraUniform,
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

    pub fn get_camera_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }

    pub fn new(
        device: &Device,
        eye: Point3<f32>,
        target: Vector3<f32>,
        up: Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj_with_matrix(
            eye,
            Self::build_view_projection_matrix_parts(eye, target, up, aspect, fovy, znear, zfar),
        );

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Camera bind group layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
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
        self.camera_uniform.update_view_proj_with_matrix(
            self.eye,
            Self::build_view_projection_matrix_parts(
                self.eye,
                self.target,
                self.up,
                self.aspect,
                self.fovy,
                self.znear,
                self.zfar,
            ),
        );
    }

    pub fn build_view_projection_matrix_parts(
        eye: Point3<f32>,
        target: Vector3<f32>,
        up: Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(eye, eye + target, up);
        let proj = perspective(Deg(fovy), aspect, znear, zfar);

        OPENGL_TO_WGPU_MATIX * proj * view
    }

    pub fn get_layout(&self) -> &BindGroupLayout {
        &self.layout
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj_with_matrix(&mut self, eye: Point3<f32>, matrix: Matrix4<f32>) {
        self.view_position = eye.to_homogeneous().into();
        self.view_proj = matrix.into();
    }
}
