//! The E5 round's rendering from the world (`og.atmosphere` and the
//! drawing of the physics that has a look): og.Fog, og.Sky, og.DayNight
//! and og.Weather into the scene's fog, sky, light and particles; og.Water
//! (with an og.WaterSurface's heights), og.Water2D over an og.WaterLine,
//! og.Fluid2D as metaballs (a group as one surface), og.Fluid3D as a
//! screen-space surface (a group's particles coloured by their mix), visible og.Rope and
//! og.Cloth; and the material shaders (`og.load.shader`, `og.render.shaderMaterial`,
//! `og.render.setShaderParam`), a module each under `atmosphere/`. Like the rest of extraction it reads the
//! world (and the physics' last step) and never writes either.

mod fluid;
mod shader;
mod sky;
mod soft;
mod water;
mod weather;

pub(crate) use shader::load_shader;
pub use shader::{
    og_load_shader, og_render_set_material_shader, og_render_set_shader_param,
    og_render_shader_material,
};
#[cfg(not(feature = "publish"))]
pub(crate) use sky::derive;
pub use sky::og_atmosphere_time_of_day;
pub(crate) use water::{ModuleState, river_of};

use crate::host::codec::Layout;
use crate::host::wire::Color;
use crate::present::extract::{Hierarchy, RenderState};
use glam::Vec3;
use og_core::IdMap;
use og_ecs::World;
use og_render::{MaterialHandle, MeshHandle};
use std::collections::BTreeMap;

/// The layouts of the records this module reads, and the physics whose
/// last step it draws.
#[derive(Default)]
pub(crate) struct AtmoLayouts {
    fog: Option<Layout>,
    sky: Option<Layout>,
    physical_sky: Option<Layout>,
    clouds: Option<Layout>,
    aurora: Option<Layout>,
    day_night: Option<Layout>,
    weather: Option<Layout>,
    weather_state: Option<Layout>,
    wind: Option<Layout>,
    wind_motor: Option<Layout>,
    water: Option<Layout>,
    water_2d: Option<Layout>,
    water_line: Option<Layout>,
    fluid: Option<Layout>,
    fluid3d: Option<Layout>,
    rope: Option<Layout>,
    cloth: Option<Layout>,
    soft: Option<Layout>,
    pub ragdoll: Option<Layout>,
    pub physics: Option<og_physics::PhysicsSystem>,
}

impl AtmoLayouts {
    /// The air a frame draws: og.Wind and the og.WindMotors where `h`
    /// places them (`wind::frame_air`).
    pub(crate) fn air(&self, world: &World, h: &mut Hierarchy) -> og_env::air::Air {
        crate::bind::wind::frame_air(world, self.wind.as_ref(), self.wind_motor.as_ref(), h).1
    }

    pub fn from(
        get: impl Fn(&str) -> Option<Layout>,
        physics: Option<og_physics::PhysicsSystem>,
    ) -> AtmoLayouts {
        AtmoLayouts {
            fog: get("og.Fog"),
            sky: get("og.Sky"),
            physical_sky: get("og.PhysicalSky"),
            clouds: get("og.Clouds"),
            aurora: get("og.Aurora"),
            day_night: get("og.DayNight"),
            weather: get("og.Weather"),
            weather_state: get("og.WeatherState"),
            wind: get("og.Wind"),
            wind_motor: get("og.WindMotor"),
            water: get("og.Water"),
            water_2d: get("og.Water2D"),
            water_line: get("og.WaterLine"),
            fluid: get("og.Fluid2D"),
            fluid3d: get("og.Fluid3D"),
            rope: get("og.Rope"),
            cloth: get("og.Cloth"),
            soft: get("og.SoftBody"),
            ragdoll: get("og.Ragdoll"),
            physics,
        }
    }
}

/// What this module keeps between frames: the weather's emitter, the
/// cloths' meshes, the loaded shaders and the shader materials.
#[derive(Default)]
pub(crate) struct AtmoState {
    weather: Option<(Vec<u32>, og_render::EmitterHandle)>,
    cloths: IdMap<u64, MeshHandle>,
    cloth_material: Option<MaterialHandle>,
    softs: IdMap<u64, MeshHandle>,
    soft_material: Option<MaterialHandle>,
    /// `og.load.shader` handles by load index, and loads by path.
    pub shaders: Vec<og_render::ShaderHandle>,
    pub shader_paths: BTreeMap<String, usize>,
    /// Ragdoll rigs by entity (`og.physics.setRagdollRig`), replayed into
    /// a reloaded world's physics, and this frame's palette of each posed
    /// model by entity.
    pub rigs: IdMap<u64, crate::bind::physics_ext::KeptRig>,
    pub palettes: IdMap<u64, u32>,
}

/// og.Fog, og.Sky, og.DayNight, og.Weather, og.Wind, og.Water, og.Water2D,
/// og.Fluid2D, og.Fluid3D, og.Rope and og.Cloth into `rs.scene`. Returns the keys of
/// the instances it added (cloth), for per-object motion.
pub(crate) fn extract(world: &World, rs: &mut RenderState, h: &mut Hierarchy) -> Vec<(u64, u32)> {
    let RenderState {
        scene, draw, atmo, ..
    } = rs;
    let layouts = &draw.atmo;
    let world_wind = crate::bind::wind::extract(
        world,
        scene,
        layouts.wind.as_ref(),
        layouts.wind_motor.as_ref(),
        h,
    );
    let fog_tint = sky::extract(world, scene, layouts, world_wind);
    weather::extract(world, scene, layouts, atmo, world_wind, fog_tint);

    // Ragdolls blended into their models' palettes.
    crate::bind::physics_ext::draw_ragdolls(world, rs, &mut |e| h.world_of(e));
    water::extract(world, rs, h);
    fluid::extract(world, rs, h);
    soft::extract(world, rs)
}

fn rgb(c: &Color) -> Vec3 {
    Vec3::new(c.r, c.g, c.b)
}
