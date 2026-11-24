//! MP4, the container ffmpeg writes H.264 and HEVC to for the cook (`-f
//! mp4`, no B-frames): its one video track's samples, each a frame in
//! presentation order with its sync flag, and the sample entry's decoder
//! configuration record (`avcC` or `hvcC`), which WebCodecs takes as
//! `VideoDecoderConfig.description`. The cook reads it once and keeps the
//! samples in the engine's own blob (og-bundle's `og.VideoStream`), as
//! [`crate::ivf`] does for AV1, so no container is read at run time.
//!
//! The boxes read are ISO/IEC 14496-12's: `moov/trak/mdia/minf/stbl` with
//! `stsd`, `stsz`, `stsc`, `stco` or `co64`, `stss` and `ctts`; ISO/IEC
//! 14496-15 defines the two records.

use crate::ivf::Unit;

/// The codecs an MP4's video track is read in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Codec {
    /// H.264 (`avc1`, an `avcC` record).
    H264,
    /// HEVC (`hvc1` or `hev1`, an `hvcC` record).
    Hevc,
}

/// An MP4's video track read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mp4 {
    pub codec: Codec,
    pub width: u32,
    pub height: u32,
    /// Its samples in order (`pts` is the sample's number).
    pub units: Vec<Unit>,
    /// The `avcC` or `hvcC` record's payload.
    pub config: Vec<u8>,
}

fn be(b: &[u8], at: usize, n: usize) -> Result<u64, String> {
    let s = b.get(at..at + n).ok_or("an MP4 box runs past its parent")?;
    Ok(s.iter().fold(0, |v, &x| (v << 8) | u64::from(x)))
}

/// The child boxes of `b[start..end]`: (type, payload range).
fn children(b: &[u8], start: usize, end: usize) -> Result<Vec<([u8; 4], usize, usize)>, String> {
    let mut out = Vec::new();
    let mut at = start;
    while at + 8 <= end {
        let mut size = be(b, at, 4)? as usize;
        let ty: [u8; 4] = b[at + 4..at + 8].try_into().expect("4 bytes");
        let mut head = 8;
        if size == 1 {
            size = be(b, at + 8, 8)? as usize;
            head = 16;
        } else if size == 0 {
            size = end - at;
        }
        if size < head || at + size > end {
            return Err(format!(
                "the MP4 box '{}' runs past its parent",
                String::from_utf8_lossy(&ty)
            ));
        }
        out.push((ty, at + head, at + size));
        at += size;
    }
    Ok(out)
}

fn child(
    b: &[u8],
    (start, end): (usize, usize),
    ty: &[u8; 4],
) -> Result<Option<(usize, usize)>, String> {
    Ok(children(b, start, end)?
        .into_iter()
        .find(|(t, _, _)| t == ty)
        .map(|(_, s, e)| (s, e)))
}

fn need(b: &[u8], parent: (usize, usize), ty: &[u8; 4]) -> Result<(usize, usize), String> {
    child(b, parent, ty)?
        .ok_or_else(|| format!("an MP4 with no '{}' box", String::from_utf8_lossy(ty)))
}

/// A full box's entries: `count` rows of `width` bytes after its version,
/// flags and count (and `skip` more bytes before the count).
fn entries(
    b: &[u8],
    (s, e): (usize, usize),
    skip: usize,
    width: usize,
) -> Result<(usize, usize), String> {
    let count = be(b, s + 4 + skip, 4)? as usize;
    let first = s + 8 + skip;
    if first + count * width > e {
        return Err("an MP4 table runs past its box".into());
    }
    Ok((first, count))
}

