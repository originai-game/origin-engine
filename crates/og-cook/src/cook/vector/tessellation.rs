//! Vector fill tessellation.

use super::*;

// ── tessellation ─────────────────────────────────────────────────────────

/// The vector's mesh as it is built.
#[derive(Default)]
pub(super) struct Mesh {
    pub(super) vertices: Vec<UiVectorVertex>,
    pub(super) indices: Vec<u32>,
    pub(super) paints: Vec<UiVectorPaint>,
}

impl Mesh {
    pub(super) fn paint(&mut self, p: UiVectorPaint) -> u32 {
        if let Some(i) = self.paints.iter().position(|q| *q == p) {
            return i as u32;
        }
        self.paints.push(p);
        self.paints.len() as u32 - 1
    }

    fn vertex(&mut self, p: Vec2, offset: Vec2, side: f32, paint: u32) -> u32 {
        self.vertices.push(UiVectorVertex {
            position: [p.x, p.y],
            normal: [offset.x, offset.y],
            aa: side,
            paint,
        });
        self.vertices.len() as u32 - 1
    }
}

/// An edge of a fill's outline, `a` to `b`.
#[derive(Clone, Copy, Debug)]
struct Edge {
    a: Vec2,
    b: Vec2,
}

/// An outline edge's ends, quantised.
type EdgeKey = ((i64, i64), (i64, i64));

/// Quantised point, for joining the outline's vertices.
fn key(p: Vec2) -> (i64, i64) {
    (
        (p.x * 65536.0).round() as i64,
        (p.y * 65536.0).round() as i64,
    )
}

/// Whether `p` is inside the outline `edges` by the fill rule.
fn inside(edges: &[Edge], p: Vec2, even_odd: bool) -> bool {
    let mut winding = 0i32;
    for e in edges {
        if (e.a.y <= p.y) != (e.b.y <= p.y) {
            let x = e.a.x + (p.y - e.a.y) / (e.b.y - e.a.y) * (e.b.x - e.a.x);
            if x > p.x {
                winding += if e.b.y > e.a.y { 1 } else { -1 };
            }
        }
    }
    if even_odd {
        winding % 2 != 0
    } else {
        winding != 0
    }
}

/// The edges split where they cross each other or where another edge's end
/// lies on them, so the fill's inside is the same along each piece.
fn split_edges(edges: &[Edge], eps: f32) -> Vec<Edge> {
    let n = edges.len();
    let mut cuts: Vec<Vec<f32>> = vec![Vec::new(); n];
    // Bounding boxes cull most pairs.
    let boxes: Vec<[f32; 4]> = edges
        .iter()
        .map(|e| {
            [
                e.a.x.min(e.b.x) - eps,
                e.a.y.min(e.b.y) - eps,
                e.a.x.max(e.b.x) + eps,
                e.a.y.max(e.b.y) + eps,
            ]
        })
        .collect();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&i, &j| boxes[i][0].total_cmp(&boxes[j][0]));
    for (oi, &i) in order.iter().enumerate() {
        for &j in &order[oi + 1..] {
            if boxes[j][0] > boxes[i][2] {
                break;
            }
            if boxes[j][1] > boxes[i][3] || boxes[i][1] > boxes[j][3] {
                continue;
            }
            let (p, r) = (edges[i].a, edges[i].b - edges[i].a);
            let (q, s) = (edges[j].a, edges[j].b - edges[j].a);
            let rxs = r.perp_dot(s);
            if rxs.abs() > 1e-12 {
                let t = (q - p).perp_dot(s) / rxs;
                let u = (q - p).perp_dot(r) / rxs;
                if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
                    cuts[i].push(t);
                    cuts[j].push(u);
                }
            }
            // T junctions and colinear overlaps: an end on the other edge.
            for (a, other) in [(i, edges[j]), (j, edges[i])] {
                let (p, r) = (edges[a].a, edges[a].b - edges[a].a);
                let len2 = r.length_squared();
                if len2 <= 0.0 {
                    continue;
                }
                for end in [other.a, other.b] {
                    let t = (end - p).dot(r) / len2;
                    if t > 0.0 && t < 1.0 && (p + r * t - end).length() < eps {
                        cuts[a].push(t);
                    }
                }
            }
        }
    }
    let mut out = Vec::with_capacity(n * 2);
    for (e, mut ts) in edges.iter().zip(cuts) {
        ts.retain(|t| *t > 0.0 && *t < 1.0);
        ts.sort_by(f32::total_cmp);
        let mut at = e.a;
        for t in ts {
            let p = e.a + (e.b - e.a) * t;
            if (p - at).length() > eps * 0.01 {
                out.push(Edge { a: at, b: p });
                at = p;
            }
        }
        if (e.b - at).length() > eps * 0.01 {
            out.push(Edge { a: at, b: e.b });
        }
    }
    out
}

