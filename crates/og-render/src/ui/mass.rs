//! Mass elements (`docs/ui.md` §14): damage numbers, overhead bars,
//! nameplate text, loot labels and status icons at thousands per frame.
//!
//! A [`UiMassTemplate`] is declared once: what each element draws (a
//! number formatted on the GPU, one of the template's strings, a bar, an
//! icon), its size, its motion as a timeline over its life
//! ([`UiMassKey`]), how distance scales it and whether the scene hides it.
//! Each element is a [`UiMassInstance`] of a few values (position, value,
//! colour, start tick, seed). A [`UiMassBatch`] of them is one instanced
//! draw of the overlay, sorted by layer with the rest of the UI: the vertex
//! stage projects the position, evaluates the timeline at the element's
//! age, formats the number and places each glyph (`shaders/ui_mass.wgsl`),
//! so nothing is laid out per element on the CPU.
//!
//! [`UiMassTemplate::sample`] is the timeline on the CPU, the same
//! arithmetic as the shader's `sample_keys` (`tests` hold them to each
//! other's values through [`format_number`] and `sample`).

use crate::{FontHandle, TextureHandle};
use glam::Vec2;
use std::sync::Arc;

/// What each element of a template draws.
#[derive(Clone, Debug, PartialEq)]
pub enum UiMassKind {
    /// The instance's value, formatted on the GPU in the template's font:
    /// `decimals` digits after the point, a `+` on positive values with
    /// `plus`, and with `compact` 12.3k past 10,000 and 1.2M past a million.
    Number {
        decimals: u32,
        plus: bool,
        compact: bool,
    },
    /// One of `strings` in the template's font (the value is its index):
    /// nameplates and loot labels, each string laid out once.
    Text { strings: Vec<String> },
    /// A bar `size` wide: a back, a ghost to `aux[2]` (a fraction: the
    /// health just lost, draining) and a fill to the value (0 to 1) in the
    /// instance's colour, with rounded ends.
    Bar {
        back: [f32; 4],
        ghost: [f32; 4],
        radius: f32,
    },
    /// Region `value` (an index) of `frames` (uv rectangles) of the
    /// template's texture, `size` across, turned by the instance's
    /// rotation.
    Icon { frames: Vec<[f32; 4]> },
    /// A particle: an analytic shape `size` across in the instance's
    /// colour, turned by its rotation (UI particles, §5.5).
    Particle { shape: UiParticleShape },
}

/// A [`UiMassKind::Particle`]'s shape.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiParticleShape {
    /// A rectangle `size` (confetti; a long thin one is a streak).
    #[default]
    Square,
    Circle,
    Diamond,
}

/// How an element's timeline moves between keys.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiMassEase {
    #[default]
    Linear,
    /// Fast then slow (quadratic).
    Out,
    /// Slow then fast (quadratic).
    In,
    /// Out, past the next key and back (a pop).
    Back,
    /// Holds the key until the next one.
    Step,
}

impl UiMassEase {
    pub(crate) fn code(self) -> f32 {
        match self {
            UiMassEase::Linear => 0.0,
            UiMassEase::Out => 1.0,
            UiMassEase::In => 2.0,
            UiMassEase::Back => 3.0,
            UiMassEase::Step => 4.0,
        }
    }

    /// The eased fraction of `k` (0 to 1).
    pub fn apply(self, k: f32) -> f32 {
        let k = k.clamp(0.0, 1.0);
        match self {
            UiMassEase::Linear => k,
            UiMassEase::Out => 1.0 - (1.0 - k) * (1.0 - k),
            UiMassEase::In => k * k,
            UiMassEase::Back => {
                let c = 1.70158;
                let u = k - 1.0;
                1.0 + (c + 1.0) * u * u * u + c * u * u
            }
            UiMassEase::Step => 0.0,
        }
    }
}

/// A key of the element's timeline at `t` (a fraction of its life): its
/// offset from the projected point (CSS px, y down), scale and opacity,
/// reached from the key before by `ease`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMassKey {
    pub t: f32,
    pub offset: Vec2,
    pub scale: f32,
    pub alpha: f32,
    pub ease: UiMassEase,
}

impl Default for UiMassKey {
    fn default() -> UiMassKey {
        UiMassKey {
            t: 0.0,
            offset: Vec2::ZERO,
            scale: 1.0,
            alpha: 1.0,
            ease: UiMassEase::Linear,
        }
    }
}

/// The most keys a timeline keeps.
pub const UI_MASS_KEYS: usize = 8;

/// The most quads one element draws (characters of a string or number).
pub const UI_MASS_QUADS: u32 = 32;

