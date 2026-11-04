//! Locale-aware uppercasing (CSS `text-transform: uppercase`, a type
//! step's `case upper`): Unicode's full case mapping (`ß` → `SS`) with the
//! language-specific rules of SpecialCasing.txt and ICU, so the capitals
//! read as that language writes them:
//!
//! - Turkish and Azerbaijani: `i` → `İ` (the dotted capital), `ı` → `I`.
//! - Lithuanian: a combining dot above kept after `i` or `j` to keep its
//!   dot in lowercase is dropped in capitals (`i̇` → `I`).
//! - Greek: capitals carry no accents or breathings (`Καλημέρα` →
//!   `ΚΑΛΗΜΕΡΑ`); the dialytika is kept, and added to an `ι` or `υ` after
//!   an accented vowel, where the accent showed the two vowels apart
//!   (`Μάιος` → `ΜΑΪΟΣ`); the iota subscript becomes a capital `Ι`; a lone
//!   `ή` ("or") keeps its tonos (`Ή`), as it does in every language with
//!   Greek text. A port of ICU's `GreekUpper` (icu4c `ustrcase.cpp` and
//!   ICU4X `icu_casemap`'s Greek uppercasing; © 2016 and later Unicode,
//!   Inc. and others, the Unicode License v3), its letter data written as
//!   the ranges of the Greek and Greek Extended blocks.

