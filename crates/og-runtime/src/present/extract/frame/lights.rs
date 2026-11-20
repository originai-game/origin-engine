//! The frame's lights.

use super::*;

/// The lights and their cookies.
pub(super) fn lights(world: &World, rs: &mut RenderState, h: &mut Hierarchy) {
    let scene = &mut rs.scene;
    // Lights: the directional light og_render::key_light draws of them all
    // (the brightest, two handing over), then point and spot lights in
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
    if let Some(dl) = &layouts.directional {
        let mut found: Vec<(u64, og_render::DirectionalLight)> = Vec::new();
        for (e, v) in rows(world, dl) {
            let Ok(v) = crate::host::wire::record::DirectionalLight::from_json(&v) else {
                continue;
            };
            let m = h.world_of(e);
            let mut light = scene.lighting.light;
            let dir = forward(m);
            light.direction_to_light = if dir == Vec3::ZERO { Vec3::Y } else { -dir };
            light.color = Vec3::new(v.color.r, v.color.g, v.color.b);
            light.intensity = v.intensity;
            light.cast_shadows = v.cast_shadow;
            light.max_shadow_distance = v.shadow_distance;
            light.shadow_box = v.shadow_box;
            light.shadow_focus = v.shadow_focus;
            light.shadow_focus_drop = v.shadow_focus_drop;
            light.shadow_bias = v.shadow_bias;
            light.shadow_normal_bias = v.shadow_normal_bias;
            light.shadow_softness = v.shadow_softness;
            light.cascade_blend = v.cascade_blend;
            found.push((e, light));
        }
        // Equal lights go to the one spawned first.
        og_core::sort::sort_by_key(&mut found, |(e, _)| Entity::from_bits(*e).index());
        let lights: Vec<_> = found.iter().map(|(_, l)| *l).collect();
        if let Some(key) = og_render::key_light(&lights) {
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
                    .map(|&i| Entity::from_bits(found[i].0).to_string())
                    .collect();
                rs.notes.push(format!(
                    "og.DirectionalLight: one of the scene's {} directional lights is drawn, the brightest (two within 1.5x of each other blend as they hand over), so the light of {} is not; fade the others out while one shines, or light with og.PointLight, og.SpotLight or og.RectLight",
                    found.len(),
                    names.join(", "),
                ));
            }
        }
    }
    // Each light with its cookie (PointLight.cookie, SpotLight.cookie).
    let textures = &rs.textures;
    let cookie = |h: u64| super::assets::index_of(h as f64).and_then(|i| textures.get(i).copied());
    let rgb = |c: &crate::host::wire::Color| Vec3::new(c.r, c.g, c.b);
    let mut locals: Vec<(u64, og_render::Light, Option<og_render::TextureHandle>)> = Vec::new();
    if let Some(pl) = &layouts.point {
        for (e, v) in rows(world, pl) {
            let Ok(v) = crate::host::wire::record::PointLight::from_json(&v) else {
                continue;
            };
            let m = h.world_of(e);
            let c = cookie(v.cookie);
            locals.push((
                e,
                og_render::Light {
                    kind: og_render::LightKind::Point,
                    position: m.transform_point3(Vec3::ZERO),
                    color: rgb(&v.color),
                    intensity: v.intensity,
                    range: v.distance,
                    radius: v.radius,
                    cast_shadows: v.cast_shadow,
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
            let m = h.world_of(e);
            let c = cookie(v.cookie);
            locals.push((
                e,
                og_render::Light {
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
            let m = h.world_of(e);
            let light = super::baked::rect_light(m, &v);
            locals.push((e, light, None));
        }
    }
    // A light a cell spawned shines up with its fade (`crate::present::reveal`).
    for (e, light, _) in &mut locals {
        light.intensity *= rs.reveal.light(*e);
    }
    og_core::sort::sort_by_key(&mut locals, |(e, ..)| Entity::from_bits(*e).index());
    scene.lighting.light_cookies = locals
        .iter()
        .enumerate()
        .filter_map(|(i, (_, _, c))| c.map(|texture| og_render::LightCookie { light: i, texture }))
        .collect();
    scene.lighting.lights = locals.iter().map(|(_, l, _)| *l).collect();
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
