//! Ruby (CSS Ruby 1): a base and its annotation (HTML's `<ruby>` and
//! `<rt>`) as one unbreakable inline box of a rich text. The annotation is
//! set at half the base's size (HTML's `rt { font-size: 50% }`, with the
//! font's `ruby` forms) over the base, or under it (`ruby-position`); in a
//! vertical line on its line-over side (the right, but the left in
//! `sideways-lr`) or its line-under one. Base and annotation share one
//! column as long as the longer of them; `ruby-align` spreads the shorter
//! along it (§4.3). An annotation longer than its base hangs over the text
//! beside it by up to one of its characters on each side under
//! `ruby-overhang: auto` (§5.1), never over another ruby. The annotation
//! takes no room in its line while it fits its line's leading; where the
//! line is too tight to hold it, the line grows on its side by what it
//! lacks (§3.6).

use super::*;
use crate::layout::flow::{VerticalText, WritingMode};

/// CSS `ruby-align` (CSS Ruby 1 §4.3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RubyAlign {
    Start,
    Center,
    SpaceBetween,
    #[default]
    SpaceAround,
}

/// CSS `ruby-overhang` (CSS Ruby 1 §5.1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RubyOverhang {
    /// Over any text beside it but another ruby.
    #[default]
    Auto,
    /// Over nothing.
    None,
    /// Over a space beside it only.
    Spaces,
}

/// A text's ruby properties (inherited): its annotations' side, how base
/// and annotation share their column, and how far annotations hang.
/// `ruby-position: alternate` (the initial value) sets a single level
/// over; `inter-character` is read as `over`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RubyStyle {
    pub under: bool,
    pub align: RubyAlign,
    pub overhang: RubyOverhang,
}

impl RubyStyle {
    /// The style computed `ruby-position`, `ruby-align` and
    /// `ruby-overhang` give (each its initial value when `None`).
    pub(crate) fn parse(
        position: Option<&str>,
        align: Option<&str>,
        overhang: Option<&str>,
    ) -> RubyStyle {
        RubyStyle {
            under: position.is_some_and(|p| p.split_whitespace().any(|w| w == "under")),
            align: match align {
                Some("start") => RubyAlign::Start,
                Some("center") => RubyAlign::Center,
                Some("space-between") => RubyAlign::SpaceBetween,
                _ => RubyAlign::SpaceAround,
            },
            overhang: match overhang {
                Some("none") => RubyOverhang::None,
                Some("spaces") => RubyOverhang::Spaces,
                _ => RubyOverhang::Auto,
            },
        }
    }
}

/// A rich span's annotation: the span's text is its base.
#[derive(Clone, Debug, PartialEq)]
pub struct Ruby {
    /// The annotation (`<rt>`).
    pub text: String,
    pub under: bool,
    pub align: RubyAlign,
    /// Whether it may hang over what comes before and after it.
    pub overhang: [bool; 2],
}

/// An annotation's size, of its base's.
pub(crate) const RUBY_SIZE: f32 = 0.5;

/// A ruby along its line, px: the room it takes there, and its column
/// (where it starts from that room's start, before it when the annotation
/// hangs over the text before; and how long it is).
pub(crate) struct RubyBox {
    pub advance: f32,
    pub column: f32,
    pub length: f32,
}

/// `text`'s clusters and each one's length along its line (`v`: down a
/// vertical one) at `size`.
fn clusters(
    s: &RichSpan,
    text: &str,
    size: f32,
    tracking: f32,
    fonts: &dyn Metrics,
    v: Option<VerticalText>,
) -> Vec<(String, f32)> {
    let m = fonts.metrics_styled(s.font, s.query(), text);
    let chars: Vec<char> = text.chars().collect();
    og_i18n::graphemes(text)
        .into_iter()
        .map(|(a, b)| {
            let c: String = chars[a..b].iter().collect();
            let len = match v {
                Some(v) => crate::text::vertical::measure(
                    m,
                    &c,
                    size,
                    tracking,
                    crate::shape::Features::NONE,
                    v,
                ),
                None => m.measure_tracked(&c, size, tracking),
            };
            (c, len)
        })
        .collect()
}

fn length(clusters: &[(String, f32)]) -> f32 {
    clusters.iter().map(|c| c.1).sum()
}

/// Ruby `r` of span `s` at its base's `size` (px) along its line.
pub(crate) fn ruby_box(
    s: &RichSpan,
    r: &Ruby,
    size: f32,
    fonts: &dyn Metrics,
    v: Option<VerticalText>,
) -> RubyBox {
    let a = size * RUBY_SIZE;
    let base = length(&clusters(
        s,
        &s.text,
        size,
        span_tracking(s, &s.text),
        fonts,
        v,
    ));
    let annot = length(&clusters(s, &r.text, a, 0.0, fonts, v));
    let excess = (annot - base).max(0.0);
    // Up to one annotation character over each side it may hang over.
    let hang = |k: usize| {
        if r.overhang[k] {
            (excess / 2.0).min(a)
        } else {
            0.0
        }
    };
    let (before, after) = (hang(0), hang(1));
    let length = base.max(annot);
    RubyBox {
        advance: length - before - after,
        column: -before,
        length,
    }
}

