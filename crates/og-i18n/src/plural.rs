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