/// The most batches one frame draws; past them a batch is not drawn.
pub const UI_MASS_BATCHES: usize = 1024;

/// Table entries per row of the table texture (`ui_mass.wgsl`'s `table`).
pub(crate) const TABLE_WIDTH: u32 = 256;

/// Where an instance's position is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiMassSpace {
    /// A world point, projected through the frame's camera.
    #[default]
    World,
    /// CSS px of the overlay.
    Screen,
}

/// Whether the scene hides an element (a depth test of its point against
/// the frame's depth, five taps around it; the frame then runs its depth
/// prepass).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum UiOcclusion {
    #[default]
    None,
    /// Drawn at `alpha` where the scene is in front.
    Fade { alpha: f32 },
    /// Not drawn where the scene is in front.
    Hide,
}

/// A mass element's template. See the module documentation.
#[derive(Clone, Debug, PartialEq)]
pub struct UiMassTemplate {
    pub kind: UiMassKind,
    /// Text: its font (one page). Icon: `texture`.
    pub font: Option<FontHandle>,
    pub texture: Option<TextureHandle>,
    /// Text and numbers: the em size; bars and icons: width and height
    /// (CSS px; `size.y` is ignored by text).
    pub size: Vec2,
    /// The point of the element on its position, fractions (0.5, 1: the
    /// bottom centre sits on it).
    pub pivot: Vec2,
    /// Ticks an element lives (its timeline runs over them); 0: it is
    /// drawn this frame only, at its first key.
    pub lifetime: u32,
    pub keys: Vec<UiMassKey>,
    /// Each element drifts sideways up to this far (CSS px) over its life,
    /// by its seed; numbers that land on one point fan out.
    pub drift: f32,
    /// Added to the offset's y, times the life fraction squared (CSS px).
    pub gravity: f32,
    /// The instances' velocity slows to rest over the life (an ease-out
    /// of the distance travelled) instead of keeping its speed.
    pub drag: bool,
    /// World space: the scale at `distance.x` world units is 1, nearer
    /// grows and farther shrinks, clamped to `distance.y`..`distance.z`.
    /// `distance.x` 0: a constant screen size.
    pub distance: glam::Vec3,
    pub occlusion: UiOcclusion,
    /// Text: an outline from the distance field, CSS px and linear RGBA.
    pub outline: Option<(f32, [f32; 4])>,
}

impl Default for UiMassTemplate {
    fn default() -> UiMassTemplate {
        UiMassTemplate {
            kind: UiMassKind::Number {
                decimals: 0,
                plus: false,
                compact: false,
            },
            font: None,
            texture: None,
            size: Vec2::new(18.0, 18.0),
            pivot: Vec2::new(0.5, 0.5),
            lifetime: 0,
            keys: Vec::new(),
            drift: 0.0,
            gravity: 0.0,
            drag: false,
            distance: glam::Vec3::new(0.0, 0.25, 4.0),
            occlusion: UiOcclusion::None,
            outline: None,
        }
    }
}

impl UiMassTemplate {
    /// The keys as drawn: at most [`UI_MASS_KEYS`], in order of `t`, and
    /// one at 0 when there are none.
    pub fn keys_drawn(&self) -> Vec<UiMassKey> {
        let mut keys: Vec<UiMassKey> = self.keys.iter().take(UI_MASS_KEYS).copied().collect();
        og_core::sort::sort_by(&mut keys, |a, b| a.t.total_cmp(&b.t));
        if keys.is_empty() {
            keys.push(UiMassKey::default());
        }
        keys
    }

    /// The timeline at life fraction `t`: (offset, scale, alpha), as the
    /// shader samples it.
    pub fn sample(&self, t: f32) -> (Vec2, f32, f32) {
        sample_keys(&self.keys_drawn(), t)
    }

    /// The quads one element draws.
    pub fn quads(&self) -> u32 {
        match &self.kind {
            UiMassKind::Number { .. } => 16,
            UiMassKind::Text { strings } => strings
                .iter()
                .map(|s| s.chars().count() as u32)
                .max()
                .unwrap_or(1)
                .clamp(1, UI_MASS_QUADS),
            UiMassKind::Bar { .. } => 3,
            UiMassKind::Icon { .. } | UiMassKind::Particle { .. } => 1,
        }
    }
}

