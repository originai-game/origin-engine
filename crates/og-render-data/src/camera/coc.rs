//! Weighted primitive CoC sources. No curve fitting and no stored history
//! outside the caller's value. Runtime snapshots own these values; shaders
//! evaluate the same source equations at each pixel's axial depth and y.

use super::PhysicalLens;
use glam::Vec2;

/// Mirrored by the IDL's MAX_CAMERA_COC_SOURCES and Camera's source list.
/// Assembly tests check that the public and render boundaries agree.
pub const COC_SOURCE_CAPACITY: usize = 4;

/// One primitive's authored parameters. A positive focal length selects a
/// thin lens; zero selects the artistic band. `gate_height` is the fitted
/// sensor height divided by zoom, in mm; all scene distances are metres.
/// Radius and max_blur are fractions of output height, not pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CocSource {
    pub focal_length: f32,
    pub gate_height: f32,
    pub aperture: f32,
    pub focus_distance: f32,
    pub focus_range: f32,
    pub falloff: f32,
    pub max_blur: f32,
    pub tilt_shift: f32,
}

impl CocSource {
    /// A valid physical channel, or a zero-radius primitive when incomplete.
    /// The validity decision never substitutes an artistic focus band.
    pub fn physical(lens: PhysicalLens, aspect: f32, zoom: f32, max_blur: f32) -> Self {
        if lens.coc_coefficients(aspect, zoom).is_none() {
            return Self::default();
        }
        Self {
            focal_length: lens.focal_length,
            gate_height: lens.gate_height(aspect).unwrap() / zoom,
            aperture: lens.aperture,
            focus_distance: lens.focus_distance,
            max_blur: max_blur.max(0.0),
            ..Self::default()
        }
    }

    /// Signed radius / output height. Equivalent to the physical shader's
    /// equation and the existing artistic band (including its zero guards).
    pub fn radius(self, depth: f32, y: f32) -> f32 {
        if self.focal_length > 0.0 {
            let lens = PhysicalLens {
                focal_length: self.focal_length,
                sensor_size: Vec2::splat(self.gate_height),
                aperture: self.aperture,
                focus_distance: self.focus_distance,
                ..Default::default()
            };
            let Some(coc) = lens.coc_coefficients(1.0, 1.0) else {
                return 0.0;
            };
            return (coc.scale + coc.bias / depth.max(f32::MIN_POSITIVE))
                .clamp(-self.max_blur, self.max_blur);
        }
        let (off, band) = if self.tilt_shift > 0.0 {
            (0.5 - y, self.tilt_shift.clamp(0.0, 0.5))
        } else {
            (depth - self.focus_distance, self.focus_range.max(1e-3))
        };
        let radius = ((off.abs() - band) / (band * self.falloff.max(1e-3)).max(1e-3))
            .clamp(0.0, 1.0)
            * self.max_blur.max(0.0);
        if off < 0.0 { -radius } else { radius }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WeightedCoc {
    pub source: CocSource,
    pub weight: f32,
}

/// Exact primitive mix up to its declared capacity. Entries remain oldest
/// first, including after coalescing; equal-weight overflow drops the oldest.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CocMix {
    pub sources: [WeightedCoc; COC_SOURCE_CAPACITY],
}

impl CocMix {
    pub fn one(source: CocSource) -> Self {
        let mut mix = Self::default();
        mix.sources[0] = WeightedCoc {
            source,
            weight: 1.0,
        };
        mix
    }

    pub fn radius(self, depth: f32, y: f32) -> f32 {
        self.sources
            .iter()
            .filter(|s| s.weight > 0.0)
            .map(|s| s.weight * s.source.radius(depth, y))
            .sum()
    }

    /// Crossfade to one primitive, coalescing before testing capacity.
    /// Returns the dropped primitive and its pre-renormalisation weight.
    /// `excluded` persists for this transition: bits 0..4 name the held
    /// slots and bit 4 the incoming source. A dropped slot cannot reappear
    /// on the next tick. Reset the mask when beginning a new transition.
    pub fn blend(
        self,
        to: CocSource,
        weight: f32,
        excluded: &mut u32,
    ) -> (Self, Option<WeightedCoc>) {
        if weight <= 0.0 {
            return (self, None);
        }
        if weight >= 1.0 {
            return (Self::one(to), None);
        }
        // Five candidates are enough: a held mix plus one incoming shot.
        let mut candidates = [WeightedCoc::default(); COC_SOURCE_CAPACITY + 1];
        let mut slots = [0u32; COC_SOURCE_CAPACITY + 1];
        let mut len = 0;
        let held = self.sources.map(|s| WeightedCoc {
            weight: s.weight * (1.0 - weight),
            ..s
        });
        for (slot, item) in held
            .into_iter()
            .chain([WeightedCoc { source: to, weight }])
            .enumerate()
        {
            let bit = 1 << slot;
            let w = item.weight;
            if w <= 0.0 || *excluded & bit != 0 {
                continue;
            }
            if let Some(index) = candidates[..len]
                .iter()
                .position(|s| s.source == item.source)
            {
                candidates[index].weight += w;
                slots[index] |= bit;
            } else {
                candidates[len] = WeightedCoc {
                    source: item.source,
                    weight: w,
                };
                slots[len] = bit;
                len += 1;
            }
        }
        let dropped = if len > COC_SOURCE_CAPACITY {
            let mut index = 0;
            for i in 1..len {
                if candidates[i].weight < candidates[index].weight {
                    index = i;
                }
            }
            let dropped = candidates[index];
            *excluded |= slots[index];
            candidates.copy_within(index + 1..len, index);
            len -= 1;
            Some(dropped)
        } else {
            None
        };
        let sum: f32 = candidates[..len].iter().map(|s| s.weight).sum();
        let mut out = Self::default();
        for (dst, mut src) in out.sources.iter_mut().zip(candidates.into_iter().take(len)) {
            src.weight /= sum;
            *dst = src;
        }
        (out, dropped)
    }
}
