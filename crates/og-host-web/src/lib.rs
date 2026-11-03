//! Origin Engine web host (`docs/design.md` §2.2, §3.1): the binding symbols
//! of `crates/og-api/api/<module>.ogt` as a wasm-bindgen module (`origin-engine.wasm`)
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
mod audio;
mod host;
mod host_depth;
mod host_e6;
mod host_p2;

use og_core::IdMap;
use og_runtime::{Dispatch, Fetch, Hooks};

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
        // again (a failure too) the next time its group loads. So is a
        // model: the runtime decodes it once into the renderer's meshes
        // (assets::load_gltf keeps the asset by path) and a refresh fetches
        // it again; kept, a large scene's models would sit in the wasm heap
        // twice, encoded beside decoded. og.motion's model is read once,
        // into the GPU's buffers.
        let known = if is_stream_pack(path)
            || is_sound_chunk(path)
            || is_model(path)
            || path.ends_with("/motion/model.ogb")
        {
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
        texture: &og_bundle::runtime::CookedTexture,
        formats: Option<&og_render::TextureFormats>,
    ) -> Fetch<og_render::TextureSource> {
        if TRANSCODING.with(|t| t.borrow().contains(&texture.id)) {
            return Fetch::Pending;
        }
        if let Some(e) = UNTRANSCODED.with(|u| u.borrow_mut().remove(&texture.id)) {
            return Fetch::Failed(e);
        }
        if let Some(t) = TEXTURES.with(|t| t.borrow().get(&texture.id).cloned()) {
            if let og_render::TextureSource::Encoded(u) = &t {
                let format = u.desc.format;
                if !formats.is_some_and(|f| f.supported.contains(&format)) {
                    // Transcoded for a device lost since: again, for this one.
                    return retranscode(texture, formats).unwrap_or_else(|| {
                        Fetch::Failed(format!(
                            "{}: transcoded to {} for another renderer than this engine's",
                            texture.id,
                            format.name()
                        ))
                    });
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

    fn image_decode_start(&self, key: u64, bytes: &[u8]) -> bool {
        IMAGE_DECODER.with(|f| match &*f.borrow() {
            Some(decode) => decode
                .call2(
                    &JsValue::NULL,
                    &JsValue::from_f64(key as f64),
                    &js_sys::Uint8Array::from(bytes),
                )
                .is_ok(),
            None => false,
        })
    }

    fn image_decode_take(&self, key: u64) -> Fetch<og_runtime::Decoded> {
        match DECODED_IMAGES.with(|m| m.borrow_mut().remove(&key)) {
            Some(Ok(d)) => Fetch::Ready(d),
            Some(Err(e)) => Fetch::Failed(e),
            None => Fetch::Pending,
        }
    }

    fn now_ms(&self) -> f64 {
        clock_ms()
    }

    fn video_decoder(&self) -> og_runtime::VideoDecoder {
        VIDEO
            .with(|v| v.borrow().as_ref().map(|(_, _, d)| d.clone()))
            .unwrap_or_default()
    }

    #[cfg(target_arch = "wasm32")]
    fn video_stream(&self, stream: u64, size: (u32, u32), table: &[u32], data: &[u8]) -> bool {
        VIDEO.with(|v| {
            let Some((put, _, _)) = &*v.borrow() else {
                return false;
            };
            let args = js_sys::Array::of5(
                &JsValue::from_f64(stream as f64),
                &JsValue::from(size.0),
                &JsValue::from(size.1),
                &js_sys::Uint32Array::from(table),
                &js_sys::Uint8Array::from(data),
            );
            put.apply(&JsValue::NULL, &args)
                .ok()
                .and_then(|r| r.as_bool())
                .unwrap_or(false)
        })
    }

    #[cfg(target_arch = "wasm32")]
    fn video_frame(
        &self,
        stream: u64,
        player: u64,
        frame: u32,
        looped: bool,
    ) -> Option<og_render::wgpu::web_sys::VideoFrame> {
        VIDEO.with(|v| {
            let (_, get, _) = v.borrow().as_ref().cloned()?;
            let args = js_sys::Array::of4(
                &JsValue::from_f64(stream as f64),
                &JsValue::from_f64(player as f64),
                &JsValue::from(frame),
                &JsValue::from_bool(looped),
            );
            let f = get.apply(&JsValue::NULL, &args).ok()?;
            (!f.is_null() && !f.is_undefined()).then(|| f.unchecked_into())
        })
    }
}

/// This thread's `performance.now()` (each Worker has its own, from its own
/// origin), or the date where there is none: the hooks' clock and
/// og-physics's for its per-phase stats, which a step on the lanes times on
/// the pool Worker it runs on.
pub(crate) fn clock_ms() -> f64 {
    PERFORMANCE.with(|p| match p {
        Some((performance, now)) => now
            .call0(performance)
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        None => js_sys::Date::now(),
    })
}

/// The page's time in milliseconds, alike on every Worker
/// (`performance.timeOrigin + performance.now()`): og-lanes' trace clock,
/// since each pool Worker's `performance.now()` counts from its own start.
pub(crate) fn epoch_ms() -> f64 {
    PERFORMANCE.with(|p| match p {
        Some((performance, now)) => {
            let origin = js_sys::Reflect::get(performance, &JsValue::from_str("timeOrigin"))
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            origin
                + now
                    .call0(performance)
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0)
        }
        None => js_sys::Date::now(),
    })
}

/// `cooked/stream/<name>.ogb`, as the Worker's fetcher tells a pack
/// (`/\/stream\/[^/]+\.ogb$/`).
fn is_stream_pack(path: &str) -> bool {
    path.rsplit_once('/').is_some_and(|(dir, name)| {
        dir.ends_with("/stream") && name.len() > ".ogb".len() && name.ends_with(".ogb")
    })
}

/// `cooked/<sound>.chunk<n>.ogb`: a chunk of a sound cooked in chunks,
/// read once each time its playhead nears it (taken, so the Worker never
/// holds a long track whole).
fn is_sound_chunk(path: &str) -> bool {
    path.strip_suffix(".ogb")
        .and_then(|p| p.rsplit_once(".chunk"))
        .is_some_and(|(_, n)| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

/// A cooked glTF model's own blob (`<asset>.gltf.ogb`, `<asset>.glb.ogb`).
fn is_model(path: &str) -> bool {
    path.ends_with(".gltf.ogb") || path.ends_with(".glb.ogb")
}

/// Hands a Basis texture to the Worker's transcoder for `formats` (a
/// device made again after a loss that does not take the encoding it was
/// transcoded to): `Pending` until the Worker answers. `None` without a
/// transcoder or for a texture that is not Basis-coded.
fn retranscode(
    texture: &og_bundle::runtime::CookedTexture,
    formats: Option<&og_render::TextureFormats>,
) -> Option<Fetch<og_render::TextureSource>> {
    if texture.meta.codec == og_bundle::format::codec::RGBA8 || texture.ktx2.is_empty() {
        return None;
    }
    TRANSCODER.with(|f| {
        let transcode = f.borrow();
        let transcode = transcode.as_ref()?;
        let target = texture.target_format(formats).name();
        let called = transcode.call3(
            &JsValue::NULL,
            &JsValue::from_str(&texture.id),
            &js_sys::Uint8Array::from(texture.ktx2.as_slice()),
            &JsValue::from_str(target),
        );
        Some(match called {
            Ok(_) => {
                TEXTURES.with(|t| t.borrow_mut().remove(&texture.id));
                TRANSCODING.with(|t| t.borrow_mut().insert(texture.id.clone()));
                Fetch::Pending
            }
            Err(e) => Fetch::Failed(format!("{}: the transcoder threw {e:?}", texture.id)),
        })
    })
}

/// The Worker's `transcode(id, ktx2, target)` (see [`TRANSCODER`]).
#[wasm_bindgen]
pub fn og_web_set_transcoder(transcode: js_sys::Function) {
    TRANSCODER.with(|f| *f.borrow_mut() = Some(transcode));
}

/// A texture the transcoder could not transcode again, with the reason.
#[wasm_bindgen]
pub fn og_web_fail_texture(id: String, reason: String) {
    TRANSCODING.with(|t| t.borrow_mut().remove(&id));
    UNTRANSCODED.with(|u| u.borrow_mut().insert(id, reason));
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
    TRANSCODING.with(|t| t.borrow_mut().remove(&id));
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
    let again = TRANSCODING.with(|t| t.borrow_mut().remove(&id));
    if let Err(e) = upload.desc.validate(&upload.levels) {
        let reason = format!("{id}: {e}");
        if again {
            UNTRANSCODED.with(|u| u.borrow_mut().insert(id, reason.clone()));
        }
        return reason;
    }
    TEXTURES.with(|t| {
        t.borrow_mut()
            .insert(id, og_render::TextureSource::Encoded(upload))
    });
    String::new()
}

/// The encoding a cooked texture should reach `engine`'s renderer in
/// (og-bundle's `CookedTexture::target_format` over the renderer's
/// `texture_formats`): `rgba8`, `bc7`, `bc5`, `astc4x4`, `etc2` or
/// `eacrg11`; `rgba8` before a renderer is attached or for a texture that
/// does not parse.
#[wasm_bindgen]
pub fn og_web_texture_target(engine: u32, manifest: String, id: String, bytes: Vec<u8>) -> String {
    let Some(bundle) = bundle_of(&manifest) else {
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

/// The Worker's streamed-video decoders (packages/engine/src/web/video.ts):
/// `stream(key, width, height, table, data)` takes a stream's AV1 units
/// once, `frame(stream, player, frame, looped)` answers the VideoFrame a
/// player shows now or null; `decoder` is what og.video.capabilities
/// reports (`webcodecs`, `rav1d-wasm` or `none`).
#[wasm_bindgen]
pub fn og_web_install_video(
    stream: js_sys::Function,
    frame: js_sys::Function,
    decoder: String,
    hardware: bool,
    max_width: u32,
    max_height: u32,
) {
    let d = og_runtime::VideoDecoder {
        name: decoder,
        hardware,
        max_width,
        max_height,
    };
    VIDEO.with(|v| *v.borrow_mut() = Some((stream, frame, d)));
}

/// The Worker's `decodeImage(key, bytes)`, called to decode an uncooked
/// image (og.load.imageData) away from the engine with the browser's own
/// decoders; it answers later with `og_web_put_image` or `og_web_fail_image`.
#[wasm_bindgen]
pub fn og_web_set_image_decoder(decode: js_sys::Function) {
    IMAGE_DECODER.with(|f| *f.borrow_mut() = Some(decode));
}

/// What an image decode answered: RGBA8 rows top to bottom.
#[wasm_bindgen]
pub fn og_web_put_image(key: f64, width: u32, height: u32, rgba: Vec<u8>) {
    DECODED_IMAGES.with(|m| {
        m.borrow_mut().insert(
            key as u64,
            Ok(og_runtime::Decoded {
                width,
                height,
                rgba,
            }),
        )
    });
}

/// An image the browser could not decode, and why.
#[wasm_bindgen]
pub fn og_web_fail_image(key: f64, reason: String) {
    DECODED_IMAGES.with(|m| m.borrow_mut().insert(key as u64, Err(reason)));
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
    match og_bundle::runtime::Bundle::from_manifest(&manifest) {
        Ok(bundle) => {
            let bundle = std::rc::Rc::new(bundle);
            BUNDLE.with(|cell| *cell.borrow_mut() = Some((manifest.clone(), bundle.clone())));
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

/// The bundle of `manifest`, parsed once and kept (s58): the texture calls
/// below run once or twice per texture of a boot with the same manifest,
/// and `Bundle::from_manifest` of a large one costs milliseconds. An empty
/// `manifest` names the one the last call parsed (the Worker then hands the
/// text over once); `None` when it does not parse, or when it is empty and
/// none was parsed yet.
fn bundle_of(manifest: &str) -> Option<std::rc::Rc<og_bundle::runtime::Bundle>> {
    BUNDLE.with(|cell| {
        let mut cell = cell.borrow_mut();
        if let Some((text, bundle)) = cell.as_ref()
            && (manifest.is_empty() || text.as_str() == manifest)
        {
            return Some(bundle.clone());
        }
        if manifest.is_empty() {
            return None;
        }
        let bundle = std::rc::Rc::new(og_bundle::runtime::Bundle::from_manifest(manifest).ok()?);
        *cell = Some((manifest.to_string(), bundle.clone()));
        Some(bundle)
    })
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
    let Some(bundle) = bundle_of(&manifest) else {
        return Vec::new();
    };
    let words = aligned(&bytes);
    // SAFETY: the words hold `bytes.len()` initialised bytes.
    let view = unsafe { std::slice::from_raw_parts(words.as_ptr().cast::<u8>(), bytes.len()) };
    match bundle.texture(&id, view) {
        Ok(t) if t.meta.codec != og_bundle::format::codec::RGBA8 => t.ktx2,
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
    let (Some(bundle), Ok(files)) = (bundle_of(&manifest), og_bundle::runtime::unpack(&pack))
    else {
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
            && t.meta.codec != og_bundle::format::codec::RGBA8
        {
            out.push(&JsValue::from_str(&o.id));
            out.push(&js_sys::Uint8Array::from(t.ktx2.as_slice()));
            out.push(&JsValue::from_str(t.target_format(formats.as_ref()).name()));
        }
    }
    out
}

/// The play surface the UI lays out in: `{ width, height, dpr, safeArea?,
/// coarsePointer?, reducedMotion?, highContrast?, reducedTransparency?,
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

/// The trace (og-runtime's `trace.rs`, the shell's `trace()`): turns the
/// engine's trace on or off.
#[wasm_bindgen]
pub fn og_web_trace(engine: u32, on: bool) {
    og_runtime::trace_set(engine, on);
}

/// The trace's spans since the last take, as JSON (`{ now, events }`, the
/// events Chrome trace events on the engine's clock); `last` waits for the
/// GPU's frames in flight where it can (natively only: the web's arrive in
/// a later take).
#[wasm_bindgen]
pub fn og_web_trace_take(engine: u32, last: bool) -> String {
    og_runtime::trace_take(engine, last)
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

/// Fork s60: the frame pipelines made with `createRenderPipelineAsync` (on)
/// or at once (off), and a draw whose pipeline is still compiling skipped
/// meanwhile (og-render `set_async_pipelines`); the Worker turns it on before
/// the first frame.
#[wasm_bindgen]
pub fn og_web_async_pipelines(on: bool) {
    og_render::set_async_pipelines(on);
}

/// Fork s60: `[pending, skipped, deferred]`: the pipelines still compiling,
/// the draws skipped for them since the last call, and (s63) the textures
/// still drawn from their stand-ins.
#[wasm_bindgen]
pub fn og_web_pipelines_state() -> Vec<u32> {
    let (pending, skipped) = og_render::async_pipelines_state();
    vec![pending, skipped, og_render::textures_deferred()]
}

/// Fork s63: the texture bytes a frame uploads at most (0: no limit); a
/// texture past it is drawn from a stand-in and uploaded by a later frame.
#[wasm_bindgen]
pub fn og_web_upload_budget(bytes: f64) {
    og_render::set_upload_budget(bytes as u64);
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
        /// The last frame's `FrameStats::triangles`.
        triangles: u64,
        /// Display frames drawn so far.
        drawn: u64,
        /// The readback of the last display frame (`og render`'s
        /// every-frame capture, `Capture::frame`): started by the frame
        /// once a reader has asked (the renderer keeps each presented frame
        /// offscreen from then on), answered once it lands.
        kept: Option<Kept>,
        /// The canvas, for a renderer on a new device after a loss.
        canvas: web_sys::OffscreenCanvas,
        /// Devices lost and the renderer made again on a new one.
        losses: u32,
        /// A new renderer is being made (the device was lost): frames
        /// draw nothing until it lands in [`RECOVERED`].
        recovering: bool,
    }

    thread_local! {
        /// Renderers made on a new device after a loss, by engine, taken by
        /// the next frame.
        static RECOVERED: std::cell::RefCell<og_core::IdMap<u32, Result<Renderer, String>>> =
            const { std::cell::RefCell::new(og_core::IdMap::new()) };
    }

    impl CanvasBackend {
        /// Recovery from a lost device (iOS Safari after the page was in
        /// the background or under memory pressure, a GPU reset, the test
        /// hook): true while there is no device to draw on. The first frame
        /// that finds the device lost asks for a new one on the same canvas
        /// (asynchronously: the browser's adapter and device are promises);
        /// the frames until it lands draw nothing while the simulation goes
        /// on, and the first frame after it draws the world as it is then,
        /// every mesh and texture uploaded again from the scene.
        fn recover_device(&mut self) -> Result<bool, String> {
            if let Some(made) = RECOVERED.with(|m| m.borrow_mut().remove(&self.engine)) {
                self.recovering = false;
                let renderer = made.map_err(|e| {
                    format!("the GPU device was lost and a new one could not be made: {e}")
                })?;
                super::FORMATS.with(|f| {
                    f.borrow_mut()
                        .insert(self.engine, renderer.info().texture_formats.clone())
                });
                self.renderer = renderer;
                self.losses += 1;
                self.pending.clear();
                self.kept = None;
            }
            let Some(why) = self.renderer.lost() else {
                return Ok(false);
            };
            if !self.recovering {
                self.recovering = true;
                super::console_warn(&format!(
                    "origin-engine: the GPU device was {why}; making the renderer again"
                ));
                let successor = self.renderer.successor();
                let canvas = self.canvas.clone();
                let engine = self.engine;
                wasm_bindgen_futures::spawn_local(async move {
                    let instance = Backend::WebGpu.instance();
                    let made = match instance
                        .create_surface(wgpu::SurfaceTarget::OffscreenCanvas(canvas))
                    {
                        Ok(surface) => successor
                            .make(Some((&instance, surface)))
                            .await
                            .map_err(|e| e.to_string()),
                        Err(e) => Err(format!("surface: {e}")),
                    };
                    RECOVERED.with(|m| m.borrow_mut().insert(engine, made));
                });
            }
            Ok(true)
        }
    }

    struct Kept {
        /// The display frame it reads.
        drawn: u64,
        req: Option<og_render::CaptureRequest>,
        image: Option<RgbaImage>,
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
        fn compute(&self) -> Option<(og_render::wgpu::Device, og_render::wgpu::Queue)> {
            Some((
                self.renderer.device().clone(),
                self.renderer.queue().clone(),
            ))
        }
        fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
            if self.recover_device()? {
                return Ok(());
            }
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
            match self.renderer.render(scene, &RenderTarget::Surface) {
                Ok(s) => {
                    self.triangles = s.triangles;
                }
                // Lost while drawing: recovered from the next frame on.
                Err(og_render::RenderError::DeviceLost(_)) => return Ok(()),
                Err(e) => return Err(e.to_string()),
            }
            self.drawn += 1;
            // A reader of every frame: this one's readback starts now (the
            // renderer copied it offscreen before presenting it).
            if self.renderer.keeps_frame() {
                self.kept = Some(Kept {
                    drawn: self.drawn,
                    req: self.renderer.begin_capture().ok(),
                    image: None,
                });
            }
            Ok(())
        }
        /// The display frame drawn last, once its readback has landed: the
        /// first call turns keeping on (and answers none: that frame was
        /// not kept), each later frame starts its own readback, and a call
        /// before it lands answers none (the reader asks again before the
        /// next frame, as a deferred capture does).
        fn read_frame(&mut self) -> Option<RgbaImage> {
            if !self.renderer.keeps_frame() {
                let _ = self.renderer.set_keep_frame(true);
                return None;
            }
            let k = self.kept.as_mut().filter(|k| k.drawn == self.drawn)?;
            if let Some(image) = &k.image {
                return Some(image.clone());
            }
            let image = k.req.as_mut()?.try_finish(&self.renderer).ok()??;
            k.req = None;
            k.image = Some(image.clone());
            Some(image)
        }
        fn set_clock(&mut self, clock: Option<fn() -> f64>) {
            self.renderer.set_clock(clock);
        }
        fn pass_timings(&self) -> Vec<og_render::PassTiming> {
            self.renderer.pass_timings().to_vec()
        }
        fn gpu_timings(&mut self, wait: bool) -> Vec<og_render::GpuFrame> {
            self.renderer.gpu_timings(wait)
        }
        fn gpu_timing_absent(&self) -> Option<&'static str> {
            self.renderer.gpu_timing_absent()
        }
        fn triangles(&self) -> u64 {
            self.triangles
        }
        fn post_errors(&self, scene: &RenderScene) -> Vec<(u32, String)> {
            self.renderer
                .post_errors(scene)
                .into_iter()
                .map(|(h, e)| (h.0, e))
                .collect()
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
            if self.recovering || self.renderer.lost().is_some() {
                return None;
            }
            if size != target {
                self.renderer.resize(size.0, size.1);
            }
            // A capture is the frame at its own size and scale 1, whatever
            // the display draws at (automatic quality, og.render.setDisplay).
            let scale = self.renderer.render_scale();
            self.renderer.set_render_scale(1.0);
            let started = self
                .renderer
                .render(scene, &RenderTarget::Offscreen)
                .ok()
                .and_then(|_| self.renderer.begin_capture().ok());
            self.renderer.set_render_scale(scale);
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
        fn warm(&mut self) -> bool {
            self.renderer.warm()
        }
        fn presents(&self) -> bool {
            true
        }
        fn set_render_scale(&mut self, scale: f32) {
            self.renderer.set_render_scale(scale);
        }
        fn scene_size(&self) -> (u32, u32) {
            self.renderer.scene_size_drawn()
        }
        fn gpu_memory(&self) -> Option<og_render::GpuMemory> {
            Some(self.renderer.gpu_memory())
        }
        fn recover(&mut self) -> Result<bool, String> {
            self.recover_device()
        }
        fn device_losses(&self) -> u32 {
            self.losses
        }
        fn lose_device(&mut self) -> bool {
            self.renderer.lose_device();
            true
        }
    }

    /// Put the renderer on an OffscreenCanvas transferred to this Worker,
    /// on WebGPU (the engine has no other web backend), its adapter asked
    /// for the low-power GPU with `low_power` (the player's power
    /// preference, kept by the shell from the last load). Resolves to the
    /// tier (`webgpu`), or rejects with the reason: no WebGPU adapter in
    /// this browser, or the canvas or device refused.
    #[wasm_bindgen]
    pub async fn og_web_attach_canvas(
        engine: u32,
        canvas: web_sys::OffscreenCanvas,
        low_power: bool,
    ) -> Result<String, JsValue> {
        if !wgpu::util::is_browser_webgpu_supported().await {
            return Err(JsValue::from_str(
                "no WebGPU adapter: this browser offers no WebGPU (navigator.gpu), which the engine needs",
            ));
        }
        let backend = Backend::WebGpu;
        let desc = og_runtime::render_desc(
            engine,
            RenderDesc {
                width: canvas.width().max(1),
                height: canvas.height().max(1),
                backend,
                low_power,
                ..RenderDesc::default()
            },
        );
        let instance = backend.instance();
        let canvas_kept = canvas.clone();
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::OffscreenCanvas(canvas))
            .map_err(|e| JsValue::from_str(&format!("surface: {e}")))?;
        let renderer = Renderer::with_surface(&desc, &instance, surface)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let tier = renderer.info().backend_tier();
        let formats = renderer.info().texture_formats.clone();
        super::FORMATS.with(|f| f.borrow_mut().insert(engine, formats));
        let device = renderer.device();
        let gpu = super::gpu_report(
            &device.limits(),
            device.features(),
            renderer.info().device_type == og_render::wgpu::DeviceType::Cpu,
        );
        super::GPUS.with(|g| g.borrow_mut().insert(engine, gpu.to_string()));
        set_render_backend(
            engine,
            Some(Box::new(CanvasBackend {
                engine,
                renderer,
                tier: tier.clone(),
                pending: Vec::new(),
                drawn: 0,
                kept: None,
                triangles: 0,
                canvas: canvas_kept,
                losses: 0,
                recovering: false,
            })),
        )
        .map_err(|e| JsValue::from_str(&e))?;
        og_runtime::set_power_preference(engine, low_power);
        Ok(tier)
    }
}

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