/// Reads an MP4's first video track of H.264 or HEVC.
pub fn read(bytes: &[u8]) -> Result<Mp4, String> {
    let b = bytes;
    let moov = need(b, (0, b.len()), b"moov")?;
    let mdia = children(b, moov.0, moov.1)?
        .into_iter()
        .filter(|(t, _, _)| t == b"trak")
        .find_map(|(_, s, e)| {
            let mdia = child(b, (s, e), b"mdia").ok()??;
            let hdlr = child(b, mdia, b"hdlr").ok()??;
            (b.get(hdlr.0 + 8..hdlr.0 + 12) == Some(b"vide")).then_some(mdia)
        })
        .ok_or("an MP4 with no video track")?;
    let stbl = need(b, need(b, mdia, b"minf")?, b"stbl")?;

    // The sample entry: a VisualSampleEntry (8 + 70 bytes) and its boxes.
    let stsd = need(b, stbl, b"stsd")?;
    let (ty, es, ee) = *children(b, stsd.0 + 8, stsd.1)?
        .first()
        .ok_or("an MP4 with no sample entry")?;
    let (codec, record) = match &ty {
        b"avc1" | b"avc3" => (Codec::H264, b"avcC"),
        b"hvc1" | b"hev1" => (Codec::Hevc, b"hvcC"),
        t => {
            return Err(format!(
                "an MP4 of '{}', not H.264 or HEVC",
                String::from_utf8_lossy(t)
            ));
        }
    };
    let width = be(b, es + 24, 2)? as u32;
    let height = be(b, es + 26, 2)? as u32;
    let rec = need(b, (es + 78, ee), record)?;
    let config = b[rec.0..rec.1].to_vec();

    // Sizes.
    let stsz = need(b, stbl, b"stsz")?;
    let fixed = be(b, stsz.0 + 4, 4)? as usize;
    let (first, n) = entries(b, stsz, 4, if fixed == 0 { 4 } else { 0 })?;
    let sizes: Vec<usize> = (0..n)
        .map(|i| {
            if fixed == 0 {
                be(b, first + i * 4, 4).map(|v| v as usize)
            } else {
                Ok(fixed)
            }
        })
        .collect::<Result<_, _>>()?;
    if sizes.is_empty() {
        return Err("an MP4 with no frames".into());
    }

    // Chunk offsets and the samples in each chunk.
    let chunks: Vec<usize> = if let Some(stco) = child(b, stbl, b"stco")? {
        let (f, c) = entries(b, stco, 0, 4)?;
        (0..c)
            .map(|i| be(b, f + i * 4, 4).map(|v| v as usize))
            .collect::<Result<_, _>>()?
    } else {
        let co64 = need(b, stbl, b"co64")?;
        let (f, c) = entries(b, co64, 0, 8)?;
        (0..c)
            .map(|i| be(b, f + i * 8, 8).map(|v| v as usize))
            .collect::<Result<_, _>>()?
    };
    let stsc = need(b, stbl, b"stsc")?;
    let (f, c) = entries(b, stsc, 0, 12)?;
    let runs: Vec<(usize, usize)> = (0..c)
        .map(|i| {
            Ok((
                be(b, f + i * 12, 4)? as usize,
                be(b, f + i * 12 + 4, 4)? as usize,
            ))
        })
        .collect::<Result<_, String>>()?;
    let mut offsets = Vec::with_capacity(sizes.len());
    for (ci, &chunk) in chunks.iter().enumerate() {
        // The last run that starts at or before this chunk (1-based).
        let per = runs
            .iter()
            .rev()
            .find(|(first, _)| *first <= ci + 1)
            .map_or(0, |r| r.1);
        let mut at = chunk;
        for _ in 0..per {
            let Some(&size) = sizes.get(offsets.len()) else {
                break;
            };
            offsets.push(at);
            at += size;
        }
    }
    if offsets.len() != sizes.len() {
        return Err(format!(
            "an MP4 whose chunks hold {} samples of its {}",
            offsets.len(),
            sizes.len()
        ));
    }

    // Sync samples (every one without `stss`); no reordering.
    let sync: Option<Vec<usize>> = match child(b, stbl, b"stss")? {
        Some(stss) => {
            let (f, c) = entries(b, stss, 0, 4)?;
            Some(
                (0..c)
                    .map(|i| be(b, f + i * 4, 4).map(|v| v as usize))
                    .collect::<Result<_, _>>()?,
            )
        }
        None => None,
    };
    if let Some(ctts) = child(b, stbl, b"ctts")? {
        let (f, c) = entries(b, ctts, 0, 8)?;
        let first = be(b, f + 4, 4)?;
        for i in 0..c {
            if be(b, f + i * 8 + 4, 4)? != first {
                return Err("an MP4 with B-frames (frames out of presentation order); encode with no B-frames".into());
            }
        }
    }

    let units = sizes
        .iter()
        .zip(&offsets)
        .enumerate()
        .map(|(i, (&size, &at))| {
            let data = b
                .get(at..at + size)
                .ok_or_else(|| format!("sample {i} runs past the file"))?
                .to_vec();
            let key = sync
                .as_ref()
                .is_none_or(|s| s.binary_search(&(i + 1)).is_ok());
            Ok(Unit {
                pts: i as u64,
                key,
                data,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    if !units[0].key {
        return Err("an MP4 whose first frame is not a sync sample".into());
    }
    Ok(Mp4 {
        codec,
        width,
        height,
        units,
        config,
    })
}

/// The codec string WebCodecs and `MediaSource` take for a stream of
/// `codec` with this configuration record (RFC 6381 for `avc1`, ISO/IEC
/// 14496-15 Annex E for `hvc1`).
pub fn codec_string(codec: Codec, config: &[u8]) -> Result<String, String> {
    match codec {
        Codec::H264 => {
            let r = config
                .get(1..4)
                .ok_or("an avcC record shorter than its header")?;
            Ok(format!("avc1.{:02X}{:02X}{:02X}", r[0], r[1], r[2]))
        }
        Codec::Hevc => {
            let r = config
                .get(..13)
                .ok_or("an hvcC record shorter than its header")?;
            let space = ["", "A", "B", "C"][usize::from(r[1] >> 6)];
            let tier = if r[1] & 0x20 != 0 { 'H' } else { 'L' };
            let compat = u32::from_be_bytes([r[2], r[3], r[4], r[5]]).reverse_bits();
            let mut s = format!("hvc1.{space}{}.{compat:X}.{tier}{}", r[1] & 0x1f, r[12]);
            let flags = &r[6..12];
            let used = flags.iter().rposition(|&x| x != 0).map_or(0, |i| i + 1);
            for x in &flags[..used] {
                s.push_str(&format!(".{x:02X}"));
            }
            Ok(s)
        }
    }
}
