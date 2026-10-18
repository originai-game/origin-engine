//! Locale tags: negotiation of a requested locale against the tables a game
//! has, and the writing direction of a locale.

/// The writing direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Ltr,
    Rtl,
}

impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Direction::Ltr => "ltr",
            Direction::Rtl => "rtl",
        }
    }
}

/// `zh_CN` and `ZH-cn` as `zh-CN`; the language lowercased, a region
/// uppercased, a script titlecased.
pub fn canonical(tag: &str) -> String {
    tag.trim()
        .split(['-', '_'])
        .filter(|p| !p.is_empty())
        .enumerate()
        .map(|(k, p)| {
            if k == 0 {
                p.to_ascii_lowercase()
            } else if p.len() == 4 {
                let mut s = p.to_ascii_lowercase();
                s[..1].make_ascii_uppercase();
                s
            } else {
                p.to_ascii_uppercase()
            }
        })
        .collect::<Vec<_>>()
        .join("-")
}

/// The primary language subtag.
pub fn language(tag: &str) -> String {
    canonical(tag).split('-').next().unwrap_or("").to_string()
}

/// The script a Chinese tag implies: Hant for TW, HK, MO, Hant; else Hans.
fn chinese_script(tag: &str) -> &'static str {
    let t = canonical(tag);
    if ["-Hant", "-TW", "-HK", "-MO"].iter().any(|s| t.contains(s)) {
        "Hant"
    } else {
        "Hans"
    }
}

/// The best of `available` for `requested` (in order of preference): an
/// exact tag, then the same language (Chinese by script: zh-TW takes
/// zh-Hant or zh-TW, zh-CN and zh-SG take zh-Hans or zh-CN), then `None`.
pub fn negotiate<'a>(requested: &[&str], available: &'a [String]) -> Option<&'a str> {
    for r in requested {
        let rc = canonical(r);
        if let Some(a) = available.iter().find(|a| canonical(a) == rc) {
            return Some(a);
        }
        let lang = language(&rc);
        let same: Vec<&String> = available.iter().filter(|a| language(a) == lang).collect();
        if same.is_empty() {
            continue;
        }
        if lang == "zh" {
            let want = chinese_script(&rc);
            if let Some(a) = same.iter().find(|a| chinese_script(a) == want) {
                return Some(a);
            }
        }
        // The bare language first, then the first listed.
        return Some(
            same.iter()
                .find(|a| canonical(a) == lang)
                .unwrap_or(&same[0]),
        );
    }
    None
}

/// The direction of a locale's script: Arabic, Hebrew, Persian, Urdu,
/// Pashto, Sindhi, Yiddish, Dhivehi and Uyghur are right to left.
pub fn direction_of_locale(tag: &str) -> Direction {
    match language(tag).as_str() {
        "ar" | "he" | "iw" | "fa" | "ur" | "ps" | "sd" | "yi" | "dv" | "ug" | "ckb" => {
            Direction::Rtl
        }
        _ => Direction::Ltr,
    }
}
