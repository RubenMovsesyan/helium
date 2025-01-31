use cgmath::{Matrix3, Matrix4, One, Quaternion, Vector3};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

use super::vertex::Vertex;

#[derive(Debug)]
pub struct Instance {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
}

impl Default for Instance {
    fn default() -> Self {
        Self {
            position: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            rotation: Quaternion::one(),
        }
    }
}

pub const INSTANCE_RAW_SIZE: usize = std::mem::size_of::<InstanceRaw>();

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
}

#[allow(unused)]
impl Instance {
    pub fn new(position: Vector3<f32>, rotation: Quaternion<f32>) -> Self {
        Self { position, rotation }
    }

    pub fn to_raw(&self) -> InstanceRaw {
        let model =
            (Matrix4::from_translation(self.position) * Matrix4::from(self.rotation)).into();
        InstanceRaw {
            model,
            normal: Matrix3::from(self.rotation).into(),
        }
    }
}

impl Vertex for InstanceRaw {
    fn desc() -> VertexBufferLayout<'static> {
        use std::mem;

        VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                // a mat4 taeks up 4 slots because it is 4 vecs
                // Start at shader location 5 to avoid conflicts
                VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as BufferAddress,
                    shader_location: 6,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as BufferAddress,
                    shader_location: 7,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as BufferAddress,
                    shader_location: 8,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as BufferAddress,
                    shader_location: 9,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as BufferAddress,
                    shader_location: 10,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as BufferAddress,
                    shader_location: 11,
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}
