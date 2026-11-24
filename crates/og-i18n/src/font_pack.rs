//! Font packs (docs/ui.md §3.4): fonts shipped like the UI packs, one per
//! script, each a directory with an `extension.json` of kind `font`. The
//! cook turns an installed pack into this index (`og.FontPack`): its name,
//! the languages it is the face of, the asset ids of its faces and the code
//! points they cover, read from their cmaps. og.i18n builds the font chain
//! from the indexes in the bundle: the pack of the active locale first,
//! then any other installed pack, loaded only when a frame draws a
//! character only it covers.
//!
//! The text form, one field per line:
//!
//! ```text
//! name arabic
//! languages ar fa ur
//! face fonts/arabic/NotoSansArabic-Regular.ttf
//! coverage 20-7e 600-6ff
//! ```

use std::ops::RangeInclusive;

/// One installed pack, as its cooked index holds it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FontPack {
    /// Its directory and manifest name (`arabic`).
    pub name: String,
    /// The language tags it is the face of (`ar`, `zh-Hant`), canonical.
    pub languages: Vec<String>,
    /// Its faces' asset ids, first first.
    pub faces: Vec<String>,
    /// The code points its faces draw, as sorted disjoint ranges.
    pub coverage: Vec<RangeInclusive<u32>>,
}

impl FontPack {
    /// The index as text (the blob's `text`).
    pub fn to_text(&self) -> String {
        let mut out = format!(
            "name {}\nlanguages {}\n",
            self.name,
            self.languages.join(" ")
        );
        for f in &self.faces {
            out.push_str(&format!("face {f}\n"));
        }
        let ranges: Vec<String> = self
            .coverage
            .iter()
            .map(|r| format!("{:x}-{:x}", r.start(), r.end()))
            .collect();
        out.push_str(&format!("coverage {}\n", ranges.join(" ")));
        out
    }

    /// Reads [`FontPack::to_text`]'s form.
    pub fn parse(text: &str) -> Result<FontPack, String> {
        let mut p = FontPack::default();
        for (n, line) in text.lines().enumerate() {
            let (key, rest) = line.split_once(' ').unwrap_or((line, ""));
            match key {
                "name" => p.name = rest.trim().to_string(),
                "languages" => {
                    p.languages = rest
                        .split_whitespace()
                        .map(crate::locale::canonical)
                        .collect()
                }
                "face" => p.faces.push(rest.trim().to_string()),
                "coverage" => {
                    for r in rest.split_whitespace() {
                        let (a, b) = r
                            .split_once('-')
                            .ok_or(format!("line {}: range {r}", n + 1))?;
                        let num = |s: &str| {
                            u32::from_str_radix(s, 16).map_err(|e| format!("line {}: {e}", n + 1))
                        };
                        p.coverage.push(num(a)?..=num(b)?);
                    }
                }
                "" => {}
                k => return Err(format!("line {}: '{k}' is not a font pack field", n + 1)),
            }
        }
        if p.name.is_empty() || p.faces.is_empty() {
            return Err("a font pack index needs a name and a face".into());
        }
        Ok(p)
    }

    /// Whether its faces draw `c`.
    pub fn covers(&self, c: char) -> bool {
        let c = c as u32;
        self.coverage
            .binary_search_by(|r| {
                if *r.end() < c {
                    std::cmp::Ordering::Less
                } else if *r.start() > c {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .is_ok()
    }
}

/// Sorted disjoint ranges of `points` (in any order, repeats allowed).
pub fn ranges(points: impl IntoIterator<Item = u32>) -> Vec<RangeInclusive<u32>> {
    let mut v: Vec<u32> = points.into_iter().collect();
    v.sort_unstable();
    v.dedup();
    let mut out: Vec<RangeInclusive<u32>> = Vec::new();
    for c in v {
        match out.last_mut() {
            Some(r) if *r.end() + 1 == c => *r = *r.start()..=c,
            _ => out.push(c..=c),
        }
    }
    out
}

/// The pack that is `locale`'s own: the one whose languages negotiate
/// best with it (`zh-TW` takes the traditional pack, never the simplified
/// one; `pt-BR` takes the pack that names `pt`).
pub fn for_locale<'a>(packs: &'a [FontPack], locale: &str) -> Option<&'a FontPack> {
    let tags: Vec<String> = packs
        .iter()
        .flat_map(|p| p.languages.iter().cloned())
        .collect();
    let tag = crate::locale::negotiate(&[locale], &tags)?;
    packs.iter().find(|p| p.languages.iter().any(|l| l == tag))
}

/// The order packs are tried for a character no loaded face draws: the
/// locale's own first, then the CJK packs in the locale's forms (a Han
/// character in a Japanese game takes Japanese forms), then the rest by
/// name, and `emoji` last: it also maps digits, `#`, `*`, © and ™, which a
/// text face draws as text. Answers indexes into `packs`.
pub fn fallback_order(packs: &[FontPack], locale: &str) -> Vec<usize> {
    let own = for_locale(packs, locale).map(|p| p.name.clone());
    let lang = crate::locale::language(&crate::locale::canonical(locale));
    let cjk = match lang.as_str() {
        "ja" => [
            "japanese",
            "chinese-traditional",
            "chinese-simplified",
            "korean",
        ],
        "ko" => [
            "korean",
            "chinese-traditional",
            "japanese",
            "chinese-simplified",
        ],
        "zh" if for_locale(packs, locale).is_some_and(|p| p.name == "chinese-traditional") => [
            "chinese-traditional",
            "chinese-simplified",
            "japanese",
            "korean",
        ],
        _ => [
            "chinese-simplified",
            "chinese-traditional",
            "japanese",
            "korean",
        ],
    };
    let rank = |p: &FontPack| {
        if Some(&p.name) == own.as_ref() {
            (0, 0, p.name.clone())
        } else if let Some(i) = cjk.iter().position(|n| *n == p.name) {
            (1, i, p.name.clone())
        } else if p.name == "emoji" {
            (3, 0, p.name.clone())
        } else {
            (2, 0, p.name.clone())
        }
    };
    let mut order: Vec<usize> = (0..packs.len()).collect();
    order.sort_by_key(|&i| rank(&packs[i]));
    order
}
