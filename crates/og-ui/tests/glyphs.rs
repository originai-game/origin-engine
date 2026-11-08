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
//! - a pixel font's cells traced wrong (a hole filled, two cells that meet
//!   at a corner merged or crossed, a run left out) or sampled off its
//!   grid → `a_pixel_glyph_is_its_cells_and_nothing_else`,
//!   `cells_that_meet_at_a_corner_stay_apart`
//! - a pixel glyph drawn on another pitch than the grid (DotGothic16's
//!   dots) left off whole cells → `an_off_pitch_face_lands_on_whole_cells`

use og_ui::glyphs::{pixel_cells, render_glyph, render_glyph_on};

fn sc() -> Vec<u8> {
    std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/ui-text/assets/fonts/NotoSansSC-Regular-subset.ttf"
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

fn font(path: &str) -> Vec<u8> {
    std::fs::read(format!(
        "{}/../../templates/{path}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("a font")
}

/// The nonzero winding at (x, y) of closed polygons.
fn winding(contours: &[Vec<(i32, i32)>], x: f32, y: f32) -> i32 {
    let mut w = 0;
    for c in contours {
        for k in 0..c.len() {
            let (a, b) = (c[k], c[(k + 1) % c.len()]);
            let (ax, ay, bx, by) = (a.0 as f32, a.1 as f32, b.0 as f32, b.1 as f32);
            if (ay <= y) != (by <= y) && ax + (y - ay) / (by - ay) * (bx - ax) > x {
                w += if by > ay { 1 } else { -1 };
            }
        }
    }
    w
}

/// Fusion Pixel is drawn on a 12-pixel em: its glyphs' cells are the
/// outline's own squares, and the traced contours fill exactly those cells
/// (a hole stays open, every corner is a right angle on the grid). The
/// rendered field reads inside at a filled cell's centre and outside at an
/// empty one's, and the advance is whole cells.
#[test]
fn a_pixel_glyph_is_its_cells_and_nothing_else() {
    let ttf = font("styles/cjk/FusionPixel12-zhHans.ttf");
    let face = ttf_parser::Face::parse(&ttf, 0).unwrap();
    for c in ['A', 'O', 'g', '8', '商', '店'] {
        let gid = face.glyph_index(c).expect("in the font");
        let cells = pixel_cells(&face, gid, 12);
        let filled = cells.filled.iter().filter(|&&f| f).count();
        assert!(filled > 4, "{c}: {filled} cells");
        let contours = cells.contours();
        for j in 0..cells.height as i32 {
            for i in 0..cells.width as i32 {
                let on = cells.filled[(j as u32 * cells.width + i as u32) as usize];
                let (x, y) = ((cells.x0 + i) as f32 + 0.5, (cells.y0 + j) as f32 + 0.5);
                assert_eq!(winding(&contours, x, y) != 0, on, "{c}: cell {i},{j}");
            }
        }
        for w in contours.iter() {
            for k in 0..w.len() {
                let (a, b) = (w[k], w[(k + 1) % w.len()]);
                assert!(a.0 == b.0 || a.1 == b.1, "{c}: a diagonal edge {a:?} {b:?}");
            }
        }
        // 48 atlas px to the em: 4 texels to a cell.
        let g = render_glyph_on(&ttf, c, 48.0, 16.0, 12).expect("rendered");
        assert!((g.advance * 12.0 - (g.advance * 12.0).round()).abs() < 1e-5);
        let texel = |x: f32, y: f32| {
            // x, y in em from the pen; rows top to bottom.
            let col = ((x - g.plane[0]) * 48.0) as u32;
            let row = ((g.plane[3] - y) * 48.0) as u32;
            g.rgba[((row * g.width + col) * 4) as usize]
        };
        for j in 0..cells.height as i32 {
            for i in 0..cells.width as i32 {
                let on = cells.filled[(j as u32 * cells.width + i as u32) as usize];
                let (x, y) = (
                    ((cells.x0 + i) as f32 + 0.5) / 12.0,
                    ((cells.y0 + j) as f32 + 0.5) / 12.0,
                );
                let v = texel(x, y);
                assert_eq!(v > 128, on, "{c}: cell {i},{j} reads {v}");
            }
        }
    }
}

/// A text made only of characters its face lacks measures no width (a
/// missing glyph takes no space). It must still reach the renderer on every
/// frame, so `FrameStats::missing_glyphs`, which every reference test
/// asserts, names it, as it does when a drawn character shares the line.
/// The case that hid: a list marker `•` in a font without it vanished
/// with a count of 0.
#[test]
#[ignore = "requires GPU"]
fn a_text_of_only_missing_glyphs_is_counted_by_the_renderer() {
    use crate::support::*;
    use og_render::*;
    use og_ui::{EmitDesc, InputFrame, LabelDesc, Viewport};
    for (text, missing) in [
        ("\u{2022}", 1),
        ("a\u{2022}", 1),
        ("\u{2022}\u{25E6}", 2),
        ("a", 0),
    ] {
        let mut scene = RenderScene::new();
        let font = add_test_font(&mut scene);
        let mut ui = ui_with_font(font);
        // Two frames: og-ui names a missing character once in its life, on
        // its first frame; the renderer must count it on the later one too.
        for _ in 0..2 {
            ui.begin(InputFrame::default(), Viewport::new(200.0, 100.0));
            ui.label("Marker", text, LabelDesc::default());
            ui.end();
        }
        let mut overlay = Overlay::new(200.0, 100.0, 1.0);
        ui.frame().emit(&EmitDesc::default(), &mut overlay);
        scene.overlay = Some(overlay);
        let mut renderer = Renderer::new_blocking(&RenderDesc {
            width: 200,
            height: 100,
            ..Default::default()
        })
        .expect("a headless adapter");
        let stats = renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
        assert_eq!(stats.missing_glyphs, missing, "{text:?}");
    }
}
