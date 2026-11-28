//! The canvas renderer: an engine drawing to the page's OffscreenCanvas.

use og_render::{Backend, RenderDesc, RenderScene, RenderTarget, Renderer, RgbaImage};
use og_runtime::{RenderBackend, set_render_backend};
use wasm_bindgen::prelude::*;

struct CanvasBackend {
    engine: u32,
    renderer: Renderer,
    /// Fallback adapters render with WebGPU but cannot share swap images
    /// with Chrome's compositor. Only their presentation crosses the CPU.
    readback: Option<ReadbackCanvas>,
    tier: String,
    cpu_pass_timing_only: bool,
    /// Captures in flight: one per size, at most [`MAX_CAPTURES`],
    /// oldest first.
    pending: Vec<Pending>,
    /// Why the last capture failed, exposed through the probe instead of
    /// silently starting a different frame's readback.
    capture_error: Option<String>,
    /// What the last frame submitted.
    stats: og_render::FrameStats,
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
    /// While the device is lost: when the loss was found and when the
    /// next attempt at a new renderer may start (ms, `Date.now`).
    lost_at: Option<(f64, f64)>,
}

/// How long a lost device is asked again for, and how often: right after
/// a loss the browser can answer no adapter for a while (Chrome's
/// SwiftShader and a restarting GPU process answer null, then one 250 ms
/// later), so one refusal is not the end.
const RECOVERY_MS: f64 = 10_000.0;
const RETRY_MS: f64 = 250.0;

