use std::error::Error;

#[cfg(target_arch = "wasm32")]
pub fn web_bytes_loader(url: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    use js_sys::ArrayBuffer;
    use js_sys::Uint8Array;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::spawn_local;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, Response};

    let url = url.to_owned();

    // TODO: Figure out how to await async calls.
    // Some quick google searches yielded it would be impossible...

    spawn_local(async move {
        let request = Request::new_with_str(&url).unwrap();

        let window = web_sys::window().unwrap();
        let resp = JsFuture::from(window.fetch_with_request(&request))
            .await
            .unwrap();

        assert!(resp.is_instance_of::<Response>());
        let resp: Response = resp.dyn_into().unwrap();

        let buff = JsFuture::from(resp.array_buffer().unwrap()).await.unwrap();

        assert!(buff.is_instance_of::<ArrayBuffer>());
        let buff = Uint8Array::new(&buff);

        // Well here is the data... ready to use... just need some way to return it...
        buff.to_vec();
    });

    Ok(Vec::new())
}
