//! A document element's CSS box decorations (`border`, `outline`,
//! `box-shadow`, `overflow: hidden`), resolved to colours and lengths when
//! the element builds and painted over whatever draws it: its own draw
//! ([`crate::Ui::set_draw`]), its style part's dressing, or its plain
//! background. Keeping them beside the draw, not in it, leaves a control's
//! dressing in place: a `<button>` with a border keeps its plate.
//!
//! Painting follows CSS Backgrounds and Borders 3: outer shadows under the
//! background, inner shadows over it, the border over both (inside the
//! box, which its width pads), the outline outside the box, taking no
//! room. The first shadow listed is drawn on top.

use crate::draw::DrawCmd;
use crate::layout::Rect;
use crate::paint::{ColorRef, Effect, ElementDraw, Paint, Placed};
use crate::style::Srgba;
use og_core::glam::Vec2;

/// How a border or outline line is drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LineStyle {
    Solid,
    Dashed,
    Dotted,
    Double,
    Inset,
    Outset,
    Groove,
    Ridge,
}

impl LineStyle {
    /// Whether its sides are shaded (inset, outset, groove, ridge).
    fn shaded(self) -> bool {
        matches!(
            self,
            LineStyle::Inset | LineStyle::Outset | LineStyle::Groove | LineStyle::Ridge
        )
    }

    /// A style word; `None` for `none` and `hidden` (nothing drawn).
    pub fn from_css(word: &str) -> Option<LineStyle> {
        Some(match word {
            "solid" => LineStyle::Solid,
            "dashed" => LineStyle::Dashed,
            "dotted" => LineStyle::Dotted,
            "double" => LineStyle::Double,
            "inset" => LineStyle::Inset,
            "outset" => LineStyle::Outset,
            "groove" => LineStyle::Groove,
            "ridge" => LineStyle::Ridge,
            _ => return None,
        })
    }
}

/// One border side, or the outline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line {
    pub width: f32,
    pub style: LineStyle,
    pub color: Srgba,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    pub inset: bool,
    pub offset: [f32; 2],
    pub blur: f32,
    pub spread: f32,
    pub color: Srgba,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BoxDecor {
    /// Top, right, bottom, left; `None` draws nothing there.
    pub borders: [Option<Line>; 4],
    /// The outline and its offset.
    pub outline: Option<(Line, f32)>,
    pub shadows: Vec<Shadow>,
    /// Its background, drawn here (under the inner shadows, over the
    /// outer ones) instead of as the element's own.
    pub background: Option<Paint>,
    /// `overflow: hidden` or `clip`: its children are cut to its shape.
    pub clip: bool,
    /// A `border-radius` written over a dressed part (a `<button>`): its
    /// rectangle takes this corner instead of the dressing's.
    pub radius: Option<f32>,
    /// Its CSS paint beyond these: gradients, mask, filters, blend, text
    /// shadows, clip path and material.
    pub css: Option<Box<crate::doc::paint_css::CssPaint>>,
    /// Per-corner, elliptical or % `border-radius`, or a `corner-shape`.
    pub corners: Option<Corners>,
    /// Its content box's edges in from its border box (its border and
    /// padding), left, top, right, bottom: a layer's `content-box`.
    pub content: [f32; 4],
}

/// CSS corners as written: each corner's horizontal and vertical radius
/// as (px, fraction of the box's side), and its `corner-shape` K.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Corners {
    pub radii: [[(f32, f32); 2]; 4],
    pub shape: [f32; 4],
}

/// Corners resolved for one box: horizontal radii and the rest.
pub(crate) type Resolved = ([f32; 4], og_render::UiCssCorners);

