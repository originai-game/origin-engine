//! Connectivity: the corner table, the EdgeBreaker walk and attribute seams, or the sequential encoding.

use super::*;

pub(crate) fn next(c: i32) -> i32 {
    if c < 0 {
        c
    } else if c % 3 == 2 {
        c - 2
    } else {
        c + 1
    }
}

pub(crate) fn prev(c: i32) -> i32 {
    if c < 0 {
        c
    } else if c % 3 == 0 {
        c + 2
    } else {
        c - 1
    }
}

/// The corner table EdgeBreaker builds.
pub(crate) struct Mesh {
    /// Per corner, its vertex (`face_to_vertex`, `corner_to_vertex_map_[0]`).
    pub(crate) ctv: Vec<i32>,
    /// Per corner, the corner across its opposite edge.
    pub(crate) opp: Vec<i32>,
    /// Per vertex, one of its corners (its left-most on a boundary).
    pub(crate) vertex_corner: Vec<i32>,
    /// Per vertex, its corners so far (stale ones are checked).
    pub(crate) corners_of: Vec<Vec<i32>>,
    pub(crate) hole: Vec<bool>,
    pub(crate) faces: usize,
}

impl Mesh {
    pub(crate) fn opposite(&self, c: i32) -> i32 {
        if c < 0 {
            return -1;
        }
        self.opp.get(c as usize).copied().unwrap_or(-1)
    }
    pub(crate) fn set_opposite(&mut self, a: i32, b: i32) -> Res<()> {
        let n = self.opp.len() as i32;
        if a < 0 || b < 0 || a >= n || b >= n {
            return bad("a corner outside the mesh");
        }
        self.opp[a as usize] = b;
        self.opp[b as usize] = a;
        Ok(())
    }
    pub(crate) fn vert(&self, c: i32) -> Res<i32> {
        match self.ctv.get(c.max(0) as usize) {
            Some(&v) if c >= 0 && v >= 0 => Ok(v),
            _ => bad("a corner without a vertex"),
        }
    }
    /// MapCornerToVertex.
    pub(crate) fn map(&mut self, c: i32, v: i32) -> Res<()> {
        if c < 0 || c as usize >= self.ctv.len() || v < 0 || v as usize >= self.vertex_corner.len()
        {
            return bad("a vertex outside the mesh");
        }
        self.ctv[c as usize] = v;
        self.vertex_corner[v as usize] = c;
        self.corners_of[v as usize].push(c);
        Ok(())
    }
    pub(crate) fn swing_left(&self, c: i32) -> i32 {
        next(self.opposite(next(c)))
    }
    pub(crate) fn swing_right(&self, c: i32) -> i32 {
        prev(self.opposite(prev(c)))
    }
}

/// A decoder's view of the corners: the positions', or an attribute's
/// own, cut at its seams.
pub(crate) struct Seams {
    /// Per corner: the edge opposite it is a seam.
    pub(crate) edge: Vec<bool>,
    /// Per position vertex: it is on a seam.
    pub(crate) vertex: Vec<bool>,
    /// Per corner, its attribute vertex.
    pub(crate) ctv: Vec<i32>,
    /// Per attribute vertex, its left-most corner.
    pub(crate) left_most: Vec<i32>,
}

pub(crate) struct View<'a> {
    pub(crate) m: &'a Mesh,
    pub(crate) s: Option<&'a Seams>,
}

impl View<'_> {
    pub(crate) fn vert(&self, c: i32) -> Res<i32> {
        match self.s {
            None => self.m.vert(c),
            Some(s) => match s.ctv.get(c.max(0) as usize) {
                Some(&v) if c >= 0 && v >= 0 => Ok(v),
                _ => bad("a corner without a vertex"),
            },
        }
    }
    pub(crate) fn vertices(&self) -> usize {
        match self.s {
            None => self.m.vertex_corner.len(),
            Some(s) => s.left_most.len(),
        }
    }
    pub(crate) fn opposite(&self, c: i32) -> i32 {
        if c < 0 {
            return -1;
        }
        if self.s.is_some_and(|s| s.edge[c as usize]) {
            return -1;
        }
        self.m.opposite(c)
    }
    pub(crate) fn swing_left(&self, c: i32) -> i32 {
        next(self.opposite(next(c)))
    }
    pub(crate) fn swing_right(&self, c: i32) -> i32 {
        prev(self.opposite(prev(c)))
    }
    pub(crate) fn on_boundary(&self, v: i32) -> Res<bool> {
        match self.s {
            None => {
                let c = self.m.vertex_corner[v as usize];
                Ok(c < 0 || self.m.swing_left(c) < 0)
            }
            Some(s) => {
                let c = s.left_most[v as usize];
                Ok(c < 0 || s.vertex[self.m.vert(c)? as usize])
            }
        }
    }
}

