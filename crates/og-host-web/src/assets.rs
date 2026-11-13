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