/// The over and under edges of `text`'s content area in `m`, em above
/// and below its baseline (positive): its faces' ascent and descent, the
/// widest of a mixed text's (CSS Inline 3 §5.1). Ruby and emphasis marks
/// sit on these.
pub(crate) fn content_edges(m: &crate::text::FontMetrics, text: &str) -> (f32, f32) {
    let own = (m.over_edge(), m.under_edge());
    if m.runs.is_none() {
        return own;
    }
    m.font_runs(text)
        .iter()
        .filter_map(|(_, k)| m.part(*k))
        .fold(own, |(a, d), p| {
            (a.max(p.over_edge()), d.max(p.under_edge()))
        })
}

/// Ruby `r`'s extents above and below its base's baseline in a line, px:
/// the base's content and half-leading, and on the annotation's side what
/// the annotation lacks of the line's leading (CSS Ruby 1 §3.6: a line as
/// tall as its line height holds its annotation when both half-leadings,
/// its own and the next line's, do).
pub(crate) fn ruby_extents(s: &RichSpan, r: &Ruby, fonts: &dyn Metrics) -> (f32, f32) {
    let size = s.step.size;
    let m = fonts.metrics_styled(s.font, s.query(), &s.text);
    let (a, d) = content_edges(m, &s.text);
    let (asc, desc) = (a * size, d * size);
    let lh = if s.text.chars().any(crate::text::is_cjk) {
        s.step.cjk_line_height
    } else {
        s.step.line_height
    };
    let lead = (lh - asc - desc) / 2.0;
    let extra = (size * RUBY_SIZE - 2.0 * lead).max(0.0);
    if r.under {
        (asc + lead, desc + lead + extra)
    } else {
        (asc + lead + extra, desc + lead)
    }
}

/// Where each of `clusters` starts in a column `length` long, by
/// `ruby-align` (CSS Ruby 1 §4.3): `space-between` and `space-around`
/// share the free room among the justification opportunities between
/// two CJK characters (`text-justify: ruby`), `space-around` with half a
/// share at each end; with none they centre, as `center` does.
pub(crate) fn distribute(clusters: &[(String, f32)], length: f32, align: RubyAlign) -> Vec<f32> {
    let free = (length - self::length(clusters)).max(0.0);
    let cjk: Vec<bool> = clusters
        .iter()
        .map(|(c, _)| c.chars().next().is_some_and(crate::text::is_cjk))
        .collect();
    let gaps: Vec<bool> = cjk.windows(2).map(|w| w[0] && w[1]).collect();
    let k = gaps.iter().filter(|g| **g).count() as f32;
    let (lead, gap) = match align {
        RubyAlign::Start => (0.0, 0.0),
        RubyAlign::SpaceBetween if k > 0.0 => (0.0, free / k),
        RubyAlign::SpaceAround if k > 0.0 => (free / (k + 1.0) / 2.0, free / (k + 1.0)),
        _ => (free / 2.0, 0.0),
    };
    let mut at = lead;
    clusters
        .iter()
        .enumerate()
        .map(|(n, (_, len))| {
            let start = at;
            at += len
                + if gaps.get(n).copied().unwrap_or(false) {
                    gap
                } else {
                    0.0
                };
            start
        })
        .collect()
}

/// Where a ruby is drawn: its stretch of its line, the base's baseline
/// (across) or the column's middle (down), its ink.
pub(super) struct RubyAt {
    pub rect: Rect,
    pub baseline: f32,
    pub color: Srgba,
}

