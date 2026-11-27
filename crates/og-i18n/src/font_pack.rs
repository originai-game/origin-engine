//! Font packs (docs/ui.md §3.4): fonts shipped like the UI packs, one per
//! script, each a directory with an `extension.json` of kind `font`. The
//! cook turns an installed pack into this index (`og.FontPack`): its name,
//! the languages it is the face of, the asset ids of its faces and the code
//! points they cover, read from their cmaps. og.i18n builds the font chain
//! from the indexes in the bundle: the pack of the active locale first,
//! then any other installed pack, loaded only when a frame draws a
//! character only it covers.

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
