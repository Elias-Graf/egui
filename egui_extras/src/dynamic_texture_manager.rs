use std::{collections::HashMap, sync::Arc};

use egui::{epaint, Color32, ColorImage, TextureFilter, TextureId};

pub mod bytes_loader;

#[cfg(feature = "svg")]
mod svg_bytes_parser;

pub use bytes_loader::BytesLoader;

#[cfg(feature = "svg")]
pub use svg_bytes_parser::SVG_BYTES_PARSER;

pub type TextureSize = (usize, usize);

/// Parses given bytes to a color image.
///
/// * `Arc<[u8]>` - bytes to be parsed
/// * [`Size`] - the target for the processing - useful when rasterizing
pub type BytesParser = fn(&[u8], Option<&TextureSize>) -> ColorImage;

/// Dynamically allocate textures using [`Self::load()`], or [`Self::load_sized()`].
///
/// The first time the functions are called the specified path will be loaded using
/// configurable [`BytesLoader`], and afterwords parsed with the configurable [`BytesParser`].
/// After that a cached version of the texture will be immediately returned.
///
/// # Example
///
/// ```rust
/// dynamic_texture_manager.load("assets/example.png");
/// ```
pub struct DynamicTextureManager {
    bytes_cache: HashMap<String, Arc<[u8]>>,
    // TODO: Currently the system is only intended to be used with a single loader,
    // but it could very well be extended to use multiple. One would need to implement
    // some sort of mechanism to determine what loader loads what.
    // I would suggest returning `Option<Result<...>>` from the loader, and `None`
    // would mean, that it couldn't figure out what to do with the given URL.
    bytes_loader: Box<dyn BytesLoader>,
    bytes_parsers: HashMap<String, BytesParser>,
    /// Will be returned as placeholder value
    placeholder_tex_id: TextureId,
    tex_id_cache: HashMap<(String, TextureSize), TextureId>,
    tex_manager: Arc<egui::mutex::RwLock<epaint::TextureManager>>,
}

impl DynamicTextureManager {
    pub fn new(
        tex_manager: Arc<egui::mutex::RwLock<egui::epaint::textures::TextureManager>>,
        bytes_loader: Box<dyn BytesLoader>,
    ) -> Self {
        let placeholder_tex_id = tex_manager.write().alloc(
            "<temporary texture>".to_owned(),
            ColorImage::new([1, 1], Color32::TRANSPARENT).into(),
            TextureFilter::Nearest,
        );

        let mut bytes_parsers = HashMap::new();

        // TODO: currently there is no parser registered if the svg feature is not
        // enabled.
        #[cfg(feature = "svg")]
        bytes_parsers.insert("svg".to_owned(), SVG_BYTES_PARSER);

        Self {
            bytes_cache: HashMap::new(),
            bytes_loader,
            bytes_parsers,
            placeholder_tex_id,
            tex_id_cache: HashMap::new(),
            tex_manager,
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
    pub fn load(&mut self, url: &str) -> TextureId {
        self.internal_load(url, None)
    }

    /// Load an image and size it accordingly. Also see [`Self::load()`].
    ///
    /// Depending on the filetype and it's parser, the size may be completely ignored.
    /// For other parsers, for example SVG, this attribute determines at what size
    /// the image will be rasterized.
    pub fn load_sized(&mut self, url: &str, size: &TextureSize) -> egui::epaint::TextureId {
        self.internal_load(url, Some(size))
    }

    /// "Completely" unloads the bytes of the image. Also see [`Self::unload_sized()`].
    ///
    /// If a sized version was allocated, it might sill remain in that cache,
    /// especially if multiple sizes were requested.
    /// If unloading is important, make sure to explicitly unload all (sized) versions
    /// with [`Self::unload_sized()`].
    pub fn unload(&mut self, url: &str) {
        self.bytes_cache.remove(url);
        // TODO: figure out if this can be done without cloning.
        self.tex_id_cache.remove(&(url.to_owned(), (0, 0)));
    }

    /// Unloads **only** a particular size of an image. Also see [`Self::unload()`].
    ///
    /// The underlying bytes of the image will remain cached.
    pub fn unload_sized(&mut self, url: &str, size: &TextureSize) {
        // TODO: figure out if this can be done without cloning.
        self.tex_id_cache.remove(&(url.to_owned(), *size));
    }

    fn internal_load(&mut self, url: &str, size: Option<&TextureSize>) -> TextureId {
        // Textures that don't have an associated size will be cached with the
        // values (0, 0).
        let size_or_zero = *(size.clone()).get_or_insert(&(0, 0));

        // TODO: figure out if this can be done without cloning.
        if let Some(cached_texture_id) = self.tex_id_cache.get(&(url.to_owned(), *size_or_zero)) {
            return cached_texture_id.clone();
        }

        let file_ext = match std::path::Path::new(url).extension() {
            None => {
                // TODO: Emit an error that the path is invalid to some sort of
                // error log.
                return self.get_and_cache_placeholder_tex_for(url, size_or_zero);
            }
            Some(ext) => ext.to_str().unwrap(),
        };

        let image = match self.bytes_cache.get(url) {
            Some(cached_bytes) => {
                let cached_bytes = cached_bytes.clone();

                self.parse_bytes(&cached_bytes, file_ext, size)
            }
            None => match self.bytes_loader.load(url) {
                bytes_loader::LoaderResult::Again => return self.placeholder_tex_id,
                bytes_loader::LoaderResult::Bytes(bytes) => {
                    self.parse_bytes(&bytes, file_ext, size)
                }
                bytes_loader::LoaderResult::Err(_) => {
                    // TODO: Emit the error to some sort of error log.
                    return self.get_and_cache_placeholder_tex_for(url, size_or_zero);
                }
            },
        };

        let texture_id =
            self.tex_manager
                .write()
                .alloc(url.to_owned(), image.into(), TextureFilter::Nearest);

        self.tex_id_cache
            .insert((url.to_owned(), *size_or_zero), texture_id.clone());

        texture_id
    }

    fn parse_bytes(&mut self, bytes: &[u8], ext: &str, size: Option<&TextureSize>) -> ColorImage {
        let parser = self
            .bytes_parsers
            .get(ext)
            .expect(&format!("no parser registered for extension '{}'", ext));

        parser(bytes, size)
    }

    fn get_and_cache_placeholder_tex_for(&mut self, url: &str, size: &TextureSize) -> TextureId {
        self.tex_id_cache
            .insert((url.to_owned(), size.clone()), self.placeholder_tex_id);
        self.placeholder_tex_id
    }
}
