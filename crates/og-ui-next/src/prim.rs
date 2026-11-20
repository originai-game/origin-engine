//! The new core's layout primitives: typed element fields the .svelte
//! compiler sets from an element's own attributes (`layout="row"`,
//! `gap="tight"`, `anchor="top-end"`, `fill`, `reverse`), never CSS-visible
//! attributes a selector matches. The cook turns them into the element's
//! first style span, beneath its sheet's rules, with the gap scale and the
//! anchor inset read from the style's tokens once.

use crate::style::{self, Step};
use crate::tokens::Tokens;

/// How an element lays out its children.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Layout {
    /// A row, children centred on the cross axis.
    Row,
    /// A column.
    Stack,
    /// A row that wraps, children centred on the cross axis.
    Wrap,
    /// A column whose `anchor` children lie over it.
    Overlay,
    /// A column that scrolls and may shrink to nothing.
    Scroll,
    /// A column, children centred, no wider than the measure.
    Center,
}

/// The gap between an element's children: a step of the style's scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Gap {
    None,
    Hair,
    Tight,
    #[default]
    Base,
    Loose,
    Wide,
}

/// Where an element lies over its parent: out of flow, at an edge or a
/// corner of its box, the anchor inset in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Anchor {
    Fill,
    TopStart,
    TopEnd,
    BottomStart,
    BottomEnd,
    TopEdge,
    BottomEdge,
}

/// A named motion an element plays (`play="pulse"`): read here, animated
/// by the motion head's springs and loops.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Motion {
    Enter,
    Exit,
    Pop,
    Stamp,
    Beat,
    Pulse,
    Spin,
    Ping,
    Shimmer,
    Float,
    Change,
}

/// An element's layout fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Fields {
    pub layout: Option<Layout>,
    pub gap: Gap,
    pub anchor: Option<Anchor>,
    /// Takes the room its parent has left on the main axis.
    pub fill: bool,
    /// Lays its children out from the end.
    pub reverse: bool,
    pub play: Option<Motion>,
}

/// An attribute the compiler did not take, with its value.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Unknown(pub String, pub String);

impl Fields {
    /// Reads one attribute; `Ok(false)` when it is not a layout field.
    pub fn set(&mut self, name: &str, value: &str) -> Result<bool, Unknown> {
        let bad = || Unknown(name.to_owned(), value.to_owned());
        match name {
            "layout" => {
                self.layout = Some(match value {
                    "row" => Layout::Row,
                    "stack" => Layout::Stack,
                    "wrap" => Layout::Wrap,
                    "overlay" => Layout::Overlay,
                    "scroll" => Layout::Scroll,
                    "center" => Layout::Center,
                    _ => return Err(bad()),
                })
            }
            "gap" => {
                self.gap = match value {
                    "none" => Gap::None,
                    "hair" => Gap::Hair,
                    "tight" => Gap::Tight,
                    "base" => Gap::Base,
                    "loose" => Gap::Loose,
                    "wide" => Gap::Wide,
                    _ => return Err(bad()),
                }
            }
            "anchor" => {
                self.anchor = Some(match value {
                    "fill" => Anchor::Fill,
                    "top-start" => Anchor::TopStart,
                    "top-end" => Anchor::TopEnd,
                    "bottom-start" => Anchor::BottomStart,
                    "bottom-end" => Anchor::BottomEnd,
                    "top-edge" => Anchor::TopEdge,
                    "bottom-edge" => Anchor::BottomEdge,
                    _ => return Err(bad()),
                })
            }
            "play" => {
                self.play = Some(match value {
                    "enter" => Motion::Enter,
                    "exit" => Motion::Exit,
                    "pop" => Motion::Pop,
                    "stamp" => Motion::Stamp,
                    "beat" => Motion::Beat,
                    "pulse" => Motion::Pulse,
                    "spin" => Motion::Spin,
                    "ping" => Motion::Ping,
                    "shimmer" => Motion::Shimmer,
                    "float" => Motion::Float,
                    "change" => Motion::Change,
                    _ => return Err(bad()),
                })
            }
            "fill" => self.fill = flag(value).ok_or_else(bad)?,
            "reverse" => self.reverse = flag(value).ok_or_else(bad)?,
            _ => return Ok(false),
        }
        Ok(true)
    }

    /// Appends the fields' steps.
    pub fn steps(&self, scale: &Scale, out: &mut Vec<Step>) {
        let mut put = |name: &str, v: &str| {
            let taken = style::expand(out, name, v);
            debug_assert!(taken, "{name}: {v}");
        };
        if let Some(layout) = self.layout {
            put("display", "flex");
            let column = !matches!(layout, Layout::Row | Layout::Wrap);
            put(
                "flex-direction",
                match (column, self.reverse) {
                    (false, false) => "row",
                    (false, true) => "row-reverse",
                    (true, false) => "column",
                    (true, true) => "column-reverse",
                },
            );
            match layout {
                Layout::Row => put("align-items", "center"),
                Layout::Wrap => {
                    put(
                        "flex-wrap",
                        if self.reverse { "wrap-reverse" } else { "wrap" },
                    );
                    put("align-items", "center");
                }
                Layout::Scroll => put("min-height", "0px"),
                Layout::Center => {
                    put("align-items", "center");
                    put("max-width", &scale.measure);
                    put("margin-inline", "auto");
                }
                Layout::Stack | Layout::Overlay => {}
            }
            put("gap", &scale.gaps[self.gap as usize]);
        }
        if self.fill {
            put("flex", "1 1 0px");
            put("min-width", "0px");
            put("min-height", "0px");
        }
        if let Some(anchor) = self.anchor {
            put("position", "absolute");
            let i = scale.inset.as_str();
            let [top, right, bottom, left] = match anchor {
                Anchor::Fill => ["0px"; 4],
                Anchor::TopStart => [i, "auto", "auto", i],
                Anchor::TopEnd => [i, i, "auto", "auto"],
                Anchor::BottomStart => ["auto", "auto", i, i],
                Anchor::BottomEnd => ["auto", i, i, "auto"],
                Anchor::TopEdge => ["0px", "0px", "auto", "0px"],
                Anchor::BottomEdge => ["auto", "0px", "0px", "0px"],
            };
            put("inset", &format!("{top} {right} {bottom} {left}"));
        }
    }
}

fn flag(v: &str) -> Option<bool> {
    match v {
        "" | "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// The style's values the primitives read, resolved once at cook time.
#[derive(Clone, Debug, PartialEq)]
pub struct Scale {
    /// Each [`Gap`]'s length.
    pub gaps: [String; 6],
    /// [`Anchor`]'s inset.
    pub inset: String,
    /// [`Layout::Center`]'s widest line.
    pub measure: String,
}

impl Scale {
    /// Reads `--layout-gap` (base), `--layout-gap-{hair,tight,loose,wide}`,
    /// `--layout-anchor-inset` and `--layout-measure` from `tokens`; a
    /// missing one is `0px` (`none` for the measure).
    pub fn of(tokens: &Tokens) -> Scale {
        let read = |name: &str, or: &str| {
            tokens
                .substitute_str(&format!("var(--{name})"))
                .map_or_else(|| or.to_owned(), |v| v.trim().to_owned())
        };
        Scale {
            gaps: ["none", "hair", "tight", "base", "loose", "wide"].map(|g| match g {
                "none" => "0px".into(),
                "base" => read("layout-gap", "0px"),
                g => read(&format!("layout-gap-{g}"), "0px"),
            }),
            inset: read("layout-anchor-inset", "0px"),
            measure: read("layout-measure", "none"),
        }
    }
}
