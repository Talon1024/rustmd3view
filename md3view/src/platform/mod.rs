#[cfg(not(target_family = "wasm"))]
mod native;
#[cfg(not(target_family = "wasm"))]
pub(crate) use native::*;

#[cfg(target_family = "wasm")]
mod wasm;
#[cfg(target_family = "wasm")]
mod wasm_run;
#[cfg(target_family = "wasm")]
pub(crate) use wasm::*;

#[non_exhaustive]
pub enum DrawingContextRequest {
    OpenGL,
}
