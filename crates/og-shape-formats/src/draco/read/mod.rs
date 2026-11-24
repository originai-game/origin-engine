//! Draco-compressed triangle meshes, decoded (`docs/research.md` §44):
//! what glTF's `KHR_draco_mesh_compression` holds. Written from the
//! Draco bitstream specification (version 2.2; Google, Apache-2.0; see
//! `third_party/draco/`), whose names the comments keep so the two
//! can be read side by side.
//!
//! Entry: [`decode`]. Read: sequential and EdgeBreaker connectivity
//! (standard and valence), both traversals, and the attributes'
//! values by every prediction the specification has but the normals',
//! which are parsed and not computed (the importer recomputes normals).
//! Integer work throughout; a quantized value is scaled in f32 as the
//! specification scales it.

use std::collections::BTreeMap;

mod attributes;
mod bits;
mod connectivity;
pub(crate) use attributes::*;
pub(crate) use bits::*;
pub(crate) use connectivity::*;

/// One attribute of a decoded mesh.
pub(crate) struct Attribute {
    /// The id glTF names it by.
    pub id: u32,
    /// Numbers a point.
    pub components: usize,
    /// Per point, its numbers. Empty for normals (not computed).
    pub values: Vec<f64>,
    /// The values are whole numbers of this many bytes, as stored
    /// (negative: signed); 0 for floats.
    pub integer: i8,
}

/// A decoded mesh: triangles over points, and each attribute a point.
pub(crate) struct Decoded {
    pub faces: Vec<[u32; 3]>,
    pub points: usize,
    pub attributes: Vec<Attribute>,
}

pub(crate) type Res<T> = Result<T, String>;

pub(crate) fn short<T>() -> Res<T> {
    Err("the Draco data is cut short".into())
}

pub(crate) fn bad<T>(what: &str) -> Res<T> {
    Err(format!("the Draco data is not valid: {what}"))
}

