//! The template's bones through the joints the art shows: each part's
//! joint where it overlaps its parent near its joint end (else at that
//! end, when it touches the parent), each bone from its joint to the next
//! part's joint (else across the part), regions placed so the setup pose
//! draws the art exactly as it was drawn.

use super::mesh::{self, Bend};
use super::parts::{Art, Part};
use super::template::{End, Shape, Spec, Template};
use og_anim::{
    Affine2, Attachment2d, AttachmentKind, Bone2d, Ik2d, Rig2dAsset, SheetFilter, Slot2d,
    TransformConstraint2d,
};

type P = [f32; 2];

fn sub(a: P, b: P) -> P {
    [a[0] - b[0], a[1] - b[1]]
}
fn len(a: P) -> f32 {
    a[0].hypot(a[1])
}

fn centroid(points: impl Iterator<Item = (i32, i32)>) -> Option<P> {
    let (mut sx, mut sy, mut n) = (0f64, 0f64, 0u64);
    for (x, y) in points {
        sx += x as f64 + 0.5;
        sy += y as f64 + 0.5;
        n += 1;
    }
    (n > 0).then(|| [(sx / n as f64) as f32, (sy / n as f64) as f32])
}

/// Whether a canvas pixel of `p` lies in its joint-end band for `end`.
fn in_band(p: &Part, end: End, y: i32) -> bool {
    let band = (p.height as f32 * 0.35).max(p.width.min(p.height) as f32 * 0.5);
    match end {
        End::Top => (y - p.top) as f32 <= band,
        End::Bottom => (p.top + p.height as i32 - 1 - y) as f32 <= band,
        End::Centre | End::Near => true,
    }
}

