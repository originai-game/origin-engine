use super::*;

// ---- resolving against the box

impl CssGradient {
    /// The gradient over a `w` × `h` box.
    pub(crate) fn resolve(&self, w: f32, h: f32) -> Gradient {
        let (w, h) = (w.max(1e-3), h.max(1e-3));
        let (kind, from, to, length) = match self.shape {
            Shape::Linear(dir) => {
                let d = match dir {
                    Dir::Angle(a) => [a.sin(), -a.cos()],
                    // Perpendicular to the diagonal through the other two
                    // corners, so they sit on the 50% line.
                    Dir::Corner(x, y) => {
                        let v = [x * h, y * w];
                        let n = (v[0] * v[0] + v[1] * v[1]).sqrt();
                        [v[0] / n, v[1] / n]
                    }
                };
                let len = (w * d[0]).abs() + (h * d[1]).abs();
                let c = [w / 2.0, h / 2.0];
                (
                    GradientKind::Linear,
                    [(c[0] - d[0] * len / 2.0) / w, (c[1] - d[1] * len / 2.0) / h],
                    [(c[0] + d[0] * len / 2.0) / w, (c[1] + d[1] * len / 2.0) / h],
                    len,
                )
            }
            Shape::Radial { circle, size, at } => {
                let c = [at[0].at(w), at[1].at(h)];
                let (dx, dy) = (
                    [c[0].abs(), (w - c[0]).abs()],
                    [c[1].abs(), (h - c[1]).abs()],
                );
                let side = |far: bool| {
                    let pick = |a: [f32; 2]| if far { a[0].max(a[1]) } else { a[0].min(a[1]) };
                    (pick(dx), pick(dy))
                };
                let (rx, ry) = match size {
                    Extent::Given(x, y) => (x.at(w), y.at(h)),
                    Extent::ClosestSide | Extent::FarthestSide => {
                        let (x, y) = side(size == Extent::FarthestSide);
                        if circle {
                            let r = if size == Extent::FarthestSide {
                                x.max(y)
                            } else {
                                x.min(y)
                            };
                            (r, r)
                        } else {
                            (x, y)
                        }
                    }
                    Extent::ClosestCorner | Extent::FarthestCorner => {
                        let (x, y) = side(size == Extent::FarthestCorner);
                        if circle {
                            let r = x.hypot(y);
                            (r, r)
                        } else {
                            (x * SQRT_2, y * SQRT_2)
                        }
                    }
                };
                let (rx, ry) = (rx.max(1e-3), ry.max(1e-3));
                (
                    GradientKind::Radial,
                    [c[0] / w, c[1] / h],
                    [rx / w, ry / h],
                    rx,
                )
            }
            Shape::Conic { from, at } => (
                GradientKind::Conic,
                [at[0].at(w) / w, at[1].at(h) / h],
                [from - PI / 2.0, 0.0],
                1.0,
            ),
        };
        let mut offsets: Vec<Option<f32>> = self
            .stops
            .iter()
            .map(|(_, p)| p.map(|p| p.of(length) / length))
            .collect();
        fix_stops(&mut offsets);
        let offsets = offsets.into_iter().map(|o| o.unwrap_or(0.0));
        Gradient {
            kind,
            from,
            to,
            stops: self
                .stops
                .iter()
                .zip(offsets)
                .map(|((c, _), offset)| Stop {
                    offset,
                    color: c.clone(),
                })
                .collect(),
            spread: if self.repeating {
                Spread::RepeatStops
            } else {
                Spread::Pad
            },
            dither: true,
        }
    }
}

