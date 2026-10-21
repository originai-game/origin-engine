//! The kinematic move both controllers make, written once and compiled for
//! each profile (like `ext.rs`): og.CharacterController (3D, and top-down
//! or side-on 2D) and og.PlatformerController (2D) move their shape
//! themselves rather than through rapier's KinematicCharacterController.
//!
//! That controller casts every move with its skin as parry's target
//! distance, and parry's support-map cast with a target distance is off by
//! up to the distance (a 0.7 × 1.25 box 2 cm over a 74 m floor: 1.98 cm of
//! travel, not 1 cm), while a cast that starts within it falls back on a
//! contact query that returns nothing at some places (x 26 of a floor from
//! 8 to 82). So a run grazing the floor met it at random with a tilted
//! normal and lost its step: in 2D the slope rule (no horizontal tangent)
//! took the whole step away (FINDINGS #16 of examples/hollowbell), in 3D a
//! capsule running over adjacent boxes or a triangle mesh dropped to under
//! 0.4 m/s for a tick every second or so; and a shape put exactly on a
//! floor fell into it (11 mm for the 2D box, #15; 9 cm for a 3D capsule)
//! and stuck, or crept up a tenth of a millimetre a tick. Exact casts (no
//! target distance) from at least a millimetre away measure to 2e-5 m, so
//! the move keeps the skin itself: rising, up then across; otherwise lifted
//! `LIFT` skins, across, then down onto the ground. A step is the same move
//! from `step_height` up, taken when the low one was blocked and the high
//! one lands on walkable ground no higher than the step height.

use super::{flat, rp, unv, v};
use og_core::glam::Vec3;

/// How many skins the move lifts a shape that is not rising before it
/// moves across, so the drop back onto the ground starts clear of it
/// (and steps over seams lower than a skin).
pub(crate) const LIFT: f32 = 2.0;

/// How far above a contact on an edge the probe for the surface under it
/// starts, and how far inside the edge (the edge's own top face).
const PROBE_UP: f32 = 0.05;
const PROBE_IN: f32 = 0.002;

/// What a move did.
pub(crate) struct Travel {
    pub translation: Vec3,
    /// Standing on walkable ground after it: the collider and the ground's
    /// normal.
    pub ground: Option<(rp::ColliderHandle, Vec3)>,
    /// Slid down a slope steeper than the walkable one.
    pub sliding: bool,
    /// What it hit on the way: the collider and its surface normal.
    pub hits: Vec<(rp::ColliderHandle, Vec3)>,
}

/// A cast's first contact.
struct Hit {
    handle: rp::ColliderHandle,
    toi: f32,
    /// The surface normal at the contact, out of the collider met.
    normal: Vec3,
    /// The contact point on the collider met (world).
    point: Vec3,
}

/// The shape and the rules it moves by.
pub(crate) struct Mover<'a> {
    pub qp: rp::QueryPipeline<'a>,
    pub shape: &'a dyn rp::Shape,
    pub rot: rp::Rotation,
    pub skin: f32,
    /// The cosine of the steepest walkable slope.
    pub walkable: f32,
    /// The highest step climbed (0: none).
    pub step_height: f32,
}

/// Where a ground move ended.
struct Landing {
    end: Vec3,
    ground: Option<(rp::ColliderHandle, Vec3)>,
    /// The height of the ground where it stands on it.
    top: f32,
    sliding: bool,
}

