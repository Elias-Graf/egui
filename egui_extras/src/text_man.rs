use std::time::SystemTime;

use egui::TextureId;

use crate::dynamic_texture_manager::TextureSize;

pub mod bytes_loader;
pub mod bytes_parser;
pub mod debug_widget;
pub mod dyn_text_man;

// TODO: Convert to Vec2?
// Sadly it's quite hard to work with them, as they are based on f32, and floats
// are not easily hashed in rust. Hashing is necessary for the HashMaps.
pub type TextSize = (usize, usize);

pub trait TextMan {
    /// Load a texture **without** specifying a size.
    ///
    /// The parser will not receive any size, and it may infer it from the
    /// content.
    fn load(&mut self, url: &str) -> TextureId;
    /// Load a texture at the specified size.
    ///
    /// The size is important for things like SVG, which can be rasterized at any
    /// size.
    ///
    /// Note that the parser may ignore the specified size, depending on the textures
    /// type and content.
    fn load_sized(&mut self, url: &str, size: &TextSize) -> TextureId;
    fn unload(&mut self, _url: &str);
    fn unload_sized(&mut self, url: &str, size: &TextureSize);
}

/// # Panics
/// That the texture manager either has no caching, or failed to implement the method.
macro_rules! not_caching_or_not_implemented {
    () => {
        panic!(
            "this texture manager does not cache any textures, or failed to implement this method"
        )
    };
}

/// Interface for displaying debug information about a texture manager.
///
/// # Panics
/// All of the methods have default implementations that panic. Depending on the
/// implementation, many of the methods may not be applicable.
///
/// **This trait is only supposed to be used for debugging purposes**.
pub trait DbgTextMan {
    fn cached_text_ids(&self) -> Vec<(&(String, TextSize), &CachedTexture)> {
        not_caching_or_not_implemented!()
    }
    fn cached_text_id_size(&self) -> usize {
        not_caching_or_not_implemented!()
    }
}

pub struct CachedTexture {
    pub last_used: SystemTime,
    pub text_id: TextureId,
}
