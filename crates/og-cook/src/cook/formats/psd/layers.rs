//! A PSD's layers as their pixels are shown: what the auto-rig reads a
//! character's parts from (`autorig.rs`).
//!
//! Read from the layer and mask section (or, in a 16-bit document, its
//! `Lr16` block): each pixel layer's name (its Unicode name when the file
//! has one), its rectangle on the canvas and its RGBA, with the layer's
//! opacity and fill opacity and its enabled layer mask multiplied into
//! the alpha, so a part is what Photoshop draws for it over nothing.
//! Groups give no pixels; a hidden group hides what it holds, and a
//! group's opacity multiplies its layers'.
//!
//! What cannot be shown that way is not dropped: a layer with layer
//! effects (a stroke, a shadow; even switched off), a blend mode other
//! than normal, a clipping mask, a vector mask, or that is an adjustment
//! or fill layer carries a [`Layer::refusal`] naming the fix, and the
//! auto-rig refuses a part with one. RGB and greyscale documents at 8 or
//! 16 bits a channel are read.

use super::*;

/// One layer of a PSD, bottom (drawn first) to top.
#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    pub name: String,
    pub kind: LayerKind,
    /// Its rectangle on the canvas, pixels from the top left.
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
    /// Its pixels as shown: RGBA8, straight alpha, `width * height`.
    pub rgba: Vec<u8>,
    /// Hidden itself or by a group it is in.
    pub hidden: bool,
    /// Why its pixels are not what Photoshop shows, and the fix.
    pub refusal: Option<String>,
    /// The groups it is in, outermost first.
    pub groups: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerKind {
    Pixels,
    /// A group (folder): its name; no pixels.
    Group,
}

/// The document's canvas and its layers, bottom to top.
#[derive(Clone, Debug, PartialEq)]
pub struct Layers {
    pub width: u32,
    pub height: u32,
    pub layers: Vec<Layer>,
}

/// Keys of additional layer information that make a layer an adjustment
/// or fill layer: its effect is on the layers under it.
const ADJUSTMENTS: [&[u8; 4]; 21] = [
    b"SoCo", b"GdFl", b"PtFl", b"brit", b"levl", b"curv", b"expA", b"vibA", b"hue ", b"hue2",
    b"blnc", b"blwh", b"phfl", b"mixr", b"clrL", b"nvrt", b"post", b"thrs", b"grdm", b"selc",
    b"CgEd",
];

/// Keys whose length is 8 bytes in a PSB.
const LONG_KEYS: [&[u8; 4]; 13] = [
    b"LMsk", b"Lr16", b"Lr32", b"Layr", b"Mt16", b"Mt32", b"Mtrn", b"Alph", b"FMsk", b"lnk2",
    b"FEid", b"FXid", b"PxSD",
];

struct Record {
    name: String,
    rect: [i32; 4],
    channels: Vec<(i16, u64)>,
    blend: [u8; 4],
    opacity: u8,
    clipping: u8,
    hidden: bool,
    mask: Option<Mask>,
    fill: u8,
    /// lsct: 1 or 2 opens a group (its own record), 3 closes one.
    section: u32,
    refusal: Option<&'static str>,
}

#[derive(Clone, Copy)]
struct Mask {
    rect: [i32; 4],
    default: u8,
    disabled: bool,
}

fn tagged<'a>(s: &mut Reader<'a>, large: bool) -> Result<([u8; 4], &'a [u8]), String> {
    let sig = s.take(4)?;
    if sig != b"8BIM" && sig != b"8B64" {
        return Err("PSD: a layer's tagged block has no 8BIM signature".into());
    }
    let key: [u8; 4] = s.take(4)?.try_into().expect("4");
    let len = if large && LONG_KEYS.contains(&&key) {
        s.u64()? as usize
    } else {
        s.u32()? as usize
    };
    Ok((key, s.take(len)?))
}

