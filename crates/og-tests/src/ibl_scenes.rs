//! The environment-map scenes: props and polished spheres lit by a
//! photographed HDR environment, baked by og-render's `bake` as the cook
//! does and drawn as the background too. The maps are Poly Haven's
//! Kloofendal Partly Cloudy (pure sky) and Spruit Sunrise, CC0, at 1024×512
//! (`assets/env/`, their origins in `assets/kit/README.md`).

use crate::kit;
use crate::scenes::asset;
use glam::{Mat4, Vec2, Vec3};
use og_render::bake::{BakeDesc, Equirect, bake};
use og_render::{
    AmbientOcclusion, AoMethod, Background, Camera, DirectionalLight, EnvironmentMap,
    EnvironmentMapData, Exposure, Material, RenderScene, shapes,
};

/// The vertical field of view of [`environment_scene`]'s camera (degrees).
pub const ENVIRONMENT_FOV: f32 = 40.0;

/// The face size the scenes bake their maps at: the background is drawn
/// from the map's sharpest level, so it is as sharp as the 40° lens needs.
const BAKED_SIZE: u32 = 256;

/// The HDRI `assets/env/<name>.hdr`: its width, height and linear RGB.
pub fn hdri_pixels(name: &str) -> Result<(u32, u32, Vec<f32>), String> {
    let rel = format!("env/{name}.hdr");
    let bytes = asset(&rel)?;
    let img = image::load_from_memory_with_format(&bytes, image::ImageFormat::Hdr)
        .map_err(|e| format!("{rel}: {e}"))?
        .to_rgb32f();
    Ok((img.width(), img.height(), img.into_raw()))
}

/// The HDRI `name` baked as og-cook bakes an environment of `size`, its
/// sun taken out into the map's `sun` with `sun`.
pub fn hdri_map(name: &str, size: u32, sun: bool) -> Result<EnvironmentMapData, String> {
    let (width, height, rgb) = hdri_pixels(name)?;
    bake(
        &Equirect {
            width,
            height,
            rgb: &rgb,
        },
        &BakeDesc {
            size,
            samples: 128,
            sun,
            ..Default::default()
        },
    )
}

/// A terrace of the kit's stone and grass below the hills, lit by nothing
/// but the environment.
fn terrace(scene: &mut RenderScene) {
    let grass = kit::surface_tinted(scene, "grass", [0.86, 1.0, 0.72]);
    let lawn = scene.add_mesh(kit::slab(
        Vec2::splat(-260.0),
        Vec2::splat(260.0),
        -0.02,
        7.0,
    ));
    scene.add_instance(lawn, grass, Mat4::IDENTITY);
    kit::hills(scene, None);
    let flags = kit::surface_tinted(scene, "cobbles", [0.95, 0.92, 0.86]);
    let floor = scene.add_mesh(kit::block(
        Vec3::new(-7.5, -0.3, -3.0),
        Vec3::new(7.5, 0.0, 3.5),
        2.0,
    ));
    scene.add_instance(floor, flags, Mat4::IDENTITY);
    // A light haze over the land, of the horizon's colour, so the hills
    // fade into the sky's distance.
    scene.atmosphere.fog = Some(og_render::Fog {
        color: Vec3::new(0.62, 0.7, 0.8),
        density: 0.0,
        height_density: 0.0012,
        height: 0.0,
        falloff: 0.02,
        max_opacity: 0.45,
        sky: 0.0,
        ..Default::default()
    });
}

/// A sculpture terrace under a partly cloudy sky: two rows of five
/// spheres on stone plinths, roughness 0 to 1 from left to right, polished
/// brass in front and white marble behind on taller plinths, set between
/// the front ones, lit by the
/// sky's map alone (no sun light), the map turned 0.6 rad and drawn behind
/// them.
pub fn environment_scene() -> Result<RenderScene, String> {
    let mut scene = RenderScene::new();
    let map = scene.add_environment_map(hdri_map("kloofendal", BAKED_SIZE, false)?);
    terrace(&mut scene);
    let stone = kit::surface_tinted(&mut scene, "stone", [0.95, 0.9, 0.84]);
    let sphere = scene.add_mesh(shapes::sphere(48, 32));
    for row in 0..2 {
        let (z, plinth, shift) = if row == 0 {
            (1.2, 0.5, 0.0)
        } else {
            (-1.6, 1.9, 1.1)
        };
        for i in 0..5 {
            let roughness = i as f32 / 4.0;
            let m = scene.add_material(if row == 0 {
                Material {
                    base_color: [1.0, 0.7, 0.36, 1.0],
                    metallic: 1.0,
                    roughness,
                    ..Default::default()
                }
            } else {
                Material {
                    base_color: [0.82, 0.8, 0.76, 1.0],
                    metallic: 0.0,
                    roughness,
                    ..Default::default()
                }
            });
            let x = -4.4 + i as f32 * 2.2 - shift * 0.5;
            let base = scene.add_mesh(kit::block(
                Vec3::new(x - 0.55, 0.0, z - 0.55),
                Vec3::new(x + 0.55, plinth, z + 0.55),
                1.6,
            ));
            scene.add_instance(base, stone, Mat4::IDENTITY);
            scene.add_instance(
                sphere,
                m,
                Mat4::from_translation(Vec3::new(x, plinth + 0.8, z))
                    * Mat4::from_scale(Vec3::splat(1.6)),
            );
        }
    }
    scene.lighting.light = DirectionalLight {
        intensity: 0.0,
        cast_shadows: false,
        ..Default::default()
    };
    scene.lighting.environment.intensity = 1.0;
    scene.lighting.environment.background = Background::Sky;
    scene.lighting.environment_map = Some(EnvironmentMap {
        rotation: 0.6,
        ..EnvironmentMap::new(map)
    });
    scene.look.post.ao = Some(AmbientOcclusion {
        method: AoMethod::Gtao,
        ..Default::default()
    });
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 3.0, 11.5),
        target: Vec3::new(0.0, 2.1, 0.0),
        projection: og_render::Projection::Perspective {
            fov_y_degrees: ENVIRONMENT_FOV,
        },
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    Ok(scene)
}

