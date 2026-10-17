//! Native headless rendering: the executor path (lavapipe or a GPU) and the
//! readback that frame capture and the gfx tests use.

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

    /// Copies the last frame rendered to [`RenderTarget::Offscreen`] into RGBA8
    /// bytes (blocking). BGRA outputs are swizzled to RGBA.
    pub fn read_offscreen(&self) -> Result<RgbaImage, RenderError> {
        let (width, height) = (self.width, self.height);
        let format = self.output_format;
        let swizzle = match format {
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => false,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => true,
            other => return Err(RenderError::Unsupported(format!("readback of {other:?}"))),
        };
        let bytes_per_row = (width * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: (bytes_per_row * height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("readback"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.targets.offscreen,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));
        let slice = buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.wait_idle()?;
        rx.recv()
            .map_err(|e| RenderError::Readback(e.to_string()))?
            .map_err(|e| RenderError::Readback(e.to_string()))?;
        let data = slice
            .get_mapped_range()
            .map_err(|e| RenderError::Readback(format!("{e:?}")))?;
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);
        for row in 0..height {
            let start = (row * bytes_per_row) as usize;
            pixels.extend_from_slice(&data[start..start + (width * 4) as usize]);
        }
        drop(data);
        buffer.unmap();
        if swizzle {
            for p in pixels.chunks_exact_mut(4) {
                p.swap(0, 2);
            }
        }
        Ok(RgbaImage {
            width,
            height,
            pixels,
        })
    }
}
