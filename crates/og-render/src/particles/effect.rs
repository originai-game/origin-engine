//! Effect materials: one parameter set for the stylised and realistic VFX
//! look (Effekseer's, Genshin's, Hades'), shared by every effect surface:
//! particles (billboards, velocity-stretched and mesh particles), trails,
//! ribbons, beams and effect meshes ([`EffectMesh`]). An
//! [`EffectMaterial`] is an asset ([`crate::RenderScene::add_effect_material`])
//! a surface names; one without keeps its own fixed look, drawn exactly as
//! before.
//!
//! What it does to a fragment, in order (`shaders/effect.wgsl`):
//!
//! 1. **UVs.** The main texture, the mask, the erosion noise and the flow
//!    map each have their own [`EffectUv`]: tiling, offset, scroll per
//!    second, a turn per second about the centre and polar coordinates
//!    (u the angle, v the radius: rings, swirls, portals).
//! 2. **Flow.** A flow map's red and green (0.5 still) push the main UV,
//!    two phases crossfaded so it streams without stretching.
//! 3. **Colour.** The main texture (one channel as alpha with white, or
//!    rgba), through a gradient [`EffectMaterial::ramp`] when one is given
//!    (a greyscale texture coloured by its value: the stylised gradient
//!    map), times the surface's colour, the tint and the
//!    [`EffectMaterial::intensity`].
//! 4. **Mask, erosion, fresnel, soft depth** multiply the alpha: a mask
//!    texture; noise erosion (dissolve) that cuts where the noise falls
//!    under [`Erosion::amount`] with a soft border and a glowing edge; a
//!    fresnel fade by the angle to the eye (a rim, or soft silhouettes);
//!    a fade against the scene behind (particles and effect meshes).
//! 5. **A fragment hook**: a material shader asset ([`crate::MaterialShader`],
//!    hooks mode) whose `surface_color` (and `surface_emissive`, added to
//!    the colour) runs last with the material's parameters
//!    ([`EffectMaterial::params`]). It reads the life, the random value and
//!    the custom data through `effect_life()`, `effect_random()`,
//!    `effect_custom1()` and `effect_custom2()` (0 in a lit material).
//!
//! **Driven parameters.** The UV offset, the erosion amount, the intensity,
//! the alpha and eight custom numbers are [`EffectValue`]s: a constant, a
//! random range per particle, or a curve over the life. A particle's life
//! is its age over its lifetime; a strip's (trail, ribbon, beam) runs along
//! it, 0 at its start (a trail's head) to 1 at its end; an effect mesh's
//! is [`EffectMesh::life`]. The random value is the particle's own (its
//! spawn hash), a strip's seed or an effect mesh's. They are evaluated per
//! vertex, so a particle's are constant over it.
//!
//! **Deterministic, visual only.** Everything is a function of the
//! surface's life, its random value and the scene's clock (`frame.time`,
//! from the tick): nothing reads it back, and the simulation never sees it.

use crate::particles::MAX_CURVE_KEYS;
use crate::scene::TextureHandle;
pub use og_render_data::effect::*;

// ---------------------------------------------------------------------------
// GPU form.
// ---------------------------------------------------------------------------

/// `struct EffectDriver` in effect.wgsl: a driven value.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct DriverGpu {
    /// x: 0 constant (y), 1 random (y to z), 2 curve (w keys).
    head: [f32; 4],
    t: [[f32; 4]; 2],
    v: [[f32; 4]; 2],
}

/// `struct Effect` in effect.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct EffectGpu {
    /// x: [`FLAG_ON`] and the rest, y: main channel, z: mask channel,
    /// w: noise channel.
    flags: [u32; 4],
    /// Per texture (main, mask, noise, flow): [tile, offset], then
    /// [scroll, rotation (rad/s), polar].
    uv: [[f32; 4]; 8],
    /// x: erosion softness, y: erosion edge, z: flow strength, w: flow speed.
    erosion: [f32; 4],
    edge_color: [f32; 4],
    /// x: fresnel power, y: strength, z: 1 inverted (the soft depth is
    /// folded into the surface's own, `particles_gpu.rs`).
    fresnel: [f32; 4],
    color: [f32; 4],
    /// x: ramp keys, y: flow channel.
    counts: [f32; 4],
    ramp_t: [[f32; 4]; 2],
    ramp_v: [[f32; 4]; MAX_CURVE_KEYS],
    drivers: [DriverGpu; EFFECT_DRIVERS],
    params: [[f32; 4]; EFFECT_PARAMS],
}