impl Mover<'_> {
    /// The first collider the shape at `from` meets moving `dist` along the
    /// unit `dir`. Exact (no target distance), and surfaces the shape
    /// touches and moves away from are passed.
    fn cast(&self, from: Vec3, dir: Vec3, dist: f32) -> Option<Hit> {
        let options = super::ShapeCastOptions {
            max_time_of_impact: dist,
            target_distance: 0.0,
            stop_at_penetration: false,
            compute_impact_geometry_on_penetration: true,
        };
        self.qp
            .cast_shape(
                &rp::Pose::from_parts(v(from), self.rot),
                v(dir),
                self.shape,
                options,
            )
            .map(|(handle, hit)| Hit {
                handle,
                toi: hit.time_of_impact,
                normal: unv(hit.normal1),
                point: unv(hit.witness1),
            })
    }

    /// Moves from `pos` by `motion`, `skin` off whatever it meets: up a
    /// walkable slope keeping its speed across, along a wall or a ceiling
    /// never up.
    fn slide(
        &self,
        mut pos: Vec3,
        motion: Vec3,
        hits: &mut Vec<(rp::ColliderHandle, Vec3)>,
    ) -> Vec3 {
        let mut rem = motion;
        for _ in 0..4 {
            let len = rem.length();
            if len < 1e-6 {
                break;
            }
            let dir = rem / len;
            let Some(hit) = self.cast(pos, dir, len + self.skin) else {
                pos += rem;
                break;
            };
            let n = hit.normal;
            let approach = -dir.dot(n);
            if approach <= 1e-4 {
                // Grazing a surface it moves along.
                pos += rem;
                break;
            }
            hits.push((hit.handle, n));
            let step = (hit.toi - self.skin / approach).clamp(0.0, len);
            pos += dir * step;
            let left = dir * (len - step);
            let across = Vec3::new(left.x, 0.0, left.z);
            rem = if n.y >= self.walkable && across != Vec3::ZERO {
                Vec3::new(across.x, -(across.x * n.x + across.z * n.z) / n.y, across.z)
            } else {
                let s = left - n * left.dot(n);
                flat(Vec3::new(s.x, s.y.min(0.0), s.z))
            };
        }
        pos
    }

    /// Moves from `pos` by `motion` in any direction, `skin` off whatever
    /// it meets and sliding along it (no up, no ground: gravityScale 0).
    fn glide(
        &self,
        mut pos: Vec3,
        motion: Vec3,
        hits: &mut Vec<(rp::ColliderHandle, Vec3)>,
    ) -> Vec3 {
        let mut rem = motion;
        for _ in 0..4 {
            let len = rem.length();
            if len < 1e-6 {
                break;
            }
            let dir = rem / len;
            let Some(hit) = self.cast(pos, dir, len + self.skin) else {
                pos += rem;
                break;
            };
            let n = hit.normal;
            let approach = -dir.dot(n);
            if approach <= 1e-4 {
                pos += rem;
                break;
            }
            hits.push((hit.handle, n));
            let step = (hit.toi - self.skin / approach).clamp(0.0, len);
            pos += dir * step;
            let left = dir * (len - step);
            rem = flat(left - n * left.dot(n));
        }
        pos
    }

    /// The ground under the shape at `pos` standing a skin over it: its
    /// lowest point less the skin.
    fn bottom(&self, pos: Vec3) -> f32 {
        let b = self
            .shape
            .compute_aabb(&rp::Pose::from_parts(v(pos), self.rot));
        unv(b.mins).y - self.skin
    }

    /// How far the shape at `pos` rises up to `want`, stopping `skin` under
    /// a ceiling.
    fn headroom(&self, pos: Vec3, want: f32) -> f32 {
        match self.cast(pos, Vec3::Y, want + self.skin) {
            Some(hit) if hit.normal.y < -1e-4 => {
                (hit.toi - self.skin / -hit.normal.y).clamp(0.0, want)
            }
            _ => want,
        }
    }

    /// Moves from `pos` up by `rise`, stopping `skin` under a ceiling.
    fn rise(&self, pos: Vec3, rise: f32, hits: &mut Vec<(rp::ColliderHandle, Vec3)>) -> Vec3 {
        match self.cast(pos, Vec3::Y, rise + self.skin) {
            Some(hit) if hit.normal.y < -1e-4 => {
                hits.push((hit.handle, hit.normal));
                pos + Vec3::Y * (hit.toi - self.skin / -hit.normal.y).clamp(0.0, rise)
            }
            _ => pos + Vec3::Y * rise,
        }
    }

    /// The walkable top of an edge a contact at `point` with normal `n`
    /// sits on: a short ray down just inside the edge. A shape resting on
    /// an edge (a step's nose, a ledge) meets it with a normal tilted by its
    /// own curve, not the ground's; a slope answers its own normal. The
    /// top's normal and height.
    fn edge_top(&self, point: Vec3, n: Vec3) -> Option<(Vec3, f32)> {
        let out = Vec3::new(n.x, 0.0, n.z);
        let len = out.length();
        if len < 1e-6 {
            return None;
        }
        let origin = point - out / len * PROBE_IN + Vec3::Y * PROBE_UP;
        let ray = rp::Ray::new(v(origin), v(-Vec3::Y));
        let (_, hit) = self
            .qp
            .cast_ray_and_get_normal(&ray, PROBE_UP + 0.01, true)?;
        let top = unv(hit.normal);
        (top.y >= self.walkable).then_some((top, origin.y - hit.time_of_impact))
    }

    /// Moves from `pos` down by `fall`, or onto walkable ground within
    /// `fall + snap`, `skin` over it; a steeper slope met slides it down.
    fn settle(
        &self,
        pos: Vec3,
        fall: f32,
        snap: f32,
        hits: &mut Vec<(rp::ColliderHandle, Vec3)>,
    ) -> Landing {
        let free = Landing {
            end: pos - Vec3::Y * fall,
            ground: None,
            top: f32::MIN,
            sliding: false,
        };
        let Some(hit) = self.cast(pos, -Vec3::Y, fall + snap + self.skin) else {
            return free;
        };
        let n = hit.normal;
        if n.y <= 1e-4 {
            return free;
        }
        let travel = hit.toi - self.skin / n.y;
        let ground = if n.y >= self.walkable {
            Some((n, hit.point.y))
        } else {
            self.edge_top(hit.point, n)
        };
        if let Some((up, top)) = ground {
            hits.push((hit.handle, n));
            // At most a skin back up: what the lift left too close.
            let travel = travel.max(-self.skin);
            return Landing {
                end: pos - Vec3::Y * travel,
                ground: Some((hit.handle, up)),
                top,
                sliding: false,
            };
        }
        if travel > fall {
            return free;
        }
        hits.push((hit.handle, n));
        let travel = travel.max(0.0);
        let at = pos - Vec3::Y * travel;
        let left = -Vec3::Y * (fall - travel);
        let along = left - n * left.dot(n);
        Landing {
            end: self.slide(at, along, hits),
            ground: None,
            top: f32::MIN,
            sliding: true,
        }
    }

    /// From `centre`, `lift` up, across by `across`, and down by the lift
    /// and `fall` (onto ground within `snap` more).
    fn walk(
        &self,
        centre: Vec3,
        lift: f32,
        across: Vec3,
        fall: f32,
        snap: f32,
        hits: &mut Vec<(rp::ColliderHandle, Vec3)>,
    ) -> Landing {
        let lifted = centre + Vec3::Y * lift;
        let moved = self.slide(lifted, across, hits);
        self.settle(moved, lift + fall, snap, hits)
    }

    /// The whole move from `centre` by `desired`: rising, up then across;
    /// otherwise lifted, across, and down by the lift and the fall (onto
    /// ground within `snap` more). Standing on the ground (`grounded`)
    /// and blocked, it tries the move from `step_height` up and takes it
    /// when that goes further and lands on walkable ground (not a dynamic
    /// body) at most `step_height` over the ground it stood on (a skin
    /// under its lowest point).
    pub fn travel(&self, centre: Vec3, desired: Vec3, snap: f32, grounded: bool) -> Travel {
        let mut hits = Vec::new();
        let across = Vec3::new(desired.x, 0.0, desired.z);
        if desired.y > 0.0 {
            let up = self.rise(centre, desired.y, &mut hits);
            let end = self.slide(up, across, &mut hits);
            return Travel {
                translation: end - centre,
                ground: None,
                sliding: false,
                hits,
            };
        }
        let lift = self.headroom(centre, self.skin * LIFT);
        let mut low = self.walk(centre, lift, across, -desired.y, snap, &mut hits);
        let wanted = across.length();
        if grounded && self.step_height > 0.0 && wanted > 1e-6 {
            let dir = across / wanted;
            let went = (low.end - centre).dot(dir);
            if went < wanted - 1e-4 {
                let high = self.headroom(centre, self.step_height + self.skin);
                if high > lift + 1e-4 {
                    let mut step_hits = Vec::new();
                    let up = self.walk(centre, high, across, -desired.y, snap, &mut step_hits);
                    let rigid = |h: rp::ColliderHandle| {
                        self.qp.colliders[h]
                            .parent()
                            .is_some_and(|b| self.qp.bodies[b].is_dynamic())
                    };
                    if let Some((h, _)) = up.ground
                        && !rigid(h)
                        && (up.end - centre).dot(dir) > went + 1e-4
                        && up.top - self.bottom(centre) <= self.step_height + 1e-3
                    {
                        low = up;
                        hits = step_hits;
                    }
                }
            }
        }
        Travel {
            translation: low.end - centre,
            ground: low.ground,
            sliding: low.sliding,
            hits,
        }
    }

    /// The move from `centre` by `desired` in any direction, sliding along
    /// what it meets (gravityScale 0).
    pub fn travel_free(&self, centre: Vec3, desired: Vec3) -> Travel {
        let mut hits = Vec::new();
        let end = self.glide(centre, desired, &mut hits);
        Travel {
            translation: end - centre,
            ground: None,
            sliding: false,
            hits,
        }
    }
}
