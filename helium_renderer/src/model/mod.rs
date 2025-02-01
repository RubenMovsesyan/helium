pub mod draw_model;
pub mod instance;
pub mod material;
pub mod mesh;
pub mod model_vertex;
pub mod vertex;

// Std
use std::{io::Error, ops::Range, path::Path};

use model_vertex::ModelVertex;
// wgpu imports
use wgpu::{Device, Queue};

// logging
use log::*;

// custom imports
use helium_io::read_lines;
use material::{load_materials, Material};
use mesh::Mesh;

pub struct Model {
    meshes: Vec<Mesh>,
    materials: Vec<Material>,
}

impl Model {
    pub fn get_meshes(&self) -> &[Mesh] {
        &self.meshes
    }

    pub fn get_materials(&self) -> &[Material] {
        &self.materials
    }

    pub fn set_instances(&mut self, instances: Range<u32>) {
        for mesh in self.meshes.iter_mut() {
            mesh.set_instances(instances.clone());
        }
    }

    pub fn get_instances(&self) -> Range<u32> {
        if !self.meshes.is_empty() {
            return self.meshes[0].get_instances();
        }

        0..1
    }

    pub fn get_num_instances(&self) -> u32 {
        if !self.meshes.is_empty() {
            return self.meshes[0].get_num_instances();
        }

        0
    }

    pub fn from_obj<P>(file_path: P, device: &Device, queue: &Queue) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        info!("Loading Object: {:?}", file_path.as_ref());
        let mut mesh_name: Option<String> = None;
        let mut vertices: Vec<(f32, f32, f32)> = Vec::new();
        let mut uv_coords: Vec<(f32, f32)> = Vec::new();
        let mut normals: Vec<(f32, f32, f32)> = Vec::new();

        let mut model_vertices: Vec<ModelVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        let mut meshes: Vec<Mesh> = Vec::new();
        let mut materials: Vec<Material> = Vec::new();

        let mut material_index: Option<usize> = None;

        match read_lines(file_path.as_ref()) {
            Ok(lines) => {
                for line in lines.map_while(Result::ok) {
                    let line_split = line.split_whitespace().collect::<Vec<_>>();

                    if line_split.is_empty() {
                        continue;
                    }

                    match line_split[0] {
                        // This is an object
                        "o" => {
                            if let Some(name) = mesh_name.take() {
                                let mut new_mesh = Mesh::new(name, model_vertices, indices, device);
                                new_mesh.set_material(material_index.take());
                                meshes.push(new_mesh);
                                model_vertices = Vec::new();
                                indices = Vec::new();
                            }

                            mesh_name = Some(line_split[1].to_string());
                        }
                        // This is a vertex
                        "v" => {
                            let vertex = (
                                line_split[1].parse::<f32>().unwrap(),
                                line_split[2].parse::<f32>().unwrap(),
                                line_split[3].parse::<f32>().unwrap(),
                            );

                            vertices.push(vertex);
                        }
                        // This is a uv coordinate
                        "vt" => {
                            let uv_coord = (
                                1.0 - line_split[1].parse::<f32>().unwrap(),
                                1.0 - line_split[2].parse::<f32>().unwrap(),
                            );

                            uv_coords.push(uv_coord);
                        }
                        // This is a normal
                        "vn" => {
                            let normal = (
                                line_split[1].parse::<f32>().unwrap(),
                                line_split[2].parse::<f32>().unwrap(),
                                line_split[3].parse::<f32>().unwrap(),
                            );

                            normals.push(normal);
                        }
                        // This is a face
                        "f" => {
                            for vertex_info in line_split[1..=3].iter() {
                                let vertex_info_split =
                                    vertex_info.split('/').collect::<Vec<&str>>();

                                // Get the index of each the vertex, uv, and normal, for each vertex of the face
                                let (vertex_index, uv_index, normal_index) = (
                                    vertex_info_split[0].parse::<usize>().unwrap() - 1,
                                    vertex_info_split[1].parse::<usize>().unwrap() - 1,
                                    vertex_info_split[2].parse::<usize>().unwrap() - 1,
                                );

                                // Add a vertex to the current model based on the face information
                                model_vertices.push(ModelVertex::new(
                                    vertices[vertex_index],
                                    uv_coords[uv_index],
                                    normals[normal_index],
                                ));

                                // WARN: This might be a problem
                                indices.push(model_vertices.len() as u32 - 1);
                            }
                        }
                        // This is a mateiral
                        "mtllib" => {
                            let path_to_material =
                                file_path.as_ref().parent().unwrap().join(line_split[1]);
                            materials.append(
                                &mut load_materials(path_to_material, device, queue).unwrap(),
                            );
                        }
                        // This is the object using the material
                        "usemtl" => {
                            for (index, material) in materials.iter().enumerate() {
                                info!(
                                    "Material: {}, line: {}",
                                    material.get_name().as_str(),
                                    line_split[1]
                                );
                                if material.get_name().as_str() == line_split[1] {
                                    info!("Match!");
                                    material_index = Some(index);
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // Add any remaining meshes in the object file
                if let Some(name) = mesh_name.take() {
                    let mut new_mesh = Mesh::new(name, model_vertices, indices, device);
                    new_mesh.set_material(material_index.take());
                    meshes.push(new_mesh);
                }

                Ok(Self { meshes, materials })
            }
            Err(e) => {
                error!("Error: {}", e);
                Err(e)
            }
        }
    }
}