impl Corners {
    /// Its radii for a `w` × `h` box, scaled down together where two
    /// meeting along a side would overlap (CSS Backgrounds 3 §5.5).
    pub(crate) fn resolve(&self, w: f32, h: f32) -> Resolved {
        let rx = self.radii.map(|[x, _]| (x.0 + x.1 * w).max(0.0));
        let ry = self.radii.map(|[_, y]| (y.0 + y.1 * h).max(0.0));
        let ratio = |side: f32, a: f32, b: f32| if a + b > 0.0 { side / (a + b) } else { 1.0 };
        let f = ratio(w, rx[0], rx[1])
            .min(ratio(h, ry[1], ry[2]))
            .min(ratio(w, rx[2], rx[3]))
            .min(ratio(h, ry[3], ry[0]))
            .min(1.0);
        // A corner with a zero radius is square, whatever its shape.
        let fix = |r: f32| if r * f < 1e-4 { 0.0 } else { r * f };
        (
            rx.map(fix),
            og_render::UiCssCorners {
                vertical: ry.map(fix),
                shape: self.shape,
            },
        )
    }
}

/// `c` grown outwards by `k` CSS px (negative: inwards), as an outline's
/// or a shadow's spread follows the border's curve.
fn grow((h, c): Resolved, k: f32) -> Resolved {
    let g = |r: f32| if r > 0.0 { (r + k).max(0.0) } else { 0.0 };
    (
        h.map(g),
        og_render::UiCssCorners {
            vertical: c.vertical.map(g),
            shape: c.shape,
        },
    )
}

impl BoxDecor {
    /// Whether a layer is fixed to the viewport: moving the element moves
    /// it over the layer, so its last commands cannot be moved instead.
    pub(crate) fn fixed(&self) -> bool {
        self.css.as_ref().is_some_and(|c| {
            c.images
                .iter()
                .any(|l| l.attach == crate::doc::paint_css::Attach::Fixed)
        })
    }

    /// The draw it paints over: `base` (the element's draw or dressing)
    /// with its background, shadows and clip added; a plain rectangle
    /// when there is none.
    /// `size` is the placed box's, which a gradient's line and a clip
    /// path's percentages are resolved against.
    /// `frame` places its layers: its size, the viewport and its scroll.
    pub(crate) fn over(
        &self,
        base: Option<&ElementDraw>,
        frame: crate::doc::paint_css::LayerFrame,
    ) -> ElementDraw {
        let size = frame.size;
        let frame = crate::doc::paint_css::LayerFrame {
            borders: [3, 0, 1, 2].map(|k| self.borders[k].map_or(0.0, |b| b.width)),
            padding: self.content,
            ..frame
        };
        let mut d = base.cloned().unwrap_or_default();
        let layered = self.css.as_ref().is_some_and(|c| {
            !c.images.is_empty()
                || !c.masks.is_empty()
                || c.mask_border.is_some()
                || c.border_image.is_some()
        });
        match &self.background {
            Some(bg) if layered => {
                // The colour is the bottom layer, cut by the last layer's clip.
                let clip = self.css.as_ref().map(|c| c.color_clip).unwrap_or_default();
                let l = frame.colour(bg.clone(), clip);
                d.boxes.get_or_insert_default().backgrounds.push(l);
            }
            Some(bg) => d.fills.insert(0, bg.clone()),
            None => {}
        }
        if let Some(c) = &self.css {
            let [w, h] = size;
            if layered {
                let b = d.boxes.get_or_insert_default();
                // The first listed is on top: drawn last.
                b.backgrounds
                    .extend(c.images.iter().rev().map(|l| frame.layer(l)));
                b.isolate = c
                    .images
                    .iter()
                    .any(|l| l.blend != crate::paint::Blend::Normal);
                // The bottom of the group blends with nothing (a transparent
                // backdrop leaves the source, Compositing 1 §5.1).
                if let Some(first) = b.backgrounds.first_mut() {
                    first.blend = crate::paint::Blend::Normal;
                }
                // The bottom mask layer combines with nothing: its operator
                // is ignored (CSS Masking 1 §6.9).
                b.masks.extend(c.masks.iter().rev().map(|l| frame.layer(l)));
                if let Some(first) = b.masks.first_mut() {
                    first.blend = crate::paint::Blend::Normal;
                }
                // The mask border and the mask layers both cut it.
                if let Some(m) = &c.mask_border {
                    let mut l = frame.border(m);
                    if !b.masks.is_empty() {
                        l.blend = crate::paint::Blend::SourceIn;
                    }
                    b.masks.push(l);
                }
                b.border = c.border_image.as_ref().map(|i| frame.border(i));
            }
            d.effects.extend(c.effects.iter().cloned());
            d.filters.extend(c.filters.iter().cloned());
            if let Some(b) = c.blend {
                d.blend = b;
            }
            if !c.text_shadows.is_empty() {
                d.text.get_or_insert_default().shadows = c.text_shadows.clone();
            }
            if let Some(l) = &c.layer {
                d.layer = Some(l.clone());
            }
            if let Some(shape) = &c.clip {
                d.shape = shape.resolve(w, h);
                d.clip = crate::paint::Clip::Shape;
            }
        }
        // The first listed on top: drawn last.
        for s in self.shadows.iter().rev() {
            let color = ColorRef::Literal(s.color);
            d.effects.push(if s.inset {
                Effect::InnerShadow {
                    offset: s.offset,
                    blur: s.blur,
                    spread: s.spread,
                    color,
                }
            } else {
                Effect::DropShadow {
                    offset: s.offset,
                    blur: s.blur,
                    spread: s.spread,
                    color,
                }
            });
        }
        if let (Some(r), crate::paint::ShapeKind::Rect { radii, .. }) = (self.radius, &mut d.shape)
        {
            *radii = Some([r; 4]);
        }
        if let (Some(c), crate::paint::ShapeKind::Rect { radii, corner }) =
            (&self.corners, &mut d.shape)
        {
            let (h, css) = c.resolve(size[0], size[1]);
            *radii = Some(h);
            *corner = og_render::UiCorner::Css(css);
        }
        if self.clip && d.clip == crate::paint::Clip::None {
            d.clip = crate::paint::Clip::Shape;
        }
        d
    }

