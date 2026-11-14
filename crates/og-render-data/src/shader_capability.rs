//! What a surface's WGSL module carries of the renderer's shared shading,
//! read from the module's own text, and the one place a module made
//! without a capability gets the functions that stand in for it.
//!
//! A module is its parts in order ([`assemble`]). A part that defines a
//! capability's function makes the module carry it; where no part does and
//! the module's callers may still name it (a material shader's hooks call
//! `planar_reflection` and `screen_ao` wherever they run), [`assemble`]
//! adds the capability's stub, so no list of parts names a stub itself.
//! `tests/shader_capabilities.rs` (og-render) holds the table of every lit
//! module against every capability.

/// One thing a lit surface can ask of the renderer beyond the sun, the
/// ambient light and the fog every lit module carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capability {
    /// Point, spot and rect lights with their cookies (`area.wgsl`).
    LocalLights,
    /// Rain, puddles and snow on the surface (`weather_surface.wgsl`).
    Weather,
    /// Projected decals (`decals.wgsl`).
    Decals,
    /// A blended surface in the order-independent passes (`oit.wgsl`).
    Oit,
    /// The frame's planar reflection (`planar.wgsl`).
    PlanarReflection,
    /// The screen-space AO and the local lights' contact shadows.
    ScreenAo,
    /// Baked lightmap pages (`lightmap_pages.wgsl`).
    Lightmap,
    /// The one gather and sum of the light around a surface
    /// (`surface_shade.wgsl`): a surface without it keeps its own copy.
    SharedShade,
}

/// How a module carries a [`Capability`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Holds {
    /// Its code: the surface can use it.
    Full,
    /// Its stub: a call compiles and does nothing.
    Stub,
    /// Neither: a call does not compile.
    None,
}

/// `planar_reflection` reading nothing: the shadow passes, the scattered
/// copies, the physical shader, and every lit shader of a renderer made for
/// one planar reflection or on a device whose skinned lit pipeline has no
/// texture slot left (`planar_slot`).
const PLANAR_STUB: &str = "fn planar_reflection(p: vec3<f32>, shift: vec2<f32>) -> vec4<f32> { return vec4<f32>(0.0); }\n";
/// What the lit surfaces and material shaders call of
/// [`crate::shader_source::SCREEN_AO_WGSL`] where there is no AO texture
/// (the shadow passes, a surface whose pipeline has no texture slot left
/// for the effect group): no occlusion, the surface's own normal.
const SCREEN_AO_STUB: &str = "fn screen_ao(p: vec3<f32>) -> f32 { return 1.0; }
fn ambient_normal(clip: vec2<f32>, n: vec3<f32>) -> vec3<f32> { return n; }
struct ScreenOcclusion {
  ambient: f32,
  all: f32,
  bounce: vec3<f32>,
};
fn screen_occlusion(frag: vec2<f32>) -> ScreenOcclusion { return ScreenOcclusion(1.0, 1.0, vec3<f32>(0.0)); }
";
/// `oit_pass` in a module no order-independent pass draws: the screen AO
/// reads it ([`crate::shader_source::SCREEN_AO_WGSL`]).
const OIT_STUB: &str = "const oit_pass: bool = false;\n";
/// What the lit modules call of [`crate::WEATHER_SURFACE_WGSL`], each
/// returning what the full code returns with `OG_WEATHER` false: its text
/// in a module made without the weather on surfaces (og-render's
/// `RenderDesc::without`, `Features::leave_out`).
pub const WEATHER_SURFACE_STUB: &str = "const WEATHER_RUNS_OFF: f32 = 0.0;
const WEATHER_STANDS: f32 = 1.0;
const WEATHER_GROUND: f32 = 2.0;
struct Weathered {
  s: Surface,
  n: vec3<f32>,
};
fn weather_surface_at(s_in: Surface, n_in: vec3<f32>, geo: vec3<f32>, p_frame: vec3<f32>, cavity: f32, amount: f32, standing: f32) -> Weathered {
  return Weathered(s_in, n_in);
}
fn weather_surface(s_in: Surface, n_in: vec3<f32>, geo: vec3<f32>, p_frame: vec3<f32>) -> Weathered {
  return Weathered(s_in, n_in);
}
fn weather_gloss(roughness: f32, metalness: f32, n: vec3<f32>, geo: vec3<f32>, p_frame: vec3<f32>, cavity: f32, amount: f32, standing: f32) -> Weathered {
  var s: Surface;
  s.albedo = vec3<f32>(1.0);
  s.metallic = metalness;
  s.roughness = roughness;
  s.f0 = mix(vec3<f32>(0.04), s.albedo, metalness);
  return Weathered(s, n);
}
fn weather_grass(s_in: Surface, n_in: vec3<f32>, p_frame: vec3<f32>, t: f32) -> Weathered {
  return Weathered(s_in, n_in);
}
";