/// Per attribute with its own connectivity: the probability of no seam
/// and the seam bits.
pub(crate) type SeamBits<'a> = Vec<(u8, &'a [u8])>;

pub(crate) const C: u32 = 0;

pub(crate) const S: u32 = 1;

pub(crate) const L: u32 = 3;

pub(crate) const R: u32 = 5;

pub(crate) const E: u32 = 7;

/// EdgeBreaker connectivity: the mesh, and each attribute's seam bits
/// still to read.
pub(crate) fn edgebreaker<'a>(r: &mut Reader<'a>) -> Res<(Mesh, SeamBits<'a>)> {
    let traversal = r.u8()?;
    if traversal != 0 && traversal != 2 {
        return bad("an EdgeBreaker traversal that is neither standard nor valence");
    }
    let valence = traversal == 2;
    let vertices = r.count(96)?;
    let faces = r.count(32)?;
    let attribute_data = usize::from(r.u8()?);
    let n_symbols = r.count(32)?;
    let n_splits = r.count(32)?;
    if n_symbols > faces || faces > u32::MAX as usize / 3 {
        return bad("more symbols than faces");
    }
    // Topology split events.
    let n_events = r.count(1)?;
    let mut source_ids = Vec::with_capacity(n_events);
    let mut split_ids = Vec::with_capacity(n_events);
    let mut last = 0i64;
    for _ in 0..n_events {
        let source = r.var()? as i64 + last;
        let split = source - r.var()? as i64;
        last = source;
        source_ids.push(source);
        split_ids.push(split);
    }
    let mut edge_bits = Vec::with_capacity(n_events);
    {
        let mut bits = Bits {
            d: &r.d[r.at..],
            bit: 0,
        };
        for _ in 0..n_events {
            edge_bits.push(bits.read(1)?);
        }
        r.at += bits.bit.div_ceil(8);
    }
    // The traversal's buffers.
    let mut symbol_bits = None;
    if !valence {
        let size = r.count(1)?;
        symbol_bits = Some(Bits {
            d: r.bytes(size)?,
            bit: 0,
        });
    }
    let start_prob = r.u8()?;
    let size = r.count(1)?;
    let start_faces = r.bytes(size)?;
    let mut seams = Vec::with_capacity(attribute_data);
    for _ in 0..attribute_data {
        let prob = r.u8()?;
        let size = r.count(1)?;
        seams.push((prob, r.bytes(size)?));
    }
    let nv = vertices + n_splits;
    let mut valences = vec![0i32; if valence { nv } else { 0 }];
    let mut contexts: Vec<Vec<u32>> = Vec::new();
    if valence {
        for _ in 0..6 {
            let n = r.var()? as usize;
            if n > faces {
                return bad("more context symbols than faces");
            }
            contexts.push(if n > 0 { symbols(r, n, 1)? } else { Vec::new() });
        }
    }
    let mut m = Mesh {
        ctv: vec![-1; faces * 3],
        opp: vec![-1; faces * 3],
        vertex_corner: vec![-1; nv],
        corners_of: vec![Vec::new(); nv],
        hole: vec![true; nv],
        faces: 0,
    };
    let mut stack: Vec<i32> = Vec::new();
    let mut split_at: BTreeMap<i64, Vec<i32>> = BTreeMap::new();
    let mut last_vert: i32 = -1;
    let mut context: i32 = -1;
    let empty = || "the Draco data is not valid: EdgeBreaker ran out of open edges".to_string();
    let bump = |valences: &mut Vec<i32>, v: i32, by: i32| {
        if let Some(x) = valences.get_mut(v as usize) {
            *x += by;
        }
    };
    for id in 0..n_symbols {
        let symbol = if valence {
            if context < 0 {
                E
            } else {
                let Some(s) = contexts[context as usize].pop() else {
                    return bad("a valence context out of symbols");
                };
                *[C, S, L, R, E]
                    .get(s as usize)
                    .ok_or("the Draco data is not valid: an EdgeBreaker symbol over 4")?
            }
        } else {
            let bits = symbol_bits.as_mut().ok_or_else(empty)?;
            let mut s = bits.read(1)?;
            if s != C {
                s |= bits.read(2)? << 1;
            }
            s
        };
        let c = (3 * id) as i32;
        let mut check_split = false;
        let (vert, nxt, prv);
        match symbol {
            C => {
                let a = *stack.last().ok_or_else(empty)?;
                let mut b = prev(a);
                let mut guard = 0;
                while m.opposite(b) >= 0 {
                    b = prev(m.opposite(b));
                    guard += 1;
                    if guard > faces * 3 {
                        return bad("a fan that does not end");
                    }
                }
                m.set_opposite(a, c + 1)?;
                m.set_opposite(b, c + 2)?;
                *stack.last_mut().ok_or_else(empty)? = c;
                vert = m.vert(next(a))?;
                nxt = m.vert(next(b))?;
                prv = m.vert(prev(a))?;
                bump(&mut valences, nxt, 1);
                bump(&mut valences, prv, 1);
                m.hole[vert as usize] = false;
                m.map(c, vert)?;
                m.map(c + 1, nxt)?;
                m.map(c + 2, prv)?;
            }
            S => {
                let b = stack.pop().ok_or_else(empty)?;
                if let Some(corners) = split_at.remove(&(id as i64)) {
                    stack.extend(corners);
                }
                let a = *stack.last().ok_or_else(empty)?;
                m.set_opposite(a, c + 2)?;
                m.set_opposite(b, c + 1)?;
                *stack.last_mut().ok_or_else(empty)? = c;
                vert = m.vert(prev(a))?;
                nxt = m.vert(next(a))?;
                prv = m.vert(prev(b))?;
                m.map(c, vert)?;
                m.map(c + 1, nxt)?;
                m.map(c + 2, prv)?;
                // The two fans' first vertices are one: `vertex_n` goes.
                let gone = m.vert(next(b))?;
                if gone != vert {
                    if valence {
                        let add = valences[gone as usize];
                        bump(&mut valences, vert, add);
                    }
                    for k in std::mem::take(&mut m.corners_of[gone as usize]) {
                        if m.ctv[k as usize] == gone {
                            m.ctv[k as usize] = vert;
                            m.corners_of[vert as usize].push(k);
                        }
                    }
                    m.vertex_corner[gone as usize] = -1;
                }
                bump(&mut valences, nxt, 1);
                bump(&mut valences, prv, 1);
            }
            R | L => {
                let a = *stack.last().ok_or_else(empty)?;
                m.set_opposite(if symbol == R { c + 2 } else { c + 1 }, a)?;
                *stack.last_mut().ok_or_else(empty)? = c;
                check_split = true;
                last_vert += 1;
                if symbol == R {
                    vert = m.vert(prev(a))?;
                    nxt = m.vert(next(a))?;
                    prv = last_vert;
                    bump(&mut valences, vert, 1);
                    bump(&mut valences, nxt, 1);
                    bump(&mut valences, prv, 2);
                } else {
                    vert = m.vert(next(a))?;
                    nxt = last_vert;
                    prv = m.vert(prev(a))?;
                    bump(&mut valences, vert, 1);
                    bump(&mut valences, nxt, 2);
                    bump(&mut valences, prv, 1);
                }
                m.map(c + 2, prv)?;
                m.map(c, vert)?;
                m.map(c + 1, nxt)?;
            }
            E => {
                stack.push(c);
                check_split = true;
                vert = last_vert + 1;
                nxt = vert + 1;
                prv = nxt + 1;
                last_vert = prv;
                for v in [vert, nxt, prv] {
                    bump(&mut valences, v, 2);
                }
                m.map(c, vert)?;
                m.map(c + 1, nxt)?;
                m.map(c + 2, prv)?;
            }
            _ => return bad("an EdgeBreaker symbol"),
        }
        if valence {
            let v = valences.get(nxt as usize).copied().unwrap_or(0);
            context = v.clamp(2, 7) - 2;
        }
        if check_split {
            let encoder_id = (n_symbols - id - 1) as i64;
            while source_ids.last() == Some(&encoder_id) {
                source_ids.pop();
                let (Some(edge), Some(split)) = (edge_bits.pop(), split_ids.pop()) else {
                    return bad("a topology split without its edge");
                };
                let top = *stack.last().ok_or_else(empty)?;
                let corner = if edge == 1 { next(top) } else { prev(top) };
                split_at
                    .entry(n_symbols as i64 - split - 1)
                    .or_default()
                    .push(corner);
            }
        }
    }
    m.faces = n_symbols;
    // ProcessInteriorEdges: the faces that close a start.
    let mut ans = Ans::new(start_faces, L_RANS_BASE)?;
    while let Some(a) = stack.pop() {
        if !ans.bit(start_prob) {
            continue;
        }
        if m.faces >= faces {
            return bad("more faces than the header counts");
        }
        let mut b = prev(a);
        let mut cc = next(a);
        let mut guard = 0;
        while m.opposite(b) >= 0 || m.opposite(cc) >= 0 {
            if m.opposite(b) >= 0 {
                b = prev(m.opposite(b));
            }
            if m.opposite(cc) >= 0 {
                cc = next(m.opposite(cc));
            }
            guard += 1;
            if guard > faces * 3 {
                return bad("a fan that does not end");
            }
        }
        let c = (m.faces * 3) as i32;
        m.set_opposite(c, a)?;
        m.set_opposite(c + 1, b)?;
        m.set_opposite(c + 2, cc)?;
        let (na, nb, nc) = (m.vert(next(a))?, m.vert(next(b))?, m.vert(next(cc))?);
        m.map(c, nb)?;
        m.map(c + 1, nc)?;
        m.map(c + 2, na)?;
        for v in [na, nb, nc] {
            m.hole[v as usize] = false;
        }
        m.faces += 1;
    }
    if m.faces != faces {
        return bad("fewer faces than the header counts");
    }
    Ok((m, seams))
}

