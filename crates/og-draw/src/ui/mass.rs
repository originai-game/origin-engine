//! Mass elements' draw commands.

use crate::{FontHandle, TextureHandle};
use glam::Vec2;
use og_tween::Ease;
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

/// A key of the element's timeline at `t` (a fraction of its life): its
/// offset from the projected point (CSS px, y down), scale and opacity,
/// reached from the key before by `ease`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMassKey {
    pub t: f32,
    pub offset: Vec2,
    pub scale: f32,
    pub alpha: f32,
    pub ease: Ease,
}

impl Default for UiMassKey {
    fn default() -> UiMassKey {
        UiMassKey {
            t: 0.0,
            offset: Vec2::ZERO,
            scale: 1.0,
            alpha: 1.0,
            ease: Ease::Linear,
        }
    }
}

/// The most keys a timeline keeps.
pub const UI_MASS_KEYS: usize = 8;

/// The most quads one element draws (characters of a string or number).
pub const UI_MASS_QUADS: u32 = 32;

/// The most batches one frame draws; past them a batch is not drawn.
pub const UI_MASS_BATCHES: usize = 1024;

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
/// [`crate::Overlay::clips`] (`docs/ui.md` §1: anchored elements): the
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
