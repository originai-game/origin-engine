//! Paint geometry helpers: filter group bounds, image object-fit,
//! text decorations, quads, 2D affine transforms, gradients, and radii.

use super::*;

/// A filter group open while its subtree paints: its element, its
/// composite's place in the arena (written by [`close_group`]), its
/// effects in [`Paint::effects`], its element's transform, and its
/// composite's look but the rect.
pub(super) struct Open {
    pub(super) e: u32,
    pub(super) quad: usize,
    pub(super) first: u32,
    pub(super) count: u32,
    pub(super) xf: u32,
    pub(super) material: Option<og_draw::paint::PaintMaterial>,
    pub(super) composite: PaintQuad,
}

/// The layers a paint may still draw ([`og_draw::paint::GROUP_LAYERS`])
/// and the area they may cover (CSS px²).
pub(super) struct Room {
    pub(super) layers: usize,
    pub(super) area: f32,
    pub(super) limit: f32,
}

impl Room {
    /// Takes a layer of `area`, when there is room for it.
    pub(super) fn take(&mut self, area: f32) -> bool {
        if self.layers >= og_draw::paint::GROUP_LAYERS || self.area + area > self.limit {
            return false;
        }
        self.layers += 1;
        self.area += area;
        true
    }
}

/// How far `effects` spread what they filter, CSS px of the space they
/// are in: three deviations a blur, and a shadow's offset.
pub(super) fn reach_of(effects: &[PaintEffect]) -> f32 {
    effects
        .iter()
        .map(|f| match *f {
            PaintEffect::Blur(sigma) => 3.0 * sigma,
            PaintEffect::DropShadow { offset, sigma, .. } => {
                3.0 * sigma + (offset[0] * offset[0] + offset[1] * offset[1]).sqrt()
            }
            PaintEffect::Matrix(_) => 0.0,
        })
        .sum()
}

/// The area of the layer a box `rect` (x, y, w, h) filtered by `effects`
/// may take on `screen`: the box grown by their reach, on the screen
/// grown by it.
pub(super) fn reached(rect: [f32; 4], effects: &[PaintEffect], screen: [f32; 4]) -> f32 {
    let r = reach_of(effects);
    let [x, y, w, h] = rect;
    let w = (x + w + r).min(screen[2] + r) - (x - r).max(screen[0] - r);
    let h = (y + h + r).min(screen[3] + r) - (y - r).max(screen[1] - r);
    w.max(0.0) * h.max(0.0)
}

/// Whether element `e` is drawn under element `a` (its arena descendant).
pub(super) fn descends(els: &[El], e: u32, a: u32) -> bool {
    let mut c = els[e as usize].arena_parent;
    while c != NONE {
        if c == a {
            return true;
        }
        c = els[c as usize].arena_parent;
    }
    false
}

