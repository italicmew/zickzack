use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, // Or `Aes128Gcm`
};

// The wasm-pack uses wasm-bindgen to build and generate JavaScript binding file.
// Import the wasm-bindgen crate.
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Document, Event, Headers, HtmlButtonElement, HtmlTextAreaElement, Location, Request,
    RequestInit, RequestMode, Response, Window, HtmlElement
};

#[wasm_bindgen]
pub fn show_alert() {
    let window: Window = web_sys::window().expect("no global `window` exists");
    let document: Document = window.document().expect("should have a document on window");
    if let Some(alert) = document.get_element_by_id("alerts") {
        if let Some(html_element) = alert.dyn_ref::<HtmlElement>() {
            html_element.style().set_property("display", "block").unwrap();
        }
    }
}


#[wasm_bindgen]
pub fn setup_submit_listener() -> Result<(), JsValue> {
    let w: Window =
        web_sys::window().ok_or_else(|| JsValue::from_str("No global `window` exists"))?;
    let document: Document = w
        .document()
        .ok_or_else(|| JsValue::from_str("No document available"))?;

    let btn = match document.get_element_by_id("submitBtn") {
        Some(button) => match button.dyn_into::<HtmlButtonElement>() {
            Ok(btn) => Ok(btn),
            Err(_) => Err("Failed to cast element to HtmlButtonElement"),
        },
        None => Err("Element with ID 'submitBtn' not found"),
    };

    let closure = Closure::wrap(Box::new(move |event: Event| {
        // Prevent the default form submit action
        event.prevent_default();

        let val = document
            .get_element_by_id("editor")
            .and_then(|el| el.dyn_into::<HtmlTextAreaElement>().ok())
            .map(|text_area| text_area.value());

        // Create the request
        let mut opts = RequestInit::new();
        let headers = Headers::new().unwrap();
        headers
            .append("Content-Type", "application/x-www-form-urlencoded")
            .unwrap();

        let content = format!("content={}", val.unwrap_or_default());
        opts.method("POST")
            .mode(RequestMode::Cors)
            .headers(&headers)
            .body(Some(&JsValue::from_str(&content)));

        let path = format!("/editor{}", w.location().pathname().unwrap());

        match Request::new_with_str_and_init(&path, &opts) {
            Ok(request) => {
                let _ = w.fetch_with_request(&request);
                show_alert();
            }
            Err(request_error) => {
                web_sys::console::log_1(&JsValue::from_str(&format!(
                    "Failed to create a new request: {:?}",
                    request_error
                )));
            }
        }
    }) as Box<dyn FnMut(_)>);

    match btn {
        Ok(btn) => {
            btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        }
        Err(e) => {
            web_sys::console::log_1(&JsValue::from_str(&format!("content={}", e)));
        }
    }

    closure.forget(); // Forget the closure to prevent it from being cleaned up

    Ok(())
}

#[wasm_bindgen(start)]
async fn main() -> Result<(), JsValue> {
    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.

    let key = Aes256Gcm::generate_key(OsRng);
    web_sys::console::log_1(&JsValue::from_str(&format!("content=")));

    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
    let ciphertext = cipher
        .encrypt(&nonce, b"plaintext message".as_ref())
        .unwrap();
    let plaintext = cipher.decrypt(&nonce, ciphertext.as_ref()).unwrap();

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    // Manufacture the element we're gonna append
    let val = document.create_element("p")?;
    val.set_inner_html(
        format!(
            "--encrypted text: {}",
            std::str::from_utf8(&plaintext).unwrap()
        )
        .as_str(),
    );

    body.append_child(&val)?;

    let text_area = document
        .get_element_by_id("editor")
        .expect("Element not found.");

    let location: Location = window.location();
    let current_path = location
        .pathname()
        .map_err(|_| JsValue::from_str("Failed to get pathname"))?;
    let editor_url = format!("/editor{}", current_path);
    let promise = window.fetch_with_str(&editor_url);
    let response: Response = JsFuture::from(promise).await?.dyn_into()?;
    let text: String = JsFuture::from(response.text()?)
        .await?
        .as_string()
        .unwrap_or_default();

    text_area.set_text_content(Some(&text));
    setup_submit_listener().unwrap();

    Ok(())
}

// hello world
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    return a + b;
}