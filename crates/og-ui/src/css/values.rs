//! CSS declaration values shared by document stylesheets and style packs.
//! [`STYLE`] maps reflected properties to their CSS spellings; [`decl_line`]
//! writes probe edits using the same value conversions as the reader.

use super::{
    boxes, calc, contain_css, image_css, interact_css, multicol_css, paint_css, position_css,
    scroll_css, tracks, view_transition,
};
use og_core::ogt::{Block, Entry, Number, Value};

mod declarations;
mod lower;
mod to_css;
mod tokens;

pub use declarations::{declarations, split_decls, zero_shift};
use lower::*;
pub use og_css::{Tok, camel, describe, quoted, tok_text, tokens};
pub use to_css::decl_line;
use tokens::*;
pub(crate) use tokens::{hex, paint_vals, want_len, want_lin, want_num};

/// How a property's value is written.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Conv {
    /// A length in CSS px: `12px` (`0` alone).
    Len,
    /// A plain number.
    Num,
    /// An angle: `0.5rad`, `30deg`, `0.25turn` (radians in the element).
    Angle,
    /// `var(--role)` or `#rrggbb[aa]`.
    Paint,
    /// `fit-content`, `stretch`, `<n>px`, `<n>%`, with `min-`/`max-`.
    Size,
    /// One to four lengths, CSS order (top right bottom left).
    Insets,
    /// One of two words for true and false.
    Bool(&'static str, &'static str),
    /// A word written in kebab case (`spaceBetween` ↔ `space-between`).
    Kebab,
    /// A word mapped through a table (element ↔ CSS).
    Kw(&'static [(&'static str, &'static str)]),
    /// A token: `var(--<namespace>-<name>)`.
    Var(&'static str),
    /// An identifier, or a string when it is not one.
    Ident,
    /// Quoted strings.
    Strs,
    /// Grid tracks: `120px 1fr auto 25% minmax(80px, 200px)`.
    Tracks,
    /// `none`, `<ms>ms [ease]`, `-og-spring(<ms>ms, <bounce>)`, `var(--motion-<name>)`.
    Transition,
    /// Two fractions as percentages: `50% 100%`.
    Origin,
    /// A margin edge: `<n>px`, `<n>em` (of its own type step) or `0`, as
    /// px and em.
    Margin,
    /// The CSS value as written, checked when it is read (`box-shadow`,
    /// `-og-text-path`'s shapes).
    Raw,
    /// Written with its partner in one property (`translate: <x> <y>`,
    /// `scale: <x> <y>`, `grid-column`/`grid-row`,
    /// `grid-auto-flow`).
    Joint,
}

const POSITIONS: &[(&str, &str)] = &[
    ("flow", "static"),
    ("absolute", "absolute"),
    ("screen", "fixed"),
    ("relative", "relative"),
    ("sticky", "sticky"),
];

/// Every element prop written as a declaration: (prop, CSS property, how).
/// A prop not here is an attribute line (`text "Buy"`, `variant primary`).
pub const STYLE: &[(&str, &str)] = &[
    ("width", "width"),
    ("height", "height"),
    ("padding", "padding"),
    ("paddingTop", "padding-top"),
    ("paddingRight", "padding-right"),
    ("paddingBottom", "padding-bottom"),
    ("paddingLeft", "padding-left"),
    ("minWidth", "min-width"),
    ("maxWidth", "max-width"),
    ("minHeight", "min-height"),
    ("maxHeight", "max-height"),
    ("boxSizing", "box-sizing"),
    ("marginTop", "margin-top"),
    ("marginRight", "margin-right"),
    ("marginBottom", "margin-bottom"),
    ("marginLeft", "margin-left"),
    ("gap", "gap"),
    ("align", "justify-content"),
    ("cross", "align-items"),
    ("wrap", "flex-wrap"),
    ("rowGap", "row-gap"),
    ("columnGap", "column-gap"),
    ("alignContent", "align-content"),
    ("justifyItems", "justify-items"),
    ("flexGrow", "flex-grow"),
    ("flexShrink", "flex-shrink"),
    ("flexBasis", "flex-basis"),
    ("order", "order"),
    ("alignSelf", "align-self"),
    ("justifySelf", "justify-self"),
    ("direction", "flex-direction"),
    ("labelDirection", "flex-direction"),
    ("background", "background"),
    ("radius", "border-radius"),
    ("radiusTopLeft", "border-top-left-radius"),
    ("radiusTopRight", "border-top-right-radius"),
    ("radiusBottomRight", "border-bottom-right-radius"),
    ("radiusBottomLeft", "border-bottom-left-radius"),
    ("cornerTopLeftShape", "corner-top-left-shape"),
    ("cornerTopRightShape", "corner-top-right-shape"),
    ("cornerBottomRightShape", "corner-bottom-right-shape"),
    ("cornerBottomLeftShape", "corner-bottom-left-shape"),
    ("borderTopWidth", "border-top-width"),
    ("borderTopStyle", "border-top-style"),
    ("borderTopColor", "border-top-color"),
    ("borderRightWidth", "border-right-width"),
    ("borderRightStyle", "border-right-style"),
    ("borderRightColor", "border-right-color"),
    ("borderBottomWidth", "border-bottom-width"),
    ("borderBottomStyle", "border-bottom-style"),
    ("borderBottomColor", "border-bottom-color"),
    ("borderLeftWidth", "border-left-width"),
    ("borderLeftStyle", "border-left-style"),
    ("borderLeftColor", "border-left-color"),
    ("outlineWidth", "outline-width"),
    ("outlineStyle", "outline-style"),
    ("outlineColor", "outline-color"),
    ("outlineOffset", "outline-offset"),
    ("boxShadow", "box-shadow"),
    ("backgroundImage", "background-image"),
    ("backgroundSize", "background-size"),
    ("backgroundPosition", "background-position"),
    ("backgroundRepeat", "background-repeat"),
    ("maskImage", "mask-image"),
    ("backgroundOrigin", "background-origin"),
    ("backgroundClip", "background-clip"),
    ("backgroundAttachment", "background-attachment"),
    ("backgroundBlendMode", "background-blend-mode"),
    ("maskSize", "mask-size"),
    ("maskPosition", "mask-position"),
    ("maskRepeat", "mask-repeat"),
    ("maskOrigin", "mask-origin"),
    ("maskClip", "mask-clip"),
    ("maskMode", "mask-mode"),
    ("maskComposite", "mask-composite"),
    ("borderImageSource", "border-image-source"),
    ("borderImageSlice", "border-image-slice"),
    ("borderImageWidth", "border-image-width"),
    ("borderImageOutset", "border-image-outset"),
    ("borderImageRepeat", "border-image-repeat"),
    ("maskBorderSource", "mask-border-source"),
    ("maskBorderSlice", "mask-border-slice"),
    ("maskBorderWidth", "mask-border-width"),
    ("maskBorderOutset", "mask-border-outset"),
    ("maskBorderRepeat", "mask-border-repeat"),
    ("maskBorderMode", "mask-border-mode"),
    ("backdropFilter", "backdrop-filter"),
    ("filter", "filter"),
    ("mixBlendMode", "mix-blend-mode"),
    ("isolation", "isolation"),
    ("textShadow", "text-shadow"),
    ("clipPath", "clip-path"),
    ("shapeOutside", "shape-outside"),
    ("shapeMargin", "shape-margin"),
    ("shapeImageThreshold", "shape-image-threshold"),
    ("material", "-og-material"),
    ("sound", "-og-sound"),
    ("scrollSnapType", "scroll-snap-type"),
    ("scrollSnapAlign", "scroll-snap-align"),
    ("overscrollBehaviorX", "overscroll-behavior-x"),
    ("overscrollBehaviorY", "overscroll-behavior-y"),
    ("scrollBehavior", "scroll-behavior"),
    ("scrollMarginTop", "scroll-margin-top"),
    ("scrollMarginRight", "scroll-margin-right"),
    ("scrollMarginBottom", "scroll-margin-bottom"),
    ("scrollMarginLeft", "scroll-margin-left"),
    ("scrollPaddingTop", "scroll-padding-top"),
    ("scrollPaddingRight", "scroll-padding-right"),
    ("scrollPaddingBottom", "scroll-padding-bottom"),
    ("scrollPaddingLeft", "scroll-padding-left"),
    ("scrollSnapStop", "scroll-snap-stop"),
    ("scrollbarColor", "scrollbar-color"),
    ("scrollbarGutter", "scrollbar-gutter"),
    ("overflowX", "overflow-x"),
    ("overflowY", "overflow-y"),
    ("scrollbarWidth", "scrollbar-width"),
    ("display", "display"),
    ("markerType", "list-style-type"),
    ("markerPosition", "list-style-position"),
    ("markerImage", "list-style-image"),
    ("aspect", "aspect-ratio"),
    ("fade", "-og-scroll-fade"),
    ("fadeInto", "-og-scroll-fade-into"),
    ("x", "translate"),
    ("y", "translate"),
    ("xShift", "translate"),
    ("yShift", "translate"),
    ("scale", "scale"),
    ("scaleX", "scale"),
    ("scaleY", "scale"),
    ("rotate", "rotate"),
    ("pivot", "transform-origin"),
    ("opacity", "opacity"),
    ("z", "z-index"),
    ("position", "position"),
    ("start", "inset-inline-start"),
    ("top", "top"),
    ("end", "inset-inline-end"),
    ("bottom", "bottom"),
    ("insetTop", "top"),
    ("insetRight", "right"),
    ("insetBottom", "bottom"),
    ("insetLeft", "left"),
    ("anchorName", "anchor-name"),
    ("viewTransitionName", "view-transition-name"),
    ("viewTransitionClass", "view-transition-class"),
    ("viewTransitionGroup", "view-transition-group"),
    ("positionAnchor", "position-anchor"),
    ("positionArea", "position-area"),
    ("positionTry", "position-try-fallbacks"),
    ("positionTryOrder", "position-try-order"),
    ("positionVisibility", "position-visibility"),
    ("anchorWidth", "width"),
    ("columnCount", "column-count"),
    ("columnWidth", "column-width"),
    ("columnSpan", "column-span"),
    ("columnFill", "column-fill"),
    ("columnRuleWidth", "column-rule-width"),
    ("columnRuleStyle", "column-rule-style"),
    ("columnRuleColor", "column-rule-color"),
    ("breakBefore", "break-before"),
    ("breakAfter", "break-after"),
    ("breakInside", "break-inside"),
    ("float", "float"),
    ("clear", "clear"),
    ("anchorHeight", "height"),
    ("columns", "grid-template-columns"),
    ("rows", "grid-template-rows"),
    ("areas", "grid-template-areas"),
    ("flow", "grid-auto-flow"),
    ("dense", "grid-auto-flow"),
    ("area", "grid-area"),
    ("cell", "grid-column"),
    ("span", "grid-column"),
    ("autoColumns", "grid-auto-columns"),
    ("autoRows", "grid-auto-rows"),
    ("columnStart", "grid-column-start"),
    ("columnEnd", "grid-column-end"),
    ("rowStart", "grid-row-start"),
    ("rowEnd", "grid-row-end"),
    ("motion", "transition"),
    ("layoutMotion", "-og-layout-transition"),
    ("cursor", "cursor"),
    ("pointerEvents", "pointer-events"),
    ("touchAction", "touch-action"),
    ("visibility", "visibility"),
    ("imageRendering", "image-rendering"),
    ("contain", "contain"),
    ("contentVisibility", "content-visibility"),
    ("containIntrinsicWidth", "contain-intrinsic-width"),
    ("containIntrinsicHeight", "contain-intrinsic-height"),
    ("resize", "resize"),
    ("fieldSizing", "field-sizing"),
    ("willChange", "will-change"),
    ("objectPosition", "object-position"),
    ("userSelect", "user-select"),
    ("textPath", "-og-text-path"),
    ("textPathOffset", "-og-text-path-offset"),
    ("textPathSide", "-og-text-path-side"),
    ("step", "font"),
    ("fitMin", "-og-fit-min"),
    ("font", "font-family"),
    ("color", "color"),
    ("uppercase", "text-transform"),
    ("textAlign", "text-align"),
    ("verticalAlign", "vertical-align"),
    ("captionSide", "caption-side"),
    ("nowrap", "white-space"),
    ("maxLines", "line-clamp"),
    ("ellipsis", "text-overflow"),
    ("ellipsisText", "text-overflow"),
    ("fit", "object-fit"),
    ("uv", "object-view-box"),
    ("appearance", "appearance"),
];

/// Reflected fields affected even when a winning declaration is invalid.
pub(crate) fn targets(name: &str) -> impl Iterator<Item = &'static str> + '_ {
    let alias = match name {
        "display" => Some("visible"),
        _ => None,
    };
    let both: Vec<&'static str> = match name {
        "margin" => vec!["marginTop", "marginRight", "marginBottom", "marginLeft"],
        "margin-block" => vec!["marginTop", "marginBottom"],
        "inset" => vec!["insetTop", "insetRight", "insetBottom", "insetLeft"],
        "inset-block" => vec!["insetTop", "insetBottom"],
        "inset-inline" => vec!["insetLeft", "insetRight"],
        "inset-block-start" => vec!["insetTop"],
        "inset-block-end" => vec!["insetBottom"],
        "inset-inline-start" => vec!["insetLeft"],
        "inset-inline-end" => vec!["insetRight"],
        "position-try" => vec!["positionTry", "positionTryOrder"],
        "columns" => vec!["columnWidth", "columnCount"],
        "contain-intrinsic-size" => vec!["containIntrinsicWidth", "containIntrinsicHeight"],
        "contain-intrinsic-inline-size" => vec!["containIntrinsicWidth"],
        "contain-intrinsic-block-size" => vec!["containIntrinsicHeight"],
        "column-rule" => vec!["columnRuleWidth", "columnRuleStyle", "columnRuleColor"],
        "page-break-before" => vec!["breakBefore"],
        "page-break-after" => vec!["breakAfter"],
        "page-break-inside" => vec!["breakInside"],
        "margin-inline" => vec!["marginRight", "marginLeft"],
        "margin-inline-start" => vec!["marginLeft"],
        "margin-inline-end" => vec!["marginRight"],
        "margin-block-start" => vec!["marginTop"],
        "margin-block-end" => vec!["marginBottom"],
        "padding" => vec!["paddingTop", "paddingRight", "paddingBottom", "paddingLeft"],
        "flex" => vec!["flexGrow", "flexShrink", "flexBasis"],
        "flex-flow" => vec!["direction", "labelDirection", "wrap", "wrapReverse"],
        "flex-wrap" => vec!["wrapReverse"],
        "justify-content" => vec!["safeJustify"],
        "align-items" | "align-self" => vec!["safeAlign"],
        "gap" => vec!["rowGap", "columnGap"],
        "grid-column" => vec!["cell", "span", "columnStart", "columnEnd"],
        "grid-row" => vec!["cell", "span", "rowStart", "rowEnd"],
        "grid-area" => vec!["area", "columnStart", "columnEnd", "rowStart", "rowEnd"],
        "grid-template" => vec!["columns", "rows", "areas"],
        "grid" => vec![
            "columns",
            "rows",
            "areas",
            "autoColumns",
            "autoRows",
            "flow",
            "dense",
        ],
        "place-content" => vec!["alignContent", "align", "safeJustify"],
        "place-items" => vec!["cross", "justifyItems", "safeAlign"],
        "place-self" => vec!["alignSelf", "justifySelf", "safeAlign"],
        "mask" => ["maskImage"]
            .iter()
            .chain(&super::paint_css::C4_PROPS[4..11])
            .chain(&super::paint_css::C4_PROPS[16..])
            .copied()
            .collect(),
        "border-image" => super::paint_css::C4_PROPS[11..16].to_vec(),
        "mask-border" => super::paint_css::C4_PROPS[16..].to_vec(),
        "padding-block" => vec!["paddingTop", "paddingBottom"],
        "padding-inline" => vec!["paddingRight", "paddingLeft"],
        "padding-block-start" => vec!["paddingTop"],
        "padding-block-end" => vec!["paddingBottom"],
        "padding-inline-start" => vec!["paddingLeft"],
        "padding-inline-end" => vec!["paddingRight"],
        "min-width" | "min-inline-size" => vec!["minWidth"],
        "max-width" | "max-inline-size" => vec!["maxWidth"],
        "min-height" | "min-block-size" => vec!["minHeight"],
        "max-height" | "max-block-size" => vec!["maxHeight"],
        n if n.starts_with("scroll-") || n.starts_with("overscroll-") => {
            super::scroll_css::targets(n)
        }
        _ => super::boxes::longhands(name),
    };
    let target = match name {
        "grid-row" => "grid-column",
        "min-width" | "max-width" | "inline-size" | "min-inline-size" | "max-inline-size" => {
            "width"
        }
        "min-height" | "max-height" | "block-size" | "min-block-size" | "max-block-size" => {
            "height"
        }
        _ => name,
    };
    alias.into_iter().chain(both).chain(
        STYLE
            .iter()
            .filter_map(move |(prop, css)| (*css == target).then_some(*prop)),
    )
}

