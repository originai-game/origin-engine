//! A game's screen sprites against og.ui (hollowbell FINDINGS #8): at a
//! layer up to `og_render::UI_LAYER` they draw under every UI element, above
//! it over every one, in their layers' order, as og-runtime places them
//! (the UI at `UI_OVERLAY_LAYER`, a screen layer by `screen_layer`). A HUD
//! panel with a label; a red square at layer 999 hidden by it, a green one
//! at 999 half off its edge (the half outside shows), a gold one at
//! `UI_LAYER + 1` on the panel and a blue one at `UI_LAYER + 2` over the
//! gold one's corner. The frame matches `references/<tier>/screen-layers.png`
//! (`OG_UI_CAPTURE=1` records it; a tier without its own reference is
//! compared with the metal one).
//!
//! Ways it could fail: every screen sprite still draws under the UI (the
//! gold square is hidden: the finding), every one draws over it (the red
//! square shows), or the lift reorders screen layers above the UI (the gold
//! square covers the blue one).

use crate::support;

use og_core::glam::{Vec2, Vec3};
use og_render::{
    Background, RenderDesc, RenderScene, RenderTarget, Renderer, Sprite, Tonemap, UI_LAYER,
    UI_OVERLAY_LAYER, screen_layer,
};
use og_ui::*;
use std::path::PathBuf;
use support::*;

fn square(x: f32, y: f32, size: f32, color: [f32; 4], layer: i32) -> Sprite {
    Sprite {
        texture: None,
        position: Vec3::new(x, y, 0.0),
        size: Vec2::splat(size),
        anchor: Vec2::ZERO,
        color,
        layer: screen_layer(layer),
        ..Default::default()
    }
}

#[test]
fn screen_sprites_above_the_ui_layer_draw_over_the_ui() {
    assert_eq!(screen_layer(UI_LAYER), UI_LAYER, "at the UI's layer: under");
    assert!(screen_layer(UI_LAYER + 1) > UI_OVERLAY_LAYER + (1 << 20) - 1);
    assert!(screen_layer(i32::MAX) >= screen_layer(UI_LAYER + 2));

    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    let atlas = scene.add_texture(atlas_texture());
    let mut ui = ui_with_font(font);
    let vp = Viewport::new(320.0, 180.0);
    ui.begin(InputFrame::default(), vp);
    ui.panel(
        "Hud",
        PanelDesc {
            width: Size::Fixed(220.0),
            height: Size::Fixed(110.0),
            ..PanelDesc::default()
        },
        |ui| ui.label("Chimes", "Chimes", LabelDesc::default()),
    );
    ui.end();
    let hud = ui.frame().element("Hud").expect("the panel").clone();
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, 1.0);
    ui.frame().emit(
        &EmitDesc {
            atlas,
            layer: UI_OVERLAY_LAYER,
        },
        &mut overlay,
    );
    let (cx, cy) = (hud.x + hud.width / 2.0, hud.y + hud.height / 2.0);
    let red = [0.9, 0.1, 0.1, 1.0];
    let green = [0.1, 0.8, 0.2, 1.0];
    let gold = [1.0, 0.75, 0.2, 1.0];
    let blue = [0.2, 0.4, 1.0, 1.0];
    // Submitted over-first, so only the layers order them.
    overlay
        .sprites
        .push(square(cx + 10.0, cy - 10.0, 30.0, blue, UI_LAYER + 2));
    overlay
        .sprites
        .push(square(cx - 10.0, cy - 20.0, 40.0, gold, UI_LAYER + 1));
    overlay
        .sprites
        .push(square(hud.x + 20.0, cy - 10.0, 30.0, red, 999));
    overlay.sprites.push(square(
        hud.x + hud.width - 20.0,
        cy + 10.0,
        40.0,
        green,
        999,
    ));
    scene.overlay = Some(overlay);
    scene.environment.background = Background::Color(Vec3::new(0.02, 0.02, 0.03));
    scene.post.tonemap = Tonemap::None;
    let desc = RenderDesc {
        width: vp.width as u32,
        height: vp.height as u32,
        ..RenderDesc::default()
    };
    let mut renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
    renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    let image = renderer.read_offscreen().expect("read back");
    let px = |x: f32, y: f32| {
        let i = ((y as u32 * image.width + x as u32) * 4) as usize;
        [image.pixels[i], image.pixels[i + 1], image.pixels[i + 2]]
    };
    // The overlay's colours are linear, encoded to sRGB on the way out.
    let srgb = |v: f32| {
        let e = if v <= 0.003_130_8 {
            v * 12.92
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        };
        e * 255.0
    };
    let near =
        |p: [u8; 3], c: [f32; 4]| (0..3).all(|k| (f32::from(p[k]) - srgb(c[k])).abs() < 12.0);
    assert!(near(px(cx, cy), gold), "the gold square is over the panel");
    assert!(
        near(px(cx + 35.0, cy + 15.0), blue),
        "the blue square is over the gold one"
    );
    let red_at = px(hud.x + 35.0, cy + 5.0);
    assert!(
        !near(red_at, red),
        "the red square is under the panel: {red_at:?}"
    );
    let inside = px(hud.x + hud.width - 10.0, cy + 30.0);
    assert!(
        !near(inside, green),
        "the green square is under the panel: {inside:?}"
    );
    assert!(
        near(px(hud.x + hud.width + 10.0, cy + 30.0), green),
        "the green square shows past the panel's edge"
    );

    let tier = renderer.info().backend_tier();
    let actual = out_dir().join(format!("screen-layers-{tier}.png"));
    image::save_buffer(
        &actual,
        &image.pixels,
        image.width,
        image.height,
        image::ColorType::Rgba8,
    )
    .expect("write frame");
    let dir = |tier: &str| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("references")
            .join(tier)
    };
    let own = dir(&tier).join("screen-layers.png");
    if std::env::var_os("OG_UI_CAPTURE").is_some() {
        std::fs::create_dir_all(own.parent().expect("dir")).expect("reference dir");
        std::fs::copy(&actual, &own).expect("record reference");
    }
    let reference = if own.exists() {
        own
    } else {
        dir("metal").join("screen-layers.png")
    };
    let expected = image::open(&reference)
        .unwrap_or_else(|e| panic!("{}: {e} (OG_UI_CAPTURE=1 records it)", reference.display()))
        .to_rgba8();
    if let Err(e) = compare(&image.pixels, expected.as_raw()) {
        panic!(
            "{} differs from {}: {e}",
            actual.display(),
            reference.display()
        );
    }
}