thread_local! {
    /// Renderers made on a new device after a loss, by engine, taken by
    /// the next frame.
    static RECOVERED: std::cell::RefCell<og_core::IdMap<u32, Result<Renderer, String>>> =
        const { std::cell::RefCell::new(og_core::IdMap::new()) };
    // Like og-render's frame_skipped, describes the last display attempt.
    static PRESENT_PENDING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub(super) fn present_pending() -> bool {
    PRESENT_PENDING.with(|p| p.get())
}

struct ReadbackCanvas {
    context: web_sys::OffscreenCanvasRenderingContext2d,
    pending: Option<(og_render::CaptureRequest, bool)>,
    failed: Option<String>,
    shown: bool,
}

impl CanvasBackend {
    /// Recovery from a lost device (iOS Safari after the page was in
    /// the background or under memory pressure, a GPU reset, the test
    /// hook): true while there is no device to draw on. The first frame
    /// that finds the device lost asks for a new one on the same canvas
    /// (asynchronously: the browser's adapter and device are promises);
    /// the frames until it lands draw nothing while the simulation goes
    /// on, and the first frame after it draws the world as it is then,
    /// every mesh and texture uploaded again from the scene. An attempt
    /// that fails is made again every [`RETRY_MS`] for [`RECOVERY_MS`]
    /// from the loss; after that every frame is refused.
    fn recover_device(&mut self) -> Result<bool, String> {
        let now = js_sys::Date::now();
        if let Some(made) = RECOVERED.with(|m| m.borrow_mut().remove(&self.engine)) {
            self.recovering = false;
            let renderer = match made {
                Ok(r) => r,
                Err(e) => {
                    let (since, _) = self.lost_at.get_or_insert((now, now));
                    if now - *since > RECOVERY_MS {
                        return Err(format!(
                            "the GPU device was lost and a new one could not be made: {e}"
                        ));
                    }
                    self.lost_at = Some((*since, now + RETRY_MS));
                    return Ok(true);
                }
            };
            super::FORMATS.with(|f| {
                f.borrow_mut()
                    .insert(self.engine, renderer.info().texture_formats.clone())
            });
            self.renderer = renderer;
            // Creation is not recovery: a device can die at its first
            // presentation. Keep the episode's deadline until it survives.
            self.lost_at = self.lost_at.map(|(since, _)| (since, now + RETRY_MS));
            self.losses += 1;
            self.kept = None;
            if let Some(readback) = &mut self.readback {
                readback.pending = None;
                readback.failed = None;
            }
        }
        let Some(why) = self.renderer.lost() else {
            if self
                .lost_at
                .is_some_and(|(_, next)| now - next > RECOVERY_MS)
            {
                self.lost_at = None;
            }
            return Ok(false);
        };
        let (since, next) = *self.lost_at.get_or_insert((now, now));
        if now - since > RECOVERY_MS {
            return Err(format!(
                "the GPU device was {why} and no new one was made in {} s",
                RECOVERY_MS / 1000.0
            ));
        }
        if !self.recovering && now >= next {
            self.recovering = true;
            if since == next {
                super::console_warn(&format!(
                    "origin-engine: the GPU device was {why}; making the renderer again"
                ));
            }
            let successor = self.renderer.successor();
            let canvas = self.canvas.clone();
            let engine = self.engine;
            let readback = self.readback.is_some();
            wasm_bindgen_futures::spawn_local(async move {
                let instance = Backend::WebGpu.instance();
                let made = if readback {
                    successor.make(None).await.map_err(|e| e.to_string())
                } else {
                    match instance.create_surface(wgpu::SurfaceTarget::OffscreenCanvas(canvas)) {
                        Ok(surface) => successor
                            .make(Some((&instance, surface)))
                            .await
                            .map_err(|e| e.to_string()),
                        Err(e) => Err(format!("surface: {e}")),
                    }
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
    tick: u64,
    camera: og_render::Camera,
    reference: bool,
    losses: u32,
}

/// The most captures of different sizes in flight at once. Refuse a new
/// request at the limit; never cancel a different caller's readback.
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
    fn history(&self) -> Option<(u64, u64)> {
        Some(self.renderer.history())
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
        PRESENT_PENDING.with(|p| p.set(true));
        if self.recover_device()? {
            return Ok(());
        }
        if let Some((w, h)) = super::take_resize(self.engine) {
            self.renderer.resize(w, h);
            if let Some(readback) = &mut self.readback {
                readback.pending = None;
                self.canvas.set_width(w);
                self.canvas.set_height(h);
            }
        }
        self.renderer.poll();
        if let Some(readback) = &mut self.readback {
            if let Some(error) = readback.failed.take() {
                return Err(error);
            }
            if let Some((request, complete)) = &mut readback.pending {
                let image = match request.try_finish(&self.renderer) {
                    Ok(image) => image,
                    Err(error) => {
                        // Buffer invalidation can precede device.lost's
                        // callback. Give it one event-loop turn to arrive
                        // so recovery, not a presentation error, owns loss.
                        readback.failed = Some(error.to_string());
                        readback.pending = None;
                        return Ok(());
                    }
                };
                if let Some(image) = image {
                    let data = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
                        wasm_bindgen::Clamped(&image.pixels),
                        image.width,
                        image.height,
                    )
                    .map_err(|e| format!("canvas image: {e:?}"))?;
                    readback
                        .context
                        .put_image_data(&data, 0.0, 0.0)
                        .map_err(|e| format!("canvas presentation: {e:?}"))?;
                    readback.shown |= *complete;
                    readback.pending = None;
                }
            }
            PRESENT_PENDING.with(|p| p.set(!readback.shown));
            // One readback in flight. A slow software GPU skips display
            // work, never simulation ticks or an unbounded queue of copies.
            if readback.pending.is_some() {
                return Ok(());
            }
        }
        let target = if self.readback.is_some() {
            RenderTarget::Offscreen
        } else {
            RenderTarget::Surface
        };
        match self.renderer.render(scene, &target) {
            Ok(s) => {
                self.stats = s;
            }
            // Neither case submitted a frame: recover or wait for visibility.
            Err(og_render::RenderError::DeviceLost(_))
            | Err(og_render::RenderError::SurfaceOccluded) => return Ok(()),
            Err(e) => return Err(e.to_string()),
        }
        self.drawn += 1;
        if let Some(readback) = &mut self.readback {
            // Bound incomplete warmup frames too: otherwise the software
            // GPU's queue grows while its first pipelines compile. Only a
            // complete frame counts as the first presentation.
            readback.pending = Some((
                self.renderer.begin_capture().map_err(|e| e.to_string())?,
                og_render::frame_skipped() == 0,
            ));
        } else {
            PRESENT_PENDING.with(|p| p.set(false));
        }
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
        self.renderer
            .set_pass_clock(clock, !self.cpu_pass_timing_only);
    }
    fn pass_timings(&self) -> Vec<og_render::PassTiming> {
        self.renderer.pass_timings().to_vec()
    }
    fn each_stage(&self, f: &mut dyn FnMut(&str, f64)) {
        for s in self.renderer.stage_timings() {
            f(&s.name, s.ms);
        }
    }
    fn gpu_timings(&mut self, wait: bool) -> Vec<og_render::GpuFrame> {
        self.renderer.gpu_timings(wait)
    }
    fn gpu_timing_absent(&self) -> Option<&'static str> {
        self.renderer.gpu_timing_absent().or(self
            .cpu_pass_timing_only
            .then_some("WebKit's GPU pass timestamp readbacks block the engine Worker"))
    }
    fn visibility_buffer_refused(&self) -> Option<String> {
        self.renderer.visibility_buffer_refused().map(str::to_owned)
    }
    fn frame_stats(&self) -> og_render::FrameStats {
        self.stats
    }
    fn post_errors(&self, scene: &RenderScene) -> Vec<(u32, String)> {
        self.renderer
            .post_errors(scene)
            .into_iter()
            .map(|(h, e)| (h.0, e))
            .collect()
    }
    fn sprite_shader_errors(&self) -> Vec<(usize, String)> {
        self.renderer.sprite_shader_errors()
    }
    fn cull_record(&mut self) -> Option<og_render::CullRecord> {
        self.renderer.cull_record().cloned()
    }
    fn cull_asked(&self) -> Option<u64> {
        self.renderer.cull_asked()
    }
    fn read_counts(&mut self) {
        self.renderer.request_culling_counts();
    }
    fn scatter_counts(&self) -> Option<og_render::ScatterCounts> {
        self.renderer.scatter_counts().cloned()
    }
    /// Request on one call, answer on a later one: the first call draws
    /// offscreen and starts the readback; a later call at the same size
    /// answers the frame once the copy has landed (the scene as it was
    /// at the request). Captures of different sizes wait side by side.
    fn capture(&mut self, scene: &RenderScene, width: u32, height: u32) -> Option<RgbaImage> {
        self.capture_error = None;
        let target = self.renderer.size();
        let size = if width == 0 || height == 0 {
            target
        } else {
            (width, height)
        };
        if let Some(i) = self.pending.iter().position(|p| p.size == size) {
            let mut p = self.pending.remove(i);
            if p.tick != scene.frame.tick {
                self.capture_error = Some(format!(
                    "capture readback belongs to tick {}, not tick {}",
                    p.tick, scene.frame.tick
                ));
                return None;
            }
            if p.camera != scene.frame.camera || p.reference != scene.frame.reference {
                self.capture_error =
                    Some("capture view changed while its readback was pending".into());
                return None;
            }
            if p.losses != self.losses {
                self.capture_error = Some("capture invalidated by GPU device recovery".into());
                return None;
            }
            match p.req.try_finish(&self.renderer) {
                Ok(Some(image)) => return Some(image),
                Ok(None) => {
                    self.pending.push(p);
                    return None;
                }
                Err(error) => {
                    self.capture_error = Some(error.to_string());
                    return None;
                }
            }
        }
        if self.pending.len() >= MAX_CAPTURES {
            self.capture_error =
                Some("capture readback capacity exhausted (4 pending captures)".into());
            return None;
        }
        // og-render's `capture_frame`; the runtime made a lost device again
        // before asking.
        match self.renderer.capture_frame(scene, size.0, size.1) {
            Ok((_, req)) => {
                self.pending.push(Pending {
                    req,
                    size,
                    tick: scene.frame.tick,
                    camera: scene.frame.camera,
                    reference: scene.frame.reference,
                    losses: self.losses,
                });
            }
            Err(error) => self.capture_error = Some(error.to_string()),
        }
        None
    }
    fn capture_error(&self) -> Option<String> {
        self.capture_error.clone()
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
        self.capture_error = None;
        let lost = self.renderer.lost();
        let recovered = self.recover_device();
        if let Some(why) = lost
            && !self.pending.is_empty()
        {
            let why = format!("capture readback lost its GPU device: {why}");
            self.capture_error = Some(why.clone());
            return Err(why);
        }
        if let Err(why) = &recovered {
            self.capture_error = Some(why.clone());
        }
        recovered
    }
    fn device_losses(&self) -> u32 {
        self.losses
    }
    fn lose_device(&mut self) -> bool {
        self.renderer.lose_device();
        true
    }
    fn frames_in_flight(&mut self) -> u32 {
        self.renderer.frames_in_flight() as u32
    }
}

/// Put the renderer on an OffscreenCanvas transferred to this Worker,
/// on WebGPU (the engine has no other web backend), its adapter asked
/// for the low-power GPU with `low_power` (the player's power
/// preference, kept by the shell from the last load) and its display
/// at standard range where `high_dynamic_range` (the page's
/// `(dynamic-range: high)`) is false. `cpu_pass_timing_only` disables
/// GPU pass profiling without changing the device's features. `render_tier`
/// `"compat"` (a harness's) makes it for `Tier::Compat` whatever the adapter
/// is. Resolves to the
/// tier (`webgpu`), or rejects with the reason: no WebGPU adapter in
/// this browser, or the canvas or device refused.
#[wasm_bindgen]
pub async fn og_web_attach_canvas(
    engine: u32,
    canvas: web_sys::OffscreenCanvas,
    low_power: bool,
    high_dynamic_range: Option<bool>,
    cpu_pass_timing_only: Option<bool>,
    render_tier: Option<String>,
) -> Result<String, JsValue> {
    if !wgpu::util::is_browser_webgpu_supported().await {
        return Err(JsValue::from_str(
            "no WebGPU adapter: this browser offers no WebGPU (navigator.gpu), which the engine needs",
        ));
    }
    let backend = Backend::WebGpu;
    let mut desc = og_runtime::render_desc(
        engine,
        RenderDesc {
            width: canvas.width().max(1),
            height: canvas.height().max(1),
            backend,
            low_power,
            // The page's `(dynamic-range: high)`: the Worker's canvas cannot ask.
            display_high_dynamic_range: high_dynamic_range,
            ..RenderDesc::default()
        },
    );
    // A harness's ask (the shell's `renderTier`): the same renderer made for
    // the compatibility tier on any adapter, so its frames are judged where
    // the browser offers no compatibility device that presents.
    if render_tier.as_deref() == Some("compat") {
        desc.tier = og_render::Tier::Compat;
    }
    let instance = backend.instance();
    let canvas_kept = canvas.clone();
    // Select before binding the canvas: its context type cannot change.
    // Chrome 157's fallback adapter accepts configure(), but allocating
    // the first swap image fails (8-bit and float) and destroys the device.
    let adapter = og_render::request_adapter(&instance, &desc, None)
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let software = adapter.get_info().device_type == wgpu::DeviceType::Cpu;
    let (surface, readback) = if software {
        use wasm_bindgen::JsCast;
        let context = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("no 2D canvas context for fallback presentation"))?
            .dyn_into::<web_sys::OffscreenCanvasRenderingContext2d>()?;
        super::console_warn(
            "origin-engine: fallback WebGPU adapter; presenting through bounded CPU readback",
        );
        (
            None,
            Some(ReadbackCanvas {
                context,
                pending: None,
                failed: None,
                shown: false,
            }),
        )
    } else {
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::OffscreenCanvas(canvas))
            .map_err(|e| JsValue::from_str(&format!("surface: {e}")))?;
        (Some(surface), None)
    };
    let renderer = Renderer::on_adapter(&desc, adapter, surface)
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let tier = renderer.info().backend_tier();
    let formats = renderer.info().texture_formats.clone();
    super::FORMATS.with(|f| f.borrow_mut().insert(engine, formats));
    let device = renderer.device();
    let gpu = super::gpu_report(
        &device.limits(),
        renderer.info().adapter_features,
        renderer.info().device_type == og_render::wgpu::DeviceType::Cpu,
    );
    super::GPUS.with(|g| g.borrow_mut().insert(engine, gpu.to_string()));
    set_render_backend(
        engine,
        Some(Box::new(CanvasBackend {
            engine,
            renderer,
            readback,
            tier: tier.clone(),
            cpu_pass_timing_only: cpu_pass_timing_only.unwrap_or(false),
            pending: Vec::new(),
            capture_error: None,
            drawn: 0,
            kept: None,
            stats: Default::default(),
            canvas: canvas_kept,
            losses: 0,
            recovering: false,
            lost_at: None,
        })),
    )
    .map_err(|e| JsValue::from_str(&e))?;
    og_runtime::set_power_preference(engine, low_power);
    Ok(tier)
}
