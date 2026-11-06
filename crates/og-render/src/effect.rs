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

use crate::particles::{ColorCurve, Curve, MAX_CURVE_KEYS, ParticleBlend};
use crate::scene::{MeshHandle, ShaderHandle, TextureHandle};
use glam::{Mat4, Vec2};

/// The driven parameters of an effect material, in the order of
/// [`EffectGpu::drivers`]: the main UV's u and v offsets, the erosion
/// amount, the intensity, the alpha, then the eight custom numbers.
pub const EFFECT_DRIVERS: usize = 13;
/// The parameters an effect material's hook declares at most.
pub const EFFECT_PARAMS: usize = crate::MAX_SHADER_PARAMS;

/// A number of an effect material over a surface's life (three.quarks'
/// value generators; Effekseer's fixed, random and F-curve parameters).
#[derive(Clone, Debug, PartialEq)]
pub enum EffectValue {
    Constant(f32),
    /// Uniform between the two, fixed per particle (or strip, or mesh).
    Random([f32; 2]),
    /// Over the life (0 to 1): at most [`MAX_CURVE_KEYS`] keys.
    Curve(Curve),
}

impl EffectValue {
    /// The value at life `t` with the surface's random value `r` (0 to 1).
    pub fn eval(&self, t: f32, r: f32) -> f32 {
        match self {
            EffectValue::Constant(v) => *v,
            EffectValue::Random([a, b]) => a + (b - a) * r,
            EffectValue::Curve(c) => c.eval(t),
        }
    }

    fn check(&self, name: &str) -> Result<(), String> {
        match self {
            EffectValue::Constant(v) if !v.is_finite() => Err(format!("{name} {v}")),
            EffectValue::Random(r) if !r.iter().all(|v| v.is_finite()) => {
                Err(format!("{name} random {r:?}"))
            }
            EffectValue::Curve(c) if c.keys.is_empty() || c.keys.len() > MAX_CURVE_KEYS => Err(
                format!("{name} has {} keys (1 to {MAX_CURVE_KEYS})", c.keys.len()),
            ),
            _ => Ok(()),
        }
    }
}

/// Which of a texture's channels an effect reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EffectChannel {
    /// Colour and alpha as stored.
    #[default]
    Rgba,
    /// One channel as the alpha, the colour white: the greyscale masks,
    /// noises and shapes VFX textures mostly are.
    Red,
    Green,
    Blue,
    Alpha,
}

impl EffectChannel {
    fn index(self) -> u32 {
        match self {
            EffectChannel::Rgba => 0,
            EffectChannel::Red => 1,
            EffectChannel::Green => 2,
            EffectChannel::Blue => 3,
            EffectChannel::Alpha => 4,
        }
    }
}

/// How one texture of an effect material is laid over the surface: from
/// the surface's UV (0 to 1 over a quad, a mesh's own UVs, along and
/// across a strip), turned into polar coordinates first when `polar`,
/// then `uv * tile + offset + scroll * time`, turned by `rotation` about
/// the centre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EffectUv {
    pub tile: Vec2,
    pub offset: Vec2,
    /// UV units per second.
    pub scroll: Vec2,
    /// Degrees per second about (0.5, 0.5).
    pub rotation: f32,
    /// u the angle about the centre (0 to 1 a turn), v the distance from
    /// it (0 at the centre, 1 at the quad's edge midpoints).
    pub polar: bool,
}

impl Default for EffectUv {
    fn default() -> Self {
        EffectUv {
            tile: Vec2::ONE,
            offset: Vec2::ZERO,
            scroll: Vec2::ZERO,
            rotation: 0.0,
            polar: false,
        }
    }
}

/// A texture of an effect material and how it is read.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EffectTexture {
    /// `None`: the main texture is the surface's own (the emitter's, the
    /// trail's); the erosion's noise is procedural value noise; a mask or
    /// a flow map without one does nothing.
    pub texture: Option<TextureHandle>,
    pub uv: EffectUv,
    pub channel: EffectChannel,
}

/// Noise erosion (a dissolve driven over the life): the fragment is cut
/// where the noise is below `amount` (0 keeps everything, 1 cuts it all),
/// faded over `softness` above the cut, its last `edge` above that tinted
/// `edge_color` (linear; above 1 glows).
#[derive(Clone, Debug, PartialEq)]
pub struct Erosion {
    pub noise: EffectTexture,
    pub amount: EffectValue,
    pub softness: f32,
    pub edge: f32,
    pub edge_color: [f32; 4],
}

impl Default for Erosion {
    fn default() -> Self {
        Erosion {
            noise: EffectTexture {
                channel: EffectChannel::Red,
                ..Default::default()
            },
            amount: EffectValue::Constant(0.0),
            softness: 0.05,
            edge: 0.0,
            edge_color: [1.0; 4],
        }
    }
}

/// A flow map pushing the main texture: its red and green (0.5 still) as
/// a direction, `strength` UV units at most, cycling `speed` times a
/// second.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flow {
    pub map: EffectTexture,
    pub strength: f32,
    pub speed: f32,
}

/// Fading by the angle between the surface and the eye: `f = (1 - |n·v|)^power`.
/// A rim (`invert` false) keeps the edges and clears the middle (a shield,
/// a bubble); inverted, the silhouette fades (soft spheres, smoke shells).
/// `strength` blends it in (0 none, 1 full).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fresnel {
    pub power: f32,
    pub strength: f32,
    pub invert: bool,
}

