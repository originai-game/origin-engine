//! Attribute values: integers decoded by their prediction scheme (parallelogram, wrap) and dequantized.

use super::*;

/// What a prediction needs of the mesh: for each value in order, its
/// corner; each vertex's place; the decoder's view of the corners.
pub(crate) struct Ctx<'a> {
    pub(crate) view: Option<View<'a>>,
    pub(crate) seq: &'a Sequence,
    /// The positions as whole numbers, by their own order, and for each
    /// of this decoder's values the position's place.
    pub(crate) positions: Option<(&'a [i32], Vec<i32>)>,
}

/// The prediction transform: wrap (within `min..=max`), or none.
#[derive(Clone, Copy)]
pub(crate) struct Wrap {
    pub(crate) min: i32,
    pub(crate) max: i32,
    pub(crate) on: bool,
}

impl Wrap {
    pub(crate) fn original(&self, predicted: i32, correction: i32) -> i32 {
        if !self.on {
            return predicted.wrapping_add(correction);
        }
        let mut v = predicted
            .clamp(self.min, self.max.max(self.min))
            .wrapping_add(correction);
        let dif = 1i32.wrapping_add(self.max).wrapping_sub(self.min);
        if v > self.max {
            v = v.wrapping_sub(dif);
        } else if v < self.min {
            v = v.wrapping_add(dif);
        }
        v
    }
}

/// The parallelogram across the edge opposite corner `ci`, if its three
/// values are already there.
pub(crate) fn parallelogram(
    ctx: &Ctx,
    entry: usize,
    ci: i32,
    data: &[i32],
    n: usize,
    out: &mut [i32],
) -> Res<bool> {
    let Some(view) = &ctx.view else {
        return Ok(false);
    };
    let oci = view.opposite(ci);
    if oci < 0 {
        return Ok(false);
    }
    let at = |c: i32| -> Res<usize> {
        let e = ctx.seq.vertex_entry[view.vert(c)? as usize];
        Ok(if e < 0 { usize::MAX } else { e as usize })
    };
    let (o, a, b) = (at(oci)?, at(next(oci))?, at(prev(oci))?);
    if o < entry && a < entry && b < entry {
        for c in 0..n {
            out[c] = data[a * n + c]
                .wrapping_add(data[b * n + c])
                .wrapping_sub(data[o * n + c]);
        }
        return Ok(true);
    }
    Ok(false)
}

/// IntSqrt.
pub(crate) fn int_sqrt(number: u64) -> u64 {
    if number == 0 {
        return 0;
    }
    let mut act = number;
    let mut root = 1u64;
    while act >= 2 {
        root *= 2;
        act /= 4;
    }
    loop {
        root = (root + number / root) / 2;
        if root.saturating_mul(root) <= number {
            return root;
        }
    }
}

