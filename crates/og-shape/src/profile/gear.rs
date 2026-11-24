//! Involute spur gear outlines.

use crate::math::DVec2;

use og_core::math::double;

use crate::profile::curve::{arc, segments};
use crate::profile::{Contour, Profile, ProfileError};

/// An involute spur gear (ISO 53 basic rack proportions).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Gear {
    /// Number of teeth, at least 6.
    pub teeth: u32,
    /// Module: pitch diameter / teeth, in millimetres.
    pub module: f64,
    /// Pressure angle, radians; 0 for 20°.
    pub pressure_angle: f64,
}

/// The involute function tan φ − φ.
fn inv(phi: f64) -> f64 {
    double::tan(phi) - phi
}

/// A gear outline centred on the origin, a tooth centred on +x.
/// Addendum 1 m, dedendum 1.25 m; below the base circle the flank runs
/// radially to the root. Flank-to-tip and flank-to-root points are
/// corners; the involutes and arcs are smooth.
pub fn gear(d: &Gear) -> Result<Profile, ProfileError> {
    if d.teeth < 6 {
        return Err(ProfileError::TooFewPoints(d.teeth as usize));
    }
    if !d.module.is_finite() || !d.pressure_angle.is_finite() {
        return Err(ProfileError::NotFinite("module"));
    }
    if d.module <= 0.0 {
        return Err(ProfileError::NotPositive("module"));
    }
    let alpha = if d.pressure_angle == 0.0 {
        double::to_radians(20.0)
    } else {
        d.pressure_angle
    };
    let z = d.teeth as f64;
    let m = d.module;
    let r = m * z / 2.0;
    let rb = r * double::cos(alpha);
    let ra = r + m;
    let rf = r - 1.25 * m;
    // Half the tooth's angular width at radius ρ (on or above the base circle).
    let half =
        |rho: f64| double::PI / (2.0 * z) + inv(alpha) - inv(double::acos((rb / rho).min(1.0)));
    let start = rb.max(rf);
    let flank_n = 8;
    let flank: Vec<(f64, f64)> = (0..=flank_n)
        .map(|i| {
            let rho = start + (ra - start) * (i as f64 / flank_n as f64);
            (rho, half(rho))
        })
        .collect();
    let tip_half = half(ra);
    if tip_half <= 0.0 {
        return Err(ProfileError::RadiusTooLarge(0));
    }
    let pitch = double::TAU / z;
    let mut c = Contour::default();
    let mut push = |p: DVec2, sharp: bool| {
        c.points.push(p);
        c.sharp.push(sharp);
    };
    let polar = |rho: f64, a: f64| {
        let (s, co) = double::sin_cos(a);
        DVec2::new(rho * co, rho * s)
    };
    for k in 0..d.teeth {
        let a0 = pitch * k as f64;
        // Rising flank (clockwise side), from the root up.
        if rf < rb {
            push(polar(rf, a0 - flank[0].1), true);
        }
        for (i, &(rho, h)) in flank.iter().enumerate() {
            push(polar(rho, a0 - h), i == 0 && rf >= rb || i == flank_n);
        }
        // The tip arc, between the two flanks.
        let tip_n = segments(ra, 2.0 * tip_half).max(2);
        let tip = arc(DVec2::ZERO, ra, a0 - tip_half, 2.0 * tip_half, tip_n);
        for p in &tip[1..tip.len() - 1] {
            push(*p, false);
        }
        // Falling flank.
        for (i, &(rho, h)) in flank.iter().enumerate().rev() {
            push(polar(rho, a0 + h), i == flank_n || i == 0 && rf >= rb);
        }
        if rf < rb {
            push(polar(rf, a0 + flank[0].1), true);
        }
        // The root arc to the next tooth.
        let from = a0 + flank[0].1;
        let to = a0 + pitch - flank[0].1;
        let root_n = segments(rf, to - from).max(2);
        let root = arc(DVec2::ZERO, rf, from, to - from, root_n);
        for p in &root[1..root.len() - 1] {
            push(*p, false);
        }
    }
    Ok(Profile::from_contour(c))
}
