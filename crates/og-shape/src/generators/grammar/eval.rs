//! Applying a grammar's rules to a lot, shape by shape.

use std::collections::BTreeMap;

use glam::{DVec2, DVec3};

use super::face::{self, EPS, Face, Uv, unit};
use super::roof::{self, Role};
use super::split;
use super::{
    Axis, Built, Cond, Grammar, Insert, Lot, MAX_DEPTH, MAX_SHAPES, Num, Op, Part, Rule, Size,
};
use crate::generators::Mesh;
use crate::generators::earcut::earcut;

/// The tags of the ids an operation gives its children (the roof's is
/// `roof.rs`'s).
const EXTRUDE: u64 = 1;
const SPLIT: u64 = 2;
const INSET: u64 = 3;
const PICK: u64 = 5;

/// One label's triangles at one detail level, as they are made.
#[derive(Default)]
struct Acc {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

struct Run {
    params: BTreeMap<String, f64>,
    base: f64,
    acc: BTreeMap<(String, u32), Acc>,
    inserts: Vec<Insert>,
}

fn list<'a>(names: impl Iterator<Item = &'a String>) -> String {
    let v: Vec<&str> = names.map(String::as_str).collect();
    if v.is_empty() {
        "none".into()
    } else {
        v.join(", ")
    }
}

/// Builds `lot` by `grammar`, its parameters at their defaults with
/// `params` over them.
pub fn build(
    grammar: &Grammar,
    lot: &Lot,
    params: &BTreeMap<String, f64>,
) -> Result<Built, String> {
    let mut values = grammar.params.clone();
    for (k, v) in params {
        if !values.contains_key(k) {
            return Err(format!(
                "the grammar has no param `{k}`; its params: {}",
                list(grammar.params.keys())
            ));
        }
        if !v.is_finite() {
            return Err(format!("param `{k}` is {v}, not a number"));
        }
        values.insert(k.clone(), *v);
    }
    let mut run = Run {
        params: values,
        base: lot.base,
        acc: BTreeMap::new(),
        inserts: Vec::new(),
    };
    let mut stack = vec![(root(lot)?, grammar.start.clone(), 0usize)];
    let mut shapes = 0usize;
    while let Some((f, name, depth)) = stack.pop() {
        shapes += 1;
        if shapes > MAX_SHAPES {
            return Err(format!(
                "the lot made more than {MAX_SHAPES} shapes (at rule `{name}`)"
            ));
        }
        if depth > MAX_DEPTH {
            return Err(format!(
                "rule `{name}` is {MAX_DEPTH} rules deep: a rule reaches itself without ending"
            ));
        }
        let rule = grammar
            .rules
            .get(&name)
            .ok_or_else(|| format!("no rule `{name}`"))?;
        let mut children = Vec::new();
        run.apply(rule, f, &mut children)
            .map_err(|e| format!("rule `{name}` (line {}): {e}", rule.line))?;
        stack.extend(children.into_iter().rev().map(|(f, r)| (f, r, depth + 1)));
    }
    let parts = run
        .acc
        .into_iter()
        .map(|((label, detail), a)| Part {
            label,
            detail,
            mesh: Mesh {
                positions: a.positions,
                normals: a.normals,
                uvs: a.uvs,
                colors: Vec::new(),
                tangents: Vec::new(),
                indices: a.indices,
            },
        })
        .collect();
    Ok(Built {
        parts,
        inserts: run.inserts,
        shapes,
    })
}

/// The lot's footprint as a face facing up: x the world's x, y the
/// world's -z, counter-clockwise with its front edge first.
fn root(lot: &Lot) -> Result<Face, String> {
    if !lot.base.is_finite() || lot.footprint.iter().any(|p| !p.is_finite()) {
        return Err("the lot's footprint or base is not finite".into());
    }
    let mut poly: Vec<DVec2> = lot
        .footprint
        .iter()
        .map(|p| DVec2::new(p.x, -p.y))
        .collect();
    if poly.len() >= 3 && face::area2(&poly) < 0.0 {
        // Turned round, its front (the first corner to the second) still first.
        let mut turned = vec![poly[1], poly[0]];
        turned.extend(poly[2..].iter().rev());
        poly = turned;
    }
    let poly = face::simplify(&poly);
    if poly.len() < 3 || face::area2(&poly) <= EPS {
        return Err("the lot's footprint has no area".into());
    }
    let origin = DVec3::new(0.0, lot.base, 0.0);
    let uv = Uv::world(origin, DVec3::X, DVec3::NEG_Z);
    Ok(Face::new(
        origin,
        DVec3::X,
        DVec3::NEG_Z,
        poly,
        uv,
        lot.seed,
        0,
    ))
}

impl Run {
    fn num(&self, n: &Num) -> Result<f64, String> {
        match n {
            Num::Lit(x) => Ok(*x),
            Num::Param(p) => self
                .params
                .get(p)
                .copied()
                .ok_or_else(|| format!("no param `{p}`")),
        }
    }

