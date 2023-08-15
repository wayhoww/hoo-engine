use std::{fs::{File, self}, io::{BufReader, Read}, path::Path};

use crate::hoo_engine;

pub fn load_binary(path: &str) -> Result<Vec<u8>, String> {
    let path = Path::new(&hoo_engine().borrow().get_configs().resources_path).join(path);
    let mut file = File::open(path).or(Err("cannot open file"))?;
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer).expect("buffer overflow");
    Ok(buffer)
}

pub fn load_string(path: &str) -> Result<String, String> {
    let binary = load_binary(path)?;
    String::from_utf8(binary).map_err(|_| "cannot decode utf8".to_string())
}