/// CSS Images 3 §3.5.3: an unpositioned first stop is at 0, last at 1;
/// a stop before an earlier one moves up to it; runs of unpositioned
/// stops share their span evenly.
fn fix_stops(o: &mut [Option<f32>]) {
    let n = o.len();
    if o[0].is_none() {
        o[0] = Some(0.0);
    }
    if o[n - 1].is_none() {
        o[n - 1] = Some(1.0);
    }
    let mut most = f32::MIN;
    for v in o.iter_mut().flatten() {
        most = most.max(*v);
        *v = most;
    }
    let mut k = 0;
    while k < n {
        if o[k].is_some() {
            k += 1;
            continue;
        }
        let a = k - 1;
        let b = (k..n).find(|&j| o[j].is_some()).unwrap();
        let (pa, pb) = (o[a].unwrap(), o[b].unwrap());
        for (j, v) in o.iter_mut().enumerate().take(b).skip(k) {
            *v = Some(pa + (pb - pa) * (j - a) as f32 / (b - a) as f32);
        }
        k = b;
    }
}

/// A cubic's controls for a quarter ellipse.
const KAPPA: f32 = 0.552_284_8;

/// A rounded rectangle's outline as straight and quarter-ellipse cubics
/// (a square corner is a straight cubic).
fn rrect(x0: f32, y0: f32, x1: f32, y1: f32, radius: f32) -> Vec<[f32; 2]> {
    let (x1, y1) = (x1.max(x0), y1.max(y0));
    let rr = radius.min((x1 - x0) / 2.0).min((y1 - y0) / 2.0).max(0.0);
    let k = rr * (1.0 - KAPPA);
    vec![
        [x0 + rr, y0],
        [x0 + rr, y0],
        [x1 - rr, y0],
        [x1 - rr, y0],
        [x1 - k, y0],
        [x1, y0 + k],
        [x1, y0 + rr],
        [x1, y0 + rr],
        [x1, y1 - rr],
        [x1, y1 - rr],
        [x1, y1 - k],
        [x1 - k, y1],
        [x1 - rr, y1],
        [x1 - rr, y1],
        [x0 + rr, y1],
        [x0 + rr, y1],
        [x0 + k, y1],
        [x0, y1 - k],
        [x0, y1 - rr],
        [x0, y1 - rr],
        [x0, y0 + rr],
        [x0, y0 + rr],
        [x0, y0 + k],
        [x0 + k, y0],
        [x0 + rr, y0],
    ]
}

impl CssClip {
    /// The shape over a `w` × `h` border box, as a filled path in CSS px
    /// from its top left: resolved over its reference box, `insets` from
    /// the border box (top, right, bottom, left), whose corners take
    /// `radius`.
    pub(crate) fn resolve(&self, w: f32, h: f32, (insets, radius): ([f32; 4], f32)) -> ShapeKind {
        let [t, r, b, l] = insets;
        let mut shape = self.local((w - l - r).max(0.0), (h - t - b).max(0.0), radius);
        if let ShapeKind::Path { points, .. } = &mut shape {
            for p in points {
                *p = [p[0] + l, p[1] + t];
            }
        }
        shape
    }