/// Bytes between effect uniform entries (a multiple of the 256-byte
/// dynamic offset alignment).
pub(crate) const EFFECT_STRIDE: u64 = 1792;
const _: () = assert!(std::mem::size_of::<EffectGpu>() as u64 <= EFFECT_STRIDE);

/// The material draws its own way at all (0: the surface's fixed look).
pub(crate) const FLAG_ON: u32 = 1;
/// The main texture is the material's (group 3), not the surface's.
pub(crate) const FLAG_MAIN: u32 = 2;
pub(crate) const FLAG_MASK: u32 = 4;
pub(crate) const FLAG_EROSION: u32 = 8;
/// The erosion reads its noise texture (else value noise).
pub(crate) const FLAG_NOISE: u32 = 16;
pub(crate) const FLAG_FLOW: u32 = 32;
pub(crate) const FLAG_FRESNEL: u32 = 64;
pub(crate) const FLAG_RAMP: u32 = 128;

fn uv_gpu(uv: &EffectUv) -> [[f32; 4]; 2] {
    [
        [uv.tile.x, uv.tile.y, uv.offset.x, uv.offset.y],
        [
            uv.scroll.x,
            uv.scroll.y,
            uv.rotation.to_radians(),
            f32::from(u8::from(uv.polar)),
        ],
    ]
}

fn driver_gpu(v: &EffectValue) -> DriverGpu {
    let mut d = DriverGpu::default();
    match v {
        EffectValue::Constant(c) => d.head = [0.0, *c, *c, 0.0],
        EffectValue::Random([a, b]) => d.head = [1.0, *a, *b, 0.0],
        EffectValue::Curve(c) => {
            let n = c.keys.len().min(MAX_CURVE_KEYS);
            d.head = [2.0, 0.0, 0.0, n as f32];
            for (i, (t, v)) in c.keys.iter().take(n).enumerate() {
                d.t[i / 4][i % 4] = *t;
                d.v[i / 4][i % 4] = *v;
            }
        }
    }
    d
}

impl EffectGpu {
    /// The entry of surfaces without a material: their fixed look.
    pub(crate) fn off() -> EffectGpu {
        bytemuck::Zeroable::zeroed()
    }

    pub(crate) fn of(m: &EffectMaterial) -> EffectGpu {
        let mut g = EffectGpu::off();
        let mut flags = FLAG_ON;
        if m.main.texture.is_some() {
            flags |= FLAG_MAIN;
        }
        let none = EffectTexture::default();
        let mask = m.mask.as_ref().filter(|t| t.texture.is_some());
        if mask.is_some() {
            flags |= FLAG_MASK;
        }
        let noise = m.erosion.as_ref().map(|e| &e.noise);
        if let Some(e) = &m.erosion {
            flags |= FLAG_EROSION;
            if e.noise.texture.is_some() {
                flags |= FLAG_NOISE;
            }
            g.erosion[0] = e.softness;
            g.erosion[1] = e.edge;
            g.edge_color = e.edge_color;
        }
        let flow = m.flow.as_ref().filter(|f| f.map.texture.is_some());
        if let Some(f) = flow {
            flags |= FLAG_FLOW;
            g.erosion[2] = f.strength;
            g.erosion[3] = f.speed;
        }
        if let Some(f) = m.fresnel {
            flags |= FLAG_FRESNEL;
            g.fresnel = [f.power, f.strength, f32::from(u8::from(f.invert)), 0.0];
        }
        if let Some(r) = &m.ramp {
            flags |= FLAG_RAMP;
            let n = r.keys.len().min(MAX_CURVE_KEYS);
            g.counts[0] = n as f32;
            for (i, (t, c)) in r.keys.iter().take(n).enumerate() {
                g.ramp_t[i / 4][i % 4] = *t;
                g.ramp_v[i] = *c;
            }
        }
        let textures = [
            &m.main,
            mask.unwrap_or(&none),
            noise.unwrap_or(&none),
            flow.map_or(&none, |f| &f.map),
        ];
        for (k, t) in textures.iter().enumerate() {
            let [a, b] = uv_gpu(&t.uv);
            g.uv[k * 2] = a;
            g.uv[k * 2 + 1] = b;
        }
        g.flags = [
            flags,
            m.main.channel.index(),
            mask.map_or(0, |t| t.channel.index()),
            noise.map_or(1, |t| t.channel.index()),
        ];
        g.counts[1] = flow.map_or(0, |f| f.map.channel.index()) as f32;
        g.color = m.color;
        for (d, v) in g.drivers.iter_mut().zip(m.drivers()) {
            *d = driver_gpu(v);
        }
        g.params = m.params;
        g
    }
}