/// The nearest solid pixel of `p` to `at`, and its distance.
fn nearest(p: &Part, at: P) -> Option<(P, f32)> {
    p.solid()
        .map(|(x, y)| {
            let q = [x as f32 + 0.5, y as f32 + 0.5];
            (q, len(sub(q, at)))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
}

/// Where `p` joins `parent`, canvas pixels: the centroid of their overlap
/// in `p`'s joint-end band, else `p`'s end by convention when it touches
/// `parent`.
fn joint(p: &Part, spec: &Spec, parent: &Part, art: &Art) -> Result<P, String> {
    let overlap = p
        .solid()
        .filter(|&(x, y)| in_band(p, spec.end, y) && parent.alpha(x, y) >= 128);
    if let Some(c) = centroid(overlap) {
        return Ok(c);
    }
    let end = (p.width.min(p.height) as f32 * 0.5).max(1.0);
    let at = match spec.end {
        End::Top => centroid(p.solid().filter(|&(_, y)| ((y - p.top) as f32) < end)),
        End::Bottom => centroid(
            p.solid()
                .filter(|&(_, y)| ((p.top + p.height as i32 - 1 - y) as f32) < end),
        ),
        End::Centre => centroid(p.solid()),
        End::Near => {
            let c = centroid(parent.solid()).unwrap_or([0.0, 0.0]);
            nearest(p, c).map(|n| n.0)
        }
    }
    .ok_or_else(|| {
        format!(
            "{}: `{}` has no solid pixels (alpha at least half)",
            p.source, p.name
        )
    })?;
    let gap = nearest(parent, at).map_or(f32::INFINITY, |n| n.1);
    let tolerance = (art.drawn_height() * 0.02).max(3.0);
    if gap > tolerance {
        return Err(format!(
            "{}: `{}` does not touch `{}` (its joint end is {gap:.0} pixels away); draw it overlapping `{}` where they join, or give its joint with `pivot {} <x> <y>`",
            p.source, p.name, parent.name, parent.name, p.name
        ));
    }
    Ok(at)
}

/// Whether a hip, knee and ankle are in a line (within about a degree).
fn straight(h: P, k: P, a: P) -> bool {
    let (t, s) = (sub(k, h), sub(a, k));
    (t[0] * s[1] - t[1] * s[0]).abs() <= 0.02 * len(t) * len(s)
}

struct Node<'a> {
    spec: &'a Spec,
    part: &'a Part,
    parent: Option<usize>,
    joint: P,
}

fn normal(deg: f32) -> f32 {
    let d = deg % 360.0;
    if d > 180.0 {
        d - 360.0
    } else if d <= -180.0 {
        d + 360.0
    } else {
        d
    }
}

/// The bones of a part laid out in the rig's space (+y up): each bone's
/// origin and angle (degrees) and length.
fn layout(n: &Node, nodes: &[Node], to_rig: &dyn Fn(P) -> P) -> Vec<(P, f32, f32)> {
    let origin = to_rig(n.joint);
    let solid: Vec<P> = n
        .part
        .solid()
        .map(|(x, y)| to_rig([x as f32 + 0.5, y as f32 + 0.5]))
        .collect();
    let tip = n.spec.tip.iter().find_map(|t| {
        nodes
            .iter()
            .find(|m| m.spec.name == *t)
            .map(|m| to_rig(m.joint))
    });
    let (dir, length) = match (tip, n.spec.shape) {
        (Some(t), _) if len(sub(t, origin)) >= 1.0 => (sub(t, origin), len(sub(t, origin))),
        (_, Shape::Chain(_)) => {
            let far = solid
                .iter()
                .copied()
                .max_by(|a, b| len(sub(*a, origin)).total_cmp(&len(sub(*b, origin))))
                .unwrap_or(origin);
            (sub(far, origin), len(sub(far, origin)))
        }
        _ => {
            let c = centroid(n.part.solid()).map(to_rig).unwrap_or(origin);
            let d = sub(c, origin);
            let d = if len(d) >= 1.0 {
                d
            } else {
                match n.spec.end {
                    End::Top => [0.0, -1.0],
                    _ => [0.0, 1.0],
                }
            };
            let unit = [d[0] / len(d), d[1] / len(d)];
            let reach = solid
                .iter()
                .map(|q| sub(*q, origin))
                .map(|q| q[0] * unit[0] + q[1] * unit[1])
                .fold(1.0f32, f32::max);
            (d, reach)
        }
    };
    let angle = dir[1].atan2(dir[0]).to_degrees();
    match n.spec.shape {
        Shape::Chain(k) => {
            let unit = [dir[0] / len(dir).max(1e-6), dir[1] / len(dir).max(1e-6)];
            let step = (length / k as f32).max(1.0);
            (0..k)
                .map(|i| {
                    let o = [
                        origin[0] + unit[0] * step * i as f32,
                        origin[1] + unit[1] * step * i as f32,
                    ];
                    (o, angle, step)
                })
                .collect()
        }
        _ => vec![(origin, angle, length.max(1.0))],
    }
}

/// The canonical rig of `art` on `template` (texture, size and unit left
/// for the caller): `rects` are the parts' atlas rectangles.
pub fn rig(
    art: &Art,
    template: &Template,
    pivots: &[(String, P)],
    rects: &[[u32; 4]],
) -> Result<Rig2dAsset, String> {
    // Every part in the template's order (parents first), with its joint.
    let mut nodes: Vec<Node> = Vec::new();
    for spec in template.parts {
        let Some(part) = art.part(spec.name) else {
            continue;
        };
        let parent = spec
            .parents
            .iter()
            .find_map(|p| nodes.iter().position(|n| n.spec.name == *p));
        if parent.is_none() && !spec.parents.is_empty() {
            return Err(format!(
                "{}: `{}` hangs from `{}`, which the art does not have; add it",
                part.source,
                spec.name,
                spec.parents.join("` or `")
            ));
        }
        let joint = match pivots.iter().find(|(n, _)| n == spec.name) {
            Some((_, at)) => *at,
            None => match parent {
                Some(i) => joint(part, spec, nodes[i].part, art)?,
                None => centroid(part.solid()).ok_or_else(|| {
                    format!("{}: `{}` has no solid pixels", part.source, part.name)
                })?,
            },
        };
        nodes.push(Node {
            spec,
            part,
            parent,
            joint,
        });
    }
    // A knee straight in the art goes onto the line from the hip to the
    // ankle, so the leg's IK (which bends a straight knee forwards) keeps
    // the setup pose as drawn.
    for leg in template.legs {
        let idx = |name: &str| nodes.iter().position(|n| n.spec.name == name);
        let (Some(u), Some(l), Some(f)) = (idx(leg.upper), idx(leg.lower), idx(leg.end)) else {
            continue;
        };
        let (h, k, a) = (nodes[u].joint, nodes[l].joint, nodes[f].joint);
        if straight(h, k, a) {
            let d = sub(a, h);
            let t = ((k[0] - h[0]) * d[0] + (k[1] - h[1]) * d[1])
                / (d[0] * d[0] + d[1] * d[1]).max(1e-6);
            nodes[l].joint = [h[0] + d[0] * t, h[1] + d[1] * t];
        }
    }
    // Upright: the head above the hips, the feet below them.
    let at = |name: &str| nodes.iter().find(|n| n.spec.name == name).map(|n| n.joint);
    if let (Some(head), Some(hips)) = (at("head"), at("pelvis").or(at("body"))) {
        if head[1] >= hips[1] && template.name.starts_with("humanoid") {
            let p = art.part("head").expect("head");
            return Err(format!(
                "{}: `head` is drawn below `pelvis`; the {} template stands upright, head up: check which layers are named head and pelvis",
                p.source, template.name
            ));
        }
        for leg in template.legs {
            if let Some(foot) = at(leg.end)
                && foot[1] <= hips[1]
            {
                let p = art.part(leg.end).expect("foot");
                return Err(format!(
                    "{}: `{}` is drawn above the hips; the {} template stands on its feet: check the layer's name",
                    p.source, leg.end, template.name
                ));
            }
        }
    }
    // The root: on the ground (the lowest drawn pixel), under the hips.
    let ground = art
        .parts
        .iter()
        .map(|p| (p.top + p.height as i32) as f32)
        .fold(f32::MIN, f32::max);
    let rx = nodes.first().map_or(art.width as f32 / 2.0, |n| n.joint[0]);
    let to_rig = move |p: P| [p[0] - rx, ground - p[1]];

    let mut bones = vec![Bone2d {
        name: "root".into(),
        parent: None,
        x: 0.0,
        y: 0.0,
        rotation: 0.0,
        skew: 0.0,
        scale: [1.0, 1.0],
        length: 0.0,
    }];
    // World (origin, angle) per bone, to make each child's local transform.
    let mut world: Vec<(P, f32)> = vec![([0.0, 0.0], 0.0)];
    let mut first_bone = vec![0usize; nodes.len()];
    let mut laid: Vec<Vec<(P, f32, f32)>> = Vec::with_capacity(nodes.len());
    for (i, n) in nodes.iter().enumerate() {
        let mut parent = n.parent.map_or(0, |p| first_bone[p]);
        // Each part hangs from its parent's first bone; a chain's bones
        // each from the one before.
        laid.push(layout(n, &nodes, &to_rig));
        for (k, &(o, a, l)) in laid[i].iter().enumerate() {
            let (po, pa) = world[parent];
            let d = sub(o, po);
            let r = (-pa).to_radians();
            let local = [
                d[0] * r.cos() - d[1] * r.sin(),
                d[0] * r.sin() + d[1] * r.cos(),
            ];
            bones.push(Bone2d {
                name: if k == 0 {
                    n.spec.name.to_string()
                } else {
                    format!("{}-{}", n.spec.name, k + 1)
                },
                parent: Some(parent),
                x: local[0],
                y: local[1],
                rotation: normal(a - pa),
                skew: 0.0,
                scale: [1.0, 1.0],
                length: l,
            });
            world.push((o, a));
            if k == 0 {
                first_bone[i] = bones.len() - 1;
            }
            parent = bones.len() - 1;
        }
    }
    // Two-bone IK on the legs, each to a target at its ankle.
    let mut iks = Vec::new();
    for leg in template.legs {
        let idx = |name: &str| nodes.iter().position(|n| n.spec.name == name);
        let (Some(u), Some(l), Some(f)) = (idx(leg.upper), idx(leg.lower), idx(leg.end)) else {
            continue;
        };
        let (h, k, a) = (
            to_rig(nodes[u].joint),
            to_rig(nodes[l].joint),
            to_rig(nodes[f].joint),
        );
        let (t, s) = (sub(k, h), sub(a, k));
        let cross = t[0] * s[1] - t[1] * s[0];
        bones.push(Bone2d {
            name: format!("ik-{}", leg.end),
            parent: Some(0),
            x: a[0],
            y: a[1],
            rotation: 0.0,
            skew: 0.0,
            scale: [1.0, 1.0],
            length: 0.0,
        });
        world.push((a, 0.0));
        iks.push(Ik2d {
            name: format!("ik-{}", leg.end),
            bones: vec![first_bone[u], first_bone[l]],
            target: bones.len() - 1,
            // As the art bends it; a knee straight in the art bends the
            // way the template says.
            counterclockwise: if straight(h, k, a) {
                leg.counterclockwise
            } else {
                cross < 0.0
            },
            weight: 1.0,
        });
    }
    // A joint bone where a limb continues into its next part (an elbow, a
    // knee, a wrist, the waist, the neck): at the joint, turned half-way
    // between the two by a transform constraint, so the joint's pixels (a
    // knee cap, an elbow) turn rigidly at half the angle instead of being
    // pinched between the two bones. Where a part only hangs from the
    // trunk (a shoulder, a hip, a cape, a tail) it turns on a hinge: those
    // joints turn far (an arm raised overhead) and their round ends hide
    // the hinge, where a blend would fold the part over itself.
    let mut transforms = Vec::new();
    let mut joint_bone: Vec<Option<usize>> = vec![None; nodes.len()];
    for (i, n) in nodes.iter().enumerate() {
        let Some(p) = n.parent else { continue };
        let bends = |s: Shape| matches!(s, Shape::Limb | Shape::Chain(_));
        if !bends(n.spec.shape) && !bends(nodes[p].spec.shape) {
            continue;
        }
        let continues = nodes[p]
            .spec
            .tip
            .iter()
            .find(|t| nodes.iter().any(|m| m.spec.name == **t))
            == Some(&n.spec.name);
        if !continues {
            continue;
        }
        let (pb, cb) = (first_bone[p], first_bone[i]);
        let (po, pa) = world[pb];
        let (o, a) = world[cb];
        let d = sub(o, po);
        let r = (-pa).to_radians();
        bones.push(Bone2d {
            name: format!("joint-{}", n.spec.name),
            parent: Some(pb),
            x: d[0] * r.cos() - d[1] * r.sin(),
            y: d[0] * r.sin() + d[1] * r.cos(),
            rotation: 0.0,
            skew: 0.0,
            scale: [1.0, 1.0],
            length: 0.0,
        });
        world.push((o, pa + normal(a - pa) / 2.0));
        joint_bone[i] = Some(bones.len() - 1);
        transforms.push(TransformConstraint2d {
            name: format!("joint-{}", n.spec.name),
            bones: vec![bones.len() - 1],
            target: cb,
            rotation: 0.0,
            at: [0.0, 0.0],
            scale: [0.0, 0.0],
            shear: 0.0,
            mix: [0.5, 0.0, 0.0, 0.0],
        });
    }
    // A bone the clips scale (a creature's body squashing, a thigh
    // foreshortened) would scale the rigid parts below it too: each keeps
    // the root's axes (unit length, square), so an eye stays round and a
    // foot keeps its shape.
    let below = |mut n: usize| {
        while let Some(p) = nodes[n].parent {
            if template.scaled.contains(&nodes[p].spec.name) {
                return true;
            }
            n = p;
        }
        false
    };
    for (i, n) in nodes.iter().enumerate() {
        if n.spec.shape == Shape::Rigid && below(i) {
            transforms.push(TransformConstraint2d {
                name: format!("rigid-{}", n.spec.name),
                bones: vec![first_bone[i]],
                target: 0,
                rotation: 0.0,
                at: [0.0, 0.0],
                scale: [0.0, 0.0],
                shear: 0.0,
                mix: [0.0, 0.0, 1.0, 1.0],
            });
        }
    }
    // The bones' world transforms in the setup pose, as og-anim poses them
    // (a joint bone half-way, as its constraint turns it).
    let mut affine: Vec<Affine2> = Vec::with_capacity(bones.len());
    for (i, b) in bones.iter().enumerate() {
        let local = Affine2::trs(b.x, b.y, b.rotation, b.scale[0], b.scale[1]);
        let composed = match b.parent {
            Some(p) => affine[p].mul(&local),
            None => local,
        };
        affine.push(if joint_bone.contains(&Some(i)) {
            let (o, a) = world[i];
            Affine2::trs(o[0], o[1], a, 1.0, 1.0)
        } else {
            composed
        });
    }
    // One slot and one region per part, back to front.
    let mut slots = Vec::new();
    let mut attachments = Vec::new();
    for (i, (p, r)) in art.parts.iter().zip(rects).enumerate() {
        let n = nodes
            .iter()
            .position(|n| std::ptr::eq(n.part, p))
            .expect("every part is a node");
        let bone = first_bone[n];
        let centre = to_rig([
            p.left as f32 + p.width as f32 / 2.0,
            p.top as f32 + p.height as f32 / 2.0,
        ]);
        let at = affine[bone].inverse().apply_to(centre);
        let node = &nodes[n];
        let kind = match node.spec.shape {
            // Limbs and chains bend: a mesh weighted along their bones.
            Shape::Limb | Shape::Chain(_) => {
                let (origin, angle, step) = laid[n][0];
                let rad = angle.to_radians();
                let child = node.spec.tip.iter().find_map(|t| {
                    nodes
                        .iter()
                        .position(|m| m.spec.name == *t && m.parent == Some(n))
                });
                let bend = Bend {
                    bones: (bone..bone + laid[n].len()).collect(),
                    origin,
                    dir: [rad.cos(), rad.sin()],
                    step,
                    start: node
                        .parent
                        .zip(joint_bone[n])
                        .map(|(p, j)| (j, first_bone[p])),
                    end: child.and_then(|c| joint_bone[c].map(|j| (j, first_bone[c]))),
                };
                mesh::mesh(p, *r, &bend, &to_rig, &affine)
            }
            Shape::Rigid => AttachmentKind::Region {
                rect: r.map(|v| v as f32),
                at,
                rotation: normal(-world[bone].1),
                skew: 0.0,
                size: [p.width as f32, p.height as f32],
            },
        };
        slots.push(Slot2d {
            name: p.name.clone(),
            bone,
            attachment: Some(i),
            color: [1.0; 4],
            offset: [0.0; 4],
        });
        attachments.push(Attachment2d {
            name: p.name.clone(),
            slot: i,
            kind,
        });
    }
    Ok(Rig2dAsset {
        texture: String::new(),
        size: [0, 0],
        filter: SheetFilter::Linear,
        unit: 1.0,
        bones,
        slots,
        attachments,
        iks,
        transforms,
        paths: Vec::new(),
        clips: Vec::new(),
        skins: Vec::new(),
        armatures: Vec::new(),
    })
}
