//! GENERATED from the platform design tokens (`packages/tokens/tokens.json`,
//! vendored verbatim as `crates/og-ui/tokens/tokens.json`) by
//! `crates/og-ui/scripts/gen-theme.sh`. Do not edit: change the tokens in the
//! platform, run the script, commit both files. `tests/theme.rs` fails when
//! this file differs from what the generator writes.
//!
//! Lengths are CSS pixels, durations milliseconds, colours sRGB as authored
//! (straight alpha); `crate::style` converts them for the renderer.

/// FNV-1a 64 of the token file this was generated from.
pub const SOURCE_FNV1A64: u64 = 0xe4ffc80fff9e3990;

/// An sRGB colour as the tokens author it: 8-bit channels, straight alpha.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Srgba {
    pub rgb: [u8; 3],
    pub alpha: f32,
}

/// The semantic colours (`color` in tokens.json; `docs/design-language.md` §Colour).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    /// `accent`
    pub accent: Srgba,
    /// `accent-active`
    pub accent_active: Srgba,
    /// `accent-fg`
    pub accent_fg: Srgba,
    /// `accent-hover`
    pub accent_hover: Srgba,
    /// `accent-ink`
    pub accent_ink: Srgba,
    /// `accent-soft`
    pub accent_soft: Srgba,
    /// `bg`
    pub bg: Srgba,
    /// `bg-subtle`
    pub bg_subtle: Srgba,
    /// `border`
    pub border: Srgba,
    /// `border-strong`
    pub border_strong: Srgba,
    /// `danger`
    pub danger: Srgba,
    /// `fg`
    pub fg: Srgba,
    /// `fg-inverse`
    pub fg_inverse: Srgba,
    /// `fg-muted`
    pub fg_muted: Srgba,
    /// `fg-subtle`
    pub fg_subtle: Srgba,
    /// `fill`
    pub fill: Srgba,
    /// `fill-strong`
    pub fill_strong: Srgba,
    /// `focus`
    pub focus: Srgba,
    /// `info`
    pub info: Srgba,
    /// `material-control`
    pub material_control: Srgba,
    /// `material-control-hover`
    pub material_control_hover: Srgba,
    /// `material-edge`
    pub material_edge: Srgba,
    /// `material-glow`
    pub material_glow: Srgba,
    /// `material-regular`
    pub material_regular: Srgba,
    /// `material-rim-soft`
    pub material_rim_soft: Srgba,
    /// `material-sheen`
    pub material_sheen: Srgba,
    /// `material-shine`
    pub material_shine: Srgba,
    /// `material-thick`
    pub material_thick: Srgba,
    /// `material-thin`
    pub material_thin: Srgba,
    /// `scrim`
    pub scrim: Srgba,
    /// `shadow`
    pub shadow: Srgba,
    /// `sheen`
    pub sheen: Srgba,
    /// `stage`
    pub stage: Srgba,
    /// `stage-border`
    pub stage_border: Srgba,
    /// `stage-danger`
    pub stage_danger: Srgba,
    /// `stage-fg`
    pub stage_fg: Srgba,
    /// `stage-muted`
    pub stage_muted: Srgba,
    /// `stage-raised`
    pub stage_raised: Srgba,
    /// `stage-surface`
    pub stage_surface: Srgba,
    /// `success`
    pub success: Srgba,
    /// `surface`
    pub surface: Srgba,
    /// `surface-hover`
    pub surface_hover: Srgba,
    /// `surface-raised`
    pub surface_raised: Srgba,
    /// `thumb`
    pub thumb: Srgba,
    /// `warn`
    pub warn: Srgba,
}

