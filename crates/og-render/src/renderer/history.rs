//! The frame history the display carries from one frame to the next, kept
//! with a world snapshot (docs/design.md §2.4, presentation checkpoints):
//! TAA's, SMAA T2x's and FSR's histories, screen-space GI's copy of the lit
//! image and the reflections' accumulation, the previous camera, clock,
//! wind and sway (motion vectors), the temporal phase (the jitter, the
//! shadow filters' spiral, the screen noise), the adapted exposure, the
//! tick-stepped wind field and trample map, and each streamed terrain
//! tile's fade. [`Renderer::save_state`] copies them on the GPU (no
//! readback) and [`Renderer::restore_state`] copies them back, so the
//! display frames after a restore are the frames the uninterrupted run drew
//! from that tick, byte for byte on one host and backend.
//!
//! A part whose GPU state is not there to copy back into (made since at
//! another size, or not made yet) starts afresh instead, as it would after
//! a resize. The newest saves keep their copies within [`HISTORY_BYTES`];
//! an older one keeps only its CPU part, and its restore starts the GPU
//! histories afresh, said by [`Renderer::restore_state`]'s answer.

use super::*;
use std::collections::VecDeque;

/// Saves kept: as many as og-runtime's presentation checkpoints.
const SAVES: usize = 64;
/// The GPU bytes the saves' copies may hold together; the newest save
/// keeps its copy whatever its size.
const HISTORY_BYTES: u64 = 768 << 20;

/// A copy of GPU textures (every level and layer) and buffers.
pub(crate) struct GpuCopy {
    textures: Vec<wgpu::Texture>,
    buffers: Vec<wgpu::Buffer>,
}

fn same_shape(a: &wgpu::Texture, b: &wgpu::Texture) -> bool {
    a.size() == b.size()
        && a.mip_level_count() == b.mip_level_count()
        && a.format() == b.format()
        && a.dimension() == b.dimension()
        && a.sample_count() == b.sample_count()
}

fn copy_texture(encoder: &mut wgpu::CommandEncoder, from: &wgpu::Texture, to: &wgpu::Texture) {
    for mip in 0..from.mip_level_count() {
        let size = from.size().mip_level_size(mip, from.dimension());
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: from,
                mip_level: mip,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: to,
                mip_level: mip,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            size,
        );
    }
}

impl GpuCopy {
    /// Copies `textures` and `buffers` (each made with `COPY_SRC` and
    /// `COPY_DST`) into new ones, recorded into `encoder`.
    pub(crate) fn of(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        textures: &[&wgpu::Texture],
        buffers: &[&wgpu::Buffer],
    ) -> GpuCopy {
        let textures = textures
            .iter()
            .map(|t| {
                let copy = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("presentation checkpoint"),
                    size: t.size(),
                    mip_level_count: t.mip_level_count(),
                    sample_count: t.sample_count(),
                    dimension: t.dimension(),
                    format: t.format(),
                    usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                copy_texture(encoder, t, &copy);
                copy
            })
            .collect();
        let buffers = buffers
            .iter()
            .map(|b| {
                let copy = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("presentation checkpoint"),
                    size: b.size(),
                    usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                encoder.copy_buffer_to_buffer(b, 0, &copy, 0, b.size());
                copy
            })
            .collect();
        GpuCopy { textures, buffers }
    }

    /// Copies back into `textures` and `buffers` when they are the ones
    /// copied (as many, of the same shapes); false, copying nothing,
    /// otherwise.
    pub(crate) fn back(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        textures: &[&wgpu::Texture],
        buffers: &[&wgpu::Buffer],
    ) -> bool {
        let fits = textures.len() == self.textures.len()
            && buffers.len() == self.buffers.len()
            && self
                .textures
                .iter()
                .zip(textures)
                .all(|(a, b)| same_shape(a, b))
            && self
                .buffers
                .iter()
                .zip(buffers)
                .all(|(a, b)| a.size() == b.size());
        if !fits {
            return false;
        }
        for (from, to) in self.textures.iter().zip(textures) {
            copy_texture(encoder, from, to);
        }
        for (from, to) in self.buffers.iter().zip(buffers) {
            encoder.copy_buffer_to_buffer(from, 0, to, 0, from.size());
        }
        true
    }

    pub(crate) fn bytes(&self) -> u64 {
        self.textures.iter().map(texture_bytes).sum::<u64>()
            + self.buffers.iter().map(wgpu::Buffer::size).sum::<u64>()
    }
}

/// The GPU parts of a save, each copied back or started afresh on its own.
struct GpuParts {
    taa: Option<GpuCopy>,
    smaa: Option<GpuCopy>,
    gi: Option<((u32, u32), GpuCopy)>,
    ssr: Option<ssr::SsrSave>,
    fsr: post::fsr::FsrSave,
    wind: wind_field::WindSave,
    trample: trample::TrampleSave,
}

