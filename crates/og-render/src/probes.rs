//! Reflection probes (`docs/design.md` §2.6): a box of the world whose
//! reflections come from a cube captured at a point inside it rather than
//! from the environment map.
//!
//! A probe's capture is six 90° views of the scene from its position, drawn
//! with the lit pipelines into a cube (the sun, the environment, every
//! local light and the sky over the opaque, skinned and transparent
//! meshes, terrain and scattered copies, then particles, lines and points
//! and world sprites and text on every layer; `lib.rs`'s probe pass),
//! copied into a radiance cube with mips and GGX-prefiltered into
//! [`crate::ENVIRONMENT_MAP_LEVELS`] levels like the environment's own
//! (`ibl.rs`). A baked probe draws its capture once, the first frame it is
//! in a scene, unless it brings one ([`ReflectionProbe::baked`], an
//! environment map `og bake probes` wrote); a realtime probe draws it again
//! every [`ReflectionProbe::interval`] ticks.
//!
//! Sampling costs no texture slot (WebGL2's skinned lit pipeline has none
//! left): a draw whose bounds fall in a probe's box binds the probe's frame
//! group, whose uniform carries the box and whose irradiance and
//! prefiltered bindings are the environment's prefiltered cube and the
//! probe's. The lit shaders then box-project the reflected ray onto the
//! probe's box and blend the probe with the environment by the distance to
//! the box's faces (`frame.wgsl`'s `reflected_light`, `ambient_light`).
//! Each instance takes one probe, the smallest box holding the centre of
//! its bounds; instances of one mesh and material in different boxes are
//! different batches (`probe_select.rs`).

use crate::frame::FrameUniform;
use crate::scene::{Camera, Projection};
use crate::{DEPTH_FORMAT, EnvironmentMapHandle, HDR_FORMAT};
use glam::{Vec2, Vec3};

/// Probes sampled in one frame: the first ones of [`crate::RenderScene::reflection_probes`].
pub const MAX_REFLECTION_PROBES: usize = 8;
/// A probe's capture read back: its face size and its
/// [`crate::ENVIRONMENT_MAP_LEVELS`] prefiltered levels, each six faces of
/// linear RGB floats (`environment::bake::PrefilteredCube`'s layout).
pub type ProbeCapture = (u32, Vec<Vec<f32>>);

/// The face sizes a probe captures at (powers of two).
pub const PROBE_MIN_RESOLUTION: u32 = crate::ENVIRONMENT_MAP_MIN_SIZE;
pub const PROBE_MAX_RESOLUTION: u32 = 1024;

/// When a probe's capture is drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ProbeMode {
    /// Once: from [`ReflectionProbe::baked`] when it has one, else drawn the
    /// first frame the probe is in the scene.
    #[default]
    Baked,
    /// Every [`ReflectionProbe::interval`] ticks.
    Realtime,
}

/// A reflection probe of a [`crate::RenderScene`]: an axis-aligned box
/// centred on `position` whose surfaces reflect the cube captured there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReflectionProbe {
    /// Who it is across frames (an entity id): its cube is kept by it.
    pub id: u64,
    /// The capture point and the box's centre (world).
    pub position: Vec3,
    /// The box's full extents (world units).
    pub size: Vec3,
    /// The face size it captures at: rounded up to a power of two and
    /// clamped to [`PROBE_MIN_RESOLUTION`]..=[`PROBE_MAX_RESOLUTION`].
    pub resolution: u32,
    pub mode: ProbeMode,
    /// Ticks between a realtime probe's captures (0: every frame).
    pub interval: u32,
    /// Scales what it reflects.
    pub intensity: f32,
    /// How far inside the box the probe blends into the environment map.
    pub blend_distance: f32,
    /// A baked capture to sample instead of drawing one (an environment map
    /// asset of the probe's face size: `og bake probes`).
    pub baked: Option<EnvironmentMapHandle>,
}

impl ReflectionProbe {
    /// A baked probe at `position` with a box of `size`: 128² faces,
    /// intensity 1, a metre of blending.
    pub fn new(id: u64, position: Vec3, size: Vec3) -> ReflectionProbe {
        ReflectionProbe {
            id,
            position,
            size,
            resolution: 128,
            mode: ProbeMode::Baked,
            interval: 30,
            intensity: 1.0,
            blend_distance: 1.0,
            baked: None,
        }
    }

    /// The face size its capture is drawn at.
    pub fn face_size(&self) -> u32 {
        self.resolution
            .clamp(PROBE_MIN_RESOLUTION, PROBE_MAX_RESOLUTION)
            .next_power_of_two()
    }

