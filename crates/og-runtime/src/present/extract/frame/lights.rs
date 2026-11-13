//! The frame's lights.

use super::*;

/// The lights and their cookies.
pub(super) fn lights(world: &World, rs: &mut RenderState, h: &mut Hierarchy) {
    let scene = &mut rs.scene;
    // Lights: the first directional light, then point and spot lights in
    // entity order (deterministic); og-render culls them to the view and
    // keeps at most MAX_LIGHTS.
    let forward = |m: Mat4| m.transform_vector3(Vec3::NEG_Z).normalize_or_zero();
    let layouts = &rs.draw;
    // The environment's sky is the built-in one each frame until the
    // directional light (its sun) and og.Sky or og.DayNight (atmosphere.rs)
    // set it: kept from the last frame, a Sky turned off went on lighting
    // the scene and filling its background.
    scene.lighting.environment.sky = og_render::ProceduralSky::default();
    if let Some(dl) = &layouts.directional
        && let Some((e, v)) = rows(world, dl).into_iter().next()
        && let Ok(v) = crate::host::wire::record::DirectionalLight::from_json(&v)
    {
        let m = h.world_of(e);
        let light = &mut scene.lighting.light;
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
        scene.lighting.environment.sky.sun_direction = light.direction_to_light.normalize_or_zero();
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