impl GpuParts {
    fn bytes(&self) -> u64 {
        self.taa.as_ref().map_or(0, GpuCopy::bytes)
            + self.smaa.as_ref().map_or(0, GpuCopy::bytes)
            + self.gi.as_ref().map_or(0, |(_, c)| c.bytes())
            + self.ssr.as_ref().map_or(0, ssr::SsrSave::bytes)
            + post::fsr::FsrPasses::saved_bytes(&self.fsr)
            + self.wind.bytes()
            + self.trample.bytes()
    }
}

/// One [`Renderer::save_state`], or what a capture puts back.
pub(crate) struct FrameSave {
    /// [`post::PostPasses::save_exposure`]'s.
    exposure: u64,
    /// `frame_index` (its parity picks the ping-pong slots) and the phase.
    parity: u64,
    phase: u64,
    taa_history: Option<Mat4>,
    taa_history_frames: u64,
    /// The GI copy held the frame before the save's next.
    gi_valid: bool,
    prev_view_proj: Option<Mat4>,
    previous_time: Option<f32>,
    previous_wind: Option<(Wind, u64, f32)>,
    previous_sway: Vec<f32>,
    drawn_origin: Vec3,
    stylise_chain: Option<u16>,
    terrain: terrain::TerrainSave,
    /// `None` once dropped for [`HISTORY_BYTES`].
    gpu: Option<GpuParts>,
}

/// The saves kept, newest last.
#[derive(Default)]
pub(crate) struct Saves {
    next: u64,
    kept: VecDeque<(u64, FrameSave)>,
}

/// What a restore brought back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Restored {
    /// Everything the save kept: the next frame is the uninterrupted run's.
    Whole,
    /// The adapted exposure and the CPU state, with the GPU histories
    /// started afresh (the save's copies were dropped for memory).
    Afresh,
    /// Nothing kept by that id (0, or a save no longer kept): the frame
    /// history starts afresh.
    None,
}

impl Renderer {
    /// Keeps the frame history (module docs) for a world snapshot; a
    /// [`Renderer::restore_state`] with the answer brings it back. Kept
    /// on the GPU, no readback: the last [`SAVES`] saves, their GPU copies
    /// within [`HISTORY_BYTES`].
    pub fn save_state(&mut self) -> RendererState {
        let save = self.take_frames();
        let saves = &mut self.saves;
        saves.next += 1;
        let id = saves.next;
        saves.kept.push_back((id, save));
        while saves.kept.len() > SAVES {
            saves.kept.pop_front();
        }
        // The newest copies within the budget, the newest whatever it holds.
        let mut bytes = 0u64;
        for (i, (_, s)) in saves.kept.iter_mut().rev().enumerate() {
            if let Some(g) = &s.gpu {
                bytes += g.bytes();
                if i > 0 && bytes > HISTORY_BYTES {
                    s.gpu = None;
                }
            }
        }
        RendererState { id }
    }

