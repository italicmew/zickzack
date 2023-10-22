use aes_gcm::{
  aead::{Aead, AeadCore, KeyInit, OsRng},
  Aes256Gcm // Or `Aes128Gcm`
};

// The wasm-pack uses wasm-bindgen to build and generate JavaScript binding file.
// Import the wasm-bindgen crate.
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Location, Response, Document, Event, RequestInit, RequestMode, Headers, Request, HtmlTextAreaElement};


#[wasm_bindgen]
pub fn setup_submit_listener() -> Result<(), JsValue> {
    let w: web_sys::Window = web_sys::window().ok_or_else(|| JsValue::from_str("No global `window` exists"))?;
    let document: Document = w.document().ok_or_else(|| JsValue::from_str("No document available"))?;

    let button = document.get_element_by_id("submitBtn").unwrap();
    let btn: web_sys::HtmlButtonElement = button.dyn_into::<web_sys::HtmlButtonElement>()?;

    let closure = Closure::wrap(Box::new(move |event: Event| {
        // Prevent the default form submit action
        event.prevent_default();


        let your_message = document
          .get_element_by_id("editor")
          .unwrap()
          .dyn_into::<HtmlTextAreaElement>()
          .unwrap();

        let val = your_message.value();
        // let text_area = document.get_element_by_id("editor").expect("Element not found.");
        // // Assuming the data you want to send is a string. Modify as per your needs.
        // let textarea: web_sys::HtmlTextAreaElement = text_area.dyn_into().ok().unwrap();
        
        
        // Create the request
        let mut opts = RequestInit::new();
        opts.method("POST");
        opts.mode(RequestMode::Cors);
        let headers = Headers::new().unwrap();
        headers.append("Content-Type", "application/x-www-form-urlencoded").unwrap();
        opts.headers(&headers);

        // opts.body(Some(&JsValue::from_str(format!("content={}", data.unwrap()).as_str())));

        opts.body(Some(&JsValue::from_str(format!("content={}", val).as_str())));
        // web_sys::console::log_1(&JsValue::from_str(format!("content={}", val).as_str()));

        let w = web_sys::window().unwrap();
        let location: Location = w.location();
        let current_path = location.pathname().map_err(|_| JsValue::from_str("Failed to get pathname"));
        let editor_url = format!("/editor{}", current_path.unwrap());
        let request = Request::new_with_str_and_init(editor_url.as_str(), &opts).unwrap();

        let _ = w.fetch_with_request(&request);
    }) as Box<dyn FnMut(_)>);

    btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
    closure.forget();  // Forget the closure to prevent it from being cleaned up

    Ok(())
}



#[wasm_bindgen(start)]
async fn main() -> Result<(), JsValue> {
    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.

    let key = Aes256Gcm::generate_key(OsRng);

    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
    let ciphertext = cipher.encrypt(&nonce, b"plaintext message".as_ref()).unwrap();
    let plaintext = cipher.decrypt(&nonce, ciphertext.as_ref()).unwrap();


    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");


    // Manufacture the element we're gonna append
    let val = document.create_element("p")?;
    val.set_inner_html(format!("encrypted text: {}", std::str::from_utf8(&plaintext).unwrap()).as_str());

    body.append_child(&val)?;

    let text_area = document.get_element_by_id("editor").expect("Element not found.");

    let location: Location = window.location();
    let current_path = location.pathname().map_err(|_| JsValue::from_str("Failed to get pathname"))?;
    let editor_url = format!("/editor{}", current_path);
    let promise = window.fetch_with_str(&editor_url);
    let response: Response = JsFuture::from(promise).await?.dyn_into()?;
    let text: String = JsFuture::from(response.text()?).await?.as_string().unwrap_or_default();

    text_area.set_text_content(Some(&text));
    setup_submit_listener().unwrap();


    Ok(())
}

// Our Add function
// wasm-pack requires "exported" functions
// to include #[wasm_bindgen]
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
  return a + b;
}