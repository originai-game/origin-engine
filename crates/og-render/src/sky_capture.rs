//! The live sky (`docs/design.md` §2.6): the image-based light and the
//! reflections' fallback (SSR's misses, surfaces outside every probe)
//! follow a procedural sky as it moves (drifting and flashing clouds, an
//! aurora, a fast day-night), not only when its sun steps.
//!
//! Unreal's real-time sky light capture, ported: while the background is
//! the per-pixel sky (`Background::Procedural`, `sky.wgsl`'s
//! `fs_sky_look`), the sky is drawn from the camera into a
//! [`CAPTURE_SIZE`]² cube with the same shader and the frame's uniform
//! under each face camera (stars off), then filtered as a reflection
//! probe is (`ibl.rs`'s probe filter: the mip chain, the irradiance
//! convolution and the GGX-prefiltered levels) into cubes of its own, and
//! copied into the cubes the frame binds when they are complete, so a
//! frame never samples a half-made sky and no binding is added. The work
//! is spread over [`STEPS`] frames, each about an eighth of it (Unreal
//! slices it over nine): a capture is at most that many frames old, and a
//! frame's cost stays bounded whatever the sky draws. The first frame of a
//! live sky draws every step at once, so it never starts on a stale one.
//!
//! The capture draws the clouds with the bound irradiance as their
//! ambient light, the last capture's: the clouds light the sky they lit,
//! converging over a few captures, as Unreal's does.

use crate::frame::FrameUniform;
use crate::ibl::{IblGenerator, PREFILTER_MIPS, PREFILTER_SIZE, ProbeFilter};
use crate::scene::Camera;
use crate::{Encoder, HDR_FORMAT};
use std::ops::Range;

/// The face size of the capture (the prefiltered cube's, which it fills).
pub(crate) const CAPTURE_SIZE: u32 = PREFILTER_SIZE;
/// The frames one capture is spread over.
pub(crate) const STEPS: usize = 8;

/// What one step draws: capture faces, the filter's passes, and whether
/// it copies the filtered cubes into the bound ones.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Step {
    pub faces: Range<u32>,
    pub filter: Range<usize>,
    pub copy: bool,
}

/// Step `k` of a capture whose radiance has `mips` levels (`None`: every
/// step at once). The filter's passes are the probe filter's with
/// irradiance: 6 copies, 6 per lower level, 6 irradiance, 6 per
/// prefiltered level ([`ProbeFilter::len`]).
pub(crate) fn step(k: Option<usize>, mips: u32) -> Step {
    let p = 6 * (mips as usize + 1);
    let all = p + 6 * PREFILTER_MIPS as usize;
    let s = |faces: Range<u32>, filter: Range<usize>, copy| Step {
        faces,
        filter,
        copy,
    };
    match k {
        None => s(0..6, 0..all, true),
        // The faces, half a frame each.
        Some(0) => s(0..3, 0..0, false),
        Some(1) => s(3..6, 0..0, false),
        // The mips (cheap) and the irradiance.
        Some(2) => s(0..0, 0..p, false),
        // The sharpest prefiltered level over two frames, the next alone,
        // the three smallest together (each a quarter of the one above).
        Some(3) => s(0..0, p..p + 3, false),
        Some(4) => s(0..0, p + 3..p + 6, false),
        Some(5) => s(0..0, p + 6..p + 12, false),
        Some(6) => s(0..0, p + 12..all, false),
        Some(_) => s(0..0, 0..0, true),
    }
}

/// The live sky's capture, filters and schedule.
pub(crate) struct SkyCapture {
    pipeline: wgpu::RenderPipeline,
    faces: [wgpu::TextureView; 6],
    irradiance: wgpu::Texture,
    prefiltered: wgpu::Texture,
    filter: ProbeFilter,
    mips: u32,
    /// The faces' frame uniforms, `stride` apart.
    uniforms: wgpu::Buffer,
    stride: u64,
    /// The next step (`None`: all at once, the next frame); a cell, as the
    /// frame's passes record with the renderer shared.
    next: std::cell::Cell<Option<usize>>,
    _textures: [wgpu::Texture; 2],
}

