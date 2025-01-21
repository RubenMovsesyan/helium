use std::sync::Arc;

use cgmath::{Quaternion, Vector3};
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BufferBindingType, Device, ShaderStages,
};

#[derive(Default, Debug)]
pub struct Transform {
    translation: Option<Vector3<f32>>,
    rotation: Option<Quaternion<f32>>,
    scale: Option<Vector3<f32>>,
    // wgpu vars
}

// const TRANSFORM_BIND_GROUP_LAYOUT_DESCRIPTOR: BindGroupLayoutDescriptor =
// BindGroupLayoutDescriptor {
//     label: Some("Transformation Bind Group Layout Descriptor"),
//     entries: &[BindGroupLayoutEntry {
//         binding: 0,
//         visibility: ShaderStages::VERTEX,
//         ty: BindingType::Buffer {
//             ty: BufferBindingType::Uni
//         }
//     }],
// };

// pub struct TransformBindGroupLayout(Arc<BindGroupLayout>);

// impl TransformBindGroupLayout {
//     pub fn from_device(device: &Device) -> Self {
//         Self(Arc::new(device.create_bind_group_layout(
//             &HELIUM_TEXTURE_BIND_GROUP_LAYOUT_DESCRIPTOR,
//         )))
//     }
// }