/// The light palette (`color`).
pub const LIGHT: Palette = Palette {
    accent: Srgba { rgb: [0xf5, 0xa3, 0x0a], alpha: 1.0 },
    accent_active: Srgba { rgb: [0xc7, 0x7d, 0x00], alpha: 1.0 },
    accent_fg: Srgba { rgb: [0x1d, 0x13, 0x00], alpha: 1.0 },
    accent_hover: Srgba { rgb: [0xe6, 0x95, 0x00], alpha: 1.0 },
    accent_ink: Srgba { rgb: [0x8f, 0x56, 0x00], alpha: 1.0 },
    accent_soft: Srgba { rgb: [0xf5, 0xa3, 0x0a], alpha: 0.14 },
    bg: Srgba { rgb: [0xf2, 0xf2, 0xf7], alpha: 1.0 },
    bg_subtle: Srgba { rgb: [0xe9, 0xe9, 0xee], alpha: 1.0 },
    border: Srgba { rgb: [0xdc, 0xdc, 0xe1], alpha: 1.0 },
    border_strong: Srgba { rgb: [0xc4, 0xc4, 0xc9], alpha: 1.0 },
    danger: Srgba { rgb: [0xb2, 0x00, 0x11], alpha: 1.0 },
    fg: Srgba { rgb: [0x18, 0x18, 0x1a], alpha: 1.0 },
    fg_inverse: Srgba { rgb: [0xfa, 0xfa, 0xfa], alpha: 1.0 },
    fg_muted: Srgba { rgb: [0x5f, 0x5f, 0x66], alpha: 1.0 },
    fg_subtle: Srgba { rgb: [0x63, 0x63, 0x69], alpha: 1.0 },
    fill: Srgba { rgb: [0x78, 0x78, 0x80], alpha: 0.12 },
    fill_strong: Srgba { rgb: [0x78, 0x78, 0x80], alpha: 0.2 },
    focus: Srgba { rgb: [0xb8, 0x72, 0x00], alpha: 1.0 },
    info: Srgba { rgb: [0x00, 0x62, 0xc4], alpha: 1.0 },
    material_control: Srgba { rgb: [0x78, 0x78, 0x80], alpha: 0.16 },
    material_control_hover: Srgba { rgb: [0x78, 0x78, 0x80], alpha: 0.09 },
    material_edge: Srgba { rgb: [0x00, 0x00, 0x00], alpha: 0.07 },
    material_glow: Srgba { rgb: [0xff, 0xff, 0xff], alpha: 0.6 },
    material_regular: Srgba { rgb: [0xfa, 0xfa, 0xfc], alpha: 0.58 },
    material_rim_soft: Srgba { rgb: [0xff, 0xff, 0xff], alpha: 0.6 },
    material_sheen: Srgba { rgb: [0xff, 0xff, 0xff], alpha: 0.5 },
    material_shine: Srgba { rgb: [0xff, 0xff, 0xff], alpha: 0.95 },
    material_thick: Srgba { rgb: [0xf6, 0xf6, 0xf8], alpha: 0.8 },
    material_thin: Srgba { rgb: [0xff, 0xff, 0xff], alpha: 0.36 },
    scrim: Srgba { rgb: [0x00, 0x00, 0x00], alpha: 0.22 },
    shadow: Srgba { rgb: [0x00, 0x00, 0x00], alpha: 1.0 },
    sheen: Srgba { rgb: [0xff, 0xff, 0xff], alpha: 0.32 },
    stage: Srgba { rgb: [0x00, 0x00, 0x00], alpha: 1.0 },
    stage_border: Srgba { rgb: [0x2c, 0x2c, 0x2e], alpha: 1.0 },
    stage_danger: Srgba { rgb: [0xff, 0x6b, 0x5e], alpha: 1.0 },
    stage_fg: Srgba { rgb: [0xf5, 0xf5, 0xf7], alpha: 1.0 },
    stage_muted: Srgba { rgb: [0xa1, 0xa1, 0xa6], alpha: 1.0 },
    stage_raised: Srgba { rgb: [0x1c, 0x1c, 0x1e], alpha: 1.0 },
    stage_surface: Srgba { rgb: [0x14, 0x14, 0x16], alpha: 1.0 },
    success: Srgba { rgb: [0x1a, 0x7a, 0x37], alpha: 1.0 },
    surface: Srgba { rgb: [0xff, 0xff, 0xff], alpha: 1.0 },
    surface_hover: Srgba { rgb: [0xe5, 0xe5, 0xea], alpha: 1.0 },
    surface_raised: Srgba { rgb: [0xff, 0xff, 0xff], alpha: 1.0 },
    thumb: Srgba { rgb: [0xff, 0xff, 0xff], alpha: 1.0 },
    warn: Srgba { rgb: [0xc2, 0x41, 0x0c], alpha: 1.0 },
};