impl SkyCapture {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        ibl: &IblGenerator,
        layouts: [&wgpu::BindGroupLayout; 2],
    ) -> SkyCapture {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("live sky"),
            source: wgpu::ShaderSource::Wgsl(crate::renderer::sky_source().into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("live sky"),
            bind_group_layouts: &[Some(layouts[0]), Some(layouts[1])],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("live sky"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_sky"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_sky_look"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: HDR_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: crate::pipeline_cache::of(device).as_ref(),
        });
        let capture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("live sky capture"),
            size: wgpu::Extent3d {
                width: CAPTURE_SIZE,
                height: CAPTURE_SIZE,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: HDR_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let mips = CAPTURE_SIZE.ilog2() + 1;
        let radiance = crate::ibl::cube_texture(device, "live sky radiance", CAPTURE_SIZE, mips);
        let irradiance =
            crate::ibl::cube_texture(device, "live sky irradiance", ibl.bound().0.width(), 1);
        let prefiltered =
            crate::ibl::cube_texture(device, "live sky prefiltered", CAPTURE_SIZE, PREFILTER_MIPS);
        let filter = ibl.probe_filter(
            device,
            queue,
            &capture,
            &radiance,
            Some(&irradiance),
            &prefiltered,
            CAPTURE_SIZE,
        );
        debug_assert_eq!(filter.len(), step(None, mips).filter.end);
        let faces = std::array::from_fn(|face| {
            capture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("live sky face"),
                dimension: Some(wgpu::TextureViewDimension::D2),
                base_array_layer: face as u32,
                array_layer_count: Some(1),
                ..Default::default()
            })
        });
        let stride = (std::mem::size_of::<FrameUniform>() as u64)
            .next_multiple_of(device.limits().min_uniform_buffer_offset_alignment as u64);
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("live sky frames"),
            size: stride * 6,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        SkyCapture {
            pipeline,
            faces,
            irradiance,
            prefiltered,
            filter,
            mips,
            uniforms,
            stride,
            next: std::cell::Cell::new(None),
            _textures: [capture, radiance],
        }
    }

    /// GPU bytes of its cubes.
    pub(crate) fn bytes(&self) -> u64 {
        use crate::renderer::texture_bytes;
        self._textures
            .iter()
            .chain([&self.irradiance, &self.prefiltered])
            .map(texture_bytes)
            .sum()
    }

    /// Draws every step at once the next frame (a new sky, or the bound
    /// cubes regenerated under it).
    pub fn restart(&mut self) {
        self.next.set(None);
    }

    /// This frame's step.
    pub fn due(&self) -> Step {
        step(self.next.get(), self.mips)
    }

    /// Writes the faces' frame uniforms, from the frame's own `base`, for
    /// a capture from `camera`'s position (when this frame draws faces).
    pub fn write(&self, queue: &wgpu::Queue, base: &FrameUniform, camera: &Camera) {
        let faces = self.due().faces;
        if faces.is_empty() {
            return;
        }
        let frames = crate::probes::face_frames(camera, camera.position, camera.far);
        let mut bytes = vec![0u8; (self.stride * 6) as usize];
        let size = std::mem::size_of::<FrameUniform>();
        for face in faces {
            let mut u = crate::probes::face_uniform(base, &frames[face as usize], camera.position);
            // No stars: they would only add noise to the light. The sky
            // as drawn: it is the environment the intensity then scales.
            u.view_up[3] = 0.0;
            u.atmosphere.sky[2][3] = 0.0;
            let at = face as usize * self.stride as usize;
            bytes[at..at + size].copy_from_slice(bytemuck::bytes_of(&u));
        }
        queue.write_buffer(&self.uniforms, 0, &bytes);
    }

    /// Records this frame's step: its faces (each with `frame(uniform)`,
    /// the frame group over the face's uniform, and the sky group), its
    /// filter passes and its copy into `ibl`'s bound cubes; then moves to
    /// the next.
    pub fn record(
        &self,
        encoder: &mut Encoder,
        ibl: &IblGenerator,
        frame: impl Fn(wgpu::BindingResource) -> wgpu::BindGroup,
        sky_group: &wgpu::BindGroup,
    ) {
        let Step {
            faces,
            filter,
            copy,
        } = self.due();
        for face in faces {
            let group = frame(wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &self.uniforms,
                offset: self.stride * face as u64,
                size: wgpu::BufferSize::new(std::mem::size_of::<FrameUniform>() as u64),
            }));
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("live sky face"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.faces[face as usize],
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &group, &[]);
            pass.set_bind_group(1, sky_group, &[]);
            pass.draw(0..3, 0..1);
        }
        ibl.record_filter_range(encoder, &self.filter, filter);
        if copy {
            let (irradiance, prefiltered) = ibl.bound();
            for (from, to) in [
                (&self.irradiance, irradiance),
                (&self.prefiltered, prefiltered),
            ] {
                for mip in 0..from.mip_level_count() {
                    let side = (from.width() >> mip).max(1);
                    encoder.copy_texture_to_texture(
                        level(from, mip),
                        level(to, mip),
                        wgpu::Extent3d {
                            width: side,
                            height: side,
                            depth_or_array_layers: 6,
                        },
                    );
                }
            }
        }
        self.next
            .set(Some(self.next.get().map_or(0, |k| (k + 1) % STEPS)));
    }
}

/// A texture's level `mip` as a copy's source or destination.
fn level(texture: &wgpu::Texture, mip: u32) -> wgpu::TexelCopyTextureInfo<'_> {
    wgpu::TexelCopyTextureInfo {
        texture,
        mip_level: mip,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Over a capture's steps every face is drawn once, every filter pass
    /// recorded once and in order (each after what it reads), and the
    /// copy comes last; all at once does the same in one step.
    #[test]
    fn a_capture_s_steps_do_each_part_once_in_order() {
        let mips = CAPTURE_SIZE.ilog2() + 1;
        let all = step(None, mips);
        assert_eq!((all.faces.clone(), all.copy), (0..6, true));
        let mut faces = Vec::new();
        let mut passes = Vec::new();
        for k in 0..STEPS {
            let s = step(Some(k), mips);
            assert!(
                faces.len() == 6 || s.filter.is_empty(),
                "filtered before every face is drawn"
            );
            assert_eq!(s.copy, k == STEPS - 1, "the copy is the last step");
            faces.extend(s.faces);
            passes.extend(s.filter);
        }
        assert_eq!(faces, (0..6).collect::<Vec<_>>());
        assert_eq!(passes, all.filter.collect::<Vec<_>>());
    }
}
