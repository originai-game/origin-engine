//! The runtime's hooks on the web host: the JavaScript dispatcher, what
//! the Worker fetched or decoded for the engine, and this thread's clocks.

use crate::assets::{drop_textures, is_model, is_sound_chunk, is_stream_pack, retranscode};
use crate::{
    DECODED_IMAGES, FETCHER, FILES, IMAGE_DECODER, INFLATED, INFLATER, ORPHANED, PERFORMANCE,
    REQUESTED, TEXTURES, TRANSCODING, UNTRANSCODED, VIDEO,
};
use og_runtime::{Dispatch, Fetch, Hooks};
use wasm_bindgen::prelude::*;

/// Calls the installed JavaScript dispatcher.
pub(crate) struct WebHooks {
    pub dispatcher: js_sys::Function,
}

impl Hooks for WebHooks {
    fn pump_platform(&self, engine: u32) -> Result<(), String> {
        self.dispatch(&Dispatch::platform_pump(engine)).map(|_| ())
    }

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
        Dispatch::answer(answer.as_string().unwrap_or_default())
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