fn conv(prop: &str) -> Conv {
    match prop {
        p if super::interact_css::form(p).is_some() => Conv::Raw,
        p if super::scroll_css::form(p).is_some() => Conv::Raw,
        p if super::image_css::form(p).is_some() => Conv::Raw,
        p if super::contain_css::form(p).is_some() => Conv::Raw,
        "width" | "height" | "minWidth" | "maxWidth" | "minHeight" | "maxHeight" => Conv::Size,
        "padding" => Conv::Insets,
        "paddingTop" | "paddingRight" | "paddingBottom" | "paddingLeft" => Conv::Len,
        "boxSizing" | "alignContent" | "justifyItems" | "alignSelf" | "justifySelf" => Conv::Kebab,
        "flexGrow" | "flexShrink" | "order" => Conv::Num,
        "flexBasis" => Conv::Size,
        "rowGap" | "columnGap" => Conv::Len,
        p if p.starts_with("border") && p.ends_with("Width") => Conv::Len,
        p if p.starts_with("border") && p.ends_with("Color") => Conv::Paint,
        p if p.starts_with("border") && p.ends_with("Style") => Conv::Kebab,
        "outlineWidth" | "outlineOffset" => Conv::Len,
        "outlineColor" => Conv::Paint,
        "outlineStyle" | "overflowX" | "overflowY" | "scrollbarWidth" => Conv::Kebab,
        "boxShadow" | "uv" | "sound" | "markerImage" => Conv::Raw,
        p if super::paint_css::C4_PROPS.contains(&p) || p == "ellipsisText" => Conv::Raw,
        "insetTop" | "insetRight" | "insetBottom" | "insetLeft" | "anchorName"
        | "positionAnchor" | "positionArea" | "positionTry" | "positionTryOrder"
        | "positionVisibility" | "anchorWidth" | "anchorHeight" => Conv::Raw,
        "columnCount" | "columnWidth" | "columnSpan" | "columnFill" | "columnRuleWidth"
        | "columnRuleStyle" | "columnRuleColor" | "breakBefore" | "breakAfter" | "breakInside"
        | "float" | "clear" => Conv::Raw,
        "viewTransitionName" | "viewTransitionClass" | "viewTransitionGroup" => Conv::Raw,
        "backgroundImage"
        | "backgroundSize"
        | "backgroundPosition"
        | "backgroundRepeat"
        | "maskImage"
        | "backdropFilter"
        | "filter"
        | "mixBlendMode"
        | "isolation"
        | "textShadow"
        | "clipPath"
        | "material"
        | "scrollSnapType"
        | "scrollSnapAlign"
        | "radiusTopLeft"
        | "radiusTopRight"
        | "radiusBottomRight"
        | "radiusBottomLeft"
        | "cornerTopLeftShape"
        | "cornerTopRightShape"
        | "cornerBottomRightShape"
        | "cornerBottomLeftShape" => Conv::Raw,
        "marginTop" | "marginRight" | "marginBottom" | "marginLeft" => Conv::Margin,
        "gap" | "radius" | "fade" | "start" | "top" | "end" | "bottom" => Conv::Len,
        "aspect" | "opacity" | "z" | "scale" | "maxLines" => Conv::Num,
        "rotate" => Conv::Angle,
        "background" | "fadeInto" | "color" | "ink" => Conv::Paint,
        "align" | "cross" | "direction" | "frame" | "textAlign" | "verticalAlign"
        | "captionSide" | "fit" | "display" | "markerType" | "markerPosition" | "userSelect"
        | "appearance" | "labelDirection" | "fitMin" => Conv::Kebab,
        "fieldSizing" => Conv::Kebab,
        "wrap" => Conv::Bool("wrap", "nowrap"),
        "uppercase" => Conv::Bool("uppercase", "none"),
        "nowrap" => Conv::Bool("nowrap", "normal"),
        "ellipsis" => Conv::Bool("ellipsis", "clip"),
        "position" => Conv::Kw(POSITIONS),
        "step" => Conv::Var("text"),
        "font" => Conv::Var("font"),
        "textPath" | "textPathOffset" => Conv::Raw,
        "textPathSide" => Conv::Kebab,
        "area" => Conv::Ident,
        "areas" => Conv::Strs,
        "columns" | "rows" | "autoColumns" | "autoRows" => Conv::Tracks,
        "columnStart" | "columnEnd" | "rowStart" | "rowEnd" => Conv::Raw,
        "motion" | "layoutMotion" => Conv::Transition,
        "pivot" => Conv::Origin,
        _ => Conv::Joint,
    }
}

