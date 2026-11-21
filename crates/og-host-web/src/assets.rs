//! What the Worker hands the engine: cooked files, the textures its
//! transcoders prepared, inflated and decoded bytes, video frames, and the
//! bundle the integrity call reads.

use crate::{
    BUNDLE, DECODED_IMAGES, FETCHER, FILES, IMAGE_DECODER, INFLATED, INFLATER, NORMAL_XYRA,
    ORPHANED, REQUESTED, TEXTURES, TRANSCODER, TRANSCODING, UNTRANSCODED, VIDEO,
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

/// A prepared texture's key: its id and og-bundle's
/// `CookedTexture::prepared_key` for the encodings asked for, the key the
/// native cache keeps it under too.
pub(crate) type TextureKey = (String, u64);

/// The key as the Worker's transcoder carries it: `<hash, 16 hex>:<id>`.
fn job_name(key: &TextureKey) -> String {
    format!("{:016x}:{}", key.1, key.0)
}

fn job_key(name: &str) -> Option<TextureKey> {
    let (hash, id) = name.split_once(':')?;
    Some((id.to_string(), u64::from_str_radix(hash, 16).ok()?))
}

/// The texture hook (`Hooks::texture`): the one way a cooked texture is
/// prepared on the web, as the native cache prepares one. An RGBA8-coded
/// texture is decoded at once. A Basis one is answered from what the
/// Worker's transcoders prepared for its key, or handed to them
/// (`og_web_set_transcoder`, a pool of Workers) and `Pending` until they
/// answer. Whichever bundle the runtime read it from, the key is its own.
pub(crate) fn texture(
    texture: &og_bundle::runtime::CookedTexture,
    formats: Option<&og_render::TextureFormats>,
) -> Fetch<og_render::TextureSource> {
    if texture.meta.codec == og_bundle::format::codec::RGBA8 || texture.ktx2.is_empty() {
        return match texture.to_texture_data() {
            Ok(t) => Fetch::Ready(og_render::TextureSource::Rgba(t)),
            Err(e) => Fetch::Failed(format!("{}: {e}", texture.id)),
        };
    }
    let key = (texture.id.clone(), texture.prepared_key(formats));
    if TRANSCODING.with(|t| t.borrow().contains(&key)) {
        return Fetch::Pending;
    }
    // A failure is kept, as natively: asking again does not transcode again.
    if let Some(e) = UNTRANSCODED.with(|u| u.borrow().get(&key).cloned()) {
        return Fetch::Failed(e);
    }
    if let Some(t) = TEXTURES.with(|t| t.borrow().get(&key).cloned()) {
        return Fetch::Ready(t);
    }
    TRANSCODER.with(|f| {
        let Some(transcode) = f.borrow().clone() else {
            return Fetch::Failed(format!(
                "{}: no Basis transcoder; the Worker installs one before the engine starts (src/web/boot.ts installTranscoder)",
                texture.id
            ));
        };
        NORMAL_XYRA.with(|set| {
            let mut set = set.borrow_mut();
            if texture.normal_xyra() {
                set.insert(key.clone());
            } else {
                set.remove(&key);
            }
        });
        let called = transcode.call3(
            &JsValue::NULL,
            &JsValue::from_str(&job_name(&key)),
            &js_sys::Uint8Array::from(texture.ktx2.as_slice()),
            &JsValue::from_str(texture.target_format(formats).name()),
        );
        match called {
            Ok(_) => {
                TRANSCODING.with(|t| t.borrow_mut().insert(key));
                Fetch::Pending
            }
            Err(e) => Fetch::Failed(format!("{}: the transcoder threw {e:?}", texture.id)),
        }
    })
}

/// The Worker's `transcode(key, ktx2, target)` (see [`TRANSCODER`]).
#[wasm_bindgen]
pub fn og_web_set_transcoder(transcode: js_sys::Function) {
    TRANSCODER.with(|f| *f.borrow_mut() = Some(transcode));
}

/// A texture the transcoder could not transcode, with the reason (`key` as
/// the transcoder was given it).
#[wasm_bindgen]
pub fn og_web_fail_texture(key: String, reason: String) {
    let Some(key) = job_key(&key) else { return };
    if TRANSCODING.with(|t| t.borrow_mut().remove(&key)) {
        UNTRANSCODED.with(|u| u.borrow_mut().insert(key, reason));
    }
}

/// Every prepared copy of the textures `ids`, and any still being prepared
/// (its answer is then dropped).
pub(crate) fn drop_textures(ids: &[String]) {
    NORMAL_XYRA.with(|t| t.borrow_mut().retain(|(id, _)| !ids.contains(id)));
    TEXTURES.with(|t| t.borrow_mut().retain(|(id, _), _| !ids.contains(id)));
    TRANSCODING.with(|t| t.borrow_mut().retain(|(id, _)| !ids.contains(id)));
    UNTRANSCODED.with(|t| t.borrow_mut().retain(|(id, _), _| !ids.contains(id)));
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

/// A texture the Worker transcoded to RGBA8 (level 0, rows top to bottom),
/// for `key` as the transcoder was given it. Answers `""`, or why it was
/// refused. One nobody waits for any more (its pack was let go of) is
/// dropped.
#[wasm_bindgen]
pub fn og_web_put_texture(
    key: String,
    width: u32,
    height: u32,
    mut rgba: Vec<u8>,
    srgb: bool,
) -> String {
    let Some(key) = job_key(&key) else {
        return format!("{key}: not a transcoder job's key");
    };
    if !TRANSCODING.with(|t| t.borrow_mut().remove(&key)) {
        return String::new();
    }
    if NORMAL_XYRA.with(|t| t.borrow().contains(&key)) {
        og_bundle::runtime::decode_normal_xyra(&mut rgba);
    }
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
            .insert(key, og_render::TextureSource::Rgba(data))
    });
    String::new()
}

