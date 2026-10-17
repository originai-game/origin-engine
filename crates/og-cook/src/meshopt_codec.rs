//! Pure-Rust decoders for meshoptimizer's vertex codec (versions 0 and 1)
//! and index codec (versions 0 and 1), ported from the scalar paths of
//! meshoptimizer 0.25 (`vertexcodec.cpp`, `indexcodec.cpp`, MIT). The cook
//! encodes with the C library; the runtime decodes here so the wasm build
//! needs no C toolchain. Malformed input is an error value, never a panic.

const VERTEX_HEADER: u8 = 0xa0;
const INDEX_HEADER: u8 = 0xe0;
const VERTEX_BLOCK_SIZE_BYTES: usize = 8192;
const VERTEX_BLOCK_MAX_SIZE: usize = 256;
const BYTE_GROUP_SIZE: usize = 16;
const BYTE_GROUP_DECODE_LIMIT: usize = 24;
const TAIL_MIN_SIZE_V0: usize = 32;
const TAIL_MIN_SIZE_V1: usize = 24;
const BITS_V0: [u32; 4] = [0, 2, 4, 8];
const BITS_V1: [u32; 5] = [0, 1, 2, 4, 8];

fn vertex_block_size(vertex_size: usize) -> usize {
    let result = (VERTEX_BLOCK_SIZE_BYTES / vertex_size) & !(BYTE_GROUP_SIZE - 1);
    result.min(VERTEX_BLOCK_MAX_SIZE)
}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
    end: usize,
}

impl Reader<'_> {
    fn left(&self) -> usize {
        self.end - self.pos
    }
}

/// Decodes one 16-byte group of `bits`-bit values into `out`.
fn decode_bytes_group(r: &mut Reader, out: &mut [u8], bits: u32) {
    match bits {
        0 => out[..BYTE_GROUP_SIZE].fill(0),
        8 => {
            out[..BYTE_GROUP_SIZE].copy_from_slice(&r.data[r.pos..r.pos + BYTE_GROUP_SIZE]);
            r.pos += BYTE_GROUP_SIZE;
        }
        1 | 2 | 4 => {
            let header_bytes = (BYTE_GROUP_SIZE * bits as usize) / 8;
            let mut var = r.pos + header_bytes;
            let per_byte = 8 / bits;
            let sentinel = (1u32 << bits) - 1;
            let mut k = 0;
            for hb in 0..header_bytes {
                let mut byte = r.data[r.pos + hb] as u32;
                if bits == 1 {
                    // 1-bit groups store their bits reversed within each byte.
                    byte = byte.reverse_bits() >> 24;
                }
                for _ in 0..per_byte {
                    let enc = (byte >> (8 - bits)) & sentinel;
                    byte = (byte << bits) & 0xff;
                    if enc == sentinel {
                        out[k] = r.data[var];
                        var += 1;
                    } else {
                        out[k] = enc as u8;
                    }
                    k += 1;
                }
            }
            r.pos = var;
        }
        _ => unreachable!("bit widths come from the codec tables"),
    }
}

fn decode_bytes(r: &mut Reader, buffer: &mut [u8], bits: &[u32]) -> Option<()> {
    debug_assert!(buffer.len().is_multiple_of(BYTE_GROUP_SIZE));
    let groups = buffer.len() / BYTE_GROUP_SIZE;
    let header_size = groups.div_ceil(4);
    if r.left() < header_size {
        return None;
    }
    let header = r.pos;
    r.pos += header_size;
    for g in 0..groups {
        if r.left() < BYTE_GROUP_DECODE_LIMIT {
            return None;
        }
        let bitsk = (r.data[header + g / 4] >> ((g % 4) * 2)) & 3;
        decode_bytes_group(
            r,
            &mut buffer[g * BYTE_GROUP_SIZE..],
            *bits.get(bitsk as usize)?,
        );
    }
    Some(())
}

fn unzigzag(v: u32) -> u32 {
    (0u32.wrapping_sub(v & 1)) ^ (v >> 1)
}

