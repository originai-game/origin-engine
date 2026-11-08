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

#[cfg(test)]
mod tests {
    use super::*;

    fn pack(name: &str, langs: &[&str], cover: &[(u32, u32)]) -> FontPack {
        FontPack {
            name: name.into(),
            languages: langs.iter().map(|l| crate::locale::canonical(l)).collect(),
            faces: vec![format!("fonts/{name}/a.ttf")],
            coverage: cover.iter().map(|&(a, b)| a..=b).collect(),
        }
    }

    fn all() -> Vec<FontPack> {
        vec![
            pack("arabic", &["ar", "fa"], &[(0x600, 0x6ff)]),
            pack(
                "chinese-simplified",
                &["zh", "zh-Hans", "zh-CN"],
                &[(0x4e00, 0x9fff)],
            ),
            pack(
                "chinese-traditional",
                &["zh-Hant", "zh-TW", "zh-HK"],
                &[(0x4e00, 0x9fff)],
            ),
            pack("japanese", &["ja"], &[(0x3040, 0x30ff), (0x4e00, 0x9fff)]),
            pack("latin", &["en", "pt", "vi"], &[(0x20, 0x24f)]),
        ]
    }

    #[test]
    fn the_index_round_trips_and_covers_by_range() {
        let p = pack("arabic", &["ar", "fa"], &[(0x20, 0x7e), (0x600, 0x6ff)]);
        assert_eq!(FontPack::parse(&p.to_text()), Ok(p.clone()));
        assert!(
            p.covers('\u{627}') && p.covers('A') && !p.covers('\u{5d0}') && !p.covers('\u{7f}')
        );
        assert_eq!(ranges([5, 3, 4, 9, 9, 10, 1]), vec![1..=1, 3..=5, 9..=10]);
    }

    #[test]
    fn a_locale_takes_its_own_pack_and_chinese_scripts_never_cross() {
        let packs = all();
        let own = |l| for_locale(&packs, l).map(|p| p.name.as_str());
        assert_eq!(own("zh-TW"), Some("chinese-traditional"));
        assert_eq!(own("zh-Hant-HK"), Some("chinese-traditional"));
        assert_eq!(own("zh-CN"), Some("chinese-simplified"));
        assert_eq!(own("zh"), Some("chinese-simplified"));
        assert_eq!(own("pt-BR"), Some("latin"));
        assert_eq!(own("fa-IR"), Some("arabic"));
        assert_eq!(own("ko"), None);
    }

    #[test]
    fn han_falls_back_in_the_locale_s_forms() {
        let packs = all();
        let first_han = |l: &str| {
            fallback_order(&packs, l)
                .into_iter()
                .map(|i| &packs[i])
                .find(|p| p.covers('\u{76f4}'))
                .map(|p| p.name.clone())
        };
        assert_eq!(first_han("ja").as_deref(), Some("japanese"));
        assert_eq!(first_han("en").as_deref(), Some("chinese-simplified"));
        assert_eq!(first_han("zh-TW").as_deref(), Some("chinese-traditional"));
        assert_eq!(first_han("ar").as_deref(), Some("chinese-simplified"));
    }

    /// The emoji pack maps text characters too (digits, ©, ™): a Latin one
    /// a face lacks takes the Latin pack, though "emoji" sorts first by
    /// name; an emoji only it covers still takes it.
    #[test]
    fn text_characters_take_a_text_pack_before_the_emoji_pack() {
        let mut packs = all();
        packs.push(pack(
            "emoji",
            &[],
            &[(0x20, 0x7e), (0xa9, 0xa9), (0x1f300, 0x1faff)],
        ));
        let first = |c: char| {
            // A locale with no pack of its own: the by-name tail decides.
            fallback_order(&packs, "ko")
                .into_iter()
                .map(|i| &packs[i])
                .find(|p| p.covers(c))
                .map(|p| p.name.clone())
        };
        assert_eq!(first('7').as_deref(), Some("latin"));
        assert_eq!(first('\u{a9}').as_deref(), Some("latin"));
        assert_eq!(first('\u{1f525}').as_deref(), Some("emoji"));
    }
}