/// The keys at `t` (sorted, at least one).
pub(crate) fn sample_keys(keys: &[UiMassKey], t: f32) -> (Vec2, f32, f32) {
    let at = |k: &UiMassKey| (k.offset, k.scale, k.alpha);
    let first = &keys[0];
    if t <= first.t {
        return at(first);
    }
    for w in keys.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        if t <= b.t {
            let span = (b.t - a.t).max(1e-6);
            let e = b.ease.apply((t - a.t) / span);
            return (
                a.offset + (b.offset - a.offset) * e,
                a.scale + (b.scale - a.scale) * e,
                a.alpha + (b.alpha - a.alpha) * e,
            );
        }
    }
    at(keys.last().expect("one key"))
}

/// The symbols of a [`UiMassKind::Number`]'s value, as the shader writes
/// them: digits `0`–`9`, `+`, `-`, `.`, `k`, `M`.
pub fn format_number(value: f32, decimals: u32, plus: bool, compact: bool) -> String {
    let mut a = value.abs();
    let mut dec = decimals.min(3);
    let mut suffix = "";
    if compact && a >= 999_950.0 {
        a /= 1_000_000.0;
        suffix = "M";
        dec = 1;
    } else if compact && a >= 10_000.0 {
        a /= 1000.0;
        suffix = "k";
        dec = 1;
    }
    let scale = [1.0f32, 10.0, 100.0, 1000.0][dec as usize];
    let fixed = (a * scale + 0.5).floor().min(4.0e9) as u64;
    let int = fixed / scale as u64;
    let frac = fixed % scale as u64;
    let mut s = String::new();
    if value < 0.0 && fixed > 0 {
        s.push('-');
    } else if plus && fixed > 0 {
        s.push('+');
    }
    s.push_str(&int.to_string());
    if dec > 0 {
        s.push('.');
        s.push_str(&format!("{frac:0width$}", width = dec as usize));
    }
    s.push_str(suffix);
    s
}

/// One element. See the module documentation.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UiMassInstance {
    /// A world point, or CSS px of the overlay ([`UiMassSpace::Screen`]).
    pub position: [f32; 3],
    /// The number, the string or frame index, or the bar's fill (0 to 1).
    pub value: f32,
    /// Linear RGBA: the text's or icon's tint, the bar's fill.
    pub color: [f32; 4],
    /// The tick it started on ([`UiList::tick`](super::UiList::tick)'s
    /// clock).
    pub start: u32,
    /// Its drift's side, any value.
    pub seed: u32,
    /// Multiplies its timeline's scale (a critical hit).
    pub scale: f32,
    /// Its batch in the frame; set when the frame is built.
    pub batch: u32,
    /// x, y: moved by this many CSS px; z: a bar's ghost; w: moved
    /// sideways by this many CSS px over its life (its own drift).
    pub aux: [f32; 4],
    /// x, y: CSS px travelled over its life (eased by the template's
    /// drag); z: turns over its life; w: its starting rotation, radians
    /// clockwise (icons and particles).
    pub velocity: [f32; 4],
}

/// [`UiMassInstance`] as the mass pipeline's instance vertex buffer.
pub(crate) const MASS_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<UiMassInstance>() as u64,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![
        0 => Float32x4, 1 => Float32x4, 2 => Uint32x2, 3 => Float32, 4 => Uint32, 5 => Float32x4,
        6 => Float32x4
    ],
};

/// One draw of mass elements of a template.
#[derive(Clone, Debug, PartialEq)]
pub struct UiMassBatch {
    pub template: Arc<UiMassTemplate>,
    pub space: UiMassSpace,
    pub instances: Vec<UiMassInstance>,
    /// The overlay layer it sorts on.
    pub layer: i32,
    /// A clip rectangle of [`crate::Overlay::clips`] (with its shape and
    /// occluder, if any).
    pub clip: Option<u32>,
}

/// A depth test against the scene for what uses clip rectangle `rect` of
/// [`crate::Overlay::clips`] (`docs/ui.md` §14: anchored elements): the
/// frame's view depth around `at` (CSS px) against `depth` (world units
/// along the camera's forward axis), five taps; where the scene is nearer
/// by more than `bias`, what uses the rectangle draws at `alpha` (0: not
/// at all), fading over `soft` world units. A frame with occluders runs its
/// depth prepass.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiOccluder {
    pub rect: u32,
    pub at: Vec2,
    pub depth: f32,
    pub alpha: f32,
    pub bias: f32,
    pub soft: f32,
}

/// `struct MassTemplate` in ui_mass.wgsl: one batch's template.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct TemplateGpu {
    head: [u32; 4],
    head2: [u32; 4],
    size: [f32; 4],
    dist: [f32; 4],
    occl: [f32; 4],
    outline: [f32; 4],
    misc: [f32; 4],
    back: [f32; 4],
    ghost: [f32; 4],
    clip: [f32; 4],
    clip_head: [u32; 4],
    keys: [[f32; 4]; UI_MASS_KEYS],
    offsets: [[f32; 4]; UI_MASS_KEYS],
}

