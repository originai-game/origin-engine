//! What the frame is presented at, and what survives the device: the
//! render scale (the 3D scene drawn at a fraction of the output and scaled
//! up to it, the UI at the output's own size), the device's loss and the
//! renderer made again on a new one, and the GPU memory the renderer holds.
//!
//! A lost device (`GPUDevice.lost` on the web: iOS Safari after the page
//! was in the background or under memory pressure; a driver reset or out
//! of memory natively) is noticed by the device-lost callback;
//! [`Renderer::render`] refuses every frame after it with
//! [`RenderError::DeviceLost`] instead of drawing on a dead device, and
//! [`Renderer::recreate`] makes a renderer on a new device from the same
//! [`RenderDesc`] at the current size and presentation. Nothing else is
//! needed: the [`RenderScene`] holds every mesh, texture, material and font
//! as the CPU data it was added from, and a new renderer uploads all of it
//! on its first frame, as the first renderer did. What lived only on the
//! GPU starts over: the adapted exposure (metered again), TAA's history,
//! GPU particles, and render targets until their views draw again.

use super::*;
use std::sync::{Arc, Mutex};

/// The render scale's bounds ([`Renderer::set_render_scale`]).
pub const MIN_RENDER_SCALE: f32 = 0.5;
pub const MAX_RENDER_SCALE: f32 = 2.0;

/// Why the device was lost, once it was: set by the device-lost callback
/// (any thread), read by the renderer.
#[derive(Clone, Default)]
pub(crate) struct LostFlag(Arc<Mutex<Option<String>>>);

impl LostFlag {
    fn set(&self, reason: String) {
        if let Ok(mut l) = self.0.lock() {
            l.get_or_insert(reason);
        }
    }

    fn get(&self) -> Option<String> {
        self.0.lock().ok().and_then(|l| l.clone())
    }

    /// Watches `device`: its loss sets the flag. Natively the device's
    /// errors still panic as wgpu's own handler does (init.rs: a broken
    /// frame fails loudly), except once the device is lost, when every
    /// call on it fails and the next frame is refused instead.
    pub(crate) fn watch(device: &wgpu::Device) -> LostFlag {
        let flag = LostFlag::default();
        let lost = flag.clone();
        device.set_device_lost_callback(move |reason, message| {
            let why = match reason {
                wgpu::DeviceLostReason::Destroyed => "destroyed",
                wgpu::DeviceLostReason::Unknown => "lost",
            };
            lost.set(if message.is_empty() {
                why.to_string()
            } else {
                format!("{why}: {message}")
            });
        });
        #[cfg(not(target_arch = "wasm32"))]
        {
            let lost = flag.clone();
            device.on_uncaptured_error(Arc::new(move |e| {
                if lost.get().is_some() {
                    log::warn!("og-render: on a lost device: {e}");
                } else {
                    panic!("wgpu error: {e}\n");
                }
            }));
        }
        flag
    }
}

/// The GPU memory a renderer holds, in bytes, counted from what it created
/// and still keeps (each entry's size as allocated, without the driver's
/// alignment and padding).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GpuMemory {
    /// The scene's textures (every level, as uploaded: RGBA8 with its
    /// generated chain, or the encoded levels), texture arrays,
    /// environment maps, palettes, colour LUTs and the image-based light.
    pub textures: u64,
    /// Vertex and index buffers of the scene's meshes (both encodings
    /// where a compact mesh keeps its f32 vertices too) and their skins.
    pub meshes: u64,
    /// Textures drawn into: the scene's render targets
    /// (`RenderScene::add_render_target`), the frame graph's targets at
    /// the scene's size (render scale included), the output, TAA's
    /// history, the shadow maps, the transmission copy, the UI's glass,
    /// the reflection probes' faces and the volumetric fog's froxels.
    pub render_targets: u64,
    /// The renderer's own buffers: instances, uniforms and light tables.
    pub buffers: u64,
}

impl GpuMemory {
    /// Every category together.
    pub fn total(&self) -> u64 {
        self.textures + self.meshes + self.render_targets + self.buffers
    }
}

/// What a renderer on a new device is made from ([`Renderer::successor`]).
#[derive(Clone, Copy, Debug)]
pub struct Successor {
    desc: RenderDesc,
    render_scale: f32,
    clock: Option<fn() -> f64>,
    keep_frame: bool,
}