    fn positive(&self, n: &Num, what: &str) -> Result<f64, String> {
        let v = self.num(n)?;
        if v > 0.0 && v.is_finite() {
            Ok(v)
        } else {
            Err(format!("{what} is above 0, not {v}"))
        }
    }

    /// A wall's mapping on edge `i` of `f` from `a`, `u` along the
    /// perimeter: on a level face, the perimeter and the height above the
    /// lot; otherwise `f`'s own pattern folded over the edge.
    fn side_uv(&self, f: &Face, i: usize, u: f64) -> Uv {
        let a = f.poly[i];
        if f.n.y > 1.0 - 1e-9 {
            Uv::wall(u, f.world(a).y - self.base)
        } else {
            let (e, inward) = face::edge(&f.poly, i);
            f.uv.turned(a, e, -inward)
        }
    }

    /// Makes `f`'s triangles under `label`.
    fn emit(&mut self, label: &str, f: &Face) {
        if f.poly.len() < 3 || face::area2(&f.poly) <= EPS {
            return;
        }
        let flat: Vec<f64> = f.poly.iter().flat_map(|p| [p.x, p.y]).collect();
        let tris = earcut(&flat, &[]);
        let acc = self.acc.entry((label.to_string(), f.detail)).or_default();
        let first = acc.positions.len() as u32;
        let n = f.n.as_vec3().to_array();
        for p in &f.poly {
            acc.positions.push(f.world(*p).as_vec3().to_array());
            acc.normals.push(n);
            let uv = f.uv.at(*p);
            // og-render's v runs down the image.
            acc.uvs.push([uv.x as f32, -uv.y as f32]);
        }
        for t in tris.as_chunks::<3>().0 {
            let [a, b, c] = *t;
            // Every triangle counter-clockwise from the face's front.
            let ccw = (f.poly[b] - f.poly[a]).perp_dot(f.poly[c] - f.poly[a]) >= 0.0;
            let t = if ccw { [a, b, c] } else { [a, c, b] };
            acc.indices.extend(t.map(|i| first + i as u32));
        }
    }

    fn apply(
        &mut self,
        rule: &Rule,
        mut f: Face,
        children: &mut Vec<(Face, String)>,
    ) -> Result<(), String> {
        for op in &rule.ops {
            match op {
                Op::Detail(d) => f.detail = f.detail.max(*d),
                Op::Recess { depth, jamb } => {
                    let d = self.positive(depth, "a recess's depth")?;
                    self.recess(&mut f, d, jamb.as_deref(), children);
                }
                Op::Label(l) => self.emit(l, &f),
                Op::Drop => {}
                Op::Insert(asset) => {
                    let (lo, hi) = f.bounds();
                    self.inserts.push(Insert {
                        asset: asset.clone(),
                        origin: f.world(lo),
                        x: f.x,
                        y: f.y,
                        z: f.n,
                        size: hi - lo,
                        detail: f.detail,
                    });
                }
                Op::Extrude { depth, faces } => {
                    let d = self.positive(depth, "an extrusion's depth")?;
                    self.extrude(&f, d, faces, children);
                }
                Op::Split { axis, parts } => self.split(&f, *axis, parts, children)?,
                Op::Inset {
                    width,
                    border,
                    inner,
                } => {
                    let w = self.positive(width, "an inset's width")?;
                    match face::offset(&f.poly, w) {
                        None => children.push((
                            f.with_poly(f.poly.clone(), f.child(INSET, 0)),
                            border.clone(),
                        )),
                        Some(inside) => {
                            let n = f.poly.len();
                            for i in 0..n {
                                let j = (i + 1) % n;
                                let ring = vec![f.poly[i], f.poly[j], inside[j], inside[i]];
                                children.push((
                                    f.with_poly(ring, f.child(INSET, i as u64)),
                                    border.clone(),
                                ));
                            }
                            children.push((
                                f.with_poly(inside, f.child(INSET, n as u64)),
                                inner.clone(),
                            ));
                        }
                    }
                }
                Op::Roof {
                    kind,
                    angle,
                    overhang,
                    slope,
                    gable,
                    eave,
                } => {
                    let (angle, overhang) = (self.num(angle)?, self.num(overhang)?);
                    for (role, piece) in roof::roof(&f, *kind, angle, overhang, self.base)? {
                        let r = match role {
                            Role::Slope => slope,
                            Role::Gable => gable.as_ref().unwrap_or(slope),
                            Role::Eave => eave.as_ref().unwrap_or(slope),
                        };
                        children.push((piece, r.clone()));
                    }
                }
                Op::Pick(options) => {
                    let total: f64 = options.iter().map(|o| o.0).sum();
                    let mut at = unit(f.child(PICK, 0)) * total;
                    let chosen = options
                        .iter()
                        .find(|(w, _)| {
                            at -= w;
                            at < 0.0
                        })
                        .unwrap_or(&options[options.len() - 1]);
                    let id = f.child(PICK, 1);
                    children.push((f.with_poly(f.poly.clone(), id), chosen.1.clone()));
                }
                Op::Case { arms, otherwise } => {
                    let (lo, hi) = f.bounds();
                    let mut chosen = otherwise.as_ref();
                    for (cond, r) in arms {
                        let holds = match cond {
                            Cond::Wider(n) => hi.x - lo.x >= self.num(n)?,
                            Cond::Taller(n) => hi.y - lo.y >= self.num(n)?,
                            Cond::Param(p, n) => {
                                self.num(&Num::Param(p.clone()))? >= self.num(n)?
                            }
                        };
                        if holds {
                            chosen = Some(r);
                            break;
                        }
                    }
                    if let Some(r) = chosen {
                        children.push((f.clone(), r.clone()));
                    }
                }
            }
        }
        Ok(())
    }