impl Capability {
    pub const ALL: [Capability; 8] = [
        Capability::LocalLights,
        Capability::Weather,
        Capability::Decals,
        Capability::Oit,
        Capability::PlanarReflection,
        Capability::ScreenAo,
        Capability::Lightmap,
        Capability::SharedShade,
    ];

    /// What a material shader's hooks may call in every module they run
    /// in (the lit, physical, shadow and scattered copies' modules).
    pub const HOOKS_CALL: [Capability; 2] = [Capability::PlanarReflection, Capability::ScreenAo];

    /// What a lit surface that no order-independent pass draws may name:
    /// the hooks' calls, and `oit_pass`, which the screen AO reads.
    pub const OPAQUE_SURFACE: [Capability; 3] = [
        Capability::PlanarReflection,
        Capability::ScreenAo,
        Capability::Oit,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Capability::LocalLights => "local lights",
            Capability::Weather => "weather",
            Capability::Decals => "decals",
            Capability::Oit => "oit",
            Capability::PlanarReflection => "planar reflection",
            Capability::ScreenAo => "screen ao",
            Capability::Lightmap => "lightmap",
            Capability::SharedShade => "shared shade",
        }
    }

    /// Whether a game sees it: a light, the weather, a decal, a blend, a
    /// mirror, the AO or a lightmap acts on a surface that takes it and
    /// not on one that does not (`og check` names the ones a material
    /// shader's surface lacks). The shared shading is how a surface is
    /// built, not something a game asks for.
    pub fn seen_by_a_game(self) -> bool {
        match self {
            Capability::LocalLights
            | Capability::Weather
            | Capability::Decals
            | Capability::Oit
            | Capability::PlanarReflection
            | Capability::ScreenAo
            | Capability::Lightmap => true,
            Capability::SharedShade => false,
        }
    }

    /// The function every use of it goes through: a module that defines
    /// it carries the capability, and a surface whose entry points reach
    /// it uses it.
    pub fn function(self) -> &'static str {
        match self {
            Capability::LocalLights => "light_cookie",
            Capability::Weather => "weather_surface_at",
            Capability::Decals => "apply_decals",
            Capability::Oit => "oit_weighted",
            Capability::PlanarReflection => "planar_reflection",
            Capability::ScreenAo => "screen_ao",
            Capability::Lightmap => "lightmap_texel",
            Capability::SharedShade => "surface_irradiance",
        }
    }

    /// The text that stands in for it in a module made without it; None
    /// for a capability only the surfaces that have it call.
    pub fn stub(self) -> Option<&'static str> {
        match self {
            Capability::Weather => Some(WEATHER_SURFACE_STUB),
            Capability::PlanarReflection => Some(PLANAR_STUB),
            Capability::ScreenAo => Some(SCREEN_AO_STUB),
            Capability::Oit => Some(OIT_STUB),
            _ => None,
        }
    }

    /// How `source` (a WGSL module) carries it.
    pub fn held_by(self, source: &str) -> Holds {
        if !source.contains(&format!("fn {}(", self.function())) {
            Holds::None
        } else if self.stub().is_some_and(|stub| source.contains(stub)) {
            Holds::Stub
        } else {
            Holds::Full
        }
    }
}

/// The module made of `parts` in order, then the stub of each of `closes`
/// that no part defines (WGSL resolves module-scope names in any order).
pub fn assemble(parts: &[&str], closes: &[Capability]) -> String {
    let mut source = parts.concat();
    for c in closes {
        if c.held_by(&source) == Holds::None {
            source.push_str(c.stub().expect("a capability a module closes has a stub"));
        }
    }
    source
}
