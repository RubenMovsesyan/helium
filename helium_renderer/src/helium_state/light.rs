use cgmath::Point3;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, Device,
    ShaderStages,
};

pub struct Lights {
    lights: Vec<Light>,
    buffer: Option<Buffer>,
    bind_group: Option<BindGroup>,
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
        }
    }

    pub fn add_light(&mut self, light: Light, device: &Device) {
        self.lights.push(light);
        self.adjust_buffer(device);
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

    /// Converts the lights vector into a storage buffer to be accessed
    /// On the GPU
    fn adjust_buffer(&mut self, device: &Device) {
        let mut light_buffer = Vec::new();

        for light in self.lights.iter() {
            light_buffer.push(LightRaw {
                position: [light.position.x, light.position.y, light.position.z],
                color: [light.color.0, light.color.1, light.color.2],
            });
            // light_buffer.push([
            //     light.position.x,
            //     light.position.y,
            //     light.position.z,
            //     light.color.0,
            //     light.color.1,
            //     light.color.2,
            // ]);
        }

        // let lights_raw: &[f32] = bytemuck::cast_slice(light_buffer.as_flattened());

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

pub struct Light {
    position: Point3<f32>,
    color: (f32, f32, f32),
}

impl Light {
    pub fn new(position: Point3<f32>, color: (f32, f32, f32)) -> Self {
        Self { position, color }
    }

    pub fn update_position(&mut self, position: Point3<f32>) -> &mut Self {
        self.position = position;
        self
    }

    pub fn update_color(&mut self, color: (f32, f32, f32)) -> &mut Self {
        self.color = color;
        self
    }
}
