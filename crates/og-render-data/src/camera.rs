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
