//! CLDR cardinal plural rules for the languages the engine's knowledge
//! names (and their families): integers and decimals by `n`, `i` (integer
//! digits) and `v` (visible fraction digits), as CLDR defines them.

/// A plural category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluralCategory {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

impl PluralCategory {
    pub fn keyword(self) -> &'static str {
        match self {
            PluralCategory::Zero => "zero",
            PluralCategory::One => "one",
            PluralCategory::Two => "two",
            PluralCategory::Few => "few",
            PluralCategory::Many => "many",
            PluralCategory::Other => "other",
        }
    }
}

/// The category of `n` in `language` (a primary subtag: `en`, `ar`, `ja`).
pub fn plural_category(language: &str, n: f64) -> PluralCategory {
    use PluralCategory::*;
    let n = n.abs();
    let i = n.trunc() as u64;
    let integer = n.fract() == 0.0;
    let v = if integer { 0 } else { 1 };
    let i10 = i % 10;
    let i100 = i % 100;
    match language {
        // No plural forms.
        "ja" | "ko" | "zh" | "th" | "vi" | "id" | "ms" | "my" | "lo" | "km" => Other,
        "en" | "de" | "nl" | "sv" | "da" | "nb" | "no" | "fi" | "et" | "it" | "el" | "hu"
        | "tr" | "bg" | "ca" | "gl" | "eu" => {
            if i == 1 && v == 0 {
                One
            } else {
                Other
            }
        }
        "es" => {
            if n == 1.0 {
                One
            } else if i != 0 && i.is_multiple_of(1_000_000) && v == 0 {
                Many
            } else {
                Other
            }
        }
        "pt" | "fr" => {
            if i <= 1 {
                One
            } else if i != 0 && i.is_multiple_of(1_000_000) && v == 0 {
                Many
            } else {
                Other
            }
        }
        "hi" | "bn" | "fa" | "gu" | "kn" | "mr" | "zu" | "am" => {
            if i == 0 || n == 1.0 {
                One
            } else {
                Other
            }
        }
        "ru" | "uk" | "be" => {
            if v != 0 {
                Other
            } else if i10 == 1 && i100 != 11 {
                One
            } else if (2..=4).contains(&i10) && !(12..=14).contains(&i100) {
                Few
            } else {
                Many
            }
        }
        "pl" => {
            if v != 0 {
                Other
            } else if i == 1 {
                One
            } else if (2..=4).contains(&i10) && !(12..=14).contains(&i100) {
                Few
            } else {
                Many
            }
        }
        "cs" | "sk" => {
            if v != 0 {
                Many
            } else if i == 1 {
                One
            } else if (2..=4).contains(&i) {
                Few
            } else {
                Other
            }
        }
        "ar" => {
            if !integer {
                Other
            } else if i == 0 {
                Zero
            } else if i == 1 {
                One
            } else if i == 2 {
                Two
            } else if (3..=10).contains(&i100) {
                Few
            } else if (11..=99).contains(&i100) {
                Many
            } else {
                Other
            }
        }
        "he" | "iw" => {
            if i == 1 && v == 0 {
                One
            } else if i == 2 && v == 0 {
                Two
            } else {
                Other
            }
        }
        _ => {
            if i == 1 && v == 0 {
                One
            } else {
                Other
            }
        }
    }
}

/// `n` as an ordinal in `language` (a primary subtag): `1st`, `22nd`,
/// `1er`/`2e`, `1.`, `1.º`, `第1`, `1번째`. CLDR's ordinal categories for
/// English (one 1, 21; two 2, 22; few 3, 23; other), the others by their
/// one written form. Integers; a fraction is cut toward zero.
pub fn format_ordinal(n: f64, language: &str) -> String {
    let i = n.trunc() as i64;
    let a = i.unsigned_abs();
    match language {
        "en" => {
            let suffix = match (a % 10, a % 100) {
                (1, x) if x != 11 => "st",
                (2, x) if x != 12 => "nd",
                (3, x) if x != 13 => "rd",
                _ => "th",
            };
            format!("{i}{suffix}")
        }
        "fr" => format!("{i}{}", if a == 1 { "er" } else { "e" }),
        "es" | "pt" | "it" | "gl" => format!("{i}.º"),
        "de" | "da" | "nb" | "no" | "fi" | "et" | "cs" | "sk" | "pl" | "tr" | "hu" => {
            format!("{i}.")
        }
        "nl" => format!("{i}e"),
        "sv" => {
            let suffix = match (a % 10, a % 100) {
                (1 | 2, x) if x != 11 && x != 12 => ":a",
                _ => ":e",
            };
            format!("{i}{suffix}")
        }
        "ru" | "uk" | "be" | "bg" => format!("{i}-й"),
        "ja" => format!("{i}番目"),
        "zh" => format!("第{i}"),
        "ko" => format!("{i}번째"),
        _ => format!("{i}."),
    }
}

#[cfg(test)]
mod ordinal_tests {
    use super::format_ordinal;

    #[test]
    fn english_ordinals_follow_the_teens_rule() {
        let got: Vec<String> = [
            1.0, 2.0, 3.0, 4.0, 11.0, 12.0, 13.0, 21.0, 22.0, 23.0, 101.0, 111.0,
        ]
        .iter()
        .map(|&n| format_ordinal(n, "en"))
        .collect();
        assert_eq!(
            got,
            [
                "1st", "2nd", "3rd", "4th", "11th", "12th", "13th", "21st", "22nd", "23rd",
                "101st", "111th"
            ]
        );
        assert_eq!(format_ordinal(1.0, "fr"), "1er");
        assert_eq!(format_ordinal(2.0, "fr"), "2e");
        assert_eq!(format_ordinal(3.0, "zh"), "第3");
    }
}
