//! Booleans and edits between parts, each recorded with what its result reports.

use super::*;

/// How [`Model::combine`] runs a boolean: a zero-value desc (a join, not
/// checked).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Combine {
    /// Join, cut or intersect.
    pub op: BoolOp,
    /// Refuse, before any work, an operand that passes through itself
    /// ([`crate::boolean::check_operands`]): for operands from outside the
    /// kernel. It costs a sweep over each operand's triangles.
    pub check: bool,
}

/// One boolean of a model and what it did ([`Model::booleans`]).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BoolRecord {
    /// `"cut"`, `"join"` or `"intersect"`.
    pub op: String,
    /// The target's name.
    pub target: String,
    /// The tool's name.
    pub tool: String,
    /// What the result alone does not say.
    pub report: BoolReport,
}

/// The boolean's snap distance on `s`: 2⁻⁴⁰ of its box's diagonal
/// (`docs/research.md` §18.1, §23.2), with the box's corners.
pub(crate) fn snap_distance(s: &Solid) -> (f64, [DVec3; 2]) {
    let (mut lo, mut hi) = (DVec3::INFINITY, DVec3::NEG_INFINITY);
    for &p in &s.positions {
        lo = lo.min(p);
        hi = hi.max(p);
    }
    ((hi - lo).length() * (1.0 / (1u64 << 40) as f64), [lo, hi])
}

/// Whether `s` has a triangle that rounding can turn over: an edge shorter
/// than the snap distance, or a corner that close to the line of the
/// opposite (longest) edge. Only these decide whether to look; what is
/// changed is decided by exact predicates.
pub(crate) fn thin(s: &Solid) -> bool {
    if s.triangles.is_empty() {
        return false;
    }
    let (tol, _) = snap_distance(s);
    let tol2 = tol * tol;
    s.triangles.iter().any(|t| {
        let [a, b, c] = t.map(|v| s.positions[v as usize]);
        let longest = (b - a)
            .length_squared()
            .max((c - b).length_squared())
            .max((a - c).length_squared());
        let shortest = (b - a)
            .length_squared()
            .min((c - b).length_squared())
            .min((a - c).length_squared());
        // Twice the area is the height on the longest edge times its length.
        shortest < tol2 || (b - a).cross(c - a).length_squared() < tol2 * longest
    })
}

