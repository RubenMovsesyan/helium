use cgmath::Vector3;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, Device,
    ShaderStages,
};

pub struct Light {
    position: Vector3<f32>,
    color: (f32, f32, f32),
    uniform: LightUniform,
    buffer: Buffer,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
}

impl Light {
    pub fn new(position: Vector3<f32>, color: (f32, f32, f32), device: &Device) -> Self {
        let uniform = Self::construct_uniform(position, color);
        let buffer = Self::create_light_buffer(uniform, device);
        let bind_group_layout = Self::get_bind_group_layout(device);
        let bind_group = Self::create_bind_group(&bind_group_layout, &buffer, device);

        Self {
            position,
            color,
            uniform,
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn update_position(&mut self, position: Vector3<f32>) -> &mut Self {
        self.position = position;
        self
    }

    pub fn update_color(&mut self, color: (f32, f32, f32)) -> &mut Self {
        self.color = color;
        self
    }

    pub fn construct_uniform(position: Vector3<f32>, color: (f32, f32, f32)) -> LightUniform {
        LightUniform {
            position: [position.x, position.y, position.z],
            _padding: 0,
            color: [color.0, color.1, color.2],
            _color_padding: 0,
        }
    }

    pub fn create_light_buffer(uniform: LightUniform, device: &Device) -> Buffer {
        device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        })
    }

    pub fn get_bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn get_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
            label: Some("Light bind group layout"),
        })
    }

    pub fn create_bind_group(
        bind_group_layout: &BindGroupLayout,
        light_buffer: &Buffer,
        device: &Device,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: Some("Light bind group"),
        })
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    position: [f32; 3],
    _padding: u32,
    color: [f32; 3],
    _color_padding: u32,
}
