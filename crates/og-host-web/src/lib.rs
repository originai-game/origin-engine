//! Origin Engine web host (`docs/design.md` §2.2, §3.1): the binding symbols
//! of `crates/og-api/api/<module>.ogt` as a wasm-bindgen module (`origin-engine.wasm`)
//! over an `og_ecs::World` and the renderer. It runs inside a dedicated
//! Worker; the main thread transfers it an OffscreenCanvas and pumps input
//! (`packages/engine/src/web/`).
//!
//! - `src/generated/glue.rs` (`og api gen`) exports every symbol and calls
//!   the `og_api::host` trait method of the same name; `src/host.rs`
//!   states the web wire types and expands the traits' shared
//!   implementation (`og_runtime::host_impl`), so a member missing there is
//!   a build error.
//! - Column views are byte offsets into the module's memory; the Worker
//!   loader makes typed arrays over `memory.buffer` at them, so a system
//!   reads and writes engine memory without a copy.
//! - The dispatcher is a JavaScript function installed once; the engine calls
//!   it for every system, command and facts call.
//! - `og_web_attach_canvas` puts the renderer on an OffscreenCanvas
//!   (WebGPU, the only web backend); `og_engine_frame` then ticks and
//!   renders.
//! - og-probe is mounted by the runtime; the Worker hands each frame of the
//!   shell's request ring to `og_engine_call`. A capture renders offscreen
//!   and starts the readback; the frame is answered by the next call once the
//!   copy has landed (requested on tick N, answered on N + 1), so the ring's
//!   `capture` is retried by the Worker until it answers.
//! - Cooked files reach the runtime through `og_web_put_file` (the Worker
//!   fetches the game's bundle before creating the engine); once the
//!   renderer is attached, the Worker asks `og_web_texture_target` which
//!   encoding each Basis texture should reach this device in (the
//!   renderer's `texture_formats`, og-bundle's choice by kind), transcodes it
//!   with Binomial's Basis transcoder (`src/web/basis.ts`) and hands it over
//!   with `og_web_put_texture` (RGBA8 level 0) or
//!   `og_web_put_texture_encoded` (every level of a block format).

/// The exports, one per binding symbol (`og api gen`).
#[doc(hidden)]
#[rustfmt::skip]
#[path = "generated/glue.rs"]
pub mod glue;
mod assets;
mod audio;
mod hooks;
mod host;

pub(crate) use hooks::{WebHooks, clock_ms, epoch_ms};
use og_core::IdMap;