    /// Its border and outline, drawn where the element is placed.
    pub(crate) fn lines(&self, at: &Placed) -> Vec<DrawCmd> {
        let mut out = Vec::new();
        let r = at.rect;
        let corners = self
            .corners
            .map(|c| c.resolve(r.width, r.height))
            .unwrap_or(([at.radius; 4], round()));
        let uniform = self.borders.iter().all(|b| *b == self.borders[0]);
        // A border image is drawn instead of the border's lines (CSS
        // Backgrounds 3 §6), with the background.
        let imaged = self.css.as_ref().is_some_and(|c| c.border_image.is_some());
        match self.borders[0] {
            _ if imaged => {}
            Some(b) if uniform && !b.style.shaded() => {
                ring(&mut out, at, r, corners, b, Inside);
            }
            _ => self.sides(&mut out, at),
        }
        if let Some((line, offset)) = self.outline {
            let grown = Rect::new(
                r.x - offset,
                r.y - offset,
                r.width + 2.0 * offset,
                r.height + 2.0 * offset,
            );
            ring(&mut out, at, grown, grow(corners, offset), line, Outside);
        }
        out
    }

    /// Borders that differ by side (or are shaded): a strip each,
    /// square-cornered and solid; a groove or ridge is two half strips.
    fn sides(&self, out: &mut Vec<DrawCmd>, at: &Placed) {
        let r = at.rect;
        let w = |k: usize| self.borders[k].map_or(0.0, |b| b.width);
        let (t, rt, b, l) = (w(0), w(1), w(2), w(3));
        let strips = [
            Rect::new(r.x, r.y, r.width, t),
            Rect::new(r.right() - rt, r.y + t, rt, (r.height - t - b).max(0.0)),
            Rect::new(r.x, r.bottom() - b, r.width, b),
            Rect::new(r.x, r.y + t, l, (r.height - t - b).max(0.0)),
        ];
        for (k, strip) in strips.into_iter().enumerate() {
            let Some(line) = self.borders[k] else {
                continue;
            };
            // CSS's inset darkens the top and left, outset the others; a
            // groove is an inset outer half over an outset inner one, a
            // ridge the reverse.
            let top_left = k == 0 || k == 3;
            let (outer, inner) = match line.style {
                LineStyle::Inset => (top_left, top_left),
                LineStyle::Outset => (!top_left, !top_left),
                LineStyle::Groove => (top_left, !top_left),
                LineStyle::Ridge => (!top_left, top_left),
                _ => (false, false),
            };
            let halves = if outer == inner {
                vec![(strip, outer)]
            } else {
                let (o, i) = halve(strip, k);
                vec![(o, outer), (i, inner)]
            };
            for (rect, dark) in halves {
                let c = if dark {
                    line.color.mix(Srgba::BLACK, 0.5)
                } else {
                    line.color
                };
                let mut color = c.linear();
                color[3] *= at.alpha;
                out.push(DrawCmd::Quad {
                    rect,
                    radius: 0.0,
                    color,
                    clip: at.clip,
                });
            }
        }
    }
}