impl Successor {
    /// The new device's texture encodings chosen with `compression`
    /// instead of the old description's (a test hook: a device that comes
    /// back taking other encodings than the lost one, as a browser may).
    pub fn with_texture_compression(mut self, compression: crate::TextureCompression) -> Successor {
        self.desc.texture_compression = compression;
        self
    }

    /// The new renderer: `surface`, with the instance it was created from,
    /// is the canvas's or window's again (a surface is configured for one
    /// device); `None` makes a headless renderer. Its first frame uploads
    /// the whole scene again. The device may come back with other texture
    /// encodings ([`RenderInfo::texture_formats`]): a host that transcoded
    /// textures for the old one checks.
    pub async fn make(
        self,
        surface: Option<(&wgpu::Instance, wgpu::Surface<'static>)>,
    ) -> Result<Renderer, RenderError> {
        let mut renderer = match surface {
            Some((instance, surface)) => {
                Renderer::with_surface(&self.desc, instance, surface).await?
            }
            None => Renderer::new(&self.desc).await?,
        };
        renderer.render_scale = self.render_scale;
        renderer.set_clock(self.clock);
        if self.keep_frame {
            let _ = renderer.set_keep_frame(true);
        }
        Ok(renderer)
    }
}

/// What `texture` allocates: every level of every layer, times its samples.
pub(crate) fn texture_bytes(texture: &wgpu::Texture) -> u64 {
    let size = texture.size();
    let format = texture.format();
    let layers = match texture.dimension() {
        wgpu::TextureDimension::D3 => 1,
        _ => size.depth_or_array_layers.max(1),
    };
    let mut bytes = 0;
    for level in 0..texture.mip_level_count() {
        let w = (size.width >> level).max(1);
        let h = (size.height >> level).max(1);
        let d = match texture.dimension() {
            wgpu::TextureDimension::D3 => (size.depth_or_array_layers >> level).max(1),
            _ => 1,
        };
        let (bw, bh) = format.block_dimensions();
        let block = format.block_copy_size(None).unwrap_or(match format {
            wgpu::TextureFormat::Depth16Unorm => 2,
            wgpu::TextureFormat::Stencil8 => 1,
            wgpu::TextureFormat::Depth32FloatStencil8 => 8,
            _ => 4,
        });
        let footprint =
            u64::from(w.div_ceil(bw)) * u64::from(h.div_ceil(bh)) * u64::from(d) * u64::from(block);
        bytes += footprint;
    }
    bytes * u64::from(layers) * u64::from(texture.sample_count().max(1))
}

/// What a view's texture allocates.
pub(crate) fn view_bytes(view: &wgpu::TextureView) -> u64 {
    texture_bytes(view.texture())
}

impl Renderer {
    /// Draws the 3D scene at `scale` times the output's size from the next
    /// frame on, scaled to the output by the last post pass (bilinear;
    /// below 1 it is softer and cheaper, above 1 it supersamples), or under
    /// TAA below 1 by TAA itself (temporal upscaling: the jitter walks the
    /// scene's samples over every output pixel, and TAA and every pass
    /// after it run at the output's size); the UI,
    /// the overlay and text over the frame stay at the output's size.
    /// Within [`MIN_RENDER_SCALE`] and [`MAX_RENDER_SCALE`] (not finite:
    /// 1), each side within 1 and the device's largest texture. A
    /// stylisation pixel grid ([`Stylise::pixel`](crate::Stylise::pixel))
    /// sets the scene's size itself and this is not applied under it.
    /// Changing it remakes the frame graph's targets and starts TAA's
    /// history over, as a resize does, except from one scale below 1 to
    /// another under TAA, where the output-sized history is kept (dynamic
    /// resolution without a reset).
    pub fn set_render_scale(&mut self, scale: f32) {
        self.render_scale = if scale.is_finite() {
            scale.clamp(MIN_RENDER_SCALE, MAX_RENDER_SCALE)
        } else {
            1.0
        };
    }

    /// The render scale ([`Renderer::set_render_scale`]).
    pub fn render_scale(&self) -> f32 {
        self.render_scale
    }