/// Ends group `o` once everything it holds has painted: its composite
/// covers the screen box of what it holds (each quad's reach, through its
/// transform, inside its clip) grown by its effects' reach (through its
/// element's transform), on what of `screen` it shows in grown as much,
/// and the group is listed. With `members` (no stacking reorder moves its
/// quads: the boxes of the groups the last one listed holds), a group
/// right after the last one listed, with the same effects, transform and
/// composite, and whose box meets none of those, joins it: one layer and
/// one filter run draw them all, as they would alone, since none's
/// effects reach another's box (a row of shadowed icons is one layer, not
/// one each).
pub(super) fn close_group(
    p: &mut Paint,
    o: &Open,
    screen: [f32; 4],
    members: Option<&mut Vec<[f32; 4]>>,
) {
    let mut bounds = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
    for q in &p.quads[o.quad + 1..] {
        let [x, y, w, h] = q.rect;
        // A shadow reaches past its box by its offset, spread and three
        // deviations; a line by its stroke; a box's edge by a pixel.
        let g = match q.kind {
            k if k == PaintKind::Shadow as u32 => {
                q.p[0].abs().max(q.p[1].abs()) + q.p[3].max(0.0) + 1.5 * q.p[2] + 1.0
            }
            k if k == PaintKind::Line as u32 => q.p[0] + 1.0,
            k if k == PaintKind::Group as u32
                || k == PaintKind::Image as u32
                || k == PaintKind::Raster as u32 =>
            {
                0.0
            }
            _ => 1.0,
        };
        let [ma, mb, mc, md, me, mf, _, _] = match q.xf {
            0 => og_draw::paint::IDENTITY,
            xf => p.transforms[xf as usize],
        };
        let mut b = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
        for [cx, cy] in [
            [x - g, y - g],
            [x + w + g, y - g],
            [x - g, y + h + g],
            [x + w + g, y + h + g],
        ] {
            let (sx, sy) = (ma * cx + mc * cy + me, mb * cx + md * cy + mf);
            b = [b[0].min(sx), b[1].min(sy), b[2].max(sx), b[3].max(sy)];
        }
        let c = q.clip;
        b = [
            b[0].max(c[0]),
            b[1].max(c[1]),
            b[2].min(c[2]),
            b[3].min(c[3]),
        ];
        if b[0] < b[2] && b[1] < b[3] {
            bounds = [
                bounds[0].min(b[0]),
                bounds[1].min(b[1]),
                bounds[2].max(b[2]),
                bounds[3].max(b[3]),
            ];
        }
    }
    let [ma, mb, mc, md, ..] = match o.xf {
        0 => og_draw::paint::IDENTITY,
        xf => p.transforms[xf as usize],
    };
    // The reach in screen px: the longer of the space's axes.
    let scale = (ma * ma + mb * mb).max(mc * mc + md * md).sqrt();
    let effects = &p.effects[o.first as usize..(o.first + o.count) as usize];
    let r = reach_of(effects) * scale;
    let c = o.composite.clip;
    let shown = [
        c[0].max(screen[0]) - r,
        c[1].max(screen[1]) - r,
        c[2].min(screen[2]) + r,
        c[3].min(screen[3]) + r,
    ];
    let x0 = (bounds[0] - r).max(shown[0]);
    let y0 = (bounds[1] - r).max(shown[1]);
    let x1 = (bounds[2] + r).min(shown[2]);
    let y1 = (bounds[3] + r).min(shown[3]);
    let rect = [x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0)];
    let end = p.quads.len() as u32;
    let mut members = members;
    if let Some(&last) = p.groups.last()
        && let Some(members) = members.as_deref_mut()
        && !last.backdrop
        && last.material.is_none()
        && o.material.is_none()
        && last.end == o.quad as u32
        && last.xf == o.xf
        && p.effects[last.first as usize..(last.first + last.count) as usize] == *effects
    {
        let at = last.quad as usize;
        let was = p.quads[at];
        let [ax, ay, aw, ah] = was.rect;
        // Apart when the boxes overlap by no more than a pixel: each box's
        // last pixel is its effects' last tap (3 deviations out, under 1%
        // of the middle's weight) on its content's anti-aliased edge. A
        // blur alone blurs its content too, so one layer differs from two
        // laid over each other by the product of their tails: under half
        // a code value (1/512) while the gap between their contents is 3.5
        // deviations (each tail under 0.044 at its middle), so each box
        // may reach 1.25 deviations into the other's.
        let overlap = 1.0
            + match *effects {
                [PaintEffect::Blur(sigma)] => 2.0 * 1.25 * sigma * scale,
                _ => 0.0,
            };
        let apart = members.iter().all(|&[mx, my, mw, mh]| {
            x1 <= mx + overlap
                || mx + mw <= x0 + overlap
                || y1 <= my + overlap
                || my + mh <= y0 + overlap
        });
        let same = PaintQuad {
            rect: was.rect,
            a: was.a,
            ..o.composite
        } == was;
        if apart && same {
            members.push(rect);
            let (ux, uy) = (ax.min(x0), ay.min(y0));
            p.quads[at].rect = [ux, uy, (ax + aw).max(x1) - ux, (ay + ah).max(y1) - uy];
            // Its own composite draws nothing in the layer they share.
            p.quads[o.quad] = PaintQuad {
                rect: [0.0; 4],
                color: [0.0; 4],
                kind: PaintKind::Solid as u32,
                ..o.composite
            };
            if let Some(g) = p.groups.last_mut() {
                g.end = end;
            }
            return;
        }
    }
    if let Some(members) = members {
        members.clear();
        members.push(rect);
    }
    p.groups.push(PaintGroup {
        quad: o.quad as u32,
        end,
        first: o.first,
        count: o.count,
        xf: o.xf,
        backdrop: false,
        material: o.material,
    });
    p.quads[o.quad] = PaintQuad {
        rect,
        a: p.groups.len() as u32 - 1,
        ..o.composite
    };
}

