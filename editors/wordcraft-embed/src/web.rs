//! The embedded browser shell. Adapted from upstream
//! `upstream/wordcraft/apps/wordcraft-web/src/web.rs` (MIT OR Apache-2.0):
//! documents come from and go to the hub through [`crate::bridge`]; pictures,
//! fonts and recipient lists still use the browser's file picker. When opened
//! outside the hub it falls back to the picker and downloads.

use std::rc::Rc;

use wasm_bindgen::JsCast as _;
use wordcraft_engine::Session;
use wordcraft_ui_egui::{Inbox, Services, WordApp};

use crate::bridge;

#[path = "../../../upstream/wordcraft/apps/wordcraft-web/src/web/font_store.rs"]
#[rustfmt::skip] // upstream code, kept unchanged
#[allow(clippy::useless_conversion)]
mod font_store;

const DOC_EXTS: &[&str] = &[
    "docx", "docm", "dotx", "dotm", "doc", "dot", "odt", "rtf", "txt", "md", "html", "htm", "tex",
    "json", "xml",
];
const IMAGE_EXTS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "bmp", "wmf", "emf"];
const RECIPIENT_EXTS: &[&str] = &["csv", "tsv", "txt", "xlsx", "xlsm", "ods"];
const CANVAS_ID: &str = "editor_canvas";
const LOADING_ID: &str = "editor_loading";

pub fn start() {
    eframe::WebLogger::init(log::LevelFilter::Info).ok();
    wasm_bindgen_futures::spawn_local(async {
        let Some(document) = web_sys::window().and_then(|w| w.document()) else {
            log::error!("no document");
            return;
        };
        let Some(canvas) = document
            .get_element_by_id(CANVAS_ID)
            .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        else {
            log::error!("missing <canvas id=\"{CANVAS_ID}\">");
            return;
        };
        let mut options = eframe::WebOptions::default();
        if query().contains("webgl")
            && let eframe::egui_wgpu::WgpuSetup::CreateNew(create) =
                &mut options.wgpu_options.wgpu_setup
        {
            create.instance_descriptor.backends = eframe::wgpu::Backends::GL;
        }
        let result = eframe::WebRunner::new()
            .start(
                canvas,
                options,
                Box::new(move |cc| {
                    font_store::load_all(cc.egui_ctx.clone());
                    let inbox: Inbox = Inbox::default();
                    let hub_inbox = inbox.clone();
                    let hub_ctx = cc.egui_ctx.clone();
                    if let Err(e) = bridge::listen(move |name, bytes| {
                        hub_inbox
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .push((name, bytes));
                        hub_ctx.request_repaint();
                    }) {
                        log::error!("can't receive documents from the hub: {e}");
                    }
                    let region = wordcraft_ui_egui::i18n::system_region();
                    let mut session = Session::new(wordcraft_engine::blank_document(&region));
                    session.system_region = region;
                    let mut app = WordApp::new(session, services(inbox, cc.egui_ctx.clone()));
                    app.autosave = false;
                    if !query().contains("new") {
                        app.start_screen();
                    }
                    bridge::ready();
                    Ok(Box::new(Shell { app }))
                }),
            )
            .await;
        if let Some(el) = document.get_element_by_id(LOADING_ID) {
            match result {
                Ok(()) => el.remove(),
                Err(e) => el.set_inner_html(&format!("<p>The editor failed to start: {e:?}</p><p>A browser with WebGPU or WebGL2 is required.</p>")),
            }
        }
    });
}

fn query() -> String {
    web_sys::window()
        .and_then(|w| w.location().search().ok())
        .unwrap_or_default()
}

struct Shell {
    app: WordApp,
}

impl eframe::App for Shell {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.app.logic(ctx);
    }

    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.app.raw_input_hook(raw);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.app.ui(ui);
    }
}

fn services(inbox: Inbox, ctx: egui::Context) -> Services {
    let open_inbox = inbox.clone();
    let last_dirty = Rc::new(std::cell::Cell::new(None));
    Services {
        open_async: Some(Box::new(move |purpose: &str| {
            // Documents come from the hub's file browser; it answers with an `open` message.
            if purpose == "document" && bridge::open_request().is_ok() {
                return;
            }
            let dialog = match purpose {
                "picture" => rfd::AsyncFileDialog::new().add_filter("Pictures", IMAGE_EXTS),
                "font" => {
                    rfd::AsyncFileDialog::new().add_filter("Fonts", wordcraft_fonts::FONT_FILE_EXTS)
                }
                "recipients" => {
                    rfd::AsyncFileDialog::new().add_filter("Recipient lists", RECIPIENT_EXTS)
                }
                _ => rfd::AsyncFileDialog::new().add_filter("Documents", DOC_EXTS),
            };
            let inbox = open_inbox.clone();
            let ctx = ctx.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let Some(file) = dialog.pick_file().await else {
                    return;
                };
                let bytes = file.read().await;
                inbox
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push((file.file_name(), bytes));
                ctx.request_repaint();
            });
        })),
        pick_save: Some(Box::new(|name: &str| Some(name.to_string()))),
        download: Some(Box::new(|path: &str, bytes: &[u8]| {
            let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
            if bridge::embedded() {
                bridge::save(name, bytes)
            } else {
                download(name, bytes)
            }
        })),
        inbox: Some(inbox),
        on_dirty: Some(Box::new(move |d| {
            // Called every pass; only tell the hub about changes.
            if last_dirty.replace(Some(d)) != Some(d) {
                bridge::dirty(d);
            }
        })),
        store_font: Some(Box::new(font_store::store)),
        forget_font: Some(Box::new(font_store::forget)),
        ..Default::default()
    }
}

/// Outside the hub: hand the bytes to the browser as a download.
fn download(name: &str, bytes: &[u8]) -> Result<(), String> {
    let js = |e: wasm_bindgen::JsValue| format!("{e:?}");
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let parts = js_sys::Array::of1(&js_sys::Uint8Array::from(bytes));
    let blob = web_sys::Blob::new_with_u8_array_sequence(&parts).map_err(js)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(js)?;
    let a = document
        .create_element("a")
        .map_err(js)?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .map_err(|_| "not an anchor")?;
    a.set_href(&url);
    a.set_download(name);
    a.click();
    let revoke = wasm_bindgen::closure::Closure::once_into_js(move || {
        let _ = web_sys::Url::revoke_object_url(&url);
    });
    let _ = window.set_timeout_with_callback(revoke.unchecked_ref());
    Ok(())
}
