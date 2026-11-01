//! Rig asset operations, pose evaluation and child playback.

use super::*;

impl Rig2dAsset {
    fn body(&self) -> Body<'_> {
        Body {
            bones: &self.bones,
            slots: &self.slots,
            attachments: &self.attachments,
            iks: &self.iks,
            transforms: &self.transforms,
            paths: &self.paths,
            clips: &self.clips,
        }
    }

    /// Reads a `type rig2d` text (the grammar is the module's docs).
    pub fn parse(text: &str) -> Result<Rig2dAsset> {
        let doc = Document::parse(text)?;
        if doc.ty != RIG2D_TYPE {
            return Err(Error::new(
                ErrorKind::Parse,
                format!("a 2D rig is `type rig2d`, not `{}`", doc.ty),
            ));
        }
        let mut texture = String::new();
        let mut size = [0, 0];
        let mut filter = SheetFilter::Linear;
        let mut unit = 1.0;
        let body = &doc.body.entries;
        for e in body {
            match e.key.as_str() {
                "texture" => {
                    texture = e
                        .string(0)
                        .map_err(|_| parse_err(e.line, "texture \"path\""))?
                        .to_string()
                }
                "size" => size = [num(e, 0)? as u32, num(e, 1)? as u32],
                "filter" => {
                    filter = match e.word(0).unwrap_or("") {
                        "nearest" => SheetFilter::Nearest,
                        "linear" => SheetFilter::Linear,
                        w => {
                            return Err(parse_err(
                                e.line,
                                format!("filter is nearest or linear, not `{w}`"),
                            ));
                        }
                    }
                }
                "unit" => {
                    unit = num(e, 0)?;
                    if unit <= 0.0 {
                        return Err(parse_err(e.line, "unit is above 0 (pixels per world unit)"));
                    }
                }
                "armature" | "dragonbones" | "atlas" | "skin" => {}
                k if BODY_KEYS.contains(&k) => {}
                k => return Err(parse_err(e.line, format!("unknown key `{k}` in a 2D rig"))),
            }
        }
        if texture.is_empty() {
            return Err(Error::new(
                ErrorKind::Parse,
                "a 2D rig needs `texture \"path\"`",
            ));
        }
        // Child armatures: names first, so any body can show any of them.
        let blocks: Vec<&Entry> = body.iter().filter(|e| e.key == "armature").collect();
        let mut names: Vec<String> = Vec::new();
        for e in &blocks {
            let name = name_of(e)?;
            if names.contains(&name) {
                return Err(parse_err(
                    e.line,
                    format!("armature `{name}` is declared twice"),
                ));
            }
            names.push(name);
        }
        let mut armatures = Vec::new();
        for (e, name) in blocks.iter().zip(&names) {
            let entries = &block(e)?.entries;
            if let Some(c) = entries
                .iter()
                .find(|c| !BODY_KEYS.contains(&c.key.as_str()) && c.key != "skin")
            {
                return Err(parse_err(
                    c.line,
                    format!("unknown key `{}` in armature {name}", c.key),
                ));
            }
            let mut a = parse_body(entries, &names).map_err(|err| {
                Error::new(
                    ErrorKind::Parse,
                    format!("armature {name}: {}", err.reason()),
                )
            })?;
            a.skins = parse_skins(entries, &a.attachments).map_err(|err| {
                Error::new(
                    ErrorKind::Parse,
                    format!("armature {name}: {}", err.reason()),
                )
            })?;
            a.name = name.clone();
            armatures.push(a);
        }
        let top = parse_body(body, &names)?;
        let rig = Rig2dAsset {
            texture,
            size,
            filter,
            unit,
            bones: top.bones,
            slots: top.slots,
            attachments: top.attachments,
            iks: top.iks,
            transforms: top.transforms,
            paths: top.paths,
            clips: top.clips,
            skins: Vec::new(),
            armatures,
        };
        let mut rig = rig;
        rig.skins = parse_skins(body, &rig.attachments)?;
        rig.check_children()?;
        Ok(rig)
    }

    /// Each child attachment names a clip its armature has, and no
    /// armature shows itself (directly or through another).
    fn check_children(&self) -> Result<()> {
        let shows = |b: Body<'_>| -> Vec<(usize, Option<String>, String)> {
            b.attachments
                .iter()
                .filter_map(|a| match &a.kind {
                    AttachmentKind::Child { armature, clip, .. } => {
                        Some((*armature, clip.clone(), a.name.clone()))
                    }
                    _ => None,
                })
                .collect()
        };
        let edges: Vec<Vec<usize>> = self
            .armatures
            .iter()
            .map(|a| shows(a.body()).into_iter().map(|s| s.0).collect())
            .collect();
        let all = std::iter::once(shows(self.body()))
            .chain(self.armatures.iter().map(|a| shows(a.body())));
        for list in all {
            for (a, clip, name) in list {
                if let Some(c) = clip
                    && !self.armatures[a].clips.iter().any(|x| x.name == c)
                {
                    return Err(Error::new(
                        ErrorKind::Parse,
                        format!(
                            "child {name}: armature {} has no clip `{c}`",
                            self.armatures[a].name
                        ),
                    ));
                }
            }
        }
        // Depth-first from each armature: reaching it again is a cycle.
        for start in 0..self.armatures.len() {
            let mut stack = edges[start].clone();
            let mut seen = vec![false; self.armatures.len()];
            while let Some(a) = stack.pop() {
                if a == start {
                    return Err(Error::new(
                        ErrorKind::Parse,
                        format!(
                            "armature {} shows itself through its child attachments",
                            self.armatures[start].name
                        ),
                    ));
                }
                if !std::mem::replace(&mut seen[a], true) {
                    stack.extend(&edges[a]);
                }
            }
        }
        Ok(())
    }

    /// The canonical text: every value spelled out. Parses back equal.
    pub fn to_text(&self) -> String {
        let mut s = String::from("type rig2d\nversion 1\n---\n");
        s += &format!("texture {}\n", quote(&self.texture));
        s += &format!("size {} {}\n", self.size[0], self.size[1]);
        s += &format!(
            "filter {}\n",
            if self.filter == SheetFilter::Nearest {
                "nearest"
            } else {
                "linear"
            }
        );
        s += &format!("unit {}\n", num_text(self.unit));
        s += &body_text(self.body(), &self.armatures);
        s += &skins_text(&self.skins, &self.attachments);
        for a in &self.armatures {
            s += &format!("armature {} {{\n", quote(&a.name));
            let inner =
                body_text(a.body(), &self.armatures) + &skins_text(&a.skins, &a.attachments);
            for line in inner.lines() {
                s += &format!("  {line}\n");
            }
            s += "}\n";
        }
        s
    }

    /// The index of a clip by name.
    pub fn clip(&self, name: &str) -> Option<u32> {
        self.clips
            .iter()
            .position(|c| c.name == name)
            .map(|i| i as u32)
    }

    pub fn bone(&self, name: &str) -> Option<usize> {
        self.bones.iter().position(|b| b.name == name)
    }

    /// The pose at `time` of `clip` (`None`: the setup pose), with IK
    /// `targets` (per constraint: a target in rig pixels and a weight that
    /// replace the target bone's origin and multiply the constraint's weight).
    pub fn pose(
        &self,
        clip: Option<&Clip2d>,
        time: f32,
        targets: &[(usize, [f32; 2], f32)],
    ) -> Pose2d {
        self.pose_of(self.body(), clip, time, targets, &[], "default")
    }

    /// [`Rig2dAsset::pose`] wearing skin `skin` (0 the default, `i + 1`
    /// [`Rig2dAsset::skins`]`[i]`; one past them is the default). Each
    /// child armature shown wears its own skin of that name, else its
    /// default.
    pub fn pose_in_skin(
        &self,
        clip: Option<&Clip2d>,
        time: f32,
        targets: &[(usize, [f32; 2], f32)],
        skin: u32,
    ) -> Pose2d {
        let worn = (skin as usize)
            .checked_sub(1)
            .and_then(|i| self.skins.get(i));
        let swaps = worn.map_or(&[][..], |k| k.swaps.as_slice());
        let name = worn.map_or("default", |k| k.name.as_str());
        self.pose_of(self.body(), clip, time, targets, swaps, name)
    }

    /// The pose of `body` wearing `swaps`; its child armatures wear their
    /// skins named `skin`, else their defaults.
    fn pose_of(
        &self,
        body: Body<'_>,
        clip: Option<&Clip2d>,
        time: f32,
        targets: &[(usize, [f32; 2], f32)],
        swaps: &[(usize, Option<usize>)],
        skin: &str,
    ) -> Pose2d {
        // x, y, rotation, scale x, scale y, skew.
        let mut local: Vec<[f32; 6]> = body
            .bones
            .iter()
            .map(|b| [b.x, b.y, b.rotation, b.scale[0], b.scale[1], b.skew])
            .collect();
        let mut slots: Vec<Option<usize>> = body.slots.iter().map(|s| s.attachment).collect();
        let mut colors: Vec<[f32; 4]> = body.slots.iter().map(|s| s.color).collect();
        let mut offsets: Vec<[f32; 4]> = body.slots.iter().map(|s| s.offset).collect();
        let mut order: Vec<usize> = (0..body.slots.len()).collect();
        let mut deforms: Vec<(usize, Vec<f32>)> = Vec::new();
        let mut ik_keys: Vec<(f32, bool)> = body
            .iks
            .iter()
            .map(|c| (c.weight, c.counterclockwise))
            .collect();
        let lerp4 =
            |a: &[f32; 4], b: &[f32; 4], f: f32| [0, 1, 2, 3].map(|i| a[i] + (b[i] - a[i]) * f);
        if let Some(c) = clip {
            for t in &c.bones {
                let l = &mut local[t.bone];
                if let Some(r) = sample(&t.rotate, time, |a, b, f| Rotation {
                    degrees: a.degrees + turn_delta(a.degrees, b.degrees, a.turn) * f,
                    turn: a.turn,
                }) {
                    l[2] += r.degrees;
                }
                if let Some(k) = sample(&t.skew, time, |a, b, f| a + (b - a) * f) {
                    l[5] += k;
                }
                if let Some([x, y]) = sample(&t.translate, time, lerp2) {
                    l[0] += x;
                    l[1] += y;
                }
                if let Some([x, y]) = sample(&t.scale, time, lerp2) {
                    l[3] *= x;
                    l[4] *= y;
                }
            }
            for t in &c.slots {
                if let Some((_, a)) = t.attachment.iter().rev().find(|(k, _)| *k <= time) {
                    slots[t.slot] = *a;
                }
                if let Some(v) = sample(&t.color, time, lerp4) {
                    colors[t.slot] = v;
                }
                if let Some(v) = sample(&t.offset, time, lerp4) {
                    offsets[t.slot] = v;
                }
            }
            for t in &c.deforms {
                if let Some(v) = sample(&t.keys, time, |a: &Vec<f32>, b: &Vec<f32>, f| {
                    a.iter().zip(b).map(|(x, y)| x + (y - x) * f).collect()
                }) {
                    deforms.push((t.attachment, v));
                }
            }
            if let Some((_, o)) = c.order.iter().rev().find(|(k, _)| *k <= time)
                && !o.is_empty()
            {
                order = o.clone();
            }
            for t in &c.iks {
                if let Some(k) = sample(&t.keys, time, |a, b, f| IkKey {
                    weight: a.weight + (b.weight - a.weight) * f,
                    counterclockwise: a.counterclockwise,
                }) {
                    ik_keys[t.ik] = (k.weight.clamp(0.0, 1.0), k.counterclockwise);
                }
            }
        }
        // The skin: what each slot shows in place of the default's.
        for a in slots.iter_mut() {
            if let Some(&(_, to)) = a.and_then(|x| swaps.iter().find(|(f, _)| *f == x)) {
                *a = to;
            }
        }
        let mut world = world_of(body.bones, &local);
        let mut ik = Vec::with_capacity(body.iks.len());
        for (i, c) in body.iks.iter().enumerate() {
            let (rig_weight, counterclockwise) = ik_keys[i];
            let (target, weight) = match targets.iter().find(|t| t.0 == i) {
                Some(&(_, t, w)) => (t, rig_weight * w.clamp(0.0, 1.0)),
                None => {
                    let w = world[c.target];
                    ([w.tx, w.ty], rig_weight)
                }
            };
            let t = [f64::from(target[0]), f64::from(target[1])];
            let w = f64::from(weight);
            let tip_of = |world: &[Affine2], b: usize| world[b].apply(body.bones[b].length, 0.0);
            match c.bones.as_slice() {
                [b] => {
                    let m = world[*b];
                    let want = atan2(t[1] - f64::from(m.ty), t[0] - f64::from(m.tx));
                    local[*b][2] += (wrap(want - m.angle()) * w / DEG) as f32;
                }
                [p, ch] => {
                    let o1 = [f64::from(world[*p].tx), f64::from(world[*p].ty)];
                    let o2 = [f64::from(world[*ch].tx), f64::from(world[*ch].ty)];
                    let tip = tip_of(&world, *ch).map(f64::from);
                    let l1 = sq(o2[0] - o1[0], o2[1] - o1[1]);
                    let l2 = sq(tip[0] - o2[0], tip[1] - o2[1]);
                    let (dx, dy) = (t[0] - o1[0], t[1] - o1[1]);
                    let d = (dx * dx + dy * dy)
                        .sqrt()
                        .clamp((l1 - l2).abs() + 1e-6, (l1 + l2 - 1e-6).max(1e-6));
                    let cos_a =
                        ((l1 * l1 + d * d - l2 * l2) / (2.0 * l1 * d).max(1e-12)).clamp(-1.0, 1.0);
                    let a = atan2((1.0 - cos_a * cos_a).max(0.0).sqrt(), cos_a);
                    let side = if counterclockwise { 1.0 } else { -1.0 };
                    let want = atan2(dy, dx) + side * a;
                    let now = atan2(o2[1] - o1[1], o2[0] - o1[0]);
                    local[*p][2] += (wrap(want - now) * w / DEG) as f32;
                    world = world_of(body.bones, &local);
                    let o2 = [f64::from(world[*ch].tx), f64::from(world[*ch].ty)];
                    let tip = tip_of(&world, *ch).map(f64::from);
                    let want = atan2(t[1] - o2[1], t[0] - o2[0]);
                    let now = atan2(tip[1] - o2[1], tip[0] - o2[0]);
                    local[*ch][2] += (wrap(want - now) * w / DEG) as f32;
                }
                chain => fabrik(body.bones, &mut local, chain, t, w, counterclockwise),
            }
            world = world_of(body.bones, &local);
            let last = *c.bones.last().expect("a chain has a bone");
            let reach = if c.bones.len() == 1 {
                // One bone points: its error is how far off its direction the target lies.
                let m = world[last];
                let dir = m.angle();
                let (s, co) = sincos(dir);
                let (vx, vy) = (t[0] - f64::from(m.tx), t[1] - f64::from(m.ty));
                (vx * s - vy * co).abs() as f32
            } else {
                let tip = tip_of(&world, last);
                sq(f64::from(tip[0] - target[0]), f64::from(tip[1] - target[1])) as f32
            };
            ik.push(reach);
        }
        // Transform, then path constraints: world transforms that hold, the
        // bones under them following.
        if !body.transforms.is_empty() || !body.paths.is_empty() {
            let mut fixed: Vec<Option<Affine2>> = vec![None; body.bones.len()];
            for c in body.transforms {
                transform_constraint(body.bones, &local, &mut fixed, &mut world, c);
            }
            for c in body.paths {
                let slot = body.attachments[c.path].slot;
                if slots[slot] != Some(c.path) {
                    continue;
                }
                let deform = deforms
                    .iter()
                    .find(|d| d.0 == c.path)
                    .map(|d| d.1.as_slice());
                path_constraint(body, &local, &mut fixed, &mut world, c, deform);
            }
        }
        // Each slot showing a child armature: its pose at this time.
        let mut children = Vec::new();
        for (s, a) in slots.iter().enumerate() {
            let Some(a) = a else { continue };
            let AttachmentKind::Child {
                armature,
                clip: own,
                ..
            } = &body.attachments[*a].kind
            else {
                continue;
            };
            let arm = &self.armatures[*armature];
            let played = child_clip(arm, clip, own.as_deref());
            let t = match played {
                Some(c) if c.looping => time - c.duration * (time / c.duration).floor(),
                Some(c) => time.clamp(0.0, c.duration),
                None => 0.0,
            };
            let swaps = arm
                .skins
                .iter()
                .find(|k| k.name == skin)
                .map_or(&[][..], |k| k.swaps.as_slice());
            children.push((s, self.pose_of(arm.body(), played, t, &[], swaps, skin)));
        }
        Pose2d {
            world,
            slots,
            colors,
            offsets,
            order,
            deforms,
            ik_error: ik,
            children,
        }
    }

    /// The events the child armatures shown in `pose` (the pose at the end
    /// of the stretch, nested ones included) pass while their parent's
    /// clip `clip` plays over `spans` of its time: forward, each `(lo,
    /// hi]` played from `lo` up to `hi` (a `lo` below 0 taking the events
    /// at 0 too); `backward`, each `[lo, hi)` played from `hi` down to
    /// `lo` (the event at the time it lands on fires, the one it leaves
    /// does not, as og.skeleton2d's own looping backward play). Answers
    /// (the child armature's name, its clip's index, the event's time in
    /// it, its name), span by span in the order they are played, each
    /// span's events by time. A child plays at its parent's time, wrapped
    /// by its own clip's duration when that loops and held at its end
    /// otherwise, so a span splits where it wraps.
    pub fn child_events(
        &self,
        clip: Option<&Clip2d>,
        spans: &[(f32, f32)],
        backward: bool,
        pose: &Pose2d,
    ) -> Vec<(String, u32, f32, String)> {
        let mut out = Vec::new();
        self.collect_child_events(self.body(), clip, spans, backward, pose, &mut out);
        out
    }

    fn collect_child_events(
        &self,
        body: Body<'_>,
        clip: Option<&Clip2d>,
        spans: &[(f32, f32)],
        backward: bool,
        pose: &Pose2d,
        out: &mut Vec<(String, u32, f32, String)>,
    ) {
        for (s, child) in &pose.children {
            let Some(a) = pose.slots[*s] else { continue };
            let AttachmentKind::Child {
                armature,
                clip: own,
                ..
            } = &body.attachments[a].kind
            else {
                continue;
            };
            let arm = &self.armatures[*armature];
            let Some(played) = child_clip(arm, clip, own.as_deref()) else {
                continue;
            };
            let index = arm
                .clips
                .iter()
                .position(|c| std::ptr::eq(c, played))
                .unwrap_or(0) as u32;
            let inner: Vec<(f32, f32)> = spans
                .iter()
                .flat_map(|&(lo, hi)| {
                    if backward {
                        wrap_span_back(lo, hi, played)
                    } else {
                        wrap_span(lo, hi, played)
                    }
                })
                .collect();
            for &(lo, hi) in &inner {
                let passed = |t: f32| {
                    if backward {
                        t >= lo && t < hi
                    } else {
                        t > lo && t <= hi
                    }
                };
                for (t, name) in played.events.iter().filter(|(t, _)| passed(*t)) {
                    out.push((arm.name.clone(), index, *t, name.clone()));
                }
            }
            self.collect_child_events(arm.body(), Some(played), &inner, backward, child, out);
        }
    }

    /// The triangles a pose draws, in its draw order (a child armature at
    /// its slot's place): corners in world units of the rig's space
    /// (pixels over `unit`), uvs (0..1, v down), colour multipliers and
    /// offsets, with triangle indices.
    pub fn triangles(&self, pose: &Pose2d) -> (Vec<RigVertex>, Vec<u32>) {
        let mut out = Vec::new();
        let mut indices = Vec::new();
        self.draw(
            self.body(),
            pose,
            &Affine2::IDENTITY,
            ([1.0; 4], [0.0; 4]),
            &mut out,
            &mut indices,
        );
        (out, indices)
    }

    fn draw(
        &self,
        body: Body<'_>,
        pose: &Pose2d,
        place: &Affine2,
        tint: ([f32; 4], [f32; 4]),
        out: &mut Vec<RigVertex>,
        indices: &mut Vec<u32>,
    ) {
        let (w, h) = (self.size[0].max(1) as f32, self.size[1].max(1) as f32);
        let inv = 1.0 / self.unit;
        for &s in &pose.order {
            let slot = &body.slots[s];
            let Some(a) = pose.slots[s] else { continue };
            // The parent's colour map applied to this slot's: m (c t + o) + p.
            let (m, p) = tint;
            let color = [0, 1, 2, 3].map(|i| m[i] * pose.colors[s][i]);
            let offset = [0, 1, 2, 3].map(|i| m[i] * pose.offsets[s][i] + p[i]);
            let bone = place.mul(&pose.world[slot.bone]);
            let base = out.len() as u32;
            match &body.attachments[a].kind {
                AttachmentKind::Region {
                    rect,
                    at,
                    rotation,
                    skew,
                    size,
                } => {
                    let m = bone.mul(&Affine2::trsk(at[0], at[1], *rotation, *skew, 1.0, 1.0));
                    let (hx, hy) = (size[0] / 2.0, size[1] / 2.0);
                    let (u0, u1) = (rect[0] / w, (rect[0] + rect[2]) / w);
                    let (v_top, v_bottom) = (rect[1] / h, (rect[1] + rect[3]) / h);
                    for (x, y, u, v) in [
                        (-hx, -hy, u0, v_bottom),
                        (hx, -hy, u1, v_bottom),
                        (hx, hy, u1, v_top),
                        (-hx, hy, u0, v_top),
                    ] {
                        let p = m.apply(x, y);
                        out.push(RigVertex {
                            position: [p[0] * inv, p[1] * inv],
                            uv: [u, v],
                            color,
                            offset,
                        });
                    }
                    indices.extend([0, 1, 2, 2, 3, 0].map(|i| base + i));
                }
                AttachmentKind::Mesh {
                    vertices,
                    triangles,
                } => {
                    let deform = pose
                        .deforms
                        .iter()
                        .find(|d| d.0 == a)
                        .map(|d| d.1.as_slice());
                    let mut k = 0usize;
                    for v in vertices {
                        let mut p = [0.0f32; 2];
                        for i in &v.influences {
                            let (dx, dy) = deform.map_or((0.0, 0.0), |d| {
                                (
                                    d.get(2 * k).copied().unwrap_or(0.0),
                                    d.get(2 * k + 1).copied().unwrap_or(0.0),
                                )
                            });
                            k += 1;
                            let q = place.apply_to(pose.world[i.bone].apply(i.x + dx, i.y + dy));
                            p[0] += q[0] * i.weight;
                            p[1] += q[1] * i.weight;
                        }
                        out.push(RigVertex {
                            position: [p[0] * inv, p[1] * inv],
                            uv: [v.uv[0] / w, v.uv[1] / h],
                            color,
                            offset,
                        });
                    }
                    indices.extend(triangles.iter().map(|t| base + t));
                }
                AttachmentKind::Box { .. } | AttachmentKind::Path { .. } => {}
                AttachmentKind::Child {
                    armature,
                    at,
                    rotation,
                    skew,
                    scale,
                    ..
                } => {
                    if let Some((_, child)) = pose.children.iter().find(|c| c.0 == s) {
                        let m = bone.mul(&Affine2::trsk(
                            at[0], at[1], *rotation, *skew, scale[0], scale[1],
                        ));
                        self.draw(
                            self.armatures[*armature].body(),
                            child,
                            &m,
                            (color, offset),
                            out,
                            indices,
                        );
                    }
                }
            }
        }
    }

    /// Whether a child armature has clip events (og.skeleton2d fires them).
    pub fn has_children(&self) -> bool {
        self.armatures
            .iter()
            .any(|a| a.clips.iter().any(|c| !c.events.is_empty()))
    }

    /// Whether any attachment (of the rig or a child armature) is a box.
    pub fn has_boxes(&self) -> bool {
        let any = |a: &[Attachment2d]| {
            a.iter()
                .any(|a| matches!(a.kind, AttachmentKind::Box { .. }))
        };
        any(&self.attachments) || self.armatures.iter().any(|x| any(&x.attachments))
    }

    /// The bounding boxes a pose shows, in its draw order (a child
    /// armature's inside its slot, named `slot/child-slot`): each polygon
    /// in the rig's pixels.
    pub fn boxes(&self, pose: &Pose2d) -> Vec<BoundingBox2d> {
        let mut out = Vec::new();
        self.collect_boxes(self.body(), pose, &Affine2::IDENTITY, "", &mut out);
        out
    }

    fn collect_boxes(
        &self,
        body: Body<'_>,
        pose: &Pose2d,
        place: &Affine2,
        prefix: &str,
        out: &mut Vec<BoundingBox2d>,
    ) {
        for &s in &pose.order {
            let Some(a) = pose.slots[s] else { continue };
            let slot = &body.slots[s];
            let bone = place.mul(&pose.world[slot.bone]);
            let attachment = &body.attachments[a];
            match &attachment.kind {
                AttachmentKind::Box { polygon } => out.push(BoundingBox2d {
                    slot: format!("{prefix}{}", slot.name),
                    name: attachment.name.clone(),
                    points: polygon.iter().map(|p| bone.apply_to(*p)).collect(),
                }),
                AttachmentKind::Child {
                    armature,
                    at,
                    rotation,
                    skew,
                    scale,
                    ..
                } => {
                    if let Some((_, child)) = pose.children.iter().find(|c| c.0 == s) {
                        let m = bone.mul(&Affine2::trsk(
                            at[0], at[1], *rotation, *skew, scale[0], scale[1],
                        ));
                        self.collect_boxes(
                            self.armatures[*armature].body(),
                            child,
                            &m,
                            &format!("{prefix}{}/", slot.name),
                            out,
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

/// The clip a child armature plays under its parent's `clip`: the one of
/// the same name, else its `clip` line's, else its first.
fn child_clip<'a>(
    arm: &'a Armature2d,
    clip: Option<&Clip2d>,
    own: Option<&str>,
) -> Option<&'a Clip2d> {
    let by_name = |n: &str| arm.clips.iter().find(|c| c.name == n);
    clip.and_then(|c| by_name(&c.name))
        .or_else(|| own.and_then(by_name))
        .or(arm.clips.first())
}

/// A span `(lo, hi]` of a parent's time as spans of a child clip's time:
/// split where a looping clip wraps (each later piece taking the events
/// at 0, as og.skeleton2d's own wrap does), clamped to a one-shot's length.
fn wrap_span(lo: f32, hi: f32, c: &Clip2d) -> Vec<(f32, f32)> {
    let d = c.duration;
    if !c.looping {
        let l = if lo < 0.0 { -1.0 } else { lo.min(d) };
        let h = hi.clamp(0.0, d);
        return if h > l { vec![(l, h)] } else { Vec::new() };
    }
    let mut out = Vec::new();
    let mut base = d * (lo.max(0.0) / d).floor();
    let mut l = if lo < 0.0 { -1.0 } else { lo - base };
    for _ in 0..64 {
        let h = (hi - base).min(d);
        if h > l {
            out.push((l, h));
        }
        if hi - base < d {
            break;
        }
        base += d;
        l = -1.0;
    }
    out
}

/// A span `[lo, hi)` of a parent's time played backwards (from `hi` down
/// to `lo`) as spans of a child clip's time, in the order they are played:
/// split where a looping clip wraps (the piece below a wrap ending short
/// of the clip's end, as og.skeleton2d's own backward wrap does), clamped
/// to a one-shot's length.
fn wrap_span_back(lo: f32, hi: f32, c: &Clip2d) -> Vec<(f32, f32)> {
    let d = c.duration;
    let lo = lo.max(0.0);
    if !c.looping || d <= 0.0 {
        let (l, h) = (lo.clamp(0.0, d), hi.clamp(0.0, d));
        return if h > l { vec![(l, h)] } else { Vec::new() };
    }
    let mut out = Vec::new();
    // The cycle holding the times just below `hi`.
    let mut base = d * ((hi / d).ceil() - 1.0).max(0.0);
    let mut h = hi - base;
    for _ in 0..64 {
        let l = (lo - base).max(0.0);
        if h > l {
            out.push((l, h.min(d)));
        }
        if lo >= base || base <= 0.0 {
            break;
        }
        base -= d;
        h = d;
    }
    out
}

fn lerp2(a: &[f32; 2], b: &[f32; 2], f: f32) -> [f32; 2] {
    [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f]
}

/// The value of keys at `t` (`None` without keys), eased by the key in effect.
fn sample<T: Clone>(keys: &[Key<T>], t: f32, lerp: impl Fn(&T, &T, f32) -> T) -> Option<T> {
    let first = keys.first()?;
    if t <= first.time {
        return Some(first.value.clone());
    }
    let i = keys.iter().rposition(|k| k.time <= t).unwrap_or(0);
    let k = &keys[i];
    match keys.get(i + 1) {
        Some(n) if n.time > k.time => {
            let f = (t - k.time) / (n.time - k.time);
            match &k.ease {
                Ease::Linear => Some(lerp(&k.value, &n.value, f)),
                Ease::Step => Some(k.value.clone()),
                Ease::Bezier(c) => Some(lerp(&k.value, &n.value, bezier(c, f))),
            }
        }
        _ => Some(k.value.clone()),
    }
}
