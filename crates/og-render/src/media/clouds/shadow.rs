//! The clouds' shadow on the world: how much of the sun's beam the layer
//! lets through, baked over one period of the cloud field
//! (cloud_shadow_bake.wgsl) and read by every lit surface with the sun's
//! own shadow (lighting.wgsl `cloud_shadow`, the frame group's
//! `cloud_shadow_map`). The field repeats every four tiles of the noise,
//! so the map tiles over the world: the wind's drift and the eye's place
//! only shift where it is read, and it is baked again only when the layer,
//! the sun's height or the shapes' turning over move it a step.

use super::{CloudNoise, Clouds};
use glam::{Vec2, Vec3};

/// The map's side in texels: a texel is half a texel of the noise at the
/// shapes' own scale, and the fine erosion is below it.
pub(crate) const SIDE: u32 = 512;
/// A storage format a shader may read without a texture slot (frame.wgsl).
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;
/// The steepest ray the map is baked along and read at, as its run per
/// metre it rises (a sun 9.5° up): lower, the shadows keep that slant.
const RUN: f32 = 6.0;

/// The map the frame groups bind (a texel of nothing until a frame's clouds
/// cast a shadow), what bakes it, and what it holds.
pub(crate) struct CloudShadow {
    pub view: wgpu::TextureView,
    pass: Option<BakePass>,
    /// What the map was last baked from.
    baked: Option<Bake>,
    /// The frame's bake, planned ([`CloudShadow::update`]) and not yet
    /// recorded ([`CloudShadow::encode`]).
    pending: bool,
    /// The frame uniform's rows for this frame's shadow, when it has one.
    rows: Option<([f32; 2], [f32; 4])>,
}

struct BakePass {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    frame: wgpu::BindGroup,
}

/// What a bake is a function of: the layer (as the frame uniform holds it,
/// which the bake reads) and the bake's own uniform.
#[derive(Clone, Copy, PartialEq)]
struct Bake {
    layer: [u32; 5],
    ray: [f32; 4],
}

impl Bake {
    /// The bake for `clouds` at `seconds` under a sun along `run` (its run
    /// per metre it rises): the ray through the layer rounded to half a
    /// texel's shear across its thickness, the shapes' turning over to a
    /// thirty-second of the noise's texel, so the map holds still between
    /// steps too small to see and is the same whatever was drawn before.
    fn of(clouds: &Clouds, seconds: f64, run: Vec2) -> Bake {
        let scale = clouds.scale.clamp(500.0, 200_000.0);
        let thickness = clouds.thickness.max(1.0);
        let texel = 4.0 * scale / SIDE as f32;
        let step = (0.5 * texel / thickness).clamp(0.005, 0.25);
        let run = (run / step).round() * step;
        let lift = scale / 2048.0;
        let period = f64::from(scale) * 4.0;
        let turned = (f64::from(clouds.wind.y) * seconds).rem_euclid(period);
        let turned = ((turned / f64::from(lift)).round() * f64::from(lift)) as f32;
        let steps = (12.0 * (1.0 + run.length_squared()).sqrt())
            .ceil()
            .clamp(12.0, 64.0);
        Bake {
            layer: [
                clouds.coverage.clamp(0.0, 1.0),
                clouds.density.max(0.0),
                clouds.base,
                clouds.thickness.max(0.0),
                scale,
            ]
            .map(f32::to_bits),
            ray: [run.x, turned, run.y, steps],
        }
    }
}

impl Clouds {
    /// How much of the layer's shadow falls on the world: all of
    /// [`Clouds::shadows`] under a broken sky, less as the deck closes
    /// ([`Clouds::light`] dims the sun by what it hides, and a closed deck
    /// has no gap to cast a shadow's edge), none without a layer.
    pub(crate) fn shadow(&self) -> f32 {
        if self.thickness <= 0.0 || self.density <= 0.0 || self.coverage <= 0.0 {
            return 0.0;
        }
        self.shadows.clamp(0.0, 1.0) * (1.0 - self.closed())
    }
}

impl CloudShadow {
    pub(crate) fn placeholder(device: &wgpu::Device) -> CloudShadow {
        CloudShadow {
            view: texture(device, 1),
            pass: None,
            baked: None,
            pending: false,
            rows: None,
        }
    }

    /// Whether the full map exists (the frame groups must be made again the
    /// frame it does).
    pub(crate) fn made(&self) -> bool {
        self.pass.is_some()
    }

    /// The frame uniform's rows while this frame's clouds cast a shadow:
    /// the ray's run per metre it rises (`frame.clouds[10].zw`) and the
    /// map's side and the shadow's strength (`[11]`).
    pub(crate) fn rows(&self) -> Option<([f32; 2], [f32; 4])> {
        self.rows
    }

