use aes_gcm::aead::generic_array::GenericArray;
use base64::engine::general_purpose;
use base64::Engine;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit},
    Aes256Gcm,
};

// The wasm-pack uses wasm-bindgen to build and generate JavaScript binding file.
// Import the wasm-bindgen crate.
use wasm_bindgen_futures::JsFuture;
use web_sys::HtmlInputElement;

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

        let text = document
            .get_element_by_id("editor")
            .and_then(|el| el.dyn_into::<HtmlTextAreaElement>().ok())
            .map(|text_area| text_area.value());

        let key: Option<String> = document
            .get_element_by_id("key-aes")
            .and_then(|el| el.dyn_into::<HtmlInputElement>().ok())
            .map(|text_area| text_area.value());

        let bytes = general_purpose::STANDARD
            .decode(key.unwrap_or_default())
            .unwrap();

        let enc = encrypt_message(&bytes, text.clone().unwrap_or_default().as_str());
        web_sys::console::log_1(&JsValue::from_str(&format!("content={:?}", enc)));

        // Create the request
        let mut opts = RequestInit::new();
        let headers = Headers::new().unwrap();
        headers
            .append("Content-Type", "application/octet-stream")
            .unwrap();

        let content = format!("content={}", text.unwrap_or_default());
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

fn decrypt_message(key: &str, message: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    if message.len() < 18 || &message[0..3] != &[0xFF, 0xFF, 0xFF] {
        return Ok(String::from_utf8(message.to_vec())?);
    }

    if key.as_bytes().len() != 32 {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid key length",
        )));
    }

    let mem = &message[15..];
    let nonce = &message[3..15];

    let cipher = Aes256Gcm::new(GenericArray::from_slice(key.as_bytes()));
    let plaintext_bytes = cipher
        .decrypt(&GenericArray::from_slice(nonce), mem)
        .unwrap();

    Ok(String::from_utf8(plaintext_bytes)?)
}

fn encrypt_message(key: &[u8], text: &str) -> Result<Vec<u8>, aes_gcm::aead::Error> {
    if key.len() != 32 {
        return Err(aes_gcm::aead::Error);
    }

    let cipher = Aes256Gcm::new(GenericArray::from_slice(key));
    let mut nonce = Aes256Gcm::generate_nonce(&mut rand::rngs::OsRng).to_vec();
    let mut ciphertext =
        cipher.encrypt(&GenericArray::from_slice(nonce.as_slice()), text.as_ref())?;

    let mut message = vec![0xff, 0xff, 0xff];

    message.append(&mut nonce);
    message.append(&mut ciphertext);

    Ok(message)
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
        let ret = encrypt_message(key.as_bytes(), text).unwrap();

        assert_eq!(&ret[0..=2], vec![0xFF, 0xFF, 0xFF]);

        let decrypted = decrypt_message(key, &ret).unwrap();
        assert_eq!(decrypted, text);

        let not_encr = decrypt_message("", "text".as_bytes()).unwrap();
        assert_eq!(not_encr, "text")
    }
}
