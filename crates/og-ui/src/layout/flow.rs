//! An element's flow (CSS Writing Modes 4): its direction, its bidi
//! embedding (`unicode-bidi`), and how its logical properties name its
//! physical sides. A document's builder renames each logical declaration
//! to the physical one it sets ([`physical`]) before lowering it.

/// CSS `unicode-bidi` (CSS Writing Modes 4 §2.2). Each element's text is a
/// paragraph of its own, so `embed` reads as `isolate`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Bidi {
    #[default]
    Normal,
    Embed,
    Isolate,
    /// `bidi-override` and `isolate-override`: its characters in the
    /// element's direction, unreordered.
    Override,
    /// `plaintext`: each paragraph's direction from its first strong
    /// character.
    Plaintext,
}

impl Bidi {
    pub fn parse(v: &str) -> Option<Bidi> {
        Some(match v.trim() {
            "normal" => Bidi::Normal,
            "embed" => Bidi::Embed,
            "isolate" => Bidi::Isolate,
            "bidi-override" | "isolate-override" => Bidi::Override,
            "plaintext" => Bidi::Plaintext,
            _ => return None,
        })
    }
}

/// The flow properties a document declares ([`check`]).
pub(crate) fn longhand(name: &str) -> bool {
    matches!(name, "direction" | "unicode-bidi")
}

/// Checks a flow property's value.
pub(crate) fn check(name: &str, value: &str) -> Result<(), String> {
    let ok = match name {
        "direction" => matches!(value.trim(), "ltr" | "rtl"),
        _ => Bidi::parse(value).is_some(),
    };
    if ok {
        Ok(())
    } else {
        Err(format!("{name}: unknown value {}", value.trim()))
    }
}

/// An element's flow.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Flow {
    /// Right to left (CSS `direction: rtl`).
    pub rtl: bool,
    pub bidi: Bidi,
    /// A document element: its physical properties are physical, and the
    /// right-to-left mirror of the whole surface does not turn its
    /// transforms round.
    pub doc: bool,
    /// Its direction or bidi was declared (`direction`, `unicode-bidi`,
    /// `dir`) on it or an ancestor. Undeclared, a paragraph takes its
    /// direction from its first strong character, as og-ui's own text does.
    pub declared: bool,
}

impl Flow {
    /// How its text's paragraphs take their direction.
    pub(crate) fn para(&self) -> crate::text::Para {
        use crate::text::Para;
        let d = self.direction();
        match self.bidi {
            Bidi::Override => Para::Override(d),
            Bidi::Plaintext => Para::Auto(d),
            _ if self.declared => Para::Fixed(d),
            _ => Para::Auto(d),
        }
    }

    pub(crate) fn direction(&self) -> og_i18n::Direction {
        if self.rtl {
            og_i18n::Direction::Rtl
        } else {
            og_i18n::Direction::Ltr
        }
    }
}

/// Node `i`'s children, placed left to right, flipped across its content
/// box (a scroller's: its scrolled content): each margin box flips, its own
/// margins kept on their sides, as `row-reverse` flips a line. An
/// out-of-flow child placed by a horizontal inset keeps its place. `sx`
/// is its horizontal scroll offset.
pub(super) fn flip_children(nodes: &mut [super::Node], i: usize, sx: f32) {
    let n = &nodes[i];
    let inner = (n.w - n.el.padding.horizontal()).max(0.0);
    let width = if n.el.scroll_x {
        inner.max(n.content_w)
    } else {
        inner
    };
    let x0 = n.x + n.el.padding.left - sx;
    let free = |c: &super::Node| {
        !c.props.out_of_flow()
            || (c.props.insets.start.is_none()
                && c.props.insets.end.is_none()
                && !c.props.insets.center_x
                && c.props.insets.across.is_none()
                && c.props.anchor.is_none())
    };
    let kids: Vec<usize> = n.children.iter().chain(&n.out).copied().collect();
    for c in kids {
        let k = &nodes[c];
        if !free(k) {
            continue;
        }
        let [ml, mr] = if k.props.out_of_flow() {
            [0.0; 2]
        } else {
            k.margin_x
        };
        nodes[c].x = 2.0 * x0 + width - k.x - k.w - mr + ml;
    }
    let pl = nodes[i].el.padding.left;
    for r in &mut nodes[i].col_rules {
        r[0] = 2.0 * pl + width - r[0];
    }
}