/// `decodeDeltas1` for one 4-byte channel group, element width `width`.
#[allow(clippy::too_many_arguments)]
fn decode_deltas(
    buffer: &[u8],
    transposed: &mut [u8],
    k: usize,
    vertex_count: usize,
    vertex_size: usize,
    last_vertex: &[u8],
    width: usize,
    xor_rot: Option<u32>,
) {
    let mask: u32 = if width == 4 {
        u32::MAX
    } else {
        (1u32 << (width * 8)) - 1
    };
    let mut sub = 0;
    while sub < 4 {
        let mut p: u32 = 0;
        for j in 0..width {
            p |= (last_vertex[k + sub + j] as u32) << (8 * j);
        }
        let plane = &buffer[sub * vertex_count..];
        let mut offset = k + sub;
        for i in 0..vertex_count {
            let mut v: u32 = 0;
            for j in 0..width {
                v |= (plane[i + vertex_count * j] as u32) << (8 * j);
            }
            v = match xor_rot {
                Some(rot) => v.rotate_left(rot) ^ p,
                None => unzigzag_width(v, width).wrapping_add(p) & mask,
            };
            for j in 0..width {
                transposed[offset + j] = (v >> (j * 8)) as u8;
            }
            p = v;
            offset += vertex_size;
        }
        sub += width;
    }
}

fn unzigzag_width(v: u32, width: usize) -> u32 {
    match width {
        1 => unzigzag(v & 0xff) & 0xff,
        2 => unzigzag(v & 0xffff) & 0xffff,
        _ => unzigzag(v),
    }
}

#[allow(clippy::too_many_arguments)]
fn decode_vertex_block(
    r: &mut Reader,
    vertex_data: &mut [u8],
    vertex_count: usize,
    vertex_size: usize,
    last_vertex: &mut [u8; 256],
    channels: Option<&[u8]>,
    version: u8,
    buffer: &mut [u8],
    transposed: &mut [u8],
) -> Option<()> {
    let aligned = vertex_count.div_ceil(BYTE_GROUP_SIZE) * BYTE_GROUP_SIZE;
    let control_size = if version == 0 { 0 } else { vertex_size / 4 };
    if r.left() < control_size {
        return None;
    }
    let control = r.pos;
    r.pos += control_size;
    for k in (0..vertex_size).step_by(4) {
        let ctrl_byte = if version == 0 {
            0
        } else {
            r.data[control + k / 4]
        };
        for j in 0..4 {
            let ctrl = (ctrl_byte >> (j * 2)) & 3;
            let plane = &mut buffer[j * vertex_count..j * vertex_count + aligned];
            match ctrl {
                3 => {
                    if r.left() < vertex_count {
                        return None;
                    }
                    plane[..vertex_count].copy_from_slice(&r.data[r.pos..r.pos + vertex_count]);
                    r.pos += vertex_count;
                }
                2 => plane[..vertex_count].fill(0),
                _ => {
                    // Group decoding writes whole 16-byte groups; decode into
                    // scratch so the neighbouring plane is not overwritten.
                    let mut scratch = [0u8; VERTEX_BLOCK_MAX_SIZE];
                    let bits: &[u32] = if version == 0 {
                        &BITS_V0
                    } else {
                        &BITS_V1[ctrl as usize..]
                    };
                    decode_bytes(r, &mut scratch[..aligned], bits)?;
                    plane[..vertex_count].copy_from_slice(&scratch[..vertex_count]);
                }
            }
        }
        let channel = channels.map_or(0, |c| c[k / 4]);
        match channel & 3 {
            0 => decode_deltas(
                buffer,
                transposed,
                k,
                vertex_count,
                vertex_size,
                last_vertex,
                1,
                None,
            ),
            1 => decode_deltas(
                buffer,
                transposed,
                k,
                vertex_count,
                vertex_size,
                last_vertex,
                2,
                None,
            ),
            2 => decode_deltas(
                buffer,
                transposed,
                k,
                vertex_count,
                vertex_size,
                last_vertex,
                4,
                Some((32 - (channel as u32 >> 4)) & 31),
            ),
            _ => return None,
        }
    }
    vertex_data[..vertex_count * vertex_size]
        .copy_from_slice(&transposed[..vertex_count * vertex_size]);
    last_vertex[..vertex_size]
        .copy_from_slice(&transposed[vertex_size * (vertex_count - 1)..vertex_size * vertex_count]);
    Some(())
}