/// An image of `size` texels showing `uv` of it, fitted into `rect` as
/// CSS's `object-fit` and `object-position` place it: the part of the box
/// it covers, and the region of the texture drawn there. An unknown size
/// stretches it.
pub(super) fn fitted(
    (fit, at): (
        nsh_tailwind::ResizeMode,
        nsh_tailwind::style::ObjectPosition,
    ),
    rect: [f32; 4],
    uv: [f32; 4],
    size: [f32; 2],
) -> ([f32; 4], [f32; 4]) {
    use nsh_tailwind::ResizeMode as R;
    let [x, y, w, h] = rect;
    let natural = [
        size[0] * (uv[2] - uv[0]).abs(),
        size[1] * (uv[3] - uv[1]).abs(),
    ];
    if fit == R::Stretch || natural[0] <= 0.0 || natural[1] <= 0.0 || w <= 0.0 || h <= 0.0 {
        return (rect, uv);
    }
    let contain = (w / natural[0]).min(h / natural[1]);
    let scale = match fit {
        R::Contain => contain,
        R::Cover => (w / natural[0]).max(h / natural[1]),
        R::Center => 1.0,
        R::ScaleDown => contain.min(1.0),
        R::Stretch => unreachable!(),
    };
    let (dw, dh) = (natural[0] * scale, natural[1] * scale);
    let dx = at.x.resolve(nsh_tailwind::geometry::Points(w - dw)).0;
    let dy = at.y.resolve(nsh_tailwind::geometry::Points(h - dh)).0;
    // The drawn image's box, then the part of it inside the element's.
    let (ix0, iy0) = (x + dx, y + dy);
    let (x0, y0) = (ix0.max(x), iy0.max(y));
    let (x1, y1) = ((ix0 + dw).min(x + w), (iy0 + dh).min(y + h));
    let along = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let u = |px: f32| along(uv[0], uv[2], (px - ix0) / dw);
    let v = |py: f32| along(uv[1], uv[3], (py - iy0) / dh);
    (
        [x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0)],
        [u(x0), v(y0), u(x1), v(y1)],
    )
}

