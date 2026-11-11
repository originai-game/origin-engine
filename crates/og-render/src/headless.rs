//! Native headless rendering: the executor path (lavapipe or a GPU). The
//! readback itself is [`Renderer::capture`], shared with the web.

use crate::{RenderDesc, RenderError, RenderScene, RenderTarget, Renderer, RgbaImage, Successor};

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

    /// [`Renderer::restart`], blocking (native only).
    pub fn restart_blocking(self, desc: &RenderDesc) -> Result<Renderer, RenderError> {
        pollster::block_on(self.restart(desc))
    }

    /// [`Renderer::recreate`] headless, blocking: a renderer like this one
    /// on a new device (after [`Renderer::lost`]).
    pub fn recreate_blocking(&self) -> Result<Renderer, RenderError> {
        pollster::block_on(self.recreate(None))
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

    /// The height of the sea the last frame drew (its first og.Water with a
    /// wind) at each world point's xz, read back from the GPU (blocking;
    /// native only): per point the height the clipmap's vertices draw there
    /// (`probe_sea` in water.wgsl), the share of it the tick's waves draw
    /// (og_core::ocean, what og.physics.waterHeight floats bodies on) and
    /// the share the FFT cascades draw (render only). Empty without a sea.
    /// A diagnostic of the drawn waterline (the former test sample
    pub fn sea_heights(&self, points: &[glam::Vec2]) -> Result<Vec<[f32; 3]>, RenderError> {
        let origin = self.history.drawn_origin;
        let frame: Vec<glam::Vec2> = points
            .iter()
            .map(|p| *p - glam::Vec2::new(origin.x, origin.z))
            .collect();
        let Some(buffer) = self
            .world
            .water
            .probe_sea(&self.device, &self.queue, &frame)
        else {
            return Ok(Vec::new());
        };
        let (tx, rx) = std::sync::mpsc::channel();
        buffer.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.wait_idle()?;
        rx.recv()
            .map_err(|e| RenderError::Readback(e.to_string()))?
            .map_err(|e| RenderError::Readback(format!("{e:?}")))?;
        let out = {
            let data = buffer
                .slice(..)
                .get_mapped_range()
                .map_err(|e| RenderError::Readback(format!("{e:?}")))?;
            bytemuck::cast_slice::<u8, [f32; 4]>(&data)
                .iter()
                .take(points.len())
                .map(|v| [v[0] + origin.y, v[1], v[2]])
                .collect()
        };
        buffer.unmap();
        Ok(out)
    }

    /// What the renderer uploaded for a texture array (`None` until a
    /// frame has synced it, once a frame has freed it after
    /// `RenderScene::remove_texture_array`, or for a foreign handle).
    pub fn texture_array_info(
        &self,
        handle: crate::TextureArrayHandle,
    ) -> Option<crate::TextureArrayInfo> {
        self.cache_texture_array(handle).map(|a| a.info)
    }

    fn cache_texture_array(
        &self,
        handle: crate::TextureArrayHandle,
    ) -> Option<&crate::texture_array::GpuTextureArray> {
        self.cache
            .texture_arrays
            .get(handle.index())?
            .as_ref()
            .filter(|a| a.generation == handle.generation())
    }

    /// Level `level` of layer `layer` of an RGBA8 texture array, read back
    /// from the GPU (blocking; native only): what the one `D2Array`
    /// texture holds there.
    pub fn read_texture_array_layer(
        &self,
        handle: crate::TextureArrayHandle,
        layer: u32,
        level: u32,
    ) -> Result<RgbaImage, RenderError> {
        let gpu = self.cache_texture_array(handle).ok_or_else(|| {
            RenderError::InvalidHandle(format!("texture array {}", handle.index()))
        })?;
        let info = gpu.info;
        if info.format != crate::TextureFormat::Rgba8
            || layer >= info.layers
            || level >= info.levels
        {
            return Err(RenderError::Unsupported(format!(
                "reading layer {layer} level {level} of a {} array of {} layers and {} levels",
                info.format.name(),
                info.layers,
                info.levels
            )));
        }
        let (w, h) = ((info.width >> level).max(1), (info.height >> level).max(1));
        let row = w * 4;
        let padded =
            row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("texture array readback"),
            size: u64::from(padded * h),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("texture array readback"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &gpu.texture,
                mip_level: level,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: layer,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(h),
                },
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));
        let (tx, rx) = std::sync::mpsc::channel();
        buffer.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.wait_idle()?;
        rx.recv()
            .map_err(|e| RenderError::Readback(e.to_string()))?
            .map_err(|e| RenderError::Readback(format!("{e:?}")))?;
        let mut pixels = Vec::with_capacity((row * h) as usize);
        {
            let data = buffer
                .slice(..)
                .get_mapped_range()
                .map_err(|e| RenderError::Readback(format!("{e:?}")))?;
            for chunk in data.chunks_exact(padded as usize) {
                pixels.extend_from_slice(&chunk[..row as usize]);
            }
        }
        buffer.unmap();
        Ok(RgbaImage {
            width: w,
            height: h,
            pixels,
        })
    }
}

impl Successor {
    /// [`Successor::make`] headless, blocking (native only).
    pub fn make_blocking(self) -> Result<Renderer, RenderError> {
        pollster::block_on(self.make(None))
    }
}
