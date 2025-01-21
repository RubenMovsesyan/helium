use std::ops::Range;

use super::material::Material;
use super::mesh::Mesh;
use wgpu::{BindGroup, IndexFormat, RenderPass};

pub trait DrawModel<'a> {
    fn draw_mesh(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        camera_bind_group: &'a BindGroup,
    );

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        instances: Range<u32>,
        camera_bind_group: &'a BindGroup,
    );
}

impl<'a, 'b> DrawModel<'b> for RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        camera_bind_group: &'b BindGroup,
    ) {
        self.draw_mesh_instanced(
            mesh,
            material,
            0..mesh.get_num_instances(),
            camera_bind_group,
        );
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        instances: Range<u32>,
        camera_bind_group: &'b BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.get_vertex_buffer().slice(..));
        self.set_vertex_buffer(1, mesh.get_instance_buffer().slice(..));
        self.set_index_buffer(mesh.get_index_buffer().slice(..), IndexFormat::Uint32);
        self.set_bind_group(
            0,
            material.get_diffuse_texture().unwrap().get_bind_group(),
            &[],
        );
        self.set_bind_group(1, camera_bind_group, &[]);
        self.draw_indexed(0..mesh.get_num_elements(), 0, instances);
    }
}
