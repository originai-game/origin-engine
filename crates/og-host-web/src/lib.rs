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
//! - og-probe is mounted by the runtime; the Worker hands each frame of the
//!   shell's request ring to `og_engine_call`. A capture renders offscreen
//!   and starts the readback; the frame is answered by the next call once the
//!   copy has landed (requested on tick N, answered on N + 1), so the ring's
//!   `capture` is retried by the Worker until it answers.
//! - Cooked files reach the runtime through `og_web_put_file` (the Worker
//!   fetches the game's bundle before creating the engine); textures are
//!   transcoded by the Worker with Binomial's Basis transcoder
//!   (`src/web/basis.ts`) and handed over with `og_web_put_texture`.

/// The exports, one per binding symbol (`og api gen`).
#[doc(hidden)]
#[rustfmt::skip]
#[path = "generated/glue.rs"]
pub mod glue;
mod audio;
mod host;

use og_api::runtime::{Dispatch, Fetch, Hooks};
use std::cell::RefCell;
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;

/// The web host: a unit type, since the engines live in the runtime's
/// thread-local tables.
pub struct WebHost;

thread_local! {
    /// Cooked files the Worker fetched, by path.
    static FILES: RefCell<BTreeMap<String, Result<Vec<u8>, String>>> = const { RefCell::new(BTreeMap::new()) };
    /// Transcoded textures the Worker handed over, by asset id.
    static TEXTURES: RefCell<BTreeMap<String, og_render::TextureData>> = const { RefCell::new(BTreeMap::new()) };
    /// Paths handed to the fetcher, so each is asked for once.
    static REQUESTED: RefCell<std::collections::BTreeSet<String>> = const { RefCell::new(std::collections::BTreeSet::new()) };
    /// The Worker's `fetch(path)` for files nobody fetched yet.
    static FETCHER: RefCell<Option<js_sys::Function>> = const { RefCell::new(None) };
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

    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        let known = FILES.with(|f| f.borrow().get(path).cloned());
        match known {
            Some(Ok(bytes)) => Fetch::Ready(bytes),
            Some(Err(e)) => Fetch::Failed(e),
            None => FETCHER.with(|f| match &*f.borrow() {
                Some(fetch) => {
                    if REQUESTED.with(|r| r.borrow_mut().insert(path.to_string())) {
                        let _ = fetch.call1(&JsValue::NULL, &JsValue::from_str(path));
                    }
                    Fetch::Pending
                }
                None => Fetch::Failed(format!(
                    "{path} was not fetched: the Worker fetches the game's cooked bundle before the engine starts"
                )),
            }),
        }
    }

    fn texture(&self, texture: &og_cook::runtime::CookedTexture) -> Fetch<og_render::TextureData> {
        if let Some(t) = TEXTURES.with(|t| t.borrow().get(&texture.id).cloned()) {
            return Fetch::Ready(t);
        }
        match texture.to_texture_data() {
            Ok(t) => Fetch::Ready(t),
            Err(_) => Fetch::Failed(format!(
                "{}: not transcoded; the Worker transcodes a bundle's Basis textures (src/web/basis.ts) before the engine starts",
                texture.id
            )),
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

/// A cooked file the Worker fetched (`path` as the runtime asks for it:
/// `<root>/cooked/<file>`).
#[wasm_bindgen]
pub fn og_web_put_file(path: String, bytes: Vec<u8>) {
    FILES.with(|f| f.borrow_mut().insert(path, Ok(bytes)));
}

/// A cooked file that could not be fetched, with the reason.
#[wasm_bindgen]
pub fn og_web_fail_file(path: String, reason: String) {
    FILES.with(|f| f.borrow_mut().insert(path, Err(reason)));
}

/// A texture the Worker transcoded to RGBA8 (level 0, rows top to bottom).
#[wasm_bindgen]
pub fn og_web_put_texture(id: String, width: u32, height: u32, rgba: Vec<u8>, srgb: bool) {
    let data = og_render::TextureData {
        width,
        height,
        rgba,
        color_space: if srgb {
            og_render::ColorSpace::Srgb
        } else {
            og_render::ColorSpace::Linear
        },
        mips: og_render::Mips::Generate,
    };
    TEXTURES.with(|t| t.borrow_mut().insert(id, data));
}

/// The Worker's `fetch(path)`, called for a file nobody fetched yet; it
/// answers later with `og_web_put_file` (a load then completes at a tick
/// boundary).
#[wasm_bindgen]
pub fn og_web_set_fetcher(fetch: js_sys::Function) {
    FETCHER.with(|f| *f.borrow_mut() = Some(fetch));
}

fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The cooked outputs a bundle's manifest lists, for the Worker's preload:
/// JSON `[{ "id", "file", "type" }]` in manifest order, or
/// `{ "error": reason }` for a manifest that does not parse.
#[wasm_bindgen]
pub fn og_web_bundle_files(manifest: String) -> String {
    match og_cook::runtime::Bundle::from_manifest(&manifest) {
        Ok(bundle) => {
            let items: Vec<String> = bundle
                .manifest()
                .outputs()
                .map(|o| {
                    format!(
                        "{{\"id\":{},\"file\":{},\"type\":{}}}",
                        json_str(&o.id),
                        json_str(&o.file),
                        json_str(&o.ty)
                    )
                })
                .collect();
            format!("[{}]", items.join(","))
        }
        Err(e) => format!("{{\"error\":{}}}", json_str(&e.to_string())),
    }
}

/// `bytes` copied into 16-aligned memory, as blob views require.
fn aligned(bytes: &[u8]) -> Vec<u128> {
    let mut words = vec![0u128; bytes.len().div_ceil(16)];
    // SAFETY: `words` holds at least `bytes.len()` bytes; u8 has no alignment.
    let dst =
        unsafe { std::slice::from_raw_parts_mut(words.as_mut_ptr().cast::<u8>(), bytes.len()) };
    dst.copy_from_slice(bytes);
    words
}

/// The KTX2 file of a Basis-coded texture blob, checked against the
/// manifest's type, schema version and cook stamp, for the Worker's Basis
/// transcoder. Empty for an RGBA8-coded texture (the runtime decodes those
/// itself) and for a blob that does not check (the runtime then refuses it
/// with the reason when it loads).
#[wasm_bindgen]
pub fn og_web_texture_ktx2(manifest: String, id: String, bytes: Vec<u8>) -> Vec<u8> {
    let Ok(bundle) = og_cook::runtime::Bundle::from_manifest(&manifest) else {
        return Vec::new();
    };
    let words = aligned(&bytes);
    // SAFETY: the words hold `bytes.len()` initialised bytes.
    let view = unsafe { std::slice::from_raw_parts(words.as_ptr().cast::<u8>(), bytes.len()) };
    match bundle.texture(&id, view) {
        Ok(t) if t.meta.codec != og_cook::format::codec::RGBA8 => t.ktx2,
        _ => Vec::new(),
    }
}

/// The play surface the UI lays out in: `{ width, height, dpr, safeArea?,
/// coarsePointer? }` in CSS pixels. Answers wire JSON.
#[wasm_bindgen]
pub fn og_web_set_viewport(engine: u32, viewport: String) -> String {
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&viewport);
    match parsed
        .map_err(|e| e.to_string())
        .and_then(|v| og_api::runtime::set_viewport(engine, v))
    {
        Ok(()) => r#"{"ok":true}"#.to_string(),
        Err(e) => serde_json::json!({ "ok": false, "reason": e }).to_string(),
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
        /// A capture in flight and the size it was asked at.
        pending: Option<(og_render::CaptureRequest, (u32, u32))>,
    }

    impl RenderBackend for CanvasBackend {
        fn tier(&self) -> String {
            self.tier.clone()
        }
        fn size(&self) -> (u32, u32) {
            self.renderer.size()
        }
        fn antialias(&self) -> String {
            match self.renderer.info().antialias {
                og_render::Antialias::Msaa4 => "msaa4",
                og_render::Antialias::Taa => "taa",
                _ => "off",
            }
            .into()
        }
        fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
            self.renderer.poll();
            self.renderer
                .render(scene, &RenderTarget::Surface)
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
        /// Request on one call, answer on a later one: the first call draws
        /// offscreen and starts the readback; a later call at the same size
        /// answers the frame once the copy has landed.
        fn capture(&mut self, scene: &RenderScene, width: u32, height: u32) -> Option<RgbaImage> {
            let target = self.renderer.size();
            let size = if width == 0 || height == 0 {
                target
            } else {
                (width, height)
            };
            if let Some((mut req, asked)) = self.pending.take() {
                self.renderer.poll();
                match req.try_finish(&self.renderer) {
                    Ok(Some(image)) if asked == size => return Some(image),
                    Ok(None) if asked == size => {
                        self.pending = Some((req, asked));
                        return None;
                    }
                    _ => {}
                }
            }
            if size != target {
                self.renderer.resize(size.0, size.1);
            }
            let started = self
                .renderer
                .render(scene, &RenderTarget::Offscreen)
                .ok()
                .and_then(|_| self.renderer.begin_capture().ok());
            if size != target {
                self.renderer.resize(target.0, target.1);
            }
            self.pending = started.map(|r| (r, size));
            None
        }
    }

    fn antialias(name: &str) -> og_render::Antialias {
        match name {
            "off" => og_render::Antialias::Off,
            "msaa4" => og_render::Antialias::Msaa4,
            "taa" => og_render::Antialias::Taa,
            _ => og_render::Antialias::Tier,
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
        let options = og_api::runtime::render_options(engine);
        let desc = RenderDesc {
            width: canvas.width().max(1),
            height: canvas.height().max(1),
            backend,
            antialias: antialias(options["antialias"].as_str().unwrap_or("tier")),
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
                pending: None,
            })),
        )
        .map_err(|e| JsValue::from_str(&e))?;
        Ok(tier)
    }
}