/// An effect material. See the module documentation.
#[derive(Clone, Debug, PartialEq)]
pub struct EffectMaterial {
    pub main: EffectTexture,
    /// A gradient map: the main texture's value (its channel, or the
    /// luminance of rgba) picks the colour along it, its alpha times it.
    pub ramp: Option<ColorCurve>,
    pub mask: Option<EffectTexture>,
    pub erosion: Option<Erosion>,
    pub flow: Option<Flow>,
    pub fresnel: Option<Fresnel>,
    /// Fade where the surface comes within this distance of the scene
    /// behind it (particles and effect meshes; 0 off). An emitter's own
    /// `soft_distance` wins when larger.
    pub soft_depth: f32,
    /// A linear tint; above 1 glows.
    pub color: [f32; 4],
    pub intensity: EffectValue,
    pub alpha: EffectValue,
    /// Added to the main UV (after its scroll).
    pub offset: [EffectValue; 2],
    /// Custom data for the hook (`effect_custom1()`, `effect_custom2()`).
    pub custom: [[EffectValue; 4]; 2],
    /// Overrides the surface's blend.
    pub blend: Option<ParticleBlend>,
    /// Mesh particles and effect meshes draw both faces (slash arcs,
    /// open cylinders, domes seen from inside).
    pub double_sided: bool,
    /// A fragment hook: a material shader in hooks mode.
    pub shader: Option<ShaderHandle>,
    /// The hook's parameters, by its declaration order.
    pub params: [[f32; 4]; EFFECT_PARAMS],
}

impl Default for EffectMaterial {
    fn default() -> Self {
        let zero = || EffectValue::Constant(0.0);
        EffectMaterial {
            main: EffectTexture::default(),
            ramp: None,
            mask: None,
            erosion: None,
            flow: None,
            fresnel: None,
            soft_depth: 0.0,
            color: [1.0; 4],
            intensity: EffectValue::Constant(1.0),
            alpha: EffectValue::Constant(1.0),
            offset: [zero(), zero()],
            custom: std::array::from_fn(|_| std::array::from_fn(|_| zero())),
            blend: None,
            double_sided: false,
            shader: None,
            params: [[0.0; 4]; EFFECT_PARAMS],
        }
    }
}

impl EffectMaterial {
    /// Refuses a material the renderer cannot draw, naming why.
    pub fn validate(&self) -> Result<(), String> {
        self.intensity.check("intensity")?;
        self.alpha.check("alpha")?;
        for (v, name) in self.offset.iter().zip(["offset u", "offset v"]) {
            v.check(name)?;
        }
        for (i, c) in self.custom.iter().flatten().enumerate() {
            c.check(&format!(
                "custom {}.{}",
                i / 4 + 1,
                ["x", "y", "z", "w"][i % 4]
            ))?;
        }
        if let Some(e) = &self.erosion {
            e.amount.check("erosion")?;
            if !(e.softness.is_finite() && e.softness >= 0.0 && e.edge.is_finite() && e.edge >= 0.0)
            {
                return Err(format!(
                    "erosion softness {} edge {} (0 or more)",
                    e.softness, e.edge
                ));
            }
        }
        if let Some(r) = &self.ramp
            && (r.keys.is_empty() || r.keys.len() > MAX_CURVE_KEYS)
        {
            return Err(format!(
                "ramp has {} keys (1 to {MAX_CURVE_KEYS})",
                r.keys.len()
            ));
        }
        if !(self.soft_depth.is_finite() && self.soft_depth >= 0.0) {
            return Err(format!("softDepth {}", self.soft_depth));
        }
        Ok(())
    }

    /// The textures it samples (the renderer uploads and checks them).
    pub fn textures(&self) -> impl Iterator<Item = TextureHandle> + '_ {
        [
            self.main.texture,
            self.mask.and_then(|m| m.texture),
            self.erosion.as_ref().and_then(|e| e.noise.texture),
            self.flow.and_then(|f| f.map.texture),
        ]
        .into_iter()
        .flatten()
    }

    /// The driven values, in [`EFFECT_DRIVERS`] order.
    fn drivers(&self) -> [&EffectValue; EFFECT_DRIVERS] {
        let zero = &EffectValue::Constant(0.0);
        let erosion = self.erosion.as_ref().map_or(zero, |e| &e.amount);
        let c = &self.custom;
        [
            &self.offset[0],
            &self.offset[1],
            erosion,
            &self.intensity,
            &self.alpha,
            &c[0][0],
            &c[0][1],
            &c[0][2],
            &c[0][3],
            &c[1][0],
            &c[1][1],
            &c[1][2],
            &c[1][3],
        ]
    }
}

/// A handle to an effect material of a [`crate::RenderScene`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EffectMaterialHandle(pub(crate) u32);

impl EffectMaterialHandle {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// A mesh drawn with an effect material this frame (a slash arc, a
/// shockwave ring, a shield dome, a portal's disc): unlit, blended, never
/// shadowing, drawn with the particles (it is one mesh particle of its
/// own). `transform` places the mesh whole, rotation and scale included.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EffectMesh {
    /// Stable identity (the entity's id).
    pub id: u64,
    pub mesh: MeshHandle,
    pub material: EffectMaterialHandle,
    pub transform: Mat4,
    /// Its life, 0 to 1, for the material's curves.
    pub life: f32,
    /// Its random value's seed.
    pub seed: u64,
    /// A linear tint times the material's (vertex colours too).
    pub color: [f32; 4],
    /// Its blend when the material names none.
    pub blend: ParticleBlend,
    /// Three.js layers, as [`crate::RenderScene::particle_layers`].
    pub layers: u32,
}

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
/// leaves out (an effect runs `surface_color` and `surface_emissive`).
/// Without a shader, the identities.
pub(crate) fn hooks_source(shader: Option<&crate::MaterialShader>) -> String {
    let (params, code) = shader.map_or((&[][..], ""), |s| (&s.params[..], &s.code[..]));
    crate::hooks::hook_functions_from(params, code, "effect.params", "effect.params")
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
