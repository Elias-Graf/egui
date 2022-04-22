#[cfg(not(target_arch = "wasm32"))]
mod fs_bytes_loader;

#[cfg(feature = "http")]
mod http_bytes_loader;

#[cfg(not(target_arch = "wasm32"))]
pub use fs_bytes_loader::FsBytesLoader;

#[cfg(feature = "http")]
pub use http_bytes_loader::HttpBytesLoader;

/// Values that may be returned by a [`BytesLoader`].
pub enum LoaderResult {
    /// Try loading again.
    ///
    /// The resource is **currently** not available, for example still loading,
    /// and will be available after future call(s) with the **same** path.
    Again,
    /// The resource loaded.
    Bytes(Vec<u8>),
    /// An error occurred trying to load the specified resource.
    Err(String),
}

/// Used to load a resource by a given URL.
pub trait BytesLoader {
    fn load(&mut self, url: &str) -> LoaderResult;
}