/// How a declared prop's value is written in CSS (the catalogue's
/// column); `None` for a prop written with its partner or as an attribute.
/// `words` are a word prop's values as the element names them.
pub fn value_form(prop: &str, words: &[&str]) -> Option<String> {
    if let Some(j) = joint_form(css_name(prop)?) {
        return Some(j.into());
    }
    Some(match conv(prop) {
        Conv::Len => "<n>px".into(),
        Conv::Num => "<number>".into(),
        Conv::Angle => "<n>deg | <n>rad | <n>turn".into(),
        Conv::Paint => "var(--<role>) | #rrggbb[aa]".into(),
        Conv::Size => {
            "auto | fit-content | fit-content(<n>px) | min-content | max-content | stretch | <n>px | <n>% | calc(…) | min(…) | max(…) | clamp(…) | round(…) …"
                .into()
        }
        Conv::Insets => "1 to 4 lengths (top right bottom left)".into(),
        Conv::Margin => "<n>px | <n>em | <n>% | auto | calc(…)".into(),
        Conv::Raw if let Some(f) = super::interact_css::form(prop) => f.into(),
        Conv::Raw if let Some(f) = super::scroll_css::form(prop) => f.into(),
        Conv::Raw if let Some(f) = super::image_css::form(prop) => f.into(),
        Conv::Raw if let Some(f) = super::view_transition::form(prop) => f.into(),
        Conv::Raw if let Some(f) = super::contain_css::form(prop) => f.into(),
        Conv::Raw if prop == "boxShadow" => {
            "[inset] <x> <y> [<blur> [<spread>]] [<colour>], … | none".into()
        }
        Conv::Raw if prop == "sound" => crate::sound_css::GRAMMAR.into(),
        Conv::Bool(t, f) => format!("{t} | {f}"),
        Conv::Kebab => words
            .iter()
            .map(|w| kebab(w))
            .collect::<Vec<_>>()
            .join(" | "),
        Conv::Kw(t) => t.iter().map(|(_, c)| *c).collect::<Vec<_>>().join(" | "),
        Conv::Var(ns) => format!("var(--{ns}-<name>)"),
        Conv::Ident => "<name>".into(),
        Conv::Strs => "\"<a>\" \"<b>\" …".into(),
        Conv::Tracks => {
            "[name] 120px 1fr auto 25% minmax(min-content, 1fr) repeat(auto-fill, 200px) | subgrid | masonry".into()
        }
        Conv::Transition => {
            "none | <ms>ms [ease] | -og-spring(<ms>ms, <bounce>) | var(--motion-<name>)".into()
        }
        Conv::Origin => "<x>% <y>%".into(),
        Conv::Raw if prop.starts_with("radius") => "<h> [<v>] (<n>px | <n>%)".into(),
        Conv::Raw if prop.starts_with("corner") => {
            "round | squircle | bevel | scoop | notch | square | superellipse(<K>)".into()
        }
        Conv::Raw if prop == "uv" => "inset(<t>% [<r>% [<b>% [<l>%]]])".into(),
        Conv::Raw if prop == "markerImage" => "none | url(<image>)".into(),
        Conv::Raw if prop == "maskImage" => {
            "linear-gradient(…) | radial-gradient(…) | conic-gradient(…) (repeating- too)".into()
        }
        Conv::Raw if prop == "backgroundImage" => {
            "<image> [<position> [/ <size>]] [<repeat>], … [<colour>], an <image> url(<file>) or a gradient".into()
        }
        Conv::Raw if prop == "backgroundSize" => {
            "cover | contain | [<n>px | <n>% | auto]{1,2}, …".into()
        }
        Conv::Raw if prop == "backgroundPosition" => {
            "[left | center | right | top | bottom | <n>px | <n>%]{1,4}, …".into()
        }
        Conv::Raw if prop == "backgroundRepeat" => {
            "repeat-x | repeat-y | [repeat | no-repeat | space | round]{1,2}, …".into()
        }
        Conv::Raw if prop == "backdropFilter" || prop == "filter" => {
            "blur(<n>px) brightness(<n>) contrast(<n>) drop-shadow([<colour>] <x> <y> [<blur>]) grayscale(<n>) hue-rotate(<angle>) invert(<n>) opacity(<n>) saturate(<n>) sepia(<n>) … | none".into()
        }
        Conv::Raw if prop == "mixBlendMode" => {
            "normal | multiply | screen | plus-lighter | overlay | soft-light | color-dodge".into()
        }
        Conv::Raw if prop == "isolation" => "auto | isolate".into(),
        Conv::Raw if prop == "textShadow" => "<x> <y> [<blur>] [<colour>], … | none".into(),
        Conv::Raw if prop == "clipPath" => {
            "[inset(…) | rect(…) | xywh(…) | circle(…) | ellipse(…) | polygon(…) | path(\"…\") | shape(…)] [<geometry-box>] | <geometry-box> | url(#id) | none".into()
        }
        Conv::Raw if prop == "scrollSnapType" => {
            "none | x | y | block | inline | both [mandatory | proximity]".into()
        }
        Conv::Raw if prop == "scrollSnapAlign" => "none | start | center | end [<inline>]".into(),
        Conv::Raw if prop == "material" => {
            "glass <regular|thick> <lift|float> | <material>(<n> …[, <colour>]) | \"<ui-material asset>\" <n> … | none".into()
        }
        Conv::Raw if prop == "textPath" => {
            "none | path(\"<svg path data>\") | circle([<n>px])".into()
        }
        Conv::Raw if prop.ends_with("Start") || prop.ends_with("End") => {
            "auto | <n> [<name>] | <name> | span [<n>] [<name>]".into()
        }
        Conv::Raw => "<n>px | <n>%".into(),
        Conv::Joint => return None,
    })
}

