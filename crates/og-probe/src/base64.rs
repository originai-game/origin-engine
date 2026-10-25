//! Standard base64 with padding (RFC 4648 §4), for the `png` and `wav`
//! fields of the wire and the runtime's outputs and saves. Encoding only:
//! the probe never reads base64.

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// `bytes` as base64, written three bytes to four characters at a time (a
/// capture's megabyte of PNG: pushing one `char` at a time was most of it).
pub fn encode(bytes: &[u8]) -> String {
    let sym = |n: u32, shift: u32| ALPHABET[((n >> shift) & 63) as usize];
    let mut out = Vec::with_capacity(bytes.len().div_ceil(3) * 4);
    let (triples, rest) = bytes.as_chunks::<3>();
    for &[a, b, c] in triples {
        let n = (u32::from(a) << 16) | (u32::from(b) << 8) | u32::from(c);
        out.extend_from_slice(&[sym(n, 18), sym(n, 12), sym(n, 6), sym(n, 0)]);
    }
    match *rest {
        [a] => {
            let n = u32::from(a) << 16;
            out.extend_from_slice(&[sym(n, 18), sym(n, 12), b'=', b'=']);
        }
        [a, b] => {
            let n = (u32::from(a) << 16) | (u32::from(b) << 8);
            out.extend_from_slice(&[sym(n, 18), sym(n, 12), sym(n, 6), b'=']);
        }
        _ => {}
    }
    String::from_utf8(out).expect("the alphabet is ASCII")
}
