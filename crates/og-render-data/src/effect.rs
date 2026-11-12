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
    pub fn index(self) -> u32 {
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

/// Fading by the angle between the surface and the eye. A rim (`invert`
/// false), `(1 - |n·v|)^power`, keeps the edges and clears the middle (a
/// shield, a bubble); inverted, `|n·v|^power`, the silhouette fades out
/// well inside the limb, so no edge of the mesh shows (soft spheres, glows,
/// columns, smoke shells); a higher power fades further in.
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
    /// A flash: a radial glow with no silhouette when drawn on a sphere,
    /// `|n·v|^3` (about `(1 − r²)^1.5` from its centre out, a tenth of the
    /// centre by 0.8 of the radius), additive, faded where it meets the
    /// scene so it never covers what it lights, and dying out fast over its
    /// life (half gone by a fifth of it, a tenth left by three fifths).
    pub fn glow() -> Self {
        EffectMaterial {
            fresnel: Some(Fresnel {
                power: 3.0,
                strength: 1.0,
                invert: true,
            }),
            intensity: EffectValue::Curve(Curve::new(&[
                (0.0, 1.0),
                (0.2, 0.45),
                (0.6, 0.1),
                (1.0, 0.0),
            ])),
            soft_depth: 0.4,
            blend: Some(ParticleBlend::Additive),
            ..Default::default()
        }
    }

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
    pub fn drivers(&self) -> [&EffectValue; EFFECT_DRIVERS] {
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
pub struct EffectMaterialHandle(pub u32);

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

/// An effect ring with its own shape (a `type vfx` mesh element's `mesh
/// ring` with `inner`, `band`, `height`, `arc`, `segments` or `colors`): an
/// arc of a ring about +z, outer radius 1 (flat in the xy plane facing +z
/// unless its edges are raised along +z: a cone, a tube), three rows of
/// vertices round it (the outer edge, a middle band and the inner edge),
/// each with its own colour, u along the arc and v 0 at the outer edge, 0.5
/// at the band, 1 at the inner (Effekseer's ring, `RingRendererBase.h`, MIT,
/// `third_party/effekseer/`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EffectRing {
    /// The inner radius (0 to 16; past 1 the inner edge lies outside).
    pub inner: f32,
    /// Where the middle band lies between the inner (0) and outer (1) edge.
    pub band: f32,
    /// The outer and the inner edge's height along +z.
    pub height: [f32; 2],
    /// Where the arc starts, degrees counter-clockwise from +x.
    pub start: f32,
    /// The arc's length in degrees (360 a whole ring).
    pub length: f32,
    /// Segments round the arc, 3 to 256.
    pub segments: u32,
    /// Linear colours at the outer edge, the band and the inner edge.
    pub colors: [[f32; 4]; 3],
}

impl Default for EffectRing {
    fn default() -> Self {
        EffectRing {
            inner: 0.5,
            band: 0.5,
            height: [0.0; 2],
            start: 0.0,
            length: 360.0,
            segments: 64,
            colors: [[1.0; 4]; 3],
        }
    }
}

/// The mesh of an [`EffectRing`].
pub fn effect_ring(r: &EffectRing) -> crate::MeshData {
    let segments = r.segments.clamp(3, 256);
    let inner = r.inner.clamp(0.0, 16.0);
    let band = r.band.clamp(0.0, 1.0);
    let [ho, hi] = r.height;
    let rows = [
        (1.0, ho, 0.0, r.colors[0]),
        (
            inner + (1.0 - inner) * band,
            hi + (ho - hi) * band,
            0.5,
            r.colors[1],
        ),
        (inner, hi, 1.0, r.colors[2]),
    ];
    let mut vertices = Vec::with_capacity((segments as usize + 1) * 3);
    for i in 0..=segments {
        let u = i as f32 / segments as f32;
        let a = (r.start + r.length * u).to_radians();
        let (s, c) = og_core::math::sin_cos(a);
        for (radius, z, v, color) in rows {
            vertices.push(crate::Vertex {
                position: [c * radius, s * radius, z],
                normal: [0.0, 0.0, 1.0],
                uv: [u, v],
                color,
                tangent: [0.0; 4],
            });
        }
    }
    let mut indices = Vec::with_capacity(segments as usize * 12);
    for i in 0..segments {
        for row in 0..2 {
            let a = i * 3 + row;
            let b = a + 3;
            indices.extend_from_slice(&[a, b, b + 1, a, b + 1, a + 1]);
        }
    }
    crate::MeshData { vertices, indices }
}

/// An effect column or tube: an open cylinder of radius 0.5 from y −0.5 to
/// 0.5 (u round it, v up it) whose vertex alpha falls to 0 at both open
/// ends (smoothly over the bottom quarter and the top third), so a column
/// of noise drawn on it has no square end or rim.
pub fn effect_cylinder() -> crate::MeshData {
    use crate::geometry::primitives as p;
    let mut m = p::cylinder(&p::Cylinder {
        radius_top: 0.5,
        radius_bottom: 0.5,
        height_segments: 12,
        open_ended: true,
        ..Default::default()
    });
    let s = |a: f32, b: f32, x: f32| {
        let u = ((x - a) / (b - a)).clamp(0.0, 1.0);
        u * u * (3.0 - 2.0 * u)
    };
    for v in &mut m.vertices {
        let t = v.position[1] + 0.5;
        v.color[3] = s(0.0, 0.25, t) * (1.0 - s(0.65, 1.0, t));
    }
    m
}
