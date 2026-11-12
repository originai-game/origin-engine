use crate::oit::{ARENA_WGSL, OIT_WGSL};
use crate::{clusters, lightmap, particles_gpu, skin};

/// The frame group (frame.wgsl), with the light volumes' buffer and the
/// probe GI's sampling that `ambient_light` reads (gi_probes.rs).
pub const FRAME_WGSL: &str = concat!(
    wgsl!("frame.wgsl"),
    wgsl!("gi_common.wgsl"),
    wgsl!("gi_sample.wgsl"),
    wgsl!("gi_probes.wgsl")
);
/// The lit shader's vertex stages, lights and decals; the standard
/// fragment shader and the prepass's follow it (`pbr_fs.wgsl`,
/// `pbr_prepass.wgsl`), the physical one replaces both (`physical.wgsl`).
const PBR_WGSL: &str = concat!(wgsl!("pbr.wgsl"), wgsl!("decals.wgsl"));
const PBR_FS_WGSL: &str = wgsl!("pbr_fs.wgsl");
const PBR_PREPASS_WGSL: &str = wgsl!("pbr_prepass.wgsl");
/// The lean entry points plain materials draw through ([`GpuMaterial::lean`]).
const PBR_LEAN_WGSL: &str = wgsl!("pbr_lean.wgsl");
const PHYSICAL_WGSL: &str = wgsl!("physical.wgsl");
use crate::ibl::SHEEN_WGSL;
/// `instance_matrix`: an instance's transform from its columns in the
/// instance buffers ([`instances::GpuTransform`]); `mul_point`: how the
/// points it moves are projected.
pub const INSTANCE_WGSL: &str = wgsl!("instance.wgsl");
/// The GGX specular visibility, one function for direct light
/// ([`LIGHTING_WGSL`]) and for the BRDF table of the image-based light
/// (og-render's `ibl::ibl_source`).
pub const GGX_WGSL: &str = wgsl!("ggx.wgsl");
/// The BRDF and the shadow lookups the lit passes share (pbr, terrain, water),
/// then the water's caustics on what they light (caustics.wgsl).
pub const LIGHTING_WGSL: &str = concat!(
    wgsl!("ggx.wgsl"),
    wgsl!("lighting.wgsl"),
    wgsl!("caustics.wgsl")
);
/// Rain, puddles and snow on lit surfaces (weather.rs), after
/// lighting.wgsl, in the standard, physical and terrain lit modules.
pub const WEATHER_SURFACE_WGSL: &str = wgsl!("weather_surface.wgsl");

/// Rect lights and cookies (area.rs), after lighting.wgsl.
pub const AREA_WGSL: &str = wgsl!("area.wgsl");
/// The lightmap pages at group 3 and their lookup (lightmap.rs), in the
/// standard and physical lit modules.
const LIGHTMAP_PAGES_WGSL: &str = wgsl!("lightmap_pages.wgsl");
/// The lightmapped entry points (lightmap.rs), after the standard lit shader.
const LIGHTMAP_WGSL: &str = wgsl!("lightmap.wgsl");
/// The physical lightmapped entry points, after the physical lit shader.
const LIGHTMAP_PHYSICAL_WGSL: &str = wgsl!("lightmap_physical.wgsl");

/// `planar_reflection` for material shaders: the frame's planar reflection
/// (planar.wgsl, the effect group's binding 4) where the lit pipelines have
/// a texture slot for it, else [`PLANAR_STUB`].
const PLANAR_WGSL: &str = wgsl!("planar.wgsl");
/// `planar_reflection` reading nothing: the shadow passes, the scattered
/// copies, the physical shader, and every lit shader of a renderer made for
/// one planar reflection or on a device whose skinned lit pipeline has no
/// texture slot left (`planar_slot`).
pub const PLANAR_STUB: &str = "fn planar_reflection(p: vec3<f32>, shift: vec2<f32>) -> vec4<f32> { return vec4<f32>(0.0); }\n";
/// `screen_ao(p)` for material shaders in the lit and physical shaders: this
/// frame's screen-space AO (the effect group's `ssao_tex`) where world point
/// `p` lands on the screen, 1 without AO. An unlit material is not darkened
/// by the AO on its own; three.js's post AO (FogShader) darkened every pixel,
/// unlit water included, and a shader can now do the same.
const SCREEN_AO_WGSL: &str = "fn screen_ao(p: vec3<f32>) -> f32 {
  if (fx.ao.x < 0.5 || oit_pass) { return 1.0; }
  let c = fx.view_proj_unjittered * vec4<f32>(p, 1.0);
  let ndc = c.xy / max(c.w, 1e-4);
  return textureSampleLevel(ssao_tex, fx_sampler, vec2<f32>(0.5 + 0.5 * ndc.x, 0.5 - 0.5 * ndc.y), 0.0).r;
}
// A local light's screen-space contact shadow at world point `p`
// (contact.wgsl `cs_contact`, its mask in the effect group's AO texture's
// g, b or a): a contact light's shadow slot carries a quarter, a half or
// three quarters past its layer (render.rs). 1 for every other light, in a
// reflection probe's face (no clusters: the mask is the camera's) and
// where the AO texture is the white placeholder (views, reflections), and
// for a blended surface in the order-independent passes (oit.wgsl
// `oit_pass`), which the opaque surface's mask behind it would darken.
fn contact_shadow(light: LocalLight, p: vec3<f32>) -> f32 {
  let w = light.direction_slot.w;
  let c = i32(round(fract(max(w, 0.0)) * 4.0)) - 1;
  if (c < 0 || frame.cluster.x <= 0.0 || oit_pass) { return 1.0; }
  let clip = fx.view_proj_unjittered * vec4<f32>(p, 1.0);
  let ndc = clip.xy / max(clip.w, 1e-4);
  let m = textureSampleLevel(ssao_tex, fx_sampler, vec2<f32>(0.5 + 0.5 * ndc.x, 0.5 - 0.5 * ndc.y), 0.0);
  return select(select(m.a, m.b, c == 1), m.g, c == 0);
}
";
/// `screen_ao` where there is no AO texture (shadow passes, scattered copies).
pub const SCREEN_AO_STUB: &str = "fn screen_ao(p: vec3<f32>) -> f32 { return 1.0; }\n";

