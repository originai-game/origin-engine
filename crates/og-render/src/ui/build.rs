//! The UI pass's CPU side: [`super::UiShape`]s, clips, gradients and vector
//! draws into the instance, gradient, clip and point tables the shaders read
//! (`shaders/ui_common.wgsl`).

use super::*;
use crate::RenderError;

/// `struct UiInstance` in ui_common.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct UiGpu {
    pub center_half: [f32; 4],
    pub xform: [f32; 4],
    pub geo: [f32; 4],
    pub geo2: [f32; 4],
    pub mode: [f32; 4],
    pub mode2: [f32; 4],
    pub color: [f32; 4],
    pub paint: [f32; 4],
    pub paint2: [f32; 4],
    pub paint3: [f32; 4],
    pub paint4: [f32; 4],
    pub clip: [f32; 4],
    pub kinds: [u32; 4],
    pub extra: [f32; 4],
}

/// `struct UiGradient` in ui_common.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GpuGradient {
    head: [f32; 4],
    geom: [f32; 4],
    bx: [f32; 4],
    offsets: [[f32; 4]; 2],
    colors: [[f32; 4]; UI_GRADIENT_STOPS],
}

/// `struct UiClipShape` in ui_common.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GpuClip {
    center_half: [f32; 4],
    inverse: [f32; 4],
    radii: [f32; 4],
    geo: [f32; 4],
    kinds: [f32; 4],
}

/// The tables one frame's UI reads (group 0, bindings 1 to 3).
#[derive(Default)]
pub(crate) struct UiTables {
    pub gradients: Vec<GpuGradient>,
    pub clips: Vec<GpuClip>,
    pub points: Vec<[f32; 4]>,
}

/// The pipelines of the shape family (src/ui/pass.rs).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ShapePipe {
    Normal,
    Multiply,
    Screen,
    Add,
    /// The blends that read the destination.
    Destination,
}

/// How a UI instance draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ShapeSource {
    Shape(ShapePipe, Option<TextureHandle>),
    Backdrop,
    /// A solid backdrop ([`UiGlass::solid`]): the backdrop's shader, which
    /// reads no pyramid for it.
    SolidBackdrop,
    /// A layer's composite.
    Composite(u32),
}

/// No clip: far outside any output.
pub(crate) const UNCLIPPED: [f32; 4] = [-1.0e9, -1.0e9, 1.0e9, 1.0e9];

pub(crate) const IDENTITY: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

/// A transform that maps nothing (zero or not finite) is the identity.
pub(crate) fn transform_of(m: [f32; 4]) -> [f32; 4] {
    let det = m[0] * m[3] - m[1] * m[2];
    if m.iter().all(|v| v.is_finite()) && det.abs() > 1e-12 {
        m
    } else {
        IDENTITY
    }
}

fn inverse(m: [f32; 4]) -> [f32; 4] {
    let [a, b, c, d] = m;
    let det = a * d - b * c;
    [d / det, -b / det, -c / det, a / det]
}

fn cap_code(c: UiCap) -> u32 {
    match c {
        UiCap::Butt => 0,
        UiCap::Round => 1,
        UiCap::Square => 2,
    }
}

/// A geometry's kind word (kind | corner << 4 | cap << 8 | join << 12),
/// its `geo` and `geo2`, for an area (a polyline's are made by
/// [`UiTables::polyline`]).
fn area(g: &UiGeometry) -> (u32, [f32; 4], [f32; 4]) {
    match g {
        UiGeometry::Rect { radii, corner } => {
            let (code, n) = match corner {
                UiCorner::Round => (0, 2.0),
                UiCorner::Squircle(n) => (1, n.max(1.0)),
                UiCorner::Bevel => (2, 2.0),
                UiCorner::Notch => (3, 2.0),
            };
            (code << 4, radii.map(|r| r.max(0.0)), [n, 0.0, 0.0, 0.0])
        }
        UiGeometry::Ellipse => (1, [0.0; 4], [0.0; 4]),
        UiGeometry::Arc {
            start,
            sweep,
            thickness,
            cap,
        } => (
            2 | cap_code(*cap) << 8,
            [*start, *sweep, thickness.max(0.0), 0.0],
            [0.0; 4],
        ),
        UiGeometry::Polygon {
            sides,
            inner,
            rounding,
            rotation,
        } => (
            3,
            [(*sides).max(3) as f32, *inner, rounding.max(0.0), *rotation],
            [0.0; 4],
        ),
        UiGeometry::Polyline { .. } => (4, [0.0; 4], [0.0; 4]),
    }
}