/// A document element's box under the surface's right-to-left mirror:
/// laid out before it, its physical left and right sides swap, so the
/// mirror puts them back.
pub(crate) fn mirror_el(el: &mut super::Element) {
    let p = &mut el.padding;
    std::mem::swap(&mut p.left, &mut p.right);
    el.margin.swap(1, 3);
}

/// [`mirror_el`] for its insets (`left` and `right`, px or `anchor()`).
pub(crate) fn mirror_props(p: &mut crate::element::Props) {
    let ins = &mut p.insets;
    std::mem::swap(&mut ins.start, &mut ins.end);
    if let Some(a) = p.anchor.as_mut() {
        a.base.insets.swap(1, 3);
    }
}

const SIDES: [&str; 4] = ["top", "right", "bottom", "left"];

/// The physical side (0 top, 1 right, 2 bottom, 3 left) of a logical one:
/// `inline` picks the inline axis, `end` its end.
fn side(flow: Flow, inline: bool, end: bool) -> usize {
    match (inline, end != flow.rtl) {
        (false, _) => usize::from(end) * 2,
        (true, false) => 3,
        (true, true) => 1,
    }
}

/// `value`'s top-level space-separated parts (`calc(1px + 2px)` is one).
fn parts(value: &str) -> Vec<&str> {
    let (mut out, mut depth, mut start) = (Vec::new(), 0i32, None);
    for (k, c) in value.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            c if c.is_whitespace() && depth == 0 => {
                if let Some(s) = start.take() {
                    out.push(&value[s..k]);
                }
                continue;
            }
            _ => {}
        }
        start.get_or_insert(k);
    }
    if let Some(s) = start {
        out.push(&value[s..]);
    }
    out
}

/// The physical property `prefix` names on physical side `s`, followed by
/// `rest` (`margin` + left → `margin-left`; `inset` + top → `top`). The
/// left and right insets keep the lowering's `inset-inline-start` and
/// `-end`, which set the left and right inset and also take `bind()`.
fn sided(prefix: &str, s: usize, rest: &str) -> String {
    match (prefix, s) {
        ("inset", 1) => "inset-inline-end".into(),
        ("inset", 3) => "inset-inline-start".into(),
        ("inset", _) => format!("{}{rest}", SIDES[s]),
        _ => format!("{prefix}-{}{rest}", SIDES[s]),
    }
}

