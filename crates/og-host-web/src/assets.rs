//! What the Worker hands the engine: cooked files, textures, inflated
//! and decoded bytes, video frames, and the bundle the texture calls read.

use crate::{
    BUNDLE, DECODED_IMAGES, FETCHER, FILES, FORMATS, IMAGE_DECODER, INFLATED, INFLATER, ORPHANED,
    REQUESTED, TEXTURES, TRANSCODER, TRANSCODING, UNTRANSCODED, VIDEO,
};
use og_runtime::Fetch;
use wasm_bindgen::prelude::*;

/// `cooked/stream/<name>.ogb`, as the Worker's fetcher tells a pack
/// (`/\/stream\/[^/]+\.ogb$/`).
pub(crate) fn is_stream_pack(path: &str) -> bool {
    path.rsplit_once('/').is_some_and(|(dir, name)| {
        dir.ends_with("/stream") && name.len() > ".ogb".len() && name.ends_with(".ogb")
    })
}

/// `cooked/<sound>.chunk<n>.ogb`: a chunk of a sound cooked in chunks,
/// read once each time its playhead nears it (taken, so the Worker never
/// holds a long track whole).
pub(crate) fn is_sound_chunk(path: &str) -> bool {
    path.strip_suffix(".ogb")
        .and_then(|p| p.rsplit_once(".chunk"))
        .is_some_and(|(_, n)| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

/// A cooked glTF model's own blob (`<asset>.gltf.ogb`, `<asset>.glb.ogb`).
pub(crate) fn is_model(path: &str) -> bool {
    path.ends_with(".gltf.ogb") || path.ends_with(".glb.ogb")
}

/// Hands a Basis texture to the Worker's transcoder for `formats` (a
/// device made again after a loss that does not take the encoding it was
/// transcoded to): `Pending` until the Worker answers. `None` without a
/// transcoder or for a texture that is not Basis-coded.
pub(crate) fn retranscode(
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

pub(crate) fn drop_textures(ids: &[String]) {
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
    crate::PROGRESS.with(|p| p.borrow_mut().remove(&path));
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

/// How much of a file the Worker is fetching has arrived: `received`
/// bytes of `total` (0 until the response names its length). Only what
/// the engine asked for counts; a progress for a file that landed or was
/// never asked for is dropped.
#[wasm_bindgen]
pub fn og_web_file_progress(path: String, received: f64, total: f64) {
    if REQUESTED.with(|r| r.borrow().contains(&path)) {
        crate::PROGRESS.with(|p| p.borrow_mut().insert(path, (received, total)));
    }
}

/// The Worker's loading screen: `report(engine, stage, fraction, label)`
/// for og.loading.stage and og.loading.done (stage `done`), and
/// `idle(engine)`, true while the renderer has nothing waiting.
#[wasm_bindgen]
pub fn og_web_set_loading(report: js_sys::Function, idle: js_sys::Function) {
    crate::LOADING.with(|l| *l.borrow_mut() = Some((report, idle)));
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

/// The Worker's streamed-video decoders and the decoder it chose (read by
/// the wasm32 hooks only).
#[derive(Clone)]
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) struct VideoHost {
    pub stream: js_sys::Function,
    pub frame: js_sys::Function,
    pub chunk: js_sys::Function,
    pub decoder: og_runtime::VideoDecoder,
}

/// The Worker's streamed-video decoders (packages/engine/src/web/video.ts):
/// `stream(key, width, height, codec, config, table, chunkFrames)` takes a
/// stream's index once (`codec` and `config` empty for AV1, else an H.264
/// or HEVC rendition's codec string and record; `table` each frame's
/// offset in its chunk, size and key), `chunk(stream, n, data)` a chunk's
/// units as they land (`data` null: let go), `frame(stream, player, frame,
/// looped)` answers the VideoFrame a player shows now or null; `decoder`
/// is what og.video.capabilities reports (`webcodecs`, `rav1d-wasm` or
/// `none`), and `codecs` the other codecs WebCodecs decodes (`hevc`,
/// `h264`), preferred first.
#[wasm_bindgen]
#[allow(clippy::too_many_arguments)]
pub fn og_web_install_video(
    stream: js_sys::Function,
    frame: js_sys::Function,
    chunk: js_sys::Function,
    decoder: String,
    hardware: bool,
    max_width: u32,
    max_height: u32,
    codecs: Vec<String>,
) {
    let decoder = og_runtime::VideoDecoder {
        name: decoder,
        hardware,
        max_width,
        max_height,
        codecs,
    };
    VIDEO.with(|v| {
        *v.borrow_mut() = Some(VideoHost {
            stream,
            frame,
            chunk,
            decoder,
        })
    });
}

/// The Worker's `decodeImage(key, bytes)`, called to decode an uncooked
/// image (og.load.imageData) away from the engine with the browser's own
/// decoders; it answers later with `og_web_put_image` or `og_web_fail_image`.
#[wasm_bindgen]
pub fn og_web_set_image_decoder(decode: js_sys::Function) {
    IMAGE_DECODER.with(|f| *f.borrow_mut() = Some(decode));
}

/// The Worker's `fetchImage(key, url, integrity)`, called for a
/// document's http(s) image (og-runtime's world/remote.rs): it `fetch()`es
/// the URL in CORS mode, without credentials or a referrer, with the
/// integrity metadata given (empty: none), decodes it with the browser, and answers with `og_web_put_image` or `og_web_fail_image`.
#[wasm_bindgen]
pub fn og_web_set_remote_fetcher(fetch: js_sys::Function) {
    crate::REMOTE_FETCHER.with(|f| *f.borrow_mut() = Some(fetch));
}

/// The Worker's `fetchText(key, url, integrity)`, called for a remote `<iframe>`'s document
/// (og-runtime's world/remote.rs): it `fetch()`es the URL as
/// `og_web_set_remote_fetcher`'s does and answers with the response's
/// bytes through `og_web_put_text`, or `og_web_fail_text`. Only in a
/// module with remote frames (the `frames` feature); the Worker installs
/// it when the module exports it.
#[cfg(feature = "frames")]
#[wasm_bindgen]
pub fn og_web_set_remote_text_fetcher(fetch: js_sys::Function) {
    crate::REMOTE_TEXT_FETCHER.with(|f| *f.borrow_mut() = Some(fetch));
}

/// The bytes a remote document's fetch answered.
#[cfg(feature = "frames")]
#[wasm_bindgen]
pub fn og_web_put_text(key: f64, bytes: Vec<u8>) {
    crate::FETCHED_TEXTS.with(|m| m.borrow_mut().insert(key as u64, Ok(bytes)));
}

/// A remote document that did not arrive, and why.
#[cfg(feature = "frames")]
#[wasm_bindgen]
pub fn og_web_fail_text(key: f64, reason: String) {
    crate::FETCHED_TEXTS.with(|m| m.borrow_mut().insert(key as u64, Err(reason)));
}

/// What an image decode answered: RGBA8 rows top to bottom as the file
/// stores them, and the EXIF orientation it names for them (1: upright).
#[wasm_bindgen]
pub fn og_web_put_image(key: f64, width: u32, height: u32, rgba: Vec<u8>, orientation: u32) {
    DECODED_IMAGES.with(|m| {
        m.borrow_mut().insert(
            key as u64,
            Ok(og_runtime::Decoded {
                width,
                height,
                rgba,
                orientation,
            }),
        )
    });
}

/// The EXIF orientation (1 to 8) a fetched PNG, JPEG or WebP's metadata
/// names: the native host's reader (og-runtime's `exif_orientation`), so
/// the Worker turns `createImageBitmap`'s upright texels back as stored
/// and both hosts hold the same texels and orientation.
#[wasm_bindgen]
pub fn og_web_image_orientation(bytes: &[u8]) -> u32 {
    og_runtime::exif_orientation(bytes)
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

use og_runtime::host_impl::json_string;

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
                        .map(|g| format!(",\"group\":{}", json_string(g)))
                        .unwrap_or_default();
                    format!(
                        "{{\"id\":{},\"file\":{},\"type\":{}{group}}}",
                        json_string(&o.id),
                        json_string(&o.file),
                        json_string(&o.ty)
                    )
                })
                .collect();
            format!("[{}]", items.join(","))
        }
        Err(e) => format!("{{\"error\":{}}}", json_string(&e.to_string())),
    }
}

/// Each file of `manifest`'s bundle under `cooked/` with its Subresource
/// Integrity metadata, as JSON `{ "<file>": "sha384-…" }`: every output,
/// streaming group pack and og.motion's model. A boot from an asset base
/// (docs/web-export.md §Asset base) fetches each file with its integrity,
/// so the browser refuses bytes that differ from what was cooked. An empty
/// `manifest` names the one last parsed; `{ "error" }` when it does not parse.
#[wasm_bindgen]
pub fn og_web_bundle_integrity(manifest: String) -> String {
    let Some(bundle) = bundle_of(&manifest) else {
        return format!(
            "{{\"error\":{}}}",
            json_string("the cook's manifest does not parse")
        );
    };
    let m = bundle.manifest();
    let files = m
        .outputs()
        .map(|o| (&o.file, &o.integrity))
        .chain(m.streams.iter().map(|g| (&g.file, &g.integrity)))
        .chain(m.motion.iter().map(|f| (&f.file, &f.integrity)));
    let items: std::collections::BTreeMap<_, _> = files.collect();
    let items: Vec<String> = items
        .into_iter()
        .map(|(f, i)| format!("{}:{}", json_string(f), json_string(i)))
        .collect();
    format!("{{{}}}", items.join(","))
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
