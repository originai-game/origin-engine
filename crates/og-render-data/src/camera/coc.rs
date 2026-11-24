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

#[cfg(test)]
mod tests {
    use super::*;

    fn physical() -> CocSource {
        // f=1m, s=5m, N=1, fitted height=0.125m -> radius 1−5/z.
        CocSource {
            focal_length: 1000.0,
            gate_height: 125.0,
            aperture: 1.0,
            focus_distance: 5.0,
            max_blur: 100.0,
            ..Default::default()
        }
    }

    fn band(distance: f32) -> CocSource {
        CocSource {
            focus_distance: distance,
            focus_range: 1.0,
            falloff: 2.0,
            max_blur: 1.0,
            ..Default::default()
        }
    }

    #[test]
    fn interrupted_three_source_mix_preserves_counterexample_and_tilt() {
        // The two different bands both saturate to −1 at z=2 and 5, and
        // are both zero at z=8. They are not identical at every depth.
        let (held, drop) = CocMix::one(physical()).blend(band(8.0), 1.0 / 3.0, &mut 0);
        assert_eq!(drop, None);
        let (mix, drop) = held.blend(band(9.0), 0.25, &mut 0);
        assert_eq!(drop, None);
        assert_eq!(mix.sources.iter().filter(|s| s.weight > 0.0).count(), 3);
        for (z, expected) in [(2.0, -1.25), (5.0, -0.5), (8.0, 0.1875)] {
            assert!((mix.radius(z, 0.5) - expected).abs() < 1e-6);
        }
        let tilt = CocSource {
            tilt_shift: 0.1,
            ..band(8.0)
        };
        let (tilted, _) = mix.blend(tilt, 0.2, &mut 0);
        for y in [0.1, 0.45, 0.9] {
            assert!(
                (tilted.radius(3.0, y) - (mix.radius(3.0, y) * 0.8 + tilt.radius(3.0, y) * 0.2))
                    .abs()
                    < 1e-6
            );
        }
    }

    #[test]
    fn fifth_source_drops_smallest_weight_and_coalesces_before_capacity() {
        let (two, _) = CocMix::one(band(2.0)).blend(band(4.0), 0.2, &mut 0);
        let (three, _) = two.blend(band(6.0), 0.3, &mut 0);
        let (four, _) = three.blend(band(8.0), 0.4, &mut 0);
        // Before overflow weights are .3024/.0756/.162/.36/.1.
        let mut excluded = 0;
        let (five, drop) = four.blend(band(10.0), 0.1, &mut excluded);
        let dropped = drop.unwrap();
        assert_eq!(dropped.source, band(4.0));
        assert!((dropped.weight - 0.0756).abs() < 1e-6);
        let expected = [(2.0, 0.3024), (6.0, 0.162), (8.0, 0.36), (10.0, 0.1)];
        for (got, (focus, w)) in five.sources.iter().zip(expected) {
            assert_eq!(got.source, band(focus));
            assert!((got.weight - w / 0.9244).abs() < 1e-6);
        }
        let expected_radius: f32 = expected
            .into_iter()
            .map(|(s, w)| band(s).radius(5.2, 0.5) * w / 0.9244)
            .sum();
        assert!((five.radius(5.2, 0.5) - expected_radius).abs() < 1e-6);
        let (later, drop) = four.blend(band(10.0), 0.5, &mut excluded);
        assert_eq!(drop, None, "a capacity drop completes once, not each tick");
        assert!(
            !later
                .sources
                .iter()
                .any(|s| s.weight > 0.0 && s.source == band(4.0))
        );
        let (_, drop) = four.blend(band(4.0), 0.1, &mut 0);
        assert_eq!(
            drop, None,
            "identical primitive coalesces in the full stack"
        );
        let mut incoming_excluded = 0;
        let (_, drop) = four.blend(band(10.0), 0.01, &mut incoming_excluded);
        assert_eq!(
            drop.unwrap().source,
            band(10.0),
            "incoming source participates in minimum selection"
        );
        let (later, drop) = four.blend(band(10.0), 0.9, &mut incoming_excluded);
        assert_eq!(drop, None);
        assert!(
            !later
                .sources
                .iter()
                .any(|s| s.weight > 0.0 && s.source == band(10.0))
        );
    }
}