/// Fills the closed polylines `contours` by the fill rule into `mesh` with
/// paint `paint`: trapezoids between the outline's vertices, moved in
/// along each outline vertex's miter by the UI pass, and a fringe quad
/// outside each outline edge.
pub(super) fn fill(
    mesh: &mut Mesh,
    contours: &[Vec<Vec2>],
    even_odd: bool,
    paint: u32,
    eps: f32,
) -> Result<(), String> {
    let mut edges = Vec::new();
    for c in contours {
        for k in 0..c.len() {
            let (a, b) = (c[k], c[(k + 1) % c.len()]);
            if (a - b).length() > 0.0 {
                edges.push(Edge { a, b });
            }
        }
    }
    if edges.is_empty() {
        return Ok(());
    }
    let edges = split_edges(&edges, eps);
    // The outline: each piece with the filled side on exactly one hand,
    // oriented so the fill is on its left (y down: its normal (dy, -dx)
    // points out).
    let mut outline: Vec<Edge> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for e in &edges {
        let d = e.b - e.a;
        let len = d.length();
        let n = Vec2::new(d.y, -d.x) / len;
        let mid = (e.a + e.b) * 0.5;
        let probe = (len * 1e-3).min(eps * 0.5).max(1e-6);
        let (left, right) = (
            inside(&edges, mid - n * probe, even_odd),
            inside(&edges, mid + n * probe, even_odd),
        );
        let oriented = match (left, right) {
            (true, false) => *e,
            (false, true) => Edge { a: e.b, b: e.a },
            _ => continue,
        };
        if seen.insert((key(oriented.a), key(oriented.b))) {
            outline.push(oriented);
        }
    }
    // Each outline vertex's miter: the sum of its edges' outward normals,
    // long enough that both offset edges pass through it.
    let mut normals: HashMap<(i64, i64), Vec2> = HashMap::new();
    let out_normal = |e: &Edge| {
        let d = (e.b - e.a).normalize_or_zero();
        Vec2::new(d.y, -d.x)
    };
    for e in &outline {
        let n = out_normal(e);
        *normals.entry(key(e.a)).or_insert(Vec2::ZERO) += n;
        *normals.entry(key(e.b)).or_insert(Vec2::ZERO) += n;
    }
    let mut miters: HashMap<(i64, i64), Vec2> = HashMap::new();
    for e in &outline {
        let n = out_normal(e);
        for p in [e.a, e.b] {
            let sum = normals[&key(p)];
            let l2 = sum.length_squared();
            // Two unit normals: 2·sum/|sum|² reaches both offset lines.
            let m = if l2 > 1e-6 { sum * (2.0 / l2) } else { n };
            let m = if m.length() > 4.0 {
                m.normalize() * 4.0
            } else {
                m
            };
            miters.insert(key(p), m);
        }
    }
    let outline_normal: HashMap<EdgeKey, Vec2> = outline
        .iter()
        .flat_map(|e| {
            let n = out_normal(e);
            [((key(e.a), key(e.b)), n), ((key(e.b), key(e.a)), n)]
        })
        .collect();
    // The trapezoids, slab by slab between the vertices' heights.
    let mut ys: Vec<f32> = edges.iter().flat_map(|e| [e.a.y, e.b.y]).collect();
    ys.sort_by(f32::total_cmp);
    ys.dedup();
    let mut active: Vec<(f32, f32, i32, usize)> = Vec::new();
    for w in ys.windows(2) {
        let (y0, y1) = (w[0], w[1]);
        if y1 - y0 <= 0.0 {
            continue;
        }
        let ym = 0.5 * (y0 + y1);
        active.clear();
        for (i, e) in edges.iter().enumerate() {
            let (lo, hi) = if e.a.y < e.b.y {
                (e.a, e.b)
            } else {
                (e.b, e.a)
            };
            if lo.y <= y0 && hi.y >= y1 && hi.y > lo.y {
                let x_at = |y: f32| lo.x + (y - lo.y) / (hi.y - lo.y) * (hi.x - lo.x);
                active.push((x_at(ym), 0.0, if e.b.y > e.a.y { 1 } else { -1 }, i));
            }
        }
        active.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut winding = 0;
        let mut open: Option<usize> = None;
        for (k, &(_, _, dir, _)) in active.iter().enumerate() {
            let was = if even_odd {
                winding % 2 != 0
            } else {
                winding != 0
            };
            winding += dir;
            let now = if even_odd {
                winding % 2 != 0
            } else {
                winding != 0
            };
            match (was, now) {
                (false, true) => open = Some(k),
                (true, false) => {
                    let l = active[open.take().expect("a span opened")].3;
                    let r = active[k].3;
                    trapezoid(
                        mesh,
                        &edges[l],
                        &edges[r],
                        y0,
                        y1,
                        &miters,
                        &outline_normal,
                        paint,
                    );
                }
                _ => {}
            }
        }
        if mesh.indices.len() / 3 > MAX_TRIANGLES {
            return Err(format!("the vector is over {MAX_TRIANGLES} triangles"));
        }
    }
    // The fringe: a quad straddling each outline edge.
    for e in &outline {
        let (ma, mb) = (miters[&key(e.a)], miters[&key(e.b)]);
        let ia = mesh.vertex(e.a, ma, -1.0, paint);
        let ib = mesh.vertex(e.b, mb, -1.0, paint);
        let oa = mesh.vertex(e.a, ma, 1.0, paint);
        let ob = mesh.vertex(e.b, mb, 1.0, paint);
        mesh.indices.extend([ia, oa, ob, ia, ob, ib]);
    }
    Ok(())
}

