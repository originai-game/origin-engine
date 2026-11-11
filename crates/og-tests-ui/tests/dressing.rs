//! CSS parts: the frames og-render draws, against `references/<tier>/`.
//! What og-ui lays out and builds for them is checked in
//! crates/og-ui/tests/dressing.rs.

use crate::support::*;
use og_ui::*;

#[test]
#[ignore = "requires GPU"]
fn css_parts_render_their_background_border_and_shadow() {
    use og_render::*;
    use og_ui::dress::PartRef;
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    let mut ui = ui_with_font(font);
    ui.set_part_css(include_str!("../../og-ui/tests/fixtures/parts.css"))
        .unwrap();
    let (width, height) = (464, 96);
    ui.begin(
        InputFrame::default(),
        Viewport::new(width as f32, height as f32),
    );
    ui.row(
        "States",
        Element {
            padding: Insets::all(24.0),
            gap: 24.0,
            ..Element::row()
        },
        |ui| {
            let base = PartRef::of("Button", "base", None);
            for (name, part) in [
                ("Rest", base),
                ("Hover", base.flag("hover")),
                (
                    "Epic",
                    base.flag("primary")
                        .flag("hover")
                        .attr("data-rarity", "epic"),
                ),
            ] {
                let draw = ui.resolve_part(&part, [120.0, 40.0]).unwrap();
                ui.column(
                    name,
                    Element {
                        width: Size::px(120.0),
                        height: Size::px(40.0),
                        ..Element::column()
                    },
                    |ui| ui.set_draw(draw),
                );
            }
        },
    );
    ui.end();
    let mut overlay = Overlay::new(width as f32, height as f32, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    scene.look.post.tonemap = Tonemap::None;
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width,
        height,
        ..Default::default()
    })
    .unwrap();
    renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
    let pixels = renderer.read_offscreen().unwrap().pixels;
    image::save_buffer(
        out_dir().join("css-parts.png"),
        &pixels,
        width,
        height,
        image::ColorType::Rgba8,
    )
    .unwrap();
    for (x, expected) in [
        (24, [254, 220, 186]),
        (84, [0x12, 0x34, 0x56]),
        (228, [255, 255, 0]),
        (372, [0, 0, 255]),
    ] {
        let at = (44 * width as usize + x) * 4;
        for c in 0..3 {
            assert!(
                (i16::from(pixels[at + c]) - expected[c]).abs() <= 3,
                "pixel at {x},44: {:?}",
                &pixels[at..at + 4]
            );
        }
    }
    let shadow = (67 * width as usize + 84) * 4;
    let ground = (85 * width as usize + 84) * 4;
    assert!(
        pixels[shadow] < pixels[ground],
        "the outer shadow darkens the ground"
    );
}
