//! An effect beam reads as light, not a slab: the strip fades a textured
//! effect across its width and at its ends (ribbon.wgsl `profile`).

use super::*;

/// The effect-materials beam (the frame's one green thing) is brightest
/// along its middle and dim at its sides, and dim at its ends against its
/// middle: averaged along its length (or across it) so the noise
/// scrolling through it evens out. A beam whose textured effect drew it
/// solid to its edges and cut its ends square (the bounced frame) has
/// sides and ends as bright as its middle.
#[test]
#[ignore = "requires GPU"]
fn an_effect_beam_fades_across_its_width_and_at_its_ends() {
    let img = gfx::render("effect-materials").expect("render").image;
    let green = |x: u32, y: u32| {
        let i = ((y * img.width + x) * 4) as usize;
        let p = &img.pixels[i..i + 3];
        let (r, g, b) = (p[0] as f64, p[1] as f64, p[2] as f64);
        (g > 70.0 && g > 1.3 * r && g > 1.3 * b).then_some(g)
    };
    // Its box: the green pixels' columns and rows.
    let (mut x0, mut x1, mut y0, mut y1) = (img.width, 0, img.height, 0);
    for y in 0..img.height {
        for x in 0..img.width {
            if green(x, y).is_some() {
                (x0, x1, y0, y1) = (x0.min(x), x1.max(x), y0.min(y), y1.max(y));
            }
        }
    }
    assert!(
        x1 > x0 + 10 && y1 > y0 + 60,
        "no beam found: ({x0}, {y0}) to ({x1}, {y1})"
    );
    let g = |x: u32, y: u32| {
        let i = ((y * img.width + x) * 4) as usize;
        img.pixels[i + 1] as f64
    };
    let mean = |xs: std::ops::Range<u32>, ys: std::ops::Range<u32>| {
        let n = (xs.len() * ys.len()) as f64;
        ys.flat_map(|y| xs.clone().map(move |x| (x, y)))
            .map(|(x, y)| g(x, y))
            .sum::<f64>()
            / n
    };
    let (w, h) = (x1 - x0 + 1, y1 - y0 + 1);
    let middle_rows = y0 + h / 3..y1 - h / 3;
    let middle_cols = x0 + w * 2 / 5..x1 - w * 2 / 5 + 1;
    let core = mean(middle_cols.clone(), middle_rows.clone());
    // Across it: each row's own green span (the beam leans in the view),
    // its outer sixths against its middle fifth.
    let (mut mid, mut left, mut right, mut rows) = (0.0, 0.0, 0.0, 0.0);
    for y in middle_rows {
        let span: Vec<u32> = (x0..=x1).filter(|&x| green(x, y).is_some()).collect();
        let (Some(&a), Some(&b)) = (span.first(), span.last()) else {
            continue;
        };
        let n = b - a + 1;
        if n < 6 {
            continue;
        }
        let s = n / 6;
        mid += mean(a + n * 2 / 5..b + 1 - n * 2 / 5, y..y + 1);
        left += mean(a..a + s, y..y + 1);
        right += mean(b + 1 - s..b + 1, y..y + 1);
        rows += 1.0;
    }
    assert!(rows > 0.0, "no row of the beam is 6 pixels wide");
    let (mid, left, right) = (mid / rows, left / rows, right / rows);
    let top = mean(middle_cols.clone(), y0..y0 + h / 12);
    let bottom = mean(middle_cols, y1 + 1 - h / 12..y1 + 1);
    println!(
        "beam ({x0}, {y0}) {w}x{h}: core {core:.0} across {mid:.0}: sides {left:.0} {right:.0}; ends {top:.0} {bottom:.0}"
    );
    for (name, v, against) in [
        ("left side", left, mid),
        ("right side", right, mid),
        ("top end", top, core),
        ("bottom end", bottom, core),
    ] {
        assert!(
            v < 0.7 * against,
            "the {name} is {v:.0} against a middle of {against:.0}"
        );
    }
}