pub(crate) const EFFECT_WGSL: &str = wgsl!("effect.wgsl");

/// The effect group's layout (group 3 of the particle and strip draws):
/// the materials' uniform entries, then the main, mask, noise and flow
/// textures and a repeating sampler.
pub(crate) fn group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let texture = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("effect material"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(
                        std::mem::size_of::<EffectGpu>() as u64
                    ),
                },
                count: None,
            },
            texture(1),
            texture(2),
            texture(3),
            texture(4),
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

/// The hooks section of an effect draw module: the material hook contract
/// (`hooks.rs`'s `Hook`, the effect inputs) with `shader`'s `Params` read
/// from the effect entry's parameters, its code, and the built-in hooks it
/// leaves out (an effect runs `surface_color` and `surface_emissive`; it
/// draws no motion, so its previous parameters are the current ones).
/// Without a shader, the identities.
pub(crate) fn hooks_source(shader: Option<&crate::MaterialShader>) -> String {
    let (params, code) = shader.map_or((&[][..], ""), |s| (&s.params[..], &s.code[..]));
    crate::hooks::hook_functions_from(params, code, "effect.params", None)
}

/// The renderer's effect materials this frame: their uniform entries
/// (entry 0 is the surfaces without one) and a bind group per material
/// over its textures (white where it has none), all at the effect group's
/// layout.
pub(crate) struct EffectRenderer {
    pub layout: wgpu::BindGroupLayout,
    /// An empty group 2 for the strips drawn from vertices (`ribbon.wgsl`
    /// `vs`), whose pipelines bind the effect group as group 3.
    pub empty_layout: wgpu::BindGroupLayout,
    pub empty_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    sampler: wgpu::Sampler,
    white: wgpu::TextureView,
    /// By material index + 1 (0: none).
    groups: Vec<wgpu::BindGroup>,
}

