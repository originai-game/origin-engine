//! The offscreen renderer `og run` and the executors draw with.

// napi registers the exports outside `cfg(test)` only.
#![cfg_attr(test, allow(dead_code))]

use crate::resident;
use crate::wire::EnvScope;
use napi::Env;
use napi_derive::napi;
use og_runtime::RenderBackend;
use std::cell::RefCell;

struct OffscreenBackend {
    engine: u32,
    renderer: resident::Lease,
    tier: String,
    /// What the last frame submitted.
    stats: og_render::FrameStats,
    /// Devices lost and the renderer made again on a new one.
    losses: u32,
    /// Why the last capture or frame read failed (`RenderBackend::capture_error`).
    capture_error: Option<String>,
    /// The window drawing this engine's frames, when the process was told
    /// one (`ORIGIN_FRAMES_SOCKET`, og-render's `shared_frames`).
    #[cfg(any(target_os = "macos", target_os = "linux", windows))]
    frames: Option<og_render::shared_frames::FrameLink>,
    #[cfg(any(target_os = "macos", target_os = "linux", windows))]
    accessibility: (Option<u64>, og_ui::native_a11y::Bridge),
    /// The cursor last handed to the window (its keyword and image id).
    #[cfg(any(target_os = "macos", target_os = "linux", windows))]
    cursor: Option<(&'static str, u64)>,
}

impl OffscreenBackend {
    /// The frame drawn offscreen, where the probe reads it. A lost device
    /// (a driver reset, out of memory, `lose_device`) is replaced first: a
    /// renderer on a new device from the same description, which uploads
    /// the scene again on this frame, so the game goes on.
    fn draw(&mut self, scene: &og_render::RenderScene) -> Result<(), String> {
        self.recreate()?;
        // GPU culling's counts come back a frame or two late, read back
        // only when asked: the probe's triangles are its.
        self.renderer.request_culling_counts();
        self.renderer
            .render(scene, &og_render::RenderTarget::Offscreen)
            .map(|s| {
                self.stats = s;
            })
            .map_err(|e| e.to_string())
    }

