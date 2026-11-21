//! The frame's lights.

use super::*;

/// The lights and their cookies.
pub(super) fn lights(world: &World, rs: &mut RenderState, h: &mut Hierarchy) {
    let scene = &mut rs.scene;
    // Lights: the key light (crate::present::key_light: the brightest of
    // the directional lights and og.DayNight's sun and moon, two handing
    // over), then point and spot lights in
    // entity order (deterministic); og-render culls them to the view and
    // keeps at most MAX_LIGHTS.
    let forward = |m: Mat4| {
        m.transform_vector3(og_core::facing::CAMERA_FORWARD)
            .normalize_or_zero()
    };
    let layouts = &rs.draw;
    // The environment's sky is the built-in one each frame until the
    // directional light (its sun) and og.Sky or og.DayNight (atmosphere.rs)
    // set it: kept from the last frame, a Sky turned off went on lighting
    // the scene and filling its background.
    scene.lighting.environment.sky = og_render::ProceduralSky::default();
    let stage = crate::present::key_light::Layouts {
        directional: layouts.directional.as_ref(),
        day_night: layouts.atmo.day_night(),
        physical_sky: layouts.atmo.physical_sky(),
        season: layouts.atmo.season(),
    };
    if let Some(key) = crate::present::key_light::key(
        world,
        stage,
        &mut |e| h.world_of(e),
        scene.lighting.light,
        &rs.baked,
    ) {
        scene.lighting.light = key.light;
        scene.lighting.environment.sky.sun_direction =
            key.light.direction_to_light.normalize_or_zero();
        // One directional light is drawn: the log names the lights
        // whose light is lost (a development build's warning).
        #[cfg(not(feature = "publish"))]
        if !key.left_out.is_empty() {
            let names: Vec<String> = key
                .left_out
                .iter()
                .map(|&e| Entity::from_bits(e).to_string())
                .collect();
            rs.notes.push(format!(
                "og.DirectionalLight: one of the scene's {} directional lights is drawn, the brightest (two within 1.5x of each other blend as they hand over), so the light of {} is not; fade the others out while one shines, or light with og.PointLight, og.SpotLight or og.RectLight",
                key.lights,
                names.join(", "),
            ));
        }
    }
    // Each light with its cookie (PointLight.cookie, SpotLight.cookie).
    let textures = &rs.textures;
    let cookie = |h: u64| super::assets::index_of(h as f64).and_then(|i| textures.get(i).copied());
    let rgb = |c: &crate::host::wire::Color| Vec3::new(c.r, c.g, c.b);
    // Each with its cookie; a static light whose light its scene's drawn
    // bake holds is left out (LightMobility).
    let baked = &rs.baked;
    let mobility = crate::present::baked::mobility;
    let mut locals: Vec<(u64, og_render::LightDesc, Option<og_render::TextureHandle>)> = Vec::new();
    if let Some(pl) = &layouts.point {
        for (e, v) in rows(world, pl) {
            let Ok(v) = crate::host::wire::record::PointLight::from_json(&v) else {
                continue;
            };
            if baked.baked_light(e) {
                continue;
            }
            let m = h.world_of(e);
            let c = cookie(v.cookie);
            locals.push((
                e,
                og_render::LightDesc {
                    kind: og_render::LightKind::Point,
                    position: m.transform_point3(Vec3::ZERO),
                    color: rgb(&v.color),
                    intensity: v.intensity,
                    range: v.distance,
                    radius: v.radius,
                    cast_shadows: v.cast_shadow,
                    mobility: mobility(v.mobility),
                },
                c,
            ));
        }
    }
    if let Some(sl) = &layouts.spot {
        for (e, v) in rows(world, sl) {
            let Ok(v) = crate::host::wire::record::SpotLight::from_json(&v) else {
                continue;
            };
            if baked.baked_light(e) {
                continue;
            }
            let m = h.world_of(e);
            let c = cookie(v.cookie);
            locals.push((
                e,
                og_render::LightDesc {
                    kind: og_render::LightKind::Spot {
                        direction: forward(m),
                        angle_degrees: v.angle.to_degrees(),
                        penumbra: v.penumbra,
                    },
                    position: m.transform_point3(Vec3::ZERO),
                    color: rgb(&v.color),
                    intensity: v.intensity,
                    range: v.distance,
                    radius: v.radius,
                    cast_shadows: v.cast_shadow,
                    mobility: mobility(v.mobility),
                },
                c,
            ));
        }
    }
    if let Some(rl) = &layouts.rect {
        for (e, v) in rows(world, rl) {
            let Ok(v) = crate::host::wire::record::RectLight::from_json(&v) else {
                continue;
            };
            if baked.baked_light(e) {
                continue;
            }
            let m = h.world_of(e);
            let light = crate::present::baked::rect_light(m, &v);
            locals.push((e, light, None));
        }
    }
    // A light a cell spawned shines up with its fade (`crate::present::reveal`).
    for (e, light, ..) in &mut locals {
        light.intensity *= rs.reveal.light(*e);
    }
    og_core::sort::sort_by_key(&mut locals, |(e, ..)| Entity::from_bits(*e).index());
    scene.lighting.light_cookies = locals
        .iter()
        .enumerate()
        .filter_map(|(i, (_, _, c))| c.map(|texture| og_render::LightCookie { light: i, texture }))
        .collect();
    scene.lighting.lights = locals.iter().map(|(_, l, ..)| *l).collect();
    // Cookies past RenderCapabilities.maxLightCookies layers are not drawn:
    // the log says which lights shine without theirs.
    let cookies = og_render::cookie_layers(scene);
    if !cookies.left_out.is_empty() {
        let lights: Vec<String> = cookies
            .left_out
            .iter()
            .map(|&i| Entity::from_bits(locals[i].0).to_string())
            .collect();
        rs.notes.push(format!(
            "light cookies: the frame's cookies take {} layers, past RenderCapabilities.maxLightCookies ({}; a SpotLight.cookie takes 1, a PointLight.cookie 6, a texture shared by lights of one kind once); {}: {}",
            cookies.asked,
            og_render::MAX_COOKIE_LAYERS,
            if lights.len() == 1 {
                "this light shines without its cookie"
            } else {
                "these lights shine without theirs"
            },
            lights.join(", ")
        ));
    }
}