/// How a property that sets several props at once is written.
fn joint_form(css: &str) -> Option<&'static str> {
    Some(match css {
        "translate" => "none | <x> [<y>]: px, or % of its own box",
        "scale" => "<both> | <x> <y>",
        "grid-column" => "<n> | <n> / span <n> | span <n>",
        "grid-auto-flow" => "row | column [dense]",
        _ => return None,
    })
}

/// The CSS property an element prop is written as; `None`: an attribute.
pub fn css_name(prop: &str) -> Option<&'static str> {
    STYLE.iter().find(|(p, _)| *p == prop).map(|(_, c)| *c)
}

/// The values a state of a group sets, as their CSS properties.
const STATE_VALUES: &[(&str, &str)] = &[
    ("x", "translate"),
    ("y", "translate"),
    ("scale", "scale"),
    ("scaleX", "scale"),
    ("scaleY", "scale"),
    ("rotate", "rotate"),
    ("opacity", "opacity"),
    ("radius", "border-radius"),
    ("z", "z-index"),
    ("background", "background"),
    ("ink", "color"),
];

/// The neutral value of an animatable value (a state's `normal` takes it
/// when the element declares none).
fn neutral(key: &str) -> Option<&'static str> {
    match key {
        "x" | "y" | "rotate" => Some("0"),
        "scale" | "scaleX" | "scaleY" | "opacity" => Some("1"),
        _ => None,
    }
}

