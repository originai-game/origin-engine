//! CSS declaration values shared by document stylesheets and style packs.
//! [`STYLE`] maps reflected properties to their CSS spellings; [`decl_line`]
//! writes probe edits using the same value conversions as the reader.

use og_core::ogt::{self, Block, Entry, Number, Value};

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
    /// `<kind> [<distance>px] [<transition>]`.
    Presence,
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
    ("overlaps", "-og-overlaps"),
    ("motion", "transition"),
    ("layoutMotion", "-og-layout-transition"),
    ("enter", "-og-enter"),
    ("exit", "-og-exit"),
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
    ("willChange", "will-change"),
    ("objectPosition", "object-position"),
    ("meta", "-og-meta"),
    ("userSelect", "user-select"),
    ("textPath", "-og-text-path"),
    ("textPathOffset", "-og-text-path-offset"),
    ("textPathSide", "-og-text-path-side"),
    ("reduced", "-og-reduced-motion"),
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
        "align" | "cross" | "direction" | "frame" | "reduced" | "textAlign" | "verticalAlign"
        | "captionSide" | "fit" | "display" | "markerType" | "markerPosition" | "userSelect"
        | "appearance" | "labelDirection" | "fitMin" => Conv::Kebab,
        "wrap" => Conv::Bool("wrap", "nowrap"),
        "uppercase" => Conv::Bool("uppercase", "none"),
        "nowrap" => Conv::Bool("nowrap", "normal"),
        "ellipsis" => Conv::Bool("ellipsis", "clip"),
        "position" => Conv::Kw(POSITIONS),
        "step" => Conv::Var("text"),
        "font" => Conv::Var("font"),
        "textPath" | "textPathOffset" | "meta" => Conv::Raw,
        "textPathSide" => Conv::Kebab,
        "area" => Conv::Ident,
        "areas" | "overlaps" => Conv::Strs,
        "columns" | "rows" | "autoColumns" | "autoRows" => Conv::Tracks,
        "columnStart" | "columnEnd" | "rowStart" | "rowEnd" => Conv::Raw,
        "motion" | "layoutMotion" => Conv::Transition,
        "enter" | "exit" => Conv::Presence,
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
        Conv::Presence => "<kind> [<distance>px] [<transition>]".into(),
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
            "<image> [<position> [/ <size>]] [<repeat>], … [<colour>], an <image> url(<icon or texture>) or a gradient".into()
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
pub(crate) fn camel(s: &str) -> String {
    let mut out = String::new();
    let mut up = false;
    for c in s.chars() {
        if c == '-' {
            up = true;
        } else if up {
            out.push(c.to_ascii_uppercase());
            up = false;
        } else {
            out.push(c);
        }
    }
    out
}

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

pub(crate) fn quoted(s: &str) -> String {
    ogt::value_text(&Value::String(s.to_string())).unwrap_or_default()
}

fn is_ident(s: &str) -> bool {
    let mut c = s.chars();
    matches!(c.next(), Some(f) if f.is_ascii_alphabetic() || f == '_')
        && c.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

// ---- element values to CSS

fn px(v: &Value) -> Result<String, String> {
    match v {
        Value::Number(n) if n.text() == "0" => Ok("0".into()),
        Value::Number(n) => Ok(format!("{}px", n.text())),
        Value::String(s) if s.starts_with("var(--") => Ok(s.clone()),
        other => Err(format!("'{}' is not a length", text_of(other))),
    }
}

fn bound(vals: &[Value]) -> Option<Result<String, String>> {
    match vals {
        [Value::Word(b), e] if b == "bind" => {
            let src = text_of(e);
            let simple = !src.is_empty()
                && src
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '$'));
            Some(Ok(if simple {
                format!("bind({src})")
            } else {
                format!("bind({})", quoted(&src))
            }))
        }
        [Value::Word(b), ..] if b == "bind" => Some(Err(
            "a declaration binds without a format (a format is a text's: `text bind gold compact`)"
                .into(),
        )),
        _ => None,
    }
}

fn paint_css(vals: &[Value]) -> Result<String, String> {
    match vals {
        [Value::Word(t), Value::Word(role)] if t == "token" => {
            Ok(format!("var(--{})", kebab(role)))
        }
        [Value::String(s)] | [Value::Word(s)] if s.starts_with('#') => Ok(s.clone()),
        [Value::Word(role)] | [Value::String(role)] => Ok(format!("var(--{})", kebab(role))),
        _ => Err("a paint: token <role> or \"#rrggbb\"".into()),
    }
}

fn transition_css(vals: &[Value]) -> Result<String, String> {
    let w: Vec<String> = vals.iter().map(text_of).collect();
    let num = |i: usize| w.get(i).cloned().ok_or("a transition needs its numbers");
    match w.first().map(String::as_str) {
        Some("instant") => Ok("none".into()),
        Some("spring") => Ok(match w.get(2) {
            Some(b) => format!("-og-spring({}ms, {b})", num(1)?),
            None => format!("-og-spring({}ms)", num(1)?),
        }),
        Some("tween") => {
            let ease = match w.get(2).map(String::as_str) {
                None => "",
                Some("linear") => " linear",
                Some("out") => " ease-out",
                Some("exit") => " ease-in",
                Some("inOut") => " ease-in-out",
                Some(e) => return Err(format!("'{e}' is not an ease")),
            };
            Ok(format!("{}ms{ease}", num(1)?))
        }
        Some("token") => Ok(format!("var(--motion-{})", kebab(&num(1)?))),
        Some(t) => Ok(format!("var(--motion-{})", kebab(t))),
        None => Err("a transition".into()),
    }
}

fn presence_css(vals: &[Value]) -> Result<String, String> {
    let kind = vals
        .first()
        .map(text_of)
        .ok_or("a presence: fade, slideUp …")?;
    let mut out = kebab(&kind);
    let mut rest = &vals[1..];
    if let Some(v) = rest.first()
        && px(v).is_ok()
    {
        out.push(' ');
        out.push_str(&px(v)?);
        rest = &rest[1..];
    }
    if !rest.is_empty() {
        out.push(' ');
        out.push_str(&transition_css(rest)?);
    }
    Ok(out)
}

fn track_css(v: &Value) -> Result<String, String> {
    match v {
        Value::Number(_) => px(v),
        Value::Word(s) | Value::String(s) => match s.strip_prefix("minmax:") {
            Some(r) => {
                let (a, b) = r.split_once(':').ok_or("minmax:<min>:<max>")?;
                let len = |t: &str| {
                    if t == "0" || t.starts_with("var(--") {
                        t.to_string()
                    } else {
                        format!("{t}px")
                    }
                };
                Ok(format!("minmax({}, {})", len(a), len(b)))
            }
            None => Ok(s.clone()),
        },
    }
}

/// One element value as its CSS value (`conv` of a prop not written jointly).
fn value_css(prop: &str, c: Conv, vals: &[Value]) -> Result<String, String> {
    if let Some(b) = bound(vals) {
        return b;
    }
    let one = || match vals {
        [v] => Ok(v),
        _ => Err(format!("{prop} takes one value")),
    };
    match c {
        Conv::Len => px(one()?),
        Conv::Num => match one()? {
            Value::Number(n) => Ok(n.text().to_string()),
            v => Err(format!("'{}' is not a number", text_of(v))),
        },
        Conv::Angle => match one()? {
            Value::Number(n) if n.text() == "0" => Ok("0".into()),
            Value::Number(n) => Ok(format!("{}rad", n.text())),
            v => Err(format!("'{}' is not an angle", text_of(v))),
        },
        Conv::Paint => paint_css(vals),
        Conv::Margin => match vals {
            [_, _, _, Value::Number(a)] if a.text() != "0" => Ok("auto".into()),
            [Value::Number(p), Value::Number(e), Value::Number(c), _] => {
                let pct = super::calc::number(c.as_f32() as f64 * 100.0);
                let terms: Vec<String> = [
                    (pct, "%"),
                    (p.text().to_string(), "px"),
                    (e.text().to_string(), "em"),
                ]
                .into_iter()
                .filter(|(v, _)| v != "0")
                .map(|(v, u)| format!("{v}{u}"))
                .collect();
                Ok(match terms.as_slice() {
                    [] => "0".into(),
                    [t] => t.clone(),
                    _ => format!("calc({})", terms.join(" + ").replace("+ -", "- ")),
                })
            }
            _ => Err(format!("{prop} takes a length or auto")),
        },
        Conv::Insets => {
            let n: Vec<String> = vals.iter().map(px).collect::<Result<_, _>>()?;
            Ok(match n.as_slice() {
                [a] => a.clone(),
                [x, y] => format!("{y} {x}"),
                [l, t, r, b] => format!("{t} {r} {b} {l}"),
                _ => return Err("padding takes 1, 2 or 4 numbers".into()),
            })
        }
        Conv::Bool(t, f) => match vals {
            [] => Ok(t.into()),
            [v] => match v.as_bool() {
                Some(true) => Ok(t.into()),
                Some(false) => Ok(f.into()),
                None => Err(format!("{prop} takes true or false")),
            },
            _ => Err(format!("{prop} takes true or false")),
        },
        Conv::Kebab => Ok(kebab(&text_of(one()?))),
        Conv::Kw(table) => {
            let w = text_of(one()?);
            table
                .iter()
                .find(|(e, _)| *e == w)
                .map(|(_, c)| c.to_string())
                .ok_or_else(|| format!("'{w}' is not a {prop}"))
        }
        Conv::Var(ns) => Ok(format!("var(--{ns}-{})", kebab(&text_of(one()?)))),
        Conv::Raw => Ok(text_of(one()?)),
        Conv::Ident => {
            let w = text_of(one()?);
            Ok(if is_ident(&w) { w } else { quoted(&w) })
        }
        Conv::Strs => Ok(vals
            .iter()
            .map(|v| quoted(&text_of(v)))
            .collect::<Vec<_>>()
            .join(" ")),
        Conv::Tracks => Ok(vals
            .iter()
            .map(track_css)
            .collect::<Result<Vec<_>, _>>()?
            .join(" ")),
        Conv::Transition => transition_css(vals),
        Conv::Presence => presence_css(vals),
        Conv::Origin => match vals {
            [Value::Number(a), Value::Number(b)] => {
                Ok(format!("{}% {}%", shift(a.text(), 2), shift(b.text(), 2)))
            }
            _ => Err("pivot takes two fractions".into()),
        },
        Conv::Size => {
            if let [v] = vals
                && px(v).is_ok()
            {
                return px(v);
            }
            let w = vals.first().map(text_of).unwrap_or_default();
            match w.as_str() {
                "fixed" => px(vals.get(1).ok_or("fixed <px>")?),
                "fit" => Ok("fit-content".into()),
                "grow" => Ok("stretch".into()),
                "minContent" => Ok("min-content".into()),
                "maxContent" => Ok("max-content".into()),
                "percent" => Ok(format!(
                    "{}%",
                    shift(&vals.get(1).map(text_of).ok_or("percent <0–1>")?, 2)
                )),
                "calc" => {
                    let n: Vec<f64> = vals[2..]
                        .iter()
                        .map(|v| v.as_number().map_or(0.0, |n| n.as_f32() as f64))
                        .collect();
                    let sum = |k: usize| {
                        let (pct, px) = (
                            n.get(2 * k).copied().unwrap_or(0.0),
                            n.get(2 * k + 1).copied().unwrap_or(0.0),
                        );
                        super::calc::sum_text(
                            &[("%", pct * 100.0), ("px", px)]
                                .into_iter()
                                .filter(|(_, v)| *v != 0.0)
                                .collect::<Vec<_>>(),
                        )
                    };
                    let op = vals.get(1).map(text_of).unwrap_or_default();
                    let k = n.len() / 2;
                    let list = (0..k).map(sum).collect::<Vec<_>>().join(", ");
                    Ok(match op.as_str() {
                        "sum" => sum(0),
                        "clamp" => format!("clamp({}, {}, {})", sum(0), sum(1), sum(2)),
                        "roundUp" => format!("round(up, {list})"),
                        "roundDown" => format!("round(down, {list})"),
                        "roundToZero" => format!("round(to-zero, {list})"),
                        op => format!("{op}({list})"),
                    })
                }
                _ => Err("a size".into()),
            }
        }
        Conv::Joint => Err(format!("{prop} is written with its partner")),
    }
}