/// A Draco triangle mesh: its faces over points, and each attribute's
/// values a point.
pub(crate) fn decode(data: &[u8]) -> Res<Decoded> {
    let mut r = Reader { d: data, at: 0 };
    if r.bytes(5)? != b"DRACO" {
        return bad("it does not start with DRACO");
    }
    let (major, minor) = (r.u8()?, r.u8()?);
    if (major, minor) != (2, 2) {
        return Err(format!(
            "Draco bitstream version {major}.{minor} (only 2.2, which glTF requires)"
        ));
    }
    let (kind, method) = (r.u8()?, r.u8()?);
    if kind != 1 {
        return Err("a Draco point cloud, not a triangle mesh".into());
    }
    let flags = r.u16()?;
    if flags & 0x8000 != 0 {
        metadata(&mut r)?;
    }
    // Connectivity.
    let mut mesh = None;
    let mut seam_data = Vec::new();
    let mut faces: Vec<[u32; 3]> = Vec::new();
    let mut points = 0usize;
    match method {
        0 => {
            let n_faces = r.count(32)?;
            points = r.count(96)?;
            match r.u8()? {
                0 => {
                    let s = symbols(&mut r, n_faces * 3, 1)?;
                    let mut last = 0i32;
                    for f in s.as_chunks::<3>().0 {
                        let mut t = [0u32; 3];
                        for (k, &v) in f.iter().enumerate() {
                            let diff = (v >> 1) as i32;
                            last = last.wrapping_add(if v & 1 != 0 { -diff } else { diff });
                            t[k] = last as u32;
                        }
                        faces.push(t);
                    }
                }
                1 => {
                    for _ in 0..n_faces {
                        let mut t = [0u32; 3];
                        for v in &mut t {
                            *v = if points < 256 {
                                u32::from(r.u8()?)
                            } else if points < 1 << 16 {
                                u32::from(r.u16()?)
                            } else if points < 1 << 21 {
                                r.var()? as u32
                            } else {
                                r.u32()?
                            };
                        }
                        faces.push(t);
                    }
                }
                _ => return bad("a way of storing indices"),
            }
        }
        1 => {
            let (m, s) = edgebreaker(&mut r)?;
            mesh = Some(m);
            seam_data = s;
        }
        _ => return bad("a mesh encoding that is neither sequential nor EdgeBreaker"),
    }
    // The attribute decoders' headers.
    let decoders = usize::from(r.u8()?);
    let mut heads = Vec::with_capacity(decoders);
    for _ in 0..decoders {
        heads.push(if mesh.is_some() {
            (r.u8()? as i8, r.u8()?, r.u8()?)
        } else {
            (-1, 0, 0)
        });
    }
    struct Att {
        kind: u8,
        data_type: u8,
        components: usize,
        id: u32,
        decoder: u8,
    }
    let mut attributes: Vec<Vec<Att>> = Vec::with_capacity(decoders);
    for _ in 0..decoders {
        let n = r.count(1)?;
        let mut list = Vec::with_capacity(n);
        for _ in 0..n {
            let (kind, data_type, components) = (r.u8()?, r.u8()?, usize::from(r.u8()?));
            r.u8()?;
            let id = r.var()? as u32;
            if components == 0 {
                return bad("an attribute of no components");
            }
            list.push(Att {
                kind,
                data_type,
                components,
                id,
                decoder: 0,
            });
        }
        for a in &mut list {
            a.decoder = r.u8()?;
        }
        attributes.push(list);
    }
    // EdgeBreaker: each attribute's seams, the points, each decoder's
    // order of values.
    let mut seams: Vec<Seams> = Vec::new();
    let mut corner_point: Vec<u32> = Vec::new();
    let mut sequences: Vec<Sequence> = Vec::new();
    let view_of = |m: &'_ Mesh, seams: &'_ [Seams], head: (i8, u8, u8)| -> Res<Option<usize>> {
        let _ = m;
        if head.1 == 0 || head.0 < 0 {
            return Ok(None);
        }
        if head.0 as usize >= seams.len() {
            return bad("an attribute without its connectivity");
        }
        Ok(Some(head.0 as usize))
    };
    if let Some(m) = &mut mesh {
        // A boundary vertex's corner: its left-most.
        for v in 0..m.vertex_corner.len() {
            let first = m.vertex_corner[v];
            if !m.hole[v] || first < 0 {
                continue;
            }
            let (mut c, mut act) = (first, m.swing_left(first));
            let mut guard = 0;
            while act >= 0 && act != first {
                c = act;
                act = m.swing_left(act);
                guard += 1;
                if guard > m.faces * 3 {
                    return bad("a fan that does not end");
                }
            }
            if act != first {
                m.vertex_corner[v] = c;
            }
        }
        for (prob, data) in &seam_data {
            seams.push(seams_of(m, *prob, data)?);
        }
        // Attribute_AssignPointsToCorners.
        let corners = m.faces * 3;
        corner_point = vec![u32::MAX; corners];
        let differs = |a: i32, b: i32| seams.iter().any(|s| s.ctv[a as usize] != s.ctv[b as usize]);
        for v in 0..m.vertex_corner.len() {
            let c = m.vertex_corner[v];
            if c < 0 {
                continue;
            }
            let mut first = c;
            if !m.hole[v] {
                'attrs: for s in &seams {
                    if !s.vertex[v] {
                        continue;
                    }
                    let id = s.ctv[c as usize];
                    let mut act = m.swing_right(c);
                    let mut guard = 0;
                    while act >= 0 && act != c {
                        if s.ctv[act as usize] != id {
                            first = act;
                            break 'attrs;
                        }
                        act = m.swing_right(act);
                        guard += 1;
                        if guard > corners {
                            return bad("a fan that does not end");
                        }
                    }
                }
            }
            corner_point[first as usize] = points as u32;
            points += 1;
            let mut before = first;
            let mut act = m.swing_right(first);
            let mut guard = 0;
            while act >= 0 && act != first {
                if differs(act, before) {
                    corner_point[act as usize] = points as u32;
                    points += 1;
                } else {
                    corner_point[act as usize] = corner_point[before as usize];
                }
                before = act;
                act = m.swing_right(act);
                guard += 1;
                if guard > corners {
                    return bad("a fan that does not end");
                }
            }
        }
        if corner_point.contains(&u32::MAX) {
            return bad("a corner no vertex reaches");
        }
        faces = corner_point.as_chunks::<3>().0.to_vec();
        for head in &heads {
            let view = View {
                m,
                s: view_of(m, &seams, *head)?.map(|k| &seams[k]),
            };
            sequences.push(sequence(&view, head.2 == 1)?);
        }
    } else {
        for _ in 0..decoders {
            sequences.push(Sequence {
                entry_corner: (0..points as i32).collect(),
                vertex_entry: Vec::new(),
            });
        }
    }
    // Each decoder's values a point: `indices_map_`.
    let mut point_entry: Vec<Vec<i32>> = Vec::with_capacity(decoders);
    for (i, seq) in sequences.iter().enumerate() {
        let Some(m) = &mesh else {
            point_entry.push((0..points as i32).collect());
            continue;
        };
        let view = View {
            m,
            s: view_of(m, &seams, heads[i])?.map(|k| &seams[k]),
        };
        let mut map = vec![-1i32; points];
        for c in 0..(m.faces * 3) as i32 {
            map[corner_point[c as usize] as usize] = seq.vertex_entry[view.vert(c)? as usize];
        }
        point_entry.push(map);
    }
    // The values.
    let mut out = Decoded {
        faces,
        points,
        attributes: Vec::new(),
    };
    let mut position_ints: Option<(Vec<i32>, usize)> = None;
    for (i, list) in attributes.iter().enumerate() {
        let seq = &sequences[i];
        let entries = seq.entry_corner.len();
        let view = match &mesh {
            Some(m) => Some(View {
                m,
                s: view_of(m, &seams, heads[i])?.map(|k| &seams[k]),
            }),
            None => None,
        };
        // For texture coordinates: each of this decoder's values' position.
        let positions = match (&position_ints, &mesh) {
            (Some((ints, dec)), Some(_)) => {
                let place = seq
                    .entry_corner
                    .iter()
                    .map(|&c| point_entry[*dec][corner_point[c as usize] as usize])
                    .collect();
                Some((ints.as_slice(), place))
            }
            _ => None,
        };
        let ctx = Ctx {
            view,
            seq,
            positions,
        };
        // DecodePortableAttributes.
        let mut portable: Vec<Vec<i32>> = Vec::with_capacity(list.len());
        let mut floats: Vec<Vec<f64>> = vec![Vec::new(); list.len()];
        for (j, a) in list.iter().enumerate() {
            match a.decoder {
                0 => {
                    // Stored outright, in the attribute's own type.
                    let size = integer_bytes(a.data_type)?;
                    let bytes = if size == 0 {
                        4
                    } else {
                        size.unsigned_abs() as usize
                    };
                    let total = entries
                        .checked_mul(bytes * a.components)
                        .ok_or("the Draco data is not valid: a count")?;
                    let b = r.bytes(total)?;
                    floats[j] = b
                        .chunks_exact(bytes)
                        .map(|c| {
                            let raw = c.iter().rev().fold(0u32, |v, x| (v << 8) | u32::from(*x));
                            match size {
                                0 => f64::from(f32::from_bits(raw)),
                                -1 => f64::from(raw as u8 as i8),
                                -2 => f64::from(raw as u16 as i16),
                                -4 => f64::from(raw as i32),
                                _ => f64::from(raw),
                            }
                        })
                        .collect();
                    portable.push(Vec::new());
                }
                1..=3 => {
                    let normals = a.decoder == 3;
                    let n = if normals { 2 } else { a.components };
                    portable.push(integers(&mut r, &ctx, entries, n, normals)?);
                }
                _ => return bad("an attribute encoder the specification does not have"),
            }
        }
        // DecodeDataNeededByPortableTransforms, and the values.
        for (j, a) in list.iter().enumerate() {
            let size = integer_bytes(a.data_type)?;
            let mut integer = size;
            let per_entry: Vec<f64> = match a.decoder {
                0 => {
                    if size == 0 {
                        integer = 0;
                    }
                    std::mem::take(&mut floats[j])
                }
                1 => portable[j].iter().map(|&v| f64::from(v)).collect(),
                2 => {
                    let mut min = Vec::with_capacity(a.components);
                    for _ in 0..a.components {
                        min.push(r.f32()?);
                    }
                    let range = r.f32()?;
                    let bits = u32::from(r.u8()?);
                    if bits == 0 || bits > 30 {
                        return bad("a quantization of no bits or more than 30");
                    }
                    let factor = 1.0f32 / ((1u32 << bits) - 1) as f32;
                    integer = 0;
                    if a.kind == 0 && position_ints.is_none() {
                        position_ints = Some((portable[j].clone(), i));
                    }
                    portable[j]
                        .iter()
                        .enumerate()
                        .map(|(k, &v)| {
                            // DequantizeFloat, in f32 as it is specified.
                            let norm = (v.unsigned_abs() as f32) * factor;
                            let norm = if v < 0 { -norm } else { norm };
                            f64::from(norm * range + min[k % a.components])
                        })
                        .collect()
                }
                _ => {
                    r.u8()?;
                    Vec::new()
                }
            };
            if a.kind == 0 && a.decoder == 1 && position_ints.is_none() {
                position_ints = Some((portable[j].clone(), i));
            }
            // By point.
            let n = a.components;
            let mut values = Vec::new();
            if !per_entry.is_empty() {
                if per_entry.len() != entries * n {
                    return bad("an attribute with fewer values than points");
                }
                values = vec![0.0; points * n];
                for (p, &e) in point_entry[i].iter().enumerate() {
                    if e < 0 || e as usize >= entries {
                        return bad("a point without a value");
                    }
                    let e = e as usize;
                    values[p * n..p * n + n].copy_from_slice(&per_entry[e * n..e * n + n]);
                }
            }
            out.attributes.push(Attribute {
                id: a.id,
                components: n,
                values,
                integer,
            });
        }
    }
    Ok(out)
}

/// The metadata, read past.
pub(crate) fn metadata(r: &mut Reader) -> Res<()> {
    fn element(r: &mut Reader, depth: usize) -> Res<()> {
        if depth > 64 {
            return bad("metadata nested too deep");
        }
        let entries = r.count(1)?;
        for _ in 0..entries {
            let n = usize::from(r.u8()?);
            r.bytes(n)?;
            let n = usize::from(r.u8()?);
            r.bytes(n)?;
        }
        let subs = r.count(1)?;
        for _ in 0..subs {
            let n = usize::from(r.u8()?);
            r.bytes(n)?;
            element(r, depth + 1)?;
        }
        Ok(())
    }
    let n = r.count(1)?;
    for _ in 0..n {
        r.var()?;
        element(r, 0)?;
    }
    element(r, 0)
}