fn record(r: &mut Reader, large: bool) -> Result<Record, String> {
    let rect = [
        r.u32()? as i32,
        r.u32()? as i32,
        r.u32()? as i32,
        r.u32()? as i32,
    ];
    let n = r.u16()? as usize;
    let mut channels = Vec::with_capacity(n);
    for _ in 0..n {
        let id = r.u16()? as i16;
        let len = if large { r.u64()? } else { r.u32()? as u64 };
        channels.push((id, len));
    }
    if r.take(4)? != b"8BIM" {
        return Err("PSD: a layer record has no blend signature".into());
    }
    let blend: [u8; 4] = r.take(4)?.try_into().expect("4");
    let opacity = r.take(1)?[0];
    let clipping = r.take(1)?[0];
    let flags = r.take(1)?[0];
    r.take(1)?;
    let extra = r.u32()? as usize;
    let mut e = Reader {
        b: r.take(extra)?,
        at: 0,
    };
    let mask_len = e.u32()? as usize;
    let mask = {
        let mut m = Reader {
            b: e.take(mask_len)?,
            at: 0,
        };
        if mask_len >= 20 {
            let rect = [
                m.u32()? as i32,
                m.u32()? as i32,
                m.u32()? as i32,
                m.u32()? as i32,
            ];
            let default = m.take(1)?[0];
            let mflags = m.take(1)?[0];
            Some(Mask {
                rect,
                default,
                disabled: mflags & 2 != 0,
            })
        } else {
            None
        }
    };
    let ranges = e.u32()? as usize;
    e.take(ranges)?;
    let name_len = e.take(1)?[0] as usize;
    let name = String::from_utf8_lossy(e.take(name_len)?).to_string();
    let pad = (4 - (1 + name_len) % 4) % 4;
    e.take(pad)?;
    let mut rec = Record {
        name,
        rect,
        channels,
        blend,
        opacity,
        clipping,
        hidden: flags & 2 != 0,
        mask,
        fill: 255,
        section: 0,
        refusal: None,
    };
    while e.at + 12 <= e.b.len() {
        let (key, data) = tagged(&mut e, large)?;
        let mut d = Reader { b: data, at: 0 };
        match &key {
            b"luni" => {
                let n = d.u32()? as usize;
                let units: Vec<u16> = (0..n).map(|_| d.u16()).collect::<Result<_, _>>()?;
                rec.name = String::from_utf16_lossy(&units)
                    .trim_end_matches('\0')
                    .to_string();
            }
            b"lsct" | b"lsdk" => rec.section = d.u32()?,
            b"iOpa" => rec.fill = d.take(1)?[0],
            b"lrFX" | b"lfx2" | b"lmfx" => {
                rec.refusal = Some(
                    "has layer effects (a stroke, shadow, glow or overlay, even switched off); rasterize them into its pixels (Layer › Rasterize › Layer Style) or clear them (Layer › Layer Style › Clear Layer Style)",
                );
            }
            b"vmsk" | b"vsms" => {
                rec.refusal = Some(
                    "has a vector mask; rasterize it (Layer › Rasterize › Vector Mask) and apply it (Layer › Layer Mask › Apply)",
                );
            }
            k if ADJUSTMENTS.contains(&k) => {
                rec.refusal = Some(
                    "is an adjustment or fill layer, which changes the layers under it; merge it into the part it adjusts (Layer › Merge Down) or delete it",
                );
            }
            _ => {}
        }
    }
    Ok(rec)
}

/// A layer's record and its channels' 8-bit samples, in the record's order.
type Planes = (Record, Vec<Vec<u8>>);

