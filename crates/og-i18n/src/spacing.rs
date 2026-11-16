//! The character classes CSS Text 4 spaces and hangs text by: fullwidth
//! punctuation, which `text-spacing-trim` sets half-width (§8.5.3),
//! ideographs beside other letters and numerals, between which
//! `text-autospace` puts room (§8.4.1), and the brackets, quotes, stops
//! and commas `hanging-punctuation` hangs (§9.2.1).

/// A class of CSS Text 4 §8.5.3's fullwidth punctuation, which
/// `text-spacing-trim` sets half-width.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fullwidth {
    /// Its start half is blank.
    Opening,
    /// Its end half is blank.
    Closing,
    /// Blank on both sides, its ink centred.
    MiddleDot,
    /// U+3000 IDEOGRAPHIC SPACE, which only neighbours trim against.
    Space,
}

/// The §8.5.3 fullwidth punctuation class of `c` in content language
/// `lang` (BCP 47): opening and closing punctuation (Ps, Pe) in the CJK
/// Symbols and Punctuation block or East Asian Fullwidth, the curly quotes,
/// the middle dots; the fullwidth colons and dots as the language draws
/// them (§8.5.3's table, horizontal): Simplified Chinese at the start
/// side, so closing; Traditional Chinese centred; Korean and Japanese
/// (and an unknown language) the colon centred and the dots closing.
pub fn fullwidth(c: char, lang: Option<&str>) -> Option<Fullwidth> {
    use icu_properties::CodePointMapData;
    use icu_properties::props::GeneralCategory as G;
    let (colon, dot) = match lang.map(str::to_ascii_lowercase).as_deref() {
        Some(l) if l.starts_with("zh") => {
            let hant = ["-hant", "-tw", "-hk", "-mo"].iter().any(|t| l.contains(t));
            if hant {
                (Fullwidth::MiddleDot, Fullwidth::MiddleDot)
            } else {
                (Fullwidth::Closing, Fullwidth::Closing)
            }
        }
        _ => (Fullwidth::MiddleDot, Fullwidth::Closing),
    };
    Some(match c {
        '\u{3000}' => Fullwidth::Space,
        '\u{2018}' | '\u{201c}' => Fullwidth::Opening,
        '\u{2019}' | '\u{201d}' => Fullwidth::Closing,
        '\u{b7}' | '\u{2027}' | '\u{30fb}' => Fullwidth::MiddleDot,
        '\u{ff1a}' | '\u{ff1b}' => colon,
        '\u{3001}' | '\u{3002}' | '\u{ff0c}' | '\u{ff0e}' => dot,
        c if ('\u{3000}'..='\u{303f}').contains(&c) || east_asian_fullwidth(c) => {
            match CodePointMapData::<G>::new().get(c) {
                G::OpenPunctuation => Fullwidth::Opening,
                G::ClosePunctuation => Fullwidth::Closing,
                _ => return None,
            }
        }
        _ => return None,
    })
}

/// Whether `c` is East Asian Fullwidth (F) by UAX #11.
fn east_asian_fullwidth(c: char) -> bool {
    use icu_properties::CodePointMapData;
    use icu_properties::props::EastAsianWidth;
    CodePointMapData::<EastAsianWidth>::new().get(c) == EastAsianWidth::Fullwidth
}

/// Whether `c` is opening (Ps) or closing (Pe) punctuation of any width,
/// which §8.5.2 trims a fullwidth neighbour against.
pub fn bracket(c: char) -> Option<Fullwidth> {
    use icu_properties::CodePointMapData;
    use icu_properties::props::GeneralCategory as G;
    match CodePointMapData::<G>::new().get(c) {
        G::OpenPunctuation => Some(Fullwidth::Opening),
        G::ClosePunctuation => Some(Fullwidth::Closing),
        _ => None,
    }
}

