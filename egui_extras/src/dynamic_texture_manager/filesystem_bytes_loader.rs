use std::{error::Error, fs};

#[cfg(not(target_arch = "wasm32"))]
pub fn filesystem_bytes_loader(url: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    fs::read(url).map_err(|e| e.into())
}