/// One attribute's whole numbers, undone of their prediction.
#[allow(clippy::too_many_arguments)]
pub(crate) fn integers(
    r: &mut Reader,
    ctx: &Ctx,
    entries: usize,
    n: usize,
    normals: bool,
) -> Res<Vec<i32>> {
    let scheme = r.u8()? as i8;
    let mut transform = 0i8;
    if scheme != -2 {
        transform = r.u8()? as i8;
    }
    let values = entries * n;
    let compressed = r.u8()?;
    let mut raw: Vec<u32> = if compressed > 0 {
        symbols(r, values, n)?
    } else {
        // Stored outright, in a number of bytes each.
        let bytes = usize::from(r.u8()?);
        if bytes == 0 || bytes > 4 {
            return bad("whole numbers of no bytes or more than four");
        }
        let b = r.bytes(
            values
                .checked_mul(bytes)
                .ok_or("the Draco data is not valid: a count")?,
        )?;
        b.chunks_exact(bytes)
            .map(|c| c.iter().rev().fold(0u32, |v, x| (v << 8) | u32::from(*x)))
            .collect()
    };
    if raw.len() < values {
        return short();
    }
    raw.truncate(values);
    // Corrections are signed but under the normals' own transform.
    let signed: Vec<i32> = if transform == 3 {
        raw.iter().map(|&v| v as i32).collect()
    } else {
        raw.iter()
            .map(|&v| {
                let half = (v >> 1) as i32;
                if v & 1 == 0 { half } else { -half - 1 }
            })
            .collect()
    };
    if scheme == -2 {
        return Ok(signed);
    }
    // The prediction's own data, then its transform's.
    let mut creases: [Vec<bool>; 4] = Default::default();
    let mut orientations: Vec<bool> = Vec::new();
    let mut wrap = Wrap {
        min: 0,
        max: 0,
        on: false,
    };
    let rans_bits = |r: &mut Reader, count: usize| -> Res<Vec<bool>> {
        let prob = r.u8()?;
        let size = r.count(1)?;
        let mut ans = Ans::new(r.bytes(size)?, L_RANS_BASE)?;
        Ok((0..count).map(|_| ans.bit(prob)).collect())
    };
    let mut read_transform = |r: &mut Reader| -> Res<()> {
        match transform {
            1 => {
                wrap = Wrap {
                    min: r.i32()?,
                    max: r.i32()?,
                    on: true,
                };
            }
            3 => {
                r.i32()?;
                r.i32()?;
            }
            _ => {}
        }
        Ok(())
    };
    match scheme {
        4 => {
            for flags in &mut creases {
                let count = r.var()? as usize;
                if count > entries.saturating_mul(4) {
                    return bad("more crease flags than values");
                }
                if count > 0 {
                    *flags = rans_bits(r, count)?;
                }
            }
            read_transform(r)?;
        }
        5 => {
            let count = r.u32()? as usize;
            if count > entries {
                return bad("more orientations than values");
            }
            let bits = rans_bits(r, count)?;
            let mut last = true;
            for b in bits {
                if !b {
                    last = !last;
                }
                orientations.push(last);
            }
            read_transform(r)?;
        }
        6 => {
            read_transform(r)?;
            rans_bits(r, entries)?;
        }
        0 | 1 => read_transform(r)?,
        _ => return bad("a prediction scheme the specification does not have"),
    }
    if normals || values == 0 {
        // Parsed; not computed.
        return Ok(Vec::new());
    }
    let mesh_scheme = scheme != 0;
    if mesh_scheme && ctx.view.is_none() {
        return bad("a mesh prediction without EdgeBreaker connectivity");
    }
    let mut out = signed.clone();
    let mut pred = vec![0i32; n];
    let put = |out: &mut Vec<i32>, p: usize, pred: &[i32]| {
        for c in 0..n {
            out[p * n + c] = wrap.original(pred[c], signed[p * n + c]);
        }
    };
    let before = |out: &Vec<i32>, p: usize| -> Vec<i32> { out[(p - 1) * n..p * n].to_vec() };
    match scheme {
        0 => {
            for p in 0..entries {
                let from = if p == 0 { vec![0; n] } else { before(&out, p) };
                put(&mut out, p, &from);
            }
        }
        1 => {
            put(&mut out, 0, &pred);
            for p in 1..entries {
                let corner = ctx.seq.entry_corner[p];
                if parallelogram(ctx, p, corner, &out, n, &mut pred)? {
                    let q = pred.clone();
                    put(&mut out, p, &q);
                } else {
                    let from = before(&out, p);
                    put(&mut out, p, &from);
                }
            }
        }
        4 => {
            let view = ctx.view.as_ref().ok_or("no view")?;
            put(&mut out, 0, &pred);
            let mut preds = vec![vec![0i32; n]; 4];
            let mut used_flags = [0usize; 4];
            for p in 1..entries {
                let start = ctx.seq.entry_corner[p];
                let mut corner = start;
                let mut count = 0usize;
                let mut first_pass = true;
                let mut guard = 0;
                while corner >= 0 {
                    if parallelogram(ctx, p, corner, &out, n, &mut preds[count])? {
                        count += 1;
                        if count == 4 {
                            break;
                        }
                    }
                    corner = if first_pass {
                        view.swing_left(corner)
                    } else {
                        view.swing_right(corner)
                    };
                    if corner == start {
                        break;
                    }
                    if corner < 0 && first_pass {
                        first_pass = false;
                        corner = view.swing_right(start);
                    }
                    guard += 1;
                    if guard > view.m.faces * 3 {
                        return bad("a fan that does not end");
                    }
                }
                let mut sum = vec![0i32; n];
                let mut used = 0i32;
                for one in preds.iter().take(count) {
                    let context = count - 1;
                    let Some(&crease) = creases[context].get(used_flags[context]) else {
                        return bad("a parallelogram without its crease flag");
                    };
                    used_flags[context] += 1;
                    if !crease {
                        used += 1;
                        for c in 0..n {
                            sum[c] = sum[c].wrapping_add(one[c]);
                        }
                    }
                }
                if used == 0 {
                    let from = before(&out, p);
                    put(&mut out, p, &from);
                } else {
                    for v in &mut sum {
                        *v /= used;
                    }
                    put(&mut out, p, &sum);
                }
            }
        }
        5 => {
            let view = ctx.view.as_ref().ok_or("no view")?;
            let Some((positions, place)) = &ctx.positions else {
                return bad("texture coordinates predicted without positions");
            };
            if n != 2 {
                return bad("texture coordinates that are not pairs");
            }
            let pos = |entry: usize| -> Res<[i64; 3]> {
                let k = *place
                    .get(entry)
                    .ok_or("the Draco data is not valid: a position")?;
                let Some(p) = positions.get(k.max(0) as usize * 3..k.max(0) as usize * 3 + 3)
                else {
                    return bad("a position outside the mesh");
                };
                Ok([i64::from(p[0]), i64::from(p[1]), i64::from(p[2])])
            };
            for p in 0..entries {
                let corner = ctx.seq.entry_corner[p];
                let entry_of = |c: i32| -> Res<usize> {
                    let e = ctx.seq.vertex_entry[view.vert(c)? as usize];
                    Ok(if e < 0 { usize::MAX } else { e as usize })
                };
                let (ne, pe) = (entry_of(next(corner))?, entry_of(prev(corner))?);
                let uv =
                    |out: &Vec<i32>, e: usize| [i64::from(out[e * 2]), i64::from(out[e * 2 + 1])];
                let mut predicted: Option<[i32; 2]> = None;
                if pe < p && ne < p {
                    let (n_uv, p_uv) = (uv(&out, ne), uv(&out, pe));
                    if p_uv == n_uv {
                        predicted = Some([p_uv[0] as i32, p_uv[1] as i32]);
                    } else {
                        let (tip, np, pp) = (pos(p)?, pos(ne)?, pos(pe)?);
                        let sub =
                            |a: [i64; 3], b: [i64; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
                        let dot = |a: [i64; 3], b: [i64; 3]| {
                            a[0].wrapping_mul(b[0])
                                .wrapping_add(a[1].wrapping_mul(b[1]))
                                .wrapping_add(a[2].wrapping_mul(b[2]))
                        };
                        let pn = sub(pp, np);
                        let pn2 = dot(pn, pn);
                        if pn2 != 0 {
                            let cn = sub(tip, np);
                            let cn_dot_pn = dot(cn, pn);
                            let pn_uv = [p_uv[0] - n_uv[0], p_uv[1] - n_uv[1]];
                            let x_uv = [
                                pn_uv[0]
                                    .wrapping_mul(cn_dot_pn)
                                    .wrapping_add(n_uv[0].wrapping_mul(pn2)),
                                pn_uv[1]
                                    .wrapping_mul(cn_dot_pn)
                                    .wrapping_add(n_uv[1].wrapping_mul(pn2)),
                            ];
                            let x_pos = [
                                np[0] + pn[0].wrapping_mul(cn_dot_pn) / pn2,
                                np[1] + pn[1].wrapping_mul(cn_dot_pn) / pn2,
                                np[2] + pn[2].wrapping_mul(cn_dot_pn) / pn2,
                            ];
                            let cx = sub(tip, x_pos);
                            let cx2 = dot(cx, cx);
                            let norm = int_sqrt((cx2 as u64).wrapping_mul(pn2 as u64)) as i64;
                            let cx_uv =
                                [pn_uv[1].wrapping_mul(norm), (-pn_uv[0]).wrapping_mul(norm)];
                            let Some(orientation) = orientations.pop() else {
                                return bad("a texture coordinate without its side");
                            };
                            let t = if orientation {
                                [
                                    x_uv[0].wrapping_add(cx_uv[0]),
                                    x_uv[1].wrapping_add(cx_uv[1]),
                                ]
                            } else {
                                [
                                    x_uv[0].wrapping_sub(cx_uv[0]),
                                    x_uv[1].wrapping_sub(cx_uv[1]),
                                ]
                            };
                            predicted = Some([(t[0] / pn2) as i32, (t[1] / pn2) as i32]);
                        }
                    }
                }
                let predicted = match predicted {
                    Some(v) => v,
                    None => {
                        // The value at the neighbour already there, or
                        // the one before, or nothing.
                        let from = if ne < p {
                            Some(ne)
                        } else if pe < p {
                            Some(pe)
                        } else if p > 0 {
                            Some(p - 1)
                        } else {
                            None
                        };
                        from.map_or([0, 0], |e| [out[e * 2], out[e * 2 + 1]])
                    }
                };
                put(&mut out, p, &predicted);
            }
        }
        _ => return bad("a prediction scheme the specification does not have"),
    }
    Ok(out)
}

/// The bytes of a Draco data type; negative for signed, 0 for floats.
pub(crate) fn integer_bytes(data_type: u8) -> Res<i8> {
    Ok(match data_type {
        1 => -1,
        2 | 11 => 1,
        3 => -2,
        4 => 2,
        5 => -4,
        6 => 4,
        9 => 0,
        _ => return bad("an attribute of 64-bit numbers"),
    })
}