/// The dark palette (`color` overlaid with `dark.color`).
pub const DARK: Palette = Palette {
    accent: Srgba { rgb: [0xff, 0xb2, 0x24], alpha: 1.0 },
    accent_active: Srgba { rgb: [0xff, 0xd3, 0x8a], alpha: 1.0 },
    accent_fg: Srgba { rgb: [0x1d, 0x13, 0x00], alpha: 1.0 },
    accent_hover: Srgba { rgb: [0xff, 0xc0, 0x4d], alpha: 1.0 },
    accent_ink: Srgba { rgb: [0xff, 0xb2, 0x24], alpha: 1.0 },
    accent_soft: Srgba { rgb: [0xff, 0xb2, 0x24], alpha: 0.16 },
    bg: Srgba { rgb: [0x00, 0x00, 0x00], alpha: 1.0 },
    bg_subtle: Srgba { rgb: [0x11, 0x11, 0x13], alpha: 1.0 },
    border: Srgba { rgb: [0x38, 0x38, 0x3a], alpha: 1.0 },
    border_strong: Srgba { rgb: [0x54, 0x54, 0x58], alpha: 1.0 },
    danger: Srgba { rgb: [0xff, 0x8a, 0x82], alpha: 1.0 },
    fg: Srgba { rgb: [0xf5, 0xf5, 0xf7], alpha: 1.0 },
    fg_inverse: Srgba { rgb: [0x18, 0x18, 0x1a], alpha: 1.0 },
    fg_muted: Srgba { rgb: [0xae, 0xae, 0xb2], alpha: 1.0 },
    fg_subtle: Srgba { rgb: [0x98, 0x98, 0x9d], alpha: 1.0 },
    fill: Srgba { rgb: [0x78, 0x78, 0x80], alpha: 0.24 },
    fill_strong: Srgba { rgb: [0x78, 0x78, 0x80], alpha: 0.36 },
    focus: Srgba { rgb: [0xff, 0xb2, 0x24], alpha: 1.0 },
    info: Srgba { rgb: [0x5c, 0xc8, 0xff], alpha: 1.0 },
    material_control: Srgba { rgb: [0xff, 0xff, 0xff], alpha: 0.13 },
    material_control_hover: Srgba { rgb: [0xff, 0xff, 0xff], alpha: 0.07 },
    material_edge: Srgba { rgb: [0xff, 0xff, 0xff], alpha: 0.08 },
    material_glow: Srgba { rgb: [0xff, 0xff, 0xff], alpha: 0.1 },
    material_regular: Srgba { rgb: [0x1e, 0x1e, 0x22], alpha: 0.52 },
    material_rim_soft: Srgba { rgb: [0xff, 0xff, 0xff], alpha: 0.13 },
    material_sheen: Srgba { rgb: [0xff, 0xff, 0xff], alpha: 0.08 },
    material_shine: Srgba { rgb: [0xff, 0xff, 0xff], alpha: 0.34 },
    material_thick: Srgba { rgb: [0x1c, 0x1c, 0x1e], alpha: 0.78 },
    material_thin: Srgba { rgb: [0x2c, 0x2c, 0x30], alpha: 0.28 },
    scrim: Srgba { rgb: [0x00, 0x00, 0x00], alpha: 0.42 },
    shadow: Srgba { rgb: [0x00, 0x00, 0x00], alpha: 1.0 },
    sheen: Srgba { rgb: [0xff, 0xff, 0xff], alpha: 0.32 },
    stage: Srgba { rgb: [0x00, 0x00, 0x00], alpha: 1.0 },
    stage_border: Srgba { rgb: [0x2c, 0x2c, 0x2e], alpha: 1.0 },
    stage_danger: Srgba { rgb: [0xff, 0x6b, 0x5e], alpha: 1.0 },
    stage_fg: Srgba { rgb: [0xf5, 0xf5, 0xf7], alpha: 1.0 },
    stage_muted: Srgba { rgb: [0xa1, 0xa1, 0xa6], alpha: 1.0 },
    stage_raised: Srgba { rgb: [0x1c, 0x1c, 0x1e], alpha: 1.0 },
    stage_surface: Srgba { rgb: [0x14, 0x14, 0x16], alpha: 1.0 },
    success: Srgba { rgb: [0x3d, 0xd6, 0x8c], alpha: 1.0 },
    surface: Srgba { rgb: [0x1c, 0x1c, 0x1e], alpha: 1.0 },
    surface_hover: Srgba { rgb: [0x3a, 0x3a, 0x3c], alpha: 1.0 },
    surface_raised: Srgba { rgb: [0x2c, 0x2c, 0x2e], alpha: 1.0 },
    thumb: Srgba { rgb: [0x63, 0x63, 0x66], alpha: 1.0 },
    warn: Srgba { rgb: [0xff, 0x8a, 0x3d], alpha: 1.0 },
};