    /// The shape over a `w` × `h` reference box from its top left.
    fn local(&self, w: f32, h: f32, radius: f32) -> ShapeKind {
        let path = |contours: Vec<(Vec<[f32; 2]>, bool)>, fill_rule, curves| ShapeKind::Path {
            contours: contours.iter().map(|(p, _)| p.len() as u32).collect(),
            points: contours.into_iter().flat_map(|(p, _)| p).collect(),
            closed: true,
            filled: true,
            curves,
            fill_rule,
            width: 0.0,
            cap: Default::default(),
            join: Default::default(),
        };
        match self {
            CssClip::Inset([t, r, b, l], radius) => {
                let (x0, y0) = (l.at(w), t.at(h));
                let pts = rrect(x0, y0, w - r.at(w), h - b.at(h), *radius);
                path(vec![(pts, true)], FillRule::NonZero, true)
            }
            CssClip::Rect([t, r, b, l], radius) => {
                let at = |p: &Option<Pos>, whole: f32, none: f32| p.map_or(none, |p| p.at(whole));
                let pts = rrect(
                    at(l, w, 0.0),
                    at(t, h, 0.0),
                    at(r, w, w),
                    at(b, h, h),
                    *radius,
                );
                path(vec![(pts, true)], FillRule::NonZero, true)
            }
            CssClip::Xywh([x, y, ww, hh], radius) => {
                let (x0, y0) = (x.at(w), y.at(h));
                let pts = rrect(x0, y0, x0 + ww.at(w), y0 + hh.at(h), *radius);
                path(vec![(pts, true)], FillRule::NonZero, true)
            }
            CssClip::Box => path(
                vec![(rrect(0.0, 0.0, w, h, radius), true)],
                FillRule::NonZero,
                true,
            ),
            CssClip::Units(fill, contours) => {
                let scaled = contours
                    .iter()
                    .map(|(p, c)| (p.iter().map(|[x, y]| [x * w, y * h]).collect(), *c))
                    .collect();
                path(scaled, *fill, true)
            }
            CssClip::Shape(fill, cmds) => {
                // SVG path data with the lengths resolved over the box: a
                // pair's x of its width, y of its height; an arc's radii
                // likewise, its rotation and flags as written.
                let mut d = String::new();
                for (c, args) in cmds {
                    d.push(*c);
                    for (k, a) in args.iter().enumerate() {
                        let whole = match (c.to_ascii_uppercase(), k) {
                            ('H', _) => w,
                            ('V', _) => h,
                            ('A', 2..=4) => 0.0,
                            (_, k) if k % 2 == 0 => w,
                            _ => h,
                        };
                        d.push_str(&format!(" {}", a.at(whole)));
                    }
                    d.push(' ');
                }
                match crate::svg::Path::parse(&d) {
                    Ok(p) => path(super::build::svg::subpaths(&p), *fill, true),
                    Err(_) => path(Vec::new(), *fill, true),
                }
            }
            CssClip::Url(_) => path(Vec::new(), FillRule::NonZero, true),
            CssClip::Ellipse(r, circle, at) => {
                let c = [at[0].at(w), at[1].at(h)];
                let (cx, cy) = ([c[0], w - c[0]], [c[1], h - c[1]]);
                // CSS's default: closest-side.
                let side = |a: [f32; 2]| a[0].abs().min(a[1].abs());
                let (rx, ry) = if *circle {
                    let r = r[0].map_or(side(cx).min(side(cy)), |p| {
                        // A percentage of a circle's radius is of the box's
                        // normalised diagonal.
                        match p {
                            Pos::Frac(f) => f * w.hypot(h) / SQRT_2,
                            Pos::Px(v) => v,
                        }
                    });
                    (r, r)
                } else {
                    (
                        r[0].map_or(side(cx), |p| p.at(w)),
                        r[1].map_or(side(cy), |p| p.at(h)),
                    )
                };
                let (kx, ky) = (rx * KAPPA, ry * KAPPA);
                let [x, y] = c;
                let pts = vec![
                    [x + rx, y],
                    [x + rx, y + ky],
                    [x + kx, y + ry],
                    [x, y + ry],
                    [x - kx, y + ry],
                    [x - rx, y + ky],
                    [x - rx, y],
                    [x - rx, y - ky],
                    [x - kx, y - ry],
                    [x, y - ry],
                    [x + kx, y - ry],
                    [x + rx, y - ky],
                    [x + rx, y],
                ];
                path(vec![(pts, true)], FillRule::NonZero, true)
            }
            CssClip::Polygon(fill, points) => {
                // Straight cubics, so one path form carries every shape.
                let p: Vec<[f32; 2]> = points.iter().map(|[x, y]| [x.at(w), y.at(h)]).collect();
                let mut pts = vec![p[0]];
                for k in 1..=p.len() {
                    let (a, b) = (p[k - 1], p[k % p.len()]);
                    pts.extend([a, b, b]);
                }
                path(vec![(pts, true)], *fill, true)
            }
            CssClip::Path(fill, contours) => path(contours.clone(), *fill, true),
        }
    }
}