    /// The box's min and max corners.
    pub fn bounds(&self) -> (Vec3, Vec3) {
        let h = self.size.abs() * 0.5;
        (self.position - h, self.position + h)
    }

    /// The frame uniform's probe rows (`FrameUniform::probe`).
    fn rows(&self) -> [[f32; 4]; 3] {
        let (lo, hi) = self.bounds();
        [
            lo.extend(1.0).to_array(),
            hi.extend(self.intensity.max(0.0)).to_array(),
            self.position
                .extend(self.blend_distance.max(0.0))
                .to_array(),
        ]
    }
}

/// The camera that draws cube face `face` (wgpu's order: +x, -x, +y, -y,
/// +z, -z) from `at`: right-handed, so each face comes out mirrored across
/// its vertical axis, which `ibl.wgsl`'s `fs_probe_copy` undoes.
pub(crate) fn face_camera(base: &Camera, at: Vec3, face: u32, far: f32) -> Camera {
    let (forward, up) = match face {
        0 => (Vec3::X, Vec3::Y),
        1 => (Vec3::NEG_X, Vec3::Y),
        2 => (Vec3::Y, Vec3::NEG_Z),
        3 => (Vec3::NEG_Y, Vec3::Z),
        4 => (Vec3::Z, Vec3::Y),
        _ => (Vec3::NEG_Z, Vec3::Y),
    };
    Camera {
        position: at,
        target: at + forward,
        up,
        projection: Projection::Perspective {
            fov_y_degrees: 90.0,
        },
        near: 0.05,
        far,
        exposure: base.exposure,
    }
}

/// One probe's cube and what draws and filters it.
pub(crate) struct ProbeSlot {
    pub id: u64,
    pub size: u32,
    samples: u32,
    /// The tick of the last capture (`None`: never drawn).
    pub captured: Option<u64>,
    /// The six faces as drawn (one level): each one's view, the MSAA
    /// target it resolves from and the depth.
    pub faces: [wgpu::TextureView; 6],
    pub msaa: Option<wgpu::TextureView>,
    pub depth: wgpu::TextureView,
    pub filter: crate::ibl::ProbeFilter,
    /// The prefiltered cube the lit shaders sample (and `og bake probes` reads).
    pub prefiltered: wgpu::Texture,
    pub view: wgpu::TextureView,
    _textures: Vec<wgpu::Texture>,
}

#[allow(clippy::too_many_arguments)]
fn texture(
    device: &wgpu::Device,
    label: &str,
    size: u32,
    layers: u32,
    mips: u32,
    samples: u32,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: layers,
        },
        mip_level_count: mips,
        sample_count: samples,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

impl ProbeSlot {
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        ibl: &crate::ibl::IblGenerator,
        id: u64,
        size: u32,
        samples: u32,
    ) -> ProbeSlot {
        let attach = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        let capture = texture(device, "probe capture", size, 6, 1, 1, HDR_FORMAT, attach);
        let radiance = texture(
            device,
            "probe radiance",
            size,
            6,
            size.ilog2() + 1,
            1,
            HDR_FORMAT,
            attach,
        );
        let prefiltered = texture(
            device,
            "probe prefiltered",
            size,
            6,
            crate::ibl::PREFILTER_MIPS,
            1,
            HDR_FORMAT,
            attach | wgpu::TextureUsages::COPY_SRC,
        );
        let depth = texture(
            device,
            "probe depth",
            size,
            1,
            1,
            samples,
            DEPTH_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        let msaa = (samples > 1).then(|| {
            texture(
                device,
                "probe msaa",
                size,
                1,
                1,
                samples,
                HDR_FORMAT,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            )
        });
        let faces = std::array::from_fn(|face| {
            capture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("probe face"),
                dimension: Some(wgpu::TextureViewDimension::D2),
                base_array_layer: face as u32,
                array_layer_count: Some(1),
                ..Default::default()
            })
        });
        let filter = ibl.probe_filter(device, queue, &capture, &radiance, &prefiltered, size);
        let view = prefiltered.create_view(&wgpu::TextureViewDescriptor {
            label: Some("probe"),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });
        let depth_view = depth.create_view(&Default::default());
        let msaa_view = msaa.as_ref().map(|t| t.create_view(&Default::default()));
        let mut textures = vec![capture, radiance, depth];
        textures.extend(msaa);
        ProbeSlot {
            id,
            size,
            samples,
            captured: None,
            faces,
            msaa: msaa_view,
            depth: depth_view,
            filter,
            prefiltered,
            view,
            _textures: textures,
        }
    }
}