/// Side `k`'s strip split into its outer and inner halves.
fn halve(r: Rect, k: usize) -> (Rect, Rect) {
    match k {
        0 => (
            Rect::new(r.x, r.y, r.width, r.height / 2.0),
            Rect::new(r.x, r.y + r.height / 2.0, r.width, r.height / 2.0),
        ),
        2 => (
            Rect::new(r.x, r.y + r.height / 2.0, r.width, r.height / 2.0),
            Rect::new(r.x, r.y, r.width, r.height / 2.0),
        ),
        1 => (
            Rect::new(r.x + r.width / 2.0, r.y, r.width / 2.0, r.height),
            Rect::new(r.x, r.y, r.width / 2.0, r.height),
        ),
        _ => (
            Rect::new(r.x, r.y, r.width / 2.0, r.height),
            Rect::new(r.x + r.width / 2.0, r.y, r.width / 2.0, r.height),
        ),
    }
}

use og_render::UiAlign::{Inside, Outside};

/// Round corners of K 1, as `corner-shape: round` is.
fn round() -> og_render::UiCssCorners {
    og_render::UiCssCorners {
        vertical: [0.0; 4],
        shape: [1.0; 4],
    }
}

/// A line of `line` round `rect` (its corners `corners`), inside or
/// outside its edge.
fn ring(
    out: &mut Vec<DrawCmd>,
    at: &Placed,
    rect: Rect,
    corners: Resolved,
    line: Line,
    align: og_render::UiAlign,
) {
    // Plain round corners stay the circular geometry (its dashes follow
    // the arcs exactly); anything else is CSS's.
    let plain = corners.1.shape == [1.0; 4]
        && (corners.1.vertical == [0.0; 4] || corners.1.vertical == corners.0);
    let w = line.width;
    let (dash, cap) = match line.style {
        LineStyle::Dashed => ([3.0 * w, 3.0 * w], og_render::UiCap::Butt),
        LineStyle::Dotted => ([w, w], og_render::UiCap::Butt),
        _ => ([0.0; 2], og_render::UiCap::Butt),
    };
    let mut stroke = |rect: Rect, (radii, css): Resolved, width: f32| {
        out.push(DrawCmd::Shape {
            shape: Box::new(og_render::UiShape {
                geometry: og_render::UiGeometry::Rect {
                    radii: radii.map(|r| r.max(0.0)),
                    corner: if plain {
                        og_render::UiCorner::Round
                    } else {
                        og_render::UiCorner::Css(css)
                    },
                },
                center: Vec2::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0),
                half: Vec2::new(rect.width / 2.0, rect.height / 2.0),
                transform: [1.0, 0.0, 0.0, 1.0],
                paint: og_render::UiPaint::Solid(line.color.linear()),
                mode: og_render::UiMode::Stroke(og_render::UiStroke {
                    width,
                    align,
                    dash,
                    dash_offset: 0.0,
                    dash_cap: cap,
                }),
                opacity: at.alpha,
                blend: og_render::UiBlend::Normal,
                layer: 0,
                clip: None,
            }),
            clip: at.clip,
        });
    };
    if line.style == LineStyle::Double && w >= 3.0 {
        // Two lines a third as wide, a third apart (CSS's double).
        let third = w / 3.0;
        stroke(rect, corners, third);
        let k = if align == Inside {
            2.0 * third
        } else {
            -2.0 * third
        };
        let inner = Rect::new(
            rect.x + k,
            rect.y + k,
            rect.width - 2.0 * k,
            rect.height - 2.0 * k,
        );
        stroke(inner, grow(corners, -k), third);
    } else {
        stroke(rect, corners, w);
    }
}