/// The engine's heap, counted for the probe's `memory` (og-core's `alloc`).
#[cfg(target_arch = "wasm32")]
#[global_allocator]
static HEAP: og_runtime::CountingAlloc<std::alloc::System> =
    og_runtime::CountingAlloc(std::alloc::System);
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
    static FORMATS: RefCell<IdMap<u32, og_render::TextureFormats>> = const { RefCell::new(IdMap::new()) };
    /// The device each engine's attached renderer runs on, as `og_web_gpu` answers it.
    static GPUS: RefCell<IdMap<u32, String>> = const { RefCell::new(IdMap::new()) };
    /// Paths handed to the fetcher and not answered yet, so each is asked for once at a time.
    static REQUESTED: RefCell<std::collections::BTreeSet<String>> = const { RefCell::new(std::collections::BTreeSet::new()) };
    /// Packs let go of while they were fetched, with their group's texture
    /// ids: dropped, textures and all, when they land.
    static ORPHANED: RefCell<BTreeMap<String, Vec<String>>> = const { RefCell::new(BTreeMap::new()) };
    /// Render-target sizes the shell asked for, by engine, applied before the next frame.
    static RESIZES: RefCell<IdMap<u32, (u32, u32)>> = const { RefCell::new(IdMap::new()) };
    /// The bundle the texture calls last parsed, with its manifest text
    /// (s58): a boot's hundreds of texture calls parse the 0.5 MB manifest
    /// once, not twice per texture.
    static BUNDLE: RefCell<Option<(String, std::rc::Rc<og_bundle::runtime::Bundle>)>> = const { RefCell::new(None) };
    /// The Worker's frame-pipeline span (fork s60, `og_web_frame_pipelines`).
    static FRAME_SPAN: RefCell<Option<js_sys::Function>> = const { RefCell::new(None) };
    /// The Worker's `fetch(path)` for files nobody fetched yet.
    static FETCHER: RefCell<Option<js_sys::Function>> = const { RefCell::new(None) };
    /// The Worker's `transcode(id, ktx2, target)`, which transcodes a Basis
    /// texture again for a device that came back with other encodings
    /// after a loss and answers with `og_web_put_texture*` or
    /// `og_web_fail_texture`.
    static TRANSCODER: RefCell<Option<js_sys::Function>> = const { RefCell::new(None) };
    /// Textures handed to the transcoder and not answered yet, and those
    /// it failed, with the reason.
    static TRANSCODING: RefCell<std::collections::BTreeSet<String>> = const { RefCell::new(std::collections::BTreeSet::new()) };
    static UNTRANSCODED: RefCell<BTreeMap<String, String>> = const { RefCell::new(BTreeMap::new()) };
    /// The Worker's streamed-video decoders (`og_web_install_video`): its
    /// `stream(key, width, height, table, data)` and `frame(stream, player,
    /// frame, looped)`, and the decoder it chose.
    static VIDEO: RefCell<Option<(js_sys::Function, js_sys::Function, og_runtime::VideoDecoder)>> = const { RefCell::new(None) };
    /// The Worker's `inflate(key, bytes)` (DecompressionStream), and what it answered by key.
    static INFLATER: RefCell<Option<js_sys::Function>> = const { RefCell::new(None) };
    static INFLATED: RefCell<IdMap<u64, Result<Vec<u8>, String>>> = const { RefCell::new(IdMap::new()) };
    /// The Worker's `decodeImage(key, bytes)` (createImageBitmap), and what it answered by key.
    static IMAGE_DECODER: RefCell<Option<js_sys::Function>> = const { RefCell::new(None) };
    static DECODED_IMAGES: RefCell<IdMap<u64, Result<og_runtime::Decoded, String>>> = const { RefCell::new(IdMap::new()) };
    /// `performance` and its `now`, where the global has them (Workers do).
    static PERFORMANCE: Option<(JsValue, js_sys::Function)> = {
        let performance = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("performance")).ok();
        performance.and_then(|p| {
            let now = js_sys::Reflect::get(&p, &JsValue::from_str("now")).ok()?;
            Some((p, now.dyn_into::<js_sys::Function>().ok()?))
        })
    };
}

/// The play surface the UI lays out in: `{ width, height, dpr, safeArea?,
/// smallWidth?, smallHeight?, largeWidth?, largeHeight?, coarsePointer?, reducedMotion?, highContrast?, reducedTransparency?,
/// macKeys? }` in CSS pixels (reducedMotion, highContrast and
/// reducedTransparency the page's `prefers-reduced-motion`,
/// `prefers-contrast` and `prefers-reduced-transparency`; macKeys an Apple
/// platform's editing keys in text controls).
/// Answers wire JSON.
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

/// The engine is destroyed: what its attached renderer recorded and a
/// resize it never drew are dropped.
pub(crate) fn forget_engine(engine: u32) {
    FORMATS.with(|f| f.borrow_mut().remove(&engine));
    GPUS.with(|g| g.borrow_mut().remove(&engine));
    RESIZES.with(|r| r.borrow_mut().remove(&engine));
}

/// What the engine holds (og-runtime's `memory_report`, the probe's
/// `memory`: the wasm linear memory, the counted heap, the world, the
/// streaming budget, the audio), as JSON: the engine Worker's `memory` call,
/// which a publish page answers without the probe's dev members.
#[wasm_bindgen]
pub fn og_web_memory(engine: u32) -> String {
    og_runtime::memory_report(engine).to_string()
}

/// The WebGPU limits a crash report and a boot sample carry (field telemetry,
/// docs/web-export.md §Telemetry; `shell/telemetry.ts` GPU_LIMITS), each with
/// the device's value.
const GPU_LIMITS: [&str; 14] = [
    "maxTextureDimension2D",
    "maxTextureArrayLayers",
    "maxBindGroups",
    "maxSampledTexturesPerShaderStage",
    "maxStorageBuffersPerShaderStage",
    "maxStorageTexturesPerShaderStage",
    "maxUniformBufferBindingSize",
    "maxStorageBufferBindingSize",
    "maxBufferSize",
    "maxVertexBuffers",
    "maxColorAttachments",
    "maxColorAttachmentBytesPerSample",
    "maxComputeWorkgroupStorageSize",
    "maxComputeInvocationsPerWorkgroup",
];

