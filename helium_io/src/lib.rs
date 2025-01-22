use log::*;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Lines, Read, Result},
    path::{Path, PathBuf},
};

pub fn read_lines<P>(file_path: P) -> Result<Lines<BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(file_path)?;
    Ok(BufReader::new(file).lines())
}

pub fn load_shader<P>(file_path: P) -> std::result::Result<String, io::Error>
where
    P: AsRef<Path>,
{
    let module_path = PathBuf::from(file_path.as_ref());
    info!("Loading Shader: {:?}", module_path);
    if !module_path.is_file() {
        error!("Shader not found: {:?}", module_path);
        panic!("Shader not found: {:?}", module_path);
    }

    let mut module_source = String::new();
    BufReader::new(File::open(&module_path)?).read_to_string(&mut module_source)?;
    let mut module_string = String::new();

    let first_line = module_source.lines().next().unwrap();
    if first_line.starts_with("//!include") {
        for include in first_line.split_whitespace().skip(1) {
            info!("Old file path: {:?}", module_path);
            let mut new_path = module_path.parent().unwrap().join(include);
            new_path.set_extension("wgsl");
            module_string.push_str(&load_shader(new_path).unwrap());
        }
    }

    module_string.push_str(&module_source);
    Ok(module_string)
}
