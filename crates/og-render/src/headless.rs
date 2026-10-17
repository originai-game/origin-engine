//! Native headless rendering: the executor path (lavapipe or a GPU). The
//! readback itself is [`Renderer::capture`], shared with the web.

use crate::{RenderDesc, RenderError, RenderScene, RenderTarget, Renderer, RgbaImage};

/// Renders one frame of `scene` offscreen and returns its RGBA8 pixels
/// (blocking; native only).
pub fn render_to_rgba(desc: &RenderDesc, scene: &RenderScene) -> Result<RgbaImage, RenderError> {
    let mut renderer = pollster::block_on(Renderer::new(desc))?;
    renderer.render(scene, &RenderTarget::Offscreen)?;
    renderer.read_offscreen()
}

impl Renderer {
    /// Creates a renderer, blocking on the adapter and device requests (native only).
    pub fn new_blocking(desc: &RenderDesc) -> Result<Renderer, RenderError> {
        pollster::block_on(Renderer::new(desc))
    }

    /// Blocks until the GPU has finished every submitted frame.
    pub fn wait_idle(&self) -> Result<(), RenderError> {
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map(|_| ())
            .map_err(|e| RenderError::Readback(e.to_string()))
    }

    /// [`Renderer::capture`], blocking (native only).
    pub fn read_offscreen(&self) -> Result<RgbaImage, RenderError> {
        pollster::block_on(self.capture())
    }
}