/// DecodeAttributeSeams and RecomputeVerticesInternal for one attribute.
pub(crate) fn seams_of(m: &Mesh, prob: u8, data: &[u8]) -> Res<Seams> {
    let corners = m.faces * 3;
    let mut s = Seams {
        edge: vec![false; corners],
        vertex: vec![false; m.vertex_corner.len()],
        ctv: vec![-1; corners],
        left_most: Vec::new(),
    };
    let mut ans = Ans::new(data, L_RANS_BASE)?;
    for c in 0..corners as i32 {
        let o = m.opposite(c);
        let seam = if o < 0 {
            true
        } else if o >= c {
            let on = ans.bit(prob);
            if on {
                s.edge[o as usize] = true;
            }
            on
        } else {
            s.edge[c as usize]
        };
        if seam {
            s.edge[c as usize] = true;
            s.vertex[m.vert(next(c))? as usize] = true;
            s.vertex[m.vert(prev(c))? as usize] = true;
        }
    }
    let view = View { m, s: None };
    let limit = corners + 1;
    for v in 0..m.vertex_corner.len() {
        let c = m.vertex_corner[v];
        if c < 0 {
            continue;
        }
        let mut id = s.left_most.len() as i32;
        let mut first = c;
        if s.vertex[v] {
            // To the corner left of which a seam (or the boundary) lies.
            let cut = |k: i32| {
                let n = next(k);
                if n < 0 || s.edge[n as usize] {
                    -1
                } else {
                    next(m.opposite(n))
                }
            };
            let mut act = cut(first);
            let mut guard = 0;
            while act >= 0 {
                first = act;
                act = cut(act);
                guard += 1;
                if guard > limit {
                    return bad("a fan that does not end");
                }
            }
        }
        s.ctv[first as usize] = id;
        s.left_most.push(first);
        let mut act = view.swing_right(first);
        let mut guard = 0;
        while act >= 0 && act != first {
            if s.edge[next(act) as usize] {
                id = s.left_most.len() as i32;
                s.left_most.push(act);
            }
            s.ctv[act as usize] = id;
            act = view.swing_right(act);
            guard += 1;
            if guard > limit {
                return bad("a fan that does not end");
            }
        }
    }
    Ok(s)
}

