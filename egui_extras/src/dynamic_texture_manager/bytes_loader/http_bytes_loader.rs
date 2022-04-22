use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use ehttp::{fetch, Request, Response};

use super::{BytesLoader, LoaderResult};

/// Loads the requested resource URL using [`ehhp`].
pub struct HttpBytesLoader {
    responses: Arc<Mutex<HashMap<String, Response>>>,
}

impl HttpBytesLoader {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl BytesLoader for HttpBytesLoader {
    fn load(&mut self, url: &str) -> LoaderResult {
        if let Some(result) = self.responses.lock().unwrap().remove(url) {
            if !result.ok {
                return LoaderResult::Err(result.status_text);
            }

            return LoaderResult::Bytes(result.bytes);
        }

        let request = Request::get(url);
        let url = url.to_owned();

        let responses = self.responses.clone();

        fetch(request, move |result| {
            responses.lock().unwrap().insert(url, result.unwrap());
        });

        return LoaderResult::Again;
    }
}
