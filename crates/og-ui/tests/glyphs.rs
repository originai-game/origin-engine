//! Run-time glyphs (`og_ui::glyphs`): a character outside a cooked atlas,
//! rendered from the font file at the cook's geometry.
//!
//! Ways it could fail, and the test that catches each:
//! - the field's inside and outside swapped, or the glyph flipped (atlas
//!   rows go top to bottom) → `a_glyph_has_ink_inside_and_none_outside`
//! - a render that differs from one call to the next (it must be the same
//!   bytes on every host) → the same test
//! - plane bounds or advance out of step with the font's metrics, so a
//!   run-time glyph sits elsewhere than a cooked one would →
//!   `its_metrics_are_the_fonts`

use og_ui::glyphs::render_glyph;

fn sc() -> Vec<u8> {
    std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/ui-kit/assets/fonts/NotoSansSC-Regular-subset.ttf"
    ))
    .expect("the kit's Chinese font")
}

#[test]
fn a_glyph_has_ink_inside_and_none_outside() {
    let ttf = sc();
    // 口: a box, ink on its frame, none in its middle or around it.
    let g = render_glyph(&ttf, '口', 32.0, 4.0).expect("the font has 口");
    let at = |x: u32, y: u32| g.rgba[((y * g.width + x) * 4) as usize];
    assert!(g.width > 20 && g.height > 20, "{}x{}", g.width, g.height);
    assert!(at(1, 1) < 64, "outside, at the padding: {}", at(1, 1));
    assert!(
        at(g.width / 2, g.height / 2) < 128,
        "the box's middle is empty"
    );
    // Down the middle column from the top, the first ink is the top stroke.
    let top = (0..g.height)
        .find(|&y| at(g.width / 2, y) > 128)
        .expect("a top stroke");
    assert!(
        top < g.height / 3,
        "the top stroke is at the top (rows go down): {top}"
    );
    // Every channel holds the same distance (a true SDF the MTSDF shader reads).
    for p in g.rgba.chunks(4) {
        assert!(p[0] == p[1] && p[1] == p[2] && p[2] == p[3]);
    }
    assert_eq!(
        render_glyph(&ttf, '口', 32.0, 4.0),
        Some(g),
        "the same bytes every time"
    );
}

#[test]
fn its_metrics_are_the_fonts() {
    let ttf = sc();
    let face = ttf_parser::Face::parse(&ttf, 0).expect("a font");
    let g = render_glyph(&ttf, '旅', 32.0, 4.0).expect("the font has 旅");
    let gid = face.glyph_index('旅').expect("in the cmap");
    let upem = f32::from(face.units_per_em());
    assert_eq!(
        g.advance,
        f32::from(face.glyph_hor_advance(gid).expect("an advance")) / upem
    );
    let bbox = face.glyph_bounding_box(gid).expect("a box");
    // The plane's left edge is the box's minus the padding, in em.
    let pad = (4.0f32 / 2.0).ceil() + 1.0;
    assert!((g.plane[0] - (f32::from(bbox.x_min) / upem - pad / 32.0)).abs() < 1e-5);
    assert!((g.plane[2] - g.plane[0] - g.width as f32 / 32.0).abs() < 1e-5);
    assert!(
        render_glyph(&ttf, 'ע', 32.0, 4.0).is_none(),
        "a character the font lacks"
    );
    let space = render_glyph(&ttf, ' ', 32.0, 4.0).expect("a space");
    assert_eq!((space.width, space.plane), (0, [0.0; 4]));
}