/// Channel `len` bytes (its compression first) as `w * h` 8-bit samples.
fn channel(data: &[u8], w: usize, h: usize, depth: u16, large: bool) -> Result<Vec<u8>, String> {
    if w == 0 || h == 0 {
        return Ok(Vec::new());
    }
    let mut r = Reader { b: data, at: 0 };
    let compression = r.u16()?;
    let row_bytes = match depth {
        8 => w,
        16 => w * 2,
        d => {
            return Err(format!(
                "PSD: layers at {d} bits a channel are not read; save the document at 8 or 16 bits"
            ));
        }
    };
    let mut raw = Vec::with_capacity(row_bytes * h);
    match compression {
        0 => raw.extend_from_slice(r.take(row_bytes * h)?),
        1 => {
            let counts: Vec<usize> = (0..h)
                .map(|_| {
                    if large {
                        r.u32().map(|v| v as usize)
                    } else {
                        r.u16().map(|v| v as usize)
                    }
                })
                .collect::<Result<_, _>>()?;
            for c in counts {
                packbits(r.take(c)?, row_bytes, &mut raw)?;
            }
        }
        2 | 3 => {
            let all = miniz_oxide::inflate::decompress_to_vec_zlib(&data[2..])
                .map_err(|e| format!("PSD: zip: {e:?}"))?;
            raw.extend_from_slice(
                all.get(..row_bytes * h)
                    .ok_or("PSD: the zip data is short")?,
            );
            if compression == 3 {
                match depth {
                    8 => raw.chunks_exact_mut(row_bytes).for_each(|row| {
                        for i in 1..row.len() {
                            row[i] = row[i].wrapping_add(row[i - 1]);
                        }
                    }),
                    _ => raw.chunks_exact_mut(row_bytes).for_each(|row| {
                        for i in 1..w {
                            let p = u16::from_be_bytes([row[2 * i - 2], row[2 * i - 1]]);
                            let v = u16::from_be_bytes([row[2 * i], row[2 * i + 1]]);
                            row[2 * i..2 * i + 2].copy_from_slice(&p.wrapping_add(v).to_be_bytes());
                        }
                    }),
                }
            }
        }
        c => return Err(format!("PSD: compression {c} is not read")),
    }
    Ok(match depth {
        8 => raw,
        _ => raw.as_chunks::<2>().0.iter().map(|p| p[0]).collect(),
    })
}

/// The layer info block (`len`-prefixed by its caller): records, then pixels.
fn layer_info(b: &[u8], depth: u16, mode: u16, large: bool) -> Result<Vec<Planes>, String> {
    let mut r = Reader { b, at: 0 };
    if b.len() < 2 {
        return Ok(Vec::new());
    }
    let count = (r.u16()? as i16).unsigned_abs() as usize;
    let mut records = Vec::with_capacity(count);
    for _ in 0..count {
        records.push(record(&mut r, large)?);
    }
    let mut out = Vec::with_capacity(count);
    for rec in records {
        let mut planes = Vec::new();
        for &(id, len) in &rec.channels {
            let data = r.take(len as usize)?;
            let [t, l, bt, rt] = match (id, rec.mask) {
                (-2, Some(m)) => m.rect,
                _ => rec.rect,
            };
            let (w, h) = ((rt - l).max(0) as usize, (bt - t).max(0) as usize);
            // Only the channels the pixels are made of.
            let wanted = match mode {
                1 => matches!(id, -2..=0),
                _ => matches!(id, -2..=2),
            };
            planes.push(if wanted && len >= 2 {
                channel(data, w, h, depth, large)?
            } else {
                Vec::new()
            });
        }
        out.push((rec, planes));
    }
    Ok(out)
}

