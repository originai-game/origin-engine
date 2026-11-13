//! Base64 (RFC 4648): how bytes cross JSON and text, the engine's one copy.
//! Saves, the probe's `png` and `wav`, the rooms' messages, the wiki's
//! embeddings and a glTF's `data:` URIs all read and write it here.

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// `bytes` as standard base64 with padding (§4), written three bytes to four
/// characters at a time (a capture's megabyte of PNG: pushing one `char` at
/// a time was most of it).
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

/// The bytes of base64 `text`, `None` for text that is not base64. It reads
/// what other writers produce as well as [`encode`]'s output: the standard
/// and the URL-safe alphabet (§5), padded or not, with white space between
/// the characters; it stops at the first `=`.
pub fn decode(text: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    let (mut acc, mut bits) = (0u32, 0u32);
    for c in text.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' => break,
            b' ' | b'\n' | b'\r' | b'\t' => continue,
            _ => return None,
        };
        acc = (acc << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::{decode, encode};

    #[test]
    fn every_length_round_trips_and_other_writers_text_decodes() {
        for n in 0..40u8 {
            let bytes: Vec<u8> = (0..n).map(|i| i.wrapping_mul(73) ^ 0xa5).collect();
            assert_eq!(decode(&encode(&bytes)).unwrap(), bytes);
        }
        assert_eq!(encode(b"Man"), "TWFu");
        assert_eq!(encode(b"Ma"), "TWE=");
        assert_eq!(encode(b"M"), "TQ==");
        assert_eq!(decode("aGVsbG8=").unwrap(), b"hello");
        assert_eq!(decode("aGVsbG8").unwrap(), b"hello");
        assert_eq!(decode("aGVs\n bG8=").unwrap(), b"hello");
        assert_eq!(decode("-_-_").unwrap(), decode("+/+/").unwrap());
        assert!(decode("a!b").is_none());
    }
}