/// `s` (canonical, closed) with its crossing pairs repaired by
/// [`crate::boolean::uncross_output`]: points closer than the snap distance
/// merged, needles split away, each change kept only if it leaves the
/// solid closed and crossing itself strictly less. Each kept triangle
/// keeps its source's surface, names and UVs (interpolated at a moved
/// corner); a named edge keeps its names on what it became; a skinned
/// vertex keeps its weights. Unchanged if nothing improves.
pub(crate) fn uncrossed(s: Solid) -> Solid {
    let (_, corners) = snap_distance(&s);
    let out = crate::boolean::CsgOutput {
        points: s.positions.clone(),
        tris: s.triangles.clone(),
        origins: (0..s.triangles.len() as u32).map(|i| (0, i)).collect(),
        coplanar: Vec::new(),
        touching: Vec::new(),
        second_pass: false,
        unmatched: 0,
        touching_edges: 0,
        non_manifold: 0,
        snapped: false,
    };
    // The box's corners stand in for the operands: they set the snap
    // distance and pin nothing a rounding error matters to.
    let (out, report) = crate::boolean::uncross_output(&corners, &[], out);
    if report.after >= report.before || out.unmatched > 0 || out.non_manifold > 0 {
        return s;
    }
    let key = |p: DVec3| p.to_array().map(|x| (x + 0.0).to_bits());
    let at: BTreeMap<[u64; 3], u32> = out
        .points
        .iter()
        .enumerate()
        .map(|(i, &p)| (key(p), i as u32))
        .collect();
    // Each old vertex's new one: itself, or for a merged point the
    // nearest point left (it was merged onto a point within the snap
    // distance); ties to the lower index.
    let new_of: Vec<u32> = s
        .positions
        .iter()
        .map(|&p| {
            at.get(&key(p)).copied().unwrap_or_else(|| {
                let mut best = (f64::INFINITY, 0u32);
                for (i, &q) in out.points.iter().enumerate() {
                    let d = (q - p).length_squared();
                    if d < best.0 {
                        best = (d, i as u32);
                    }
                }
                best.1
            })
        })
        .collect();
    let mut r = Solid {
        positions: out.points,
        names: s.names.clone(),
        ..Default::default()
    };
    let mut uvs = s.uvs.as_ref().map(|_| Vec::with_capacity(out.tris.len()));
    for (t, &(_, src)) in out.tris.iter().zip(&out.origins) {
        let src = src as usize;
        r.triangles.push(*t);
        r.surfaces.push(s.surfaces[src]);
        r.face_tags.push(s.face_tags[src].clone());
        if let (Some(u), Some(old)) = (&mut uvs, &s.uvs) {
            u.push(t.map(|v| s.uv_at(src, r.positions[v as usize]).unwrap_or(old[src][0])));
        }
    }
    r.uvs = uvs;
    // Named edges: on the edge their ends became, or, split at a point,
    // on both its pieces.
    let mut edges: BTreeSet<(u32, u32)> = BTreeSet::new();
    for t in &r.triangles {
        for k in 0..3 {
            edges.insert(crate::mesh::edge_key(t[k], t[(k + 1) % 3]));
        }
    }
    let mut tagged: BTreeMap<(u32, u32), Vec<crate::mesh::Tag>> = BTreeMap::new();
    for (&(a, b), tags) in &s.edge_tags {
        let (a, b) = (new_of[a as usize], new_of[b as usize]);
        if a == b {
            continue;
        }
        let pieces: Vec<(u32, u32)> = if edges.contains(&crate::mesh::edge_key(a, b)) {
            vec![crate::mesh::edge_key(a, b)]
        } else {
            (0..r.positions.len() as u32)
                .find(|&m| {
                    edges.contains(&crate::mesh::edge_key(a, m))
                        && edges.contains(&crate::mesh::edge_key(m, b))
                })
                .map(|m| vec![crate::mesh::edge_key(a, m), crate::mesh::edge_key(m, b)])
                .unwrap_or_default()
        };
        for e in pieces {
            let entry = tagged.entry(e).or_default();
            entry.extend(tags);
            entry.sort_unstable();
            entry.dedup();
        }
    }
    r.edge_tags = tagged;
    if let Some(skin) = &s.skin {
        let mut weights = vec![Default::default(); r.positions.len()];
        for (old, &new) in new_of.iter().enumerate() {
            if r.positions[new as usize] == s.positions[old] {
                weights[new as usize] = skin.weights.get(old).copied().unwrap_or_default();
            }
        }
        r.skin = Some(crate::mesh::Skin {
            bones: skin.bones.clone(),
            weights,
        });
    }
    let _ = r.canonicalize();
    r
}

impl Model {
    /// Replaces part `id`'s solid with `f` of it (in its own space).
    pub(crate) fn edit(
        &mut self,
        id: Id,
        f: impl FnOnce(&Solid, &Frame) -> Result<Solid, ModelError>,
    ) -> Result<(), ModelError> {
        let placement = self.node(id)?.placement;
        let s = f(self.solid_of(id)?, &placement)?;
        self.nodes[id.0 as usize].solid = Some(s);
        Ok(())
    }

    /// As [`Self::edit`], for an edit that reports what its result alone
    /// does not say.
    pub(crate) fn edit_report(
        &mut self,
        id: Id,
        f: impl FnOnce(&Solid, &Frame) -> Result<(Solid, EditReport), ModelError>,
    ) -> Result<EditReport, ModelError> {
        let placement = self.node(id)?.placement;
        let (s, r) = f(self.solid_of(id)?, &placement)?;
        self.nodes[id.0 as usize].solid = Some(s);
        Ok(r)
    }

    /// Why `tool` may not be used up.
    pub(crate) fn usable(&self, tool: Id, target: Id) -> Result<(), ModelError> {
        let n = self.node(tool)?;
        if tool == target {
            return Err(ModelError::Refused(
                n.name.clone(),
                "a part cannot be its own tool".into(),
            ));
        }
        if self
            .nodes
            .iter()
            .any(|m| !m.used_up && m.parent == Some(tool))
        {
            return Err(ModelError::Refused(
                n.name.clone(),
                "it holds other nodes; a tool is a single part".into(),
            ));
        }
        if let Some(j) = self
            .joints
            .iter()
            .find(|j| j.child == tool || j.parent == tool)
        {
            return Err(ModelError::Refused(
                n.name.clone(),
                format!(
                    "joint {:?} needs it; seat cuts a cavity and leaves the part",
                    j.desc.name
                ),
            ));
        }
        Ok(())
    }