    /// The frame history now, copied (module docs).
    pub(crate) fn take_frames(&mut self) -> FrameSave {
        let mut encoder = Encoder::untimed(self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("save state"),
            },
        ));
        let exposure = self.post.save_exposure(&mut encoder);
        let gpu = {
            let device = &self.device;
            let e: &mut wgpu::CommandEncoder = &mut encoder;
            let pair = |p: &Option<[wgpu::TextureView; 2]>, e: &mut wgpu::CommandEncoder| {
                p.as_ref()
                    .map(|[a, b]| GpuCopy::of(device, e, &[a.texture(), b.texture()], &[]))
            };
            GpuParts {
                taa: pair(&self.persistent.history, e),
                smaa: pair(&self.persistent.smaa_history, e),
                gi: self.persistent.gi.as_ref().map(|(size, [a, b])| {
                    (
                        *size,
                        GpuCopy::of(device, e, &[a.texture(), b.texture()], &[]),
                    )
                }),
                ssr: self.ssr.save(device, e),
                fsr: self.post.fsr.save(device, e),
                wind: self.wind_field.save(device, e),
                trample: self.trample.save(device, e),
            }
        };
        self.queue.submit(Some(encoder.finish()));
        FrameSave {
            exposure,
            parity: self.frame_index % 2,
            phase: self.phase(),
            taa_history: self.history.taa_history,
            taa_history_frames: self.history.taa_history_frames,
            gi_valid: self.gi_history.is_some_and(|f| f + 1 == self.frame_index),
            prev_view_proj: self.history.prev_view_proj,
            previous_time: self.history.previous_time,
            previous_wind: self.history.previous_wind,
            previous_sway: self.instances.previous_sway.clone(),
            drawn_origin: self.history.drawn_origin,
            stylise_chain: self.stylise_chain,
            terrain: self.terrain.save(),
            gpu: Some(gpu),
        }
    }

    /// Brings back what [`Renderer::save_state`] kept: the frames drawn
    /// next go on from the saved frame's history. A save no longer kept
    /// (or 0) starts the frame history afresh, as a camera cut does, and
    /// the next adapted frame takes the metered exposure.
    pub fn restore_state(&mut self, state: RendererState) -> Restored {
        let at = self
            .saves
            .kept
            .iter()
            .position(|(id, _)| *id == state.id && state.id != 0);
        let restored = match at {
            None => {
                self.forget_frames();
                let mut encoder = Encoder::untimed(self.device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor {
                        label: Some("restore state"),
                    },
                ));
                self.post.restore_exposure(&mut encoder, 0);
                self.queue.submit(Some(encoder.finish()));
                Restored::None
            }
            Some(at) => {
                let (id, save) = self.saves.kept.remove(at).expect("found");
                let whole = self.put_frames(&save);
                self.saves.kept.insert(at, (id, save));
                if whole {
                    Restored::Whole
                } else {
                    Restored::Afresh
                }
            }
        };
        self.views.stale = true;
        restored
    }

    /// Puts back the frame history of [`Renderer::take_frames`]: true when
    /// every part came back, false when some started afresh.
    pub(crate) fn put_frames(&mut self, s: &FrameSave) -> bool {
        self.forget_frames();
        let mut encoder = Encoder::untimed(self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("restore state"),
            },
        ));
        self.post.restore_exposure(&mut encoder, s.exposure);
        // The ping-pong slots go by the frame's parity: on the saved one, so
        // each history is read from the slot it was written to.
        // `frame_index` only grows (readbacks are stamped with it).
        if self.frame_index % 2 != s.parity {
            self.frame_index += 1;
        }
        self.phase_base = self.frame_index.saturating_sub(s.phase);
        self.history.prev_view_proj = s.prev_view_proj;
        self.history.previous_time = s.previous_time;
        self.history.previous_wind = s.previous_wind;
        self.instances.previous_sway.clone_from(&s.previous_sway);
        self.history.drawn_origin = s.drawn_origin;
        self.stylise_chain = s.stylise_chain;
        self.terrain.restore(&s.terrain);
        let mut whole = s.gpu.is_some();
        if let Some(g) = &s.gpu {
            let e: &mut wgpu::CommandEncoder = &mut encoder;
            let pair = |c: &Option<GpuCopy>,
                        p: &Option<[wgpu::TextureView; 2]>,
                        e: &mut wgpu::CommandEncoder| match (c, p) {
                (Some(c), Some([a, b])) => c.back(e, &[a.texture(), b.texture()], &[]),
                (None, None) => true,
                _ => false,
            };
            if pair(&g.taa, &self.persistent.history, e) {
                self.history.taa_history = s.taa_history;
                self.history.taa_history_frames = s.taa_history_frames;
            } else {
                whole = false;
            }
            whole &= pair(&g.smaa, &self.persistent.smaa_history, e);
            let gi = match (&g.gi, &self.persistent.gi) {
                (Some((size, c)), Some((now, [a, b]))) if size == now => {
                    c.back(e, &[a.texture(), b.texture()], &[])
                }
                (None, _) => !s.gi_valid,
                _ => false,
            };
            if gi && s.gi_valid {
                self.gi_history = Some(self.frame_index - 1);
            }
            whole &= gi;
            whole &= self.ssr.restore(e, g.ssr.as_ref());
            whole &= self.post.fsr.restore(e, &g.fsr);
            whole &= self.wind_field.restore(e, &g.wind);
            whole &= self.trample.restore(e, &g.trample);
        }
        self.queue.submit(Some(encoder.finish()));
        whole
    }
}

/// What the frame carries to the next: the previous camera, clock and
/// wind (motion vectors), TAA's history, and the origin it is in.
pub(crate) struct FrameHistory {
    /// The previous frame's unjittered view-projection (motion vectors).
    pub(crate) prev_view_proj: Option<Mat4>,
    /// The last drawn clock, including frames with no simulation tick.
    pub(crate) previous_time: Option<f32>,
    /// The last drawn wind, tick and tick rate, including paused frames.
    pub(crate) previous_wind: Option<(Wind, u64, f32)>,
    /// The previous frame's unjittered view-projection, when TAA history is valid.
    pub(crate) taa_history: Option<Mat4>,
    /// Frames TAA has accumulated into its history since it was last
    /// reset, this one included (0 without TAA): [`Renderer::history`].
    pub(crate) taa_history_frames: u64,
    /// The scene origin the last frame was drawn at ([`RenderScene::origin`]):
    /// the history above is in its coordinates.
    pub(crate) drawn_origin: Vec3,
}