/// The physical declarations a logical one sets in `flow` (CSS Logical
/// Properties 1); `None` when `name` is physical already.
pub(crate) fn physical(name: &str, value: &str, flow: Flow) -> Option<Vec<(String, String)>> {
    let one = |n: String| Some(vec![(n, value.to_string())]);
    // Sizes and axes: the inline axis is the horizontal one.
    for (logical, inline) in [("inline", true), ("block", false)] {
        let axis = |h: &str, v: &str| if inline { h.to_string() } else { v.to_string() };
        match name.strip_suffix(&format!("{logical}-size")) {
            Some(p @ ("" | "min-" | "max-")) => {
                return one(format!("{p}{}", axis("width", "height")));
            }
            Some("contain-intrinsic-") => {
                return one(format!("contain-intrinsic-{}", axis("width", "height")));
            }
            _ => {}
        }
        for p in ["overflow", "overscroll-behavior"] {
            if name.strip_prefix(p).and_then(|r| r.strip_prefix('-')) == Some(logical) {
                return one(format!("{p}-{}", axis("x", "y")));
            }
        }
    }
    // Corners: block side, then inline side.
    if let Some(corner) = name
        .strip_prefix("border-")
        .and_then(|r| r.strip_suffix("-radius"))
        && let Some((b, i)) = corner.split_once('-')
        && matches!(b, "start" | "end")
        && matches!(i, "start" | "end")
    {
        let mut s = [side(flow, false, b == "end"), side(flow, true, i == "end")];
        s.sort_by_key(|s| s % 2);
        return one(format!("border-{}-{}-radius", SIDES[s[0]], SIDES[s[1]]));
    }
    let prefixes = [
        "scroll-margin",
        "scroll-padding",
        "margin",
        "padding",
        "border",
        "inset",
    ];
    let (prefix, tail) = prefixes
        .iter()
        .find_map(|p| Some((*p, name.strip_prefix(p)?.strip_prefix('-')?)))?;
    let (inline, tail) = if let Some(t) = tail.strip_prefix("inline") {
        (true, t)
    } else {
        (false, tail.strip_prefix("block")?)
    };
    for (end, word) in [(false, "-start"), (true, "-end")] {
        if let Some(rest) = tail.strip_prefix(word) {
            return one(sided(prefix, side(flow, inline, end), rest));
        }
    }
    if !matches!(tail, "" | "-width" | "-style" | "-color")
        || (prefix != "border" && !tail.is_empty())
    {
        return None;
    }
    // Two sides: start then end (`border-inline` gives both its one value).
    let (a, b) = match parts(value).as_slice() {
        [a, b] if !(prefix == "border" && tail.is_empty()) => (a.to_string(), b.to_string()),
        _ => (value.to_string(), value.to_string()),
    };
    Some(vec![
        (sided(prefix, side(flow, inline, false), tail), a),
        (sided(prefix, side(flow, inline, true), tail), b),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_properties_name_the_sides_of_the_flow() {
        let (ltr, rtl) = (
            Flow::default(),
            Flow {
                rtl: true,
                ..Flow::default()
            },
        );
        let p = |n, v, f| physical(n, v, f).unwrap();
        let pair = |a: &str, b: &str| (a.to_string(), b.to_string());
        assert_eq!(
            p("margin-inline-start", "4px", ltr),
            [pair("margin-left", "4px")]
        );
        assert_eq!(
            p("margin-inline-start", "4px", rtl),
            [pair("margin-right", "4px")]
        );
        assert_eq!(
            p("inset-inline-end", "0", rtl),
            [pair("inset-inline-start", "0")]
        );
        assert_eq!(p("inset-block-end", "0", rtl), [pair("bottom", "0")]);
        assert_eq!(
            p("border-inline-end-color", "red", rtl),
            [pair("border-left-color", "red")]
        );
        assert_eq!(
            p("padding-inline", "1px calc(2px + 1em)", rtl),
            [
                pair("padding-right", "1px"),
                pair("padding-left", "calc(2px + 1em)")
            ]
        );
        assert_eq!(
            p("border-inline", "1px solid red", rtl),
            [
                pair("border-right", "1px solid red"),
                pair("border-left", "1px solid red")
            ]
        );
        assert_eq!(
            p("border-start-end-radius", "3px", rtl),
            [pair("border-top-left-radius", "3px")]
        );
        assert_eq!(
            p("border-end-start-radius", "3px", ltr),
            [pair("border-bottom-left-radius", "3px")]
        );
        assert_eq!(p("max-inline-size", "9px", rtl), [pair("max-width", "9px")]);
        assert_eq!(
            p("margin-block", "1px 2px", rtl),
            [pair("margin-top", "1px"), pair("margin-bottom", "2px")]
        );
        assert_eq!(
            p("overflow-block", "auto", ltr),
            [pair("overflow-y", "auto")]
        );
        for physical_name in [
            "margin-left",
            "left",
            "border-top-left-radius",
            "width",
            "margin",
        ] {
            assert!(
                physical(physical_name, "1px", rtl).is_none(),
                "{physical_name}"
            );
        }
    }
}
