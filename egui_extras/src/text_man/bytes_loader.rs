use std::fs;

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

pub trait BytesLoader {
    fn load(&self, url: &str) -> LoaderResult;
}

impl<T> BytesLoader for T
where
    T: Fn(&str) -> LoaderResult,
{
    fn load(&self, url: &str) -> LoaderResult {
        self(url)
    }
}

/// Loads the requested URL using [`std::fs`].
pub fn fs_bytes_loader(url: &str) -> LoaderResult {
    match fs::read(url) {
        Ok(bytes) => LoaderResult::Bytes(bytes),
        Err(err) => LoaderResult::Err(format!("failed to load [{}]: '{}'", url, err)),
    }
}

#[cfg(feature = "http")]
use http::*;
#[cfg(feature = "http")]
mod http {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    /// Loads the requested URL using []
    #[derive(Default)]
    pub struct HttpBytesLoader {
        responses: Arc<Mutex<HashMap<String, ehttp::Response>>>,
    }

    impl BytesLoader for HttpBytesLoader {
        fn load(&self, url: &str) -> LoaderResult {
            if let Some(result) = self.responses.lock().unwrap().remove(url) {
                if !result.ok {
                    return LoaderResult::Err(result.status_text);
                }

                return LoaderResult::Bytes(result.bytes);
            }

            let request = ehttp::Request::get(url);
            let responses = self.responses.clone();
            let url = url.to_owned();

            ehttp::fetch(request, move |result| {
                responses.lock().unwrap().insert(url, result.unwrap());
            });

            return LoaderResult::Again;
        }
    }
}