/// The declarations of `props` (element props, or a state's values when
/// `state`), in order, as one line: `width: stretch; max-width: 640px`.
/// Props written jointly (`x` and `y` as `translate`) share one
/// declaration; one missing its partner takes `partner(prop)`'s value
/// (the element's own, else its neutral value).
pub(crate) fn decl_line(
    props: &[(String, Vec<Value>)],
    state: bool,
    partner: &dyn Fn(&str) -> Option<Vec<Value>>,
) -> Result<String, String> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut done: Vec<&str> = Vec::new();
    let get = |k: &str| props.iter().find(|(p, _)| p == k).map(|(_, v)| v.clone());
    let fill = |k: &str| -> Result<String, String> {
        let v = get(k)
            .or_else(|| partner(k))
            .or_else(|| neutral(k).map(|n| vec![Value::Number(Number::parse(n).unwrap())]))
            .ok_or_else(|| format!("{k} has no value"))?;
        if let Some(b) = bound(&v) {
            return b;
        }
        match conv_joint(k) {
            Conv::Angle => value_css(k, Conv::Angle, &v),
            Conv::Len => value_css(k, Conv::Len, &v),
            _ => value_css(k, Conv::Num, &v),
        }
    };
    for (prop, vals) in props {
        let prop = prop.as_str();
        if done.contains(&prop) {
            continue;
        }
        let css = if state {
            STATE_VALUES
                .iter()
                .find(|(p, _)| *p == prop)
                .map(|(_, c)| *c)
        } else {
            css_name(prop)
        }
        .ok_or_else(|| format!("'{prop}' has no CSS form"))?;
        match prop {
            "x" | "y" => {
                done.extend(["x", "y"]);
                out.push((css.into(), format!("{} {}", fill("x")?, fill("y")?)));
            }
            "scaleX" | "scaleY" => {
                done.extend(["scaleX", "scaleY"]);
                out.push((
                    css.into(),
                    format!("{} {}", fill("scaleX")?, fill("scaleY")?),
                ));
            }
            "flow" | "dense" => {
                done.extend(["flow", "dense"]);
                let mut v = Vec::new();
                if let Some(f) = get("flow") {
                    v.push(value_css("flow", Conv::Kebab, &f)?);
                }
                if let Some(d) = get("dense")
                    && d.first().and_then(Value::as_bool).unwrap_or(true)
                {
                    v.push("dense".into());
                }
                if v.is_empty() {
                    v.push("row".into());
                }
                out.push((css.into(), v.join(" ")));
            }
            "cell" | "span" => {
                done.extend(["cell", "span"]);
                let pair = |k: &str| -> Result<Option<[String; 2]>, String> {
                    match get(k).as_deref() {
                        None => Ok(None),
                        Some([a, b]) => Ok(Some([text_of(a), text_of(b)])),
                        Some(_) => Err(format!("{k} takes two numbers")),
                    }
                };
                let (cell, span) = (pair("cell")?, pair("span")?);
                for (axis, name) in [(0, "grid-column"), (1, "grid-row")] {
                    let start = cell.as_ref().map(|c| c[axis].clone()).filter(|s| s != "0");
                    let n = span.as_ref().map(|c| c[axis].clone()).filter(|s| s != "0");
                    let v = match (start, n) {
                        (Some(s), Some(n)) => format!("{s} / span {n}"),
                        (Some(s), None) => s,
                        (None, Some(n)) => format!("span {n}"),
                        (None, None) => continue,
                    };
                    out.push((name.into(), v));
                }
            }
            "width" | "height" if !state => {
                done.push(prop);
                if bound(vals).is_some() {
                    out.push((css.into(), value_css(prop, Conv::Len, vals)?));
                    continue;
                }
                let w = vals.first().map(text_of).unwrap_or_default();
                match w.as_str() {
                    "fit" | "grow" => {
                        out.push((
                            css.into(),
                            if w == "fit" { "fit-content" } else { "stretch" }.into(),
                        ));
                        if let Some(min) = vals.get(1)
                            && text_of(min) != "0"
                        {
                            out.push((format!("min-{css}"), px(min)?));
                        }
                        if let Some(max) = vals.get(2) {
                            out.push((format!("max-{css}"), px(max)?));
                        }
                    }
                    _ => out.push((css.into(), value_css(prop, Conv::Size, vals)?)),
                }
            }
            _ => {
                done.push(prop);
                let c = if prop == "ink" {
                    Conv::Paint
                } else {
                    conv(prop)
                };
                out.push((css.into(), value_css(prop, c, vals)?));
            }
        }
    }
    Ok(out
        .into_iter()
        .map(|(n, v)| format!("{n}: {v}"))
        .collect::<Vec<_>>()
        .join("; "))
}

fn conv_joint(k: &str) -> Conv {
    match k {
        "x" | "y" => Conv::Len,
        _ => Conv::Num,
    }
}

// ---- reading CSS values

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Tok {
    Ident(String),
    /// A number with its unit (`px`, `%`, `ms`, empty).
    Num(String, String),
    Str(String),
    Hash(String),
    Func(String, Vec<Vec<Tok>>),
    Slash,
}

pub(crate) fn describe(t: &Tok) -> String {
    match t {
        Tok::Ident(s) | Tok::Hash(s) => s.clone(),
        Tok::Num(n, u) => format!("{n}{u}"),
        Tok::Str(s) => quoted(s),
        Tok::Func(f, _) => format!("{f}(…)"),
        Tok::Slash => "/".into(),
    }
}

/// The value of one declaration as tokens.
pub(crate) fn tokens(s: &str) -> Result<Vec<Tok>, String> {
    let c: Vec<char> = s.chars().collect();
    let mut i = 0;
    let (toks, end) = tokens_in(&c, &mut i, false)?;
    if end {
        return Err("a ')' closes nothing".into());
    }
    Ok(toks.into_iter().flatten().collect())
}

/// Tokens up to the end (or, in a function, its `)`), split at commas.
fn tokens_in(c: &[char], i: &mut usize, func: bool) -> Result<(Vec<Vec<Tok>>, bool), String> {
    let mut args: Vec<Vec<Tok>> = vec![Vec::new()];
    while *i < c.len() {
        let ch = c[*i];
        if ch.is_whitespace() {
            *i += 1;
        } else if ch == ')' {
            *i += 1;
            let _ = func;
            return Ok((args, true));
        } else if ch == ',' {
            *i += 1;
            args.push(Vec::new());
        } else if ch == '/' {
            *i += 1;
            args.last_mut().unwrap().push(Tok::Slash);
        } else if ch == '*' {
            // A product in a relative colour's channel (`calc(l * 0.8)`).
            *i += 1;
            args.last_mut().unwrap().push(Tok::Ident("*".into()));
        } else if matches!(ch, '+' | '-') && c.get(*i + 1).is_none_or(|d| d.is_whitespace()) {
            // A math operator (CSS needs spaces around + and -).
            *i += 1;
            args.last_mut().unwrap().push(Tok::Ident(ch.to_string()));
        } else if matches!(ch, '"' | '\'') {
            let quote = ch;
            let mut value = String::new();
            *i += 1;
            while *i < c.len() && c[*i] != quote {
                if c[*i] == '\\' {
                    *i += 1;
                    let start = *i;
                    while *i < c.len() && *i - start < 6 && c[*i].is_ascii_hexdigit() {
                        *i += 1;
                    }
                    if *i > start {
                        let hex: String = c[start..*i].iter().collect();
                        let scalar = u32::from_str_radix(&hex, 16).unwrap();
                        value.push(
                            char::from_u32(scalar)
                                .filter(|c| *c != '\0')
                                .unwrap_or('\u{fffd}'),
                        );
                        if c.get(*i).is_some_and(|c| c.is_whitespace()) {
                            *i += 1;
                        }
                        continue;
                    }
                    if c.get(*i).is_some_and(|c| matches!(c, '\n' | '\r')) {
                        *i += 1;
                        continue;
                    }
                } else if matches!(c[*i], '\n' | '\r') {
                    return Err("a CSS string cannot contain an unescaped newline".into());
                }
                let Some(ch) = c.get(*i) else { break };
                value.push(*ch);
                *i += 1;
            }
            if *i >= c.len() {
                return Err("a string is not closed".into());
            }
            *i += 1;
            args.last_mut().unwrap().push(Tok::Str(value));
        } else if ch == '#' {
            let start = *i;
            *i += 1;
            while *i < c.len() && c[*i].is_ascii_alphanumeric() {
                *i += 1;
            }
            args.last_mut()
                .unwrap()
                .push(Tok::Hash(c[start..*i].iter().collect()));
        } else if ch.is_ascii_digit()
            || (ch == '-' || ch == '.')
                && c.get(*i + 1)
                    .is_some_and(|d| d.is_ascii_digit() || *d == '.')
        {
            let start = *i;
            *i += 1;
            while *i < c.len()
                && (c[*i].is_ascii_digit()
                    || c[*i] == '.'
                    || (matches!(c[*i], 'e' | 'E')
                        && c.get(*i + 1)
                            .is_some_and(|d| d.is_ascii_digit() || *d == '-')))
            {
                if matches!(c[*i], 'e' | 'E') {
                    *i += 1;
                }
                *i += 1;
            }
            let n: String = c[start..*i].iter().collect();
            let us = *i;
            while *i < c.len() && (c[*i].is_ascii_alphabetic() || c[*i] == '%') {
                *i += 1;
            }
            let unit: String = c[us..*i].iter().collect();
            // A physical length is px at 96px to the inch (CSS Values 4 §6.2).
            let (n, unit) = match super::calc::physical(&unit) {
                Some(k) => (
                    super::calc::number(n.parse::<f64>().unwrap_or(0.0) * k),
                    "px".to_string(),
                ),
                None => (n, unit),
            };
            args.last_mut().unwrap().push(Tok::Num(n, unit));
        } else if ch.is_ascii_alphabetic() || matches!(ch, '-' | '_' | '$') {
            let start = *i;
            while *i < c.len()
                && (c[*i].is_ascii_alphanumeric() || matches!(c[*i], '-' | '_' | '.' | '$'))
            {
                *i += 1;
            }
            let id: String = c[start..*i].iter().collect();
            if c.get(*i) == Some(&'(') {
                *i += 1;
                let (inner, closed) = tokens_in(c, i, true)?;
                if !closed {
                    return Err(format!("{id}( is not closed"));
                }
                args.last_mut().unwrap().push(Tok::Func(id, inner));
            } else {
                args.last_mut().unwrap().push(Tok::Ident(id));
            }
        } else {
            return Err(format!("'{ch}' is not part of a value"));
        }
    }
    Ok((args, false))
}