/// [`environment_scene`] with the sky's background blurred half way
/// (`Environment::background_blurriness` 0.5): the spheres are lit exactly
/// as in it, only the background softens.
pub fn blurry_scene() -> Result<RenderScene, String> {
    let mut scene = environment_scene()?;
    scene.lighting.environment.background_blurriness = 0.5;
    Ok(scene)
}

/// [`environment_scene`] half way through a blend from the day's sky to
/// a sunrise over a meadow (light and background).
pub fn blend_scene(weight: f32) -> Result<RenderScene, String> {
    let mut scene = environment_scene()?;
    let to = scene.add_environment_map(hdri_map("spruit", BAKED_SIZE, false)?);
    if let Some(m) = &mut scene.lighting.environment_map {
        m.blend = Some(og_render::EnvironmentBlend { to, weight });
    }
    Ok(scene)
}

/// A camp on a meadow at sunrise, its ground the photograph's own meadow
/// projected onto a grounded dome (height 1.5 m, radius 40 m, centred on
/// the origin; three.js GroundedSkybox): an oak barrel, crates, a boulder
/// and a chrome gazing ball on a stump, seen from 1.6 m up, lit by the map
/// alone. `grounded` false draws the same map at infinity, where the
/// meadow bends away towards the horizon instead.
pub fn grounded_scene(grounded: bool) -> Result<RenderScene, String> {
    let mut scene = RenderScene::new();
    let (w, h, rgb) = hdri_pixels("spruit")?;
    let mut irradiance = og_render::EquirectIrradiance::new(w, h);
    for (y, row) in rgb.chunks(w as usize * 3).enumerate() {
        irradiance.add_row(y as u32, row);
    }
    let texels = rgb
        .as_chunks::<3>()
        .0
        .iter()
        .map(|c| og_render::rgb9e5(*c))
        .collect();
    let map = scene.add_environment_equirect(
        og_render::EnvironmentEquirect {
            width: w,
            height: h,
            texels,
            size: 512,
        },
        irradiance.finish(),
    )?;
    kit::prop(&mut scene, "winebarrel").place(&mut scene, Vec3::new(-1.6, 0.0, 0.2), 0.6, 1.0);
    let crate_ = kit::prop(&mut scene, "crate");
    crate_.place(&mut scene, Vec3::new(-2.6, 0.0, -0.6), 0.3, 0.62);
    crate_.place(&mut scene, Vec3::new(-2.55, 0.62, -0.62), -0.2, 0.6);
    kit::prop(&mut scene, "boulder").place(&mut scene, Vec3::new(2.2, -0.2, -0.8), 2.0, 1.1);
    kit::prop(&mut scene, "column_broken").place(&mut scene, Vec3::new(0.3, 0.0, 0.0), 0.4, 0.9);
    let chrome = scene.add_material(Material {
        base_color: [0.95, 0.93, 0.88, 1.0],
        metallic: 1.0,
        roughness: 0.05,
        ..Default::default()
    });
    let sphere = scene.add_mesh(shapes::sphere(48, 32));
    scene.add_instance(
        sphere,
        chrome,
        Mat4::from_translation(Vec3::new(0.3, 1.35, 0.0)) * Mat4::from_scale(Vec3::splat(0.9)),
    );
    scene.lighting.light = DirectionalLight {
        intensity: 0.0,
        cast_shadows: false,
        ..Default::default()
    };
    scene.lighting.environment.intensity = 1.0;
    scene.lighting.environment.background = Background::Sky;
    scene.lighting.environment.ground = grounded.then_some(og_render::GroundProjection {
        height: 1.5,
        radius: 40.0,
        center: Vec3::ZERO,
        ..Default::default()
    });
    scene.lighting.environment_map = Some(EnvironmentMap {
        rotation: GROUNDED_TURN,
        ..EnvironmentMap::new(map)
    });
    scene.look.post.ao = Some(AmbientOcclusion {
        method: AoMethod::Gtao,
        ..Default::default()
    });
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 1.6, 5.5),
        target: Vec3::new(0.0, 0.7, 0.0),
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    Ok(scene)
}