/// The order a decoder's values come in: the corner each was first met
/// at, and each vertex's place in that order.
pub(crate) struct Sequence {
    pub(crate) entry_corner: Vec<i32>,
    pub(crate) vertex_entry: Vec<i32>,
}

/// GenerateSequence for EdgeBreaker: depth first, or by prediction degree.
pub(crate) fn sequence(view: &View, by_degree: bool) -> Res<Sequence> {
    let faces = view.m.faces;
    let nv = view.vertices();
    let mut seq = Sequence {
        entry_corner: Vec::new(),
        vertex_entry: vec![-1; nv],
    };
    let mut face_seen = vec![false; faces];
    let mut seen = vec![false; nv];
    let mut degree = vec![0u32; if by_degree { nv } else { 0 }];
    let visit = |seq: &mut Sequence, seen: &mut Vec<bool>, c: i32| -> Res<bool> {
        let v = view.vert(c)? as usize;
        if seen[v] {
            return Ok(false);
        }
        seen[v] = true;
        seq.vertex_entry[v] = seq.entry_corner.len() as i32;
        seq.entry_corner.push(c);
        Ok(true)
    };
    let face_done = |face_seen: &Vec<bool>, c: i32| c < 0 || face_seen[c as usize / 3];
    let right = |c: i32| view.opposite(next(c));
    let left = |c: i32| view.opposite(prev(c));
    for f in 0..faces {
        let start = (3 * f) as i32;
        if by_degree {
            let mut stacks: [Vec<i32>; 3] = [vec![start], Vec::new(), Vec::new()];
            let mut best = 0usize;
            visit(&mut seq, &mut seen, next(start))?;
            visit(&mut seq, &mut seen, prev(start))?;
            visit(&mut seq, &mut seen, start)?;
            loop {
                let mut popped = None;
                for (i, stack) in stacks.iter_mut().enumerate().skip(best) {
                    if let Some(c) = stack.pop() {
                        best = i;
                        popped = Some(c);
                        break;
                    }
                }
                let Some(mut c) = popped else { break };
                if face_done(&face_seen, c) {
                    continue;
                }
                loop {
                    face_seen[c as usize / 3] = true;
                    visit(&mut seq, &mut seen, c)?;
                    let (rc, lc) = (right(c), left(c));
                    let (r_done, l_done) = (face_done(&face_seen, rc), face_done(&face_seen, lc));
                    let mut priority = |corner: i32| -> Res<usize> {
                        let tip = view.vert(corner)? as usize;
                        if seen[tip] {
                            return Ok(0);
                        }
                        degree[tip] += 1;
                        Ok(if degree[tip] > 1 { 1 } else { 2 })
                    };
                    if !l_done {
                        let p = priority(lc)?;
                        if r_done && p <= best {
                            c = lc;
                            continue;
                        }
                        stacks[p].push(lc);
                        best = best.min(p);
                    }
                    if !r_done {
                        let p = priority(rc)?;
                        if p <= best {
                            c = rc;
                            continue;
                        }
                        stacks[p].push(rc);
                        best = best.min(p);
                    }
                    break;
                }
            }
            continue;
        }
        if face_seen[f] {
            continue;
        }
        let mut stack = vec![start];
        visit(&mut seq, &mut seen, next(start))?;
        visit(&mut seq, &mut seen, prev(start))?;
        while let Some(&top) = stack.last() {
            let mut c = top;
            if face_done(&face_seen, c) {
                stack.pop();
                continue;
            }
            loop {
                face_seen[c as usize / 3] = true;
                let v = view.vert(c)?;
                if visit(&mut seq, &mut seen, c)? && !view.on_boundary(v)? {
                    c = right(c);
                    if c < 0 {
                        return bad("an inner vertex on an open edge");
                    }
                    continue;
                }
                let (rc, lc) = (right(c), left(c));
                match (face_done(&face_seen, rc), face_done(&face_seen, lc)) {
                    (true, true) => {
                        stack.pop();
                        break;
                    }
                    (true, false) => c = lc,
                    (false, true) => c = rc,
                    (false, false) => {
                        if let Some(top) = stack.last_mut() {
                            *top = lc;
                        }
                        stack.push(rc);
                        break;
                    }
                }
            }
        }
    }
    Ok(seq)
}