/// The device the renderer runs on, for field telemetry: its limits by their
/// WebGPU names (GPU_LIMITS), the WebGPU features it was created with, and
/// whether its adapter is a fallback (software) one. The vendor and
/// architecture are the Worker's to add (wgpu does not carry them on the web).
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn gpu_report(
    limits: &og_render::wgpu::Limits,
    features: og_render::wgpu::Features,
    fallback: bool,
) -> serde_json::Value {
    let l = limits;
    let values: [u64; 14] = [
        l.max_texture_dimension_2d.into(),
        l.max_texture_array_layers.into(),
        l.max_bind_groups.into(),
        l.max_sampled_textures_per_shader_stage.into(),
        l.max_storage_buffers_per_shader_stage.into(),
        l.max_storage_textures_per_shader_stage.into(),
        l.max_uniform_buffer_binding_size,
        l.max_storage_buffer_binding_size,
        l.max_buffer_size,
        l.max_vertex_buffers.into(),
        l.max_color_attachments.into(),
        l.max_color_attachment_bytes_per_sample.into(),
        l.max_compute_workgroup_storage_size.into(),
        l.max_compute_invocations_per_workgroup.into(),
    ];
    let named = [
        (
            "texture-compression-bc",
            og_render::wgpu::Features::TEXTURE_COMPRESSION_BC,
        ),
        (
            "texture-compression-astc",
            og_render::wgpu::Features::TEXTURE_COMPRESSION_ASTC,
        ),
        (
            "texture-compression-etc2",
            og_render::wgpu::Features::TEXTURE_COMPRESSION_ETC2,
        ),
        ("shader-f16", og_render::wgpu::Features::SHADER_F16),
        (
            "timestamp-query",
            og_render::wgpu::Features::TIMESTAMP_QUERY,
        ),
        (
            "float32-filterable",
            og_render::wgpu::Features::FLOAT32_FILTERABLE,
        ),
        (
            "rg11b10ufloat-renderable",
            og_render::wgpu::Features::RG11B10UFLOAT_RENDERABLE,
        ),
        (
            "indirect-first-instance",
            og_render::wgpu::Features::INDIRECT_FIRST_INSTANCE,
        ),
        (
            "depth-clip-control",
            og_render::wgpu::Features::DEPTH_CLIP_CONTROL,
        ),
        ("subgroups", og_render::wgpu::Features::SUBGROUP),
    ];
    serde_json::json!({
        "fallback": fallback,
        "limits": GPU_LIMITS.iter().zip(values).map(|(k, v)| (k.to_string(), serde_json::json!(v))).collect::<serde_json::Map<_, _>>(),
        "features": named.iter().filter(|(_, f)| features.contains(*f)).map(|(n, _)| *n).collect::<Vec<_>>(),
    })
}

/// The device the engine's renderer runs on (`gpu_report`), as JSON, or
/// `null` before a canvas is attached: the Worker reads it after attaching
/// and sends it with `ready` when the page counts field telemetry.
#[wasm_bindgen]
pub fn og_web_gpu(engine: u32) -> String {
    GPUS.with(|g| g.borrow().get(&engine).cloned())
        .unwrap_or_else(|| "null".into())
}

/// The display's refresh interval the Worker measured (its
/// requestAnimationFrame's), ms: automatic quality's budget where the frame
/// cap is the display's rate (og-runtime's `display.rs`).
#[wasm_bindgen]
pub fn og_web_display_interval(engine: u32, ms: f64) {
    og_runtime::set_display_interval(engine, ms);
}

/// Brackets the probe's `frame` (a harness stepping one tick and one
/// render): a scripted frame draws at the configured quality and is no
/// sample of automatic quality (og-runtime's `set_scripted_frames`).
#[wasm_bindgen]
pub fn og_web_scripted_frame(engine: u32, on: bool) {
    og_runtime::set_scripted_frames(engine, on);
}

/// The page went fullscreen or left it (the shell's `fullscreenchange`):
/// og.render.display's `fullscreen`, which the player's setting follows.
#[wasm_bindgen]
pub fn og_web_fullscreen(engine: u32, on: bool) {
    og_runtime::set_fullscreen(engine, on);
}

/// What the browser granted the pointer lock: raw movement now (`active`),
/// and whether it refused unadjustedMovement (`unsupported`): og.input.mouse.
#[wasm_bindgen]
pub fn og_web_mouse_report(engine: u32, active: bool, unsupported: bool) {
    og_runtime::set_mouse_report(engine, active, unsupported);
}

/// A test hook: destroy the engine's GPU device, as iOS Safari does to a
/// page left in the background; the next frame finds it lost and the
/// renderer is made again on a new device, the game going on. The engine
/// Worker's `loseDevice` call (the browser check of device loss). False
/// without a renderer.
#[wasm_bindgen]
pub fn og_web_lose_device(engine: u32) -> bool {
    og_runtime::lose_device(engine)
}

