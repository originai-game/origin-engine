use super::*;

fn peak(e0: f64, line: usize) -> f64 {
    let n = 640;
    let rows = emission_rows(e0, n);
    let k = (0..n)
        .max_by(|&a, &b| rows[a][line].total_cmp(&rows[b][line]))
        .unwrap();
    TABLE_BOTTOM + (k as f64 + 0.5) * (TABLE_TOP - TABLE_BOTTOM) / n as f64
}

fn column(e0: f64, line: usize) -> f64 {
    let n = 640;
    let dz = (TABLE_TOP - TABLE_BOTTOM) / n as f64;
    emission_rows(e0, n).iter().map(|r| r[line] * dz).sum()
}

/// The heights the lines are seen at (Whiter et al. 2023): for typical
/// precipitation the green and the blue lines peak near 114 km, harder
/// electrons reach lower, and the red line glows above 200 km in soft
/// aurora, much brighter against the green there than in hard aurora.
#[test]
fn the_lines_peak_where_they_are_observed() {
    let (green, blue) = (peak(2.0, 1), peak(2.0, 0));
    assert!((105.0..125.0).contains(&green), "green {green}");
    assert!((blue - green).abs() < 6.0, "blue {blue} green {green}");
    assert!(peak(10.0, 1) < green - 5.0, "hard {}", peak(10.0, 1));
    let red = peak(0.5, 2);
    assert!(red > 195.0, "red {red}");
    // The soft electrons that come with every arc keep a little red
    // over hard aurora too (Evans 1974), a fifth of the green's light
    // at most at 8 keV (the 630.0 nm line's efficiency counted).
    let soft = column(0.5, 2) / column(0.5, 1);
    let hard = column(8.0, 2) / column(8.0, 1);
    assert!(soft > 3.0 * hard, "red/green soft {soft} hard {hard}");
    assert!(hard * f64::from(EFFICIENCY_6300) < 0.25, "hard {hard}");
    // Below 95 km the green is quenched and the nitrogen bands (the
    // pink-violet lower border) outshine it relative to the peak.
    let rows = emission_rows(10.0, 640);
    let at = |z: f64| ((z - TABLE_BOTTOM) / (TABLE_TOP - TABLE_BOTTOM) * 640.0) as usize;
    let low = rows[at(90.0)];
    let mid = rows[at(115.0)];
    assert!(low[0] / low[1] > 1.5 * mid[0] / mid[1], "{low:?} {mid:?}");
}

/// An arc's lower edge is its sharpest: below the green peak the light
/// rises from a tenth to full within 10 km, and above it takes over
/// twice as far to fall to a fifth (the accelerated primaries stop
/// within a few kilometres of one depth).
#[test]
fn the_arcs_lower_edge_is_sharp() {
    let n = 640;
    let rows = emission_rows(2.0, n);
    let height = |k: usize| TABLE_BOTTOM + (k as f64 + 0.5) * (TABLE_TOP - TABLE_BOTTOM) / n as f64;
    let g = rows.iter().map(|r| r[1]).fold(0.0, f64::max);
    let k = rows.iter().position(|r| r[1] == g).expect("a peak");
    let below = (0..k)
        .rev()
        .find(|&j| rows[j][1] < 0.1 * g)
        .expect("an edge");
    let above = (k..n).find(|&j| rows[j][1] < 0.2 * g).expect("a top");
    let (rise, fall) = (height(k) - height(below), height(above) - height(k));
    assert!(rise < 10.0, "rise {rise} km");
    assert!(fall > 2.0 * rise, "rise {rise} km fall {fall} km");
}

/// The lines' colours: the green line green, the red one red, the
/// nitrogen bands violet to magenta (blue and red over green).
#[test]
fn the_lines_have_their_colours() {
    let [n2, green, red] = line_colors();
    assert!(
        green.y > 2.0 * green.x && green.y > 4.0 * green.z.max(0.01),
        "{green}"
    );
    assert!(red.x > 5.0 * red.y.max(0.01), "{red}");
    assert!(n2.z > n2.y && n2.x > n2.y, "{n2}");
}

/// The default colours are the lines' own.
#[test]
fn the_spectral_defaults_are_the_lines_colours() {
    let [n2, green, red] = line_colors();
    for (c, d) in [green, red, n2].iter().zip(Aurora::SPECTRAL) {
        let c = *c / c.max_element();
        assert!((c - d).abs().max_element() < 0.01, "{c} vs {d}");
    }
}

/// The sheet starts smooth and rolls up into curls: the later
/// snapshots vary more along the curtain than the first.
#[test]
fn the_strip_rolls_up() {
    let strip = kh_strip(1);
    let variation = |c: usize| {
        let row = STRIP_Y / 2 + 3;
        let v: Vec<f32> = (0..STRIP_X)
            .map(|x| f32::from(strip[row * STRIP_X + x][c]))
            .collect();
        let mean = v.iter().sum::<f32>() / v.len() as f32;
        v.iter().map(|x| (x - mean).abs()).sum::<f32>() / v.len() as f32
    };
    assert!(
        variation(2) > 4.0 * variation(0).max(1.0),
        "{} {}",
        variation(0),
        variation(2)
    );
    assert_eq!(strip, kh_strip(1), "deterministic");
}

/// The curtains are a function of the clock: the same at the same
/// second, the folds moved a second later, and the strip's coordinate
/// continuous along each curve.
#[test]
fn the_curtains_follow_the_clock() {
    let a = Aurora::default();
    let p = a.footprint(100.0, Vec2::ZERO);
    let q = a.footprint(100.0, Vec2::ZERO);
    assert_eq!(bytemuck::bytes_of(&p), bytemuck::bytes_of(&q));
    let r = a.footprint(101.0, Vec2::ZERO);
    assert_ne!(p.points[60], r.points[60]);
    for c in 0..3 {
        for j in 1..CURTAIN_POINTS {
            let (u0, u1) = (
                p.points[c * CURTAIN_POINTS + j - 1][2],
                p.points[c * CURTAIN_POINTS + j][2],
            );
            assert!((u1 - u0).abs() < 2.0, "curtain {c} point {j}: {u0} {u1}");
        }
    }
    // The first arc's middle lies `distance` out along `direction`
    // (90°: towards -z).
    let mid = p.points[CURTAIN_POINTS / 2];
    assert!(mid[1] < -100.0 && mid[0].abs() < 80.0, "{mid:?}");
}