/// Whether the effect group carries the planar reflection material shaders
/// read (binding 4): with two planar reflections
/// ([`RenderDesc::planar_reflections`]). With one, the effect group is the
/// four bindings it was before material shaders could read the mirror.
pub fn planar_slot(planar_reflections: u32) -> bool {
    planar_reflections > 1
}

/// The lit shader's source with `hooks` (`hooks.rs`); with `planar`
/// ([`planar_slot`]) `planar_reflection` reads the effect group's binding 4,
/// else it is [`PLANAR_STUB`].
pub fn lit_source_with(hooks: &str, planar: bool) -> String {
    format!(
        "{FRAME_WGSL}{OIT_WGSL}{ARENA_WGSL}{}{}{MATERIAL_WGSL}{MATERIAL_MAPS_WGSL}{MATERIAL_DISSOLVE_WGSL}{}{hooks}{LIGHTING_WGSL}{WEATHER_SURFACE_WGSL}{}{AREA_WGSL}{INSTANCE_WGSL}{PBR_WGSL}{}{SCREEN_AO_WGSL}{PBR_FS_WGSL}{PBR_PREPASS_WGSL}{PBR_LEAN_WGSL}{LIGHTMAP_PAGES_WGSL}{LIGHTMAP_WGSL}",
        clusters::wgsl(),
        particles_gpu::NOISE_WGSL,
        skin::palette_wgsl(3),
        lightmap::probe_wgsl(),
        if planar { PLANAR_WGSL } else { PLANAR_STUB },
    )
}

/// The physical lit shader's source (`physical.rs`): [`lit_source_with`]
/// with the physical material group and fragment shader.
pub fn physical_source(hooks: &str) -> String {
    format!(
        "{FRAME_WGSL}{OIT_WGSL}{ARENA_WGSL}{}{}{MATERIAL_WGSL}{MATERIAL_PHYSICAL_WGSL}{MATERIAL_DISSOLVE_WGSL}{}{hooks}{LIGHTING_WGSL}{WEATHER_SURFACE_WGSL}{}{AREA_WGSL}{INSTANCE_WGSL}{PBR_WGSL}{PLANAR_STUB}{SCREEN_AO_WGSL}{SHEEN_WGSL}{PHYSICAL_WGSL}{LIGHTMAP_PAGES_WGSL}{LIGHTMAP_PHYSICAL_WGSL}",
        clusters::wgsl(),
        particles_gpu::NOISE_WGSL,
        skin::palette_wgsl(3),
        lightmap::probe_wgsl(),
    )
}

/// [`lit_source_with`] with the planar reflection (two planar reflections,
/// [`planar_slot`]): the cook validates a material shader against the
/// module that reads it.
pub fn lit_source(hooks: &str) -> String {
    lit_source_with(hooks, planar_slot(MAX_PLANAR_REFLECTIONS))
}
pub const SHADOW_WGSL: &str = wgsl!("shadow.wgsl");
/// The shadow pass's slot and the paraboloid projection every caster shares.
pub const SHADOW_PASS_WGSL: &str = wgsl!("shadow_pass.wgsl");

/// The shadow shader's source with `hooks` (a material shader's, or the
/// built-in identities).
pub fn shadow_source(hooks: &str) -> String {
    format!(
        "{}{MATERIAL_WGSL}{MATERIAL_MAPS_WGSL}{MATERIAL_DISSOLVE_WGSL}{}{hooks}{PLANAR_STUB}{SCREEN_AO_STUB}{SHADOW_PASS_WGSL}{INSTANCE_WGSL}{SHADOW_WGSL}",
        particles_gpu::NOISE_WGSL,
        skin::palette_wgsl(2)
    )
}

pub const SPRITE_WGSL: &str = wgsl!("sprite.wgsl");
/// The material uniform; the standard group's textures and the dissolve
/// follow it (`material_maps.wgsl`, `material_dissolve.wgsl`), the
/// physical group's replace the first (`material_physical.wgsl`).
pub const MATERIAL_WGSL: &str = wgsl!("material.wgsl");
pub const MATERIAL_MAPS_WGSL: &str = wgsl!("material_maps.wgsl");
pub const MATERIAL_DISSOLVE_WGSL: &str = wgsl!("material_dissolve.wgsl");
const MATERIAL_PHYSICAL_WGSL: &str = wgsl!("material_physical.wgsl");

/// The sprite pass's module: the frame, OIT's fragment code (alpha
/// billboards join the main camera's order-independent passes, oit.rs),
/// the lighting and the sprites.
pub fn sprite_source() -> String {
    format!("{FRAME_WGSL}{OIT_WGSL}{ARENA_WGSL}{LIGHTING_WGSL}{SPRITE_WGSL}")
}

/// The most planar reflections one frame draws
/// ([`RenderDesc::planar_reflections`]).
pub const MAX_PLANAR_REFLECTIONS: u32 = 2;
