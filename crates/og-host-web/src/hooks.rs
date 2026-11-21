//! The runtime's hooks on the web host: the JavaScript dispatcher, what
//! the Worker fetched or decoded for the engine, and this thread's clocks.

use crate::assets::{drop_textures, is_model, is_sound_chunk, is_stream_pack};
use crate::{
    DECODED_IMAGES, FETCHER, FILES, IMAGE_DECODER, INFLATED, INFLATER, ORPHANED, PERFORMANCE,
    REQUESTED, VIDEO,
};
use og_runtime::{Dispatch, Fetch, Hooks};
use wasm_bindgen::prelude::*;

/// Calls the installed JavaScript dispatcher.
pub(crate) struct WebHooks {
    pub dispatcher: js_sys::Function,
}

impl Hooks for WebHooks {
    fn pump_platform(&self, pump: &Dispatch) -> Result<(), String> {
        self.dispatch(pump).map(|_| ())
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
            None => request(path, None),
        }
    }

    fn fetching(&self, path: &str) -> bool {
        // Landed (or failed) is for read_asset to take: never taken here.
        if FILES.with(|f| f.borrow().contains_key(path)) {
            return false;
        }
        matches!(request(path, None), Fetch::Pending)
    }

    fn read_content(&self, path: &str, integrity: &str) -> Fetch<Vec<u8>> {
        // Read once by the engine, which keeps what it needs: taken, never
        // kept in the wasm heap beside its copy, and fetched again (a
        // failure too) the next time it is asked for.
        match FILES.with(|f| f.borrow_mut().remove(path)) {
            Some(Ok(bytes)) => Fetch::Ready(bytes),
            Some(Err(e)) => Fetch::Failed(e),
            None => request(path, Some(integrity).filter(|i| !i.is_empty())),
        }
    }

    fn read_progress(&self, path: &str) -> Option<(f64, f64)> {
        crate::PROGRESS.with(|p| p.borrow().get(path).copied())
    }

    fn loading(&self, engine: u32, stage: &str, fraction: f64, label: &str) {
        crate::LOADING.with(|l| {
            if let Some((report, _)) = &*l.borrow() {
                let args = js_sys::Array::of4(
                    &JsValue::from(engine),
                    &JsValue::from_str(stage),
                    &JsValue::from_f64(fraction),
                    &JsValue::from_str(label),
                );
                let _ = report.apply(&JsValue::NULL, &args);
            }
        });
    }

    fn loading_idle(&self, engine: u32) -> bool {
        crate::LOADING.with(|l| match &*l.borrow() {
            Some((_, idle)) => idle
                .call1(&JsValue::NULL, &JsValue::from(engine))
                .map_or(true, |v| v.as_bool().unwrap_or(true)),
            None => true,
        })
    }

    fn texture(
        &self,
        texture: &og_bundle::runtime::CookedTexture,
        formats: Option<&og_render::TextureFormats>,
    ) -> Fetch<og_render::TextureSource> {
        crate::assets::texture(texture, formats)
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

    fn prefetch_modules(&self, modules: &[String]) {
        crate::MODULE_PREFETCHER.with(|f| {
            if let Some(prefetch) = &*f.borrow() {
                let names: js_sys::Array = modules.iter().map(|m| JsValue::from_str(m)).collect();
                // A failed prefetch is the first call's to load or report.
                let _ = prefetch.call1(&JsValue::NULL, &names);
            }
        });
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
            .with(|v| v.borrow().as_ref().map(|h| h.decoder.clone()))
            .unwrap_or_default()
    }

    #[cfg(target_arch = "wasm32")]
    fn video_stream(
        &self,
        stream: u64,
        size: (u32, u32),
        codec: &str,
        config: &[u8],
        table: &[u32],
        chunk_frames: u32,
    ) -> bool {
        VIDEO.with(|v| {
            let Some(h) = &*v.borrow() else {
                return false;
            };
            let args = js_sys::Array::new();
            args.push(&JsValue::from_f64(stream as f64));
            args.push(&JsValue::from(size.0));
            args.push(&JsValue::from(size.1));
            args.push(&JsValue::from_str(codec));
            args.push(&js_sys::Uint8Array::from(config));
            args.push(&js_sys::Uint32Array::from(table));
            args.push(&JsValue::from(chunk_frames));
            h.stream
                .apply(&JsValue::NULL, &args)
                .ok()
                .and_then(|r| r.as_bool())
                .unwrap_or(false)
        })
    }

    #[cfg(target_arch = "wasm32")]
    fn video_chunk(&self, stream: u64, n: u32, data: Option<&[u8]>) {
        VIDEO.with(|v| {
            let Some(h) = &*v.borrow() else { return };
            let data = match data {
                Some(d) => JsValue::from(js_sys::Uint8Array::from(d)),
                None => JsValue::NULL,
            };
            let _ = h.chunk.call3(
                &JsValue::NULL,
                &JsValue::from_f64(stream as f64),
                &JsValue::from(n),
                &data,
            );
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
            let h = v.borrow().as_ref().cloned()?;
            let args = js_sys::Array::of4(
                &JsValue::from_f64(stream as f64),
                &JsValue::from_f64(player as f64),
                &JsValue::from(frame),
                &JsValue::from_bool(looped),
            );
            let f = h.frame.apply(&JsValue::NULL, &args).ok()?;
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

/// Asks the Worker's fetcher for `path` once (against `integrity`, which
/// the fetcher checks: src/web/boot.ts), `Pending` until it lands.
fn request(path: &str, integrity: Option<&str>) -> Fetch<Vec<u8>> {
    FETCHER.with(|f| match &*f.borrow() {
        Some(fetch) => {
            // A pack let go of in flight is wanted again: kept when it lands.
            ORPHANED.with(|o| o.borrow_mut().remove(path));
            if REQUESTED.with(|r| r.borrow_mut().insert(path.to_string()))
                && let Err(e) = fetch.call2(
                    &JsValue::NULL,
                    &JsValue::from_str(path),
                    &integrity.map_or(JsValue::UNDEFINED, JsValue::from_str),
                )
            {
                REQUESTED.with(|r| r.borrow_mut().remove(path));
                return Fetch::Failed(format!("{path}: the fetcher threw {e:?}"));
            }
            Fetch::Pending
        }
        None => Fetch::Failed(format!(
            "{path} was not fetched: the Worker fetches the game's cooked bundle before the engine starts"
        )),
    })
}