/// The twelve category hues (`hue`): categories, levels, badges, series. Never body text.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hues {
    pub blue: Srgba,
    pub brown: Srgba,
    pub cyan: Srgba,
    pub green: Srgba,
    pub indigo: Srgba,
    pub mint: Srgba,
    pub orange: Srgba,
    pub pink: Srgba,
    pub purple: Srgba,
    pub red: Srgba,
    pub teal: Srgba,
    pub yellow: Srgba,
}

pub const HUES_LIGHT: Hues = Hues {
    blue: Srgba { rgb: [0x00, 0x7a, 0xff], alpha: 1.0 },
    brown: Srgba { rgb: [0xa2, 0x84, 0x5e], alpha: 1.0 },
    cyan: Srgba { rgb: [0x32, 0xad, 0xe6], alpha: 1.0 },
    green: Srgba { rgb: [0x34, 0xc7, 0x59], alpha: 1.0 },
    indigo: Srgba { rgb: [0x58, 0x56, 0xd6], alpha: 1.0 },
    mint: Srgba { rgb: [0x00, 0xc7, 0xbe], alpha: 1.0 },
    orange: Srgba { rgb: [0xff, 0x95, 0x00], alpha: 1.0 },
    pink: Srgba { rgb: [0xff, 0x2d, 0x55], alpha: 1.0 },
    purple: Srgba { rgb: [0xaf, 0x52, 0xde], alpha: 1.0 },
    red: Srgba { rgb: [0xff, 0x3b, 0x30], alpha: 1.0 },
    teal: Srgba { rgb: [0x30, 0xb0, 0xc7], alpha: 1.0 },
    yellow: Srgba { rgb: [0xff, 0xcc, 0x00], alpha: 1.0 },
};

