//! The push a blast and a strike give the loose bodies.

use super::*;

/// The explosions' push on every loose body in reach: a body made this
/// tick takes it with its starting velocity, another on its og.Velocity.
#[allow(clippy::too_many_arguments)]
pub(super) fn push_blasted_bodies(
    ctx: &mut SystemContext<'_>,
    l: &Ls,
    st: &State,
    assets: &Assets,
    settings: &BTreeMap<u64, Settings>,
    poses: &Poses,
    explosions: &[Explosion],
    out: &mut Broken,
) -> og_core::Result<()> {
    let made = &mut out.made;
    for &(point, inner, radius, impulse) in explosions {
        if impulse <= 0.0 {
            continue;
        }
        let falloff = Damage {
            point,
            min_radius: inner,
            max_radius: radius,
            amount: impulse,
        };
        for (&k, p) in &st.props {
            let a = &assets[&p.asset];
            let scale = uniform(pose_of_frame(&p.frame).2);
            let density = settings.get(&k).map_or(1000.0, |s| s.mat.density);
            let bodies = loose_bodies(ctx, l, p, a, scale, density, &poses[&k], made, point);
            for (b, (m, at)) in bodies {
                if m <= 0.0 {
                    continue;
                }
                let first = (0..p.entities.len())
                    .find(|&c| p.entities[c] == b || (c < a.support as usize && p.body[c] == b))
                    .unwrap_or(0);
                let Some((dv, dw)) = blast_push(&falloff, impulse, at, m, density, first as u64)
                else {
                    continue;
                };
                match made.get_mut(&b) {
                    Some(v) => {
                        v.0 += dv;
                        v.1 += dw;
                    }
                    None => {
                        let e = Entity::from_bits(b);
                        if let Ok(vb) = ctx.get_bytes(e, l.velocity.0) {
                            let mut v = l.velocity.1.decode(&vb);
                            let now = record::Velocity::from_json(&v).map_err(invalid)?;
                            v["linear"] = jv(now.linear + dv);
                            v["angular"] = jv(now.angular + dw);
                            let vel = l.velocity.1.enc(Some(&vb), Some(&v))?;
                            ctx.set_bytes(e, l.velocity.0, &vel)?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// A hit's or a blow's push on the pieces it broke loose: the impulse
/// past the threshold (momentum the striker gave up breaking through), or
/// the blow's, shared by falloff over them, none driven faster than the
/// striker closed.
#[allow(clippy::too_many_arguments)]
pub(super) fn push_struck(
    ctx: &SystemContext<'_>,
    l: &Ls,
    st: &State,
    assets: &Assets,
    settings: &BTreeMap<u64, Settings>,
    poses: &Poses,
    strikes: &[Strike],
    out: &mut Broken,
) {
    for &(k, point, dir, radius, amount, speed) in strikes {
        let (Some(p), Some(s)) = (st.props.get(&k), settings.get(&k)) else {
            continue;
        };
        let a = &assets[&p.asset];
        let scale = uniform(pose_of_frame(&p.frame).2);
        let falloff = Damage {
            point,
            min_radius: radius / 3.0,
            max_radius: radius,
            amount: 1.0,
        };
        let made = &out.made;
        let freed: Vec<(u64, f32, f32)> =
            loose_bodies(ctx, l, p, a, scale, s.mat.density, &poses[&k], made, point)
                .into_iter()
                .filter(|(b, _)| made.contains_key(b))
                .map(|(b, (m, at))| (b, m, falloff.at(at)))
                .filter(|&(_, m, w)| m > 0.0 && w > 0.0)
                .collect();
        // One velocity for the whole freed mass, scaled by falloff: the
        // momentum is conserved and the lightest piece is not flung.
        let weighed: f32 = freed.iter().map(|&(_, m, w)| m * w).sum();
        // The fine pieces it broke off this tick move with them.
        let pieces: f32 = out
            .debris
            .iter()
            .filter(|(d, _)| d.prop == k)
            .map(|(d, m)| m * falloff.at(Vec3::new(d.pose[0], d.pose[1], d.pose[2])))
            .sum();
        let weighed = weighed + pieces;
        for (b, _, w) in freed {
            let dv = (amount * w / weighed).min(speed);
            if let Some(v) = out.made.get_mut(&b) {
                v.0 += dir * dv;
            }
        }
        for (d, _) in out.debris.iter_mut().filter(|(d, _)| d.prop == k) {
            let w = falloff.at(Vec3::new(d.pose[0], d.pose[1], d.pose[2]));
            if w > 0.0 && weighed > 0.0 {
                let dv = (amount * w / weighed).min(speed);
                d.velocity = (Vec3::from(d.velocity) + dir * dv).to_array();
            }
        }
        for (c, _) in out.crumbs.iter_mut().filter(|(c, _)| c.prop == k) {
            let w = falloff.at(Vec3::from(c.point));
            if w > 0.0 && weighed > 0.0 {
                let dv = (amount * w / weighed).min(speed);
                c.velocity = (Vec3::from(c.velocity) + dir * dv).to_array();
            }
        }
    }
}
