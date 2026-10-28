//! Pixel-art rotation for `ArtPixels.rotation prerotated`: a port of
//! torcado's cleanEdge from a fragment shader to a sampler the cook calls
//! at each rotated texel's source point. The source is the shadertoy
//! shader of gist.github.com/torcado194/e2794f5a4b22049ac0a41f972d14c329,
//! with SLOPE and CLEANUP on, highestColor white, similarThreshold 0 and
//! lineWidth 1. MIT, Copyright (c) 2022 torcado: the notice is
//! third_party/cleanedge/LICENSE.
//!
//! It never blends: every texel it returns is one of the source's, so a
//! palette-mapped cell stays on its palette. Texels outside the cell read
//! as transparent.

type V4 = [f64; 4];

const HIGHEST: [f64; 3] = [1.0, 1.0, 1.0];
const LINE_WIDTH: f64 = 1.0;

fn dist4(a: V4, b: V4) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2) + (a[3] - b[3]).powi(2))
        .sqrt()
}

fn dist3(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// `similar` with similarThreshold 0: equal, or both transparent.
fn similar(a: V4, b: V4) -> bool {
    (a[3] == 0.0 && b[3] == 0.0) || dist4(a, b) <= 0.0
}

fn similar3(a: V4, b: V4, c: V4) -> bool {
    similar(a, b) && similar(b, c)
}

fn similar4(a: V4, b: V4, c: V4, d: V4) -> bool {
    similar(a, b) && similar(b, c) && similar(c, d)
}

fn higher(this: V4, other: V4) -> bool {
    if similar(this, other) {
        return false;
    }
    if this[3] == other[3] {
        dist3([this[0], this[1], this[2]], HIGHEST) < dist3([other[0], other[1], other[2]], HIGHEST)
    } else {
        this[3] > other[3]
    }
}

fn cd(a: V4, b: V4) -> f64 {
    dist4(a, b)
}

fn dist_to_line(test: [f64; 2], p1: [f64; 2], p2: [f64; 2], dir: [f64; 2]) -> f64 {
    let line = [p2[0] - p1[0], p2[1] - p1[1]];
    let perp = [line[1], -line[0]];
    let to_p1 = [p1[0] - test[0], p1[1] - test[1]];
    let len = (perp[0] * perp[0] + perp[1] * perp[1]).sqrt();
    let n = [perp[0] / len, perp[1] / len];
    let sign = if perp[0] * dir[0] + perp[1] * dir[1] > 0.0 {
        1.0
    } else {
        -1.0
    };
    sign * (n[0] * to_p1[0] + n[1] * to_p1[1])
}

fn at(center: [f64; 2], off: [f64; 2], pd: [f64; 2]) -> [f64; 2] {
    [center[0] + off[0] * pd[0], center[1] + off[1] * pd[1]]
}

/// The neighbours `sliceDist` reads, named as the shader names them.
#[derive(Clone, Copy)]
struct N {
    ub: V4,
    u: V4,
    uf: V4,
    uff: V4,
    b: V4,
    c: V4,
    f: V4,
    ff: V4,
    db: V4,
    d: V4,
    df: V4,
    dff: V4,
    ddb: V4,
    dd: V4,
    ddf: V4,
}

/// `sliceDist`: the colour a slice from a neighbour lays over the point,
/// or `None`.
fn slice_dist(point: [f64; 2], main_dir: [f64; 2], pd: [f64; 2], n: N) -> Option<V4> {
    let N {
        ub,
        u,
        uf,
        uff,
        b,
        c,
        f,
        ff,
        db,
        d,
        df,
        dff,
        ddb,
        dd,
        ddf,
    } = n;
    let (min_w, max_w) = (0.45, 1.142);
    let lw = LINE_WIDTH.clamp(min_w, max_w);
    let point = [
        main_dir[0] * (point[0] - 0.5) + 0.5,
        main_dir[1] * (point[1] - 0.5) + 0.5,
    ];

    // Edge detection.
    let against = 4.0 * cd(f, d) + cd(uf, c) + cd(c, db) + cd(ff, df) + cd(df, dd);
    let towards = 4.0 * cd(c, df) + cd(u, f) + cd(f, dff) + cd(b, d) + cd(d, ddf);
    let mut should = against < towards || (against < towards + 0.001 && !higher(c, f));
    if similar4(f, d, b, u) && similar4(uf, df, db, ub) && !similar(c, f) {
        should = false;
    }
    if !should {
        return None;
    }
    let mut flip = false;
    let center = [0.5, 0.5];
    let neg = [-pd[0], -pd[1]];
    let pick = |dist: f64, a: V4, x: V4, y: V4| -> Option<V4> {
        (dist <= 0.0).then(|| if cd(a, x) <= cd(a, y) { x } else { y })
    };

    if similar3(f, d, db) && !similar3(f, d, b) && !similar(uf, db) {
        // Lower shallow 2:1 slant.
        if similar(c, df) && higher(c, f) {
        } else {
            if higher(c, f) {
                flip = true;
            }
            if similar(u, f) && !similar(c, df) && !higher(c, u) {
                flip = true;
            }
        }
        let mut dist = if flip {
            lw - dist_to_line(
                point,
                at(center, [1.5, -1.0], pd),
                at(center, [-0.5, 0.0], pd),
                neg,
            )
        } else {
            dist_to_line(
                point,
                at(center, [1.5, 0.0], pd),
                at(center, [-0.5, 1.0], pd),
                pd,
            )
        };
        if !flip
            && similar(c, uf)
            && !(similar3(c, uf, uff) && !similar3(c, uf, ff) && !similar(d, uff))
        {
            let d2 = dist_to_line(
                point,
                at(center, [2.0, -1.0], pd),
                at(center, [0.0, 1.0], pd),
                pd,
            );
            dist = dist.min(d2);
        }
        dist -= lw / 2.0;
        return pick(dist, c, f, d);
    } else if similar3(uf, f, d) && !similar3(u, f, d) && !similar(uf, db) {
        // Forward steep 2:1 slant.
        if similar(c, df) && higher(c, d) {
        } else {
            if higher(c, d) {
                flip = true;
            }
            if similar(b, d) && !similar(c, df) && !higher(c, d) {
                flip = true;
            }
        }
        let mut dist = if flip {
            lw - dist_to_line(
                point,
                at(center, [0.0, -0.5], pd),
                at(center, [-1.0, 1.5], pd),
                neg,
            )
        } else {
            dist_to_line(
                point,
                at(center, [1.0, -0.5], pd),
                at(center, [0.0, 1.5], pd),
                pd,
            )
        };
        if !flip
            && similar(c, db)
            && !(similar3(c, db, ddb) && !similar3(c, db, dd) && !similar(f, ddb))
        {
            let d2 = dist_to_line(
                point,
                at(center, [1.0, 0.0], pd),
                at(center, [-1.0, 2.0], pd),
                pd,
            );
            dist = dist.min(d2);
        }
        dist -= lw / 2.0;
        return pick(dist, c, f, d);
    } else if similar(f, d) {
        // 45-degree diagonal.
        if similar(c, df) && higher(c, f) {
            if !similar(c, dd) && !similar(c, ff) {
                flip = true;
            }
        } else {
            if higher(c, f) {
                flip = true;
            }
            if !similar(c, b) && similar4(b, f, d, u) {
                flip = true;
            }
        }
        if ((similar(f, db) && similar3(u, f, df)) || (similar(uf, d) && similar3(b, d, df)))
            && !similar(c, df)
        {
            flip = true;
        }
        let mut dist = if flip {
            lw - dist_to_line(
                point,
                at(center, [1.0, -1.0], pd),
                at(center, [-1.0, 1.0], pd),
                neg,
            )
        } else {
            dist_to_line(
                point,
                at(center, [1.0, 0.0], pd),
                at(center, [0.0, 1.0], pd),
                pd,
            )
        };
        if !flip && similar3(c, uf, uff) && !similar3(c, uf, ff) && !similar(d, uff) {
            let d2 = dist_to_line(
                point,
                at(center, [1.5, 0.0], pd),
                at(center, [-0.5, 1.0], pd),
                pd,
            );
            dist = dist.max(d2);
        }
        if !flip && similar3(ddb, db, c) && !similar3(dd, db, c) && !similar(ddb, f) {
            let d2 = dist_to_line(
                point,
                at(center, [1.0, -0.5], pd),
                at(center, [0.0, 1.5], pd),
                pd,
            );
            dist = dist.max(d2);
        }
        dist -= lw / 2.0;
        return pick(dist, c, f, d);
    } else if similar3(ff, df, d) && !similar3(ff, df, c) && !similar(uff, d) {
        // Far corner of a shallow slant.
        if similar(f, dff) && higher(f, ff) {
        } else {
            if higher(f, ff) {
                flip = true;
            }
            if similar(uf, ff) && !similar(f, dff) && !higher(f, uf) {
                flip = true;
            }
        }
        let mut dist = if flip {
            lw - dist_to_line(
                point,
                at(center, [2.5, -1.0], pd),
                at(center, [0.5, 0.0], pd),
                neg,
            )
        } else {
            dist_to_line(
                point,
                at(center, [2.5, 0.0], pd),
                at(center, [0.5, 1.0], pd),
                pd,
            )
        };
        dist -= lw / 2.0;
        return pick(dist, f, ff, df);
    } else if similar3(f, df, dd) && !similar3(c, df, dd) && !similar(f, ddb) {
        // Far corner of a steep slant.
        if similar(d, ddf) && higher(d, dd) {
        } else {
            if higher(d, dd) {
                flip = true;
            }
            if similar(db, dd) && !similar(d, ddf) && !higher(d, dd) {
                flip = true;
            }
        }
        let mut dist = if flip {
            lw - dist_to_line(
                point,
                at(center, [0.0, 0.5], pd),
                at(center, [-1.0, 2.5], pd),
                neg,
            )
        } else {
            dist_to_line(
                point,
                at(center, [1.0, 0.5], pd),
                at(center, [0.0, 2.5], pd),
                pd,
            )
        };
        dist -= lw / 2.0;
        return pick(dist, d, df, dd);
    }
    None
}

/// The colour cleanEdge shows at the point `p` (texel units, x right and y
/// down from the cell's corner) of a `w` x `h` cell whose texels `tex`
/// reads (RGBA, 0..1).
pub fn sample(w: u32, h: u32, tex: &dyn Fn(u32, u32) -> V4, p: [f64; 2]) -> V4 {
    let local = [p[0] - p[0].floor(), p[1] - p[1].floor()];
    let px = [p[0].ceil() - 0.5, p[1].ceil() - 0.5];
    let pd = [local[0].round() * 2.0 - 1.0, local[1].round() * 2.0 - 1.0];
    let read = |dx: f64, dy: f64| -> V4 {
        let x = (px[0] + dx * pd[0]).floor();
        let y = (px[1] + dy * pd[1]).floor();
        if x < 0.0 || y < 0.0 || x >= f64::from(w) || y >= f64::from(h) {
            [0.0; 4]
        } else {
            tex(x as u32, y as u32)
        }
    };
    let (uub, uu, uuf) = (read(-1.0, -2.0), read(0.0, -2.0), read(1.0, -2.0));
    let (ubb, ub, u, uf, uff) = (
        read(-2.0, -2.0),
        read(-1.0, -1.0),
        read(0.0, -1.0),
        read(1.0, -1.0),
        read(2.0, -1.0),
    );
    let (bb, b, c, f, ff) = (
        read(-2.0, 0.0),
        read(-1.0, 0.0),
        read(0.0, 0.0),
        read(1.0, 0.0),
        read(2.0, 0.0),
    );
    let (dbb, db, d, df, dff) = (
        read(-2.0, 1.0),
        read(-1.0, 1.0),
        read(0.0, 1.0),
        read(1.0, 1.0),
        read(2.0, 1.0),
    );
    let (ddb, dd, ddf) = (read(-1.0, 2.0), read(0.0, 2.0), read(1.0, 2.0));
    // As the shader: the back slice's `uff` is `ubb`, two up and two back.

    let mut col = c;
    let c_col = slice_dist(
        local,
        [1.0, 1.0],
        pd,
        N {
            ub,
            u,
            uf,
            uff,
            b,
            c,
            f,
            ff,
            db,
            d,
            df,
            dff,
            ddb,
            dd,
            ddf,
        },
    );
    let b_col = slice_dist(
        local,
        [-1.0, 1.0],
        pd,
        N {
            ub: uf,
            u,
            uf: ub,
            uff: ubb,
            b: f,
            c,
            f: b,
            ff: bb,
            db: df,
            d,
            df: db,
            dff: dbb,
            ddb: ddf,
            dd,
            ddf: ddb,
        },
    );
    let u_col = slice_dist(
        local,
        [1.0, -1.0],
        pd,
        N {
            ub: db,
            u: d,
            uf: df,
            uff: dff,
            b,
            c,
            f,
            ff,
            db: ub,
            d: u,
            df: uf,
            dff: uff,
            ddb: uub,
            dd: uu,
            ddf: uuf,
        },
    );
    if let Some(x) = c_col {
        col = x;
    }
    if let Some(x) = b_col {
        col = x;
    }
    if let Some(x) = u_col {
        col = x;
    }
    col
}

/// `cell` turned by `degrees` counter-clockwise as seen (y down in the
/// image), drawn into an `ow` x `oh` cell about the centres of both.
pub fn rotate(
    w: u32,
    h: u32,
    tex: &dyn Fn(u32, u32) -> V4,
    degrees: f64,
    ow: u32,
    oh: u32,
) -> Vec<V4> {
    let r = degrees.to_radians();
    // Exact quarter turns.
    let (s, c) = match degrees.rem_euclid(360.0) {
        0.0 => (0.0, 1.0),
        90.0 => (1.0, 0.0),
        180.0 => (0.0, -1.0),
        270.0 => (-1.0, 0.0),
        _ => (r.sin(), r.cos()),
    };
    let (cx, cy) = (f64::from(ow) / 2.0, f64::from(oh) / 2.0);
    let (sx, sy) = (f64::from(w) / 2.0, f64::from(h) / 2.0);
    let mut out = Vec::with_capacity((ow * oh) as usize);
    for y in 0..oh {
        for x in 0..ow {
            // The output texel's centre, turned back into the source: a
            // counter-clockwise turn on screen (y down) is clockwise in
            // these coordinates, so its inverse turns the other way.
            let (dx, dy) = (f64::from(x) + 0.5 - cx, f64::from(y) + 0.5 - cy);
            let p = [c * dx - s * dy + sx, s * dx + c * dy + sy];
            out.push(sample(w, h, tex, p));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const K: V4 = [0.0, 0.0, 0.0, 1.0];
    const W: V4 = [1.0, 1.0, 1.0, 1.0];
    const T: V4 = [0.0; 4];

    #[test]
    fn a_quarter_turn_of_an_asymmetric_shape_is_exact() {
        // A 3 x 2 L: black top row, white below-left, clear below-right.
        let cell = [K, K, K, W, T, T];
        let tex = |x: u32, y: u32| cell[(y * 3 + x) as usize];
        // Unturned into its own size: itself.
        assert_eq!(rotate(3, 2, &tex, 0.0, 3, 2), cell.to_vec());
        // A counter-clockwise quarter turn: the top row becomes the left
        // column, read bottom to top; the white texel (0, 1) goes to the
        // bottom of the next column.
        let got = rotate(3, 2, &tex, 90.0, 2, 3);
        assert_eq!(got, vec![K, T, K, T, K, W]);
    }

    #[test]
    fn never_invents_a_colour() {
        let r: V4 = [1.0, 0.0, 0.0, 1.0];
        let b: V4 = [0.0, 0.0, 1.0, 1.0];
        let cell: Vec<V4> = (0..64)
            .map(|i| {
                if (i % 8) + (i / 8) < 8 {
                    r
                } else if i % 3 == 0 {
                    b
                } else {
                    T
                }
            })
            .collect();
        let tex = |x: u32, y: u32| cell[(y * 8 + x) as usize];
        for deg in [22.5, 45.0, 137.0, 300.0] {
            for p in rotate(8, 8, &tex, deg, 12, 12) {
                assert!(p == r || p == b || p[3] == 0.0, "{deg}: {p:?}");
            }
        }
    }
}
