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
                self.warnings.push(format!(
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
        let t = self.theme;
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
            };
            for (_, _, style) in ranges.iter().filter(|(a, b, _)| *a <= start && end <= *b) {
                if let Some(c) = &style.color {
                    s.color = resolve_color(c, &t);
                }
                if let Some(step) = style.step {
                    s.step = step;
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
            s.synth = self.fonts.for_styled(s.font, s.weight, s.italic, &s.text).1;
            s.synth.bold &= s.bold;
            runs.push(s);
        }
        let node = &mut self.nodes[i];
        node.el.width = desc.width;
        node.probe_text = text.to_owned();
        node.content = Content::Rich(RichContent {
            spans: runs,
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