impl UiTables {
    /// Adds a gradient (`abs`: an absolute box in CSS px, else the shape's).
    pub fn gradient(&mut self, g: &UiGradient, abs: Option<[f32; 4]>) -> u32 {
        let mut stops: Vec<UiStop> = g
            .stops
            .iter()
            .take(UI_GRADIENT_STOPS)
            .map(|s| UiStop {
                offset: if s.offset.is_finite() {
                    s.offset.clamp(0.0, 1.0)
                } else {
                    0.0
                },
                color: s.color,
            })
            .collect();
        if stops.is_empty() {
            stops.push(UiStop {
                offset: 0.0,
                color: [0.0; 4],
            });
        }
        // Offsets never go back (CSS: a stop before the one above it sits on it).
        for i in 1..stops.len() {
            stops[i].offset = stops[i].offset.max(stops[i - 1].offset);
        }
        let mut out = GpuGradient {
            head: [
                match g.kind {
                    UiGradientKind::Linear => 0.0,
                    UiGradientKind::Radial => 1.0,
                    UiGradientKind::Conic => 2.0,
                    UiGradientKind::Diamond => 3.0,
                },
                stops.len() as f32,
                match g.spread {
                    UiSpread::Pad => 0.0,
                    UiSpread::Repeat => 1.0,
                    UiSpread::Reflect => 2.0,
                },
                f32::from(u8::from(g.dither)),
            ],
            geom: [g.from.x, g.from.y, g.to.x, g.to.y],
            bx: abs.unwrap_or([0.0; 4]),
            ..Default::default()
        };
        for (i, s) in stops.iter().enumerate() {
            out.offsets[i / 4][i % 4] = s.offset;
            let a = s.color[3].clamp(0.0, 1.0);
            let lab = oklab([
                s.color[0].max(0.0),
                s.color[1].max(0.0),
                s.color[2].max(0.0),
            ]);
            out.colors[i] = [lab[0] * a, lab[1] * a, lab[2] * a, a];
        }
        self.gradients.push(out);
        self.gradients.len() as u32 - 1
    }

    /// A solid colour as a one-stop gradient (a vector's flat paints).
    pub fn solid(&mut self, color: [f32; 4]) -> u32 {
        self.gradient(
            &UiGradient {
                stops: vec![UiStop { offset: 0.0, color }],
                dither: false,
                ..Default::default()
            },
            None,
        )
    }

    /// Adds a polyline's points relative to the middle of their bounds and
    /// returns the instance's centre offset (local), half extent, `geo` and
    /// `geo2`, and its kind word.
    #[allow(clippy::type_complexity)]
    fn polyline(
        &mut self,
        points: &[Vec2],
        closed: bool,
        width: f32,
        cap: UiCap,
        join: UiJoin,
    ) -> (Vec2, Vec2, [f32; 4], [f32; 4], u32) {
        let pts: Vec<Vec2> = points.iter().copied().filter(|p| p.is_finite()).collect();
        let (mut lo, mut hi) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
        for p in &pts {
            lo = lo.min(*p);
            hi = hi.max(*p);
        }
        if pts.is_empty() {
            lo = Vec2::ZERO;
            hi = Vec2::ZERO;
        }
        let w = width.max(0.0);
        let (join_code, limit) = match join {
            UiJoin::Miter(l) => (0, l.max(1.0)),
            UiJoin::Round => (1, 1.0),
            UiJoin::Bevel => (2, 1.0),
        };
        // The line reaches past its points by half its width, a miter up to
        // its limit.
        let reach = 0.5 * w * limit.max(1.5);
        let mid = (lo + hi) * 0.5;
        let half = (hi - lo) * 0.5 + Vec2::splat(reach);
        let first = self.points.len() as u32;
        let mut along = 0.0;
        for (i, p) in pts.iter().enumerate() {
            if i > 0 {
                along += p.distance(pts[i - 1]);
            }
            let q = *p - mid;
            self.points.push([q.x, q.y, along, 0.0]);
        }
        (
            mid,
            half,
            [
                first as f32,
                pts.len() as f32,
                f32::from(u8::from(closed)),
                0.0,
            ],
            [0.0, w, limit, 0.0],
            4 | cap_code(cap) << 8 | join_code << 12,
        )
    }

