use std::{error::Error, fmt::Display, fs};

pub enum LoaderResult {
    /// Try loading again.
    ///
    /// The resource is **currently** not available, for example still loading,
    /// and will be available after future call(s) with the **same** path.
    Again,
    /// The resource loaded.
    Bytes(Vec<u8>),
    /// An error occurred trying to load the specified resource.
    Err(BytesLoaderErr),
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
        Err(err) => LoaderResult::Err(match err.kind() {
            std::io::ErrorKind::NotFound => BytesLoaderErr::NotFound,
            _ => BytesLoaderErr::Unknown(format!("{}", err)),
        }),
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

#[derive(Debug)]
pub enum BytesLoaderErr {
    NotFound,
    Unknown(String),
}

impl Error for BytesLoaderErr {}

impl Display for BytesLoaderErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BytesLoaderErr::NotFound => f.write_str("not found"),
            BytesLoaderErr::Unknown(msg) => write!(f, "unknown error: {}", msg),
        }
    }
}
