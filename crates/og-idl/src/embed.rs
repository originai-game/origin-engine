//! The static word-embedding half of `og api find`: a model2vec model
//! (minishlab/potion-base-8M, MIT; `third_party/potion-base-8M/`) cut to 128
//! dimensions and int8 (`crates/og-api/search/words.bin`, written by
//! `make-model.py` beside it). A text's vector is the mean of its WordPiece
//! tokens' vectors, normalised; there is no network and no inference beyond
//! that mean, so a query embeds in microseconds on one core.
//!
//! The tokenizer is the model's own (bge-base's uncased WordPiece): text
//! lowercased, split at whitespace and at every punctuation character, each
//! word cut greedily into the longest vocabulary pieces (`##` marks a piece
//! inside a word); a word with no cut is unknown and left out.

use std::collections::HashMap;
use std::sync::OnceLock;

const WORDS: &[u8] = include_bytes!("../../og-api/search/words.bin");

/// The model: every token's vector, dequantised on use.
pub struct Model {
    pub dim: usize,
    ids: HashMap<String, u32>,
    scale: Vec<f32>,
    data: Vec<i8>,
}

/// The model `og` ships, read once.
pub fn model() -> &'static Model {
    static M: OnceLock<Model> = OnceLock::new();
    M.get_or_init(|| {
        Model::read(WORDS).expect("crates/og-api/search/words.bin is an OGWORDS1 model")
    })
}

impl Model {
    pub fn read(b: &[u8]) -> Option<Model> {
        let u32_at = |at: usize| -> Option<u32> {
            Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
        };
        if b.get(..8)? != b"OGWORDS1" {
            return None;
        }
        let n = u32_at(8)? as usize;
        let dim = u32_at(12)? as usize;
        let mut at = 20;
        let mut ids = HashMap::with_capacity(n);
        for i in 0..n {
            let len = usize::from(*b.get(at)?);
            let w = std::str::from_utf8(b.get(at + 1..at + 1 + len)?).ok()?;
            ids.insert(w.to_string(), i as u32);
            at += 1 + len;
        }
        let scale = b
            .get(at..at + 4 * n)?
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| f32::from_le_bytes(*c))
            .collect();
        at += 4 * n;
        let data = b.get(at..at + n * dim)?.iter().map(|&x| x as i8).collect();
        Some(Model {
            dim,
            ids,
            scale,
            data,
        })
    }

    /// The WordPiece token ids of `text`, unknown words left out.
    pub fn tokens(&self, text: &str) -> Vec<u32> {
        let mut out = Vec::new();
        for word in words(text) {
            let chars: Vec<char> = word.chars().collect();
            if chars.len() > 100 {
                continue;
            }
            let mut pieces = Vec::new();
            let mut start = 0;
            while start < chars.len() {
                let mut end = chars.len();
                let mut found = None;
                while end > start {
                    let s: String = chars[start..end].iter().collect();
                    let s = if start > 0 { format!("##{s}") } else { s };
                    if let Some(&id) = self.ids.get(&s) {
                        found = Some(id);
                        break;
                    }
                    end -= 1;
                }
                match found {
                    Some(id) => {
                        pieces.push(id);
                        start = end;
                    }
                    None => {
                        pieces.clear();
                        break;
                    }
                }
            }
            out.extend(pieces);
        }
        out
    }

    /// The normalised mean of `text`'s token vectors; all zeros when no
    /// token is known.
    pub fn embed(&self, text: &str) -> Vec<f32> {
        let mut v = vec![0f32; self.dim];
        for id in self.tokens(text) {
            let id = id as usize;
            let s = self.scale[id];
            for (x, &q) in v
                .iter_mut()
                .zip(&self.data[id * self.dim..(id + 1) * self.dim])
            {
                *x += s * f32::from(q);
            }
        }
        let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut v {
                *x /= norm;
            }
        }
        v
    }
}

/// BERT's basic tokenizer: lowercased, accents off for ASCII letters' kin
/// left as they are, split at whitespace and around every punctuation and
/// CJK character.
fn words(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in text.chars() {
        if c.is_control() && !c.is_whitespace() || c == '\u{fffd}' {
            continue;
        }
        if c.is_whitespace() {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
        } else if is_punct(c) || is_cjk(c) {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            out.push(c.to_string());
        } else {
            cur.extend(c.to_lowercase());
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn is_punct(c: char) -> bool {
    c.is_ascii_punctuation() || (!c.is_alphanumeric() && !c.is_whitespace() && !c.is_ascii())
}

fn is_cjk(c: char) -> bool {
    matches!(c as u32, 0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0x20000..=0x2A6DF | 0xF900..=0xFAFF | 0x2F800..=0x2FA1F)
}

/// An embedding stored in the index: int8 components and the scale that
/// gives them back (`component = q * scale`).
pub fn quantize(v: &[f32]) -> (Vec<i8>, f32) {
    let max = v.iter().fold(0f32, |m, x| m.max(x.abs()));
    if max == 0.0 {
        return (vec![0; v.len()], 0.0);
    }
    let scale = max / 127.0;
    (
        v.iter()
            .map(|x| (x / scale).round().clamp(-127.0, 127.0) as i8)
            .collect(),
        scale,
    )
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for c in bytes.chunks(3) {
        let n = (u32::from(c[0]) << 16)
            | (u32::from(*c.get(1).unwrap_or(&0)) << 8)
            | u32::from(*c.get(2).unwrap_or(&0));
        for i in 0..4 {
            if i <= c.len() {
                out.push(B64[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

pub fn unbase64(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
    for c in s.as_bytes().chunks(4) {
        if c.len() != 4 {
            return None;
        }
        let mut n = 0u32;
        let mut pad = 0;
        for &x in c {
            let v = match x {
                b'=' => {
                    pad += 1;
                    0
                }
                _ => B64.iter().position(|&b| b == x)? as u32,
            };
            n = (n << 6) | v;
        }
        out.extend_from_slice(&n.to_be_bytes()[1..4 - pad]);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wordpiece_cuts_words_into_the_longest_pieces() {
        let m = model();
        assert_eq!(m.dim, 128);
        let ids = m.tokens("Shake, camera!");
        assert_eq!(ids.len(), 4, "{ids:?}");
        // Related words sit closer than unrelated ones.
        let dot = |a: &[f32], b: &[f32]| a.iter().zip(b).map(|(x, y)| x * y).sum::<f32>();
        let tremble = m.embed("the screen trembles");
        assert!(dot(&tremble, &m.embed("camera shake")) > dot(&tremble, &m.embed("save slot")));
        assert!(m.embed("zzqxv").iter().all(|&x| x == 0.0) || !m.tokens("zzqxv").is_empty());
    }

    #[test]
    fn base64_round_trips() {
        for n in 0..8 {
            let b: Vec<u8> = (0..n).map(|i| (i * 37 + 200) as u8).collect();
            assert_eq!(unbase64(&base64(&b)).unwrap(), b);
        }
    }
}
