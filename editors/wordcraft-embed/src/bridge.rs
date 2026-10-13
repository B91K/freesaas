//! The hub ↔ editor message protocol (window.postMessage, same origin only).
//!
//! Editor → hub (`source: "freesaas-editor"`):
//! - `{type: "ready", app}`: listening; the hub may now send a document.
//! - `{type: "open-request", app}`: the user chose File › Open.
//! - `{type: "save", app, name, bytes}`: a save or export; `bytes` is an ArrayBuffer.
//! - `{type: "dirty", app, dirty}`: whether the document has unsaved changes.
//!
//! Hub → editor (`source: "freesaas-hub"`):
//! - `{type: "open", name, bytes}`: open this document.

use wasm_bindgen::JsCast as _;
use wasm_bindgen::JsValue;
use wasm_bindgen::closure::Closure;

pub const APP: &str = "wordcraft";
const FROM_EDITOR: &str = "freesaas-editor";
const FROM_HUB: &str = "freesaas-hub";

fn window() -> Option<web_sys::Window> {
    web_sys::window()
}

fn origin() -> Option<String> {
    window()?.location().origin().ok()
}

/// The hub's window, when this editor is framed by a page on our own origin.
fn hub() -> Option<web_sys::Window> {
    let w = window()?;
    let parent = w.parent().ok().flatten()?;
    if js_sys::Object::is(&parent, &w) {
        None
    } else {
        Some(parent)
    }
}

pub fn embedded() -> bool {
    hub().is_some()
}

fn set(obj: &js_sys::Object, key: &str, value: &JsValue) {
    let _ = js_sys::Reflect::set(obj, &JsValue::from_str(key), value);
}

fn message(kind: &str) -> js_sys::Object {
    let obj = js_sys::Object::new();
    set(&obj, "source", &JsValue::from_str(FROM_EDITOR));
    set(&obj, "type", &JsValue::from_str(kind));
    set(&obj, "app", &JsValue::from_str(APP));
    obj
}

fn post(msg: &js_sys::Object, transfer: Option<&js_sys::Array>) -> Result<(), String> {
    let hub = hub().ok_or("not running inside the hub")?;
    let origin = origin().ok_or("no origin")?;
    let r = match transfer {
        Some(t) => hub.post_message_with_transfer(msg, &origin, t),
        None => hub.post_message(msg, &origin),
    };
    r.map_err(|e| format!("{e:?}"))
}

pub fn ready() {
    if let Err(e) = post(&message("ready"), None) {
        log::warn!("couldn't tell the hub the editor is ready: {e}");
    }
}

pub fn open_request() -> Result<(), String> {
    post(&message("open-request"), None)
}

pub fn dirty(dirty: bool) {
    let msg = message("dirty");
    set(&msg, "dirty", &JsValue::from_bool(dirty));
    // Best effort: the hub only uses it for its leave-page warning.
    let _ = post(&msg, None);
}

pub fn save(name: &str, bytes: &[u8]) -> Result<(), String> {
    let msg = message("save");
    let array = js_sys::Uint8Array::from(bytes);
    set(&msg, "name", &JsValue::from_str(name));
    set(&msg, "bytes", &array.buffer());
    post(&msg, Some(&js_sys::Array::of1(&array.buffer())))
}

/// Call `on_open(name, bytes)` for every `open` message from the hub.
pub fn listen(on_open: impl Fn(String, Vec<u8>) + 'static) -> Result<(), String> {
    let w = window().ok_or("no window")?;
    let expected = origin().ok_or("no origin")?;
    let handler =
        Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |e: web_sys::MessageEvent| {
            if e.origin() != expected {
                return;
            }
            let data = e.data();
            let get = |k: &str| js_sys::Reflect::get(&data, &JsValue::from_str(k)).ok();
            if get("source").and_then(|v| v.as_string()).as_deref() != Some(FROM_HUB)
                || get("type").and_then(|v| v.as_string()).as_deref() != Some("open")
            {
                return;
            }
            let Some(name) = get("name").and_then(|v| v.as_string()) else {
                return;
            };
            let Some(buffer) = get("bytes").filter(|v| v.is_instance_of::<js_sys::ArrayBuffer>())
            else {
                log::warn!("hub sent {name} without bytes");
                return;
            };
            on_open(name, js_sys::Uint8Array::new(&buffer).to_vec());
        });
    w.add_event_listener_with_callback("message", handler.as_ref().unchecked_ref())
        .map_err(|e| format!("{e:?}"))?;
    // The listener lives as long as the page.
    handler.forget();
    Ok(())
}
