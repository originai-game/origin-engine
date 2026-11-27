/// The built-in presets `extends` names by slug: the engine's
/// `templates/looks/<slug>.ts`, compiled into the cook: one per style of
/// the knowledge package's catalogue (`packages/knowledge/looks/catalogue.json`).
pub const PRESETS: &[(&str, &str)] = &[
    (
        "cartoon-2d",
        include_str!("../../../../../templates/looks/cartoon-2d.ts"),
    ),
    (
        "faceted-low-poly",
        include_str!("../../../../../templates/looks/faceted-low-poly.ts"),
    ),
    (
        "inked-comic-3d",
        include_str!("../../../../../templates/looks/inked-comic-3d.ts"),
    ),
    (
        "neon-vector",
        include_str!("../../../../../templates/looks/neon-vector.ts"),
    ),
    (
        "pixel",
        include_str!("../../../../../templates/looks/pixel.ts"),
    ),
    (
        "toy-3d",
        include_str!("../../../../../templates/looks/toy-3d.ts"),
    ),
];

/// The presets' variants (`templates/looks/variants/<slug>/<variant>.ts`,
/// docs/art.md §2.11), compiled into the cook as `(slug, variant, text)`.
pub const VARIANT_LAYERS: &[(&str, &str, &str)] = &[
    (
        "pixel",
        "topdown",
        include_str!("../../../../../templates/looks/variants/pixel/topdown.ts"),
    ),
    (
        "toy-3d",
        "paper",
        include_str!("../../../../../templates/looks/variants/toy-3d/paper.ts"),
    ),
];

/// The moods' layers (`templates/looks/moods/`, docs/art.md §2.10), compiled
/// into the cook: `(mood, None)` is the layer for every style; a keyed entry
/// replaces it for one preset or `<slug>.<variant>`.
pub const MOOD_LAYERS: &[(&str, Option<&str>, &str)] = &[
    (
        "bright",
        None,
        include_str!("../../../../../templates/looks/moods/bright.ts"),
    ),
    (
        "dark",
        None,
        include_str!("../../../../../templates/looks/moods/dark.ts"),
    ),
    (
        "warm",
        None,
        include_str!("../../../../../templates/looks/moods/warm.ts"),
    ),
    (
        "cold",
        None,
        include_str!("../../../../../templates/looks/moods/cold.ts"),
    ),
    (
        "soft",
        None,
        include_str!("../../../../../templates/looks/moods/soft.ts"),
    ),
    (
        "high-contrast",
        None,
        include_str!("../../../../../templates/looks/moods/high-contrast.ts"),
    ),
];

/// The engine's built-in look assets, by bundle id: the canvas textures a
/// look names in `Stylise.paintCanvas` and the palettes it names in
/// `Stylise.palette`, made by `templates/looks/assets/generate.ts` (CC0)
/// and compiled into the cook. A look writes `"@canvas/paper"` or
/// `"@canvas/paper.png"`; a game that names one has it cooked into its
/// bundle under the id, where `og.load.texture`/`og.load.palette` find it
/// too.
pub static BUILTINS: &[(&str, &[u8])] = &[
    (
        "@canvas/linen.png",
        include_bytes!("../../../../../templates/looks/assets/canvas/linen.png"),
    ),
    (
        "@canvas/paper.png",
        include_bytes!("../../../../../templates/looks/assets/canvas/paper.png"),
    ),
    (
        "@canvas/watercolor.png",
        include_bytes!("../../../../../templates/looks/assets/canvas/watercolor.png"),
    ),
    (
        "@palettes/1bit-bright.hex",
        include_bytes!("../../../../../templates/looks/assets/palettes/1bit-bright.hex"),
    ),
    (
        "@palettes/1bit-cold.hex",
        include_bytes!("../../../../../templates/looks/assets/palettes/1bit-cold.hex"),
    ),
    (
        "@palettes/1bit-dark.hex",
        include_bytes!("../../../../../templates/looks/assets/palettes/1bit-dark.hex"),
    ),
    (
        "@palettes/1bit-soft.hex",
        include_bytes!("../../../../../templates/looks/assets/palettes/1bit-soft.hex"),
    ),
    (
        "@palettes/1bit-warm.hex",
        include_bytes!("../../../../../templates/looks/assets/palettes/1bit-warm.hex"),
    ),
    (
        "@palettes/1bit.hex",
        include_bytes!("../../../../../templates/looks/assets/palettes/1bit.hex"),
    ),
    (
        "@palettes/bright32.hex",
        include_bytes!("../../../../../templates/looks/assets/palettes/bright32.hex"),
    ),
    (
        "@palettes/dusk16.hex",
        include_bytes!("../../../../../templates/looks/assets/palettes/dusk16.hex"),
    ),
    (
        "@palettes/handheld4-bright.hex",
        include_bytes!("../../../../../templates/looks/assets/palettes/handheld4-bright.hex"),
    ),
    (
        "@palettes/handheld4-cold.hex",
        include_bytes!("../../../../../templates/looks/assets/palettes/handheld4-cold.hex"),
    ),
    (
        "@palettes/handheld4-dark.hex",
        include_bytes!("../../../../../templates/looks/assets/palettes/handheld4-dark.hex"),
    ),
    (
        "@palettes/handheld4-high-contrast.hex",
        include_bytes!(
            "../../../../../templates/looks/assets/palettes/handheld4-high-contrast.hex"
        ),
    ),
    (
        "@palettes/handheld4-warm.hex",
        include_bytes!("../../../../../templates/looks/assets/palettes/handheld4-warm.hex"),
    ),
    (
        "@palettes/handheld4.hex",
        include_bytes!("../../../../../templates/looks/assets/palettes/handheld4.hex"),
    ),
    (
        "@palettes/lcd2.hex",
        include_bytes!("../../../../../templates/looks/assets/palettes/lcd2.hex"),
    ),
];
