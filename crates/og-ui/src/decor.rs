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
}

impl BoxDecor {
    /// The draw it paints over: `base` (the element's draw or dressing)
    /// with its background, shadows and clip added; a plain rectangle
    /// when there is none.
    pub(crate) fn over(&self, base: Option<&ElementDraw>) -> ElementDraw {
        let mut d = base.cloned().unwrap_or_default();
        if let Some(bg) = &self.background {
            d.fills.insert(0, bg.clone());
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
        if self.clip && d.clip == crate::paint::Clip::None {
            d.clip = crate::paint::Clip::Shape;
        }
        d
    }

    /// Its border and outline, drawn where the element is placed.
    pub(crate) fn lines(&self, at: &Placed) -> Vec<DrawCmd> {
        let mut out = Vec::new();
        let r = at.rect;
        let uniform = self.borders.iter().all(|b| *b == self.borders[0]);
        match self.borders[0] {
            Some(b) if uniform && !b.style.shaded() => {
                ring(&mut out, at, r, at.radius, b, Inside);
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
            let radius = if at.radius > 0.0 {
                (at.radius + offset).max(0.0)
            } else {
                0.0
            };
            ring(&mut out, at, grown, radius, line, Outside);
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

/// A line of `line` round `rect`, inside or outside its edge.
fn ring(
    out: &mut Vec<DrawCmd>,
    at: &Placed,
    rect: Rect,
    radius: f32,
    line: Line,
    align: og_render::UiAlign,
) {
    let w = line.width;
    let (dash, cap) = match line.style {
        LineStyle::Dashed => ([3.0 * w, 3.0 * w], og_render::UiCap::Butt),
        LineStyle::Dotted => ([w, w], og_render::UiCap::Butt),
        _ => ([0.0; 2], og_render::UiCap::Butt),
    };
    let mut stroke = |rect: Rect, radius: f32, width: f32| {
        out.push(DrawCmd::Shape {
            shape: Box::new(og_render::UiShape {
                geometry: og_render::UiGeometry::Rect {
                    radii: [radius.max(0.0); 4],
                    corner: og_render::UiCorner::Round,
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
        stroke(rect, radius, third);
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
        stroke(
            inner,
            if radius > 0.0 {
                (radius - k).max(0.0)
            } else {
                0.0
            },
            third,
        );
    } else {
        stroke(rect, radius, w);
    }
}
