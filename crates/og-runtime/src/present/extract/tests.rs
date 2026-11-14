use super::*;

/// A text in a font of two pages is drawn again with the second, with
/// the first's shadows and outline; a text in a one-page font, or one
/// before `first`, is left alone.
#[test]
fn a_text_drawn_on_its_fonts_other_pages_keeps_its_effects() {
    let mut scene = RenderScene::new();
    let atlas = scene.add_texture(og_render::TextureData {
        width: 1,
        height: 1,
        rgba: vec![0; 4],
        color_space: og_render::ColorSpace::Linear,
        mips: og_render::Mips::None,
    });
    let font = || og_render::MsdfFont {
        atlas,
        distance_range: 4.0,
        foreground: false,
        silhouette: false,
        line_height: 1.2,
        glyphs: Vec::new(),
        kerning: Vec::new(),
    };
    let (a, a2, b) = (
        scene.add_font(font()),
        scene.add_font(font()),
        scene.add_font(font()),
    );
    let mut overlay = og_render::Overlay::new(100.0, 100.0, 1.0);
    let text = og_render::Text::new;
    overlay.texts = vec![text(a, "before"), text(a, "styled"), text(b, "one page")];
    let shadow = og_render::UiTextShadow {
        color: [0.0, 0.0, 0.0, 1.0],
        offset: og_core::glam::Vec2::new(2.0, 2.0),
        blur: 3.0,
    };
    let effects = |text| og_render::UiTextEffects {
        text,
        shadows: vec![shadow],
        outline: None,
        fill: None,
        fill_box: [0.0; 4],
        transform: [1.0, 0.0, 0.0, 1.0],
    };
    overlay.ui.text_effects = vec![effects(0), effects(1)];
    let pages = BTreeMap::from([(a, vec![a, a2]), (b, vec![b])]);
    draw_other_pages(&mut overlay, 1, &pages);
    let drawn: Vec<_> = overlay
        .texts
        .iter()
        .map(|t| (t.font, t.text.as_str()))
        .collect();
    assert_eq!(
        drawn,
        [
            (a, "before"),
            (a, "styled"),
            (b, "one page"),
            (a2, "styled")
        ]
    );
    let on: Vec<u32> = overlay.ui.text_effects.iter().map(|e| e.text).collect();
    assert_eq!(on, [0, 1, 3]);
    assert_eq!(overlay.ui.text_effects[2].shadows, [shadow]);
}

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
    };
    let lods = ModelLods {
        // One part with levels 3 cm and 12 cm off, another with one.
        parts: vec![vec![(mesh, 0.03), (mesh, 0.12)], vec![(mesh, 0.02)]],
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