/// The text in capitals as `locale` (a BCP 47 tag, `tr-TR`, `el`) writes
/// them.
pub fn to_upper(text: &str, locale: &str) -> String {
    let lang = locale
        .split(['-', '_'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    match lang.as_str() {
        "tr" | "az" => text
            .chars()
            .map(|c| if c == 'i' { 'İ' } else { c })
            .flat_map(char::to_uppercase)
            .collect(),
        "lt" => {
            let mut out = String::with_capacity(text.len());
            let mut soft_dotted = false;
            for c in text.chars() {
                if c == '\u{307}' && soft_dotted {
                    continue;
                }
                // Marks between the letter and its dot leave it soft-dotted.
                if !is_mark_above(c) {
                    soft_dotted = matches!(c, 'i' | 'j' | 'į' | 'ɨ' | 'ʝ' | 'ⅈ' | 'ⅉ');
                }
                out.extend(c.to_uppercase());
            }
            out
        }
        _ if text.chars().any(is_greek) => greek_upper(text),
        _ => text.to_uppercase(),
    }
}

/// Combining marks of class 230 (above) that Lithuanian writes between a
/// soft-dotted letter and its kept dot.
fn is_mark_above(c: char) -> bool {
    matches!(c, '\u{300}' | '\u{301}' | '\u{303}')
}

fn is_greek(c: char) -> bool {
    matches!(c, '\u{370}'..='\u{3ff}' | '\u{1f00}'..='\u{1fff}')
}

// The letter data flags (ICU's GreekUpper).
const VOWEL: u8 = 1;
const YPOGEGRAMMENI: u8 = 2;
const ACCENT: u8 = 4;
const DIALYTIKA: u8 = 8;
/// A dialytika as a combining mark after the letter.
const COMBINING_DIALYTIKA: u8 = 16;
/// Breathings, vrachy, macron: dropped like accents.
const OTHER: u8 = 32;

/// A Greek letter's capital without diacritics and what it carried; a
/// combining Greek diacritic as `('\0', its flag)`.
fn greek_data(c: char) -> Option<(char, u8)> {
    const A: char = 'Α';
    const E: char = 'Ε';
    const H: char = 'Η';
    const I: char = 'Ι';
    const O: char = 'Ο';
    const Y: char = 'Υ';
    const W: char = 'Ω';
    const R: char = 'Ρ';
    let v = |base: char, f: u8| Some((base, VOWEL | f));
    let u = c as u32;
    match c {
        '\u{300}' | '\u{301}' | '\u{342}' | '\u{302}' | '\u{303}' | '\u{311}' => {
            Some(('\0', ACCENT))
        }
        '\u{308}' => Some(('\0', COMBINING_DIALYTIKA)),
        '\u{344}' => Some(('\0', COMBINING_DIALYTIKA | ACCENT)),
        '\u{345}' => Some(('\0', YPOGEGRAMMENI)),
        '\u{304}' | '\u{306}' | '\u{313}' | '\u{314}' | '\u{343}' => Some(('\0', OTHER)),
        'Ά' | 'ά' => v(A, ACCENT),
        'Έ' | 'έ' => v(E, ACCENT),
        'Ή' | 'ή' => v(H, ACCENT),
        'Ί' | 'ί' => v(I, ACCENT),
        'Ό' | 'ό' => v(O, ACCENT),
        'Ύ' | 'ύ' => v(Y, ACCENT),
        'Ώ' | 'ώ' => v(W, ACCENT),
        'ΐ' => v(I, ACCENT | DIALYTIKA),
        'ΰ' => v(Y, ACCENT | DIALYTIKA),
        'Ϊ' | 'ϊ' => v(I, DIALYTIKA),
        'Ϋ' | 'ϋ' => v(Y, DIALYTIKA),
        'α' | 'Α' => v(A, 0),
        'ε' | 'Ε' => v(E, 0),
        'η' | 'Η' => v(H, 0),
        'ι' | 'Ι' => v(I, 0),
        'ο' | 'Ο' => v(O, 0),
        'υ' | 'Υ' => v(Y, 0),
        'ω' | 'Ω' => v(W, 0),
        '\u{1f00}'..='\u{1f6f}' => {
            // Rows of sixteen, eight lowercase then eight capitals, each
            // letter with a breathing and maybe an accent.
            let base = [A, E, H, I, O, Y, W][((u - 0x1f00) / 16) as usize];
            let accent = if (u & 7) >= 2 { ACCENT } else { 0 };
            v(base, OTHER | accent)
        }
        '\u{1f70}'..='\u{1f7d}' => v([A, E, H, I, O, Y, W][((u - 0x1f70) / 2) as usize], ACCENT),
        '\u{1f80}'..='\u{1faf}' => {
            let base = [A, H, W][((u - 0x1f80) / 16) as usize];
            let accent = if (u & 7) >= 2 { ACCENT } else { 0 };
            v(base, YPOGEGRAMMENI | OTHER | accent)
        }
        '\u{1fb0}'..='\u{1fff}' => {
            // The rest of Greek Extended, a row of sixteen per letter pair
            // (ᾰ … ᾼ, ῂ … ῌ, ῐ … Ί, ῠ … Ῥ, ῲ … ῼ); the empty slots hold
            // spacing marks.
            if matches!(
                u,
                0x1fb5
                    | 0x1fbd..=0x1fc1
                    | 0x1fc5
                    | 0x1fcd..=0x1fcf
                    | 0x1fd4
                    | 0x1fd5
                    | 0x1fdc..=0x1fdf
                    | 0x1fed..=0x1ff1
                    | 0x1ff5
                    | 0x1ffd..=0x1fff
            ) {
                return None;
            }
            let (row, col) = ((u - 0x1fb0) / 16, u & 0xf);
            let letter = match (row, col) {
                (0, _) => A,
                (1, 0x8 | 0x9) => E,
                (1, _) => H,
                (2, _) => I,
                (3, 0x4 | 0x5 | 0xc) => return Some((R, OTHER)),
                (3, _) => Y,
                (4, 0x8 | 0x9) => O,
                _ => W,
            };
            let f = match col {
                // Vrachy and macron (ᾰ ᾱ Ᾰ Ᾱ), or (rows 1 and 4) Ὲ Έ Ὸ Ό.
                0x0 | 0x1 if row != 1 && row != 4 => OTHER,
                0x8 | 0x9 if row == 1 || row == 4 => ACCENT,
                0x8 | 0x9 => OTHER,
                0x2 | 0x3 | 0x7 if row == 2 || row == 3 => DIALYTIKA | ACCENT,
                0x2 | 0x4 | 0x7 => YPOGEGRAMMENI | ACCENT,
                0x3 | 0xc => YPOGEGRAMMENI,
                0x6 | 0xa | 0xb => ACCENT,
                _ => return None,
            };
            v(letter, f)
        }
        _ => None,
    }
}

/// Whether `c` is a letter with case (ICU's `isCased`, as near as `char`
/// says it).
fn cased(c: char) -> bool {
    c.is_lowercase() || c.is_uppercase()
}

/// ICU's GreekUpper over a text with Greek in it.
fn greek_upper(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    // Whether the last letter was cased, and whether it was a vowel with
    // an accent and no dialytika.
    let (mut after_cased, mut after_accented_vowel) = (false, false);
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        i += 1;
        let Some((upper, mut data)) = greek_data(c).filter(|(u, _)| *u != '\0') else {
            // Not a Greek letter (a combining Greek mark here follows no
            // Greek letter, and is kept): its plain capitals.
            out.extend(c.to_uppercase());
            after_cased = cased(c);
            after_accented_vowel = false;
            continue;
        };
        // The combining diacritics after it count as its own.
        let mut ypo = u8::from(data & YPOGEGRAMMENI != 0);
        while i < chars.len()
            && let Some(('\0', f)) = greek_data(chars[i])
        {
            data |= f;
            if f & YPOGEGRAMMENI != 0 {
                ypo += 1;
            }
            i += 1;
        }
        if data & VOWEL != 0 && after_accented_vowel && (upper == 'Ι' || upper == 'Υ') {
            data |= DIALYTIKA;
        }
        let next_accented =
            data & (VOWEL | ACCENT | DIALYTIKA | COMBINING_DIALYTIKA) == (VOWEL | ACCENT);
        let mut add_tonos = false;
        if upper == 'Η'
            && data & ACCENT != 0
            && ypo == 0
            && data & (DIALYTIKA | COMBINING_DIALYTIKA | OTHER) == 0
            && !after_cased
            && !followed_by_cased(&chars[i..])
        {
            // The disjunctive ή ("or") keeps its tonos.
            add_tonos = c == 'ή' || c == 'Ή' || c == 'η' || c == 'Η';
        }
        let dialytika = data & (DIALYTIKA | COMBINING_DIALYTIKA) != 0;
        match (upper, dialytika, add_tonos) {
            ('Ι', true, _) => out.push('Ϊ'),
            ('Υ', true, _) => out.push('Ϋ'),
            ('Η', _, true) => out.push('Ή'),
            (u, true, _) => {
                out.push(u);
                out.push('\u{308}');
            }
            (u, _, _) => out.push(u),
        }
        for _ in 0..ypo {
            out.push('Ι');
        }
        after_cased = true;
        after_accented_vowel = next_accented;
    }
    out
}

/// Whether a cased letter follows, past case-ignorable marks (ICU's
/// Final_Sigma context, which the disjunctive ή reads).
fn followed_by_cased(rest: &[char]) -> bool {
    rest.iter()
        .find(|c| !matches!(c, '\u{300}'..='\u{36f}' | '\'' | '’' | '.' | ':' | '·'))
        .is_some_and(|c| cased(*c))
}

#[cfg(test)]
mod tests {
    use super::to_upper;

    #[test]
    fn capitals_follow_the_language() {
        // Full mappings, every language.
        assert_eq!(to_upper("Straße", "de"), "STRASSE");
        // Turkish and Azerbaijani keep the dot; English does not add one.
        assert_eq!(to_upper("istanbul ılık", "tr-TR"), "İSTANBUL ILIK");
        assert_eq!(to_upper("bilgi", "az"), "BİLGİ");
        assert_eq!(to_upper("bilgi", "en"), "BILGI");
        // Lithuanian drops the dot it kept over an accented i.
        assert_eq!(to_upper("i\u{307}\u{300}", "lt"), "I\u{300}");
        assert_eq!(to_upper("i\u{307}\u{300}", "en"), "I\u{307}\u{300}");
    }

    #[test]
    fn greek_capitals_drop_their_accents() {
        assert_eq!(to_upper("Καλημέρα", "el"), "ΚΑΛΗΜΕΡΑ");
        // An accent that kept two vowels apart becomes a dialytika.
        assert_eq!(to_upper("Μάιος", "el"), "ΜΑΪΟΣ");
        // Accented on the second vowel, a diphthong: nothing added.
        assert_eq!(to_upper("παιδί", "el"), "ΠΑΙΔΙ");
        assert_eq!(to_upper("ϊ ΐ", "el"), "Ϊ Ϊ");
        // The disjunctive ή keeps its tonos; inside a word it does not.
        assert_eq!(to_upper("ναι ή όχι", "el"), "ΝΑΙ Ή ΟΧΙ");
        assert_eq!(to_upper("ήλιος", "el"), "ΗΛΙΟΣ");
        // Polytonic: breathings and accents go, the subscript is an Ι.
        assert_eq!(to_upper("ᾠδή ἄλφα", "el"), "ΩΙΔΗ ΑΛΦΑ");
        assert_eq!(to_upper("ῥήτωρ", "el"), "ΡΗΤΩΡ");
        // Decomposed accents too, and Greek in a menu of another language.
        assert_eq!(to_upper("α\u{301}λφα", "en"), "ΑΛΦΑ");
        assert_eq!(to_upper("Score: Μάιος", "fr"), "SCORE: ΜΑΪΟΣ");
    }
}
