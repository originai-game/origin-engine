use crate::oit::{ARENA_WGSL, OIT_WGSL};
use crate::shader_capability::{Capability, assemble};
use crate::{clusters, lightmap, particles_gpu, skin};

/// What any shader may call, one copy for every module: the integer
/// hashes (hash.wgsl), then the octahedral map, the fullscreen triangle,
/// the view-space point under a pixel and interleaved gradient noise
/// (common.wgsl). [`FRAME_WGSL`] starts with it, and a module without the
/// frame group puts this first.
pub const LIBRARY_WGSL: &str = concat!(wgsl!("hash.wgsl"), wgsl!("common.wgsl"));
/// The frame group (frame.wgsl) after the library ([`LIBRARY_WGSL`]), with the
/// light volumes' buffer and the probe GI's sampling that `ambient_light`
/// reads (gi_probes.rs).
pub const FRAME_WGSL: &str = concat!(
    wgsl!("hash.wgsl"),
    wgsl!("common.wgsl"),
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

/// The screen-space AO on lit surfaces and the local lights' contact
/// shadows (screen_ao.wgsl), after the effect group ([`fx_wgsl`]):
/// `screen_ao(p)` for material shaders, `screen_occlusion` and
/// `ambient_normal` for the lit surfaces.
pub const SCREEN_AO_WGSL: &str = wgsl!("screen_ao.wgsl");

/// What every lit surface does after its own sun term (surface_shade.wgsl):
/// `surface_ambient`, the one gather of the light around it, and
/// `surface_shade`, its sum with the caustics and the local lights. After
/// [`LIGHTING_WGSL`], [`AREA_WGSL`], the light volumes' hook
/// ([`lightmap::probe_wgsl`]) and [`SCREEN_AO_WGSL`] or its stub.
pub const SURFACE_SHADE_WGSL: &str = wgsl!("surface_shade.wgsl");

/// The lit passes' effect group (fx.wgsl) declared at bind group `group`:
/// 2 in the lit and physical modules, wherever a surface with its own
/// groups has room for it. The module also carries decals.wgsl, which
/// declares the decal records.
pub fn fx_wgsl(group: u32) -> String {
    wgsl!("fx.wgsl").replace("FX_GROUP", &group.to_string())
}

/// `planar_reflection` over the effect group at `group` (planar.wgsl).
pub fn planar_wgsl(group: u32) -> String {
    wgsl!("planar.wgsl").replace("FX_GROUP", &group.to_string())
}
/// Whether the effect group carries the planar reflection material shaders
/// read (binding 4): with two planar reflections
/// ([`RenderDesc::planar_reflections`]). With one, the effect group is the
/// four bindings it was before material shaders could read the mirror.
pub fn planar_slot(planar_reflections: u32) -> bool {
    planar_reflections > 1
}

/// The lit shader's source with `hooks` (`hooks.rs`); with `planar`
/// ([`planar_slot`]) `planar_reflection` reads the effect group's binding 4,
/// else it is its stub.
pub fn lit_source_with(hooks: &str, planar: bool) -> String {
    assemble(
        &[
            FRAME_WGSL,
            OIT_WGSL,
            ARENA_WGSL,
            &clusters::wgsl(),
            particles_gpu::NOISE_WGSL,
            MATERIAL_WGSL,
            MATERIAL_MAPS_WGSL,
            MATERIAL_DISSOLVE_WGSL,
            &skin::palette_wgsl(3),
            hooks,
            LIGHTING_WGSL,
            WEATHER_SURFACE_WGSL,
            &lightmap::probe_wgsl(),
            AREA_WGSL,
            INSTANCE_WGSL,
            &fx_wgsl(2),
            PBR_WGSL,
            &if planar {
                planar_wgsl(2)
            } else {
                String::new()
            },
            SCREEN_AO_WGSL,
            SURFACE_SHADE_WGSL,
            PBR_FS_WGSL,
            PBR_PREPASS_WGSL,
            PBR_LEAN_WGSL,
            LIGHTMAP_PAGES_WGSL,
            LIGHTMAP_WGSL,
        ],
        &Capability::HOOKS_CALL,
    )
}

/// The physical lit shader's source (`physical.rs`): [`lit_source_with`]
/// with the physical material group and fragment shader.
pub fn physical_source(hooks: &str) -> String {
    assemble(
        &[
            FRAME_WGSL,
            OIT_WGSL,
            ARENA_WGSL,
            &clusters::wgsl(),
            particles_gpu::NOISE_WGSL,
            MATERIAL_WGSL,
            MATERIAL_PHYSICAL_WGSL,
            MATERIAL_DISSOLVE_WGSL,
            &skin::palette_wgsl(3),
            hooks,
            LIGHTING_WGSL,
            WEATHER_SURFACE_WGSL,
            &lightmap::probe_wgsl(),
            AREA_WGSL,
            INSTANCE_WGSL,
            &fx_wgsl(2),
            PBR_WGSL,
            SCREEN_AO_WGSL,
            SURFACE_SHADE_WGSL,
            SHEEN_WGSL,
            PHYSICAL_WGSL,
            LIGHTMAP_PAGES_WGSL,
            LIGHTMAP_PHYSICAL_WGSL,
        ],
        &Capability::HOOKS_CALL,
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
    assemble(
        &[
            LIBRARY_WGSL,
            particles_gpu::NOISE_WGSL,
            MATERIAL_WGSL,
            MATERIAL_MAPS_WGSL,
            MATERIAL_DISSOLVE_WGSL,
            &skin::palette_wgsl(2),
            hooks,
            SHADOW_PASS_WGSL,
            INSTANCE_WGSL,
            SHADOW_WGSL,
        ],
        &Capability::HOOKS_CALL,
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
