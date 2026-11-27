//! File contracts consumed by the pack reader and file reference.
use og_core::ogt::line::Line;

pub const LINES: &[Line] = &[
    Line::new("about", "any JSON", "", "Ignored author note.", ""),
    Line::new(
        "scheme",
        "light | dark",
        "inherited",
        "Default colour scheme.",
        "",
    ),
    Line::new(
        "seeds",
        "{name: CSS colour or {light,dark}}",
        "inherited",
        "Palette seeds; accepted names are listed below.",
        "",
    ),
    Line::new(
        "theme",
        "{section: values}",
        "inherited",
        "Tailwind theme overrides. Leaves may be exact {light,dark} pairs.",
        "",
    ),
    Line::new(
        "contrast",
        "{section: values}",
        "inherited",
        "Theme overrides for prefers-contrast:more.",
        "",
    ),
    Line::new(
        "variants",
        "{Component.slot[.variant]: class string}",
        "{}",
        "Append classes to named component slots; empty strings declare slots.",
        "",
    ),
    Line::new(
        "fonts",
        "[path, ...]",
        "[]",
        "Font files relative to this pack, or shared asset paths. Font metadata comes from the files.",
        "",
    ),
    Line::new("icons", "{name: path}", "{}", "Named image paths.", ""),
    Line::new(
        "iconSet",
        "path",
        "none",
        "Cooked icon set (file.icons), below individually named icons.",
        "",
    ),
    Line::new(
        "images",
        "{kebab-name: nineSlice or {light,dark}}",
        "{}",
        "Named nine-slice art.",
        "",
    ),
    Line::new(
        "sounds",
        "{event: sound, Component.slot[.variant]: {event: sound}}",
        "{}",
        "Pack-wide or slot-local interaction sounds.",
        "",
    ),
    Line::new(
        "haptics",
        "{event: preset, Component.slot[.variant]: {event: preset}}",
        "{}",
        "Pack-wide or slot-local haptic names.",
        "",
    ),
];
pub const NINE: &[Line] = &[
    Line::new("src", "path", "required", "Source image.", ""),
    Line::new(
        "slice",
        "number >= 0 | [top,right,bottom,left]",
        "required",
        "Insets into the source image, in pixels.",
        "",
    ),
    Line::new(
        "width",
        "number >= 0 | [top,right,bottom,left]",
        "slice",
        "Drawn border widths.",
        "",
    ),
    Line::new("fill", "boolean", "true", "Paint the centre.", ""),
    Line::new(
        "repeat",
        "stretch | tile",
        "stretch",
        "Border repeat mode.",
        "",
    ),
    Line::new(
        "tint",
        "boolean",
        "false",
        "Tint art with the element colour.",
        "",
    ),
];
pub const SOUND: &[Line] = &[
    Line::new(
        "clips",
        "[path, ...]",
        "[]",
        "Audio clips relative to this pack or shared assets; empty only at volume 0.",
        "",
    ),
    Line::new(
        "volume",
        "number | [low,high]",
        "1",
        "Gain or sampling range.",
        "",
    ),
    Line::new(
        "pitch",
        "number | [low,high]",
        "1",
        "Playback rate or sampling range.",
        "",
    ),
];
