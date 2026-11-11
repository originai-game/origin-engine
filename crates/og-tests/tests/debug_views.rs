//! Diagnostic views must change the surface being inspected, not merely
//! its background. The removed-subject control selects that surface's pixels.
use glam::Vec3;
use og_render::{
    AmbientOcclusion, Background, DebugView, Exposure, RenderScene, RenderTarget, Renderer,
    RgbaImage, Tonemap,
};
use og_tests::scenes;

const VIEWS: [DebugView; 13] = [
    DebugView::Cascades,
    DebugView::AmbientOcclusion,
    DebugView::Albedo,
    DebugView::IblDiffuse,
    DebugView::IblSpecular,
    DebugView::DirectLight,
    DebugView::AoDepth,
    DebugView::AoRaw,
    DebugView::Terrain,
    DebugView::TerrainLayers,
    DebugView::TerrainMip,
    DebugView::TerrainProjection,
    DebugView::TerrainDensity,
];

fn scene(name: &str, remove: bool) -> RenderScene {
    let mut s = scenes::build(name).unwrap().scene;
    if name == "shader-full-dissolve" {
        // The standard-material ground dominates this scene but is not
        // the custom shading path under test. Keep only its custom draws.
        s.instances = s
            .instances
            .iter()
            .filter(|i| s.material_shader(i.material).is_some())
            .copied()
            .collect();
    }
    s.look.post = Default::default();
    s.look.post.ao = Some(AmbientOcclusion::default());
    s.camera.exposure = Exposure::linear(1.0);
    s.look.post.tonemap = Tonemap::None;
    if name != "sky-only" {
        s.lighting.environment.background = Background::Color(Vec3::ZERO);
    }
    if remove {
        match name {
            "terrain-near" => s.world.terrains.clear(),
            "grass-near" => s.world.grass.clear(),
            "terrain-grass" => s.world.scatters.clear(),
            "water-planar" => s.world.water.clear(),
            "particles" => s.particles.clear(),
            "trails-beams" => {
                s.trails.clear();
                s.beams.clear();
            }
            "decals" => s.decals.clear(),
            "sky-only" => s.lighting.environment.background = Background::Color(Vec3::ZERO),
            "impostors" => s.impostors.clear(),
            _ => {
                s.instances.clear();
                s.skinned_instances.clear();
            }
        }
    }
    s
}

fn capture(r: &mut Renderer, s: &RenderScene) -> RgbaImage {
    r.render(s, &RenderTarget::Offscreen).unwrap();
    r.read_offscreen().unwrap()
}

fn difference(a: &[u8], b: &[u8]) -> u8 {
    (0..3).map(|i| a[i].abs_diff(b[i])).max().unwrap()
}

#[test]
#[ignore = "requires GPU"]
fn every_shading_path_honours_each_diagnostic_or_marks_it_unsupported() {
    for name in [
        "terrain-near",
        "grass-near",
        "terrain-grass",
        "water-planar",
        "particles",
        "trails-beams",
        "decals",
        "sky-only",
        "materials",
        "materials-physical",
        "shader-full-dissolve",
        "impostors",
    ] {
        let desc = og_render::RenderDesc {
            width: 320,
            height: 180,
            ..scenes::base_desc()
        };
        let mut r = Renderer::new_blocking(&desc).unwrap();
        let mut s = scene(name, false);
        // Advance only the particle fixture; other fixtures are static.
        if name == "particles" {
            for tick in 0..60 {
                s.tick = tick;
                r.render(&s, &RenderTarget::Offscreen).unwrap();
            }
            s.tick = 60;
        }
        let beauty = capture(&mut r, &s);
        let absent = capture(
            &mut Renderer::new_blocking(&desc).unwrap(),
            &scene(name, true),
        );
        let mask: Vec<_> = beauty
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .zip(absent.pixels.as_chunks::<4>().0)
            .enumerate()
            .filter_map(|(i, (a, b))| (difference(a, b) > 48).then_some(i))
            .collect();
        assert!(
            mask.len() > 20,
            "{name}: fixture does not expose its subject ({} pixels)",
            mask.len()
        );
        for view in VIEWS {
            s.debug = view;
            let image = capture(&mut r, &s);
            let changed = mask
                .iter()
                .filter(|&&i| {
                    difference(
                        &beauty.pixels[i * 4..i * 4 + 4],
                        &image.pixels[i * 4..i * 4 + 4],
                    ) > 8
                })
                .count();
            assert!(
                changed * 4 > mask.len(),
                "{name}/{view:?}: diagnostic left the subject in beauty shading ({changed}/{})",
                mask.len()
            );
            let unsupported = match name {
                "particles" | "trails-beams" | "sky-only" => true,
                "water-planar" => !matches!(
                    view,
                    DebugView::AmbientOcclusion | DebugView::AoDepth | DebugView::AoRaw
                ),
                "impostors" | "shader-full-dissolve" => view != DebugView::Albedo,
                "materials-physical" => matches!(
                    view,
                    DebugView::IblDiffuse
                        | DebugView::IblSpecular
                        | DebugView::DirectLight
                        | DebugView::Terrain
                        | DebugView::TerrainLayers
                        | DebugView::TerrainMip
                        | DebugView::TerrainProjection
                        | DebugView::TerrainDensity
                ),
                "terrain-grass" => !matches!(
                    view,
                    DebugView::Albedo
                        | DebugView::AmbientOcclusion
                        | DebugView::AoDepth
                        | DebugView::AoRaw
                ),
                "terrain-near" => matches!(
                    view,
                    DebugView::AmbientOcclusion | DebugView::AoDepth | DebugView::AoRaw
                ),
                _ => matches!(
                    view,
                    DebugView::Terrain
                        | DebugView::TerrainLayers
                        | DebugView::TerrainMip
                        | DebugView::TerrainProjection
                        | DebugView::TerrainDensity
                ),
            };
            if unsupported {
                let marked = mask
                    .iter()
                    .filter(|&&i| {
                        let p = &image.pixels[i * 4..i * 4 + 4];
                        p[0] > 120 && p[1] < 32 && p[2] > 120
                    })
                    .count();
                assert!(
                    marked * 10 > mask.len(),
                    "{name}/{view:?}: missing unsupported checker ({marked}/{})",
                    mask.len()
                );
            }
        }
    }
}

