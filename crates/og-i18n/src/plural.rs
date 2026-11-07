//! CLDR cardinal and ordinal categories from ICU's compiled data, shared
//! by message formatting and Yarn replacement markup on every host.

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

/// The cardinal category of `n` in a BCP 47 locale (`en`, `pt-PT`, `ar`).
pub fn plural_category(locale: &str, n: f64) -> PluralCategory {
    category(locale, n, false)
}

/// The ordinal category used by localized selection messages, not a suffix.
pub fn ordinal_category(locale: &str, n: f64) -> PluralCategory {
    category(locale, n, true)
}

fn category(locale: &str, n: f64, ordinal: bool) -> PluralCategory {
    use icu_plurals::{PluralCategory as Icu, PluralOperands, PluralRules};
    let locale = crate::locale::canonical(locale)
        .parse::<icu_locale_core::Locale>()
        .unwrap_or(icu_locale_core::locale!("und"));
    let rules = if ordinal {
        PluralRules::try_new_ordinal(locale.into())
    } else {
        PluralRules::try_new_cardinal(locale.into())
    };
    let Ok(decimal) = n.abs().to_string().parse::<fixed_decimal::Decimal>() else {
        return PluralCategory::Other;
    };
    let operands = PluralOperands::from(&decimal);
    match rules
        .map(|r| r.category_for(operands))
        .unwrap_or(Icu::Other)
    {
        Icu::Zero => PluralCategory::Zero,
        Icu::One => PluralCategory::One,
        Icu::Two => PluralCategory::Two,
        Icu::Few => PluralCategory::Few,
        Icu::Many => PluralCategory::Many,
        _ => PluralCategory::Other,
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