/// Decodes `vertex_count` vertices of `vertex_size` bytes (a multiple of 4,
/// at most 256) from a meshopt vertex stream.
pub fn decode_vertex_buffer(
    encoded: &[u8],
    vertex_count: usize,
    vertex_size: usize,
) -> Result<Vec<u8>, String> {
    if vertex_size == 0 || vertex_size > 256 || !vertex_size.is_multiple_of(4) {
        return Err(format!(
            "vertex size {vertex_size} is not a multiple of 4 up to 256"
        ));
    }
    let malformed = || "malformed meshopt vertex stream".to_string();
    let header = *encoded.first().ok_or_else(malformed)?;
    if header & 0xf0 != VERTEX_HEADER {
        return Err("not a meshopt vertex stream".into());
    }
    let version = header & 0x0f;
    if version > 1 {
        return Err(format!(
            "meshopt vertex codec version {version} is not supported"
        ));
    }
    let tail_size = vertex_size + if version == 0 { 0 } else { vertex_size / 4 };
    let tail_min = if version == 0 {
        TAIL_MIN_SIZE_V0
    } else {
        TAIL_MIN_SIZE_V1
    };
    let tail_pad = tail_size.max(tail_min);
    if encoded.len() - 1 < tail_pad {
        return Err(malformed());
    }
    let tail = encoded.len() - tail_size;
    let mut last_vertex = [0u8; 256];
    last_vertex[..vertex_size].copy_from_slice(&encoded[tail..tail + vertex_size]);
    let channels = (version != 0).then(|| &encoded[tail + vertex_size..]);
    let block = vertex_block_size(vertex_size);
    let mut out = vec![0u8; vertex_count * vertex_size];
    let mut buffer = vec![0u8; VERTEX_BLOCK_MAX_SIZE * 4 + BYTE_GROUP_SIZE];
    let mut transposed = vec![0u8; VERTEX_BLOCK_SIZE_BYTES];
    let mut r = Reader {
        data: encoded,
        pos: 1,
        end: encoded.len(),
    };
    let mut offset = 0;
    while offset < vertex_count {
        let n = block.min(vertex_count - offset);
        decode_vertex_block(
            &mut r,
            &mut out[offset * vertex_size..],
            n,
            vertex_size,
            &mut last_vertex,
            channels,
            version,
            &mut buffer,
            &mut transposed,
        )
        .ok_or_else(malformed)?;
        offset += n;
    }
    if r.left() != tail_pad {
        return Err(malformed());
    }
    Ok(out)
}

fn decode_vbyte(data: &[u8], pos: &mut usize) -> u32 {
    let lead = data[*pos];
    *pos += 1;
    if lead < 128 {
        return lead as u32;
    }
    let mut result = (lead & 127) as u32;
    let mut shift = 7;
    for _ in 0..4 {
        let group = data[*pos];
        *pos += 1;
        result |= ((group & 127) as u32) << shift;
        shift += 7;
        if group < 128 {
            break;
        }
    }
    result
}

fn decode_index(data: &[u8], pos: &mut usize, last: u32) -> u32 {
    let v = decode_vbyte(data, pos);
    let d = (v >> 1) ^ 0u32.wrapping_sub(v & 1);
    last.wrapping_add(d)
}

fn push_vertex(fifo: &mut [u32; 16], off: &mut usize, v: u32, cond: bool) {
    fifo[*off] = v;
    *off = (*off + cond as usize) & 15;
}

fn push_edge(fifo: &mut [[u32; 2]; 16], off: &mut usize, a: u32, b: u32) {
    fifo[*off] = [a, b];
    *off = (*off + 1) & 15;
}

