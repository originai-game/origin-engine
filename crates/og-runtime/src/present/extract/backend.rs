//! The renderer as the runtime sees it.

use super::*;

/// A renderer the host attached to an engine (an OffscreenCanvas surface on
/// the web, an offscreen target on native).
pub trait RenderBackend {
    /// The IDL `RenderBackend` (`webgpu`, `metal`, `vulkan`, `vulkan-cpu`,
    /// `dx12`), og-render's `tier_name`, which also names the
    /// reference frames.
    fn tier(&self) -> String;
    /// The render target in device pixels.
    fn size(&self) -> (u32, u32);
    /// What `RenderOptions.antialias` resolved to: `off`, `msaa4`, `taa`
    /// or `fxaa` ([`antialias_name`]).
    fn antialias(&self) -> String {
        "off".into()
    }
    /// The pointer's shape the UI resolved this frame ([`crate::host::ui_cursor`]),
    /// given every frame: a backend that shows the frames in a window (the
    /// native host's shared frame) hands it on when it changes.
    fn cursor(&mut self, shape: &crate::host::ui_cursor::CursorShape) {
        let _ = shape;
    }
    /// The frames drawn so far and the frames TAA's history holds
    /// (og-render's `Renderer::history`); `None` for a backend without one.
    fn history(&self) -> Option<(u64, u64)> {
        None
    }
    /// What the renderer runs on and resolved its options to (the
    /// capabilities' shadow limits); `None` for a backend without one.
    fn info(&self) -> Option<og_render::RenderInfo> {
        None
    }
    /// The texture encodings the device takes (og-render's
    /// `RenderInfo::texture_formats`): loads transcode to them. `None`: RGBA8.
    fn texture_formats(&self) -> Option<og_render::TextureFormats> {
        None
    }
    /// The renderer's device and queue, for compute that runs beside the
    /// frames (og.motion's model); `None` for a backend without a GPU.
    /// After a lost device is replaced ([`RenderBackend::recover`]) this
    /// answers the new one.
    fn compute(&self) -> Option<(og_render::wgpu::Device, og_render::wgpu::Queue)> {
        None
    }
    /// Draw one frame of the scene to the display target.
    fn render(&mut self, scene: &RenderScene) -> Result<(), String>;
    /// Times each render-graph pass's encoding with `clock` (og-render's
    /// `Renderer::set_clock`); a backend without passes ignores it.
    fn set_clock(&mut self, _clock: Option<fn() -> f64>) {}
    /// The last frame's passes in execution order, each with the start
    /// and length of its CPU encoding (og-render's
    /// `Renderer::pass_timings`); empty without a clock.
    fn pass_timings(&self) -> Vec<og_render::PassTiming> {
        Vec::new()
    }
    /// The frames timed on the GPU read back since the last call, oldest
    /// first (og-render's `Renderer::gpu_timings`; `wait` waits for those
    /// in flight, natively); empty without a clock or timestamp queries.
    fn gpu_timings(&mut self, _wait: bool) -> Vec<og_render::GpuFrame> {
        Vec::new()
    }
    /// Why passes are not timed on the GPU here (og-render's
    /// `Renderer::gpu_timing_absent`), or `None` when they are.
    fn gpu_timing_absent(&self) -> Option<&'static str> {
        Some("this backend has no GPU passes")
    }
    /// What the device said when it refused the visibility buffer's
    /// resolve (og-render's `Renderer::visibility_buffer_refused`), or
    /// `None`: `RenderCapabilities.visibilityBuffer`.
    fn visibility_buffer_refused(&self) -> Option<String> {
        None
    }
    /// What the last frame submitted (og-render's `FrameStats`): its
    /// triangles are `ProbeTimings.triangles`, the rest the render dump's.
    fn frame_stats(&self) -> og_render::FrameStats {
        og_render::FrameStats::default()
    }
    /// Reflection probe `id`'s capture read back (og-render's
    /// `Renderer::read_reflection_probe`): native hosts, for `og bake probes`.
    fn read_reflection_probe(&self, _id: u64) -> Option<Result<og_render::ProbeCapture, String>> {
        None
    }
    /// The custom post passes of `scene` the device failed to compile, by
    /// post index, with its message (og-render's `Renderer::post_errors`).
    fn post_errors(&self, _scene: &RenderScene) -> Vec<(u32, String)> {
        Vec::new()
    }
    /// The material shaders whose sprite pipelines failed to compile, by
    /// shader index, with the message (og-render's
    /// `Renderer::sprite_shader_errors`).
    fn sprite_shader_errors(&self) -> Vec<(usize, String)> {
        Vec::new()
    }
    /// Draw the scene offscreen at `width`×`height` (0×0: the target's size)
    /// and read it back. Native hosts block on the readback; the web host
    /// answers a frame it requested earlier (request on tick N, answer on
    /// N + 1) and `None` while the copy is in flight.
    fn capture(
        &mut self,
        scene: &RenderScene,
        width: u32,
        height: u32,
    ) -> Option<og_render::RgbaImage>;
    /// Whether `capture` answers on a later call (the web host): a probe
    /// waits for the frame instead of failing on the first `None`.
    fn deferred(&self) -> bool {
        false
    }
    /// Why the last `capture` answered `None` (a readback that timed out,
    /// a lost device), for the probe's message; `None` when it gave a frame
    /// or does not say.
    fn capture_error(&self) -> Option<String> {
        None
    }
    /// og-render's `Renderer::save_state` (the adapted exposure) as its
    /// number, for a probe snapshot; 0 keeps nothing.
    fn save_state(&mut self) -> u64 {
        0
    }
    /// og-render's `Renderer::restore_state` of what `save_state` answered.
    fn restore_state(&mut self, state: u64) {
        let _ = state;
    }
    /// og-render's `Renderer::warm`: make one pipeline no frame has used
    /// yet, in the host's idle time; false when there is none left.
    fn warm(&mut self) -> bool {
        false
    }
    /// The last frame `render` drew, read back without drawing it again
    /// (`og render`'s every-frame capture); `None` when nothing was drawn,
    /// or on the web until that frame's readback has landed (the canvas's
    /// frames are kept offscreen once a reader asked; ask again before the
    /// next frame).
    fn read_frame(&mut self) -> Option<og_render::RgbaImage> {
        None
    }
    /// Whether it presents to a display (the web canvas): only then are
    /// the render scale and automatic quality applied (`crate::present::display`);
    /// a native run draws at its fixed size and scale 1.
    fn presents(&self) -> bool {
        false
    }
    /// og-render's `Renderer::set_render_scale`, from the next frame.
    fn set_render_scale(&mut self, _scale: f32) {}
    /// The size the 3D scene was drawn at last (og-render's
    /// `Renderer::scene_size_drawn`).
    fn scene_size(&self) -> (u32, u32) {
        self.size()
    }
    /// The GPU memory the renderer holds (og-render's `Renderer::gpu_memory`).
    fn gpu_memory(&self) -> Option<og_render::GpuMemory> {
        None
    }
    /// Before a frame: when the device was lost, make the renderer again
    /// on a new one (natively at once; the web host asks for one and
    /// draws nothing until it lands). `Ok(true)` while there is no device
    /// to draw on. The runtime calls it before it extracts the frame, so
    /// textures the new device takes in other encodings are decoded again
    /// first (`crate::world::assets::retranscode`); `render` recovers too.
    fn recover(&mut self) -> Result<bool, String> {
        Ok(false)
    }
    /// How many times the device was lost and the renderer made again on
    /// a new one (each backend recovers in its own `render`).
    fn device_losses(&self) -> u32 {
        0
    }
    /// A test hook: destroy the device (og-render's `Renderer::lose_device`);
    /// false where there is none.
    fn lose_device(&mut self) -> bool {
        false
    }
}