    /// `tool`'s solid in `target`'s own space.
    pub(crate) fn tool_in(&self, tool: Id, target: Id) -> Result<Solid, ModelError> {
        let w = self.world_solid(tool)?;
        Ok(placed(w, &self.node(target)?.placement.inverse()))
    }

    /// `target` combined with `tool` by `d.op` ([`crate::boolean::boolean_report`]);
    /// the tool is used up. Answers what the result alone does not say
    /// (voids left, edges where the operands touch), which the model also
    /// keeps ([`Model::booleans`]).
    pub fn combine(&mut self, target: Id, tool: Id, d: &Combine) -> Result<BoolReport, ModelError> {
        self.solid_of(target)?;
        self.solid_of(tool)?;
        self.usable(tool, target)?;
        let mut t = self.tool_in(tool, target)?;
        if d.op == BoolOp::Cut {
            // The cut turns the tool's faces inside out; its UVs would
            // read mirrored from the cavity (`crate::edit::cut_tool_uvs`).
            t = crate::edit::cut_tool_uvs(&t);
        }
        let (tn, un) = (
            self.nodes[target.0 as usize].name.clone(),
            self.nodes[tool.0 as usize].name.clone(),
        );
        let mut report = BoolReport::default();
        self.edit(target, |s, _| {
            if d.check {
                crate::boolean::check_operands(s, &t)?;
            }
            let (out, r) = crate::boolean::boolean_report(
                s,
                &t,
                &Boolean {
                    op: d.op,
                    target: &tn,
                    tool: &un,
                },
            )?;
            report = r;
            Ok(out)
        })?;
        self.nodes[tool.0 as usize].used_up = true;
        let op = match d.op {
            BoolOp::Cut => "cut",
            BoolOp::Join => "join",
            BoolOp::Intersect => "intersect",
        };
        self.booleans.push(BoolRecord {
            op: op.into(),
            target: tn,
            tool: un,
            report: report.clone(),
        });
        Ok(report)
    }

    /// Every boolean [`Model::combine`] (and so `cut`, `join`,
    /// `intersect`) ran, in order, with its report.
    pub fn booleans(&self) -> &[BoolRecord] {
        &self.booleans
    }

    /// `target` minus `tool`; the tool is used up. Names as
    /// [`crate::boolean::boolean`] gives them (`cut`, `<target>.cut`, the
    /// tool's name).
    pub fn cut(&mut self, target: Id, tool: Id) -> Result<BoolReport, ModelError> {
        self.combine(
            target,
            tool,
            &Combine {
                op: BoolOp::Cut,
                ..Default::default()
            },
        )
    }

    /// `target` joined with `tool`; the tool is used up.
    pub fn join(&mut self, target: Id, tool: Id) -> Result<BoolReport, ModelError> {
        self.combine(target, tool, &Combine::default())
    }

    /// `target` intersected with `tool`; the tool is used up.
    pub fn intersect(&mut self, target: Id, tool: Id) -> Result<BoolReport, ModelError> {
        self.combine(
            target,
            tool,
            &Combine {
                op: BoolOp::Intersect,
                ..Default::default()
            },
        )
    }

    /// `tool`'s solid added to `target`'s as it is: no boolean, so shells
    /// that touch or cross stay as they are ([`crate::edit::merge`]); faces
    /// and edges keep their names. The tool is used up.
    pub fn merge(&mut self, target: Id, tool: Id) -> Result<(), ModelError> {
        self.solid_of(target)?;
        self.solid_of(tool)?;
        self.usable(tool, target)?;
        let t = self.tool_in(tool, target)?;
        if t.uvs.is_some() != self.solid_of(target)?.uvs.is_some() {
            let without = if t.uvs.is_none() { tool } else { target };
            return Err(ModelError::Refused(
                self.nodes[without.0 as usize].name.clone(),
                "it has no UVs and the part it would merge with has: give both UVs, or neither"
                    .into(),
            ));
        }
        self.edit(target, |s, _| Ok(crate::edit::merge(&[s, &t])))?;
        self.nodes[tool.0 as usize].used_up = true;
        Ok(())
    }

