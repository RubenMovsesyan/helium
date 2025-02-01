use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

use super::vertex::Vertex;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    position: [f32; 3],
    uv_coords: [f32; 2],
    normal_vec: [f32; 3],
}

impl ModelVertex {
    pub fn new<PN, UV>(position: PN, uv_coords: UV, normal_vec: PN) -> Self
    where
        PN: Into<[f32; 3]>,
        UV: Into<[f32; 2]>,
    {
        Self {
            position: position.into(),
            uv_coords: uv_coords.into(),
            normal_vec: normal_vec.into(),
        }
    }
}

impl Vertex for ModelVertex {
    fn desc() -> VertexBufferLayout<'static> {
        use std::mem;
        VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                // Position
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                // UV coordinates
                VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
                // Normal Vector
                VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}