    fn recess(
        &mut self,
        f: &mut Face,
        d: f64,
        jamb: Option<&str>,
        children: &mut Vec<(Face, String)>,
    ) {
        let n = f.poly.len();
        for i in 0..n {
            let (a, b) = (f.poly[i], f.poly[(i + 1) % n]);
            let (e, inward) = face::edge(&f.poly, i);
            let len = a.distance(b);
            // x along the edge, y into the wall: it faces into the opening.
            // Its pattern goes on from the wall round the opening's edge,
            // unfolded over the face.
            let poly = vec![
                DVec2::ZERO,
                DVec2::new(len, 0.0),
                DVec2::new(len, d),
                DVec2::new(0.0, d),
            ];
            let uv = f.uv.turned(a, e, inward);
            let side = Face::new(
                f.world(a),
                f.along(e),
                -f.n,
                poly,
                uv,
                f.child(EXTRUDE, 100 + i as u64),
                f.detail,
            );
            match jamb {
                Some(r) => children.push((side, r.to_string())),
                None => self.emit("jamb", &side),
            }
        }
        f.origin -= f.n * d;
    }

    fn extrude(&self, f: &Face, d: f64, faces: &super::Faces, children: &mut Vec<(Face, String)>) {
        if let Some(r) = &faces.top {
            let mut top = f.with_poly(f.poly.clone(), f.child(EXTRUDE, 0));
            top.origin += f.n * d;
            children.push((top, r.clone()));
        }
        if let Some(r) = &faces.bottom {
            children.push((f.flipped(f.child(EXTRUDE, 1)), r.clone()));
        }
        let n = f.poly.len();
        let front_in = face::edge(&f.poly, 0).1;
        let mut u = 0.0;
        for i in 0..n {
            let (a, b) = (f.poly[i], f.poly[(i + 1) % n]);
            let (e, inward) = face::edge(&f.poly, i);
            let len = a.distance(b);
            let which = if i == 0 {
                &faces.front
            } else if inward.dot(front_in) < -0.7 {
                &faces.back
            } else {
                &faces.side
            };
            if let Some(r) = which.as_ref().or(faces.sides.as_ref()) {
                let poly = vec![
                    DVec2::ZERO,
                    DVec2::new(len, 0.0),
                    DVec2::new(len, d),
                    DVec2::new(0.0, d),
                ];
                let uv = self.side_uv(f, i, u);
                let side = Face::new(
                    f.world(a),
                    f.along(e),
                    f.n,
                    poly,
                    uv,
                    f.child(EXTRUDE, 2 + i as u64),
                    f.detail,
                );
                children.push((side, r.clone()));
            }
            u += len;
        }
    }

    fn split(
        &self,
        f: &Face,
        axis: Axis,
        parts: &[super::SplitPart],
        children: &mut Vec<(Face, String)>,
    ) -> Result<(), String> {
        let (lo, hi) = f.bounds();
        let (g, from, to) = match axis {
            Axis::X => (DVec2::X, lo.x, hi.x),
            Axis::Y => (DVec2::Y, lo.y, hi.y),
        };
        let lengths = parts
            .iter()
            .map(|p| Ok((p.size, self.positive(&p.length, "a split part's length")?)))
            .collect::<Result<Vec<(Size, f64)>, String>>()?;
        for (k, (i, start, end)) in split::pieces(&lengths, to - from)?.into_iter().enumerate() {
            let (start, end) = (from + start, if end == to - from { to } else { from + end });
            let strip = face::simplify(&face::clip(&face::clip(&f.poly, g, end), -g, -start));
            if strip.len() >= 3 && face::area2(&strip) > EPS {
                children.push((
                    f.with_poly(strip, f.child(SPLIT, k as u64)),
                    parts[i].rule.clone(),
                ));
            }
        }
        Ok(())
    }
}
