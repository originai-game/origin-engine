//! Automatic hyphenation (CSS Text 4 §6.1 `hyphens: auto`) from
//! dictionaries a game ships in its font packs (docs/ui.md §3.4): hyph-utf8's
//! TeX patterns compiled as hypher's finite automata, installed per
//! language as the runtime reads the packs. A language with no dictionary
//! installed is not hyphenated.
//!
//! The automaton walk is ported from hypher 0.1.8 (Laurenz Mädje,
//! <https://github.com/typst/hypher>, MIT OR Apache-2.0; licence in
//! `third_party/hypher/LICENSE-MIT`), reading the dictionary's bytes as
//! data: a malformed dictionary finds no break instead of panicking.

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

/// The installed dictionaries by language (ISO 639-1). A language's
/// dictionary is the same bytes whichever game ships it.
static DICTIONARIES: RwLock<BTreeMap<String, Arc<[u8]>>> = RwLock::new(BTreeMap::new());

/// The dictionary language of a BCP 47 tag: its primary subtag, Norwegian
/// Bokmål and Nynorsk sharing Norwegian's patterns.
pub fn dictionary_language(tag: &str) -> String {
    let primary = tag
        .split(['-', '_'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    match primary.as_str() {
        "nb" | "nn" => "no".into(),
        _ => primary,
    }
}

/// Installs `dictionary` (a compiled `.hyph` file) for language `tag`.
pub fn install(tag: &str, dictionary: Arc<[u8]>) {
    if let Ok(mut d) = DICTIONARIES.write() {
        d.insert(dictionary_language(tag), dictionary);
    }
}

/// Whether a dictionary is installed for language `tag`.
pub fn installed(tag: &str) -> bool {
    DICTIONARIES
        .read()
        .is_ok_and(|d| d.contains_key(&dictionary_language(tag)))
}

/// The fewest characters a language's patterns break before and after
/// (TeX's `\lefthyphenmin` and `\righthyphenmin`, as hypher gives them).
pub fn bounds(tag: &str) -> (usize, usize) {
    match dictionary_language(tag).as_str() {
        "af" | "ka" => (1, 2),
        "el" => (1, 1),
        "en" | "et" | "pt" | "sk" => (2, 3),
        _ => (2, 2),
    }
}

/// `text` with a soft hyphen (U+00AD) at each of its words' hyphenation
/// points in language `tag`, as CSS Text 4 §6.1's `hyphens: auto` and
/// §6.3's `hyphenate-limit-chars` ask: a word is a run of letters and
/// marks of at least `word` characters, broken no nearer its start than
/// `before` characters and its end than `after` (`None`: the language's
/// [`bounds`]). A word that already holds a soft hyphen keeps only its own
/// (§6.1). With no dictionary installed for the language, `text` as it is.
pub fn hyphenate(
    text: &str,
    tag: &str,
    word: usize,
    before: Option<usize>,
    after: Option<usize>,
) -> String {
    let Some(data) = DICTIONARIES
        .read()
        .ok()
        .and_then(|d| d.get(&dictionary_language(tag)).cloned())
    else {
        return text.to_owned();
    };
    let (own_before, own_after) = bounds(tag);
    let (before, after) = (before.unwrap_or(own_before), after.unwrap_or(own_after));
    let letter = |c: char| c.is_alphabetic() || crate::text::is_mark(c);
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find(letter) {
        out.push_str(&rest[..start]);
        rest = &rest[start..];
        let end = rest
            .find(|c: char| !letter(c) && c != '\u{ad}')
            .unwrap_or(rest.len());
        let w = &rest[..end];
        if w.contains('\u{ad}') || w.chars().count() < word.max(1) {
            out.push_str(w);
        } else {
            let breaks = points(&data, w, before.max(1), after.max(1));
            let mut from = 0;
            for at in breaks {
                out.push_str(&w[from..at]);
                out.push('\u{ad}');
                from = at;
            }
            out.push_str(&w[from..]);
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

/// The byte offsets in `word` where its patterns allow a break, no nearer
/// its start than `before` characters and its end than `after`.
fn points(data: &[u8], word: &str, before: usize, after: usize) -> Vec<usize> {
    let Some(root) = data
        .get(..4)
        .and_then(|b| b.try_into().ok())
        .and_then(|b| State::at(data, u32::from_be_bytes(b) as usize))
    else {
        return Vec::new();
    };
    // The word lowercased (where that keeps its length) between dots, so
    // patterns can match at its edges.
    let mut dotted = Vec::with_capacity(word.len() + 2);
    dotted.push(b'.');
    let mut buf = [0u8; 4];
    for c in word.chars() {
        let mut lower = c.to_lowercase();
        let c = match (lower.next(), lower.next()) {
            (Some(l), None) if l.len_utf8() == c.len_utf8() => l,
            _ => c,
        };
        dotted.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
    }
    dotted.push(b'.');
    // Break bounds as byte indices in the dotted word.
    let min = 1 + word.chars().take(before).map(char::len_utf8).sum::<usize>();
    let max = (1 + word.len()).saturating_sub(
        word.chars()
            .rev()
            .take(after)
            .map(char::len_utf8)
            .sum::<usize>(),
    );
    // The level between each two inner bytes: the highest any match gives.
    let mut levels = vec![0u8; word.len().saturating_sub(1)];
    for start in 0..dotted.len() {
        // At character boundaries only.
        if (dotted[start] as i8) < -0x40 {
            continue;
        }
        let mut state = root;
        for &b in &dotted[start..] {
            let Some(next) = state.transition(b) else {
                break;
            };
            state = next;
            for (offset, level) in state.levels() {
                let split = start + offset;
                if (min..=max).contains(&split)
                    && let Some(slot) = split.checked_sub(2).and_then(|k| levels.get_mut(k))
                {
                    *slot = (*slot).max(level);
                }
            }
        }
    }
    // An odd level allows a break after that byte.
    levels
        .iter()
        .enumerate()
        .filter(|(k, l)| *l % 2 == 1 && word.is_char_boundary(k + 1))
        .map(|(k, _)| k + 1)
        .collect()
}

/// A state of the automaton.
#[derive(Clone, Copy)]
struct State<'a> {
    data: &'a [u8],
    addr: usize,
    stride: usize,
    levels: &'a [u8],
    trans: &'a [u8],
    targets: &'a [u8],
}

impl<'a> State<'a> {
    /// The node at `addr`: a header byte (levels flag, target stride,
    /// transition count), an optional extended count, an optional levels
    /// reference, then its transition labels and their relative targets.
    fn at(data: &'a [u8], addr: usize) -> Option<Self> {
        let node = data.get(addr..)?;
        let head = *node.first()?;
        let has_levels = head >> 7 != 0;
        let stride = usize::from((head >> 5) & 3);
        let mut count = usize::from(head & 31);
        let mut pos = 1;
        if count == 31 {
            count = usize::from(*node.get(pos)?);
            pos += 1;
        }
        let mut levels: &[u8] = &[];
        if has_levels {
            let (hi, lo) = (*node.get(pos)?, *node.get(pos + 1)?);
            let offset = (usize::from(hi) << 4) | (usize::from(lo) >> 4);
            levels = data.get(offset..offset + usize::from(lo & 15))?;
            pos += 2;
        }
        let trans = node.get(pos..pos + count)?;
        pos += count;
        let targets = node.get(pos..pos + stride * count)?;
        Some(Self {
            data,
            addr,
            stride,
            levels,
            trans,
            targets,
        })
    }

    /// The state the transition labelled `b` reaches.
    fn transition(self, b: u8) -> Option<Self> {
        let k = self.trans.iter().position(|&x| x == b)?;
        let delta = signed(self.targets.get(self.stride * k..self.stride * (k + 1))?)?;
        Self::at(self.data, self.addr.checked_add_signed(delta)?)
    }

    /// Its levels: each an offset from the match's start and a level.
    fn levels(self) -> impl Iterator<Item = (usize, u8)> + 'a {
        let mut offset = 0;
        self.levels.iter().map(move |&packed| {
            offset += usize::from(packed / 10);
            (offset, packed % 10)
        })
    }
}

/// A big-endian signed number of one, two or three bytes.
fn signed(b: &[u8]) -> Option<isize> {
    match *b {
        [a] => Some(a as i8 as isize),
        [a, c] => Some(i16::from_be_bytes([a, c]) as isize),
        [a, c, d] => Some(
            ((usize::from(a) << 16 | usize::from(c) << 8 | usize::from(d)) as isize) - (1 << 23),
        ),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dictionary(language: &str) -> Arc<[u8]> {
        let path = format!(
            "{}/../../templates/fonts/latin/{language}.hyph",
            env!("CARGO_MANIFEST_DIR")
        );
        std::fs::read(path).unwrap().into()
    }

    /// The ported walk breaks where TeX's patterns do (pyphen, over
    /// LibreOffice's dictionaries, agrees): English at 2 and 3 characters
    /// from the ends, German at 2 and 2, a capitalised word as its lower
    /// case. A dictionary cut short finds no break and does not panic.
    #[test]
    fn breaks_where_the_patterns_do() {
        let (en, de) = (dictionary("en"), dictionary("de"));
        let shy = |s: &str| s.replace('-', "\u{ad}");
        assert_eq!(points(&en, "hyphenation", 2, 3), [2, 6]);
        assert_eq!(points(&en, "Extensive", 2, 3), [2, 5]);
        assert_eq!(points(&de, "silbentrennung", 2, 2), [3, 6, 10]);
        let _ = points(&en[..en.len() / 3], "hyphenation", 2, 3);
        assert!(points(&[0xff; 8], "hyphenation", 2, 3).is_empty());
        install("de-AT", de);
        assert_eq!(
            hyphenate("Die Silbentrennung.", "de", 5, None, None),
            shy("Die Sil-ben-tren-nung.")
        );
        assert_eq!(
            hyphenate("Silbentrennung", "fr", 5, None, None),
            "Silbentrennung"
        );
    }
}