pub(crate) fn want_len(t: &Tok) -> Result<Value, String> {
    match t {
        Tok::Func(f, _) if f == "var" => {
            let name = kebab(&want_var(t, "")?);
            Err(format!("unresolved CSS variable --{name}"))
        }
        Tok::Num(n, u)
            if u == "px"
                || (u.is_empty()
                    && n.trim_start_matches('-')
                        .trim_matches(['0', '.'])
                        .is_empty()) =>
        {
            number(n)
        }
        Tok::Num(n, u) if u.is_empty() => Err(format!("'{n}' needs its unit: {n}px")),
        Tok::Func(f, a) if f == "bind" => Err(format!(
            "bind({}) binds a whole declaration",
            a.iter()
                .flatten()
                .map(describe)
                .collect::<Vec<_>>()
                .join(" ")
        )),
        t => Err(format!("'{}' is not a length (px)", describe(t))),
    }
}

/// A margin edge as its px, em, fraction of the parent's content width and
/// auto flag: `12px`, `0.67em`, `5%`, `0`, `auto` or a reduced
/// `calc(50% - 12px)`.
fn margin_vals(t: &Tok) -> Result<Vec<Value>, String> {
    let n = |v: f64| number(&super::calc::number(v));
    if matches!(t, Tok::Ident(w) if w == "auto") {
        return [0.0, 0.0, 0.0, 1.0].into_iter().map(n).collect();
    }
    if let Tok::Func(f, _) = t
        && f == "var"
    {
        let v = want_len(t)?;
        return Ok(vec![v, n(0.0)?, n(0.0)?, n(0.0)?]);
    }
    let [pct, px, em] = want_lin(t)?;
    [px, em, pct, 0.0].into_iter().map(n).collect()
}

/// A size's reduced math function as its ogt form, `calc <op> <pct> <px>
/// …`: `calc(100% - 12px)` → `calc sum 1 -12`, `min(100%, 600px)` → `calc
/// min 1 0 0 600`.
fn size_calc(name: &str, f: &str, args: &[Vec<Tok>]) -> Result<Vec<Value>, String> {
    // `round(<strategy>, …)` names its strategy first.
    let (op, args) = match (f, args.first().map(Vec::as_slice)) {
        ("round", Some([Tok::Ident(s)])) => (
            match s.as_str() {
                "nearest" => "round",
                "up" => "roundUp",
                "down" => "roundDown",
                "to-zero" => "roundToZero",
                s => return Err(format!("{name}: '{s}' is not a rounding strategy")),
            },
            &args[1..],
        ),
        ("calc", _) => ("sum", args),
        _ => (f, args),
    };
    let lengths: Vec<[f64; 3]> = if f == "calc" {
        vec![want_lin(&Tok::Func(f.into(), args.to_vec()))?]
    } else {
        args.iter()
            .map(|arg| match arg.as_slice() {
                [t] => want_lin(t),
                _ => Err(format!("{f}() takes one length an argument")),
            })
            .collect::<Result<_, _>>()?
    };
    let (count, how) = match f {
        "calc" | "abs" => (1..=1, "one length"),
        "clamp" => (3..=3, "three lengths"),
        "round" => (1..=2, "a length and its step"),
        "mod" | "rem" => (2..=2, "two lengths"),
        _ => (1..=3, "one to three lengths"),
    };
    if !count.contains(&lengths.len()) {
        return Err(format!("{name}: {f}() takes {how}"));
    }
    if lengths.iter().any(|l| l[2] != 0.0) {
        return Err(format!("{name} takes px and %, not em"));
    }
    let mut out = vec![Value::Word("calc".into()), Value::Word(op.into())];
    for [pct, px, _] in lengths {
        out.push(number(&super::calc::number(pct))?);
        out.push(number(&super::calc::number(px))?);
    }
    Ok(out)
}

/// A length as its fraction, px and em terms: `12px`, `50%`, `1.5em`, `0`,
/// or a `calc()` doc/calc.rs reduced to a sum (`calc(50% - 12px + 1em)`).
pub(crate) fn want_lin(t: &Tok) -> Result<[f64; 3], String> {
    fn term(t: &Tok, sign: f64, out: &mut [f64; 3]) -> Result<(), String> {
        let Tok::Num(n, u) = t else {
            return Err(format!("'{}' is not a length (px, %, em)", describe(t)));
        };
        let v: f64 = n.parse().map_err(|_| format!("'{n}' is not a number"))?;
        let k = match u.as_str() {
            "%" => 0,
            "px" => 1,
            "em" => 2,
            "" if v == 0.0 => 1,
            _ => return Err(format!("'{n}{u}' is not a length (px, %, em)")),
        };
        out[k] += sign * if k == 0 { v / 100.0 } else { v };
        Ok(())
    }
    let mut out = [0.0; 3];
    match t {
        Tok::Func(f, args) if f == "calc" => {
            let [toks] = args.as_slice() else {
                return Err("calc() takes one expression".into());
            };
            let mut sign = 1.0;
            for (k, t) in toks.iter().enumerate() {
                match t {
                    Tok::Ident(op) if k % 2 == 1 && (op == "+" || op == "-") => {
                        sign = if op == "-" { -1.0 } else { 1.0 };
                    }
                    t if k % 2 == 0 => term(t, sign, &mut out)?,
                    t => return Err(format!("'{}' is not part of a reduced calc()", describe(t))),
                }
            }
        }
        Tok::Func(f, _) if matches!(f.as_str(), "min" | "max" | "clamp") => {
            return Err(format!(
                "{f}() over lengths in more than one unit takes a width or a height; a margin takes calc()"
            ));
        }
        t => term(t, 1.0, &mut out)?,
    }
    Ok(out)
}

pub(crate) fn want_num(t: &Tok) -> Result<Value, String> {
    match t {
        Tok::Num(n, u) if u.is_empty() => number(n),
        t => Err(format!("'{}' is not a number", describe(t))),
    }
}

fn want_angle(t: &Tok) -> Result<Value, String> {
    match t {
        Tok::Num(n, u) if u == "rad" => number(n),
        Tok::Num(n, u)
            if u.is_empty()
                && n.trim_start_matches('-')
                    .trim_matches(['0', '.'])
                    .is_empty() =>
        {
            number(n)
        }
        Tok::Num(n, u) if u == "deg" || u == "turn" || u == "grad" => {
            let v: f64 = n.parse().map_err(|_| format!("'{n}' is not a number"))?;
            let r = match u.as_str() {
                "deg" => v.to_radians(),
                "grad" => (v * 0.9).to_radians(),
                _ => v * std::f64::consts::TAU,
            };
            Ok(Value::Number(
                Number::from_f32(r as f32).map_err(|e| e.reason().to_string())?,
            ))
        }
        t => Err(format!(
            "'{}' is not an angle (rad, deg or turn)",
            describe(t)
        )),
    }
}

pub(crate) fn want_ms(t: &Tok) -> Result<Value, String> {
    match t {
        Tok::Num(n, u) if u == "ms" => number(n),
        Tok::Num(n, u) if u == "s" => number(&shift(n, 3)),
        t => Err(format!("'{}' is not a duration (ms or s)", describe(t))),
    }
}

fn want_ident<'a>(t: &'a Tok, what: &str) -> Result<&'a str, String> {
    match t {
        Tok::Ident(s) => Ok(s),
        t => Err(format!("'{}' is not {what}", describe(t))),
    }
}

/// `var(--<ns>-<name>)`'s name (camel case).
fn want_var(t: &Tok, ns: &str) -> Result<String, String> {
    match t {
        Tok::Func(f, a) if f == "var" => match a.as_slice() {
            [v] => match v.as_slice() {
                [Tok::Ident(n)] => {
                    let n = n.strip_prefix("--").ok_or("var(--name)")?;
                    match ns {
                        "" => Ok(camel(n)),
                        _ => n
                            .strip_prefix(ns)
                            .and_then(|r| r.strip_prefix('-'))
                            .map(camel)
                            .ok_or_else(|| format!("var(--{n}) is not a --{ns}- token")),
                    }
                }
                _ => Err("var(--name)".into()),
            },
            _ => Err("var(--name) takes no fallback".into()),
        },
        t => Err(format!(
            "'{}' is not a token: var(--{ns}{}…)",
            describe(t),
            if ns.is_empty() { "" } else { "-" }
        )),
    }
}