    /// The volume `part` fills, taken out of `host`; `part` is untouched
    /// ([`crate::edit::seat`]).
    pub fn seat(&mut self, host: Id, part: Id) -> Result<EditReport, ModelError> {
        if host == part {
            return Err(ModelError::Refused(
                self.node(host)?.name.clone(),
                "a part cannot seat itself".into(),
            ));
        }
        let t = self.tool_in(part, host)?;
        let name = self.node(part)?.name.clone();
        self.edit_report(host, |s, _| Ok(crate::edit::seat_report(s, &t, &name)?))
    }

    /// Holes ([`crate::edit::hole`]); `d.at` frames are in the world.
    pub fn hole(&mut self, id: Id, d: &Hole) -> Result<EditReport, ModelError> {
        self.edit_report(id, |s, p| {
            let inv = p.inverse();
            let local = Hole {
                at: d.at.iter().map(|f| inv.compose(f)).collect(),
                ..d.clone()
            };
            Ok(crate::edit::hole_report(s, &local)?)
        })
    }

    /// A pocket ([`crate::edit::pocket`]); `d.at` frames are in the world.
    pub fn pocket(&mut self, id: Id, d: &Pocket) -> Result<EditReport, ModelError> {
        self.edit_report(id, |s, p| {
            let inv = p.inverse();
            let at = if d.at.is_empty() {
                vec![inv]
            } else {
                d.at.iter().map(|f| inv.compose(f)).collect()
            };
            let local = Pocket { at, ..d.clone() };
            Ok(crate::edit::pocket_report(s, &local)?)
        })
    }

    /// The part cut by the plane of `frame` (world), keeping one side.
    pub fn split(&mut self, id: Id, frame: &Frame, keep: Keep) -> Result<(), ModelError> {
        finite_frame(frame, "split")?;
        self.edit(id, |s, p| {
            Ok(crate::edit::split(s, &p.inverse().compose(frame), keep)?)
        })
    }

    /// Hollows the part ([`crate::edit::shell`]); directions in `d.open`
    /// are the part's own axes.
    pub fn shell(&mut self, id: Id, d: &Shell) -> Result<EditReport, ModelError> {
        self.edit_report(id, |s, _| Ok(crate::edit::shell_report(s, d)?))
    }

    /// Dents ([`crate::edit::dent`]); `d.at` points are in the world.
    pub fn dent(&mut self, id: Id, d: &Dent) -> Result<EditReport, ModelError> {
        self.edit_report(id, |s, p| {
            let inv = p.inverse();
            let local = Dent {
                at: d.at.iter().map(|&q| inv.point_to_world(q)).collect(),
                ..d.clone()
            };
            Ok(crate::edit::dent_report(s, &local)?)
        })
    }

    /// Fillets on named edges ([`crate::bevel::fillet`]).
    pub fn fillet(&mut self, id: Id, d: &Fillet) -> Result<(), ModelError> {
        self.edit(id, |s, _| Ok(crate::bevel::fillet(s, d)?))
    }

    /// Rounds the part by subdivision ([`crate::edit::subdivide`]): its
    /// flat faces are the cage, edges named in `d.sharp` stay sharp.
    pub fn subdivide(&mut self, id: Id, d: &Subdivide) -> Result<(), ModelError> {
        self.edit(id, |s, _| Ok(crate::edit::subdivide(s, d)?))
    }

    /// Chamfers on named edges ([`crate::bevel::chamfer`]).
    pub fn chamfer(&mut self, id: Id, d: &Chamfer) -> Result<(), ModelError> {
        self.edit(id, |s, _| Ok(crate::bevel::chamfer(s, d)?))
    }

    /// Softened edges ([`crate::bevel::soften`]).
    pub fn soften(&mut self, id: Id, d: &Soften) -> Result<(), ModelError> {
        self.edit(id, |s, _| Ok(crate::bevel::soften(s, d)?))
    }
}
