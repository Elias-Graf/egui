use std::fs;

use super::{BytesLoader, LoaderResult};

/// Loads the requested resource URL using [`std::fs`].
pub struct FsBytesLoader;

impl FsBytesLoader {
    pub fn new() -> Self {
        Self
    }
}

impl BytesLoader for FsBytesLoader {
    fn load(&mut self, url: &str) -> LoaderResult {
        match fs::read(url) {
            Err(err) => match err.kind() {
                _ => LoaderResult::Err(format!("{}", err)),
            },
            Ok(bytes) => LoaderResult::Bytes(bytes),
        }
    }
}