/// Texels of the table texture one template takes (`ui_mass.wgsl`'s
/// `mass_template`), at the texture's start; the glyph entries follow them.
pub(crate) const TEMPLATE_TEXELS: usize = 32;

impl TemplateGpu {
    /// Its texels: integers as exact floats.
    pub(crate) fn texels(&self) -> [[f32; 4]; TEMPLATE_TEXELS] {
        let u = |v: [u32; 4]| v.map(|x| x as f32);
        let mut out = [[0.0; 4]; TEMPLATE_TEXELS];
        let rows = [
            u(self.head),
            u(self.head2),
            self.size,
            self.dist,
            self.occl,
            self.outline,
            self.misc,
            self.back,
            self.ghost,
            self.clip,
            u(self.clip_head),
        ];
        out[..rows.len()].copy_from_slice(&rows);
        out[11..11 + UI_MASS_KEYS].copy_from_slice(&self.keys);
        out[11 + UI_MASS_KEYS..11 + 2 * UI_MASS_KEYS].copy_from_slice(&self.offsets);
        out
    }
}

/// `struct MassCamera` in ui_mass.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct MassCameraGpu {
    pub view_proj: [[f32; 4]; 4],
    pub eye: [f32; 4],
    pub forward: [f32; 4],
    pub css: [f32; 4],
    pub tick: [u32; 4],
}

/// The symbols a number draws, in the order of the shader's codes.
const NUMBER_SYMBOLS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '+', '-', '.', 'k', 'M', ' ',
];

const FLAG_WORLD: u32 = 1;
const FLAG_OCCLUDE: u32 = 2;
const FLAG_HIDE: u32 = 4;
const FLAG_OUTLINE: u32 = 8;
const FLAG_PLUS: u32 = 16;
const FLAG_COMPACT: u32 = 32;
const FLAG_DISTANCE: u32 = 64;
const FLAG_DRAG: u32 = 128;

/// A glyph's three table entries: its uv, its plane bounds moved by `pen`
/// (em), its advance.
fn glyph_entries(font: &crate::sprites::FontLayout, ch: char, pen: f32, out: &mut Vec<[f32; 4]>) {
    match font.glyphs.get(&ch) {
        Some(g) => {
            let [al, at, ar, ab] = g.atlas_bounds;
            let size = font.atlas_size.max(Vec2::ONE);
            let [l, b, r, t] = g.plane_bounds;
            out.push([al / size.x, at / size.y, ar / size.x, ab / size.y]);
            out.push([l + pen, b, r + pen, t]);
            out.push([g.advance, 0.0, 0.0, 0.0]);
        }
        None => {
            out.push([0.0; 4]);
            out.push([0.0; 4]);
            out.push([0.0; 4]);
        }
    }
}