/// Paints the decoration through the existing solid/polyline primitives.
pub(super) fn decorate(
    p: &mut Paint,
    rect: [f32; 4],
    clip: [f32; 4],
    color: Rgba,
    style: nsh_tailwind::style::DecorationStyle,
) {
    use nsh_tailwind::style::DecorationStyle::*;
    let [x, y, width, th] = rect;
    let solid = |p: &mut Paint, r, radius| {
        p.quads.push(quad(
            PaintKind::Solid,
            r,
            clip,
            color,
            [radius; 4],
            [0.; 4],
            0,
            0,
        ));
    };
    match style {
        Solid => solid(p, rect, 0.),
        Double => {
            solid(p, [x, y - th, width, th], 0.);
            solid(p, [x, y + th, width, th], 0.);
        }
        Dotted | Dashed => {
            let length = if style == Dotted { th } else { 3. * th };
            let mut at = 0.;
            while at < width {
                solid(
                    p,
                    [x + at, y, length.min(width - at), th],
                    if style == Dotted { th / 2. } else { 0. },
                );
                at += length + th;
            }
        }
        Wavy => {
            let first = p.points.len() as u32;
            let steps = (width / th).ceil() as u32;
            if steps == 0 {
                return;
            }
            for k in 0..=steps {
                let at = width * k as f32 / steps as f32;
                p.points.push([
                    at + th,
                    2. * th + th * og_core::math::sin(at / (4. * th) * std::f32::consts::TAU),
                ]);
            }
            p.quads.push(quad(
                PaintKind::Line,
                [x - th, y - 1.5 * th, width + 2. * th, 4. * th],
                clip,
                color,
                [0.; 4],
                [th, 0., 0., 0.],
                first,
                steps + 1,
            ));
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn quad(
    kind: PaintKind,
    rect: [f32; 4],
    clip: [f32; 4],
    color: Rgba,
    radius: [f32; 4],
    p: [f32; 4],
    a: u32,
    b: u32,
) -> PaintQuad {
    PaintQuad {
        rect,
        clip,
        color,
        radius,
        p,
        kind: kind as u32,
        a,
        b,
        xf: 0,
        uv: [0.0; 4],
        wipe: [0.0; 4],
        mask: 0,
        filter: 0,
        round: 0,
        blend: 0,
    }
}

/// Adds the transform of a box `rect` (x, y, w, h) composed onto its
/// parent's `parent` to `p.transforms`: its index, or `parent` when its own
/// is the identity. CSS's order: move to the origin, translate (its
/// percentages of the box's own size), rotate, scale, move back.
pub(super) fn transformed(
    p: &mut Paint,
    parent: u32,
    t: &crate::record::Transform,
    rect: [f32; 4],
) -> u32 {
    if t.is_identity() {
        return parent;
    }
    if p.transforms.is_empty() {
        p.transforms.push(og_draw::paint::IDENTITY);
    }
    let [a, b, c, d, e, f] = affine(t, rect);
    let [pa, pb, pc, pd, pe, pf, _, _] = p.transforms[parent as usize];
    p.transforms.push([
        pa * a + pc * b,
        pb * a + pd * b,
        pa * c + pc * d,
        pb * c + pd * d,
        pa * e + pc * f + pe,
        pb * e + pd * f + pf,
        0.0,
        0.0,
    ]);
    p.transforms.len() as u32 - 1
}

/// `t` on a box at `rect` as a 2D affine map (a, b, c, d, e, f): a point
/// (x, y) goes to (a x + c y + e, b x + d y + f).
pub(in crate::tree) fn affine(t: &crate::record::Transform, rect: [f32; 4]) -> [f32; 6] {
    let (sin, cos) = og_core::math::sin_cos(t.rotate);
    let [sx, sy] = t.scale;
    let tan = |r: f32| {
        if r == 0.0 {
            0.0
        } else {
            let (s, c) = og_core::math::sin_cos(r);
            if c.abs() < 1e-6 { 0.0 } else { s / c }
        }
    };
    let (tx, ty) = (tan(t.skew[0]), tan(t.skew[1]));
    let (k00, k01, k10, k11) = ((1.0 + tx * ty) * sx, tx * sy, ty * sx, sy);
    let (a, b, c, d) = (
        cos * k00 - sin * k10,
        sin * k00 + cos * k10,
        cos * k01 - sin * k11,
        sin * k01 + cos * k11,
    );
    let ox = rect[0] + t.origin[0] * rect[2];
    let oy = rect[1] + t.origin[1] * rect[3];
    let e = ox + t.translate[0] + t.percent[0] * rect[2] - (a * ox + c * oy);
    let f = oy + t.translate[1] + t.percent[1] * rect[3] - (b * ox + d * oy);
    [a, b, c, d, e, f]
}

/// Corner radii (top left, top right, bottom right, bottom left) on a
/// `w`×`h` box, all scaled down by one factor when two on a side would
/// overlap, as CSS scales them.
/// Gradient `g` over the box `area` (x y w h, CSS px) as the paint list's:
/// a linear one at its angle (a `to <corner>` one aimed at the box's
/// corner), a radial one's ending shape sized by CSS Images 3's keywords, a
/// conic one from its angle; centres in CSS px from the box's centre.
pub(super) fn paint_gradient(
    g: &crate::record::Gradient,
    area: [f32; 4],
) -> og_draw::paint::PaintGradient {
    use crate::record::Shape;
    use nsh_tailwind::effects::GradientSize as Size;
    let [_, _, w, h] = area;
    // A `to <corner>` line runs perpendicular to the diagonal between the
    // other two corners, so its angle is the box's.
    let angle = match g.corner {
        [0, _] | [_, 0] => g.angle,
        [cx, cy] => og_core::math::atan2(f32::from(cx) * h, -f32::from(cy) * w),
    };
    let (cx, cy) = (g.center[0] * w, g.center[1] * h);
    let center = [cx - w / 2.0, cy - h / 2.0];
    // The distances from the centre to the nearer and the farther side.
    let (near, far) = (
        [cx.min(w - cx).abs(), cy.min(h - cy).abs()],
        [cx.max(w - cx).abs(), cy.max(h - cy).abs()],
    );
    let (shape, radii) = match g.shape {
        Shape::Linear => (og_draw::paint::GRADIENT_LINEAR, [0.0; 2]),
        Shape::Conic => (og_draw::paint::GRADIENT_CONIC, [0.0; 2]),
        Shape::Radial { circle, size } => {
            let radii = match (circle, size) {
                (true, Size::ClosestSide) => [near[0].min(near[1]); 2],
                (true, Size::FarthestSide) => [far[0].max(far[1]); 2],
                (true, Size::ClosestCorner) => [(near[0] * near[0] + near[1] * near[1]).sqrt(); 2],
                (true, Size::FarthestCorner) => [(far[0] * far[0] + far[1] * far[1]).sqrt(); 2],
                (false, Size::ClosestSide) => near,
                (false, Size::FarthestSide) => far,
                // The ellipse of the sides' proportions through the corner.
                (false, Size::ClosestCorner) => near.map(|r| r * std::f32::consts::SQRT_2),
                (false, Size::FarthestCorner) => far.map(|r| r * std::f32::consts::SQRT_2),
            };
            (og_draw::paint::GRADIENT_RADIAL, radii)
        }
    };
    og_draw::paint::PaintGradient {
        angle,
        first: 0,
        count: 0,
        shape,
        center,
        radii,
        area,
    }
}

pub(in crate::tree) fn radii(r: [f32; 4], w: f32, h: f32) -> [f32; 4] {
    let [tl, tr, br, bl] = r.map(|k| k.max(0.0));
    let fit = |side: f32, sum: f32| if sum > side { side / sum } else { 1.0 };
    let f = fit(w, tl + tr)
        .min(fit(w, bl + br))
        .min(fit(h, tl + bl))
        .min(fit(h, tr + br));
    [tl * f, tr * f, br * f, bl * f]
}

/// The right edge of the content box that set `e`'s `text-overflow`
/// (itself or the ancestor it inherits it from), where its text is cut;
/// `None` when that box was not placed this frame.
pub(super) fn overflow_edge(
    els: &[El],
    abs: &[(f32, f32, bool, [f32; 4])],
    motion: &crate::motion::Motion,
    arena: &Arena,
    e: u32,
) -> Option<f32> {
    let mut a = e;
    while els[a as usize].record.text.set & Text::OVERFLOW == 0 {
        a = els[a as usize].parent;
        if a == NONE {
            return None;
        }
    }
    let &(x, _, shown, _) = abs.get(a as usize)?;
    let node = els[a as usize].node;
    let b = motion.box_of(node).unwrap_or_else(|| arena.box_of(node));
    shown.then_some(x + b.w - arena.box_of(node).inset[2])
}