/// The trapezoid between edges `l` and `r` from `y0` to `y1`.
#[allow(clippy::too_many_arguments)]
fn trapezoid(
    mesh: &mut Mesh,
    l: &Edge,
    r: &Edge,
    y0: f32,
    y1: f32,
    miters: &HashMap<(i64, i64), Vec2>,
    outline: &HashMap<EdgeKey, Vec2>,
    paint: u32,
) {
    // A point on edge `e` at height `y`, and its offset: the miter at one
    // of its ends, else the edge's own normal if it is on the outline.
    let at = |e: &Edge, y: f32| -> (Vec2, Vec2) {
        for p in [e.a, e.b] {
            if p.y == y {
                return (p, miters.get(&key(p)).copied().unwrap_or(Vec2::ZERO));
            }
        }
        let t = (y - e.a.y) / (e.b.y - e.a.y);
        let p = Vec2::new(e.a.x + t * (e.b.x - e.a.x), y);
        (
            p,
            outline
                .get(&(key(e.a), key(e.b)))
                .copied()
                .unwrap_or(Vec2::ZERO),
        )
    };
    let (tl, tlo) = at(l, y0);
    let (bl, blo) = at(l, y1);
    let (tr, tro) = at(r, y0);
    let (br, bro) = at(r, y1);
    let a = mesh.vertex(tl, tlo, -1.0, paint);
    let b = mesh.vertex(tr, tro, -1.0, paint);
    let c = mesh.vertex(br, bro, -1.0, paint);
    let d = mesh.vertex(bl, blo, -1.0, paint);
    mesh.indices.extend([a, b, c, a, c, d]);
}
