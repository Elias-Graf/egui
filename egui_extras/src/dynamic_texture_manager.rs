use std::{collections::HashMap, error::Error, sync::Arc};

use egui::{epaint, ColorImage, TextureId};

mod filesystem_bytes_loader;
mod svg_bytes_parser;
mod web_bytes_loader;

#[cfg(not(target_arch = "wasm32"))]
pub use filesystem_bytes_loader::filesystem_bytes_loader;

#[cfg(feature = "svg")]
pub use svg_bytes_parser::SVG_BYTES_PARSER;

#[cfg(target_arch = "wasm32")]
pub use web_bytes_loader::web_bytes_loader;

pub type TextureURL = String;
pub type TextureSize = (usize, usize);

// TODO: Error handling

/// Used to load a resource with a given URL.
///
/// May load from resources the filesystem, fetch a resource over the web, ...
pub type BytesLoader = fn(url: &str) -> Result<Vec<u8>, Box<dyn Error>>;

/// Parses given bytes to a color image.
///
/// * `Arc<[u8]>` - bytes to be parsed
/// * [`Size`] - the target for the processing - useful when rasterizing
pub type BytesParser = fn(Arc<[u8]>, Option<&TextureSize>) -> ColorImage;

pub struct DynamicTextureManager {
    tex_manager: Arc<egui::mutex::RwLock<epaint::TextureManager>>,
    bytes_cache: HashMap<TextureURL, Arc<[u8]>>,
    texture_id_cache: HashMap<(TextureURL, TextureSize), TextureId>,
    // TODO: Currently the system is only intended to be used with a single loader,
    // but it could very well be extended to use multiple. One would need to implement
    // some sort of mechanism to determine what loader loads what.
    // I would suggest returning `Option<Result<...>>` from the loader, and `None`
    // would mean, that it couldn't figure out what to do with the given URL.
    bytes_loader: BytesLoader,
    bytes_parsers: HashMap<String, BytesParser>,
}

impl DynamicTextureManager {
    pub fn new(
        tex_manager: Arc<egui::mutex::RwLock<egui::epaint::textures::TextureManager>>,
    ) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let bytes_loader: BytesLoader = filesystem_bytes_loader;
        #[cfg(target_arch = "wasm32")]
        let bytes_loader: BytesLoader = web_bytes_loader;

        let mut bytes_parsers = HashMap::new();

        // TODO: What would be the default parser?
        #[cfg(feature = "svg")]
        bytes_parsers.insert("svg".to_owned(), SVG_BYTES_PARSER);

        Self {
            tex_manager,
            bytes_cache: HashMap::new(),
            texture_id_cache: HashMap::new(),
            bytes_loader,
            bytes_parsers,
        }
    }

    /// Add a new parser for a given file (extension).
    pub fn set_byte_parser_for(&mut self, extension: &str, parser: BytesParser) {
        self.bytes_parsers.insert(extension.to_owned(), parser);
    }

    /// Load an image **without** specifying a size. Also see [`Self::load_sized()`].
    ///
    /// **Given that a size is needed for caching, size (0, 0) will be used.**
    ///
    /// The parser will not receive any size, and will be asked to infer it from
    /// the files content.
    pub fn load(&mut self, path: &str) -> TextureId {
        self.internal_load(path, None)
    }

    /// Load an image and size it accordingly. Also see [`Self::load()`].
    ///
    /// Depending on the filetype and it's parser, the size may be completely ignored.
    /// For other parsers, for example SVG, this attribute determines at what size
    /// the image will be rasterized.
    pub fn load_sized(&mut self, path: &str, size: &TextureSize) -> egui::epaint::TextureId {
        self.internal_load(path, Some(size))
    }

    /// "Completely" unloads the bytes of the image. Also see [`Self::unload_sized()`].
    ///
    /// If a sized version was allocated, it might sill remain in that cache,
    /// especially if multiple sizes were requested.
    /// If unloading is important, make sure to explicitly unload all (sized) versions
    /// with [`Self::unload_sized()`].
    pub fn unload(&mut self, path: &str) {
        self.bytes_cache.remove(path);
        // TODO: figure out if this can be done without cloning.
        self.texture_id_cache.remove(&(path.to_owned(), (0, 0)));
    }

    /// Unloads **only** a particular size of an image. Also see [`Self::unload()`].
    ///
    /// The underlying bytes of the image will remain cached.
    pub fn unload_sized(&mut self, path: &str, size: &TextureSize) {
        // TODO: figure out if this can be done without cloning.
        self.texture_id_cache.remove(&(path.to_owned(), *size));
    }

    fn internal_load(&mut self, path: &str, size: Option<&TextureSize>) -> TextureId {
        // Textures that don't have an associated size will be cached with the
        // values (0, 0).
        let size_or_zero = *(size.clone()).get_or_insert(&(0, 0));

        // TODO: figure out if this can be done without cloning.
        if let Some(cached_texture_id) =
            self.texture_id_cache.get(&(path.to_owned(), *size_or_zero))
        {
            return cached_texture_id.clone();
        }

        // TODO: Error handling
        let file_ext = std::path::Path::new(path)
            .extension()
            .unwrap()
            .to_str()
            .unwrap();

        // TODO: Error handling
        let bytes = self.load_bytes(path).unwrap();

        let image = self.parse_bytes(bytes, file_ext, size);

        let texture_id = self
            .tex_manager
            .write()
            .alloc(path.to_owned(), image.into());
        let texture_id = texture_id;

        self.texture_id_cache
            .insert((path.to_owned(), *size_or_zero), texture_id.clone());

        texture_id
    }

    fn parse_bytes(
        &mut self,
        bytes: Arc<[u8]>,
        ext: &str,
        size: Option<&TextureSize>,
    ) -> ColorImage {
        let parser = self
            .bytes_parsers
            .get(ext)
            .expect(&format!("no parser registered for extension '{}'", ext));

        parser(bytes, size)
    }

    fn load_bytes(&mut self, path: &str) -> Result<Arc<[u8]>, Box<dyn Error>> {
        if let Some(cached_bytes) = self.bytes_cache.get(path) {
            return Ok(cached_bytes.clone());
        }

        let bytes: Arc<[u8]> = (self.bytes_loader)(path)?.into();

        self.bytes_cache.insert(path.to_owned(), bytes.clone());

        Ok(bytes)
    }
}
