//! Metric-based outer line trimming (CSS Inline 3 §6). Trimming changes
//! the first/last line boxes, never the distance between their baselines.

use crate::layout::Metrics;
use crate::rich::{RichContent, RichLine};

/// Which outer edges of a text block are trimmed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Trim {
    #[default]
    None,
    Start,
    End,
    Both,
}

impl Trim {
    pub(crate) fn parse(v: &str) -> Option<Self> {
        Some(match v {
            "none" => Self::None,
            "trim-start" => Self::Start,
            "trim-end" => Self::End,
            "trim-both" => Self::Both,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Edge {
    #[default]
    Text,
    Cap,
    Ex,
    Alphabetic,
    Ideographic,
}

/// The font metrics used for the text-over and text-under edges.
/// `auto` uses text/text (the initial line-fit-edge is leading).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Edges([Edge; 2]);

impl Edges {
    pub(crate) fn parse(v: &str) -> Option<Self> {
        let words: Vec<_> = v.split_whitespace().collect();
        if words == ["auto"] {
            return Some(Self::default());
        }
        let parse = |s| match s {
            "text" => Some(Edge::Text),
            "cap" => Some(Edge::Cap),
            "ex" => Some(Edge::Ex),
            "alphabetic" => Some(Edge::Alphabetic),
            "ideographic" => Some(Edge::Ideographic),
            _ => None,
        };
        let [over, under] = match words.as_slice() {
            [a] if matches!(*a, "text" | "ideographic") => [parse(a)?, parse(a)?],
            [a, b] => [parse(a)?, parse(b)?],
            _ => return None,
        };
        (over != Edge::Alphabetic && !matches!(under, Edge::Cap | Edge::Ex))
            .then_some(Self([over, under]))
    }
}

/// CSS `text-box-trim` and `text-box-edge` on a paragraph's root text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextBox {
    pub trim: Trim,
    pub edge: Edges,
}

/// The shorthand's longhand values. Omitted trim means trim-both, while
/// `normal` resets both longhands, including an inherited edge.
pub(crate) fn shorthand(v: &str) -> Option<(String, String)> {
    if v == "normal" {
        return Some(("none".into(), "auto".into()));
    }
    let mut trim = None;
    let mut edges = Vec::new();
    for w in v.split_whitespace() {
        if Trim::parse(w).is_some() {
            if trim.replace(w).is_some() {
                return None;
            }
        } else {
            edges.push(w);
        }
    }
    if trim.is_none() && edges.is_empty() {
        return None;
    }
    let edge = if edges.is_empty() {
        "auto".into()
    } else {
        edges.join(" ")
    };
    Edges::parse(&edge)?;
    Some((trim.unwrap_or("trim-both").into(), edge))
}

/// Trim once after the entire paragraph has broken, including forced
/// breaks and discretionary hyphen reshaping. The root's first available
/// font defines the edges, not the ink of the particular string or the
/// font of a larger inline span; such ink may overflow the trimmed box.
pub(crate) fn apply(content: &RichContent, fonts: &dyn Metrics, lines: &mut [RichLine]) {
    let trim = content.text_box.trim;
    if trim == Trim::None || lines.is_empty() {
        return;
    }
    let (step, role) = content.base;
    let m = fonts.metrics_styled(role, crate::FontQuery::new(step.weight, false), "");
    let b = crate::layout::baselines_of(m);
    let [over, under] = content.text_box.edge.0.map(|edge| match edge {
        Edge::Text => [b.text_top, b.text_bottom],
        Edge::Cap => [m.cap_height(), 0.0],
        Edge::Ex => [m.x_height(), 0.0],
        Edge::Alphabetic => [0.0, 0.0],
        Edge::Ideographic => [2.0 * b.central - b.ideographic, b.ideographic],
    });
    if matches!(trim, Trim::Start | Trim::Both) {
        let first = &mut lines[0];
        let top = over[0] * step.size;
        first.height -= first.baseline - top;
        first.baseline = top;
    }
    if matches!(trim, Trim::End | Trim::Both) {
        let last = lines.last_mut().expect("nonempty paragraph");
        last.height = last.baseline - under[1] * step.size;
    }
}
