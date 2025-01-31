use cgmath::{Vector3, Zero};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, Device, Queue,
    ShaderStages,
};

#[allow(unused_imports)]
use log::*;

pub struct Lights {
    lights: Vec<Light>,
    buffer: Option<Buffer>,
    bind_group: Option<BindGroup>,
    pub update_flag: bool,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct LightRaw {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Lights {
    pub fn new() -> Self {
        Self {
            lights: Vec::new(),
            buffer: None,
            bind_group: None,
            update_flag: false,
        }
    }

    pub fn add_light(&mut self, light: &mut Light, device: &Device) {
        light.index = self.lights.len();
        self.lights.push(light.clone());
        self.adjust_buffer(device);
    }

    // HACK: This needs to be fixed in a much better way
    pub fn update_light(&mut self, light: &Light, queue: &Queue) {
        use std::mem;
        let index = light.index * mem::size_of::<LightRaw>();

        queue.write_buffer(
            self.buffer.as_ref().unwrap(),
            index as u64,
            bytemuck::cast_slice(&[light.to_raw()]),
        );
    }

    pub fn get_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Lights Bind Group"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }

    pub fn get_bind_group(&self) -> &BindGroup {
        self.bind_group.as_ref().unwrap()
    }

    pub fn get_buffer(&self) -> &Buffer {
        self.buffer.as_ref().unwrap()
    }

    /// Converts the lights vector into a storage buffer to be accessed
    /// On the GPU
    /// Only use when adding or removing lights because it reconstructs the buffer
    pub fn adjust_buffer(&mut self, device: &Device) {
        let mut light_buffer = Vec::new();

        for light in self.lights.iter() {
            light_buffer.push(LightRaw {
                position: [light.position.x, light.position.y, light.position.z],
                color: [light.color.0, light.color.1, light.color.2],
            });
        }

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Lights Buffer"),
            contents: bytemuck::cast_slice(&light_buffer),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        self.buffer = Some(buffer);

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Lights Bind Group"),
            layout: &Self::get_bind_group_layout(device),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: self.buffer.as_ref().unwrap().as_entire_binding(),
            }],
        });

        self.bind_group = Some(bind_group);
    }
}

#[derive(Clone, Copy)]
pub struct Light {
    position: Vector3<f32>,
    color: (f32, f32, f32),
    pub index: usize,
}

impl Light {
    // pub fn new(position: Vector3<f32>, color: (f32, f32, f32)) -> Self {
    //     Self {
    //         position,
    //         color,
    //         index: 0,
    //     }
    // }
    pub fn new(color: (f32, f32, f32)) -> Self {
        Self {
            position: Vector3::zero(),
            color,
            index: 0,
        }
    }

    pub fn update_position(&mut self, position: &Vector3<f32>) -> &mut Self {
        self.position = position.clone();
        self
    }

    pub fn update_color(&mut self, color: (f32, f32, f32)) -> &mut Self {
        self.color = color;
        self
    }

    fn to_raw(&self) -> LightRaw {
        LightRaw {
            position: [self.position.x, self.position.y, self.position.z],
            color: [self.color.0, self.color.1, self.color.2],
        }
    }
}