/// Batch `b`'s template on the GPU, its glyphs, strings or frames appended
/// to `table`. `clip`: its clip rectangle (x0, y0, x1, y1) and clip head.
pub(crate) fn template_gpu(
    b: &UiMassBatch,
    clip: ([f32; 4], u32),
    font: Option<&crate::sprites::FontLayout>,
    table: &mut Vec<[f32; 4]>,
) -> TemplateGpu {
    let t = &*b.template;
    let base = table.len() as u32;
    let mut flags = 0;
    if b.space == UiMassSpace::World {
        flags |= FLAG_WORLD;
    }
    match t.occlusion {
        UiOcclusion::None => {}
        UiOcclusion::Fade { .. } => flags |= FLAG_OCCLUDE,
        UiOcclusion::Hide => flags |= FLAG_HIDE,
    }
    if t.outline.is_some_and(|(w, c)| w > 0.0 && c[3] > 0.0) {
        flags |= FLAG_OUTLINE;
    }
    if t.distance.x > 0.0 {
        flags |= FLAG_DISTANCE;
    }
    if t.drag {
        flags |= FLAG_DRAG;
    }
    let (kind, entries, decimals) = match &t.kind {
        UiMassKind::Number {
            decimals,
            plus,
            compact,
        } => {
            if *plus {
                flags |= FLAG_PLUS;
            }
            if *compact {
                flags |= FLAG_COMPACT;
            }
            for ch in NUMBER_SYMBOLS {
                match font {
                    Some(f) => glyph_entries(f, ch, 0.0, table),
                    None => table.extend([[0.0; 4]; 3]),
                }
            }
            (0, 16, *decimals)
        }
        UiMassKind::Text { strings } => {
            // The runs first (start, count, width), then every string's
            // glyphs laid out with the font's advances and kerning.
            let runs = table.len();
            table.extend(std::iter::repeat_n([0.0; 4], strings.len()));
            let mut start = 0usize;
            for (i, s) in strings.iter().enumerate() {
                let mut pen = 0.0;
                let mut prev: Option<char> = None;
                let mut count = 0usize;
                for ch in s.chars().take(UI_MASS_QUADS as usize) {
                    if let (Some(f), Some(p)) = (font, prev) {
                        pen += f.kerning.get(&(p, ch)).copied().unwrap_or(0.0);
                    }
                    match font {
                        Some(f) => {
                            glyph_entries(f, ch, pen, table);
                            pen += f.glyphs.get(&ch).map_or(0.0, |g| g.advance);
                        }
                        None => table.extend([[0.0; 4]; 3]),
                    }
                    prev = Some(ch);
                    count += 1;
                }
                table[runs + i] = [start as f32, count as f32, pen, 0.0];
                start += count;
            }
            (1, strings.len() as u32, 0)
        }
        UiMassKind::Bar { .. } => (2, 0, 0),
        UiMassKind::Icon { frames } => {
            table.extend(frames.iter().copied());
            (3, frames.len() as u32, 0)
        }
        UiMassKind::Particle { .. } => (4, 0, 0),
    };
    let keys = t.keys_drawn();
    let mut g = TemplateGpu {
        head: [kind, t.quads(), base, flags],
        head2: [t.lifetime, keys.len() as u32, decimals, entries],
        size: [t.size.x, t.size.y, t.pivot.x, t.pivot.y],
        dist: [t.distance.x, t.distance.y, t.distance.z, t.drift],
        occl: [
            match t.occlusion {
                UiOcclusion::Fade { alpha } => alpha.clamp(0.0, 1.0),
                _ => 0.0,
            },
            0.25,
            0.5,
            t.gravity,
        ],
        clip: clip.0,
        clip_head: [0, 0, 0, clip.1],
        ..Default::default()
    };
    if let Some((w, c)) = t.outline {
        g.outline = c;
        g.misc[0] = w;
    }
    if let UiMassKind::Bar {
        back,
        ghost,
        radius,
    } = &t.kind
    {
        g.back = *back;
        g.ghost = *ghost;
        g.misc[1] = *radius;
    }
    if let UiMassKind::Particle { shape } = &t.kind {
        g.misc[2] = match shape {
            UiParticleShape::Square => 0.0,
            UiParticleShape::Circle => 1.0,
            UiParticleShape::Diamond => 2.0,
        };
    }
    for (i, k) in keys.iter().enumerate() {
        g.keys[i] = [k.t, k.scale, k.alpha, k.ease.code()];
        g.offsets[i] = [k.offset.x, k.offset.y, 0.0, 0.0];
    }
    g
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_format_as_the_shader_writes_them() {
        assert_eq!(format_number(1234.0, 0, false, false), "1234");
        assert_eq!(format_number(-17.0, 0, false, false), "-17");
        assert_eq!(format_number(17.0, 0, true, false), "+17");
        // Zero takes no sign, rounded or not.
        assert_eq!(format_number(0.2, 0, true, false), "0");
        assert_eq!(format_number(7.456, 2, false, false), "7.46");
        assert_eq!(format_number(0.05, 1, false, false), "0.1");
        assert_eq!(format_number(12_345.0, 0, false, true), "12.3k");
        // Under the compact threshold numbers stay whole.
        assert_eq!(format_number(9_999.0, 0, false, true), "9999");
        assert_eq!(format_number(2_500_000.0, 0, true, true), "+2.5M");
    }

    #[test]
    fn a_timeline_eases_between_its_keys_and_holds_its_ends() {
        let tpl = UiMassTemplate {
            keys: vec![
                UiMassKey {
                    t: 1.0,
                    offset: Vec2::new(0.0, -40.0),
                    alpha: 0.0,
                    ease: UiMassEase::Out,
                    ..Default::default()
                },
                UiMassKey {
                    t: 0.0,
                    scale: 2.0,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        // Keys are sorted: t = 0 is the first.
        let (o, s, a) = tpl.sample(0.0);
        assert_eq!((o, s, a), (Vec2::ZERO, 2.0, 1.0));
        // Half way out of a quadratic ease-out is three quarters of the way.
        let (o, s, a) = tpl.sample(0.5);
        assert!((o.y + 30.0).abs() < 1e-4, "{o}");
        assert!((s - 1.25).abs() < 1e-4 && (a - 0.25).abs() < 1e-4);
        assert_eq!(tpl.sample(3.0).2, 0.0);
    }
}