#[test]
#[ignore = "requires GPU"]
fn terrain_projection_and_vertical_density_use_the_sampled_axes() {
    use glam::{Mat4, Vec2};
    use og_render::{
        Camera, ColorSpace, Mips, Projection, Terrain, TerrainData, TerrainLayer, TextureData,
    };
    let desc = og_render::RenderDesc {
        width: 64,
        height: 64,
        ..scenes::base_desc()
    };
    let mut r = Renderer::new_blocking(&desc).unwrap();
    let mut s = RenderScene::new();
    let texture = s.add_texture(TextureData {
        width: 64,
        height: 32,
        rgba: vec![255; 64 * 32 * 4],
        color_space: ColorSpace::Srgb,
        mips: Mips::Generate,
    });
    let h = s.add_terrain(TerrainData {
        size: Vec2::splat(16.0),
        columns: 17,
        rows: 17,
        heights: vec![0.0; 17 * 17].into(),
        control: None,
        holes: None,
    });
    s.world.terrains.push(Terrain {
        layers: vec![TerrainLayer {
            albedo: Some(texture),
            tile: 2.0,
            triplanar: true,
            ..Default::default()
        }],
        ..Terrain::new(h)
    });
    s.look.post.tonemap = Tonemap::None;
    s.camera = Camera {
        position: Vec3::new(0.0, 10.0, 0.0),
        target: Vec3::ZERO,
        up: Vec3::Z,
        projection: Projection::Orthographic { height: 8.0 },
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    for wall in [false, true] {
        if wall {
            s.world.terrains[0].transform = Mat4::from_rotation_x(std::f32::consts::FRAC_PI_2);
            s.camera.position = Vec3::new(0.0, 0.0, 10.0);
            s.camera.up = Vec3::Y;
        }
        // 8m / 64 pixels / 2m per repeat: 4 texels along the texture's
        // 64-wide U axis, 2 along its 32-high V axis. A collapsed axis is 0.
        for (view, expected) in [
            (
                DebugView::TerrainProjection,
                if wall {
                    [0.0, 0.0, 1.0]
                } else {
                    [0.0, 1.0, 0.0]
                },
            ),
            (
                DebugView::TerrainDensity,
                if wall {
                    [2.0 / 3.0, 0.0, 2.0 / 3.0]
                } else {
                    [0.8, 2.0 / 3.0, 0.0]
                },
            ),
        ] {
            s.debug = view;
            let image = capture(&mut r, &s);
            let at = (32 * 64 + 32) * 4;
            for (channel, want) in expected.into_iter().enumerate() {
                let srgb = image.pixels[at + channel] as f32 / 255.0;
                let linear = if srgb <= 0.04045 {
                    srgb / 12.92
                } else {
                    ((srgb + 0.055) / 1.055).powf(2.4)
                };
                assert!(
                    (linear - want).abs() < 0.025,
                    "wall={wall}/{view:?} channel={channel}: {linear} expected {want}"
                );
            }
        }
    }
}