/// A texture the Worker transcoded to the block format `format` (the
/// target the transcoder was given with `key`): every mip level, level 0
/// first, back to back in `levels`. Two-channel formats are linear
/// whatever `srgb` says. Answers `""`, or the reason the levels were
/// refused (the texture then fails). One nobody waits for any more is
/// dropped.
#[wasm_bindgen]
pub fn og_web_put_texture_encoded(
    key: String,
    format: String,
    width: u32,
    height: u32,
    srgb: bool,
    levels: Vec<u8>,
) -> String {
    let Some(key) = job_key(&key) else {
        return format!("{key}: not a transcoder job's key");
    };
    if !TRANSCODING.with(|t| t.borrow_mut().remove(&key)) {
        return String::new();
    }
    let id = key.0.clone();
    let Some(format) = og_render::TextureFormat::from_name(&format) else {
        let reason = format!("{id}: {format} is not a texture format");
        UNTRANSCODED.with(|u| u.borrow_mut().insert(key, reason.clone()));
        return reason;
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
    if format == og_render::TextureFormat::Rgba8 && NORMAL_XYRA.with(|t| t.borrow().contains(&key))
    {
        for mip in &mut split {
            og_bundle::runtime::decode_normal_xyra(mip);
        }
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
        let reason = format!("{id}: {e}");
        UNTRANSCODED.with(|u| u.borrow_mut().insert(key, reason.clone()));
        return reason;
    }
    TEXTURES.with(|t| {
        t.borrow_mut()
            .insert(key, og_render::TextureSource::Encoded(upload))
    });
    String::new()
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

/// The Worker's `prefetch(names)`, called with the engine wasm modules
/// content being prepared needs (the cooked manifest's `modules`): it
/// fetches and puts in place those not loaded yet (docs/web-export.md §Modules).
#[wasm_bindgen]
pub fn og_web_set_module_prefetcher(prefetch: js_sys::Function) {
    crate::MODULE_PREFETCHER.with(|f| *f.borrow_mut() = Some(prefetch));
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

/// The boot's cache fill (og-runtime's `prepare_textures`): the game's
/// bundle's Basis textures asked for through the texture hook, as the
/// engine asks for them later. JSON `{ total, pending, formats }`; the
/// Worker asks again until none is pending.
#[wasm_bindgen]
pub fn og_web_prepare_textures(engine: u32) -> String {
    og_runtime::prepare_textures(engine)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A job handed to the transcoder: waited for, a normal map or not.
    fn asked(key: &TextureKey, normal: bool) {
        TRANSCODING.with(|t| t.borrow_mut().insert(key.clone()));
        NORMAL_XYRA.with(|t| {
            let mut t = t.borrow_mut();
            if normal {
                t.insert(key.clone());
            } else {
                t.remove(key);
            }
        });
    }

    #[test]
    fn rgba_normal_fallback_restores_xy_and_z() {
        let key: TextureKey = ("normal-fallback-test".into(), 7);
        for encoded in [false, true] {
            asked(&key, true);
            let rgba = vec![179, 179, 179, 64];
            if encoded {
                assert_eq!(
                    og_web_put_texture_encoded(job_name(&key), "rgba8".into(), 1, 1, false, rgba),
                    ""
                );
            } else {
                assert_eq!(og_web_put_texture(job_name(&key), 1, 1, rgba, false), "");
            }
            TEXTURES.with(|t| {
                let t = t.borrow();
                let data = match &t[&key] {
                    og_render::TextureSource::Rgba(t) => &t.rgba,
                    og_render::TextureSource::Encoded(t) => &t.levels[0],
                    _ => panic!("fallback was not uploaded"),
                };
                assert_eq!(&data[..2], &[179, 64]);
                assert!((224..=228).contains(&data[2]));
            });
        }
        drop_textures(std::slice::from_ref(&key.0));
        asked(&key, false);
        og_web_put_texture(job_name(&key), 1, 1, vec![179, 179, 179, 64], false);
        TEXTURES.with(|t| {
            let t = t.borrow();
            let og_render::TextureSource::Rgba(data) = &t[&key] else {
                panic!("RGBA upload");
            };
            assert_eq!(
                data.rgba,
                [179, 179, 179, 64],
                "reused id is no longer a normal"
            );
        });
        drop_textures(std::slice::from_ref(&key.0));
    }

    /// An answer for a texture nobody waits for any more (its pack was let
    /// go of while it was transcoded) is dropped, not kept for good.
    #[test]
    fn an_answer_nobody_waits_for_is_dropped() {
        let key: TextureKey = ("dropped-test".into(), 9);
        asked(&key, false);
        drop_textures(std::slice::from_ref(&key.0));
        assert_eq!(
            og_web_put_texture(job_name(&key), 1, 1, vec![1, 2, 3, 4], false),
            ""
        );
        assert!(TEXTURES.with(|t| !t.borrow().contains_key(&key)));
    }
}