    /// Adds clip `c`'s shape; `parent` is its parent's record + 1 (0: none).
    pub fn clip(&mut self, c: &UiClip, parent: u32) -> u32 {
        let (word, geo, geo2) = area(&c.geometry);
        let kind = word & 15;
        let corner = (word >> 4) & 15;
        let m = transform_of(c.transform);
        self.clips.push(GpuClip {
            center_half: [c.center.x, c.center.y, c.half.x.max(0.0), c.half.y.max(0.0)],
            inverse: inverse(m),
            radii: geo,
            geo,
            kinds: [kind as f32, corner as f32, geo2[0], parent as f32],
        });
        self.clips.len() as u32
    }

    /// Adds occluder `o` over an overlay `size` CSS px; `parent` is the
    /// record + 1 of its rectangle's clip shape (0: none).
    pub fn occluder(&mut self, o: &UiOccluder, size: Vec2, parent: u32) -> u32 {
        self.clips.push(GpuClip {
            center_half: [o.at.x, o.at.y, size.x, size.y],
            inverse: IDENTITY,
            radii: [0.0; 4],
            geo: [
                o.depth,
                o.alpha.clamp(0.0, 1.0),
                o.bias.max(0.0),
                o.soft.max(1e-3),
            ],
            kinds: [5.0, 0.0, 0.0, parent as f32],
        });
        self.clips.len() as u32
    }
}

/// The clip shape head (record + 1, 0: none) of every clip rectangle of
/// `clip_count`, from the list's clips (a parent before its children is
/// not required).
pub(crate) fn clip_heads(
    list: &UiList,
    clip_count: usize,
    tables: &mut UiTables,
) -> Result<Vec<u32>, RenderError> {
    let mut heads = vec![0u32; clip_count];
    let mut records: Vec<Option<u32>> = vec![None; list.clips.len()];
    fn record(
        k: usize,
        list: &UiList,
        records: &mut Vec<Option<u32>>,
        tables: &mut UiTables,
        depth: u32,
    ) -> Result<u32, RenderError> {
        if let Some(r) = records[k] {
            return Ok(r);
        }
        if depth > 16 {
            return Err(RenderError::InvalidData(format!(
                "ui clip {k}: its parents go round"
            )));
        }
        let c = &list.clips[k];
        let parent = match c.parent {
            Some(p) if (p as usize) < list.clips.len() => {
                record(p as usize, list, records, tables, depth + 1)?
            }
            Some(p) => {
                return Err(RenderError::InvalidHandle(format!(
                    "ui clip {k}: parent {p} of {}",
                    list.clips.len()
                )));
            }
            None => 0,
        };
        let r = if matches!(c.geometry, UiGeometry::Polyline { .. }) {
            parent
        } else {
            tables.clip(c, parent)
        };
        records[k] = Some(r);
        Ok(r)
    }
    for k in 0..list.clips.len() {
        let r = record(k, list, &mut records, tables, 0)?;
        let rect = list.clips[k].rect as usize;
        let slot = heads.get_mut(rect).ok_or_else(|| {
            RenderError::InvalidHandle(format!(
                "ui clip {k}: clip rectangle {rect} of {clip_count}"
            ))
        })?;
        *slot = r;
    }
    Ok(heads)
}