    /// The size the scene was drawn at in the last frame (the output's at
    /// render scale 1 without a pixel grid).
    pub fn scene_size_drawn(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Why the device was lost (`destroyed: …` or `lost: …`), once it was;
    /// [`Renderer::render`] refuses every frame from then on
    /// ([`RenderError::DeviceLost`]) and the host makes a new renderer
    /// ([`Renderer::recreate`]).
    pub fn lost(&self) -> Option<String> {
        self.lost.get()
    }

    /// A test hook for device loss: destroys the device, as a GPU reset or
    /// the browser losing it would, and (natively) polls it so the
    /// device-lost callback has run when this returns. On the web the
    /// browser reports the loss a moment later.
    pub fn lose_device(&self) {
        self.device.destroy();
        #[cfg(not(target_arch = "wasm32"))]
        let _ = self.device.poll(wgpu::PollType::Poll);
    }

    /// What a renderer like this one on a new device is made from
    /// ([`Successor::make`]): the same [`RenderDesc`] at the output's
    /// current size, with this one's render scale, clock and frame
    /// keeping. Plain data, so a host can make the new renderer in a task
    /// of its own (the web's device is asked for asynchronously).
    pub fn successor(&self) -> Successor {
        Successor {
            desc: RenderDesc {
                width: self.out_width,
                height: self.out_height,
                ..self.desc
            },
            render_scale: self.render_scale,
            clock: self.clock,
            keep_frame: self.keep_frame,
        }
    }

    /// [`Renderer::successor`] made at once: `surface` (with the instance
    /// it came from) for a canvas or window, `None` for a headless one.
    pub async fn recreate(
        &self,
        surface: Option<(&wgpu::Instance, wgpu::Surface<'static>)>,
    ) -> Result<Renderer, RenderError> {
        self.successor().make(surface).await
    }

    /// The GPU memory this renderer holds now ([`GpuMemory`]).
    pub fn gpu_memory(&self) -> GpuMemory {
        let (targets, textures): (Vec<&GpuTexture>, Vec<&GpuTexture>) =
            self.cache.textures.iter().partition(|t| t.target);
        let mesh_bytes = |m: &GpuMesh| {
            m.vertices.size()
                + m.indices.size()
                + m.compact.as_ref().map_or(0, |(b, _)| b.size())
                + m.surface.as_ref().map_or(0, surface::SurfaceMesh::bytes)
        };
        let meshes = self.cache.meshes.iter().map(mesh_bytes).sum::<u64>()
            + self
                .cache
                .garments
                .values()
                .map(crate::garment_gpu::Mesh::bytes)
                .sum::<u64>()
            + self
                .cache
                .skins
                .iter()
                .flatten()
                .map(wgpu::Buffer::size)
                .sum::<u64>();
        let persistent = texture_bytes(&self.persistent.offscreen)
            + self
                .persistent
                .history
                .iter()
                .chain(self.persistent.smaa_history.iter())
                .flatten()
                .map(view_bytes)
                .sum::<u64>()
            + self.post.fsr.bytes();
        let shadows = texture_bytes(&self.cascade_map.texture)
            + texture_bytes(&self.local_map.texture)
            + view_bytes(&self.cascade_map.depth)
            + view_bytes(&self.local_map.depth)
            + self.local_mid.bytes();
        let render_targets = targets.iter().map(|t| t.bytes).sum::<u64>()
            + persistent
            + self.graph.target_bytes()
            + shadows
            + self
                .transmission
                .as_ref()
                .map_or(0, |(t, ..)| texture_bytes(t))
            + self.glass_targets.as_ref().map_or(0, |g| g.bytes())
            + self.probes.bytes()
            + self.sky_capture.as_ref().map_or(0, |c| c.bytes())
            + self.gi_probes.as_ref().map_or(0, |g| g.bytes())
            + self.volumetric.bytes();
        let buffers = [
            &self.frame_buffer,
            &self.ltc,
            &self.shadow_buffer,
            &self.fx_buffer,
            &self.sprite_fx_buffer,
            &self.skinned_buffer,
            &self.prev_instance_buffer,
            &self.ribbon_buffer,
            &self.instance_buffer,
            &self.sprite_buffer,
        ]
        .into_iter()
        .chain(&self.light_buffers)
        .chain(&self.reflection_buffer)
        .chain(&self.reflection_fx_buffer)
        .chain(self.reflection_light_buffers.iter().flatten())
        .map(wgpu::Buffer::size)
        .sum::<u64>()
            + self.oit.bytes();
        GpuMemory {
            textures: textures.iter().map(|t| t.bytes).sum::<u64>()
                + self.asset_bytes()
                + self.ibl.bytes(),
            meshes,
            render_targets,
            buffers,
        }
    }
}