/// CSS that is not part of the subset, and what to write instead.
pub(crate) fn unsupported(name: &str) -> Option<&'static str> {
    Some(match name {
        "visibility" => "write the attribute `visible false` (or `visible bind …`)",
        "-og-meta" => {
            "write <og-overlay effect=\"vignette #a00\"> (tint <colour>, material <material>) and give it opacity"
        }
        "-og-overlaps" => {
            "an overlap is a finding of og.ui.overlaps and og ui check, not a style; one the design means stays as it is"
        }
        "-og-reduced-motion" => {
            "write the element's transition under @media (prefers-reduced-motion: reduce) { … } (transition: none jumps; one written there plays as written)"
        }
        _ => return None,
    })
}

// ---- words and numbers

/// `spaceBetween` → `space-between`.
pub(crate) fn kebab(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_uppercase() {
            out.push('-');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// `space-between` → `spaceBetween`.
/// A decimal text times 10^k, exactly (`0.25`, 2 → `25`).
pub(crate) fn shift(text: &str, k: i32) -> String {
    if text.contains(['e', 'E']) {
        let v: f64 = text.parse().unwrap_or(0.0);
        return format!("{}", v * 10f64.powi(k));
    }
    let (neg, t) = match text.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, text),
    };
    let (int, frac) = t.split_once('.').unwrap_or((t, ""));
    let digits: String = format!("{int}{frac}");
    let point = int.len() as i32 + k;
    let mut s = if point <= 0 {
        format!("0.{}{digits}", "0".repeat((-point) as usize))
    } else if point as usize >= digits.len() {
        format!("{digits}{}", "0".repeat(point as usize - digits.len()))
    } else {
        format!(
            "{}.{}",
            &digits[..point as usize],
            &digits[point as usize..]
        )
    };
    if s.contains('.') {
        s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    }
    let s = s.trim_start_matches('0');
    let s = if s.is_empty() || s.starts_with('.') {
        format!("0{s}")
    } else {
        s.to_string()
    };
    if neg && s != "0" { format!("-{s}") } else { s }
}

pub(crate) fn number(text: &str) -> Result<Value, String> {
    Number::parse(text)
        .map(Value::Number)
        .map_err(|_| format!("'{text}' is not a number"))
}

fn text_of(v: &Value) -> String {
    match v {
        Value::Number(n) => n.text().to_string(),
        Value::String(s) | Value::Word(s) => s.clone(),
    }
}

fn is_ident(s: &str) -> bool {
    let mut c = s.chars();
    matches!(c.next(), Some(f) if f.is_ascii_alphabetic() || f == '_')
        && c.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_shifts_are_exact() {
        assert_eq!(shift("0.25", 2), "25");
        assert_eq!(shift("0.5", 2), "50");
        assert_eq!(shift("1", 2), "100");
        assert_eq!(shift("33.3", -2), "0.333");
        assert_eq!(shift("-4", -2), "-0.04");
        assert_eq!(shift("0.3", 3), "300");
        assert_eq!(shift("0", 2), "0");
    }

    #[test]
    fn kebab_and_camel_are_inverse() {
        for w in [
            "spaceBetween",
            "inkMuted",
            "xl2",
            "displaySm",
            "team1",
            "slideUp",
        ] {
            assert_eq!(camel(&kebab(w)), w);
        }
    }
}
