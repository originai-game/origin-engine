//! Steering behaviours (Reynolds), as pure functions on the movement plane:
//! each answers a desired velocity (or a correction), and an agent blends
//! them by weight. Positions and velocities are plane vectors: x and y in
//! the 2D profile, x and z in 3D.

use og_core::glam::Vec2;

/// Full speed towards `target`.
pub fn seek(pos: Vec2, target: Vec2, speed: f32) -> Vec2 {
    (target - pos).normalize_or_zero() * speed
}

/// Full speed away from `threat` while it is within `radius` (0: always),
/// stronger the closer it is.
pub fn flee(pos: Vec2, threat: Vec2, speed: f32, radius: f32) -> Vec2 {
    let away = pos - threat;
    let d = away.length();
    if radius > 0.0 && d >= radius {
        return Vec2::ZERO;
    }
    let urgency = if radius > 0.0 { 1.0 - d / radius } else { 1.0 };
    away.normalize_or_zero() * speed * urgency.clamp(0.25, 1.0)
}

/// Towards `target`, slowing linearly inside `slow_radius` and stopping
/// within `stop` of it.
pub fn arrive(pos: Vec2, target: Vec2, speed: f32, slow_radius: f32, stop: f32) -> Vec2 {
    let to = target - pos;
    let d = to.length();
    if d <= stop.max(1e-4) {
        return Vec2::ZERO;
    }
    let s = if slow_radius > 0.0 && d < slow_radius {
        speed * d / slow_radius
    } else {
        speed
    };
    to / d * s
}

/// Where a mover at `target` with velocity `target_vel` will be when an
/// agent at `pos` moving at `speed` could reach it (capped at two seconds).
pub fn predict(pos: Vec2, target: Vec2, target_vel: Vec2, speed: f32) -> Vec2 {
    let d = (target - pos).length();
    let t = if speed > 1e-4 {
        (d / speed).min(2.0)
    } else {
        0.0
    };
    target + target_vel * t
}

/// Seek the predicted position of a mover.
pub fn pursue(pos: Vec2, target: Vec2, target_vel: Vec2, speed: f32) -> Vec2 {
    seek(pos, predict(pos, target, target_vel, speed), speed)
}

/// Flee the predicted position of a mover.
pub fn evade(pos: Vec2, threat: Vec2, threat_vel: Vec2, speed: f32, radius: f32) -> Vec2 {
    flee(pos, predict(pos, threat, threat_vel, speed), speed, radius)
}

/// The wander circle: its radius, how far ahead it is, and how fast the
/// point moves round it (radians per second at full noise).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WanderCircle {
    pub radius: f32,
    pub distance: f32,
    pub jitter: f32,
}

/// A wander step: the angle on the wander circle moves by `jitter · dt ·
/// noise` (noise in [-1, 1], from the caller's deterministic source), and
/// the desired velocity points at that spot on the circle held ahead along
/// `heading`. Returns (velocity, new angle).
pub fn wander(
    heading: Vec2,
    angle: f32,
    noise: f32,
    circle: WanderCircle,
    dt: f32,
    speed: f32,
) -> (Vec2, f32) {
    let mut a = angle + noise.clamp(-1.0, 1.0) * circle.jitter * dt;
    // Keep the angle in (-π, π] without a float remainder.
    let tau = std::f32::consts::TAU;
    while a > std::f32::consts::PI {
        a -= tau;
    }
    while a <= -std::f32::consts::PI {
        a += tau;
    }
    let h = if heading.length_squared() > 1e-8 {
        heading.normalize()
    } else {
        Vec2::X
    };
    let (s, c) = og_core::math::sin_cos(a);
    let spot = h * circle.distance + Vec2::new(c, s) * circle.radius;
    (spot.normalize_or_zero() * speed, a)
}

/// A push away from neighbours closer than `radius`, inversely
/// proportional to the distance; `others` are their positions.
pub fn separation(
    pos: Vec2,
    others: impl IntoIterator<Item = Vec2>,
    radius: f32,
    speed: f32,
) -> Vec2 {
    let mut push = Vec2::ZERO;
    let mut n = 0;
    for o in others {
        let away = pos - o;
        let d = away.length();
        if d < radius && d > 1e-5 {
            push += away / d * (1.0 - d / radius);
            n += 1;
        } else if d <= 1e-5 {
            // Exactly on top of each other: split along x, by order.
            push += Vec2::X * if n % 2 == 0 { 1.0 } else { -1.0 };
            n += 1;
        }
    }
    if n == 0 {
        Vec2::ZERO
    } else {
        push.clamp_length_max(1.0) * speed
    }
}

/// What an obstacle probe found: distance along the probe and the surface
/// normal on the plane.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Feeler {
    pub distance: f32,
    pub normal: Vec2,
}

/// Steers away from the nearest hit of three feelers (ahead, and 30° to
/// either side, `probe` returning a hit for a direction): along the wall,
/// stronger the nearer it is.
pub fn avoid(
    heading: Vec2,
    reach: f32,
    speed: f32,
    mut probe: impl FnMut(Vec2) -> Option<Feeler>,
) -> Vec2 {
    let h = if heading.length_squared() > 1e-8 {
        heading.normalize()
    } else {
        return Vec2::ZERO;
    };
    let (s, c) = og_core::math::sin_cos(std::f32::consts::FRAC_PI_6);
    let left = Vec2::new(h.x * c - h.y * s, h.x * s + h.y * c);
    let right = Vec2::new(h.x * c + h.y * s, -h.x * s + h.y * c);
    let mut best: Option<Feeler> = None;
    for dir in [h, left, right] {
        if let Some(f) = probe(dir)
            && f.distance < reach
            && best.is_none_or(|b| f.distance < b.distance)
        {
            best = Some(f);
        }
    }
    match best {
        None => Vec2::ZERO,
        Some(f) => {
            let strength = 1.0 - (f.distance / reach.max(1e-4)).clamp(0.0, 1.0);
            let n = f.normal.normalize_or_zero();
            // Slide: the normal plus the tangent on the side the agent is going.
            let tangent = Vec2::new(-n.y, n.x);
            let side = if tangent.dot(h) >= 0.0 {
                tangent
            } else {
                -tangent
            };
            (n + side).normalize_or_zero() * speed * strength
        }
    }
}

/// Moves `velocity` towards `desired` by at most `accel · dt` and caps it at `max_speed`.
pub fn accelerate(velocity: Vec2, desired: Vec2, accel: f32, dt: f32, max_speed: f32) -> Vec2 {
    let dv = (desired - velocity).clamp_length_max((accel * dt).max(0.0));
    (velocity + dv).clamp_length_max(max_speed.max(0.0))
}
