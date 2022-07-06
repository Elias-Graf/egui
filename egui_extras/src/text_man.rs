use std::{borrow::Borrow, collections::HashMap, hash::Hash, ops::ControlFlow, sync::Arc};

use egui::{Color32, ColorImage, TextureFilter, TextureId};

use crate::dynamic_texture_manager::TextureSize;

use self::{bytes_loader::BytesLoader, bytes_parser::BytesParser};

pub mod bytes_loader;
pub mod bytes_parser;

pub type TextSize = (usize, usize);

pub trait TextMan {
    /// Load a texture **without** specifying a size.
    ///
    /// **Given that a size is needed for caching, size (0, 0) will be used.**
    ///
    /// The parser will not receive any size, and it may infer it from the
    /// content.
    fn load(&mut self, url: &str) -> TextureId {
        self.load_sized(url, &(0, 0))
    }
    /// Load a texture at the specified size.
    ///
    /// The size is important for things like SVG, which can be rasterized at any
    /// size.
    ///
    /// Note that the parser may ignore the specified size, depending on the textures
    /// type and content.
    fn load_sized(&mut self, url: &str, size: &TextSize) -> TextureId;
    /// Unloads the saved bytes of the texture.
    fn unload(&mut self, _url: &str) {
        // TODO: determine if all sizes of the particular texture should be cleared.
        todo!("determine if all sizes of the particular texture should be cleared.");
    }
    fn unload_sized(&mut self, url: &str, size: &TextureSize);
}

pub struct DynTextMan {
    bytes_loader: Box<dyn BytesLoader>,
    bytes_parser: Box<dyn BytesParser>,
    internal_text_man: Arc<egui::mutex::RwLock<egui::epaint::textures::TextureManager>>,
    /// TODO: Would be better as a constant, but has to be obtained through the
    /// [`egui::epaint::textures::TextureManager`].
    placeholder_text_id: TextureId,
    text_id_cache: HashMap<(String, TextSize), TextureId>,
}

impl DynTextMan {
    fn alloc(&self, name: String, text: ColorImage) -> TextureId {
        Self::alloc_in(self.internal_text_man.clone(), name, text)
    }

    /// Allocate a given [`ColorImage`] in a given [`egui::epaint::textures::TextureManager`].
    fn alloc_in(
        egui_text_man: Arc<egui::mutex::RwLock<egui::epaint::textures::TextureManager>>,
        name: String,
        text: ColorImage,
    ) -> TextureId {
        egui_text_man
            .write()
            .alloc(name, text.into(), TextureFilter::Nearest)
    }

    pub fn load(&mut self, url: &str, size: &TextSize) -> TextureId {
        if let Some(text_id) = self.text_id_cache.get(&(url, size) as &dyn TextIdCacheKey) {
            return text_id.clone();
        }

        let file_ext = match self.try_get_file_ext(url) {
            ControlFlow::Continue(file_ext) => file_ext,
            ControlFlow::Break(text_id) => return text_id,
        };

        let bytes = match self.bytes_loader.load(file_ext) {
            bytes_loader::LoaderResult::Again => todo!(),
            bytes_loader::LoaderResult::Bytes(bytes) => bytes,
            bytes_loader::LoaderResult::Err(_) => todo!(),
        };

        let text = self.bytes_parser.parse(&bytes, size);
        let texture_id = self.alloc(url.to_string(), text);

        self.text_id_cache
            .insert((url.to_string(), size.clone()), texture_id.clone());

        texture_id
    }

    pub fn new(
        internal_text_man: Arc<egui::mutex::RwLock<egui::epaint::textures::TextureManager>>,
        bytes_loader: Box<dyn BytesLoader>,
        bytes_parser: Box<dyn BytesParser>,
    ) -> Self {
        let placeholder_text_id = Self::alloc_in(
            internal_text_man.clone(),
            "<temporary texture>".to_owned(),
            ColorImage::new([1, 1], Color32::TRANSPARENT),
        );

        Self {
            bytes_loader,
            bytes_parser,
            internal_text_man,
            placeholder_text_id,
            text_id_cache: HashMap::new(),
        }
    }

    fn try_get_file_ext<'a>(&self, url: &'a str) -> ControlFlow<TextureId, &'a str> {
        if let Some(ext) = std::path::Path::new(url).extension() {
            return ControlFlow::Continue(ext.to_str().unwrap());
        }

        tracing::error!(
            "texture url {} is missing extension, using placeholder texture",
            url
        );

        // TODO: Test this
        return ControlFlow::Break(self.placeholder_text_id);
    }

    pub fn unload(&mut self, url: &str, size: &TextSize) {
        self.text_id_cache
            .remove(&(url, size) as &dyn TextIdCacheKey);
    }
}

impl TextMan for DynTextMan {
    fn load_sized(&mut self, url: &str, size: &TextSize) -> TextureId {
        DynTextMan::load(self, url, size)
    }

    fn unload_sized(&mut self, url: &str, size: &TextureSize) {
        DynTextMan::unload(self, url, size)
    }
}

/// Allows indexing the hashmap with two key (url and size), without having to copy
/// the string values.
/// Original idea from: https://stackoverflow.com/a/45795699/10315665.
trait TextIdCacheKey {
    fn url(&self) -> &str;
    fn size(&self) -> &TextSize;
}

impl<'a> Borrow<dyn TextIdCacheKey + 'a> for (String, TextSize) {
    fn borrow(&self) -> &(dyn TextIdCacheKey + 'a) {
        self
    }
}

impl Hash for dyn TextIdCacheKey + '_ {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.url().hash(state);
        self.size().hash(state);
    }
}

impl PartialEq for dyn TextIdCacheKey + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.url() == other.url() && self.size() == other.size()
    }
}

impl Eq for dyn TextIdCacheKey + '_ {}

impl TextIdCacheKey for (String, TextSize) {
    fn url(&self) -> &str {
        &self.0
    }

    fn size(&self) -> &TextSize {
        &self.1
    }
}

impl TextIdCacheKey for (&str, &TextSize) {
    fn url(&self) -> &str {
        self.0
    }

    fn size(&self) -> &TextSize {
        self.1
    }
}