/// One shape's instance and how it draws. `clip`: its clip rectangle
/// (x0, y0, x1, y1) and shape head.
pub(crate) fn shape(
    i: usize,
    s: &UiShape,
    clip: ([f32; 4], u32),
    tables: &mut UiTables,
    texture_count: usize,
    layer_count: usize,
) -> Result<(UiGpu, ShapeSource), RenderError> {
    let m = transform_of(s.transform);
    let mut g = UiGpu {
        center_half: [s.center.x, s.center.y, s.half.x.max(0.0), s.half.y.max(0.0)],
        xform: m,
        clip: clip.0,
        extra: [s.opacity.clamp(0.0, 1.0), 0.0, 0.0, s.layer as f32],
        ..Default::default()
    };
    g.kinds[3] = clip.1;
    match &s.geometry {
        UiGeometry::Polyline {
            points,
            closed,
            width,
            cap,
            join,
        } => {
            let (mid, half, geo, geo2, word) =
                tables.polyline(points, *closed, *width, *cap, *join);
            // The local origin moves to the points' middle.
            let c = s.center + Vec2::new(m[0] * mid.x + m[2] * mid.y, m[1] * mid.x + m[3] * mid.y);
            g.center_half = [c.x, c.y, half.x, half.y];
            g.geo = geo;
            g.geo2 = geo2;
            g.kinds[0] = word;
        }
        other => {
            let (word, geo, geo2) = area(other);
            g.geo = geo;
            g.geo2 = geo2;
            g.kinds[0] = word;
        }
    }
    let (mode, mode_word) = match s.mode {
        UiMode::Fill => ([0.0; 4], 0),
        UiMode::Stroke(st) => {
            let w = st.width.max(0.0);
            let offset = match st.align {
                UiAlign::Inside => -0.5 * w,
                UiAlign::Center => 0.0,
                UiAlign::Outside => 0.5 * w,
            };
            g.mode2 = [st.dash_offset, cap_code(st.dash_cap) as f32, 0.0, 0.0];
            let dash = if st.dash[0] > 0.0 { st.dash[0] } else { 0.0 };
            ([w, offset, dash, st.dash[1].max(0.0)], 1)
        }
        UiMode::Shadow {
            offset,
            blur,
            spread,
        } => ([blur.max(0.0), spread, offset.x, offset.y], 2),
        UiMode::InnerShadow {
            offset,
            blur,
            spread,
        } => ([blur.max(0.0), spread, offset.x, offset.y], 3),
        UiMode::Backdrop {
            blur,
            saturation,
            brightness,
            glass,
        } => {
            // The material (ui_common.wgsl's UiInstance): the hairline in
            // mode.w and mode2, the shadows in paint to paint4 (a solid
            // paint's free fields), the flags in extra.z.
            g.mode2 = glass.edge_color;
            for (k, sh) in glass.shadows.iter().enumerate() {
                let [a, b] = if k == 0 {
                    [&mut g.paint, &mut g.paint2]
                } else {
                    [&mut g.paint3, &mut g.paint4]
                };
                *a = [sh.offset.x, sh.offset.y, sh.blur.max(0.0), sh.spread];
                *b = sh.color;
            }
            g.extra[2] = f32::from(u8::from(glass.solid) | u8::from(glass.srgb_mix) << 1);
            (
                [blur.max(0.0), saturation, brightness, glass.edge.max(0.0)],
                4,
            )
        }
    };
    g.mode = mode;
    let blend = match s.blend {
        UiBlend::Normal => 0,
        UiBlend::Multiply => 1,
        UiBlend::Screen => 2,
        UiBlend::Add => 3,
        UiBlend::Overlay => 4,
        UiBlend::SoftLight => 5,
        UiBlend::ColorDodge => 6,
    };
    g.kinds[2] = mode_word | blend << 4;
    let mut texture = None;
    let mut composite = None;
    match &s.paint {
        UiPaint::Solid(c) => {
            g.color = *c;
            g.kinds[1] = 0;
        }
        UiPaint::Gradient(gr) => {
            g.color = [1.0; 4];
            g.extra[1] = tables.gradient(gr, None) as f32;
            g.kinds[1] = 1;
        }
        UiPaint::Image(im) => {
            if im.texture.index() >= texture_count {
                return Err(RenderError::InvalidHandle(format!(
                    "ui shape {i}: texture {}",
                    im.texture.index()
                )));
            }
            texture = Some(im.texture);
            g.color = im.tint;
            g.paint = im.uv;
            g.paint2 = [
                im.focal.x,
                im.focal.y,
                im.size.x.max(1.0),
                im.size.y.max(1.0),
            ];
            let (fit, edges, center) = match im.fit {
                UiFit::Fill => (0, 0, 0),
                UiFit::Cover => (1, 0, 0),
                UiFit::Contain => (2, 0, 0),
                UiFit::None => (3, 0, 0),
                UiFit::NineSlice {
                    source,
                    dest,
                    edges,
                    center,
                } => {
                    g.paint3 = source.map(|v| v.max(0.0));
                    g.paint4 = dest.map(|v| v.max(0.0));
                    let e = match edges {
                        UiEdge::Stretch => 0,
                        UiEdge::Repeat => 1,
                        UiEdge::Round => 2,
                    };
                    (4, e, u32::from(center))
                }
                UiFit::Pattern {
                    tile,
                    rotation,
                    offset,
                    scroll,
                } => {
                    g.paint2[0] = tile.x;
                    g.paint2[1] = tile.y;
                    g.paint3 = [offset.x, offset.y, scroll.x, scroll.y];
                    g.geo2[3] = rotation;
                    (5, 0, 0)
                }
            };
            g.kinds[1] = 2 | fit << 4 | edges << 8 | center << 12;
        }
        UiPaint::Layer(k) => {
            if *k as usize >= layer_count {
                return Err(RenderError::InvalidHandle(format!(
                    "ui shape {i}: layer {k} of {layer_count}"
                )));
            }
            g.color = [1.0; 4];
            g.kinds[1] = 3;
            composite = Some(*k);
        }
    }
    let source = if let Some(k) = composite {
        ShapeSource::Composite(k)
    } else if let UiMode::Backdrop { glass, .. } = s.mode {
        if glass.solid {
            ShapeSource::SolidBackdrop
        } else {
            ShapeSource::Backdrop
        }
    } else if s.blend.reads_destination() {
        ShapeSource::Shape(ShapePipe::Destination, texture)
    } else {
        let pipe = match s.blend {
            UiBlend::Multiply => ShapePipe::Multiply,
            UiBlend::Screen => ShapePipe::Screen,
            UiBlend::Add => ShapePipe::Add,
            _ => ShapePipe::Normal,
        };
        ShapeSource::Shape(pipe, texture)
    };
    Ok((g, source))
}