/// Make one pipeline the engine's renderer has not used yet (og-render's
/// `Renderer::warm`): the engine Worker calls it in idle time behind the
/// play gate, until it answers false, so the pipelines of the passes the
/// first frame did not draw compile before play rather than in a frame of
/// it.
#[wasm_bindgen]
pub fn og_web_warm(engine: u32) -> bool {
    og_runtime::warm_renderer(engine)
}

/// Fork s60: the Worker's `span(on)`, called when the renderer begins and
/// ends making its frame pipelines (og-render `set_frame_pipelines_hook`);
/// `null` calls nothing. The Worker makes the pipelines asked for in between
/// with `createRenderPipelineAsync` (src/web/pipelines.ts).
#[wasm_bindgen]
pub fn og_web_frame_pipelines(span: Option<js_sys::Function>) {
    let on = span.is_some();
    FRAME_SPAN.with(|cell| *cell.borrow_mut() = span);
    og_render::set_frame_pipelines_hook(on.then_some(frame_span as fn(bool)));
}

fn frame_span(on: bool) {
    FRAME_SPAN.with(|cell| {
        if let Some(f) = cell.borrow().as_ref() {
            let _ = f.call1(&JsValue::NULL, &JsValue::from_bool(on));
        }
    });
}

/// Fork s63: the texture bytes a frame uploads at most (0: no limit); a
/// texture past it is drawn from a stand-in and uploaded by a later frame.
#[wasm_bindgen]
pub fn og_web_upload_budget(bytes: f64) {
    og_render::set_upload_budget(bytes as u64);
}

/// Fork s63: the textures still drawn from their stand-ins after the last
/// frame (the Worker draws the first frame again until none is left).
#[wasm_bindgen]
pub fn og_web_textures_deferred() -> u32 {
    og_render::textures_deferred()
}

/// Font-pack faces the locale's chain still waits for (docs/ui.md §3.4):
/// the Worker draws the first frame again until none is left.
#[wasm_bindgen]
pub fn og_web_fonts_pending() -> u32 {
    og_runtime::fonts_pending()
}

/// Fork s70: the page's loading screen covers the canvas (on) or no longer
/// does (off); a covered frame prepares without drawing (og-render
/// `set_covered`).
#[wasm_bindgen]
pub fn og_web_covered(on: bool) {
    og_render::set_covered(on);
}

/// `og render`'s every-frame capture on the web, as the native host's
/// `og_node_render_frame` (development only): the display frame drawn last
/// as RGBA8 rows at the canvas's size, read back without drawing it again
/// (the probe's `Capture::frame`). Empty until its readback lands: the
/// first call turns the keeping of presented frames on, and a later one
/// answers once that frame's copy has landed, so the Worker asks again
/// before it draws the next frame.
#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_web_render_frame(engine: u32) -> Vec<u8> {
    og_runtime::render_frame(engine)
        .map(|f| f.rgba)
        .unwrap_or_default()
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

/// `console.warn(message)` on this thread.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn console_warn(message: &str) {
    let console = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("console"));
    if let Ok(console) = console
        && let Ok(warn) = js_sys::Reflect::get(&console, &JsValue::from_str("warn"))
        && let Some(f) = warn.dyn_ref::<js_sys::Function>()
    {
        let _ = f.call1(&console, &JsValue::from_str(message));
    }
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
mod canvas;

#[cfg(test)]
mod gpu_tests {
    #[test]
    fn the_gpu_report_names_webgpu_limits_and_the_features_the_device_has() {
        let limits = og_render::wgpu::Limits {
            max_texture_dimension_2d: 16384,
            max_buffer_size: 1 << 32,
            ..og_render::wgpu::Limits::downlevel_defaults()
        };
        let features = og_render::wgpu::Features::TEXTURE_COMPRESSION_ASTC
            | og_render::wgpu::Features::SHADER_F16;
        let r = super::gpu_report(&limits, features, false);
        assert_eq!(r["limits"]["maxTextureDimension2D"], 16384);
        assert_eq!(r["limits"]["maxBufferSize"], 1u64 << 32);
        assert_eq!(r["limits"].as_object().unwrap().len(), 14);
        assert_eq!(
            r["features"],
            serde_json::json!(["texture-compression-astc", "shader-f16"])
        );
        assert_eq!(r["fallback"], false);
    }
}
