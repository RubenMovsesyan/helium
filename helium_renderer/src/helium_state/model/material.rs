use helium_io::read_lines;
use log::*;
use std::{fs, io, path::Path};
use wgpu::{Device, Queue};

use crate::helium_state::helium_texture::HeliumTexture;

pub struct Material {
    name: String,
    diffuse_texture: Option<HeliumTexture>,
}

impl Material {
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_diffuse_texture(&self) -> Option<&HeliumTexture> {
        self.diffuse_texture.as_ref()
    }
}

pub fn load_materials<P>(
    file_path: P,
    device: &Device,
    queue: &Queue,
) -> Result<Vec<Material>, io::Error>
where
    P: AsRef<Path>,
{
    info!("Loading Material: {:?}", file_path.as_ref());
    let lines = read_lines(file_path.as_ref())?;

    let mut current_material: Option<String> = None;
    let mut materials: Vec<Material> = Vec::new();
    for line in lines.map_while(Result::ok) {
        let line_split = line.split_whitespace().collect::<Vec<_>>();
        if line_split.len() < 1 {
            continue;
        }

        match line_split[0] {
            "newmtl" => {
                current_material = Some(line_split[1].to_string());
            }
            "map_Kd" => {
                let new_path = file_path.as_ref().parent().unwrap().join(line_split[1]);
                info!("Texture Path: {:?}", new_path);
                let file_contents = fs::read(new_path).unwrap();
                let texture = HeliumTexture::from_bytes(device, queue, &file_contents).unwrap();

                materials.push(Material {
                    name: current_material.take().unwrap(),
                    diffuse_texture: Some(texture),
                });
            }
            _ => {}
        }
    }

    Ok(materials)
}