impl Ui {
    /// Ruby `r` of span `s`'s commands: base and annotation each cluster
    /// by cluster where `ruby-align` puts it in their column.
    pub(super) fn ruby_commands(
        &self,
        i: usize,
        s: &RichSpan,
        r: &Ruby,
        at: RubyAt,
        v: Option<VerticalText>,
        cmds: &mut Vec<crate::draw::DrawCmd>,
    ) {
        let n = &self.tree.nodes[i];
        let sk = n.placed.scale;
        let size = s.step.size * sk;
        let a = size * RUBY_SIZE;
        let fonts = &self.env.fonts;
        let geometry = ruby_box(s, r, size, fonts, v);
        let base = clusters(s, &s.text, size, span_tracking(s, &s.text), fonts, v);
        let annot = clusters(s, &r.text, a, 0.0, fonts, v);
        let ink = at.color.fade(n.placed.alpha).linear();
        let clip = self.paint_clip(n);
        let features = s.step.features;
        let ruby_forms = features.with_settings(&[(*b"ruby", 1)]);
        let weight = if s.synth.bold {
            bold_weight(size, 0.0)
        } else {
            0.0
        };
        let base_font = fonts.for_styled(s.font, s.query(), &s.text).0;
        let annot_font = fonts.for_styled(s.font, s.query(), &r.text).0;
        let rect = at.rect;
        let text_cmd = |x: f32,
                        baseline: f32,
                        size: f32,
                        text: String,
                        font: Option<og_draw::FontHandle>,
                        placements: Vec<og_draw::GlyphPlacement>,
                        bounds: Rect,
                        transform: [f32; 4],
                        tracking: f32|
         -> Option<crate::draw::DrawCmd> {
            let font = font.filter(|_| !text.is_empty())?;
            let cmd = crate::draw::DrawCmd::Text {
                x,
                baseline,
                size,
                text,
                color: ink,
                font,
                bounds,
                clip,
                tracking,
                weight,
                glow: None,
                placements,
            };
            Some(if transform == [1.0, 0.0, 0.0, 1.0] {
                cmd
            } else {
                crate::draw::DrawCmd::StyledText {
                    text: Box::new(cmd),
                    effects: Box::new(og_draw::UiTextEffects {
                        transform,
                        ..Default::default()
                    }),
                }
            })
        };
        let para = self.text_flow(i).1;
        let Some(v) = v else {
            // Across: the annotation's em box on the base's content edge.
            let (ba, bd) = content_edges(&base_font.metrics, &s.text);
            let (aa, ad) = content_edges(&annot_font.metrics, &r.text);
            let middle = if r.under {
                at.baseline + bd * size + a / 2.0
            } else {
                at.baseline - ba * size - a / 2.0
            };
            // The annotation's content area centred in its em box (its
            // line height 1, HTML's `rt`).
            let annot_baseline = middle + (aa - ad) / 2.0 * a;
            let x0 = rect.x + geometry.column;
            for (list, starts, font, size, baseline, features, tracking) in [
                (
                    &base,
                    distribute(&base, geometry.length, r.align),
                    base_font,
                    size,
                    at.baseline,
                    features,
                    span_tracking(s, &s.text),
                ),
                (
                    &annot,
                    distribute(&annot, geometry.length, r.align),
                    annot_font,
                    a,
                    annot_baseline,
                    ruby_forms,
                    0.0,
                ),
            ] {
                // On the device pixel grid, as the line's other text.
                let baseline = crate::layout::snap(baseline, self.env.viewport.dpr());
                for ((c, len), start) in list.iter().zip(starts) {
                    for pc in crate::text::pieces_with(font, c, para, tracking, features) {
                        let x = x0 + start + pc.dx * size;
                        let bounds = Rect::new(x, baseline - size, *len, size * 1.2);
                        cmds.extend(text_cmd(
                            x,
                            baseline,
                            size,
                            pc.text,
                            pc.font,
                            pc.placements,
                            bounds,
                            [1.0, 0.0, 0.0, 1.0],
                            tracking,
                        ));
                    }
                }
            }
            return;
        };
        // Down a vertical line: the annotation's column beside the base's,
        // on the line-over side unless `under`.
        let from_bottom = self.rich_from_bottom(i, v);
        let over = if v.mode == WritingMode::SidewaysLr {
            -1.0
        } else {
            1.0
        };
        let side = if r.under { -over } else { over };
        let annot_mid = at.baseline + side * (size + a) / 2.0;
        for (list, whole, font, size, mid, tracking, features) in [
            (
                &base,
                s.text.as_str(),
                base_font,
                size,
                at.baseline,
                span_tracking(s, &s.text),
                features,
            ),
            (
                &annot,
                r.text.as_str(),
                annot_font,
                a,
                annot_mid,
                0.0,
                ruby_forms,
            ),
        ] {
            let starts = distribute(list, geometry.length, r.align);
            let run = crate::ui::VerticalRun {
                tracking,
                features,
                ..self.span_run(
                    i,
                    s,
                    font,
                    whole,
                    size,
                    v,
                    (mid, mid - size / 2.0, size),
                    true,
                )
            };
            for ((c, _), start) in list.iter().zip(starts) {
                let offset = geometry.column + start;
                let top = |at: f32, len: f32| {
                    if from_bottom {
                        rect.bottom() - offset - at - len
                    } else {
                        rect.y + offset + at
                    }
                };
                let mut at = 0.0;
                for (range, set) in crate::text::vertical::runs(c, v) {
                    at = crate::ui::vertical_run(
                        &run,
                        &c[range],
                        set,
                        at,
                        &top,
                        &mut || 0.0,
                        &mut |_, _, _| {},
                        &mut |x, baseline, t, handle, placements, transform, bounds| {
                            cmds.extend(text_cmd(
                                x,
                                baseline,
                                size,
                                t,
                                handle,
                                placements,
                                bounds,
                                transform,
                                run.tracking,
                            ));
                        },
                    );
                }
            }
        }
    }
}
