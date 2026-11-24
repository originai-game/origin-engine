//! Physical lens facts. Lengths cross this boundary in millimetres (lens
//! and gate) and metres (scene depth); no ECS, history or renderer state.
//! Thin-lens equations and saturation calibration: Filament's camera model
//! (https://google.github.io/filament/Filament.html#imagingpipeline/physicallybasedcamera).
//! Independently implemented from the published equations; no source port.

use glam::Vec2;

mod coc;
pub use coc::{COC_SOURCE_CAPACITY, CocMix, CocSource, WeightedCoc};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GateFit {
    #[default]
    Vertical,
    Horizontal,
    Fill,
    Overscan,
}

/// An authored physical lens. Zero does not select a photographic preset.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PhysicalLens {
    pub focal_length: f32,
    pub sensor_size: Vec2,
    pub gate_fit: GateFit,
    pub aperture: f32,
    pub focus_distance: f32,
    pub shutter_speed: f32,
    pub iso: f32,
    pub shift: Vec2,
}

/// Signed thin-lens radius as a fraction of output height:
/// `scale + bias / axial_depth_metres`. Negative is in front of focus.
/// Coefficients blend linearly even when their focus planes differ.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ThinLensCoc {
    pub scale: f32,
    pub bias: f32,
}

impl PhysicalLens {
    /// Fitted gate height in mm. Invalid/incomplete lenses have no projection.
    pub fn gate_height(self, aspect: f32) -> Option<f32> {
        let positive = |v: f32| v.is_finite() && v > 0.0;
        if !positive(aspect)
            || !positive(self.focal_length)
            || !positive(self.sensor_size.x)
            || !positive(self.sensor_size.y)
            || !self.shift.is_finite()
        {
            return None;
        }
        let h = self.sensor_size.y;
        let horizontal = self.sensor_size.x / aspect;
        Some(match self.gate_fit {
            GateFit::Vertical => h,
            GateFit::Horizontal => horizontal,
            GateFit::Fill => h.min(horizontal),
            GateFit::Overscan => h.max(horizontal),
        })
    }

    /// Vertical projection focal factor (cotangent of the half angle).
    pub fn focal_y(self, aspect: f32, zoom: f32) -> Option<f32> {
        let gate = self.gate_height(aspect)?;
        let focal = 2.0 * self.focal_length / gate * zoom;
        (focal.is_finite() && focal > 0.0).then_some(focal)
    }

    /// Thin-lens sensor diameter f²/[N(s−f)]·(1−s/z), divided by twice
    /// the fitted gate height to give a radius. Zoom crops both projection
    /// and the displayed radius. Invalid or closed apertures have no CoC.
    pub fn coc_coefficients(self, aspect: f32, zoom: f32) -> Option<ThinLensCoc> {
        let gate = self.gate_height(aspect)? * 0.001;
        let focal = self.focal_length * 0.001;
        if !(self.aperture.is_finite()
            && self.aperture > 0.0
            && self.focus_distance.is_finite()
            && self.focus_distance > focal
            && zoom.is_finite()
            && zoom > 0.0)
        {
            return None;
        }
        let scale =
            focal * focal / (self.aperture * (self.focus_distance - focal)) / (2.0 * gate) * zoom;
        let bias = -scale * self.focus_distance;
        (scale.is_finite() && bias.is_finite()).then_some(ThinLensCoc { scale, bias })
    }

    /// Fraction of the interval represented by the motion vectors that
    /// the shutter exposes. A closed shutter or absent history gives zero.
    /// Values above one extrapolate the measured velocity; no silent cap.
    pub fn shutter_fraction(self, interval: f32) -> f32 {
        if interval.is_finite()
            && interval > 0.0
            && self.shutter_speed.is_finite()
            && self.shutter_speed > 0.0
        {
            self.shutter_speed / interval
        } else {
            0.0
        }
    }

    /// Blend the continuous authored values, not independently derived effects.
    /// The caller resolves different discrete gate fits at their endpoints.
    pub fn blend(self, to: Self, weight: f32) -> Self {
        let l = |a: f32, b: f32| a + (b - a) * weight;
        Self {
            focal_length: l(self.focal_length, to.focal_length),
            sensor_size: self.sensor_size.lerp(to.sensor_size, weight),
            gate_fit: self.gate_fit,
            aperture: l(self.aperture, to.aperture),
            focus_distance: l(self.focus_distance, to.focus_distance),
            shutter_speed: l(self.shutter_speed, to.shutter_speed),
            iso: l(self.iso, to.iso),
            shift: self.shift.lerp(to.shift, weight),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_fit_crops_without_stretching_in_both_orientations() {
        let mut lens = PhysicalLens {
            focal_length: 50.0,
            sensor_size: Vec2::new(36.0, 24.0),
            ..Default::default()
        };
        for (fit, landscape, portrait) in [
            (GateFit::Vertical, 24.0, 24.0),
            (GateFit::Horizontal, 18.0, 72.0),
            (GateFit::Fill, 18.0, 24.0),
            (GateFit::Overscan, 24.0, 72.0),
        ] {
            lens.gate_fit = fit;
            assert_eq!(lens.gate_height(2.0), Some(landscape));
            assert_eq!(lens.gate_height(0.5), Some(portrait));
        }
        lens.gate_fit = GateFit::Vertical;
        assert!((lens.focal_y(2.0, 1.0).unwrap() - 4.166_666_5).abs() < 1e-6);
        assert!((lens.focal_y(2.0, 1.7).unwrap() - 7.083_333).abs() < 1e-6);
        assert_eq!(PhysicalLens::default().focal_y(2.0, 1.0), None);
        lens.sensor_size.x = f32::NAN;
        assert_eq!(lens.focal_y(2.0, 1.0), None);
    }

    #[test]
    fn thin_lens_radius_has_correct_sign_units_focus_and_crop() {
        let mut lens = PhysicalLens {
            focal_length: 50.0,
            sensor_size: Vec2::new(36.0, 24.0),
            aperture: 2.0,
            focus_distance: 5.0,
            ..Default::default()
        };
        let coc = lens.coc_coefficients(16.0 / 9.0, 1.0).unwrap();
        for (depth, pixels) in [
            (2.0, -8.522_727),
            (5.0, 0.0),
            (10.0, 2.840_909),
            (f32::INFINITY, 5.681_818),
        ] {
            assert!(((coc.scale + coc.bias / depth) * 1080.0 - pixels).abs() < 1e-5);
        }
        lens.aperture = 1.0;
        let cropped = lens.coc_coefficients(16.0 / 9.0, 1.7).unwrap();
        assert!((cropped.scale / coc.scale - 3.4).abs() < 1e-6);
        lens.focus_distance = 0.05;
        assert_eq!(lens.coc_coefficients(1.0, 1.0), None);
        assert_eq!(PhysicalLens::default().coc_coefficients(1.0, 1.0), None);
    }

    #[test]
    fn physical_shutter_uses_the_represented_interval_without_a_hidden_cap() {
        let lens = PhysicalLens {
            shutter_speed: 0.01,
            ..Default::default()
        };
        for interval in [1.0 / 30.0, 1.0 / 144.0, 0.023] {
            let vector_pixels = 200.0 * interval;
            assert!((vector_pixels * lens.shutter_fraction(interval) - 2.0).abs() < 1e-6);
        }
        assert!(lens.shutter_fraction(1.0 / 240.0) > 1.0);
        assert_eq!(lens.shutter_fraction(0.0), 0.0);
        assert_eq!(PhysicalLens::default().shutter_fraction(0.02), 0.0);
    }
}
