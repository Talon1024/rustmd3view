use anyhow::Error;
use super::DrawingContextRequest;
use base64::Engine;
use glow::Context;
use std::{future::Future, panic, str, sync::Arc};
use wasm_bindgen::{prelude::*, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    console, window, HtmlElement, WebGl2RenderingContext, WebGlRenderingContext,
    Response,
};
use js_sys::{ArrayBuffer, Uint8Array, JsString};
use winit::platform::web::WindowExtWebSys;
use winit::{event_loop::EventLoopWindowTarget, window::Window};

pub(crate) struct WindowContext;

impl WindowContext {
    pub(crate) fn swap_buffers(&self) -> Result<(), ()> {
        Ok(())
    }
}

pub(crate) fn spawn_local(f: impl Future<Output = ()> + 'static) {
    wasm_bindgen_futures::spawn_local(f)
}

pub(crate) fn init() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

pub(crate) fn show_window<CE>(
    win: &Window,
    _el: &EventLoopWindowTarget<CE>,
    _ctxr: DrawingContextRequest,
) -> (WindowContext, Arc<Context>) {
    let canvas = win.canvas();
    let document =
        window().expect("No window!").document().expect("No document!");
    document
        .body()
        .expect("Failed to get document body!")
        .append_child(&canvas)
        .expect("Failed to add canvas to document body!");
    let glc = canvas
        .get_context("webgl2")
        .and_then(|ctx| {
            ctx.ok_or(JsValue::from_str("WebGL2 context not created"))
                .and_then(|ctx| {
                    ctx.dyn_into::<WebGl2RenderingContext>()
                        .map_err(JsValue::from)
                })
                .map(Context::from_webgl2_context)
                .map_err(|e| {
                    JsValue::from_str(&format!(
                        "Failed to get WebGL2 context!\n\n{e:?}"
                    ))
                })
        })
        .or_else(|e| {
            console::error_1(&e);
            canvas.get_context("webgl").and_then(|ctx| {
                ctx.ok_or(JsValue::from_str("WebGL context not created"))
                    .and_then(|ctx| {
                        ctx.dyn_into::<WebGlRenderingContext>()
                            .map_err(JsValue::from)
                    })
                    .map(Context::from_webgl1_context)
                    .map_err(|e| {
                        JsValue::from_str(&format!(
                            "Failed to get WebGL context!\n\n{e:?}"
                        ))
                    })
            })
        })
        .map(Arc::new)
        .unwrap();
    (WindowContext, glc)
}

/*
macro_rules! set_attribute {
    ($element: ident, $attribute: literal, $value: expr) => {
        let
        $element.set_attribute($attribute, $value).expect("{}")
    };
}
 */

pub(crate) async fn save_file(contents: Vec<u8>, fname: &str) {
    let bwindow = window().expect("No window!");
    let document = bwindow.document().expect("No document!");
    let body = document.body().expect("No body!");
    let anchor = match document.get_element_by_id("app-downloader") {
        Some(el) => el,
        None => {
            let el = document
                .create_element("a")
                .expect("`a` should be a valid element name");
            el.set_id("app-downloader");
            body.append_child(&el)
                .expect("Could not add downloader to document!");
            el
        }
    }
    .dyn_into::<HtmlElement>()
    .expect("anchor is not an HTML element!");
    // Download and class attributes
    anchor
        .set_attribute("download", fname)
        .expect("`download` should be a valid attribute name");
    anchor
        .set_attribute("style", "display: none;")
        .expect("`style` should be a valid attribute name");
    // The data to download
    let (is_text, ftype) = match str::from_utf8(&contents).is_ok() {
        true => (true, "text/plain"),
        false => (false, "application/octet-stream"),
    };
    let base64 = if is_text { "" } else { ";base64" };
    let data = if !is_text {
        let engine = base64::engine::general_purpose::URL_SAFE;
        engine.encode(&contents)
    } else {
        percent_encoding::percent_encode(
            &contents,
            percent_encoding::NON_ALPHANUMERIC,
        )
        .collect()
    };
    let href = format!("data:{ftype}{base64},{data}");
    anchor
        .set_attribute("href", &href)
        .expect("`href` should be a valid attribute name");
    anchor.click();
}

// An asset is a file that should be part of the application distribution
pub(crate) async fn load_asset(relative_path: impl AsRef<str>) -> Result<Vec<u8>, Error> {
    let window = window().expect("No window!");
    let fut = JsFuture::from(window.fetch_with_str(relative_path.as_ref()));
    let response = fut.await
        .map_err(|e| {
            let jstr = JsString::from(e);
            Error::msg(ToString::to_string(&jstr))
        })
        .map(|f| f.dyn_into::<Response>().expect("Not a response!"))?;
    match response.status() {
        200..=299 => (),
        _ => return Err(Error::msg(response.status_text())),
    }
    let fut = JsFuture::from(response.array_buffer().unwrap());
    let array_buffer = fut.await
        .map_err(|e| {
            let jstr = JsString::from(e);
            Error::msg(ToString::to_string(&jstr))
        })
        .map(|f| f.dyn_into::<ArrayBuffer>().expect("Not a ArrayBuffer!"))?;
    let size = array_buffer.byte_length() as usize;
    let data = Uint8Array::new(&array_buffer).to_vec();
    assert_eq!(size, data.len());
    Ok(data)
}
