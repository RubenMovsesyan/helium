use std::ops::Range;

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device,
};

use super::{
    // instance::{Instance, InstanceRaw},
    model_vertex::ModelVertex,
};

#[allow(unused)]
pub struct Mesh {
    name: String,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    // instance_buffer: Buffer,
    num_elements: u32,
    // num_instances: u32,
    instances: Range<u32>,
    material: Option<usize>,
}

impl Mesh {
    // pub fn get_num_instances(&self) -> u32 {
    //     self.num_instances
    // }

    pub fn get_vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }

    // pub fn get_instance_buffer(&self) -> &Buffer {
    //     &self.instance_buffer
    // }

    pub fn get_instances(&self) -> Range<u32> {
        self.instances.clone()
    }

    pub fn get_num_instances(&self) -> u32 {
        self.instances.end - self.instances.start
    }

    pub fn set_instances(&mut self, instances: Range<u32>) {
        self.instances = instances;
    }

    pub fn get_index_buffer(&self) -> &Buffer {
        &self.index_buffer
    }

    pub fn get_num_elements(&self) -> u32 {
        self.num_elements
    }

    pub fn get_material_index(&self) -> Option<&usize> {
        self.material.as_ref()
    }

    pub fn new(
        name: String,
        vertices: Vec<ModelVertex>,
        indices: Vec<u32>,
        device: &Device,
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(&(name.clone() + " Vertex Buffer")),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(&(name.clone() + " Index Buffer")),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });

        // let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
        //     label: Some(&(name.clone() + " Instance Buffer")),
        //     // Create an instance buffer with only 1 instance
        //     contents: bytemuck::cast_slice(&[Instance::default().to_raw()]),
        //     usage: BufferUsages::VERTEX,
        // });

        Self {
            name,
            vertex_buffer,
            // instance_buffer,
            index_buffer,
            num_elements: indices.len() as u32,
            // num_instances: 1,
            instances: 0..1,
            material: None,
        }
    }

    pub fn set_material(&mut self, material_index: Option<usize>) {
        self.material = material_index;
    }
}
