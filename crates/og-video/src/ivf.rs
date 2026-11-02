//! IVF, the container ffmpeg writes AV1 to for the cook (`-f ivf`): a
//! 32-byte header (`DKIF`, the fourcc `AV01`, the size, the time base),
//! then each temporal unit behind a 12-byte header (its size and pts). The
//! cook reads it once and keeps the units in the engine's own blob
//! (og-bundle's `og.VideoStream`), so no container is read at run time.

/// One temporal unit: what decodes to one shown frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unit {
    /// Its presentation time in the stream's time base.
    pub pts: u64,
    /// Whether decoding may start here: a key frame shown at once.
    pub key: bool,
    pub data: Vec<u8>,
}

/// An IVF file read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ivf {
    pub width: u32,
    pub height: u32,
    /// The time base, `rate` units per `scale` seconds.
    pub rate: u32,
    pub scale: u32,
    pub units: Vec<Unit>,
    /// The sequence header OBU (the first unit's), the decoder's config.
    pub sequence_header: Vec<u8>,
}

fn u16le(b: &[u8], at: usize) -> u32 {
    u32::from(u16::from_le_bytes([b[at], b[at + 1]]))
}

fn u32le(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

/// Reads an IVF file of AV1.
pub fn read(bytes: &[u8]) -> Result<Ivf, String> {
    if bytes.len() < 32 || &bytes[0..4] != b"DKIF" {
        return Err("not an IVF file (no DKIF header)".into());
    }
    if &bytes[8..12] != b"AV01" {
        return Err(format!(
            "an IVF of '{}', not AV1",
            String::from_utf8_lossy(&bytes[8..12])
        ));
    }
    let header = u16le(bytes, 6) as usize;
    let (width, height) = (u16le(bytes, 12), u16le(bytes, 14));
    let (rate, scale) = (u32le(bytes, 16), u32le(bytes, 20));
    if rate == 0 || scale == 0 {
        return Err("an IVF with no time base".into());
    }
    let mut at = header.max(32);
    let mut units = Vec::new();
    while at + 12 <= bytes.len() {
        let size = u32le(bytes, at) as usize;
        let pts = u64::from(u32le(bytes, at + 4)) | (u64::from(u32le(bytes, at + 8)) << 32);
        at += 12;
        let data = bytes
            .get(at..at + size)
            .ok_or_else(|| format!("unit {} runs past the file", units.len()))?
            .to_vec();
        at += size;
        units.push(Unit {
            pts,
            key: false,
            data,
        });
    }
    if units.is_empty() {
        return Err("an IVF with no frames".into());
    }
    let sequence_header = obus(&units[0].data)?
        .into_iter()
        .find(|o| o.ty == OBU_SEQUENCE_HEADER)
        .map(|o| units[0].data[o.start..o.end].to_vec())
        .ok_or("the first frame carries no sequence header")?;
    let reduced = reduced_still_picture(&sequence_header)?;
    for u in &mut units {
        u.key = is_key(&u.data, reduced)?;
    }
    if !units[0].key {
        return Err("the first frame is not a key frame".into());
    }
    Ok(Ivf {
        width,
        height,
        rate,
        scale,
        units,
        sequence_header,
    })
}

const OBU_SEQUENCE_HEADER: u8 = 1;
const OBU_FRAME_HEADER: u8 = 3;
const OBU_FRAME: u8 = 6;

/// An OBU's type and where it lies (header included) and where its payload starts.
struct Obu {
    ty: u8,
    start: usize,
    payload: usize,
    end: usize,
}

fn leb128(b: &[u8], at: &mut usize) -> Result<u64, String> {
    let mut v = 0u64;
    for i in 0..8 {
        let byte = *b.get(*at).ok_or("an OBU size runs past its unit")?;
        *at += 1;
        v |= u64::from(byte & 0x7f) << (7 * i);
        if byte & 0x80 == 0 {
            return Ok(v);
        }
    }
    Err("an OBU size longer than 8 bytes".into())
}

/// A temporal unit's OBUs (the low-overhead format: each has its size).
fn obus(b: &[u8]) -> Result<Vec<Obu>, String> {
    let mut out = Vec::new();
    let mut at = 0;
    while at < b.len() {
        let start = at;
        let h = b[at];
        at += 1;
        if h & 0x80 != 0 {
            return Err("an OBU with its forbidden bit set".into());
        }
        if h & 0x04 != 0 {
            at += 1; // the extension byte
        }
        if h & 0x02 == 0 {
            return Err("an OBU without a size (not the low-overhead format)".into());
        }
        let size = leb128(b, &mut at)? as usize;
        let payload = at;
        at += size;
        if at > b.len() {
            return Err("an OBU runs past its unit".into());
        }
        out.push(Obu {
            ty: (h >> 3) & 0x0f,
            start,
            payload,
            end: at,
        });
    }
    Ok(out)
}

/// Whether the sequence header says `reduced_still_picture_header`.
fn reduced_still_picture(seq: &[u8]) -> Result<bool, String> {
    let o = obus(seq)?;
    let first = o.first().ok_or("an empty sequence header")?;
    let byte = *seq.get(first.payload).ok_or("an empty sequence header")?;
    // seq_profile (3 bits), still_picture (1), reduced_still_picture_header (1).
    Ok(byte & 0x08 != 0)
}

/// Whether a unit holds a key frame that is shown: its first frame
/// header's `show_existing_frame` 0 and `frame_type` KEY_FRAME, and
/// `show_frame` 1 (a hidden key frame is not a place to start).
fn is_key(unit: &[u8], reduced: bool) -> Result<bool, String> {
    if reduced {
        return Ok(true);
    }
    for o in obus(unit)? {
        if o.ty == OBU_FRAME || o.ty == OBU_FRAME_HEADER {
            let Some(&b) = unit.get(o.payload) else {
                return Ok(false);
            };
            // show_existing_frame (1 bit), frame_type (2), show_frame (1).
            let show_existing = b & 0x80 != 0;
            let frame_type = (b >> 5) & 0x03;
            let show = b & 0x10 != 0;
            return Ok(!show_existing && frame_type == 0 && show);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An IVF of `units` (each a list of (OBU type, payload)).
    fn ivf(units: &[Vec<(u8, Vec<u8>)>]) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(b"DKIF");
        b.extend_from_slice(&0u16.to_le_bytes());
        b.extend_from_slice(&32u16.to_le_bytes());
        b.extend_from_slice(b"AV01");
        b.extend_from_slice(&64u16.to_le_bytes());
        b.extend_from_slice(&36u16.to_le_bytes());
        b.extend_from_slice(&30u32.to_le_bytes());
        b.extend_from_slice(&1u32.to_le_bytes());
        b.extend_from_slice(&(units.len() as u32).to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        for (pts, u) in units.iter().enumerate() {
            let mut data = Vec::new();
            for (ty, payload) in u {
                data.push((ty << 3) | 0x02);
                data.push(payload.len() as u8);
                data.extend_from_slice(payload);
            }
            b.extend_from_slice(&(data.len() as u32).to_le_bytes());
            b.extend_from_slice(&(pts as u64).to_le_bytes());
            b.extend_from_slice(&data);
        }
        b
    }

    #[test]
    fn reads_units_and_finds_shown_key_frames() {
        let seq = (OBU_SEQUENCE_HEADER, vec![0x00, 0x00]);
        let td = (2u8, vec![]);
        // Key and shown; inter; key but hidden; a shown existing frame.
        let bytes = ivf(&[
            vec![td.clone(), seq.clone(), (OBU_FRAME, vec![0b0001_0000, 1])],
            vec![td.clone(), (OBU_FRAME, vec![0b0011_0000, 2])],
            vec![td.clone(), (OBU_FRAME_HEADER, vec![0b0000_0000])],
            vec![td.clone(), (OBU_FRAME_HEADER, vec![0b1000_0000])],
        ]);
        let f = read(&bytes).unwrap();
        assert_eq!((f.width, f.height, f.rate, f.scale), (64, 36, 30, 1));
        let keys: Vec<bool> = f.units.iter().map(|u| u.key).collect();
        assert_eq!(keys, [true, false, false, false]);
        assert_eq!(f.units[1].pts, 1);
        assert_eq!(f.sequence_header, vec![0x0a, 2, 0, 0]);
        // A first frame that is not a key frame is refused.
        let bad = ivf(&[vec![seq, (OBU_FRAME, vec![0b0011_0000])]]);
        assert!(read(&bad).unwrap_err().contains("not a key frame"));
        assert!(read(b"RIFF").unwrap_err().contains("DKIF"));
    }
}