/// What the frame does with its probes ([`Probes::prepare`]).
#[derive(Default)]
pub(crate) struct ProbeFrame {
    /// The probes sampled this frame (the scene's first
    /// [`MAX_REFLECTION_PROBES`]), each with its slot (`None`: baked from
    /// an environment map).
    pub active: Vec<(ReflectionProbe, Option<usize>)>,
    /// Slots whose capture is drawn this frame, with their active index.
    pub due: Vec<(usize, usize)>,
}

/// The renderer's probes: a slot per probe that draws its own capture.
#[derive(Default)]
pub(crate) struct Probes {
    pub slots: Vec<ProbeSlot>,
    /// Frame uniforms: one per active probe, then six per due capture.
    pub uniforms: Option<(wgpu::Buffer, u64)>,
}

impl Probes {
    /// The frame's probes: slots made for new ones (and remade for a new
    /// face size or sample count), dropped for those gone, and the
    /// captures due at `tick`. `baked` tells whether a baked probe's
    /// environment map is loaded.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        ibl: &crate::ibl::IblGenerator,
        probes: &[ReflectionProbe],
        tick: u64,
        samples: u32,
        baked: impl Fn(EnvironmentMapHandle) -> bool,
    ) -> ProbeFrame {
        let mut frame = ProbeFrame::default();
        let active: Vec<ReflectionProbe> = probes
            .iter()
            .filter(|p| p.size.cmpgt(Vec3::ZERO).all())
            .take(MAX_REFLECTION_PROBES)
            .copied()
            .collect();
        let own =
            |p: &ReflectionProbe| !(p.mode == ProbeMode::Baked && p.baked.is_some_and(&baked));
        self.slots.retain(|s| {
            active
                .iter()
                .any(|p| p.id == s.id && own(p) && p.face_size() == s.size && samples == s.samples)
        });
        for (i, p) in active.iter().enumerate() {
            if !own(p) {
                frame.active.push((*p, None));
                continue;
            }
            let slot = match self.slots.iter().position(|s| s.id == p.id) {
                Some(s) => s,
                None => {
                    self.slots.push(ProbeSlot::new(
                        device,
                        queue,
                        ibl,
                        p.id,
                        p.face_size(),
                        samples,
                    ));
                    self.slots.len() - 1
                }
            };
            let due = match (p.mode, self.slots[slot].captured) {
                (_, None) => true,
                (ProbeMode::Baked, Some(_)) => false,
                (ProbeMode::Realtime, Some(at)) => {
                    tick.saturating_sub(at) >= p.interval as u64 || tick < at
                }
            };
            if due {
                frame.due.push((slot, i));
            }
            frame.active.push((*p, Some(slot)));
        }
        let slots = frame.active.len() + 6 * frame.due.len();
        let stride = (std::mem::size_of::<FrameUniform>() as u64)
            .next_multiple_of(device.limits().min_uniform_buffer_offset_alignment as u64);
        if slots > 0
            && self
                .uniforms
                .as_ref()
                .is_none_or(|(b, _)| b.size() < stride * slots as u64)
        {
            self.uniforms = Some((
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("probe frames"),
                    size: stride * (MAX_REFLECTION_PROBES * 7) as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                stride,
            ));
        }
        frame
    }

    /// The uniform binding of slot `k` of the frame's uniforms.
    pub fn binding(&self, k: usize) -> wgpu::BindingResource<'_> {
        let (buffer, stride) = self.uniforms.as_ref().expect("made with the probes");
        wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer,
            offset: stride * k as u64,
            size: wgpu::BufferSize::new(std::mem::size_of::<FrameUniform>() as u64),
        })
    }

    /// Writes the probes' frame uniforms from the frame's own `base`: each
    /// active probe's (its rows set), then each due capture's six faces
    /// (`faces[i]`: the face cameras' view-projection, inverse, position
    /// and forward), drawn without clusters (every light), caustics,
    /// stylisation or probe.
    pub fn write(
        &self,
        queue: &wgpu::Queue,
        base: &FrameUniform,
        frame: &ProbeFrame,
        faces: &[[crate::frame::CameraFrame; 6]],
        positions: &[Vec3],
    ) {
        let Some((buffer, stride)) = &self.uniforms else {
            return;
        };
        let stride = *stride as usize;
        let count = frame.active.len() + 6 * frame.due.len();
        let mut bytes = vec![0u8; stride * count];
        let size = std::mem::size_of::<FrameUniform>();
        let mut put = |k: usize, u: &FrameUniform| {
            bytes[k * stride..k * stride + size].copy_from_slice(bytemuck::bytes_of(u));
        };
        for (k, (p, _)) in frame.active.iter().enumerate() {
            let mut u = *base;
            u.probe = p.rows();
            put(k, &u);
        }
        for (d, cams) in faces.iter().enumerate() {
            for (face, f) in cams.iter().enumerate() {
                let mut u = *base;
                u.view_proj = f.view_proj.to_cols_array_2d();
                u.inv_view_proj = f.view_proj.inverse().to_cols_array_2d();
                u.view_pos = positions[d].extend(0.0).to_array();
                u.view_forward = f.forward.extend(base.view_forward[3]).to_array();
                u.cluster = [0.0; 4];
                u.caustics = [[0.0; 4]; 2];
                u.stylise = [0.0; 4];
                u.probe = [[0.0; 4]; 3];
                put(frame.active.len() + 6 * d + face, &u);
            }
        }
        queue.write_buffer(buffer, 0, &bytes);
    }
}