/// Decodes `index_count` (a multiple of 3) indices from a meshopt index buffer.
pub fn decode_index_buffer(encoded: &[u8], index_count: usize) -> Result<Vec<u32>, String> {
    if !index_count.is_multiple_of(3) {
        return Err(format!(
            "{index_count} indices is not a whole number of triangles"
        ));
    }
    let malformed = || "malformed meshopt index buffer".to_string();
    if encoded.len() < 1 + index_count / 3 + 16 {
        return Err(malformed());
    }
    if encoded[0] & 0xf0 != INDEX_HEADER {
        return Err("not a meshopt index buffer".into());
    }
    let version = encoded[0] & 0x0f;
    if version > 1 {
        return Err(format!(
            "meshopt index codec version {version} is not supported"
        ));
    }
    let mut edge = [[u32::MAX; 2]; 16];
    let mut vertex = [u32::MAX; 16];
    let (mut edge_off, mut vertex_off) = (0usize, 0usize);
    let (mut next, mut last) = (0u32, 0u32);
    let fecmax = if version >= 1 { 13 } else { 15 };
    let mut data = 1 + index_count / 3;
    let data_safe_end = encoded.len() - 16;
    let codeaux_table = &encoded[data_safe_end..];
    let mut out = vec![0u32; index_count];
    // Every read of `data` is bounded by data_safe_end + 16 (checked per triangle).
    for (code, i) in (1usize..).zip((0..index_count).step_by(3)) {
        if data > data_safe_end {
            return Err(malformed());
        }
        let codetri = encoded[code];
        if codetri < 0xf0 {
            let fe = (codetri >> 4) as usize;
            let [a, b] = edge[(edge_off.wrapping_sub(1 + fe)) & 15];
            let fec = (codetri & 15) as u32;
            let c;
            if fec < fecmax {
                let cf = vertex[(vertex_off.wrapping_sub(1 + fec as usize)) & 15];
                c = if fec == 0 { next } else { cf };
                let fec0 = fec == 0;
                next += fec0 as u32;
                push_vertex(&mut vertex, &mut vertex_off, c, fec0);
            } else {
                c = if fec != 15 {
                    // 13, 14 decode to -1, +1.
                    last.wrapping_add((fec as i32 - (fec ^ 3) as i32) as u32)
                } else {
                    decode_index(encoded, &mut data, last)
                };
                last = c;
                push_vertex(&mut vertex, &mut vertex_off, c, true);
            }
            push_edge(&mut edge, &mut edge_off, c, b);
            push_edge(&mut edge, &mut edge_off, a, c);
            out[i..i + 3].copy_from_slice(&[a, b, c]);
        } else if codetri < 0xfe {
            let codeaux = codeaux_table[(codetri & 15) as usize];
            let feb = (codeaux >> 4) as usize;
            let fec = (codeaux & 15) as usize;
            let a = next;
            next += 1;
            let bf = vertex[(vertex_off.wrapping_sub(feb)) & 15];
            let b = if feb == 0 { next } else { bf };
            next += (feb == 0) as u32;
            let cf = vertex[(vertex_off.wrapping_sub(fec)) & 15];
            let c = if fec == 0 { next } else { cf };
            next += (fec == 0) as u32;
            out[i..i + 3].copy_from_slice(&[a, b, c]);
            push_vertex(&mut vertex, &mut vertex_off, a, true);
            push_vertex(&mut vertex, &mut vertex_off, b, feb == 0);
            push_vertex(&mut vertex, &mut vertex_off, c, fec == 0);
            push_edge(&mut edge, &mut edge_off, b, a);
            push_edge(&mut edge, &mut edge_off, c, b);
            push_edge(&mut edge, &mut edge_off, a, c);
        } else {
            let codeaux = encoded[data];
            data += 1;
            let fea = if codetri == 0xfe { 0 } else { 15 };
            let feb = (codeaux >> 4) as usize;
            let fec = (codeaux & 15) as usize;
            if codeaux == 0 {
                next = 0;
            }
            let mut a = if fea == 0 {
                next += 1;
                next - 1
            } else {
                0
            };
            let mut b = if feb == 0 {
                next += 1;
                next - 1
            } else {
                vertex[(vertex_off.wrapping_sub(feb)) & 15]
            };
            let mut c = if fec == 0 {
                next += 1;
                next - 1
            } else {
                vertex[(vertex_off.wrapping_sub(fec)) & 15]
            };
            if fea == 15 {
                a = decode_index(encoded, &mut data, last);
                last = a;
            }
            if feb == 15 {
                b = decode_index(encoded, &mut data, last);
                last = b;
            }
            if fec == 15 {
                c = decode_index(encoded, &mut data, last);
                last = c;
            }
            out[i..i + 3].copy_from_slice(&[a, b, c]);
            push_vertex(&mut vertex, &mut vertex_off, a, true);
            push_vertex(&mut vertex, &mut vertex_off, b, feb == 0 || feb == 15);
            push_vertex(&mut vertex, &mut vertex_off, c, fec == 0 || fec == 15);
            push_edge(&mut edge, &mut edge_off, b, a);
            push_edge(&mut edge, &mut edge_off, c, b);
            push_edge(&mut edge, &mut edge_off, a, c);
        }
    }
    if data != data_safe_end {
        return Err(malformed());
    }
    Ok(out)
}