pub const HUES_DARK: Hues = Hues {
    blue: Srgba { rgb: [0x0a, 0x84, 0xff], alpha: 1.0 },
    brown: Srgba { rgb: [0xac, 0x8e, 0x68], alpha: 1.0 },
    cyan: Srgba { rgb: [0x64, 0xd2, 0xff], alpha: 1.0 },
    green: Srgba { rgb: [0x30, 0xd1, 0x58], alpha: 1.0 },
    indigo: Srgba { rgb: [0x5e, 0x5c, 0xe6], alpha: 1.0 },
    mint: Srgba { rgb: [0x63, 0xe6, 0xe2], alpha: 1.0 },
    orange: Srgba { rgb: [0xff, 0x9f, 0x0a], alpha: 1.0 },
    pink: Srgba { rgb: [0xff, 0x37, 0x5f], alpha: 1.0 },
    purple: Srgba { rgb: [0xbf, 0x5a, 0xf2], alpha: 1.0 },
    red: Srgba { rgb: [0xff, 0x45, 0x3a], alpha: 1.0 },
    teal: Srgba { rgb: [0x40, 0xc8, 0xe0], alpha: 1.0 },
    yellow: Srgba { rgb: [0xff, 0xd6, 0x0a], alpha: 1.0 },
};

/// The 4px spacing scale (`space`), CSS pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Space {
    pub xl3: f32,
    pub lg: f32,
    pub md: f32,
    pub sm: f32,
    pub xl: f32,
    pub xs: f32,
    pub xxl: f32,
    pub xxs: f32,
}

pub const SPACE: Space = Space {
    xl3: 48.0,
    lg: 16.0,
    md: 12.0,
    sm: 8.0,
    xl: 24.0,
    xs: 4.0,
    xxl: 32.0,
    xxs: 2.0,
};

/// The concentric radius scale (`radius`), CSS pixels; `button` is the capsule.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Radius {
    pub block: f32,
    pub button: f32,
    pub card: f32,
    pub control: f32,
    pub field: f32,
    pub panel: f32,
    pub sheet: f32,
}

pub const RADIUS: Radius = Radius {
    block: 10.0,
    button: 9999.0,
    card: 14.0,
    control: 6.0,
    field: 8.0,
    panel: 18.0,
    sheet: 22.0,
};

/// One step of the type ramp (`font.scale`): size and leading in CSS pixels,
/// Latin tracking in em, and the leading Chinese text takes (`font.cjk`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeStep {
    pub size: f32,
    pub line_height: f32,
    pub tracking_em: f32,
    pub cjk_line_height: f32,
    /// Tracking of text with CJK characters, in em (CSS `letter-spacing`, which spaces Han too); 0 in the tokens.
    pub cjk_tracking_em: f32,
}

/// The type ramp (`font.scale`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeRamp {
    /// `text-2xl`
    pub xl2: TypeStep,
    /// `text-3xl`
    pub xl3: TypeStep,
    /// `text-4xl`
    pub xl4: TypeStep,
    /// `text-5xl`
    pub xl5: TypeStep,
    /// `text-6xl`
    pub xl6: TypeStep,
    /// `text-7xl`
    pub xl7: TypeStep,
    /// `text-8xl`
    pub xl8: TypeStep,
    /// `text-base`
    pub base: TypeStep,
    /// `text-body`
    pub body: TypeStep,
    /// `text-caption`
    pub caption: TypeStep,
    /// `text-display-2xl`
    pub display_xl2: TypeStep,
    /// `text-display-lg`
    pub display_lg: TypeStep,
    /// `text-display-md`
    pub display_md: TypeStep,
    /// `text-display-sm`
    pub display_sm: TypeStep,
    /// `text-display-xl`
    pub display_xl: TypeStep,
    /// `text-lg`
    pub lg: TypeStep,
    /// `text-meta`
    pub meta: TypeStep,
    /// `text-reading`
    pub reading: TypeStep,
    /// `text-sm`
    pub sm: TypeStep,
    /// `text-xl`
    pub xl: TypeStep,
    /// `text-xs`
    pub xs: TypeStep,
}