/// The face cameras of a capture at `at` (their matrices, unjittered).
pub(crate) fn face_frames(base: &Camera, at: Vec3, far: f32) -> [crate::frame::CameraFrame; 6] {
    std::array::from_fn(|face| {
        crate::frame::camera_frame(&face_camera(base, at, face as u32, far), 1.0, Vec2::ZERO)
    })
}

/// IEEE half to f32 (the prefiltered cube's texels, read back).
fn f16_to_f32(h: u16) -> f32 {
    let sign = if h & 0x8000 != 0 { -1.0 } else { 1.0 };
    let exp = ((h >> 10) & 0x1f) as i32;
    let man = (h & 0x3ff) as f32;
    match exp {
        0 => sign * man * 2f32.powi(-24),
        31 => {
            if man == 0.0 {
                sign * f32::INFINITY
            } else {
                f32::NAN
            }
        }
        _ => sign * (1.0 + man / 1024.0) * 2f32.powi(exp - 15),
    }
}

/// Reads a slot's prefiltered cube back (blocking): its
/// [`crate::ENVIRONMENT_MAP_LEVELS`] levels, each six faces of linear RGB
/// floats in `environment::bake::PrefilteredCube`'s layout (what
/// `og bake probes` turns into an environment map).
pub(crate) fn read_back(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    slot: &ProbeSlot,
) -> Result<Vec<Vec<f32>>, String> {
    let mut levels = Vec::new();
    for mip in 0..crate::ibl::PREFILTER_MIPS {
        let n = (slot.size >> mip).max(1);
        let row = (n * 8).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("probe read back"),
            size: (row * n * 6) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &slot.prefiltered,
                mip_level: mip,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(row),
                    rows_per_image: Some(n),
                },
            },
            wgpu::Extent3d {
                width: n,
                height: n,
                depth_or_array_layers: 6,
            },
        );
        queue.submit(Some(encoder.finish()));
        let slice = buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| format!("reading the probe back: {e}"))?;
        let data = slice
            .get_mapped_range()
            .map_err(|e| format!("reading the probe back: {e:?}"))?;
        let mut rgb = Vec::with_capacity((n * n * 18) as usize);
        for y in 0..n * 6 {
            let line = &data[(y * row) as usize..(y * row + n * 8) as usize];
            for px in line.as_chunks::<8>().0 {
                for c in 0..3 {
                    rgb.push(f16_to_f32(u16::from_le_bytes([px[2 * c], px[2 * c + 1]])).max(0.0));
                }
            }
        }
        drop(data);
        buffer.unmap();
        levels.push(rgb);
    }
    Ok(levels)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Face sizes are powers of two in range; a probe of resolution 100
    /// captures at 128.
    #[test]
    fn face_sizes_round_up_to_powers_of_two() {
        let mut p = ReflectionProbe::new(1, Vec3::ZERO, Vec3::ONE);
        for (asked, size) in [(100, 128), (1, 16), (4096, 1024), (256, 256)] {
            p.resolution = asked;
            assert_eq!(p.face_size(), size);
        }
    }

    /// Each face camera looks along its cube axis, and its right (forward ×
    /// up) is the opposite of the cube's +u axis for that face: the mirror
    /// `fs_probe_copy` undoes.
    #[test]
    fn face_cameras_mirror_the_cube_faces() {
        for face in 0..6u32 {
            let c = face_camera(&Camera::default(), Vec3::ZERO, face, 100.0);
            let forward = (c.target - c.position).normalize();
            assert!(forward.distance(crate::face_direction(face, 0.0, 0.0)) < 1e-5);
            let u = crate::face_direction(face, 1.0, 0.0) * 2f32.sqrt() - forward;
            assert!((forward.cross(c.up) + u).length() < 1e-4, "face {face}");
        }
    }

    #[test]
    fn halves_decode() {
        assert_eq!(f16_to_f32(0x3c00), 1.0);
        assert_eq!(f16_to_f32(0x4000), 2.0);
        assert_eq!(f16_to_f32(0x3800), 0.5);
        assert_eq!(f16_to_f32(0), 0.0);
    }
}
