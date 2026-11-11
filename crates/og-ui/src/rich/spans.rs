//! Structured narrative ranges: plain text in, styled paragraph runs out.

use super::*;
use crate::paint::ColorRef;
use std::collections::BTreeMap;

/// A narrative attribute value. Objects, arrays and null are not attributes.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum TextSpanValue {
    String(String),
    Number(f64),
    Bool(bool),
}

/// A range of Unicode scalar values in plain text, not UTF-8 bytes, UTF-16
/// code units or grapheme clusters. Zero-length ranges are metadata markers.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TextSpan {
    pub name: String,
    pub position: u32,
    pub length: u32,
    pub properties: BTreeMap<String, TextSpanValue>,
}

/// Declarations from the active style's `::part(name)` cascade. Unset fields
/// inherit the paragraph or earlier overlapping attributes. Colour tokens
/// remain symbolic until the paragraph is built against its active theme.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextSpanStyle {
    pub color: Option<ColorRef>,
    pub step: Option<TypeStep>,
    pub font: Option<FontRole>,
    pub weight: Option<u16>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub strike: Option<bool>,
    pub outline: Option<(f32, ColorRef)>,
    pub motion: Option<SpanFx>,
    /// A link's id (a document's inline `<a href>`): accent ink and an
    /// underline unless declared, focusable, its click answered by
    /// [`Ui::respond_links`].
    pub link: Option<String>,
    /// Its font size, line height and tracking over its type step's (CSS
    /// `font-size`, `line-height` in px, `letter-spacing` in em).
    pub size: Option<f32>,
    pub line_height: Option<f32>,
    pub tracking_em: Option<f32>,
    /// It is an inline picture, icon or action glyph (its one character the
    /// placeholder U+FFFC), one em tall in the line.
    pub atom: Option<InlineAtom>,
    /// A CSS `text-decoration` beyond the plain underline and strike: its
    /// lines, style, colour, thickness and offset (the lines it sets win
    /// over `underline` and `strike`).
    pub decoration: Option<crate::doc::text_css::Decoration>,
    /// CSS `vertical-align`: where it sits in its line against the
    /// paragraph's text (`Baseline` when unset).
    pub vertical_align: Option<crate::VerticalAlign>,
    /// Its font size and line height as a fraction of what it would
    /// otherwise take (HTML's `<sub>` and `<sup>`: CSS `font-size: smaller`).
    pub size_scale: Option<f32>,
}

/// An inline object of a text (a document's `<img>`, `<og-icon>` or `<kbd
/// data-action>` among its words).
#[derive(Clone, Debug, PartialEq)]
pub enum InlineAtom {
    /// A picture the game registered (`UiDesc.images`).
    Image(String),
    /// An icon of the style's atlas.
    Icon(String),
    /// An action's glyph on the device used last.
    Action(String),
}

impl Ui {
    /// Plain text with typed attributes; never interpreted as markup.
    /// Offsets count Unicode scalars. Overlaps apply in array order, with
    /// later explicit declarations winning independently for each property.
    /// Invalid ranges warn and do not style text; all attributes are returned
    /// unchanged, including unknown names and zero-length metadata markers.
    pub fn rich_text_spans(
        &mut self,
        id: impl Into<crate::Id>,
        text: &str,
        attributes: &[TextSpan],
        desc: RichTextDesc,
        resolve: impl FnMut(&TextSpan) -> TextSpanStyle,
    ) -> RichTextResponse {
        let i = self.leaf("richtext", id.into(), Element::default());
        self.apply_text_spans(i, text, attributes, desc, resolve);
        RichTextResponse {
            spans: attributes.to_vec(),
            ..Default::default()
        }
    }