    /// Plans the frame's shadow for `clouds` at `seconds` under a light
    /// towards `to_light`, and its bake when the map holds another:
    /// [`CloudShadow::encode`] records it before any pass reads the map.
    /// `None`, or clouds that cast none, or a light from below: no shadow.
    /// `frame`: the main camera's frame uniform, whose clouds the bake reads.
    pub(crate) fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        clouds: Option<(&Clouds, f64, Vec3)>,
        noise: &CloudNoise,
        frame: &wgpu::Buffer,
    ) {
        self.rows = None;
        let Some((clouds, seconds, to_light)) = clouds else {
            return;
        };
        let strength = clouds.shadow();
        let to_light = to_light.normalize_or_zero();
        if strength <= 0.0 || to_light.y <= 1e-3 || !noise.made {
            return;
        }
        let run = (Vec2::new(to_light.x, to_light.z) / to_light.y).clamp_length_max(RUN);
        let bake = Bake::of(clouds, seconds, run);
        if self.pass.is_none() {
            self.view = texture(device, SIDE);
            self.pass = Some(BakePass::new(device, frame));
        }
        let pass = self.pass.as_ref().expect("made above");
        if self.baked != Some(bake) {
            queue.write_buffer(&pass.uniform, 0, bytemuck::cast_slice(&bake.ray));
            self.baked = Some(bake);
            self.pending = true;
        }
        self.rows = Some((run.to_array(), [SIDE as f32, strength, 0.0, 0.0]));
    }

    /// Records the bake [`CloudShadow::update`] planned: nothing without one.
    pub(crate) fn encode(
        &mut self,
        device: &wgpu::Device,
        noise: &CloudNoise,
        encoder: &mut crate::Encoder,
    ) {
        let (true, Some(pass)) = (std::mem::take(&mut self.pending), &self.pass) else {
            return;
        };
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cloud shadow"),
            layout: &pass.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: pass.uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&noise.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&noise.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&self.view),
                },
            ],
        });
        let mut compute = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("cloud shadow"),
            timestamp_writes: None,
        });
        compute.set_pipeline(&pass.pipeline);
        compute.set_bind_group(0, &pass.frame, &[]);
        compute.set_bind_group(1, &group, &[]);
        compute.dispatch_workgroups(SIDE / 8, SIDE / 8, 1);
    }

    /// The lit shaders' read of the map (the frame group's
    /// `cloud_shadow_map`, frame.wgsl), in `stages`.
    pub(crate) fn layout_entry(
        binding: u32,
        stages: wgpu::ShaderStages,
    ) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: stages,
            ty: wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::ReadOnly,
                format: FORMAT,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None,
        }
    }
}

fn texture(device: &wgpu::Device, side: u32) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("cloud shadow"),
            size: wgpu::Extent3d {
                width: side,
                height: side,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        })
        .create_view(&Default::default())
}

impl BakePass {
    fn new(device: &wgpu::Device, frame: &wgpu::Buffer) -> BakePass {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cloud shadow"),
            source: wgpu::ShaderSource::Wgsl(bake_source().into()),
        });
        let entry = |binding, ty| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty,
            count: None,
        };
        let uniform_ty = wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        };
        let frame_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cloud shadow frame"),
            entries: &[entry(0, uniform_ty)],
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cloud shadow"),
            entries: &[
                entry(0, uniform_ty),
                entry(
                    1,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                ),
                entry(
                    2,
                    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                ),
                entry(
                    3,
                    wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: FORMAT,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                ),
            ],
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("cloud shadow"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("cloud shadow"),
                    bind_group_layouts: &[Some(&frame_layout), Some(&layout)],
                    immediate_size: 0,
                }),
            ),
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cloud shadow"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let frame = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cloud shadow frame"),
            layout: &frame_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: frame.as_entire_binding(),
            }],
        });
        BakePass {
            pipeline,
            layout,
            uniform,
            frame,
        }
    }
}

/// The bake's module: the frame, the clouds' shape, the bake.
pub(crate) fn bake_source() -> String {
    format!(
        "{}{}{}",
        crate::FRAME_WGSL,
        wgsl!("clouds_shape.wgsl"),
        wgsl!("cloud_shadow_bake.wgsl")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The map holds still while the sun and the shapes move less than a
    /// step, and is baked again when the layer changes or they move one.
    #[test]
    fn the_bake_is_a_function_of_rounded_inputs() {
        let clouds = Clouds::default();
        let at = |seconds: f64, run: Vec2| Bake::of(&clouds, seconds, run);
        let a = at(10.0, Vec2::new(0.50, 0.20));
        assert!(a == at(10.5, Vec2::new(0.501, 0.201)));
        assert!(a != at(10.0, Vec2::new(0.56, 0.20)));
        // 0.4 m a second: 4.4 m (a step) in 11 s.
        assert!(a != at(21.0, Vec2::new(0.50, 0.20)));
        let thicker = Clouds {
            coverage: 0.5,
            ..clouds
        };
        assert!(a != Bake::of(&thicker, 10.0, Vec2::new(0.50, 0.20)));
        // A low sun takes more steps along its longer way through, within
        // the shader's bound.
        assert_eq!(a.ray[3], 14.0);
        assert_eq!(at(0.0, Vec2::new(RUN, 0.0)).ray[3], 64.0);
    }

    /// A broken sky casts its whole shadow, a closed deck none, and a
    /// layer of no thickness none.
    #[test]
    fn the_shadow_fades_as_the_deck_closes() {
        let of = |coverage: f32| {
            Clouds {
                coverage,
                ..Clouds::default()
            }
            .shadow()
        };
        assert_eq!(of(0.45), 1.0);
        assert!(of(0.8) > 0.2 && of(0.8) < 0.6, "{}", of(0.8));
        assert_eq!(of(1.0), 0.0);
        assert_eq!(of(0.0), 0.0);
        let flat = Clouds {
            thickness: 0.0,
            ..Clouds::default()
        };
        assert_eq!(flat.shadow(), 0.0);
        let off = Clouds {
            shadows: 0.0,
            ..Clouds::default()
        };
        assert_eq!(off.shadow(), 0.0);
    }
}
