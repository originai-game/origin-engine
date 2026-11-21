use super::*;

/// A model's impostor is the level after its parts' coarsest finer
/// than it: chosen by `lod::select` once its error (an atlas texel)
/// projects under a pixel, and replacing the mesh levels coarser than
/// it.
#[test]
fn the_impostor_ends_the_level_chain() {
    let mut scene = RenderScene::new();
    let mesh = scene.add_mesh(og_render::shapes::cuboid(Vec3::ONE));
    let impostor = og_render::Impostor {
        material: scene.add_standard_material(Default::default()),
        grid: 8,
        center: Vec3::ZERO,
        radius: 1.0,
        material_atlas: false,
        below: false,
        trim: og_render::CardTrim::FULL,
    };
    let lods = ModelLods {
        // One part with levels 3 cm and 12 cm off, another with one.
        parts: vec![vec![(mesh, 0.03), (mesh, 0.12)], vec![(mesh, 0.02)]],
        materials: Vec::new(),
        center: Vec3::ZERO,
        radius: 1.0,
        #[cfg(not(feature = "profile2d"))]
        scatter_meshes: Vec::new(),
        impostor: Some((impostor, 0.05)),
    };
    // 5 cm: finer than level 2 (12 cm), which it replaces; drawn once
    // it would show under a pixel (20 px per unit), level 1 nearer.
    assert_eq!(lods.errors(), vec![0.0, 0.03, 0.05]);
    assert_eq!(lods.impostor_level(), Some(2));
    assert_eq!(og_render::lod::select(&lods.errors(), 30.0, 1.0, None), 1);
    assert_eq!(og_render::lod::select(&lods.errors(), 19.0, 1.0, None), 2);
    // Finer than every mesh level (a palm's, whose coarse fronds are
    // metres off): it follows the full mesh directly.
    let fine = ModelLods {
        impostor: Some((impostor, 2.0 / 128.0)),
        ..lods.clone()
    };
    assert_eq!(fine.errors(), vec![0.0, 2.0 / 128.0]);
    assert_eq!(fine.impostor_level(), Some(1));
    assert_eq!(og_render::lod::select(&fine.errors(), 60.0, 1.0, None), 1);
    // Coarser than every one: after the coarsest.
    let coarse = ModelLods {
        impostor: Some((impostor, 0.5)),
        ..lods.clone()
    };
    assert_eq!(coarse.errors(), vec![0.0, 0.03, 0.12, 0.5]);
    assert_eq!(coarse.impostor_level(), Some(3));
    let none = ModelLods {
        impostor: None,
        ..lods
    };
    assert_eq!(none.errors().len(), 3);
    assert_eq!(none.impostor_level(), None);
}