    /// Replaces structured text in its existing render node. A style's
    /// `link` makes its range a link; responses are [`Ui::respond_links`]'.
    pub(crate) fn apply_text_spans(
        &mut self,
        i: usize,
        text: &str,
        attributes: &[TextSpan],
        desc: RichTextDesc,
        mut resolve: impl FnMut(&TextSpan) -> TextSpanStyle,
    ) {
        let bytes: Vec<usize> = text
            .char_indices()
            .map(|(i, _)| i)
            .chain([text.len()])
            .collect();
        let count = bytes.len() - 1;
        let mut boundaries = vec![0, count];
        let mut ranges = Vec::new();
        for a in attributes {
            let start = a.position as usize;
            let end = start.checked_add(a.length as usize);
            let Some(end) = end.filter(|&end| start <= count && end <= count) else {
                self.out.warnings.push(format!(
                    "rich text: span '{}' range {}+{} exceeds {count} Unicode scalars",
                    a.name, a.position, a.length
                ));
                continue;
            };
            if start != end {
                let style = resolve(a);
                if style != TextSpanStyle::default() {
                    boundaries.extend([start, end]);
                    ranges.push((start, end, style));
                }
            }
        }
        boundaries.sort_unstable();
        boundaries.dedup();
        let t = self.env.theme;
        let mut runs = Vec::new();
        for pair in boundaries.windows(2) {
            let (start, end) = (pair[0], pair[1]);
            let step = desc.step.unwrap_or(t.type_ramp.body);
            let mut s = RichSpan {
                text: text[bytes[start]..bytes[end]].to_owned(),
                step,
                color: desc.color.unwrap_or(t.palette.ink),
                font: desc.font,
                bold: false,
                italic: false,
                weight: step.weight,
                synth: Default::default(),
                underline: false,
                strike: false,
                outline: None,
                atom: None,
                link: None,
                fx: SpanFx::None,
                decoration: None,
                valign: crate::VerticalAlign::Baseline,
            };
            for (_, _, style) in ranges.iter().filter(|(a, b, _)| *a <= start && end <= *b) {
                if let Some(c) = &style.color {
                    s.color = resolve_color(c, &t);
                }
                if let Some(step) = style.step {
                    s.step = step;
                }
                if let Some(size) = style.size {
                    s.step.size = size;
                }
                if let Some(line) = style.line_height {
                    s.step.line_height = line;
                }
                if let Some(tracking) = style.tracking_em {
                    s.step.tracking_em = tracking;
                }
                if let Some(k) = style.size_scale {
                    s.step.size *= k;
                    s.step.line_height *= k;
                    s.step.cjk_line_height *= k;
                }
                if let Some(v) = style.vertical_align {
                    s.valign = v;
                }
                if let Some(atom) = &style.atom {
                    s.text.clear();
                    s.atom = match atom {
                        InlineAtom::Image(name) => match self.env.images.get(name) {
                            Some(im) => Some(crate::rich::Atom::Image(*im)),
                            None => {
                                self.out.warnings.push(format!(
                                    "text: <img src=\"{name}\"> is no picture the game registered (UiDesc.images)"
                                ));
                                None
                            }
                        },
                        InlineAtom::Icon(name) => match self.env.icons.find(name) {
                            Some(k) => Some(crate::rich::Atom::Icon(k)),
                            None => {
                                self.out.warnings.push(format!(
                                    "text: <og-icon name=\"{name}\"> is not in the icon atlas"
                                ));
                                None
                            }
                        },
                        InlineAtom::Action(name) => match self.glyph(name) {
                            Some(g) => match self.env.icons.find(&g.icon) {
                                Some(k) => Some(crate::rich::Atom::Icon(k)),
                                None => Some(crate::rich::Atom::Cap(g.text)),
                            },
                            None => None,
                        },
                    };
                }
                if let Some(font) = style.font {
                    s.font = font;
                }
                if let Some(bold) = style.bold {
                    s.bold = bold;
                }
                if let Some(italic) = style.italic {
                    s.italic = italic;
                }
                if let Some(underline) = style.underline {
                    s.underline = underline;
                }
                if let Some(strike) = style.strike {
                    s.strike = strike;
                }
                if let Some((width, color)) = &style.outline {
                    s.outline = Some((*width, resolve_color(color, &t)));
                }
                if let Some(motion) = style.motion {
                    s.fx = motion;
                }
                if let Some(d) = &style.decoration {
                    s.underline = d.underline;
                    s.strike = d.through;
                    s.decoration = Some(d.clone());
                }
                if let Some(link) = &style.link {
                    s.link = Some(link.clone());
                    if style.color.is_none() {
                        s.color = t.palette.highlight;
                    }
                    s.underline |= style.underline.is_none();
                }
            }
            // Weight and bold share a CSS property; the last explicit one wins.
            s.weight = s.step.weight;
            for (_, _, style) in ranges.iter().filter(|(a, b, _)| *a <= start && end <= *b) {
                if let Some(bold) = style.bold {
                    s.weight = if bold { 700 } else { s.step.weight };
                }
                if let Some(weight) = style.weight {
                    s.weight = weight;
                }
            }
            s.bold = s.weight >= 700;
            s.synth = self
                .env
                .fonts
                .for_styled(s.font, s.weight, s.italic, &s.text)
                .1;
            s.synth.bold &= s.bold;
            runs.push(s);
        }
        let node = &mut self.tree.nodes[i];
        node.el.width = desc.width;
        node.probe_text = text.to_owned();
        node.content = Content::Rich(RichContent {
            spans: runs,
            base: (desc.step.unwrap_or(t.type_ramp.body), desc.font),
            align: desc.align,
            wrap: !desc.nowrap,
        });
        node.links.clear();
        for r in &node_runs_links(&node.content) {
            if !node.links.contains(r) {
                node.links.push(r.clone());
            }
        }
    }
}

fn node_runs_links(content: &Content) -> Vec<String> {
    match content {
        Content::Rich(rc) => rc.spans.iter().filter_map(|s| s.link.clone()).collect(),
        _ => Vec::new(),
    }
}

fn resolve_color(color: &ColorRef, theme: &crate::Theme) -> Srgba {
    color
        .srgb(theme)
        .unwrap_or_else(|| Srgba::from_linear(crate::paint::MISSING_TOKEN))
}
