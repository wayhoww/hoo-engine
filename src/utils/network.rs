// use wasm_bindgen::*;
// use wasm_bindgen_futures::JsFuture;
// use web_sys::{Request, RequestCache, RequestInit, Response};

// // TODO: 应当使用 cache。现在不用 cache 只是 python3 服务器这个 cache 似乎有问题
// pub async fn get_text_from_url(url: &str) -> Result<String, ()> {
//     let mut init = RequestInit::new();
//     init.cache(RequestCache::NoStore);

//     let request = Request::new_with_str_and_init(url, &init).or(Err(()))?;
//     let response = JsFuture::from(web_sys::window().unwrap().fetch_with_request(&request))
//         .await
//         .or(Err(()))?;
//     let response: Response = response.dyn_into::<Response>().or(Err(()))?;

//     JsFuture::from(response.text().or(Err(()))?)
//         .await
//         .or(Err(()))?
//         .as_string()
//         .ok_or(())
// }
