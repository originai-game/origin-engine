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
//!   fetches the game's bundle before creating the engine); once the
//!   renderer is attached, the Worker asks `og_web_texture_target` which
//!   encoding each Basis texture should reach this device in (the
//!   renderer's `texture_formats`, og-cook's choice by kind), transcodes it
//!   with Binomial's Basis transcoder (`src/web/basis.ts`) and hands it over
//!   with `og_web_put_texture` (RGBA8 level 0) or
//!   `og_web_put_texture_encoded` (every level of a block format).

/// The exports, one per binding symbol (`og api gen`).
#[doc(hidden)]
#[rustfmt::skip]
#[path = "generated/glue.rs"]
pub mod glue;
mod audio;
mod host;
mod host_e6;
mod host_p2;

use og_runtime::{Dispatch, Fetch, Hooks};
use std::cell::RefCell;
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;

/// The web host: a unit type, since the engines live in the runtime's
/// thread-local tables.
pub struct WebHost;

thread_local! {
    /// Cooked files the Worker fetched, by path. A streaming group's pack
    /// stays only until the engine reads it (or lets go of it).
    static FILES: RefCell<BTreeMap<String, Result<Vec<u8>, String>>> = const { RefCell::new(BTreeMap::new()) };
    /// Transcoded textures the Worker handed over, by asset id.
    static TEXTURES: RefCell<BTreeMap<String, og_render::TextureSource>> = const { RefCell::new(BTreeMap::new()) };
    /// The encodings each engine's attached renderer takes.
    static FORMATS: RefCell<BTreeMap<u32, og_render::TextureFormats>> = const { RefCell::new(BTreeMap::new()) };
    /// Paths handed to the fetcher and not answered yet, so each is asked for once at a time.
    static REQUESTED: RefCell<std::collections::BTreeSet<String>> = const { RefCell::new(std::collections::BTreeSet::new()) };
    /// Packs let go of while they were fetched, with their group's texture
    /// ids: dropped, textures and all, when they land.
    static ORPHANED: RefCell<BTreeMap<String, Vec<String>>> = const { RefCell::new(BTreeMap::new()) };
    /// Render-target sizes the shell asked for, by engine, applied before the next frame.
    static RESIZES: RefCell<BTreeMap<u32, (u32, u32)>> = const { RefCell::new(BTreeMap::new()) };
    /// The Worker's `fetch(path)` for files nobody fetched yet.
    static FETCHER: RefCell<Option<js_sys::Function>> = const { RefCell::new(None) };
    /// The Worker's `inflate(key, bytes)` (DecompressionStream), and what it answered by key.
    static INFLATER: RefCell<Option<js_sys::Function>> = const { RefCell::new(None) };
    static INFLATED: RefCell<BTreeMap<u64, Result<Vec<u8>, String>>> = const { RefCell::new(BTreeMap::new()) };
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
        // A pack is read once per load: taken, not copied, and fetched
        // again (a failure too) the next time its group loads.
        let known = if is_stream_pack(path) {
            FILES.with(|f| f.borrow_mut().remove(path))
        } else {
            FILES.with(|f| f.borrow().get(path).cloned())
        };
        match known {
            Some(Ok(bytes)) => Fetch::Ready(bytes),
            Some(Err(e)) => Fetch::Failed(e),
            None => FETCHER.with(|f| match &*f.borrow() {
                Some(fetch) => {
                    // A pack let go of in flight is wanted again: kept when it lands.
                    ORPHANED.with(|o| o.borrow_mut().remove(path));
                    if REQUESTED.with(|r| r.borrow_mut().insert(path.to_string()))
                        && let Err(e) = fetch.call1(&JsValue::NULL, &JsValue::from_str(path))
                    {
                        REQUESTED.with(|r| r.borrow_mut().remove(path));
                        return Fetch::Failed(format!("{path}: the fetcher threw {e:?}"));
                    }
                    Fetch::Pending
                }
                None => Fetch::Failed(format!(
                    "{path} was not fetched: the Worker fetches the game's cooked bundle before the engine starts"
                )),
            }),
        }
    }

    fn texture(
        &self,
        texture: &og_cook::runtime::CookedTexture,
        formats: Option<&og_render::TextureFormats>,
    ) -> Fetch<og_render::TextureSource> {
        if let Some(t) = TEXTURES.with(|t| t.borrow().get(&texture.id).cloned()) {
            if let og_render::TextureSource::Encoded(u) = &t {
                let format = u.desc.format;
                if !formats.is_some_and(|f| f.supported.contains(&format)) {
                    return Fetch::Failed(format!(
                        "{}: transcoded to {} for another renderer than this engine's",
                        texture.id,
                        format.name()
                    ));
                }
            }
            return Fetch::Ready(t);
        }
        match texture.to_texture_data() {
            Ok(t) => Fetch::Ready(og_render::TextureSource::Rgba(t)),
            Err(_) => Fetch::Failed(format!(
                "{}: not transcoded; the Worker transcodes a bundle's Basis textures (src/web/basis.ts) before the engine starts",
                texture.id
            )),
        }
    }

    fn release_stream(&self, pack: &str, textures: &[String]) {
        FILES.with(|f| f.borrow_mut().remove(pack));
        if REQUESTED.with(|r| r.borrow().contains(pack)) {
            // Its textures may still be arriving: dropped when the pack lands.
            ORPHANED.with(|o| o.borrow_mut().insert(pack.to_string(), textures.to_vec()));
        } else {
            drop_textures(textures);
        }
    }

    fn inflate_start(&self, key: u64, zlib: &[u8]) -> bool {
        INFLATER.with(|f| match &*f.borrow() {
            Some(inflate) => inflate
                .call2(
                    &JsValue::NULL,
                    &JsValue::from_f64(key as f64),
                    &js_sys::Uint8Array::from(zlib),
                )
                .is_ok(),
            None => false,
        })
    }

    fn inflate_take(&self, key: u64) -> Fetch<Vec<u8>> {
        match INFLATED.with(|m| m.borrow_mut().remove(&key)) {
            Some(Ok(bytes)) => Fetch::Ready(bytes),
            Some(Err(e)) => Fetch::Failed(e),
            None => Fetch::Pending,
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

/// `cooked/stream/<name>.ogb`, as the Worker's fetcher tells a pack
/// (`/\/stream\/[^/]+\.ogb$/`).
fn is_stream_pack(path: &str) -> bool {
    path.rsplit_once('/').is_some_and(|(dir, name)| {
        dir.ends_with("/stream") && name.len() > ".ogb".len() && name.ends_with(".ogb")
    })
}

fn drop_textures(ids: &[String]) {
    TEXTURES.with(|t| {
        let mut t = t.borrow_mut();
        for id in ids {
            t.remove(id);
        }
    });
}

/// A fetch landed: kept for the engine, or dropped with its textures when
/// its group let go of it meanwhile.
fn land(path: String, file: Result<Vec<u8>, String>) {
    REQUESTED.with(|r| r.borrow_mut().remove(&path));
    if let Some(textures) = ORPHANED.with(|o| o.borrow_mut().remove(&path)) {
        drop_textures(&textures);
        return;
    }
    FILES.with(|f| f.borrow_mut().insert(path, file));
}

/// A cooked file the Worker fetched (`path` as the runtime asks for it:
/// `<root>/cooked/<file>`).
#[wasm_bindgen]
pub fn og_web_put_file(path: String, bytes: Vec<u8>) {
    land(path, Ok(bytes));
}

/// A cooked file that could not be fetched, with the reason.
#[wasm_bindgen]
pub fn og_web_fail_file(path: String, reason: String) {
    land(path, Err(reason));
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
    TEXTURES.with(|t| {
        t.borrow_mut()
            .insert(id, og_render::TextureSource::Rgba(data))
    });
}

/// A texture the Worker transcoded to the block format `format`
/// ([`og_web_texture_target`]'s answer): every mip level, level 0 first,
/// back to back in `levels`. Two-channel formats are linear whatever
/// `srgb` says. Answers `""`, or the reason the levels were refused.
#[wasm_bindgen]
pub fn og_web_put_texture_encoded(
    id: String,
    format: String,
    width: u32,
    height: u32,
    srgb: bool,
    levels: Vec<u8>,
) -> String {
    let Some(format) = og_render::TextureFormat::from_name(&format) else {
        return format!("{id}: {format} is not a texture format");
    };
    let mut split = Vec::new();
    let mut at = 0;
    let mut level = 0;
    while at < levels.len() && level < 32 {
        let n = format.level_bytes(width, height, level);
        let end = (at + n).min(levels.len());
        split.push(levels[at..end].to_vec());
        at = end;
        level += 1;
    }
    let color_space = if srgb && !format.two_channel() {
        og_render::ColorSpace::Srgb
    } else {
        og_render::ColorSpace::Linear
    };
    let upload = og_render::TextureUpload {
        desc: og_render::TextureDesc {
            width,
            height,
            format,
            mip_levels: split.len() as u32,
            color_space,
        },
        levels: split,
    };
    if let Err(e) = upload.desc.validate(&upload.levels) {
        return format!("{id}: {e}");
    }
    TEXTURES.with(|t| {
        t.borrow_mut()
            .insert(id, og_render::TextureSource::Encoded(upload))
    });
    String::new()
}

/// The encoding a cooked texture should reach `engine`'s renderer in
/// (og-cook's `CookedTexture::target_format` over the renderer's
/// `texture_formats`): `rgba8`, `bc7`, `bc5`, `astc4x4`, `etc2` or
/// `eacrg11`; `rgba8` before a renderer is attached or for a texture that
/// does not parse.
#[wasm_bindgen]
pub fn og_web_texture_target(engine: u32, manifest: String, id: String, bytes: Vec<u8>) -> String {
    let Ok(bundle) = og_cook::runtime::Bundle::from_manifest(&manifest) else {
        return "rgba8".into();
    };
    let words = aligned(&bytes);
    // SAFETY: the words hold `bytes.len()` initialised bytes.
    let view = unsafe { std::slice::from_raw_parts(words.as_ptr().cast::<u8>(), bytes.len()) };
    let formats = FORMATS.with(|f| f.borrow().get(&engine).cloned());
    match bundle.texture(&id, view) {
        Ok(t) => t.target_format(formats.as_ref()).name().into(),
        Err(_) => "rgba8".into(),
    }
}

/// The Worker's `inflate(key, zlibBytes)`, called to inflate a terrain
/// tile away from the engine (a DecompressionStream); it answers later with
/// `og_web_put_inflated` or `og_web_fail_inflated`.
#[wasm_bindgen]
pub fn og_web_set_inflater(inflate: js_sys::Function) {
    INFLATER.with(|f| *f.borrow_mut() = Some(inflate));
}

/// What an inflate the engine started answered.
#[wasm_bindgen]
pub fn og_web_put_inflated(key: f64, bytes: Vec<u8>) {
    INFLATED.with(|m| m.borrow_mut().insert(key as u64, Ok(bytes)));
}

/// An inflate that failed (the engine inflates that tile itself).
#[wasm_bindgen]
pub fn og_web_fail_inflated(key: f64, reason: String) {
    INFLATED.with(|m| m.borrow_mut().insert(key as u64, Err(reason)));
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
/// JSON `[{ "id", "file", "type" }]` in manifest order, with `"group"` on an
/// output of a streaming group (it is read from the group's pack, not from
/// its own file), or `{ "error": reason }` for a manifest that does not
/// parse.
#[wasm_bindgen]
pub fn og_web_bundle_files(manifest: String) -> String {
    match og_cook::runtime::Bundle::from_manifest(&manifest) {
        Ok(bundle) => {
            let items: Vec<String> = bundle
                .manifest()
                .outputs()
                .map(|o| {
                    let group = o
                        .group
                        .as_deref()
                        .map(|g| format!(",\"group\":{}", json_str(g)))
                        .unwrap_or_default();
                    format!(
                        "{{\"id\":{},\"file\":{},\"type\":{}{group}}}",
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

/// The Basis-coded textures of a streaming group's pack, for the Worker's
/// transcoder before it hands the pack to the engine: `[id, ktx2, target,
/// ...]` (triples; `target` is [`og_web_texture_target`]'s answer for
/// `engine`'s renderer, `rgba8` without one). Empty for a pack or manifest
/// that does not read (the runtime then fails the group with the reason)
/// and for RGBA8-coded textures.
#[wasm_bindgen]
pub fn og_web_pack_ktx2(engine: Option<u32>, manifest: String, pack: Vec<u8>) -> js_sys::Array {
    let out = js_sys::Array::new();
    let formats = engine.and_then(|e| FORMATS.with(|f| f.borrow().get(&e).cloned()));
    let (Ok(bundle), Ok(files)) = (
        og_cook::runtime::Bundle::from_manifest(&manifest),
        og_cook::runtime::unpack(&pack),
    ) else {
        return out;
    };
    for (file, bytes) in files {
        let Some(o) = bundle
            .manifest()
            .outputs()
            .find(|o| o.file == file && o.ty == "og.Texture")
        else {
            continue;
        };
        let words = aligned(&bytes);
        // SAFETY: the words hold `bytes.len()` initialised bytes.
        let view = unsafe { std::slice::from_raw_parts(words.as_ptr().cast::<u8>(), bytes.len()) };
        if let Ok(t) = bundle.texture(&o.id, view)
            && t.meta.codec != og_cook::format::codec::RGBA8
        {
            out.push(&JsValue::from_str(&o.id));
            out.push(&js_sys::Uint8Array::from(t.ktx2.as_slice()));
            out.push(&JsValue::from_str(t.target_format(formats.as_ref()).name()));
        }
    }
    out
}

/// The play surface the UI lays out in: `{ width, height, dpr, safeArea?,
/// coarsePointer? }` in CSS pixels. Answers wire JSON.
#[wasm_bindgen]
pub fn og_web_set_viewport(engine: u32, viewport: String) -> String {
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&viewport);
    match parsed
        .map_err(|e| e.to_string())
        .and_then(|v| og_runtime::set_viewport(engine, v))
    {
        Ok(()) => r#"{"ok":true}"#.to_string(),
        Err(e) => serde_json::json!({ "ok": false, "reason": e }).to_string(),
    }
}

/// Resize an engine's display target to `width`×`height` device pixels (the
/// shell's canvas after a resize or an orientation change). Applied before
/// the next rendered frame, never during one; a size of 0 is taken as 1.
#[wasm_bindgen]
pub fn og_web_resize(engine: u32, width: u32, height: u32) {
    RESIZES.with(|r| r.borrow_mut().insert(engine, (width.max(1), height.max(1))));
}

/// The size `og_web_resize` asked for since the last frame, taken once.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn take_resize(engine: u32) -> Option<(u32, u32)> {
    RESIZES.with(|r| r.borrow_mut().remove(&engine))
}

/// The lanes the module got, as JSON (og-runtime's `lanes_report`: on,
/// threads, pool, why, parallelCalls, parallelItems, sites): the engine Worker's
/// `lanes` call, so a publish page without the probe's commands can see them.
#[wasm_bindgen]
pub fn og_web_lanes() -> String {
    og_runtime::lanes_report().to_string()
}

/// A test hook: the next item the engine runs at lanes site `site` (`anim`,
/// `extract`, `cull`, `physics`) panics, inside a lane when the call splits
/// across the pool and on the engine's thread when it runs serially
/// (og_lanes::inject_fault). False for an unknown site. The browser test of
/// a lane's panic (`packages/engine/tests/web/lane-panic.ts`) calls it
/// through the engine Worker's `fault` call.
#[wasm_bindgen]
pub fn og_web_lanes_fault(site: &str) -> bool {
    match og_lanes::Site::from_name(site) {
        Some(s) => {
            og_lanes::inject_fault(s);
            true
        }
        None => false,
    }
}

/// The first Rust panic the engine recorded, as JSON (og-runtime's
/// `panic_report`: kind, message, location, tick, and lane, the lane's site
/// or null on the engine's thread), or `null`. The engine Worker reads it
/// after a trap, so the crash it reports names the panic, its tick and its
/// lane instead of `unreachable`.
#[wasm_bindgen]
pub fn og_web_panic() -> String {
    og_runtime::panic_report().to_string()
}

/// Routes Rust panics to the console instead of an opaque `unreachable`,
/// after recording them for `og_web_panic` (on any thread: a lane's panic
/// on a pool Worker wakes the engine thread waiting for it, og-lanes).
#[wasm_bindgen]
pub fn og_web_init() {
    std::panic::set_hook(Box::new(|info| {
        og_runtime::record_panic(info);
        let message = format!("origin-engine panicked: {info}");
        let console = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("console"));
        if let Ok(console) = console
            && let Ok(error) = js_sys::Reflect::get(&console, &JsValue::from_str("error"))
            && let Some(f) = error.dyn_ref::<js_sys::Function>()
        {
            let _ = f.call1(&console, &JsValue::from_str(&message));
        }
    }));
}

#[cfg(target_arch = "wasm32")]
mod canvas {
    use og_render::{Backend, RenderDesc, RenderScene, RenderTarget, Renderer, RgbaImage};
    use og_runtime::{RenderBackend, set_render_backend};
    use wasm_bindgen::prelude::*;

    struct CanvasBackend {
        engine: u32,
        renderer: Renderer,
        tier: String,
        /// Captures in flight: one per size, at most [`MAX_CAPTURES`],
        /// oldest first.
        pending: Vec<Pending>,
        /// The last frame's `FrameStats::missing_glyphs`.
        missing: u32,
    }

    struct Pending {
        req: og_render::CaptureRequest,
        /// The size it was asked at.
        size: (u32, u32),
        /// Landed by a display frame and not taken since.
        landed: bool,
    }

    /// The most captures of different sizes in flight at once; asking for
    /// another drops the oldest.
    const MAX_CAPTURES: usize = 4;

    impl RenderBackend for CanvasBackend {
        fn tier(&self) -> String {
            self.tier.clone()
        }
        fn texture_formats(&self) -> Option<og_render::TextureFormats> {
            Some(self.renderer.info().texture_formats.clone())
        }
        fn size(&self) -> (u32, u32) {
            self.renderer.size()
        }
        fn antialias(&self) -> String {
            og_runtime::antialias_name(self.renderer.info().antialias).into()
        }
        fn info(&self) -> Option<og_render::RenderInfo> {
            Some(self.renderer.info().clone())
        }
        fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
            if let Some((w, h)) = super::take_resize(self.engine) {
                self.renderer.resize(w, h);
            }
            self.renderer.poll();
            // Its asker takes a landed frame on its next call, before the
            // next display frame; one left a whole display frame was given up on.
            self.pending.retain_mut(|p| {
                let kept = !p.landed;
                p.landed = p.req.is_ready();
                kept
            });
            self.renderer
                .render(scene, &RenderTarget::Surface)
                .map(|s| self.missing = s.missing_glyphs)
                .map_err(|e| e.to_string())
        }
        fn missing_glyphs(&self) -> u32 {
            self.missing
        }
        /// Request on one call, answer on a later one: the first call draws
        /// offscreen and starts the readback; a later call at the same size
        /// answers the frame once the copy has landed (the scene as it was
        /// at the request). Captures of different sizes wait side by side.
        fn capture(&mut self, scene: &RenderScene, width: u32, height: u32) -> Option<RgbaImage> {
            let target = self.renderer.size();
            let size = if width == 0 || height == 0 {
                target
            } else {
                (width, height)
            };
            if let Some(i) = self.pending.iter().position(|p| p.size == size) {
                let mut p = self.pending.remove(i);
                match p.req.try_finish(&self.renderer) {
                    Ok(Some(image)) => return Some(image),
                    Ok(None) => {
                        self.pending.push(p);
                        return None;
                    }
                    // A failed readback is asked again.
                    Err(_) => {}
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
            if let Some(req) = started {
                if self.pending.len() >= MAX_CAPTURES {
                    self.pending.remove(0);
                }
                self.pending.push(Pending {
                    req,
                    size,
                    landed: false,
                });
            }
            None
        }
        fn deferred(&self) -> bool {
            true
        }
        fn save_state(&mut self) -> u64 {
            self.renderer.save_state().id()
        }
        fn restore_state(&mut self, state: u64) {
            self.renderer
                .restore_state(og_render::RendererState::from_id(state));
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
        let desc = og_runtime::render_desc(
            engine,
            RenderDesc {
                width: canvas.width().max(1),
                height: canvas.height().max(1),
                backend,
                ..RenderDesc::default()
            },
        );
        let instance = backend.instance();
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::OffscreenCanvas(canvas))
            .map_err(|e| JsValue::from_str(&format!("surface: {e}")))?;
        let renderer = Renderer::with_surface(&desc, &instance, surface)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let tier = renderer.info().backend_tier();
        let formats = renderer.info().texture_formats.clone();
        super::FORMATS.with(|f| f.borrow_mut().insert(engine, formats));
        set_render_backend(
            engine,
            Some(Box::new(CanvasBackend {
                engine,
                renderer,
                tier: tier.clone(),
                pending: Vec::new(),
                missing: 0,
            })),
        )
        .map_err(|e| JsValue::from_str(&e))?;
        Ok(tier)
    }
}