pub const TYPE_RAMP: TypeRamp = TypeRamp {
    xl2: TypeStep { size: 22.0, line_height: 28.0, tracking_em: -0.015, cjk_line_height: 32.0, cjk_tracking_em: 0.0 },
    xl3: TypeStep { size: 26.0, line_height: 32.0, tracking_em: -0.02, cjk_line_height: 36.0, cjk_tracking_em: 0.0 },
    xl4: TypeStep { size: 30.0, line_height: 36.0, tracking_em: -0.022, cjk_line_height: 40.0, cjk_tracking_em: 0.0 },
    xl5: TypeStep { size: 36.0, line_height: 40.0, tracking_em: -0.025, cjk_line_height: 46.0, cjk_tracking_em: 0.0 },
    xl6: TypeStep { size: 44.0, line_height: 48.0, tracking_em: -0.03, cjk_line_height: 54.0, cjk_tracking_em: 0.0 },
    xl7: TypeStep { size: 52.0, line_height: 56.0, tracking_em: -0.032, cjk_line_height: 62.0, cjk_tracking_em: 0.0 },
    xl8: TypeStep { size: 64.0, line_height: 64.0, tracking_em: -0.035, cjk_line_height: 74.0, cjk_tracking_em: 0.0 },
    base: TypeStep { size: 16.0, line_height: 24.0, tracking_em: 0.0, cjk_line_height: 24.0, cjk_tracking_em: 0.0 },
    body: TypeStep { size: 14.0, line_height: 20.0, tracking_em: 0.0, cjk_line_height: 20.0, cjk_tracking_em: 0.0 },
    caption: TypeStep { size: 13.0, line_height: 18.0, tracking_em: 0.0, cjk_line_height: 18.0, cjk_tracking_em: 0.0 },
    display_xl2: TypeStep { size: 56.0, line_height: 60.0, tracking_em: -0.025, cjk_line_height: 68.0, cjk_tracking_em: 0.0 },
    display_lg: TypeStep { size: 40.0, line_height: 44.0, tracking_em: -0.02, cjk_line_height: 50.0, cjk_tracking_em: 0.0 },
    display_md: TypeStep { size: 34.0, line_height: 38.0, tracking_em: -0.016, cjk_line_height: 44.0, cjk_tracking_em: 0.0 },
    display_sm: TypeStep { size: 28.0, line_height: 32.0, tracking_em: -0.012, cjk_line_height: 38.0, cjk_tracking_em: 0.0 },
    display_xl: TypeStep { size: 48.0, line_height: 52.0, tracking_em: -0.022, cjk_line_height: 60.0, cjk_tracking_em: 0.0 },
    lg: TypeStep { size: 18.0, line_height: 26.0, tracking_em: 0.0, cjk_line_height: 26.0, cjk_tracking_em: 0.0 },
    meta: TypeStep { size: 13.0, line_height: 18.0, tracking_em: 0.0, cjk_line_height: 18.0, cjk_tracking_em: 0.0 },
    reading: TypeStep { size: 15.0, line_height: 24.0, tracking_em: 0.0, cjk_line_height: 24.0, cjk_tracking_em: 0.0 },
    sm: TypeStep { size: 14.0, line_height: 20.0, tracking_em: 0.0, cjk_line_height: 20.0, cjk_tracking_em: 0.0 },
    xl: TypeStep { size: 20.0, line_height: 28.0, tracking_em: -0.01, cjk_line_height: 30.0, cjk_tracking_em: 0.0 },
    xs: TypeStep { size: 12.0, line_height: 16.0, tracking_em: 0.0, cjk_line_height: 16.0, cjk_tracking_em: 0.0 },
};

/// Control, row and header heights and the gutter (`density`), CSS pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Density {
    pub control_lg: f32,
    pub control_md: f32,
    pub control_sm: f32,
    pub gutter: f32,
    pub header: f32,
    pub row: f32,
    pub section: f32,
}

/// Compact density (`density.compact`), the default for tools.
pub const COMPACT: Density = Density {
    control_lg: 36.0,
    control_md: 32.0,
    control_sm: 28.0,
    gutter: 16.0,
    header: 48.0,
    row: 36.0,
    section: 24.0,
};