/// A vector's fitted scale and the box it fills: the view box contained
/// and centred in the draw's rectangle.
pub(crate) fn vector_fit(view_box: [f32; 4], rect: [f32; 4]) -> (f32, [f32; 4]) {
    let k = (rect[2] / view_box[2].max(1e-6)).min(rect[3] / view_box[3].max(1e-6));
    let (w, h) = (view_box[2] * k, view_box[3] * k);
    (
        k,
        [
            rect[0] + 0.5 * (rect[2] - w),
            rect[1] + 0.5 * (rect[3] - h),
            w,
            h,
        ],
    )
}

/// A vector's paint as the gradient table takes it: its `from` and `to`
/// out of view box units into fractions of the view box.
pub(crate) fn vector_gradient(g: &UiGradient, vb: [f32; 4]) -> UiGradient {
    let size = Vec2::new(vb[2].max(1e-6), vb[3].max(1e-6));
    let origin = Vec2::new(vb[0], vb[1]);
    let mut out = g.clone();
    out.from = (g.from - origin) / size;
    out.to = match g.kind {
        UiGradientKind::Linear => (g.to - origin) / size,
        UiGradientKind::Radial | UiGradientKind::Diamond => g.to / size,
        UiGradientKind::Conic => g.to,
    };
    out
}

