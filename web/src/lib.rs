use aes_gcm::aead::generic_array::GenericArray;
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
    Document, Event, Headers, HtmlButtonElement, HtmlElement, HtmlTextAreaElement, Location,
    Request, RequestInit, RequestMode, Response, Window,
};

#[wasm_bindgen]
pub fn show_alert() {
    let window: Window = web_sys::window().expect("no global `window` exists");
    let document: Document = window.document().expect("should have a document on window");
    if let Some(alert) = document.get_element_by_id("alerts") {
        if let Some(html_element) = alert.dyn_ref::<HtmlElement>() {
            html_element
                .style()
                .set_property("display", "block")
                .unwrap();
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

fn decrypt_message(
    key: &str,
    nonce: &Vec<u8>,
    ciphertext: &[u8],
) -> Result<Vec<u8>, aes_gcm::aead::Error> {
    if key.as_bytes().len() != 32 {
        return Err(aes_gcm::aead::Error);
    }

    let cipher = Aes256Gcm::new(GenericArray::from_slice(key.as_bytes()));
    let plaintext = cipher.decrypt(&GenericArray::from_slice(nonce), ciphertext);

    plaintext
}

fn encrypt_message(key: &str, text: &str) -> Result<(Vec<u8>, Vec<u8>), aes_gcm::aead::Error> {
    if key.as_bytes().len() != 32 {
        return Err(aes_gcm::aead::Error);
    }

    let cipher = Aes256Gcm::new(GenericArray::from_slice(key.as_bytes()));
    let nonce = Aes256Gcm::generate_nonce(&mut rand::rngs::OsRng).to_vec();
    let ciphertext = cipher.encrypt(&GenericArray::from_slice(nonce.as_slice()), text.as_ref())?;

    Ok((nonce, ciphertext))
}

#[wasm_bindgen(start)]
async fn main() -> Result<(), JsValue> {
    let w: Window =
        web_sys::window().ok_or_else(|| JsValue::from_str("No global `window` exists"))?;
    let document: Document = w
        .document()
        .ok_or_else(|| JsValue::from_str("No document available"))?;

    let text_area = document
        .get_element_by_id("editor")
        .expect("Element not found.");

    let location: Location = w.location();
    let current_path = location
        .pathname()
        .map_err(|_| JsValue::from_str("Failed to get pathname"))?;
    let editor_url = format!("/editor{}", current_path);
    let promise = w.fetch_with_str(&editor_url);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(1, 2), 3);
    }

    #[test]
    fn test_encrypt_message() {
        let key = "12341234123412341234123412341234";
        let text = "my test text";
        let ret = encrypt_message(key, text).unwrap();
        let decrypted = decrypt_message(key, &ret.0, &ret.1).unwrap();

        assert_eq!(decrypted, text.as_bytes());
    }
}