impl EffectRenderer {
    pub(crate) fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> EffectRenderer {
        let layout = group_layout(device);
        let empty_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("empty"),
            entries: &[],
        });
        let empty_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("empty"),
            layout: &empty_layout,
            entries: &[],
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("effect material"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let white = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("effect white"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            white.as_image_copy(),
            &[255; 4],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        let white = white.create_view(&Default::default());
        let buffer = Self::buffer(device, EFFECT_STRIDE);
        queue.write_buffer(&buffer, 0, bytemuck::bytes_of(&EffectGpu::off()));
        let mut r = EffectRenderer {
            layout,
            empty_layout,
            empty_group,
            buffer,
            sampler,
            white,
            groups: Vec::new(),
        };
        let none = r.group(device, [None; 4]);
        r.groups.push(none);
        r
    }

    fn buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("effect materials"),
            size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn group(
        &self,
        device: &wgpu::Device,
        views: [Option<&wgpu::TextureView>; 4],
    ) -> wgpu::BindGroup {
        let view = |i: usize| wgpu::BindingResource::TextureView(views[i].unwrap_or(&self.white));
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("effect material"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<EffectGpu>() as u64),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: view(0),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: view(1),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: view(2),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: view(3),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }

    /// Writes the scene's materials and makes their groups, checking
    /// every texture and hook they name. `view` is a texture's view (`None`:
    /// not a texture of the scene).
    pub(crate) fn prepare<'v>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        materials: &[EffectMaterial],
        shaders: &[crate::MaterialShader],
        view: impl Fn(TextureHandle) -> Option<&'v wgpu::TextureView>,
    ) -> Result<(), crate::RenderError> {
        self.groups.truncate(1);
        if materials.is_empty() {
            return Ok(());
        }
        let size = EFFECT_STRIDE * (materials.len() as u64 + 1);
        if size > self.buffer.size() {
            self.buffer = Self::buffer(device, size.next_power_of_two());
            let none = self.group(device, [None; 4]);
            self.groups[0] = none;
        }
        let mut bytes = vec![0u8; size as usize];
        for (i, m) in materials.iter().enumerate() {
            let bad = |why: String| {
                crate::RenderError::InvalidData(format!("effect material {i}: {why}"))
            };
            m.validate().map_err(bad)?;
            if let Some(s) = m.shader {
                match shaders.get(s.index()) {
                    Some(sh) if sh.mode == crate::ShaderMode::Hooks => {}
                    Some(_) => return Err(bad("its shader is a full shader, not hooks".into())),
                    None => {
                        return Err(crate::RenderError::InvalidHandle(format!(
                            "effect material {i}: shader {}",
                            s.index()
                        )));
                    }
                }
            }
            let at = (i + 1) * EFFECT_STRIDE as usize;
            let g = EffectGpu::of(m);
            bytes[at..at + std::mem::size_of::<EffectGpu>()]
                .copy_from_slice(bytemuck::bytes_of(&g));
            let mut views = [None; 4];
            let textures = [
                m.main.texture,
                m.mask.and_then(|t| t.texture),
                m.erosion.as_ref().and_then(|e| e.noise.texture),
                m.flow.and_then(|f| f.map.texture),
            ];
            for (slot, t) in views.iter_mut().zip(textures) {
                if let Some(t) = t {
                    *slot = Some(view(t).ok_or_else(|| {
                        crate::RenderError::InvalidHandle(format!(
                            "effect material {i}: texture {}",
                            t.index()
                        ))
                    })?);
                }
            }
            let group = self.group(device, views);
            self.groups.push(group);
        }
        queue.write_buffer(&self.buffer, 0, &bytes);
        Ok(())
    }

    /// The group and dynamic offset a surface with `material` binds.
    pub(crate) fn bind(&self, material: Option<EffectMaterialHandle>) -> (&wgpu::BindGroup, u32) {
        let i = material.map_or(0, |m| m.index() + 1);
        match self.groups.get(i) {
            Some(g) => (g, (i as u64 * EFFECT_STRIDE) as u32),
            None => (&self.groups[0], 0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::particles::{ColorCurve, Curve};

    #[test]
    fn values_follow_their_generator() {
        assert_eq!(EffectValue::Constant(2.0).eval(0.7, 0.3), 2.0);
        assert_eq!(EffectValue::Random([1.0, 3.0]).eval(0.7, 0.25), 1.5);
        let c = EffectValue::Curve(Curve::new(&[(0.0, 0.0), (0.5, 1.0), (1.0, 0.0)]));
        assert!((c.eval(0.25, 0.9) - 0.5).abs() < 1e-6);
        assert!((c.eval(0.75, 0.9) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn a_material_packs_what_it_uses() {
        let m = EffectMaterial {
            erosion: Some(Erosion {
                amount: EffectValue::Curve(Curve::linear(0.0, 1.0)),
                ..Default::default()
            }),
            ramp: Some(ColorCurve::new(&[(0.0, [0.0; 4]), (1.0, [1.0; 4])])),
            ..Default::default()
        };
        let g = EffectGpu::of(&m);
        assert_eq!(g.flags[0], FLAG_ON | FLAG_EROSION | FLAG_RAMP);
        // The erosion's driver is the third: a two-key curve.
        assert_eq!(g.drivers[2].head, [2.0, 0.0, 0.0, 2.0]);
        assert_eq!(g.drivers[2].v[0][1], 1.0);
        // Intensity and alpha default to 1.
        assert_eq!(g.drivers[3].head[1], 1.0);
        assert_eq!(g.drivers[4].head[1], 1.0);
        assert_eq!(EffectGpu::off().flags[0], 0);
    }

    #[test]
    fn bad_values_are_refused() {
        let m = EffectMaterial {
            intensity: EffectValue::Curve(Curve::new(&[])),
            ..Default::default()
        };
        assert!(m.validate().unwrap_err().contains("intensity"));
        let m = EffectMaterial {
            soft_depth: -1.0,
            ..Default::default()
        };
        assert!(m.validate().is_err());
    }
}