/// Comfortable density (`density.comfortable`), for browsing surfaces.
pub const COMFORTABLE: Density = Density {
    control_lg: 40.0,
    control_md: 36.0,
    control_sm: 32.0,
    gutter: 24.0,
    header: 56.0,
    row: 44.0,
    section: 40.0,
};

/// Touch density (`density.touch`): coarse pointers and widths under 768px.
/// Keys the tokens do not raise (`gutter`, `section`) keep the compact value.
pub const TOUCH: Density = Density {
    control_lg: 48.0,
    control_md: 44.0,
    control_sm: 44.0,
    gutter: 16.0,
    header: 52.0,
    row: 48.0,
    section: 24.0,
};

/// Motion durations (`motion.duration`), milliseconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Durations {
    pub base: f32,
    pub bounce: f32,
    pub fast: f32,
    pub instant: f32,
    pub slow: f32,
    pub spring: f32,
}

pub const DURATIONS_MS: Durations = Durations {
    base: 240.0,
    bounce: 640.0,
    fast: 180.0,
    instant: 120.0,
    slow: 400.0,
    spring: 520.0,
};

/// A spring (`motion.spring`): damping ratio and response in milliseconds.
///
/// Springs as damping ratio (1 = critically damped, below 1 overshoots) and
/// response (seconds to reach the target; not a duration), the parameters Apple
/// ships. build.mjs emits --ease-spring-<name> (a linear() easing),
/// --dur-spring-<name> (its settle time) and --spring-<name>-damping/-response
/// for the runtime spring (ui-svelte motion/spring.ts); Swift gets the pair for
/// .spring(response:dampingFraction:). default: overlays, menus, highlights;
/// sheet: sheets and drawers; momentum: releases that carried velocity (a
/// flick, a throw).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    pub damping: f32,
    pub response_ms: f32,
}

pub const SPRING_DEFAULT: Spring = Spring { damping: 1.0, response_ms: 350.0 };
pub const SPRING_MOMENTUM: Spring = Spring { damping: 0.8, response_ms: 400.0 };
pub const SPRING_SHEET: Spring = Spring { damping: 0.85, response_ms: 300.0 };

/// `motion.easing.exit`: cubic Bézier control points (x1, y1, x2, y2).
pub const EASE_EXIT: [f32; 4] = [0.4, 0.0, 1.0, 1.0];
/// `motion.easing.out`: cubic Bézier control points (x1, y1, x2, y2).
pub const EASE_OUT: [f32; 4] = [0.23, 1.0, 0.32, 1.0];

/// One inset layer of a `shadow-*` token, drawn inside a surface's edge:
/// the surface less itself moved by (`dx`, `dy`) and shrunk by `spread`,
/// in `color` (CSS px; the tokens' edge layers have no blur).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InsetShadow {
    pub dx: f32,
    pub dy: f32,
    pub spread: f32,
    pub color: Srgba,
}

/// `shadow.edge`: the card edge's inset layers.
pub const SHADOW_EDGE_LIGHT: &[InsetShadow] = &[
    InsetShadow { dx: 0.0, dy: 0.0, spread: 0.5, color: Srgba { rgb: [0x00, 0x00, 0x00], alpha: 0.06 } },
];

/// `dark.shadow.edge`: the card edge's inset layers.
pub const SHADOW_EDGE_DARK: &[InsetShadow] = &[
    InsetShadow { dx: 0.0, dy: 0.5, spread: 0.0, color: Srgba { rgb: [0xff, 0xff, 0xff], alpha: 0.1 } },
    InsetShadow { dx: 0.0, dy: 0.0, spread: 0.5, color: Srgba { rgb: [0xff, 0xff, 0xff], alpha: 0.07 } },
];

/// `vars.press-scale`: how far a pressed control shrinks.
pub const PRESS_SCALE: f32 = 0.96;