/// Preserve CSS colour text until typed parsing; converting to hex here
/// quantizes fractional alpha before the painter receives it.
pub(crate) fn paint_vals(t: &[Tok]) -> Result<Vec<Value>, String> {
    match t {
        [Tok::Func(f, _)] if f == "var" => Err("unresolved CSS colour variable".into()),
        [t] => {
            crate::css_color::parse(t)?;
            Ok(vec![Value::String(tok_text(t))])
        }
        _ => Err(
            "a colour: a named colour, #hex, rgb(), hsl(), oklch(), color-mix(), var(--role) …"
                .into(),
        ),
    }
}

/// `#rrggbb`, or `#rrggbbaa` when not opaque.
pub(crate) fn hex(c: crate::Srgba) -> String {
    let [r, g, b] = c.rgb;
    let a = (c.alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
    if a == 255 {
        format!("#{r:02x}{g:02x}{b:02x}")
    } else {
        format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
    }
}

/// A token written back as CSS.
pub(crate) fn tok_text(t: &Tok) -> String {
    match t {
        Tok::Ident(s) | Tok::Hash(s) => s.clone(),
        Tok::Num(n, u) => format!("{n}{u}"),
        Tok::Str(s) => quoted(s),
        Tok::Slash => "/".into(),
        Tok::Func(f, args) => format!(
            "{f}({})",
            args.iter()
                .map(|a| a.iter().map(tok_text).collect::<Vec<_>>().join(" "))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn transition_vals(t: &[Tok]) -> Result<Vec<Value>, String> {
    let t = match t.first() {
        Some(Tok::Ident(a)) if a == "all" => &t[1..],
        _ => t,
    };
    match t {
        [Tok::Ident(n)] if n == "none" => Ok(vec![Value::Word("instant".into())]),
        [v @ Tok::Func(f, _)] if f == "var" => Ok(vec![
            Value::Word("token".into()),
            Value::Word(want_var(v, "motion")?),
        ]),
        [Tok::Func(f, a)] if f == "-og-spring" => {
            let mut out = vec![Value::Word("spring".into())];
            match a.as_slice() {
                [d] => out.push(want_ms(d.first().ok_or("-og-spring(<ms>ms, <bounce>)")?)?),
                [d, b] => {
                    out.push(want_ms(d.first().ok_or("-og-spring(<ms>ms, <bounce>)")?)?);
                    out.push(want_num(b.first().ok_or("-og-spring(<ms>ms, <bounce>)")?)?);
                }
                _ => return Err("-og-spring(<ms>ms, <bounce>)".into()),
            }
            Ok(out)
        }
        [d, rest @ ..] if matches!(d, Tok::Num(..)) => {
            let mut out = vec![Value::Word("tween".into()), want_ms(d)?];
            match rest {
                [] => {}
                [Tok::Ident(e)] => out.push(Value::Word(
                    match e.as_str() {
                        "linear" => "linear",
                        "ease-out" => "out",
                        "ease-in" => "exit",
                        "ease-in-out" => "inOut",
                        _ => {
                            return Err(format!(
                                "'{e}' is not an ease: linear, ease-out, ease-in, ease-in-out"
                            ));
                        }
                    }
                    .into(),
                )),
                _ => return Err("<ms>ms [ease]".into()),
            }
            Ok(out)
        }
        _ => Err(
            "a transition: none, <ms>ms [ease], -og-spring(<ms>ms, <bounce>) or var(--motion-<name>)"
                .into(),
        ),
    }
}

fn one(t: &[Tok], name: &str) -> Result<Tok, String> {
    match t {
        [v] => Ok(v.clone()),
        _ => Err(format!("{name} takes one value")),
    }
}

/// A grid line: `<start>`, `<start> / span <n>` or `span <n>` (0: auto).
fn grid_line(t: &[Tok]) -> Result<(Option<Value>, Option<Value>), String> {
    let span = |t: &[Tok]| match t {
        [Tok::Ident(s), n] if s == "span" => want_num(n).map(Some),
        _ => Err("span <n>".to_string()),
    };
    match t {
        [n @ Tok::Num(..)] => Ok((Some(want_num(n)?), None)),
        [n @ Tok::Num(..), Tok::Slash, rest @ ..] => Ok((Some(want_num(n)?), span(rest)?)),
        [Tok::Ident(s), _] if s == "span" => Ok((None, span(t)?)),
        _ => Err("a grid line: <n>, <n> / span <n> or span <n>".into()),
    }
}

/// A `display` value as its prop word (CSS Display 3 §2): one keyword or
/// the multi-keyword form `[block | inline | run-in] [flow | flow-root |
/// flex | grid | table] [list-item]`.
fn display_word(v: &str) -> Result<&'static str, String> {
    let one = match v {
        "block" | "run-in" => Some("block"),
        "inline" => Some("inline"),
        "inline-block" => Some("inlineBlock"),
        "flow-root" => Some("flowRoot"),
        "list-item" => Some("listItem"),
        "flex" => Some("flex"),
        "inline-flex" => Some("inlineFlex"),
        "grid" => Some("grid"),
        "inline-grid" => Some("inlineGrid"),
        "table" => Some("table"),
        "inline-table" => Some("inlineTable"),
        "contents" => Some("contents"),
        "table-row-group" => Some("tableRowGroup"),
        "table-header-group" => Some("tableHeaderGroup"),
        "table-footer-group" => Some("tableFooterGroup"),
        "table-row" => Some("tableRow"),
        "table-cell" => Some("tableCell"),
        "table-caption" => Some("tableCaption"),
        "table-column" => Some("tableColumn"),
        "table-column-group" => Some("tableColumnGroup"),
        _ => None,
    };
    if let Some(w) = one {
        return Ok(w);
    }
    let how = || {
        format!(
            "'{v}' is not a display: block, inline, inline-block, flow-root, flex, grid, table, contents, none, list-item, a table part, or [block | inline] [flow | flow-root | flex | grid | table] [list-item]"
        )
    };
    let (mut outer, mut inner, mut item) = (None, None, false);
    for w in v.split_whitespace() {
        match w {
            "block" | "inline" | "run-in" if outer.is_none() => {
                outer = Some(if w == "inline" { "inline" } else { "block" })
            }
            "flow" | "flow-root" | "flex" | "grid" | "table" if inner.is_none() => inner = Some(w),
            "list-item" if !item => item = true,
            _ => return Err(how()),
        }
    }
    let inline = outer == Some("inline");
    Ok(match (inline, inner.unwrap_or("flow"), item) {
        (false, "flow", true) | (false, "flow-root", true) => "listItem",
        (_, "flex" | "grid" | "table", true) => return Err(how()),
        (true, _, true) => return Err(how()),
        (false, "flow", false) => "block",
        (true, "flow", false) => "inline",
        (false, "flow-root", false) => "flowRoot",
        (true, "flow-root", false) => "inlineBlock",
        (false, "flex", false) => "flex",
        (true, "flex", false) => "inlineFlex",
        (false, "grid", false) => "grid",
        (true, "grid", false) => "inlineGrid",
        (false, "table", false) => "table",
        (true, "table", false) => "inlineTable",
        _ => return Err(how()),
    })
}

/// The entries one element declaration (or state value when `state`)
/// sets, in order. Sizes, grid lines and `grid-auto-flow` come back as
/// parts the block joins ([`Joined`]).
fn lower_decl(name: &str, value: &str, state: bool) -> Result<Vec<Part>, String> {
    let e = |key: &str, values: Vec<Value>| Part::Entry(key.to_string(), values);
    if !value.contains("bind(")
        && let Some(parts) = super::tracks::lower(name, value)
    {
        return Ok(parts?.into_iter().map(|(k, v)| e(k, v)).collect());
    }
    if let Some(parts) = super::position_css::lower(name, value) {
        return Ok(parts?.into_iter().map(|(k, v)| e(k, v)).collect());
    }
    if let Some(parts) = super::multicol_css::lower(name, value) {
        return Ok(parts?.into_iter().map(|(k, v)| e(k, v)).collect());
    }
    if let Some(parts) = super::view_transition::lower(name, value) {
        return Ok(parts?.into_iter().map(|(k, v)| e(k, v)).collect());
    }
    if let Some(parts) = super::contain_css::lower(name, value) {
        return Ok(parts?.into_iter().map(|(k, v)| e(k, v)).collect());
    }
    let t = tokens(value)?;
    if t.is_empty() {
        return Err(format!("{name} has no value"));
    }
    if name == "display" {
        let word = match value.trim() {
            "none" => {
                return Ok(vec![e("visible", vec![Value::Word("false".into())])]);
            }
            "initial" => {
                return Ok(vec![e("visible", vec![Value::Word("true".into())])]);
            }
            v => display_word(v)?,
        };
        return Ok(vec![
            e("visible", vec![Value::Word("true".into())]),
            e("display", vec![Value::Word(word.into())]),
        ]);
    }
    // A bound declaration: the whole value is the binding.
    if let [Tok::Func(f, a)] = t.as_slice()
        && f == "bind"
        && !matches!(name, "translate" | "scale")
    {
        let src = match a.as_slice() {
            [v] => match v.as_slice() {
                [Tok::Ident(p)] => Value::Word(p.clone()),
                [Tok::Str(s)] => Value::String(s.clone()),
                _ => return Err("bind(path) or bind(\"expression\")".into()),
            },
            _ => return Err("bind(path) or bind(\"expression\")".into()),
        };
        let prop = if state {
            return Err("a state's values are literals".into());
        } else {
            match name {
                "grid-column" | "grid-row" | "grid-auto-flow" => {
                    return Err(format!("{name} binds each part: write it as values"));
                }
                n => STYLE
                    .iter()
                    .find(|(_, c)| *c == n)
                    .map(|(p, _)| *p)
                    .ok_or_else(|| format!("'{n}' is not a property"))?,
            }
        };
        return Ok(vec![e(prop, vec![Value::Word("bind".into()), src])]);
    }
    // A part of a joint property: its value, or `bind(…)`.
    let part = |k: &str, t: &Tok| -> Result<Vec<Value>, String> {
        if let Tok::Func(f, a) = t
            && f == "bind"
        {
            return match a.as_slice() {
                [v] => match v.as_slice() {
                    [Tok::Ident(p)] => Ok(vec![Value::Word("bind".into()), Value::Word(p.clone())]),
                    [Tok::Str(s)] => Ok(vec![Value::Word("bind".into()), Value::String(s.clone())]),
                    _ => Err("bind(path) or bind(\"expression\")".into()),
                },
                _ => Err("bind(path) or bind(\"expression\")".into()),
            };
        }
        if k.starts_with("scale") {
            want_num(t).map(|v| vec![v])
        } else {
            want_len(t).map(|v| vec![v])
        }
    };
    match name {
        "translate" => {
            let (x, y) = match t.as_slice() {
                [Tok::Ident(w)] if name == "translate" && w == "none" => (
                    Tok::Num("0".into(), String::new()),
                    Tok::Num("0".into(), String::new()),
                ),
                [x] => (x.clone(), Tok::Num("0".into(), String::new())),
                [x, y] => (x.clone(), y.clone()),
                _ => return Err(format!("{name}: <x> <y>")),
            };
            // A percentage is of the element's own box, as CSS's: its
            // fraction moves it after layout beside the px (`xShift`), so
            // `translate: -50% -50%` centres it on its place.
            // The px pair first, as every reader of `translate` takes it;
            // the shifts after.
            let (mut px, mut shifts) = (Vec::new(), Vec::new());
            for (k, shift, t) in [("x", "xShift", &x), ("y", "yShift", &y)] {
                let fraction = match t {
                    Tok::Num(n, u) if u == "%" && state => {
                        return Err(format!("a state's translate takes px, not {n}%"));
                    }
                    Tok::Num(n, u) if u == "%" => {
                        px.push(e(k, vec![number("0")?]));
                        let f = n
                            .parse::<f64>()
                            .map_err(|_| format!("'{n}%' is not a number"))?;
                        (f / 100.0).to_string()
                    }
                    t => {
                        px.push(e(k, part(k, t)?));
                        "0".to_string()
                    }
                };
                if !state {
                    shifts.push(e(shift, vec![number(&fraction)?]));
                }
            }
            px.extend(shifts);
            Ok(px)
        }
        "scale" => match t.as_slice() {
            [s] => Ok(vec![e("scale", part("scale", s)?)]),
            [x, y] => Ok(vec![
                e("scaleX", part("scaleX", x)?),
                e("scaleY", part("scaleY", y)?),
            ]),
            _ => Err("scale: <both> or <x> <y>".into()),
        },
        "rotate" => Ok(vec![e("rotate", vec![want_angle(&one(&t, name)?)?])]),
        "opacity" => Ok(vec![e("opacity", vec![want_num(&one(&t, name)?)?])]),
        "z-index" => Ok(vec![e("z", vec![want_num(&one(&t, name)?)?])]),
        "border-radius" if state => Ok(vec![e("radius", vec![want_len(&one(&t, name)?)?])]),
        n if !state && !super::boxes::corner_props(n).is_empty() => {
            super::boxes::lower_corners(n, &t)
                .map(|parts| parts.into_iter().map(|(k, v)| e(k, v)).collect())
        }
        "background" | "background-image"
            if !state && super::paint_css::background(value).is_some() =>
        {
            let (_, colour) = super::paint_css::background(value).unwrap()?;
            let mut out = vec![e(
                "backgroundImage",
                vec![Value::String(value.trim().to_owned())],
            )];
            if let Some(c) = colour {
                out.push(e("background", paint_vals(&[c])?));
            } else if name == "background" && value.trim() == "none" {
                out.push(e(
                    "background",
                    paint_vals(&[Tok::Ident("transparent".into())])?,
                ));
            }
            // The shorthand resets the longhands to its own layers' (an
            // empty value), so an earlier `background-size` stops applying
            // and a later one still does.
            if name == "background" {
                for (_, prop) in super::paint_css::BACKGROUND_LONGHANDS {
                    out.push(e(prop, vec![Value::String(String::new())]));
                }
            }
            Ok(out)
        }
        "background" | "background-color" => Ok(vec![e("background", paint_vals(&t)?)]),
        "color" => Ok(vec![e(
            if state { "ink" } else { "color" },
            paint_vals(&t)?,
        )]),
        _ if state => Err(format!(
            "'{name}' is not a state's: {}",
            [
                "translate",
                "scale",
                "rotate",
                "opacity",
                "border-radius",
                "z-index",
                "background",
                "color"
            ]
            .join(", ")
        )),
        "background-image" if value.trim() == "none" => Ok(vec![e(
            "backgroundImage",
            vec![Value::String("none".into())],
        )]),
        n if let Some(parts) = super::scroll_css::lower(n, value) => Ok(parts?
            .into_iter()
            .map(|(k, v)| e(k, vec![Value::String(v)]))
            .collect()),
        n if let Some(prop) = super::interact_css::prop_of(n) => {
            super::interact_css::check(n, value)?;
            Ok(vec![e(prop, vec![Value::String(value.trim().to_owned())])])
        }
        n if let Some(prop) = super::image_css::prop_of(n) => {
            super::image_css::check(n, value)?;
            Ok(vec![e(prop, vec![Value::String(value.trim().to_owned())])])
        }
        n if let Some(prop) = super::contain_css::prop_of(n) => {
            super::contain_css::check(n, value)?;
            Ok(vec![e(prop, vec![Value::String(value.trim().to_owned())])])
        }
        "scroll-snap-type" | "scroll-snap-align" => {
            let prop = match name {
                "scroll-snap-type" => super::paint_css::snap_type(value).map(|_| "scrollSnapType"),
                _ => super::paint_css::snap_align(value).map(|_| "scrollSnapAlign"),
            }?;
            Ok(vec![e(prop, vec![Value::String(value.trim().to_owned())])])
        }
        // CSS UI 4 §5.2: clip, ellipsis or a string the cut ends in (one
        // value: the end edge's).
        "text-overflow" if !state => {
            let (on, cut) = match one(&t, name)? {
                Tok::Ident(w) if w == "clip" => (false, String::new()),
                Tok::Ident(w) if w == "ellipsis" => (true, String::new()),
                Tok::Str(s) => (true, s),
                t => {
                    return Err(format!(
                        "text-overflow is clip, ellipsis or a string, not '{}'",
                        describe(&t)
                    ));
                }
            };
            Ok(vec![
                e("ellipsis", vec![Value::Word(on.to_string())]),
                e("ellipsisText", vec![Value::String(cut)]),
            ])
        }
        n if !state && let Some(parts) = super::paint_css::lower(n, value) => Ok(parts?
            .into_iter()
            .map(|(prop, v)| e(prop, vec![Value::String(v)]))
            .collect()),
        "list-style-image" => {
            super::lists::image(value)?;
            Ok(vec![e(
                "markerImage",
                vec![Value::String(value.trim().into())],
            )])
        }
        "-og-sound" => {
            crate::sound_css::parse(value)?;
            Ok(vec![e(
                "sound",
                vec![Value::String(value.trim().to_owned())],
            )])
        }
        "backdrop-filter" | "filter" | "mix-blend-mode" | "isolation" | "text-shadow"
        | "clip-path" | "-og-material" => {
            use super::paint_css as p;
            let prop = match name {
                "backdrop-filter" => {
                    p::filter_fns(value, "backdrop-filter").map(|_| "backdropFilter")
                }
                "filter" => p::filter_fns(value, "filter").map(|_| "filter"),
                "mix-blend-mode" => p::blend(value).map(|_| "mixBlendMode"),
                "isolation" => p::isolation(value).map(|_| "isolation"),
                "text-shadow" => p::text_shadows(
                    value,
                    crate::paint::ColorRef::Literal(crate::theme::Srgba::TRANSPARENT),
                )
                .map(|_| "textShadow"),
                "clip-path" => p::clip(value).map(|_| "clipPath"),
                _ => p::glass_material(value).and_then(|glass| {
                    if glass.is_some() {
                        Ok("material")
                    } else {
                        p::material(value).map(|_| "material")
                    }
                }),
            }?;
            Ok(vec![e(prop, vec![Value::String(value.trim().to_owned())])])
        }
        "width" | "height" | "min-width" | "max-width" | "min-height" | "max-height"
        | "inline-size" | "block-size" | "min-inline-size" | "max-inline-size"
        | "min-block-size" | "max-block-size" => {
            let axis = usize::from(!(name.ends_with("width") || name.ends_with("inline-size")));
            let v = one(&t, name)?;
            let bound = name
                .strip_prefix("min-")
                .map(|_| true)
                .or(name.strip_prefix("max-").map(|_| false));
            // `auto` (and a bound's `none`) is the initial value: the
            // element's own size, unbounded (CSS Sizing 3 §3.1).
            if matches!(&v, Tok::Ident(k) if k == "auto" || (k == "none" && bound == Some(false))) {
                return Ok(Vec::new());
            }
            let mut parts = Vec::new();
            let vals = match &v {
                Tok::Ident(k) if k == "fit-content" => vec![Value::Word("fit".into())],
                Tok::Ident(k) if k == "stretch" => vec![Value::Word("grow".into())],
                Tok::Ident(k) if k == "min-content" => vec![Value::Word("minContent".into())],
                Tok::Ident(k) if k == "max-content" => vec![Value::Word("maxContent".into())],
                // fit-content(<length>): its content, at most the length and
                // at least its min-content (CSS Sizing 3 §3.2).
                Tok::Func(f, args) if f == "fit-content" && bound.is_none() => {
                    let [arg] = args.as_slice() else {
                        return Err(format!("{name}: fit-content(<length>)"));
                    };
                    let [len] = arg.as_slice() else {
                        return Err(format!("{name}: fit-content(<length>)"));
                    };
                    match len {
                        Tok::Num(n, u) if u == "%" => {
                            parts.push(Part::Size(
                                axis,
                                SizeBit::Range(
                                    false,
                                    vec![Value::Word("percent".into()), number(&shift(n, -2))?],
                                ),
                            ));
                            vec![Value::Word("fit".into())]
                        }
                        len => vec![Value::Word("fit".into()), number("0")?, want_len(len)?],
                    }
                }
                Tok::Num(n, u) if u == "%" => {
                    vec![Value::Word("percent".into()), number(&shift(n, -2))?]
                }
                Tok::Func(f, args)
                    if matches!(
                        f.as_str(),
                        "calc"
                            | "min"
                            | "max"
                            | "clamp"
                            | "round"
                            | "mod"
                            | "rem"
                            | "abs"
                            | "hypot"
                    ) =>
                {
                    size_calc(name, f, args)?
                }
                v => vec![want_len(v)?],
            };
            parts.push(Part::Size(
                axis,
                match bound {
                    Some(min) => SizeBit::Range(min, vals),
                    None => SizeBit::Kind(vals),
                },
            ));
            Ok(parts)
        }
        "aspect-ratio" => {
            // `auto`, `<ratio>` or both (CSS Sizing 4 §2.1); `auto` alone is
            // the initial value, and a ratio is width / height.
            let ratio: Vec<&Tok> = t
                .iter()
                .filter(|v| !matches!(v, Tok::Ident(a) if a == "auto"))
                .collect();
            let n = |v: &Tok| -> Result<f64, String> {
                match v {
                    Tok::Num(n, u) if u.is_empty() => {
                        n.parse().map_err(|_| format!("'{n}' is not a number"))
                    }
                    v => Err(format!(
                        "aspect-ratio: auto | <width> [/ <height>], not '{}'",
                        describe(v)
                    )),
                }
            };
            let r = match ratio.as_slice() {
                [] => return Ok(Vec::new()),
                [a] => n(a)?,
                [a, Tok::Slash, b] => {
                    let b = n(b)?;
                    if b == 0.0 { 0.0 } else { n(a)? / b }
                }
                _ => return Err("aspect-ratio: auto | <width> [/ <height>]".into()),
            };
            Ok(vec![e("aspect", vec![number(&super::calc::number(r))?])])
        }
        "padding-top"
        | "padding-right"
        | "padding-bottom"
        | "padding-left"
        | "padding-block"
        | "padding-inline"
        | "padding-block-start"
        | "padding-block-end"
        | "padding-inline-start"
        | "padding-inline-end" => {
            // CSS order: top, right, bottom, left (left to right).
            let sides: [Option<&Tok>; 4] = match (name, t.as_slice()) {
                ("padding-top" | "padding-block-start", [a]) => [Some(a), None, None, None],
                ("padding-right" | "padding-inline-end", [a]) => [None, Some(a), None, None],
                ("padding-bottom" | "padding-block-end", [a]) => [None, None, Some(a), None],
                ("padding-left" | "padding-inline-start", [a]) => [None, None, None, Some(a)],
                ("padding-block", [a]) => [Some(a), None, Some(a), None],
                ("padding-block", [a, b]) => [Some(a), None, Some(b), None],
                ("padding-inline", [a]) => [None, Some(a), None, Some(a)],
                ("padding-inline", [start, end]) => [None, Some(end), None, Some(start)],
                _ => {
                    return Err(format!(
                        "{name} takes one length (two for -block and -inline)"
                    ));
                }
            };
            let mut out = Vec::new();
            for (side, prop) in sides.into_iter().zip([
                "paddingTop",
                "paddingRight",
                "paddingBottom",
                "paddingLeft",
            ]) {
                if let Some(side) = side {
                    out.push(e(prop, vec![want_len(side)?]));
                }
            }
            Ok(out)
        }
        "padding" => {
            let n: Vec<Value> = t.iter().map(want_len).collect::<Result<_, _>>()?;
            let vals = match n.as_slice() {
                [a] => vec![a.clone()],
                [v, h] => vec![h.clone(), v.clone()],
                [top, h, b] => vec![h.clone(), top.clone(), h.clone(), b.clone()],
                [top, r, b, l] => vec![l.clone(), top.clone(), r.clone(), b.clone()],
                _ => return Err("padding takes 1 to 4 lengths".into()),
            };
            Ok(vec![e("padding", vals)])
        }
        "margin"
        | "margin-block"
        | "margin-inline"
        | "margin-inline-start"
        | "margin-inline-end"
        | "margin-block-start"
        | "margin-block-end" => {
            // CSS order: top, right, bottom, left (left to right).
            let sides: [Option<&Tok>; 4] = match (name, t.as_slice()) {
                ("margin-block-start", [a]) => [Some(a), None, None, None],
                ("margin-inline-end", [a]) => [None, Some(a), None, None],
                ("margin-block-end", [a]) => [None, None, Some(a), None],
                ("margin-inline-start", [a]) => [None, None, None, Some(a)],
                ("margin", [a]) => [Some(a); 4],
                ("margin", [v, h]) => [Some(v), Some(h), Some(v), Some(h)],
                ("margin", [top, h, b]) => [Some(top), Some(h), Some(b), Some(h)],
                ("margin", [top, r, b, l]) => [Some(top), Some(r), Some(b), Some(l)],
                ("margin-block", [a]) => [Some(a), None, Some(a), None],
                ("margin-block", [a, b]) => [Some(a), None, Some(b), None],
                ("margin-inline", [a]) => [None, Some(a), None, Some(a)],
                ("margin-inline", [start, end]) => [None, Some(end), None, Some(start)],
                _ => {
                    return Err(format!(
                        "{name} takes 1 to {} values",
                        if name == "margin" { 4 } else { 2 }
                    ));
                }
            };
            let mut out = Vec::new();
            for (side, prop) in
                sides
                    .into_iter()
                    .zip(["marginTop", "marginRight", "marginBottom", "marginLeft"])
            {
                if let Some(side) = side {
                    out.push(e(prop, margin_vals(side)?));
                }
            }
            Ok(out)
        }
        n if super::boxes::longhands(n).len() > 1
            || super::boxes::logical(n).is_some()
            || n == "box-shadow"
            || n.ends_with("-width") && (n.starts_with("border-") || n == "outline-width")
            || n.ends_with("-color") && (n.starts_with("border-") || n == "outline-color") =>
        {
            super::boxes::lower(n, value, &t)
                .map(|parts| parts.into_iter().map(|(k, v)| e(k, v)).collect())
        }
        "grid-column" | "grid-row" => {
            let (start, span) = grid_line(&t)?;
            let axis = if name == "grid-column" { 0 } else { 1 };
            Ok(vec![Part::Grid(axis, start, span)])
        }
        "-og-text-path" => {
            crate::text_path::parse_shape(value)?;
            Ok(vec![e(
                "textPath",
                vec![Value::String(value.trim().to_owned())],
            )])
        }
        // Engine effects with no CSS standard (docs/ui.md): read by the
        // document build, which names a bad value.
        "-og-meta" => Ok(vec![e(
            "meta",
            vec![Value::String(value.trim().to_owned())],
        )]),
        "-og-text-path-offset" => {
            crate::text_path::parse_offset(value)?;
            Ok(vec![e(
                "textPathOffset",
                vec![Value::String(value.trim().to_owned())],
            )])
        }
        "object-view-box" => {
            // CSS's inset() of the texture, in percentages: the box's
            // u0 v0 u1 v1 (px need the texture's size, which the box lacks).
            let [Tok::Func(f, args)] = t.as_slice() else {
                return Err("object-view-box: none | inset(<t>% [<r>% [<b>% [<l>%]]])".into());
            };
            if f != "inset" {
                return Err(format!("object-view-box takes inset(…), not {f}(…)"));
            }
            let pct: Vec<f64> = args
                .iter()
                .flatten()
                .map(|v| match v {
                    Tok::Num(n, u) if u == "%" => n
                        .parse::<f64>()
                        .map(|n| n / 100.0)
                        .map_err(|_| format!("'{n}%' is not a number")),
                    Tok::Num(n, _) if n.parse::<f64>() == Ok(0.0) => Ok(0.0),
                    v => Err(format!(
                        "object-view-box: inset() takes percentages of the image, not '{}'",
                        describe(v)
                    )),
                })
                .collect::<Result<_, _>>()?;
            let [top, right, bottom, left] = match pct[..] {
                [a] => [a; 4],
                [v, h] => [v, h, v, h],
                [top, h, b] => [top, h, b, h],
                [top, r, b, l] => [top, r, b, l],
                _ => return Err("inset() takes 1 to 4 percentages".into()),
            };
            if left + right >= 1.0 || top + bottom >= 1.0 {
                return Err("object-view-box: inset() leaves none of the image".into());
            }
            Ok(vec![e(
                "uv",
                [left, top, 1.0 - right, 1.0 - bottom]
                    .into_iter()
                    .map(|v| {
                        let s = format!("{v:.6}");
                        number(s.trim_end_matches('0').trim_end_matches('.'))
                    })
                    .collect::<Result<_, _>>()?,
            )])
        }
        "flex-direction" => {
            let w = camel(want_ident(&one(&t, name)?, "a direction")?);
            let both = vec![Value::Word(w)];
            Ok(vec![
                e("direction", both.clone()),
                e("labelDirection", both),
            ])
        }
        "flex-wrap" => {
            let (wrap, reverse) = match want_ident(&one(&t, name)?, "nowrap, wrap or wrap-reverse")?
            {
                "nowrap" => (false, false),
                "wrap" => (true, false),
                "wrap-reverse" => (true, true),
                w => {
                    return Err(format!(
                        "'{w}' is not a flex-wrap: nowrap, wrap, wrap-reverse"
                    ));
                }
            };
            Ok(vec![
                e("wrap", vec![bool_word(wrap)]),
                e("wrapReverse", vec![bool_word(reverse)]),
            ])
        }
        "flex-flow" => {
            // <flex-direction> || <flex-wrap> (CSS Flexbox §5.3).
            let mut out = Vec::new();
            for v in &t {
                let w = want_ident(v, "a direction or a wrap")?;
                match w {
                    "row" | "row-reverse" | "column" | "column-reverse" => {
                        out.extend(lower_decl("flex-direction", w, false)?)
                    }
                    "nowrap" | "wrap" | "wrap-reverse" => {
                        out.extend(lower_decl("flex-wrap", w, false)?)
                    }
                    w => return Err(format!("'{w}' is not a flex-flow: a direction and a wrap")),
                }
            }
            Ok(out)
        }
        "flex" => {
            // none | auto | <grow> [<shrink>] || <basis> (CSS Flexbox §7.1);
            // a number alone has a 0 basis, and `auto` a basis of auto.
            let num = |v: &Tok| matches!(v, Tok::Num(_, u) if u.is_empty());
            let (grow, shrink, basis): (Value, Value, Option<&Tok>) = match t.as_slice() {
                [Tok::Ident(w)] if w == "none" => (number("0")?, number("0")?, None),
                [Tok::Ident(w)] if w == "auto" => (number("1")?, number("1")?, None),
                [Tok::Ident(w)] if w == "initial" => (number("0")?, number("1")?, None),
                [g] if num(g) => (
                    want_num(g)?,
                    number("1")?,
                    Some(&Tok::Num("0".into(), "px".into())),
                ),
                [g, s] if num(g) && num(s) => (
                    want_num(g)?,
                    want_num(s)?,
                    Some(&Tok::Num("0".into(), "px".into())),
                ),
                [g, b] if num(g) => (want_num(g)?, number("1")?, Some(b)),
                [b, g] if num(g) => (want_num(g)?, number("1")?, Some(b)),
                [g, s, b] if num(g) && num(s) => (want_num(g)?, want_num(s)?, Some(b)),
                [b, g, s] if num(g) && num(s) => (want_num(g)?, want_num(s)?, Some(b)),
                [b] => (number("1")?, number("1")?, Some(b)),
                _ => return Err("flex: none | auto | <grow> [<shrink>] [<basis>]".into()),
            };
            let mut out = vec![e("flexGrow", vec![grow]), e("flexShrink", vec![shrink])];
            if let Some(b) = basis {
                out.extend(lower_decl("flex-basis", &css_text(b), false)?);
            }
            Ok(out)
        }
        "flex-basis" => match one(&t, name)? {
            Tok::Ident(w) if w == "auto" => Ok(Vec::new()),
            Tok::Ident(w) if w == "content" => {
                Ok(vec![e("flexBasis", vec![Value::Word("maxContent".into())])])
            }
            _ => {
                let parts = lower_decl("width", value, false)?;
                Ok(parts
                    .into_iter()
                    .map(|p| match p {
                        Part::Size(_, SizeBit::Kind(v)) => e("flexBasis", v),
                        p => p,
                    })
                    .collect())
            }
        },
        "order" => match one(&t, name)? {
            Tok::Num(n, u) if u.is_empty() && !n.contains('.') => {
                Ok(vec![e("order", vec![number(&n)?])])
            }
            v => Err(format!("order takes an integer, not '{}'", describe(&v))),
        },
        "gap" => match t.as_slice() {
            [g] => Ok(vec![e("gap", vec![want_len(g)?])]),
            [r, c] => Ok(vec![
                e("rowGap", vec![want_len(r)?]),
                e("columnGap", vec![want_len(c)?]),
            ]),
            _ => Err("gap: <row-gap> [<column-gap>]".into()),
        },
        "row-gap" | "column-gap" => match one(&t, name)? {
            Tok::Ident(w) if w == "normal" => Ok(Vec::new()),
            v => Ok(vec![e(
                if name == "row-gap" {
                    "rowGap"
                } else {
                    "columnGap"
                },
                vec![want_len(&v)?],
            )]),
        },
        "justify-content" | "align-content" | "align-items" | "align-self" | "justify-items"
        | "justify-self" => alignment(name, &t),
        "place-content" | "place-items" | "place-self" => {
            // <align> [<justify>]: one value sets both (CSS Box Alignment §5.3, §6.3, §6.4).
            let (align, justify) = match name {
                "place-content" => ("align-content", "justify-content"),
                "place-items" => ("align-items", "justify-items"),
                _ => ("align-self", "justify-self"),
            };
            let split = t
                .iter()
                .enumerate()
                .skip(1)
                .find(|(k, v)| {
                    !matches!(t[k - 1], Tok::Ident(ref w) if w == "safe" || w == "unsafe" || w == "first" || w == "last")
                        && matches!(v, Tok::Ident(_))
                })
                .map(|(k, _)| k)
                .unwrap_or(t.len());
            let (a, j) = t.split_at(split);
            let j = if j.is_empty() { a } else { j };
            let mut out = alignment(align, a)?;
            out.extend(alignment(justify, j)?);
            Ok(out)
        }
        "grid-auto-flow" => {
            let mut out = Vec::new();
            for v in &t {
                match want_ident(v, "row, column or dense")? {
                    "dense" => out.push(e("dense", vec![Value::Word("true".into())])),
                    f @ ("row" | "column") => out.push(e("flow", vec![Value::Word(f.into())])),
                    f => return Err(format!("'{f}' is not a grid flow: row, column, dense")),
                }
            }
            Ok(out)
        }
        _ => {
            let Some(prop) = STYLE.iter().find(|(_, c)| *c == name).map(|(p, _)| *p) else {
                return Err(match unsupported(name) {
                    Some(how) => format!("'{name}' is not supported: {how}"),
                    None => format!("'{name}' is not a property{}", hint_for(name)),
                });
            };
            let vals = match conv(prop) {
                Conv::Len => vec![want_len(&one(&t, name)?)?],
                Conv::Num => vec![want_num(&one(&t, name)?)?],
                Conv::Angle => vec![want_angle(&one(&t, name)?)?],
                Conv::Paint => paint_vals(&t)?,
                Conv::Bool(yes, no) => {
                    match want_ident(&one(&t, name)?, "a word")?.to_string().as_str() {
                        w if w == yes => vec![Value::Word("true".into())],
                        w if w == no => vec![Value::Word("false".into())],
                        w => return Err(format!("{name} is {yes} or {no}, not '{w}'")),
                    }
                }
                Conv::Kebab => vec![Value::Word(camel(want_ident(&one(&t, name)?, "a word")?))],
                Conv::Kw(table) => {
                    let v = one(&t, name)?;
                    let w = want_ident(&v, "a word")?;
                    let el = table
                        .iter()
                        .find(|(_, c)| *c == w)
                        .map(|(e, _)| *e)
                        .ok_or_else(|| {
                            format!(
                                "'{w}' is not a {name}: {}",
                                table.iter().map(|(_, c)| *c).collect::<Vec<_>>().join(", ")
                            )
                        })?;
                    vec![Value::Word(el.into())]
                }
                Conv::Var(ns) => vec![Value::Word(want_var(&one(&t, name)?, ns)?)],
                Conv::Ident => match one(&t, name)? {
                    Tok::Str(s) => vec![Value::String(s)],
                    Tok::Ident(s) => vec![Value::Word(s)],
                    v => return Err(format!("{name} takes a name, not '{}'", describe(&v))),
                },
                Conv::Strs => t
                    .iter()
                    .map(|v| match v {
                        Tok::Str(s) => Ok(Value::String(s.clone())),
                        v => Err(format!(
                            "{name} takes quoted strings, not '{}'",
                            describe(v)
                        )),
                    })
                    .collect::<Result<_, _>>()?,
                Conv::Tracks => {
                    super::tracks::parse(value)?;
                    vec![Value::String(value.trim().to_string())]
                }
                Conv::Transition => transition_vals(&t)?,
                Conv::Presence => {
                    let kind = want_ident(
                        t.first().ok_or("a presence")?,
                        "a presence: fade, slide-up …",
                    )?;
                    let mut out = vec![Value::Word(camel(kind))];
                    let mut rest = &t[1..];
                    if let Some(d) = rest.first()
                        && (matches!(d, Tok::Num(..))
                            || matches!(d, Tok::Func(f, _) if f == "var" && want_len(d).is_ok()))
                    {
                        out.push(want_len(d)?);
                        rest = &rest[1..];
                    }
                    if !rest.is_empty() {
                        out.extend(transition_vals(rest)?);
                    }
                    out
                }
                Conv::Origin => match t.as_slice() {
                    [Tok::Num(a, u), Tok::Num(b, v)] if u == "%" && v == "%" => {
                        vec![number(&shift(a, -2))?, number(&shift(b, -2))?]
                    }
                    _ => return Err("transform-origin: <x>% <y>%".into()),
                },
                Conv::Margin => margin_vals(&one(&t, name)?)?,
                Conv::Size | Conv::Insets | Conv::Joint | Conv::Raw => {
                    unreachable!("handled above")
                }
            };
            Ok(vec![e(prop, vals)])
        }
    }
}

fn bool_word(b: bool) -> Value {
    Value::Word(if b { "true" } else { "false" }.into())
}

/// A token as CSS text (a lowered shorthand hands its part on).
fn css_text(t: &Tok) -> String {
    match t {
        Tok::Func(f, args) => format!(
            "{f}({})",
            args.iter()
                .map(|a| a.iter().map(css_text).collect::<Vec<_>>().join(" "))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        t => describe(t),
    }
}

/// A CSS Box Alignment value as its prop's word (and its `safe` flag):
/// `[safe | unsafe] <position>`, a distribution, `normal`, `stretch` or a
/// baseline. `auto` on an item is the initial value (its parent's).
fn alignment(name: &str, t: &[Tok]) -> Result<Vec<Part>, String> {
    let e = |key: &str, values: Vec<Value>| Part::Entry(key.to_string(), values);
    let words: Vec<&str> = t
        .iter()
        .map(|v| want_ident(v, "an alignment"))
        .collect::<Result<_, _>>()?;
    let (safe, words) = match words.as_slice() {
        ["safe", rest @ ..] => (true, rest),
        ["unsafe", rest @ ..] => (false, rest),
        w => (false, w),
    };
    let content = name.ends_with("-content");
    let justify = name.starts_with("justify");
    let word = match (words, content) {
        (["auto"], _) if name.ends_with("-self") => return Ok(Vec::new()),
        (["normal"], true) if justify => "normal",
        (["normal"], true) => "stretch",
        (["normal" | "legacy"], false) => "stretch",
        (["stretch"], _) => "stretch",
        (["flex-start"], true) => "flexStart",
        (["flex-end"], true) => "flexEnd",
        (["start" | "left"], _) => "start",
        (["end" | "right"], _) => "end",
        (["flex-start" | "self-start"], false) => "start",
        (["flex-end" | "self-end"], false) => "end",
        (["center"], _) => "center",
        (["space-between"], true) => "spaceBetween",
        (["space-around"], true) => "spaceAround",
        (["space-evenly"], true) => "spaceEvenly",
        (["baseline"] | ["first", "baseline"], false) => "baseline",
        (["last", "baseline"], false) => "lastBaseline",
        _ => {
            return Err(format!("'{}' is not a {name} value", words.join(" ")));
        }
    };
    let prop = match name {
        "justify-content" => "align",
        "align-items" => "cross",
        "align-content" => "alignContent",
        "justify-items" => "justifyItems",
        "align-self" => "alignSelf",
        _ => "justifySelf",
    };
    // align-content takes no flex-relative words: its lines' start is
    // the flex start.
    let word = match (prop, word) {
        ("alignContent", "flexStart") => "start",
        ("alignContent", "flexEnd") => "end",
        (_, w) => w,
    };
    let mut out = vec![e(prop, vec![Value::Word(word.into())])];
    match name {
        "justify-content" => out.push(e("safeJustify", vec![bool_word(safe)])),
        "align-items" | "align-self" => out.push(e("safeAlign", vec![bool_word(safe)])),
        _ => {}
    }
    Ok(out)
}

/// What a property name was meant to be, when it is a near miss.
fn hint_for(name: &str) -> String {
    let c = camel(name);
    if let Some(css) = css_name(&c).or(css_name(name)) {
        return format!(" (write `{css}`)");
    }
    if crate::doc::schema::prop(&c).is_some() || crate::doc::schema::prop(name).is_some() {
        return format!(" ({c} is an attribute: `{c} <value>`, without a colon)");
    }
    let near: Vec<&str> = STYLE
        .iter()
        .map(|(_, c)| *c)
        .filter(|c| c.contains(name) || name.contains(c.trim_start_matches("-og-")))
        .collect();
    if near.is_empty() {
        " (the properties: docs/ui.md §3.2)".into()
    } else {
        format!(" (did you mean {}?)", near.join(", "))
    }
}

enum SizeBit {
    Kind(Vec<Value>),
    /// `min-` (true) or `max-`, a size's values.
    Range(bool, Vec<Value>),
}

enum Part {
    Entry(String, Vec<Value>),
    Size(usize, SizeBit),
    /// An axis, its start, its span.
    Grid(usize, Option<Value>, Option<Value>),
}

/// A size being joined: its entry's index in the block and its kind (once
/// one is declared), its `min-` and its `max-`, and the line of the first.
type SizeParts = (
    Option<(usize, Vec<Value>)>,
    Option<Vec<Value>>,
    Option<Vec<Value>>,
    usize,
);

/// A `translate` written in px also writes its shifts as 0, so it replaces
/// an earlier percentage in a document's cascade; readers that hold no
/// shift (a dressing, a keyframe's channels) skip these.
pub(crate) fn zero_shift(entry: &Entry) -> bool {
    matches!(entry.key.as_str(), "xShift" | "yShift")
        && entry
            .values
            .first()
            .and_then(og_core::ogt::Value::as_number)
            .is_some_and(|n| n.as_f32() == 0.0)
}

pub(crate) fn declarations(text: &str, line: usize) -> Result<Vec<Entry>, String> {
    declarations_at([(line, text)])
}

pub(crate) fn declarations_at<'a>(
    lines: impl IntoIterator<Item = (usize, &'a str)>,
) -> Result<Vec<Entry>, String> {
    let mut frame = Frame::new(Entry::new("style"));
    for (line, text) in lines {
        for (name, value) in split_decls(text)? {
            for part in lower_decl(&name, &value, false)? {
                frame.part(part, line)?;
            }
        }
    }
    frame.finish()?;
    Ok(frame.entry.block.map_or_else(Vec::new, |b| b.entries))
}

struct Frame {
    entry: Entry,
    /// Sizes and grid lines, joined at the block's end.
    size: [Option<SizeParts>; 2],
    grid: [Option<usize>; 2],
    grid_parts: [[Option<Value>; 2]; 2],
}

impl Frame {
    fn new(entry: Entry) -> Frame {
        Frame {
            entry,
            size: [None, None],
            grid: [None, None],
            grid_parts: [[None, None], [None, None]],
        }
    }

    fn block(&mut self) -> &mut Block {
        self.entry.block.get_or_insert_with(Block::new)
    }

    fn push(&mut self, e: Entry) {
        self.block().push(e);
    }

    fn len(&self) -> usize {
        self.entry.block.as_ref().map_or(0, |b| b.entries.len())
    }

    fn part(&mut self, p: Part, line: usize) -> Result<(), String> {
        match p {
            Part::Entry(k, values) => {
                const SIDES: [&str; 4] =
                    ["paddingTop", "paddingRight", "paddingBottom", "paddingLeft"];
                // The shorthand resets the sides a longhand set before it.
                if k == "gap"
                    && self.entry.block.as_ref().is_some_and(|b| {
                        b.entries
                            .iter()
                            .any(|e| e.key == "rowGap" || e.key == "columnGap")
                    })
                {
                    let v = values.clone();
                    for side in ["rowGap", "columnGap"] {
                        let mut e = Entry::new(side);
                        e.values = v.clone();
                        e.line = line;
                        self.push(e);
                    }
                }
                let reset =
                    k == "padding"
                        && self.entry.block.as_ref().is_some_and(|b| {
                            b.entries.iter().any(|e| SIDES.contains(&e.key.as_str()))
                        });
                let sides = reset.then(|| match values.as_slice() {
                    [a] => [a.clone(), a.clone(), a.clone(), a.clone()],
                    [x, y] => [y.clone(), x.clone(), y.clone(), x.clone()],
                    [l, t, r, b] => [t.clone(), r.clone(), b.clone(), l.clone()],
                    _ => std::array::from_fn(|_| Value::Number(Number::parse("0").unwrap())),
                });
                let mut e = Entry::new(k);
                e.values = values;
                e.line = line;
                self.push(e);
                for (side, v) in SIDES.into_iter().zip(sides.into_iter().flatten()) {
                    let mut e = Entry::new(side);
                    e.values = vec![v];
                    e.line = line;
                    self.push(e);
                }
            }
            Part::Size(axis, bit) => {
                let at = self.len();
                let s = self.size[axis].get_or_insert((None, None, None, line));
                match bit {
                    SizeBit::Kind(k) => match &mut s.0 {
                        Some((_, kind)) => *kind = k,
                        None => {
                            s.0 = Some((at, k));
                            let mut e = Entry::new(if axis == 0 { "width" } else { "height" });
                            e.line = line;
                            self.push(e);
                        }
                    },
                    SizeBit::Range(true, v) => s.1 = Some(v),
                    SizeBit::Range(false, v) => s.2 = Some(v),
                }
            }
            Part::Grid(axis, start, span) => {
                for (i, v) in [start, span].into_iter().enumerate() {
                    let Some(v) = v else { continue };
                    if self.grid[i].is_none() {
                        let mut e = Entry::new(if i == 0 { "cell" } else { "span" });
                        e.line = line;
                        self.grid[i] = Some(self.len());
                        self.push(e);
                    }
                    self.grid_parts[i][axis] = Some(v);
                }
            }
        }
        Ok(())
    }

    /// Joins sizes and grid lines.
    fn finish(&mut self) -> Result<(), String> {
        for axis in 0..2 {
            let Some((kind, mut min, mut max, line)) = self.size[axis].take() else {
                continue;
            };
            // A px bound of a fit-content or stretch size is the size's own
            // range (as `width fit <min> <max>` writes it); any other bound
            // is its own prop, applied to whatever size the element takes.
            let px = |b: &Option<Vec<Value>>| matches!(b.as_deref(), Some([Value::Number(_)]));
            if let Some((at, mut v)) = kind {
                let ranged = matches!(v.as_slice(), [Value::Word(w)] if w == "fit" || w == "grow");
                if ranged
                    && (px(&min) || px(&max))
                    && (min.is_none() || px(&min))
                    && (max.is_none() || px(&max))
                {
                    let zero = Value::Number(Number::parse("0").unwrap());
                    v.push(min.take().map_or(zero, |m| m[0].clone()));
                    if let Some(m) = max.take() {
                        v.push(m[0].clone());
                    }
                }
                self.entry.block.as_mut().unwrap().entries[at].values = v;
            }
            let names = if axis == 0 {
                ["minWidth", "maxWidth"]
            } else {
                ["minHeight", "maxHeight"]
            };
            for (name, b) in names.into_iter().zip([min, max]) {
                if let Some(values) = b {
                    let mut e = Entry::new(name);
                    e.values = values;
                    e.line = line;
                    self.push(e);
                }
            }
        }
        for i in 0..2 {
            let Some(at) = self.grid[i].take() else {
                continue;
            };
            let zero = || Value::Number(Number::parse("0").unwrap());
            let [a, b] = std::mem::take(&mut self.grid_parts[i]);
            self.entry.block.as_mut().unwrap().entries[at].values =
                vec![a.unwrap_or_else(zero), b.unwrap_or_else(zero)];
        }
        Ok(())
    }
}

/// A line's declarations, split at `;` outside strings and parentheses, and
/// cut at a `#` comment (a `#` that starts no colour).
pub(crate) fn split_decls(t: &str) -> Result<Vec<(String, String)>, String> {
    let c: Vec<char> = t.chars().collect();
    let mut parts = Vec::new();
    let (mut cur, mut depth, mut i, mut in_str) = (String::new(), 0, 0, false);
    while i < c.len() {
        let ch = c[i];
        if in_str {
            cur.push(ch);
            if ch == '\\' && i + 1 < c.len() {
                cur.push(c[i + 1]);
                i += 1;
            } else if ch == '"' {
                in_str = false;
            }
        } else if ch == '"' {
            in_str = true;
            cur.push(ch);
        } else if ch == '#' && depth == 0 && !c.get(i + 1).is_some_and(|d| d.is_ascii_hexdigit()) {
            // A comment; inside parentheses a `#` is a reference (`url(#id)`).
            break;
        } else if ch == '(' {
            depth += 1;
            cur.push(ch);
        } else if ch == ')' {
            depth -= 1;
            cur.push(ch);
        } else if ch == ';' && depth == 0 {
            parts.push(std::mem::take(&mut cur));
        } else {
            cur.push(ch);
        }
        i += 1;
    }
    parts.push(cur);
    let mut out = Vec::new();
    for p in parts {
        let p = p.trim();
        if p.is_empty() {
            continue;
        }
        let (name, value) = p
            .split_once(':')
            .ok_or_else(|| format!("'{p}' is not a declaration (name: value)"))?;
        let value = value.trim();
        if value.ends_with("!important") {
            return Err("there is no !important: an element's later declaration wins".into());
        }
        out.push((name.trim().to_string(), value.to_string()));
    }
    Ok(out)
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
