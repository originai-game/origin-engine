//! Origin Engine web host (`docs/design.md` §2.2, §3.1): the binding symbols
//! of `crates/og-api/api.ogt` as a wasm-bindgen module (`origin-engine.wasm`)
//! over an `og_ecs::World` and the renderer. It runs inside a dedicated
//! Worker; the main thread transfers it an OffscreenCanvas and pumps input
//! (`packages/engine/src/web/`).
//!
//! - `src/generated/glue.rs` (`og api gen`) exports every symbol and calls
//!   the `og_api::host` trait method of the same name; `src/host.rs`
//!   implements the traits, so a member missing here is a build error.
//! - Column views are byte offsets into the module's memory; the Worker
//!   loader makes typed arrays over `memory.buffer` at them, so a system
//!   reads and writes engine memory without a copy.
//! - The dispatcher is a JavaScript function installed once; the engine calls
//!   it for every system, command and facts call.
//! - `og_web_attach_canvas` puts the renderer on an OffscreenCanvas (WebGPU,
//!   or WebGL2); `og_engine_frame` then ticks and renders.

/// The exports, one per binding symbol (`og api gen`).
#[doc(hidden)]
#[rustfmt::skip]
#[path = "generated/glue.rs"]
pub mod glue;
mod host;

use og_api::runtime::{Dispatch, Hooks};
use wasm_bindgen::prelude::*;

/// The web host: a unit type, since the engines live in the runtime's
/// thread-local tables.
pub struct WebHost;

thread_local! {
    /// `performance` and its `now`, where the global has them (Workers do).
    static PERFORMANCE: Option<(JsValue, js_sys::Function)> = {
        let performance = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("performance")).ok();
        performance.and_then(|p| {
            let now = js_sys::Reflect::get(&p, &JsValue::from_str("now")).ok()?;
            Some((p, now.dyn_into::<js_sys::Function>().ok()?))
        })
    };
}

/// Calls the installed JavaScript dispatcher.
pub(crate) struct WebHooks {
    pub dispatcher: js_sys::Function,
}

impl Hooks for WebHooks {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        let args = js_sys::Array::new();
        for v in [
            JsValue::from(call.kind),
            JsValue::from(call.engine),
            JsValue::from(call.id),
            JsValue::from(call.query),
            JsValue::from(call.tick),
            JsValue::from(call.dt),
            JsValue::from(call.elapsed),
            JsValue::from_str(&call.payload),
        ] {
            args.push(&v);
        }
        let answer = self
            .dispatcher
            .apply(&JsValue::NULL, &args)
            .map_err(|e| format!("{e:?}"))?;
        let text = answer.as_string().unwrap_or_default();
        match text.strip_prefix('=') {
            Some(payload) => Ok(payload.to_string()),
            None if text.is_empty() => Ok(String::new()),
            None => Err(text),
        }
    }

    fn now_ms(&self) -> f64 {
        PERFORMANCE.with(|p| match p {
            Some((performance, now)) => now
                .call0(performance)
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            None => js_sys::Date::now(),
        })
    }
}

/// Routes Rust panics to the console instead of an opaque `unreachable`.
#[wasm_bindgen]
pub fn og_web_init() {
    std::panic::set_hook(Box::new(|info| {
        let message = format!("origin-engine panicked: {info}");
        let console = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("console"));
        if let Ok(console) = console {
            if let Ok(error) = js_sys::Reflect::get(&console, &JsValue::from_str("error")) {
                if let Some(f) = error.dyn_ref::<js_sys::Function>() {
                    let _ = f.call1(&console, &JsValue::from_str(&message));
                }
            }
        }
    }));
}

#[cfg(target_arch = "wasm32")]
mod canvas {
    use og_api::runtime::{set_render_backend, RenderBackend};
    use og_render::{Backend, RenderDesc, RenderScene, RenderTarget, Renderer, RgbaImage};
    use wasm_bindgen::prelude::*;

    struct CanvasBackend {
        renderer: Renderer,
        tier: String,
    }

    impl RenderBackend for CanvasBackend {
        fn tier(&self) -> String {
            self.tier.clone()
        }
        fn size(&self) -> (u32, u32) {
            self.renderer.size()
        }
        fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
            self.renderer
                .render(scene, &RenderTarget::Surface)
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
        fn capture(&mut self, _scene: &RenderScene) -> Option<RgbaImage> {
            // og-render reads back natively only so far (M3 owes the web).
            None
        }
    }

    /// Put the renderer on an OffscreenCanvas transferred to this Worker.
    /// Resolves to the tier (`webgpu`, `webgl2`), or rejects with the reason.
    #[wasm_bindgen]
    pub async fn og_web_attach_canvas(
        engine: u32,
        canvas: web_sys::OffscreenCanvas,
        webgl: bool,
    ) -> Result<String, JsValue> {
        let backend = if webgl {
            Backend::WebGl2
        } else {
            Backend::WebGpu
        };
        let desc = RenderDesc {
            width: canvas.width().max(1),
            height: canvas.height().max(1),
            backend,
            ..RenderDesc::default()
        };
        let instance = backend.instance();
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::OffscreenCanvas(canvas))
            .map_err(|e| JsValue::from_str(&format!("surface: {e}")))?;
        let renderer = Renderer::with_surface(&desc, &instance, surface)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let tier = renderer.info().backend_tier();
        set_render_backend(
            engine,
            Some(Box::new(CanvasBackend {
                renderer,
                tier: tier.clone(),
            })),
        )
        .map_err(|e| JsValue::from_str(&e))?;
        Ok(tier)
    }
}