/// How far [`grounded_scene`] turns the sunrise: the sun low on the left
/// of the view, the props lit from the side.
const GROUNDED_TURN: f32 = 2.2;

/// [`environment_scene`] lit by the sky's image loaded uncooked: the
/// renderer resamples and GGX-prefilters it on the GPU
/// (`RenderScene::add_environment_equirect`), with the irradiance gathered
/// from its rows as og.load.environmentData gathers it. With `cooked` the
/// cook's CPU bake of the same image instead, the frame it should match.
pub fn uncooked_scene(cooked: bool) -> Result<RenderScene, String> {
    let mut scene = environment_scene()?;
    let map = if cooked {
        scene.add_environment_map(hdri_map("kloofendal", BAKED_SIZE, false)?)
    } else {
        let (w, h, rgb) = hdri_pixels("kloofendal")?;
        let mut irradiance = og_render::EquirectIrradiance::new(w, h);
        for (y, row) in rgb.chunks(w as usize * 3).enumerate() {
            irradiance.add_row(y as u32, row);
        }
        let texels = rgb
            .as_chunks::<3>()
            .0
            .iter()
            .map(|c| og_render::rgb9e5(*c))
            .collect();
        scene.add_environment_equirect(
            og_render::EnvironmentEquirect {
                width: w,
                height: h,
                texels,
                size: BAKED_SIZE,
            },
            irradiance.finish(),
        )?
    };
    if let Some(m) = &mut scene.lighting.environment_map {
        m.map = map;
    }
    Ok(scene)
}

/// The colonnade's ruins on a lawn under the partly cloudy sky: a column,
/// a broken one, a boulder and three spheres (rough brass, white marble,
/// red lacquer) on a cobbled patch. With `extract` the bake takes the sun
/// out of the map (`sun extract`) and the scene's directional light is
/// that sun, so it casts shadows; without, the map lights the scene alone,
/// its sun still in it: the same light on an open surface, and no shadow.
pub fn sunny_scene(extract: bool) -> Result<RenderScene, String> {
    let mut scene = RenderScene::new();
    let baked = hdri_map("kloofendal", BAKED_SIZE, extract)?;
    let sun = baked.sun;
    let map = scene.add_environment_map(baked);
    terrace(&mut scene);
    kit::prop(&mut scene, "column").place(&mut scene, Vec3::new(-2.6, 0.0, -1.2), 0.3, 4.2);
    kit::prop(&mut scene, "column_broken").place(&mut scene, Vec3::new(3.4, 0.0, -1.6), 1.1, 1.8);
    kit::prop(&mut scene, "boulder").place(&mut scene, Vec3::new(5.5, -0.2, 1.0), 0.7, 1.0);
    let sphere = scene.add_mesh(shapes::sphere(48, 32));
    let looks = [
        ([0.95, 0.78, 0.5, 1.0], 1.0, 0.35),
        ([0.82, 0.8, 0.76, 1.0], 0.0, 0.5),
        ([0.7, 0.06, 0.04, 1.0], 0.0, 0.15),
    ];
    for (i, (base_color, metallic, roughness)) in looks.into_iter().enumerate() {
        let m = scene.add_material(Material {
            base_color,
            metallic,
            roughness,
            ..Default::default()
        });
        let r = 0.6;
        let at = Vec3::new(-0.9 + i as f32 * 1.6, r, -0.4 * (i as f32 - 1.0).abs());
        scene.add_instance(
            sphere,
            m,
            Mat4::from_translation(at) * Mat4::from_scale(Vec3::splat(2.0 * r)),
        );
    }
    scene.lighting.light = DirectionalLight {
        direction_to_light: if extract {
            Vec3::from_array(sun.direction)
        } else {
            Vec3::Y
        },
        color: Vec3::from_array(sun.color),
        intensity: sun.intensity,
        cast_shadows: extract,
        max_shadow_distance: 30.0,
        ..Default::default()
    };
    scene.lighting.environment.intensity = 1.0;
    scene.lighting.environment.background = Background::Sky;
    scene.lighting.environment_map = Some(EnvironmentMap::new(map));
    scene.frame.camera = Camera {
        position: Vec3::new(-0.6, 1.9, 7.5),
        target: Vec3::new(0.2, 1.0, 0.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: -0.5 },
        ..Default::default()
    };
    Ok(scene)
}