/// A class of CSS Text 4 §8.4.1, between which `text-autospace` puts an
/// eighth of an ideograph's advance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterScript {
    Ideograph,
    /// A non-ideographic letter or mark.
    Letter,
    /// A non-ideographic decimal digit.
    Numeral,
}

/// The §8.4.1 class of `c` set horizontally: ideographs are the kana
/// block's letters (U+3041 to U+30FF but punctuation), CJK strokes,
/// Katakana phonetic extensions and every other character whose
/// Script_Extensions has Han but punctuation, symbols and spaces; letters and marks (L*, M*) and decimal
/// digits (Nd) are non-ideographic unless East Asian Wide or Fullwidth.
pub fn inter_script(c: char) -> Option<InterScript> {
    use icu_properties::CodePointMapData;
    use icu_properties::props::{
        EastAsianWidth as W, GeneralCategory as G, GeneralCategoryGroup as Group, Script,
    };
    use icu_properties::script::ScriptWithExtensions;
    let gc = CodePointMapData::<G>::new().get(c);
    let punctuation = Group::Punctuation.contains(gc);
    // Han's punctuation and symbols (`、`, `〒`) name Han among their
    // scripts too, but are no ideographs.
    let han = ScriptWithExtensions::new().has_script(c, Script::Han)
        && !punctuation
        && !Group::Symbol.contains(gc)
        && !Group::Separator.contains(gc);
    if (('\u{3041}'..='\u{30ff}').contains(&c) && !punctuation)
        || ('\u{31c0}'..='\u{31ff}').contains(&c)
        || han
    {
        return Some(InterScript::Ideograph);
    }
    let width = CodePointMapData::<W>::new().get(c);
    if matches!(width, W::Wide | W::Fullwidth) {
        return None;
    }
    if gc == G::DecimalNumber {
        Some(InterScript::Numeral)
    } else if Group::Letter.contains(gc) || Group::Mark.contains(gc) {
        Some(InterScript::Letter)
    } else {
        None
    }
}

/// Whether `c` is a space character of any kind (General Category Z*),
/// which `text-autospace` does not add to.
pub fn separator(c: char) -> bool {
    use icu_properties::CodePointMapData;
    use icu_properties::props::{GeneralCategory as G, GeneralCategoryGroup as Group};
    Group::Separator.contains(CodePointMapData::<G>::new().get(c))
}

/// Which edge of a line §9.2.1's `hanging-punctuation` may hang `c` at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hangs {
    /// An opening bracket or quote, or an ideographic space (`first`).
    Opening,
    /// A closing bracket or quote (`last`).
    Closing,
    /// A quote that opens or closes (Pi, Pf, `'` and `"`): either.
    Quote,
    /// A stop or comma (`force-end`, `allow-end`).
    Stop,
}

/// The §9.2.1 class of `c`: Ps and U+3000 open, Pe closes, Pi, Pf and
/// the ASCII quotes do either; the stops and commas are §9.2.1's list.
pub fn hangs(c: char) -> Option<Hangs> {
    use icu_properties::CodePointMapData;
    use icu_properties::props::GeneralCategory as G;
    if matches!(
        c,
        ',' | '.'
            | '\u{60c}'
            | '\u{6d4}'
            | '\u{3001}'
            | '\u{3002}'
            | '\u{ff0c}'
            | '\u{ff0e}'
            | '\u{fe50}'
            | '\u{fe51}'
            | '\u{fe52}'
            | '\u{ff61}'
            | '\u{ff64}'
    ) {
        return Some(Hangs::Stop);
    }
    match c {
        '\'' | '"' => return Some(Hangs::Quote),
        '\u{3000}' => return Some(Hangs::Opening),
        _ => {}
    }
    match CodePointMapData::<G>::new().get(c) {
        G::OpenPunctuation => Some(Hangs::Opening),
        G::ClosePunctuation => Some(Hangs::Closing),
        G::InitialPunctuation | G::FinalPunctuation => Some(Hangs::Quote),
        _ => None,
    }
}