/// The layers of a PSD or PSB, bottom to top, as their pixels are shown.
pub fn layers(bytes: &[u8]) -> Result<Layers, String> {
    let mut r = Reader { b: bytes, at: 0 };
    if r.take(4)? != b"8BPS" {
        return Err("PSD: not a Photoshop file".into());
    }
    let large = match r.u16()? {
        1 => false,
        2 => true,
        v => return Err(format!("PSD: version {v} is not PSD (1) or PSB (2)")),
    };
    r.take(6)?;
    r.u16()?;
    let height = r.u32()?;
    let width = r.u32()?;
    let depth = r.u16()?;
    let mode = r.u16()?;
    if mode != 1 && mode != 3 {
        return Err(
            "PSD: the layers of a document that is not RGB or greyscale are not read; convert it (Image › Mode › RGB Color)".into(),
        );
    }
    let n = r.u32()? as usize;
    r.take(n)?;
    let n = r.u32()? as usize;
    r.take(n)?;
    let section = if large {
        r.u64()? as usize
    } else {
        r.u32()? as usize
    };
    let mut s = Reader {
        b: r.take(section)?,
        at: 0,
    };
    let mut raw = Vec::new();
    if section > 0 {
        let info = if large {
            s.u64()? as usize
        } else {
            s.u32()? as usize
        };
        raw = layer_info(s.take(info)?, depth, mode, large)?;
        // The global layer mask, then tagged blocks: a 16-bit document
        // keeps its layers in `Lr16`.
        if s.at + 4 <= s.b.len() {
            let g = s.u32()? as usize;
            s.take(g)?;
            while raw.is_empty() && s.at + 12 <= s.b.len() {
                let (key, data) = tagged(&mut s, large)?;
                if &key == b"Lr16" || &key == b"Lr32" {
                    raw = layer_info(data, depth, mode, large)?;
                }
            }
        }
    }
    // Groups: walked top to bottom, a group's record opens it and its
    // divider closes it.
    let mut stack: Vec<(String, bool, u8, Option<String>)> = Vec::new();
    let mut out: Vec<Layer> = Vec::new();
    for (rec, planes) in raw.into_iter().rev() {
        match rec.section {
            1 | 2 => {
                let hidden = rec.hidden || stack.iter().any(|g| g.1);
                let opacity = stack.last().map_or(255, |g| g.2) as u32 * rec.opacity as u32 / 255;
                out.push(Layer {
                    name: rec.name.clone(),
                    kind: LayerKind::Group,
                    left: 0,
                    top: 0,
                    width: 0,
                    height: 0,
                    rgba: Vec::new(),
                    hidden,
                    refusal: None,
                    groups: stack.iter().map(|g| g.0.clone()).collect(),
                });
                let blends = (&rec.blend != b"pass" && &rec.blend != b"norm").then(|| {
                    format!(
                        "is in the group `{}`, which blends with the mode `{}`; set the group to Pass Through or Normal, or merge it (Layer › Merge Group)",
                        rec.name,
                        String::from_utf8_lossy(&rec.blend).trim()
                    )
                });
                let blends = blends.or_else(|| stack.last().and_then(|g| g.3.clone()));
                stack.push((rec.name, hidden, opacity as u8, blends));
                continue;
            }
            3 => {
                stack.pop();
                continue;
            }
            _ => {}
        }
        let [t, l, b, rt] = rec.rect;
        let (w, h) = ((rt - l).max(0) as u32, (b - t).max(0) as u32);
        let plane = |id: i16| {
            rec.channels
                .iter()
                .position(|c| c.0 == id)
                .map(|i| &planes[i])
                .filter(|p| !p.is_empty())
        };
        let n = (w * h) as usize;
        let mut rgba = vec![0u8; n * 4];
        let group_opacity = stack.last().map_or(255, |g| g.2) as u32;
        let opacity = rec.opacity as u32 * rec.fill as u32 / 255 * group_opacity / 255;
        if n > 0 {
            let (rp, gp, bp) = if mode == 1 {
                (plane(0), plane(0), plane(0))
            } else {
                (plane(0), plane(1), plane(2))
            };
            let ap = plane(-1);
            let mask = rec.mask.filter(|m| !m.disabled).zip(plane(-2));
            for i in 0..n {
                let (x, y) = (l + (i as u32 % w) as i32, t + (i as u32 / w) as i32);
                let mut a = ap.map_or(255, |p| p[i]) as u32 * opacity / 255;
                if let Some((m, mp)) = mask {
                    let [mt, ml, mb, mr] = m.rect;
                    let mv = if x >= ml && x < mr && y >= mt && y < mb {
                        mp[((y - mt) * (mr - ml) + (x - ml)) as usize]
                    } else {
                        m.default
                    };
                    a = a * mv as u32 / 255;
                }
                rgba[i * 4] = rp.map_or(0, |p| p[i]);
                rgba[i * 4 + 1] = gp.map_or(0, |p| p[i]);
                rgba[i * 4 + 2] = bp.map_or(0, |p| p[i]);
                rgba[i * 4 + 3] = a as u8;
            }
        }
        let refusal = rec.refusal.map(str::to_string).or_else(|| stack.last().and_then(|g| g.3.clone())).or_else(|| {
            if rec.clipping != 0 {
                Some("is a clipping mask over the layer under it; merge it down into that layer (Layer › Merge Down)".into())
            } else if &rec.blend != b"norm" {
                Some(format!(
                    "blends with the mode `{}`, not Normal; set it to Normal, or merge it down into the part it shades",
                    String::from_utf8_lossy(&rec.blend).trim()
                ))
            } else {
                None
            }
        });
        out.push(Layer {
            name: rec.name,
            kind: LayerKind::Pixels,
            left: l,
            top: t,
            width: w,
            height: h,
            rgba,
            hidden: rec.hidden || stack.iter().any(|g| g.1),
            refusal,
            groups: stack.iter().map(|g| g.0.clone()).collect(),
        });
    }
    out.reverse();
    Ok(Layers {
        width,
        height,
        layers: out,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A layer to write: what Photoshop stores for it.
    #[derive(Clone)]
    pub(crate) struct Out {
        pub name: &'static str,
        /// top, left, bottom, right.
        pub rect: [i32; 4],
        pub rgba: Vec<[u8; 4]>,
        pub blend: [u8; 4],
        pub opacity: u8,
        pub clipping: u8,
        pub flags: u8,
        pub blocks: Vec<([u8; 4], Vec<u8>)>,
        /// rect and pixels of a layer mask.
        pub mask: Option<([i32; 4], Vec<u8>)>,
        /// Write the channels PackBits compressed (one literal run a row).
        pub rle: bool,
    }

    impl Out {
        pub(crate) fn new(name: &'static str, rect: [i32; 4], fill: [u8; 4]) -> Out {
            let n = ((rect[2] - rect[0]) * (rect[3] - rect[1])).max(0) as usize;
            Out {
                name,
                rect,
                rgba: vec![fill; n],
                blend: *b"norm",
                opacity: 255,
                clipping: 0,
                flags: 0,
                blocks: Vec::new(),
                mask: None,
                rle: false,
            }
        }
        pub(crate) fn section(name: &'static str, kind: u32) -> Out {
            let mut o = Out::new(name, [0; 4], [0; 4]);
            o.blocks.push((*b"lsct", kind.to_be_bytes().to_vec()));
            if kind != 3 {
                o.blend = *b"pass";
            }
            o
        }
    }

    fn plane(data: &[u8], w: usize, rle: bool) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend((rle as u16).to_be_bytes());
        if !rle || w == 0 {
            b.extend_from_slice(data);
            return b;
        }
        let rows: Vec<&[u8]> = data.chunks(w).collect();
        let mut body = Vec::new();
        for row in &rows {
            let mut enc = Vec::new();
            for chunk in row.chunks(128) {
                enc.push((chunk.len() - 1) as u8);
                enc.extend_from_slice(chunk);
            }
            b.extend((enc.len() as u16).to_be_bytes());
            body.extend(enc);
        }
        b.extend(body);
        b
    }

    /// An 8-bit RGB PSD of `w x h` holding `layers`, bottom first, with an
    /// all-clear composite.
    pub(crate) fn psd(w: u32, h: u32, layers: &[Out]) -> Vec<u8> {
        let mut recs = Vec::new();
        let mut pixels = Vec::new();
        for l in layers {
            let [_, le, _, r] = l.rect;
            let lw = (r - le).max(0) as usize;
            let mut chans: Vec<(i16, Vec<u8>)> = [-1i16, 0, 1, 2]
                .iter()
                .map(|&id| {
                    let c = if id < 0 { 3 } else { id as usize };
                    let data: Vec<u8> = l.rgba.iter().map(|p| p[c]).collect();
                    (id, plane(&data, lw, l.rle))
                })
                .collect();
            if let Some((mr, md)) = &l.mask {
                chans.push((-2, plane(md, (mr[3] - mr[1]) as usize, l.rle)));
            }
            for v in l.rect {
                recs.extend(v.to_be_bytes());
            }
            recs.extend((chans.len() as u16).to_be_bytes());
            for (id, d) in &chans {
                recs.extend(id.to_be_bytes());
                recs.extend((d.len() as u32).to_be_bytes());
                pixels.extend_from_slice(d);
            }
            recs.extend(b"8BIM");
            recs.extend(l.blend);
            recs.extend([l.opacity, l.clipping, l.flags, 0]);
            let mut extra = Vec::new();
            match &l.mask {
                Some((mr, _)) => {
                    extra.extend(20u32.to_be_bytes());
                    for v in mr {
                        extra.extend(v.to_be_bytes());
                    }
                    extra.extend([0, 0, 0, 0]);
                }
                None => extra.extend(0u32.to_be_bytes()),
            }
            extra.extend(0u32.to_be_bytes());
            let name = l.name.as_bytes();
            extra.push(name.len() as u8);
            extra.extend(name);
            while extra.len() % 4 != 0 {
                extra.push(0);
            }
            for (key, data) in &l.blocks {
                extra.extend(b"8BIM");
                extra.extend(key);
                extra.extend((data.len() as u32).to_be_bytes());
                extra.extend(data);
            }
            recs.extend((extra.len() as u32).to_be_bytes());
            recs.extend(extra);
        }
        let mut info = Vec::new();
        info.extend((-(layers.len() as i16)).to_be_bytes());
        info.extend(recs);
        info.extend(pixels);
        if info.len() % 2 == 1 {
            info.push(0);
        }
        let mut b = Vec::new();
        b.extend(b"8BPS");
        b.extend(1u16.to_be_bytes());
        b.extend([0; 6]);
        b.extend(4u16.to_be_bytes());
        b.extend(h.to_be_bytes());
        b.extend(w.to_be_bytes());
        b.extend(8u16.to_be_bytes());
        b.extend(3u16.to_be_bytes());
        b.extend(0u32.to_be_bytes());
        b.extend(0u32.to_be_bytes());
        b.extend(((info.len() + 8) as u32).to_be_bytes());
        b.extend((info.len() as u32).to_be_bytes());
        b.extend(info);
        b.extend(0u32.to_be_bytes());
        // The composite: raw, all clear.
        b.extend(0u16.to_be_bytes());
        b.extend(vec![0u8; (w * h * 4) as usize]);
        b
    }

    #[test]
    fn layers_keep_their_names_rects_pixels_and_order() {
        let mut arm = Out::new("arm-upper-l", [2, 1, 4, 4], [200, 10, 20, 255]);
        arm.rgba[0] = [1, 2, 3, 0];
        arm.rle = true;
        let mut head = Out::new("hed", [0, 0, 2, 2], [9, 9, 9, 255]);
        // A Unicode name wins over the Pascal one.
        let units: Vec<u16> = "head".encode_utf16().collect();
        let mut luni = (units.len() as u32).to_be_bytes().to_vec();
        units.iter().for_each(|u| luni.extend(u.to_be_bytes()));
        head.blocks.push((*b"luni", luni));
        let l = layers(&psd(8, 6, &[arm, head])).unwrap();
        assert_eq!((l.width, l.height), (8, 6));
        let names: Vec<_> = l.layers.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(names, ["arm-upper-l", "head"]);
        let a = &l.layers[0];
        assert_eq!((a.left, a.top, a.width, a.height), (1, 2, 3, 2));
        assert_eq!(&a.rgba[..8], &[1, 2, 3, 0, 200, 10, 20, 255]);
        assert!(a.refusal.is_none() && !a.hidden && a.groups.is_empty());
    }

    #[test]
    fn opacity_fill_and_mask_multiply_the_alpha() {
        let mut part = Out::new("torso", [0, 0, 1, 4], [50, 50, 50, 255]);
        part.opacity = 128;
        part.blocks.push((*b"iOpa", vec![255, 0, 0, 0]));
        // The mask covers the two right pixels; outside it, default 0.
        part.mask = Some(([0, 2, 1, 4], vec![255, 64]));
        let l = layers(&psd(4, 1, &[part])).unwrap();
        let alpha: Vec<u8> = l.layers[0].rgba.chunks(4).map(|p| p[3]).collect();
        assert_eq!(alpha, [0, 0, 128, 32]);
    }

    #[test]
    fn groups_name_their_layers_and_hide_them() {
        let mut group = Out::section("arm-l", 1);
        group.flags = 2;
        let ls = [
            Out::section("</Layer group>", 3),
            Out::new("hand-l", [0, 0, 1, 1], [1, 1, 1, 255]),
            group,
            Out::new("head", [0, 0, 1, 1], [1, 1, 1, 255]),
        ];
        let l = layers(&psd(1, 1, &ls)).unwrap();
        let hand = l.layers.iter().find(|l| l.name == "hand-l").unwrap();
        assert_eq!(hand.groups, ["arm-l"]);
        assert!(hand.hidden);
        assert!(!l.layers.iter().find(|l| l.name == "head").unwrap().hidden);
        let g = l.layers.iter().find(|l| l.name == "arm-l").unwrap();
        assert_eq!(g.kind, LayerKind::Group);
    }

    #[test]
    fn effects_blend_modes_and_clipping_are_named_not_dropped() {
        let mut fx = Out::new("head", [0, 0, 1, 1], [1, 1, 1, 255]);
        fx.blocks.push((*b"lfx2", vec![0; 8]));
        let mut mul = Out::new("torso", [0, 0, 1, 1], [1, 1, 1, 255]);
        mul.blend = *b"mul ";
        let mut clip = Out::new("shade", [0, 0, 1, 1], [1, 1, 1, 255]);
        clip.clipping = 1;
        let mut adj = Out::new("Curves 1", [0, 0, 0, 0], [0; 4]);
        adj.blocks.push((*b"curv", vec![0; 4]));
        let mut screen = Out::section("glow", 1);
        screen.blend = *b"scrn";
        let ls = [
            fx,
            mul,
            clip,
            adj,
            Out::section("", 3),
            Out::new("hair", [0, 0, 1, 1], [1; 4]),
            screen,
        ];
        let l = layers(&psd(1, 1, &ls)).unwrap();
        let hair = l.layers.iter().find(|l| l.name == "hair").unwrap();
        assert!(
            hair.refusal.as_deref().unwrap().contains("`glow`"),
            "{:?}",
            hair.refusal
        );
        let why: Vec<_> = l.layers[..4]
            .iter()
            .map(|l| l.refusal.clone().unwrap())
            .collect();
        assert!(
            why[0].contains("layer effects") && why[0].contains("Rasterize"),
            "{}",
            why[0]
        );
        assert!(
            why[1].contains("`mul`") && why[1].contains("Normal"),
            "{}",
            why[1]
        );
        assert!(why[2].contains("clipping mask"), "{}", why[2]);
        assert!(why[3].contains("adjustment"), "{}", why[3]);
    }
}
