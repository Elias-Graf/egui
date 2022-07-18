use std::{borrow::Borrow, collections::HashMap, hash::Hash, sync::Arc, time::SystemTime};

use egui::{ColorImage, TextureFilter, TextureId};

use super::{bytes_loader, bytes_parser, CachedTexture, DbgTextMan, TextMan, TextSize};

pub struct DynTextMan {
    bytes_loader: Box<dyn bytes_loader::BytesLoader>,
    bytes_parser: Box<dyn bytes_parser::BytesParser>,
    internal_text_man: Arc<egui::mutex::RwLock<egui::epaint::textures::TextureManager>>,
    /// TODO: Would be better as a constant, but has to be obtained through the
    /// [`egui::epaint::textures::TextureManager`].
    // placeholder_text_id: TextureId,
    text_id_cache: HashMap<(String, TextSize), CachedTexture>,
    /// Size of the text id cache in bytes.
    text_id_cache_size: usize,
    unload_strategy: UnloadStrategy,
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

    pub fn load(&mut self, url: &str, size: &TextSize) -> Result<TextureId, &'static str> {
        if let Some(CachedTexture { last_used, text_id }) = self
            .text_id_cache
            .get_mut(&(url, size) as &dyn TextIdCacheKey)
        {
            *last_used = SystemTime::now();

            return Ok(text_id.clone());
        }

        // let file_ext = match self.try_get_file_ext(url) {
        //     ControlFlow::Continue(file_ext) => file_ext,
        //     ControlFlow::Break(text_id) => return text_id,
        // };

        let bytes = match self.bytes_loader.load(url) {
            bytes_loader::LoaderResult::Again => todo!(),
            bytes_loader::LoaderResult::Bytes(bytes) => bytes,
            bytes_loader::LoaderResult::Err(err) => {
                eprintln!("{}", err);
                // TODO: figure out how the log_err macro works
                todo!("figure out how the log_err macro works");
            }
        };

        let text = self.bytes_parser.parse(&bytes, size);
        let text_id = self.alloc(url.to_string(), text);
        let text_id_size = byte_size_of_text_id(text_id, &self.internal_text_man.read())?;

        self.text_id_cache_size += text_id_size;
        self.text_id_cache.insert(
            (url.to_owned(), *size),
            CachedTexture {
                last_used: SystemTime::now(),
                text_id,
            },
        );

        self.automatic_unload()?;

        Ok(text_id)
    }

    pub fn new(
        internal_text_man: Arc<egui::mutex::RwLock<egui::epaint::textures::TextureManager>>,
        bytes_loader: Box<dyn bytes_loader::BytesLoader>,
        bytes_parser: Box<dyn bytes_parser::BytesParser>,
        unload_strategy: UnloadStrategy,
    ) -> Self {
        // let placeholder_text_id = Self::alloc_in(
        //     internal_text_man.clone(),
        //     "<temporary texture>".to_owned(),
        //     ColorImage::new([1, 1], Color32::TRANSPARENT),
        // );

        Self {
            bytes_loader,
            bytes_parser,
            internal_text_man,
            // placeholder_text_id,
            text_id_cache: HashMap::new(),
            text_id_cache_size: 0,
            unload_strategy,
        }
    }

    fn automatic_unload(&mut self) -> Result<(), &'static str> {
        let target_cache_size = match self.unload_strategy {
            UnloadStrategy::None => todo!(),
            UnloadStrategy::TargetCacheSize(target_cache_size) => target_cache_size,
        };

        while self.text_id_cache_size > target_cache_size {
            let oldest = self
                .text_id_cache
                .iter()
                .min_by_key(|text| text.1.last_used);

            if let Some(((url, size), _)) = oldest {
                // TODO: Remove this cloning
                // Could technically be done with unsafe code:
                // ```
                // let (url, size) = unsafe {
                //     let url: &String = std::mem::transmute(url);
                //     let size: &TextSize = std::mem::transmute(size);
                //
                //     (url, size)
                // };
                // ```
                // https://stackoverflow.com/a/73013426/10315665
                let url = &url.clone();
                let size = &size.clone();

                self.unload(url, size)?;
            }
        }

        Ok(())
    }

    // fn try_get_file_ext<'a>(&self, url: &'a str) -> ControlFlow<TextureId, &'a str> {
    //     if let Some(ext) = std::path::Path::new(url).extension() {
    //         return ControlFlow::Continue(ext.to_str().unwrap());
    //     }

    //     tracing::error!(
    //         "texture url {} is missing extension, using placeholder texture",
    //         url
    //     );

    //     // TODO: Test this
    //     return ControlFlow::Break(self.placeholder_text_id);
    // }

    pub fn unload(&mut self, url: &str, size: &TextSize) -> Result<(), &'static str> {
        let text = self
            .text_id_cache
            .remove(&(url, size) as &dyn TextIdCacheKey);

        if let Some(CachedTexture { text_id, .. }) = text {
            let mut internal_text_man = self.internal_text_man.write();
            let size = byte_size_of_text_id(text_id, &internal_text_man)?;

            self.text_id_cache_size -= size;

            internal_text_man.free(text_id);
        }

        Ok(())
    }
}

fn byte_size_of_text_id(
    id: TextureId,
    text_man: &egui::epaint::textures::TextureManager,
) -> Result<usize, &'static str> {
    let meta = text_man.meta(id).ok_or_else(|| {
        "could not find texture id that was just allocated,  while trying to read size"
    })?;

    Ok(meta.bytes_used())
}

/// Determines how the [`DynTextMan`] retains textures (or removes them).
pub enum UnloadStrategy {
    /// Does not remove any texture. Unloading has to be done manually.
    None,
    /// Allocates textures up to the specified size (in bytes).
    ///
    /// If the size has been reached, it starts unloading the textures that have
    /// not been accessed the longest.
    ///
    /// Note that the size is a targe, and textures could be allocation so the size
    /// surpasses the target.
    TargetCacheSize(usize),
}

impl TextMan for DynTextMan {
    fn load_sized(&mut self, url: &str, size: &TextSize) -> TextureId {
        // TODO: error handling
        DynTextMan::load(self, url, size).unwrap()
    }

    fn unload_sized(&mut self, url: &str, size: &TextSize) {
        // TODO: error handling
        DynTextMan::unload(self, url, size).unwrap()
    }
}

impl DbgTextMan for DynTextMan {
    fn cached_text_ids(&self) -> Vec<(&(String, TextSize), &CachedTexture)> {
        self.text_id_cache.iter().collect()
    }
    fn cached_text_id_size(&self) -> usize {
        self.text_id_cache_size
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
