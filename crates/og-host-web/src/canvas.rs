//! The canvas renderer: an engine drawing to the page's OffscreenCanvas.

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
                let made =
                    match instance.create_surface(wgpu::SurfaceTarget::OffscreenCanvas(canvas)) {
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
        // GPU culling's counts come back a frame or two late, read
        // back only when asked: the probe's triangles are its.
        self.renderer.request_culling_counts();
        match self.renderer.render(scene, &RenderTarget::Surface) {
            Ok(s) => {
                self.stats = s;
            }
            // Neither case submitted a frame: recover or wait for visibility.
            Err(og_render::RenderError::DeviceLost(_))
            | Err(og_render::RenderError::SurfaceOccluded) => return Ok(()),
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
        // The scene's alone, not the frames drawn before it
        // (og-render's `render_capture`).
        let started = self
            .renderer
            .render_capture(scene, &RenderTarget::Offscreen)
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
            stats: Default::default(),
            canvas: canvas_kept,
            losses: 0,
            recovering: false,
        })),
    )
    .map_err(|e| JsValue::from_str(&e))?;
    og_runtime::set_power_preference(engine, low_power);
    Ok(tier)
}