/// The polyline shapes of a vector draw's strokes, in its local space,
/// each with the box (CSS px) its gradient spans: the fitted view box.
pub(crate) fn vector_strokes(d: &UiVectorDraw, data: &UiVectorData) -> Vec<(UiShape, [f32; 4])> {
    let (k, fitted) = vector_fit(data.view_box, d.rect);
    let vb = data.view_box;
    let vb_mid = Vec2::new(vb[0] + 0.5 * vb[2], vb[1] + 0.5 * vb[3]);
    let center = Vec2::new(d.rect[0] + 0.5 * d.rect[2], d.rect[1] + 0.5 * d.rect[3]);
    let t = d.tint;
    data.strokes
        .iter()
        .map(|st| {
            let paint = match data.paints.get(st.paint as usize) {
                Some(UiVectorPaint::Gradient(g)) => UiPaint::Gradient(vector_gradient(g, vb)),
                Some(UiVectorPaint::Solid(c)) => {
                    UiPaint::Solid([c[0] * t[0], c[1] * t[1], c[2] * t[2], c[3] * t[3]])
                }
                None => UiPaint::Solid([0.0, 0.0, 0.0, t[3]]),
            };
            let shape = UiShape {
                geometry: UiGeometry::Polyline {
                    points: st.points.iter().map(|p| (*p - vb_mid) * k).collect(),
                    closed: st.closed,
                    width: st.width * k,
                    cap: st.cap,
                    join: st.join,
                },
                center,
                half: Vec2::new(fitted[2], fitted[3]) * 0.5,
                transform: d.transform,
                paint,
                mode: if st.dash[0] > 0.0 {
                    UiMode::Stroke(UiStroke {
                        width: st.width * k,
                        dash: [st.dash[0] * k, st.dash[1] * k],
                        dash_cap: st.cap,
                        ..Default::default()
                    })
                } else {
                    UiMode::Fill
                },
                opacity: 1.0,
                blend: UiBlend::Normal,
                layer: d.layer,
                clip: d.clip,
            };
            (shape, fitted)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gradient_stops_are_clamped_ordered_and_premultiplied_in_oklab() {
        let mut t = UiTables::default();
        let g = UiGradient {
            stops: vec![
                UiStop {
                    offset: 0.6,
                    color: [1.0, 0.0, 0.0, 0.5],
                },
                UiStop {
                    offset: 0.2,
                    color: [0.0, 0.0, 1.0, 1.0],
                },
                UiStop {
                    offset: 7.0,
                    color: [1.0, 1.0, 1.0, 1.0],
                },
            ],
            ..Default::default()
        };
        let i = t.gradient(&g, None) as usize;
        let r = &t.gradients[i];
        assert_eq!(r.head[1], 3.0);
        // The second stop never goes back past the first; the last is at 1.
        assert_eq!(r.offsets[0][..3], [0.6, 0.6, 1.0]);
        // Red at half alpha: OKLab premultiplied by it.
        let lab = oklab([1.0, 0.0, 0.0]);
        assert!((r.colors[0][0] - lab[0] * 0.5).abs() < 1e-6);
        assert_eq!(r.colors[0][3], 0.5);
    }

    #[test]
    fn a_polyline_is_centred_on_its_bounds_with_lengths_along_it() {
        let mut t = UiTables::default();
        let s = UiShape {
            geometry: UiGeometry::Polyline {
                points: vec![
                    Vec2::new(0.0, 0.0),
                    Vec2::new(30.0, 0.0),
                    Vec2::new(30.0, 40.0),
                ],
                closed: false,
                width: 4.0,
                cap: UiCap::Round,
                join: UiJoin::Round,
            },
            center: Vec2::new(100.0, 100.0),
            ..UiShape::rect([0.0; 4], 0.0, [1.0; 4])
        };
        let (g, src) = shape(0, &s, (UNCLIPPED, 0), &mut t, 0, 0).expect("a shape");
        assert_eq!(src, ShapeSource::Shape(ShapePipe::Normal, None));
        assert_eq!(&g.center_half[..2], &[115.0, 120.0]);
        assert_eq!(t.points.len(), 3);
        assert_eq!(t.points[0], [-15.0, -20.0, 0.0, 0.0]);
        assert_eq!(t.points[2][2], 70.0);
        assert_eq!(g.kinds[0] & 15, 4);
    }

    #[test]
    fn clips_chain_to_their_parents_and_name_their_rectangles() {
        let mut t = UiTables::default();
        let rect = |r: u32, parent| UiClip {
            rect: r,
            geometry: UiGeometry::Rect {
                radii: [8.0; 4],
                corner: UiCorner::Round,
            },
            center: Vec2::ZERO,
            half: Vec2::splat(10.0),
            transform: [1.0, 0.0, 0.0, 1.0],
            parent,
        };
        // The child before its parent.
        let list = UiList {
            clips: vec![rect(2, Some(1)), rect(0, None)],
            ..Default::default()
        };
        let heads = clip_heads(&list, 3, &mut t).expect("heads");
        assert_eq!(t.clips.len(), 2);
        assert_eq!(heads[1], 0);
        // Rectangle 0 is the parent's record, rectangle 2 the child's,
        // which names the parent's.
        let parent = heads[0];
        let child = heads[2];
        assert_eq!(t.clips[child as usize - 1].kinds[3], parent as f32);
        let cyclic = UiList {
            clips: vec![rect(0, Some(1)), rect(1, Some(0))],
            ..Default::default()
        };
        assert!(clip_heads(&cyclic, 2, &mut UiTables::default()).is_err());
    }
}
