//! Native invariants checked before a rig enters playback, from either an
//! author declaration/import or a cooked blob. All indices are checked before
//! following any attachment, constraint, channel or child-armature edge.
use super::*;

fn invalid(what: &str) -> Error {
    Error::new(ErrorKind::Parse, format!("2D rig: {what}"))
}

fn unique<'a>(names: impl Iterator<Item = &'a str>) -> bool {
    let mut seen = std::collections::BTreeSet::new();
    names.into_iter().all(|s| !s.is_empty() && seen.insert(s))
}

fn finite(values: &[f32]) -> bool {
    values.iter().all(|v| v.is_finite())
}

fn times(times: impl Iterator<Item = f32>) -> bool {
    let mut previous = 0.0;
    times.into_iter().all(|t| {
        let valid = t.is_finite() && t >= previous;
        previous = t;
        valid
    })
}

fn keys<T>(keys: &[Key<T>], value: impl Fn(&T) -> bool) -> bool {
    times(keys.iter().map(|k| k.time))
        && keys.iter().all(|k| {
            value(&k.value)
                && match &k.ease {
                    Ease::Bezier(c) => finite(c) && check_curve(c).is_ok(),
                    _ => true,
                }
        })
}

impl Rig2dAsset {
    pub fn validate(&self) -> Result<()> {
        if self.texture.is_empty() || !self.unit.is_finite() || self.unit <= 0.0 {
            return Err(invalid(
                "texture is required and unit is finite and above zero",
            ));
        }
        if !unique(self.armatures.iter().map(|a| a.name.as_str())) {
            return Err(invalid("armature names are nonempty and unique"));
        }
        let bodies = std::iter::once((self.body(), self.skins.as_slice())).chain(
            self.armatures
                .iter()
                .map(|a| (a.body(), a.skins.as_slice())),
        );
        for (body, skins) in bodies {
            validate_body(body, skins, self.armatures.len())?;
        }
        self.check_children()
    }
}

