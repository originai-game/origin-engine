//! Canonical rig text writing.

use super::*;

fn ease_text(e: &Ease) -> String {
    match e {
        Ease::Linear => String::new(),
        Ease::Step => " step".into(),
        Ease::Bezier(c) => {
            let mut s = String::from(" curve");
            for v in c {
                s += &format!(" {}", num_text(*v));
            }
            s
        }
    }
}

/// Writes an armature's bones, slots, attachments, constraints and clips.
/// `skin` blocks of `skins`, whose swaps index `attachments`.
pub(super) fn skins_text(skins: &[Skin2d], attachments: &[Attachment2d]) -> String {
    let mut s = String::new();
    for k in skins {
        s += &format!("skin {} {{\n", quote(&k.name));
        for (from, to) in &k.swaps {
            let to = to.map_or("none".to_string(), |t| quote(&attachments[t].name));
            s += &format!("  swap {} {to}\n", quote(&attachments[*from].name));
        }
        s += "}\n";
    }
    s
}

pub(super) fn body_text(rig: Body<'_>, armatures: &[Armature2d]) -> String {
    let n = num_text;
    let mut s = String::new();
    for b in rig.bones {
        s += &format!("bone {} {{\n", quote(&b.name));
        if let Some(p) = b.parent {
            s += &format!("  parent {}\n", quote(&rig.bones[p].name));
        }
        s += &format!(
            "  at {} {}\n  rotation {}\n  skew {}\n",
            n(b.x),
            n(b.y),
            n(b.rotation),
            n(b.skew)
        );
        s += &format!(
            "  scale {} {}\n  length {}\n}}\n",
            n(b.scale[0]),
            n(b.scale[1]),
            n(b.length)
        );
    }
    for sl in rig.slots {
        s += &format!(
            "slot {} {{\n  bone {}\n",
            quote(&sl.name),
            quote(&rig.bones[sl.bone].name)
        );
        if let Some(a) = sl.attachment {
            s += &format!("  attachment {}\n", quote(&rig.attachments[a].name));
        }
        let [r, g, b, a] = sl.color;
        s += &format!("  color {} {} {} {}\n", n(r), n(g), n(b), n(a));
        let [r, g, b, a] = sl.offset;
        s += &format!("  offset {} {} {} {}\n}}\n", n(r), n(g), n(b), n(a));
    }
    for a in rig.attachments {
        let slot = quote(&rig.slots[a.slot].name);
        match &a.kind {
            AttachmentKind::Region {
                rect,
                at,
                rotation,
                skew,
                size,
            } => {
                s += &format!("region {} {{\n  slot {slot}\n", quote(&a.name));
                s += &format!(
                    "  rect {} {} {} {}\n",
                    n(rect[0]),
                    n(rect[1]),
                    n(rect[2]),
                    n(rect[3])
                );
                s += &format!(
                    "  at {} {}\n  rotation {}\n  skew {}\n",
                    n(at[0]),
                    n(at[1]),
                    n(*rotation),
                    n(*skew)
                );
                s += &format!("  size {} {}\n}}\n", n(size[0]), n(size[1]));
            }
            AttachmentKind::Mesh {
                vertices,
                triangles,
            } => {
                s += &format!("mesh {} {{\n  slot {slot}\n", quote(&a.name));
                let bone = rig.slots[a.slot].bone;
                for v in vertices {
                    s += &format!("  vertex {} {}", n(v.uv[0]), n(v.uv[1]));
                    match v.influences.as_slice() {
                        [i] if i.bone == bone && i.weight == 1.0 => {
                            s += &format!(" {} {}", n(i.x), n(i.y))
                        }
                        all => {
                            for i in all {
                                s += &format!(
                                    " {} {} {} {}",
                                    quote(&rig.bones[i.bone].name),
                                    n(i.x),
                                    n(i.y),
                                    n(i.weight)
                                );
                            }
                        }
                    }
                    s += "\n";
                }
                s += "  triangles";
                for t in triangles {
                    s += &format!(" {t}");
                }
                s += "\n}\n";
            }
            AttachmentKind::Path {
                points,
                closed,
                constant,
                lengths,
            } => {
                s += &format!(
                    "path {} {{\n  slot {slot}\n  closed {closed}\n  constant {constant}\n",
                    quote(&a.name)
                );
                if !lengths.is_empty() {
                    s += "  lengths";
                    for l in lengths {
                        s += &format!(" {}", n(*l));
                    }
                    s += "\n";
                }
                let bone = rig.slots[a.slot].bone;
                for p in points {
                    s += "  vertex";
                    match p.as_slice() {
                        [i] if i.bone == bone && i.weight == 1.0 => {
                            s += &format!(" {} {}", n(i.x), n(i.y))
                        }
                        all => {
                            for i in all {
                                s += &format!(
                                    " {} {} {} {}",
                                    quote(&rig.bones[i.bone].name),
                                    n(i.x),
                                    n(i.y),
                                    n(i.weight)
                                );
                            }
                        }
                    }
                    s += "\n";
                }
                s += "}\n";
            }
            AttachmentKind::Box { polygon } => {
                s += &format!("box {} {{\n  slot {slot}\n  polygon", quote(&a.name));
                for p in polygon {
                    s += &format!(" {} {}", n(p[0]), n(p[1]));
                }
                s += "\n}\n";
            }
            AttachmentKind::Child {
                armature,
                clip,
                at,
                rotation,
                skew,
                scale,
            } => {
                s += &format!(
                    "child {} {{\n  slot {slot}\n  armature {}\n",
                    quote(&a.name),
                    quote(&armatures[*armature].name)
                );
                if let Some(c) = clip {
                    s += &format!("  clip {}\n", quote(c));
                }
                s += &format!(
                    "  at {} {}\n  rotation {}\n  skew {}\n  scale {} {}\n}}\n",
                    n(at[0]),
                    n(at[1]),
                    n(*rotation),
                    n(*skew),
                    n(scale[0]),
                    n(scale[1])
                );
            }
        }
    }
    let bend_text = |ccw: bool| if ccw { "counterclockwise" } else { "clockwise" };
    for ik in rig.iks {
        s += &format!("ik {} {{\n  bones", quote(&ik.name));
        for b in &ik.bones {
            s += &format!(" {}", quote(&rig.bones[*b].name));
        }
        s += &format!(
            "\n  target {}\n  bend {}\n  weight {}\n}}\n",
            quote(&rig.bones[ik.target].name),
            bend_text(ik.counterclockwise),
            n(ik.weight)
        );
    }
    let bones_text = |list: &[usize]| -> String {
        list.iter()
            .map(|b| format!(" {}", quote(&rig.bones[*b].name)))
            .collect()
    };
    for t in rig.transforms {
        s += &format!(
            "transform {} {{\n  bones{}\n  target {}\n",
            quote(&t.name),
            bones_text(&t.bones),
            quote(&rig.bones[t.target].name)
        );
        s += &format!(
            "  rotation {}\n  at {} {}\n  scale {} {}\n  shear {}\n",
            n(t.rotation),
            n(t.at[0]),
            n(t.at[1]),
            n(t.scale[0]),
            n(t.scale[1]),
            n(t.shear)
        );
        let [r, m, k, h] = t.mix;
        s += &format!("  mix {} {} {} {}\n}}\n", n(r), n(m), n(k), n(h));
    }
    for p in rig.paths {
        s += &format!(
            "along {} {{\n  path {}\n  bones{}\n",
            quote(&p.name),
            quote(&rig.attachments[p.path].name),
            bones_text(&p.bones)
        );
        let unit = match p.position_unit {
            PathUnit::Percent => "percent",
            PathUnit::Fixed => "fixed",
        };
        let spacing = match p.spacing_mode {
            PathSpacing::Length => "length",
            PathSpacing::Fixed => "fixed",
            PathSpacing::Percent => "percent",
        };
        let rotate = match p.rotate {
            PathRotate::Tangent => "tangent",
            PathRotate::Chain => "chain",
            PathRotate::ChainScale => "chainscale",
        };
        s += &format!(
            "  position {} {unit}\n  spacing {} {spacing}\n  rotate {rotate} {}\n  mix {} {}\n}}\n",
            n(p.position),
            n(p.spacing),
            n(p.rotation),
            n(p.mix[0]),
            n(p.mix[1])
        );
    }
    for c in rig.clips {
        s += &format!(
            "clip {} {{\n  duration {}\n  loop {}\n",
            quote(&c.name),
            n(c.duration),
            c.looping
        );
        for t in &c.bones {
            s += &format!("  bone {} {{\n", quote(&rig.bones[t.bone].name));
            for k in &t.rotate {
                let turn = match k.value.turn {
                    Turn::Shorter => String::new(),
                    Turn::Clockwise(t) => format!(" clockwise {t}"),
                    Turn::Counterclockwise(t) => format!(" counterclockwise {t}"),
                };
                s += &format!(
                    "    rotate {} {}{turn}{}\n",
                    n(k.time),
                    n(k.value.degrees),
                    ease_text(&k.ease)
                );
            }
            for k in &t.skew {
                s += &format!(
                    "    skew {} {}{}\n",
                    n(k.time),
                    n(k.value),
                    ease_text(&k.ease)
                );
            }
            for k in &t.translate {
                s += &format!(
                    "    translate {} {} {}{}\n",
                    n(k.time),
                    n(k.value[0]),
                    n(k.value[1]),
                    ease_text(&k.ease)
                );
            }
            for k in &t.scale {
                s += &format!(
                    "    scale {} {} {}{}\n",
                    n(k.time),
                    n(k.value[0]),
                    n(k.value[1]),
                    ease_text(&k.ease)
                );
            }
            s += "  }\n";
        }
        for t in &c.slots {
            s += &format!("  slot {} {{\n", quote(&rig.slots[t.slot].name));
            for (time, a) in &t.attachment {
                let a = a.map_or("none".to_string(), |a| quote(&rig.attachments[a].name));
                s += &format!("    attachment {} {a}\n", n(*time));
            }
            for (key, keys) in [("color", &t.color), ("offset", &t.offset)] {
                for k in keys {
                    let [r, g, b, a] = k.value;
                    s += &format!(
                        "    {key} {} {} {} {} {}{}\n",
                        n(k.time),
                        n(r),
                        n(g),
                        n(b),
                        n(a),
                        ease_text(&k.ease)
                    );
                }
            }
            s += "  }\n";
        }
        for t in &c.deforms {
            s += &format!(
                "  deform {} {{\n",
                quote(&rig.attachments[t.attachment].name)
            );
            for k in &t.keys {
                s += &format!("    key {}", n(k.time));
                for v in &k.value {
                    s += &format!(" {}", n(*v));
                }
                s += &format!("{}\n", ease_text(&k.ease));
            }
            s += "  }\n";
        }
        for (time, order) in &c.order {
            s += &format!("  order {}", n(*time));
            for o in order {
                s += &format!(" {}", quote(&rig.slots[*o].name));
            }
            s += "\n";
        }
        for t in &c.iks {
            s += &format!("  ik {} {{\n", quote(&rig.iks[t.ik].name));
            for k in &t.keys {
                s += &format!(
                    "    key {} {} {}{}\n",
                    n(k.time),
                    n(k.value.weight),
                    bend_text(k.value.counterclockwise),
                    ease_text(&k.ease)
                );
            }
            s += "  }\n";
        }
        for (t, name) in &c.events {
            s += &format!("  event {} {}\n", n(*t), quote(name));
        }
        s += "}\n";
    }
    s
}

pub(super) fn quote(s: &str) -> String {
    let mut q = String::from("\"");
    for ch in s.chars() {
        match ch {
            '"' => q += "\\\"",
            '\\' => q += "\\\\",
            '\n' => q += "\\n",
            c => q.push(c),
        }
    }
    q + "\""
}

pub(super) fn num_text(v: f32) -> String {
    Number::from_f32(v)
        .map(|n| n.text().to_string())
        .unwrap_or_else(|_| "0".into())
}
