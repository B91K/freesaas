//! WordCraft inside the FreeSaaS hub. See `bridge` for the message protocol.
//!
//! URL flags: `?new` starts on a blank document instead of File › Home;
//! `?webgl` forces WebGL2.

#[cfg(target_arch = "wasm32")]
mod bridge;
#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
fn main() {
    web::start();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("wordcraft-embed only runs in the browser: build it with `trunk build --release`");
}
