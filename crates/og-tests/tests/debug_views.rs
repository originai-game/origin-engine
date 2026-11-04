//! Diagnostic views must change the surface being inspected, not merely
//! its background. The removed-subject control selects that surface's pixels.
use glam::Vec3;
use og_render::{
    AmbientOcclusion, Background, DebugView, Exposure, RenderScene, RenderTarget, Renderer,
    RgbaImage, Tonemap,
};
use og_tests::scenes;

const VIEWS: [DebugView; 11] = [
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
    s.post = Default::default();
    s.post.ao = Some(AmbientOcclusion::default());
    s.camera.exposure = Exposure::linear(1.0);
    s.post.tonemap = Tonemap::None;
    if name != "sky-only" {
        s.environment.background = Background::Color(Vec3::ZERO);
    }
    if remove {
        match name {
            "terrain-near" => s.terrains.clear(),
            "grass-near" => s.grass.clear(),
            "terrain-grass" => s.scatters.clear(),
            "water-planar" => s.water.clear(),
            "particles" => s.particles.clear(),
            "trails-beams" => {
                s.trails.clear();
                s.beams.clear();
            }
            "decals" => s.decals.clear(),
            "sky-only" => s.environment.background = Background::Color(Vec3::ZERO),
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
            .chunks_exact(4)
            .zip(absent.pixels.chunks_exact(4))
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
                    DebugView::Terrain | DebugView::TerrainLayers | DebugView::TerrainMip
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
fn terrain_albedo_is_independent_of_direct_light() {
    let desc = og_render::RenderDesc {
        width: 320,
        height: 180,
        ..scenes::base_desc()
    };
    let mut r = Renderer::new_blocking(&desc).unwrap();
    let mut s = scene("terrain-near", false);
    s.debug = DebugView::Albedo;
    s.light.intensity = 0.0;
    let dark = capture(&mut r, &s);
    s.light.intensity = 8.0;
    s.light.color = Vec3::new(0.1, 0.7, 1.0);
    let lit = capture(&mut r, &s);
    assert_eq!(
        dark.pixels, lit.pixels,
        "albedo must not include direct light"
    );
}
