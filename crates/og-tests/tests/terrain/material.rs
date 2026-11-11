//! Terrain material v2 read from rendered frames: hex-tiling breaks a
//! layer's repeat, and past `projectSlope` a steep face is sampled by the
//! projections facing it instead of streaking.

use super::*;

#[test]
#[ignore = "requires GPU"]
fn cell_variation_leaves_opaque_normal_maps_unchanged() {
    for anti_tiling in [0.0, 1.0] {
        let mut scene = ts::tiling_scene(anti_tiling);
        let normal = scene.add_texture(og_render::TextureData {
            width: 1,
            height: 1,
            rgba: vec![128, 128, 255, 255],
            mips: og_render::Mips::None,
            color_space: og_render::ColorSpace::Linear,
        });
        scene.world.terrains[0].layers[0].normal = Some(normal);
        let (plain, _) = render(&scene);
        scene.world.terrains[0].layers[0].cell_variation = 1.0;
        let (varied, _) = render(&scene);
        assert!(plain.pixels == varied.pixels, "anti_tiling {anti_tiling}");
    }
}