fn validate_body(b: Body<'_>, skins: &[Skin2d], armatures: usize) -> Result<()> {
    if b.bones.is_empty()
        || !unique(b.bones.iter().map(|x| x.name.as_str()))
        || !unique(b.slots.iter().map(|x| x.name.as_str()))
        || !unique(b.attachments.iter().map(|x| x.name.as_str()))
        || !unique(b.iks.iter().map(|x| x.name.as_str()))
        || !unique(b.transforms.iter().map(|x| x.name.as_str()))
        || !unique(b.paths.iter().map(|x| x.name.as_str()))
        || !unique(b.clips.iter().map(|x| x.name.as_str()))
        || !unique(skins.iter().map(|x| x.name.as_str()))
    {
        return Err(invalid(
            "each armature needs bones and nonempty unique names",
        ));
    }
    let bone = |i: usize| i < b.bones.len();
    let slot = |i: usize| i < b.slots.len();
    let attachment = |i: usize| i < b.attachments.len();
    for (i, x) in b.bones.iter().enumerate() {
        if x.parent.is_some_and(|p| p >= i)
            || !finite(&[
                x.x, x.y, x.rotation, x.skew, x.scale[0], x.scale[1], x.length,
            ])
        {
            return Err(invalid(
                "bones are parent-first with finite setup transforms",
            ));
        }
    }
    let belongs =
        |a: Option<usize>, s: usize| a.is_none_or(|a| attachment(a) && b.attachments[a].slot == s);
    for (i, x) in b.slots.iter().enumerate() {
        if !bone(x.bone) || !belongs(x.attachment, i) || !finite(&x.color) || !finite(&x.offset) {
            return Err(invalid("invalid slot bone, setup attachment or colour"));
        }
    }
    let influences = |values: &[Influence]| {
        !values.is_empty()
            && values
                .iter()
                .all(|v| bone(v.bone) && finite(&[v.x, v.y, v.weight]) && v.weight >= 0.0)
            && (values.iter().map(|v| v.weight).sum::<f32>() - 1.0).abs() <= 1e-3
    };
    for x in b.attachments {
        let valid = slot(x.slot)
            && match &x.kind {
                AttachmentKind::Region {
                    rect,
                    at,
                    rotation,
                    skew,
                    size,
                } => finite(rect) && finite(at) && finite(&[*rotation, *skew]) && finite(size),
                AttachmentKind::Mesh {
                    vertices,
                    triangles,
                } => {
                    !triangles.is_empty()
                        && triangles.len().is_multiple_of(3)
                        && triangles.iter().all(|i| (*i as usize) < vertices.len())
                        && vertices
                            .iter()
                            .all(|v| finite(&v.uv) && influences(&v.influences))
                }
                AttachmentKind::Box { polygon } => {
                    polygon.len() >= 3 && polygon.iter().all(|p| finite(p))
                }
                AttachmentKind::Path {
                    points,
                    closed,
                    constant,
                    lengths,
                } => {
                    points.len() >= 6
                        && points.len().is_multiple_of(3)
                        && points.iter().all(|p| influences(p))
                        && finite(lengths)
                        && (*constant || lengths.len() == points.len() / 3 - usize::from(!closed))
                }
                AttachmentKind::Child {
                    armature,
                    at,
                    rotation,
                    skew,
                    scale,
                    ..
                } => {
                    *armature < armatures
                        && finite(at)
                        && finite(&[*rotation, *skew])
                        && finite(scale)
                }
            };
        if !valid {
            return Err(invalid(
                "invalid attachment slot, geometry, influence or child armature",
            ));
        }
    }
    for x in b.iks {
        if x.bones.is_empty()
            || !x.bones.iter().copied().all(bone)
            || !bone(x.target)
            || x.bones
                .windows(2)
                .any(|w| b.bones[w[1]].parent != Some(w[0]))
            || !(0.0..=1.0).contains(&x.weight)
        {
            return Err(invalid(
                "IK needs a parent-child chain, target bone and weight in 0..1",
            ));
        }
    }
    for x in b.transforms {
        if x.bones.is_empty()
            || !x.bones.iter().copied().all(bone)
            || !bone(x.target)
            || x.bones.contains(&x.target)
            || !finite(&[x.rotation, x.shear])
            || !finite(&x.at)
            || !finite(&x.scale)
            || !x.mix.iter().all(|v| (0.0..=1.0).contains(v))
        {
            return Err(invalid(
                "invalid transform constraint bones, target, offsets or mixes",
            ));
        }
    }
    for x in b.paths {
        if x.bones.is_empty()
            || !x.bones.iter().copied().all(bone)
            || !attachment(x.path)
            || !matches!(b.attachments[x.path].kind, AttachmentKind::Path { .. })
            || !finite(&[x.position, x.spacing, x.rotation])
            || !x.mix.iter().all(|v| (0.0..=1.0).contains(v))
        {
            return Err(invalid(
                "invalid path constraint bones, path, offsets or mixes",
            ));
        }
    }
    for skin in skins {
        if skin.name == "default"
            || skin
                .swaps
                .iter()
                .any(|(a, to)| !attachment(*a) || !belongs(*to, b.attachments[*a].slot))
        {
            return Err(invalid(
                "skins have distinct names and swaps stay in their slot",
            ));
        }
    }
    for c in b.clips {
        if !c.duration.is_finite() || c.duration <= 0.0 {
            return Err(invalid("a clip's duration is finite and above zero"));
        }
        for x in &c.bones {
            if !bone(x.bone)
                || !keys(&x.rotate, |r| r.degrees.is_finite())
                || !keys(&x.skew, |v| v.is_finite())
                || !keys(&x.translate, |v| finite(v))
                || !keys(&x.scale, |v| finite(v))
            {
                return Err(invalid(
                    "invalid bone channel index, key time, ease or value",
                ));
            }
        }
        for x in &c.slots {
            if !slot(x.slot)
                || !times(x.attachment.iter().map(|k| k.0))
                || !x.attachment.iter().all(|(_, a)| belongs(*a, x.slot))
                || !keys(&x.color, |v| finite(v))
                || !keys(&x.offset, |v| finite(v))
            {
                return Err(invalid(
                    "invalid slot channel index, attachment, key time, ease or value",
                ));
            }
        }
        for x in &c.deforms {
            if !attachment(x.attachment)
                || b.attachments[x.attachment].influences() == 0
                || !keys(&x.keys, |v| {
                    v.len() == 2 * b.attachments[x.attachment].influences() && finite(v)
                })
            {
                return Err(invalid(
                    "deform keys need x/y offsets for every mesh or path influence",
                ));
            }
        }
        if !times(c.order.iter().map(|k| k.0))
            || c.order.iter().any(|(_, order)| {
                let mut seen = order.clone();
                seen.sort_unstable();
                !order.is_empty() && seen != (0..b.slots.len()).collect::<Vec<_>>()
            })
        {
            return Err(invalid(
                "draw-order keys name every slot once, or none for setup order",
            ));
        }
        for x in &c.iks {
            if x.ik >= b.iks.len() || !keys(&x.keys, |v| (0.0..=1.0).contains(&v.weight)) {
                return Err(invalid(
                    "invalid IK channel index, key time, ease or weight",
                ));
            }
        }
        if !times(c.events.iter().map(|k| k.0)) || c.events.iter().any(|(_, n)| n.len() > 63) {
            return Err(invalid(
                "events are in time order with names at most 63 bytes",
            ));
        }
    }
    Ok(())
}
