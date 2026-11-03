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

use og_ui::glyphs::{PixelCells, pixel_cells, render_glyph, render_glyph_on};

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

/// Two cells that touch at a corner only are two squares, and a ring keeps
/// its hole: the outer contour counter-clockwise, the hole's clockwise.
#[test]
fn cells_that_meet_at_a_corner_stay_apart() {
    let diagonal = PixelCells {
        x0: 0,
        y0: 0,
        width: 2,
        height: 2,
        filled: vec![true, false, false, true],
    };
    let c = diagonal.contours();
    assert_eq!(c.len(), 2, "{c:?}");
    assert!(c.iter().all(|k| k.len() == 4), "{c:?}");
    let ring = PixelCells {
        x0: 0,
        y0: 0,
        width: 3,
        height: 3,
        filled: vec![true, true, true, true, false, true, true, true, true],
    };
    let c = ring.contours();
    assert_eq!(c.len(), 2, "{c:?}");
    let area = |k: &Vec<(i32, i32)>| -> i32 {
        (0..k.len())
            .map(|i| {
                let (a, b) = (k[i], k[(i + 1) % k.len()]);
                a.0 * b.1 - b.0 * a.1
            })
            .sum()
    };
    let mut areas: Vec<i32> = c.iter().map(area).collect();
    areas.sort();
    assert_eq!(areas, [-2, 18], "twice the signed areas: {c:?}");
    assert_eq!(winding(&c, 1.5, 1.5), 0, "the hole is open");
    assert_ne!(winding(&c, 0.5, 1.5), 0);
}

/// DotGothic16's dots sit on a pitch about 1.5 % finer than its 16-pixel
/// em: sampled at the cells' centres, each of its glyphs still fills whole
/// cells, one cell per dot, and advances whole cells.
#[test]
fn an_off_pitch_face_lands_on_whole_cells() {
    let ttf = font("styles/pixel/DotGothic16-Regular.ttf");
    let face = ttf_parser::Face::parse(&ttf, 0).unwrap();
    for c in ['O', 'v', 'i', 'e', 'w', 'N', 'l'] {
        let gid = face.glyph_index(c).unwrap();
        let cells = pixel_cells(&face, gid, 16);
        let filled = cells.filled.iter().filter(|&&f| f).count();
        assert!(filled >= 5, "{c}: {filled}");
        let g = render_glyph_on(&ttf, c, 48.0, 16.0, 16).unwrap();
        assert_eq!(g.advance, 0.5, "{c}: eight cells");
        // The plane is on the grid: its edges a whole number of cells from
        // the pen plus the padding.
        let pad = ((16.0f32 / 2.0).ceil() + 1.0) / 48.0;
        let on_grid = |v: f32| ((v * 16.0) - (v * 16.0).round()).abs() < 1e-3;
        assert!(
            on_grid(g.plane[0] + pad) && on_grid(g.plane[1] + pad),
            "{c}: {:?}",
            g.plane
        );
    }
}