    /// A lost device replaced by a renderer on a new one from the same
    /// description (`RenderBackend::recover`, and `draw` for the frames a
    /// probe captures between two of the loop's).
    fn recreate(&mut self) -> Result<(), String> {
        let Some(why) = self.renderer.lost() else {
            return Ok(());
        };
        eprintln!("og-host-node: the GPU device was {why}; making the renderer again");
        let renderer = self
            .renderer
            .recreate_blocking()
            .map_err(|e| format!("the GPU device was {why}, and a new one failed: {e}"))?;
        let device = renderer.device().clone();
        self.renderer = renderer.into();
        self.losses += 1;
        DEVICES.with(|d| d.borrow_mut().insert(self.engine, device));
        Ok(())
    }
}

impl Drop for OffscreenBackend {
    fn drop(&mut self) {
        let _ = DEVICES.try_with(|d| d.borrow_mut().remove(&self.engine));
    }
}

impl RenderBackend for OffscreenBackend {
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
    /// The loop's frame: at the size the window asked for since the last
    /// one (as the web host takes its canvas's), drawn offscreen, then
    /// shown in the window when one draws this engine's frames.
    fn render(&mut self, scene: &og_render::RenderScene) -> Result<(), String> {
        #[cfg(any(target_os = "macos", target_os = "linux", windows))]
        if let Some((w, h)) = self.frames.as_mut().and_then(|f| f.take_resize()) {
            self.renderer.resize(w, h);
        }
        self.draw(scene)?;
        #[cfg(any(target_os = "macos", target_os = "linux", windows))]
        if let Some(frames) = self.frames.as_mut() {
            let presented = frames.present(&self.renderer);
            let actions = frames.take_accessibility_actions();
            let session = frames.accessibility_session();
            if self.accessibility.0 != session {
                self.accessibility = (session, Default::default());
            }
            if session.is_some()
                && let Some(shared) = og_runtime::shared_ui(self.engine)
            {
                let mut ui = shared.lock();
                for value in actions {
                    if let Ok(action) = serde_json::from_value(value)
                        && let Some(request) = self.accessibility.1.action(action)
                    {
                        ui.queue_accessibility(request);
                    }
                }
                if let og_render::shared_frames::Presented::Slot {
                    generation, seq, ..
                } = presented
                {
                    let frame = ui.frame();
                    let (w, h) = self.renderer.size();
                    let scale = [
                        f64::from(frame.unit * w as f32 / frame.viewport.width.max(1.0)),
                        f64::from(frame.unit * h as f32 / frame.viewport.height.max(1.0)),
                    ];
                    if let Some(update) = self.accessibility.1.update(&ui, scale) {
                        frames.accessibility_update(
                            generation,
                            seq,
                            serde_json::to_value(update).expect("AccessKit tree"),
                        );
                    }
                }
            }
        }
        Ok(())
    }
    fn set_clock(&mut self, clock: Option<fn() -> f64>) {
        self.renderer.set_clock(clock);
    }
    /// The pointer's shape over the frame (CSS `cursor`), handed to the
    /// window when it changes; the window sets the OS cursor.
    fn cursor(&mut self, shape: &og_runtime::ui_cursor::CursorShape) {
        #[cfg(any(target_os = "macos", target_os = "linux", windows))]
        if let Some(frames) = self.frames.as_mut()
            && self.cursor != Some(shape.id())
        {
            self.cursor = Some(shape.id());
            let image = shape
                .image
                .as_ref()
                .map(|i| og_render::shared_frames::CursorImage {
                    width: i.width,
                    height: i.height,
                    rgba: og_core::base64::encode(&i.rgba),
                    hotspot: i.hotspot,
                });
            frames.set_cursor(shape.keyword, image);
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
        let _ = shape;
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
    fn read_reflection_probe(&self, id: u64) -> Option<Result<og_render::ProbeCapture, String>> {
        self.renderer.read_reflection_probe(id)
    }
    fn post_errors(&self, scene: &og_render::RenderScene) -> Vec<(u32, String)> {
        self.renderer
            .post_errors(scene)
            .into_iter()
            .map(|(h, e)| (h.0, e))
            .collect()
    }
    fn sprite_shader_errors(&self) -> Vec<(usize, String)> {
        self.renderer.sprite_shader_errors()
    }
    fn capture(
        &mut self,
        scene: &og_render::RenderScene,
        width: u32,
        height: u32,
    ) -> Option<og_render::RgbaImage> {
        let target = self.renderer.size();
        let size = if width == 0 || height == 0 {
            target
        } else {
            (width, height)
        };
        if size != target {
            self.renderer.resize(size.0, size.1);
        }
        // The capture is the scene's alone, not the frames drawn before it
        // (og-render's `render_capture`).
        let image = self
            .recreate()
            .and_then(|()| {
                self.renderer.request_culling_counts();
                self.renderer
                    .render_capture(scene, &og_render::RenderTarget::Offscreen)
                    .map(|s| self.stats = s)
                    .map_err(|e| e.to_string())
            })
            .and_then(|_| self.renderer.read_offscreen().map_err(|e| e.to_string()));
        self.capture_error = image.as_ref().err().cloned();
        if size != target {
            self.renderer.resize(target.0, target.1);
        }
        image.ok()
    }
    fn capture_error(&self) -> Option<String> {
        self.capture_error.clone()
    }
    fn read_frame(&mut self) -> Option<og_render::RgbaImage> {
        let image = self.renderer.read_offscreen().map_err(|e| e.to_string());
        self.capture_error = image.as_ref().err().cloned();
        image.ok()
    }
    fn gpu_memory(&self) -> Option<og_render::GpuMemory> {
        Some(self.renderer.gpu_memory())
    }
    fn recover(&mut self) -> Result<bool, String> {
        self.recreate().map(|()| false)
    }
    fn device_losses(&self) -> u32 {
        self.losses
    }
    fn lose_device(&mut self) -> bool {
        self.renderer.lose_device();
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

/// Give an engine a headless renderer of `width`×`height` (Metal or Vulkan),
/// created with the game's RenderOptions. A Result of the tier.
///
/// Without a renderer (no usable adapter, or `OG_RENDERER=none`) the answer
/// is a refusal naming why, and the play surface is still `width`×`height`
/// (the viewport a renderer of that size would give, so the UI lays out and
/// the simulation runs exactly as with one). The engine draws on a GPU only
/// (docs/design.md §2.6): a run that asked for a renderer fails on a
/// refused adapter (`attachRenderer` in packages/engine's node/index.ts);
/// only `OG_RENDERER=none` runs on without one, where a `screenshot` step
/// notes its frame unchecked and `capture` answers null.
#[napi(js_name = "og_node_attach_offscreen")]
pub fn og_node_attach_offscreen(env: Env, engine: u32, width: u32, height: u32) -> String {
    let _env = EnvScope::enter(&env);
    let desc = og_runtime::render_desc(
        engine,
        og_render::RenderDesc {
            width: width.max(1),
            height: height.max(1),
            // OG_GFX_CULLING=cpu culls on the CPU, as og-gfx does: lanes.test.ts
            // needs the CPU cull, which GPU culling skips.
            culling: if std::env::var("OG_GFX_CULLING").is_ok_and(|v| v == "cpu") {
                og_render::Culling::Cpu
            } else {
                og_render::Culling::Auto
            },
            ..og_render::RenderDesc::default()
        },
    );
    // Compiled pipelines kept across runs where the backend can (Vulkan):
    // OG_PIPELINE_CACHE names the directory, `off` keeps none; by default
    // ~/.cache/origin-engine/pipelines.
    og_render::set_pipeline_cache_dir(match std::env::var("OG_PIPELINE_CACHE") {
        Ok(v) if v == "off" => None,
        Ok(v) if !v.is_empty() => Some(v.into()),
        _ => std::env::var_os("HOME")
            .map(|h| std::path::PathBuf::from(h).join(".cache/origin-engine/pipelines")),
    });
    // A window that draws this process's frames (Origin Desktop's
    // Preview): the renderer outputs the format its slots copy from.
    #[cfg(any(target_os = "macos", target_os = "linux", windows))]
    let frames_socket =
        std::env::var_os(og_render::shared_frames::SOCKET_ENV).filter(|p| !p.is_empty());
    #[cfg(any(target_os = "macos", target_os = "linux", windows))]
    let desc = og_render::RenderDesc {
        output_format: match frames_socket {
            Some(_) => Some(og_render::shared_frames::OUTPUT_FORMAT),
            None => desc.output_format,
        },
        ..desc
    };
    let answer = resident::Lease::new(&desc)
        .map_err(|e| e.to_string())
        .and_then(|renderer| {
            let tier = renderer.info().backend_tier();
            let device = renderer.device().clone();
            #[cfg(any(target_os = "macos", target_os = "linux", windows))]
            let frames = frames_socket
                .map(|path| og_render::shared_frames::FrameLink::connect(path, &renderer));
            og_runtime::set_render_backend(
                engine,
                Some(Box::new(OffscreenBackend {
                    engine,
                    renderer,
                    tier: tier.clone(),
                    stats: Default::default(),
                    losses: 0,
                    capture_error: None,
                    #[cfg(any(target_os = "macos", target_os = "linux", windows))]
                    frames,
                    #[cfg(any(target_os = "macos", target_os = "linux", windows))]
                    accessibility: Default::default(),
                    #[cfg(any(target_os = "macos", target_os = "linux", windows))]
                    cursor: None,
                })),
            )?;
            DEVICES.with(|d| d.borrow_mut().insert(engine, device));
            Ok(tier)
        });
    match answer {
        Ok(tier) => format!(r#"{{"ok":true,"value":"{tier}"}}"#),
        Err(e) => {
            // The surface a renderer would have sized: rendering never
            // feeds back into simulation, so neither may its absence.
            let surface = serde_json::json!({ "width": width.max(1), "height": height.max(1) });
            let e = match og_runtime::set_viewport(engine, surface) {
                Ok(()) => e,
                Err(v) => format!("{e}; and the play surface: {v}"),
            };
            og_runtime::host_impl::refuse(&e)
        }
    }
}

thread_local! {
    /// Each offscreen renderer's device, for og_node_wait_gpu.
    pub(crate) static DEVICES: RefCell<std::collections::BTreeMap<u32, og_render::wgpu::Device>> =
        const { RefCell::new(std::collections::BTreeMap::new()) };
}

/// Block until the GPU has finished everything an engine's offscreen
/// renderer submitted, so a frame's time includes its GPU work (benches).
/// A Result of null; refused for an engine without og_node_attach_offscreen.
#[napi(js_name = "og_node_wait_gpu")]
pub fn og_node_wait_gpu(engine: u32) -> String {
    let device = DEVICES.with(|d| d.borrow().get(&engine).cloned());
    let answer = match device {
        None => Err(format!(
            "engine {engine} has no offscreen renderer; call og_node_attach_offscreen first"
        )),
        Some(device) => device
            .poll(og_render::wgpu::PollType::wait_indefinitely())
            .map(|_| ())
            .map_err(|e| e.to_string()),
    };
    match answer {
        Ok(()) => r#"{"ok":true,"value":null}"#.to_string(),
        Err(e) => og_runtime::host_impl::refuse(&e),
    }
}

/// `og run` without `--realtime` steps every frame for its script: those
/// frames draw at the configured quality and are no sample of automatic
/// quality (og-runtime's `set_scripted_frames`).
#[napi(js_name = "og_node_scripted_frames")]
pub fn og_node_scripted_frames(engine: u32, on: bool) {
    og_runtime::set_scripted_frames(engine, on);
}

/// `og run --capture-only`: frames step the ticks and draw nothing; the
/// script's screenshots and frame checks are probe captures, the PNGs a
/// run drawing every frame takes (og-runtime's `set_capture_only`).
#[napi(js_name = "og_node_capture_only")]
pub fn og_node_capture_only(engine: u32, on: bool) {
    og_runtime::set_capture_only(engine, on);
}
