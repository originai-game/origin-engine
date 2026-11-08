//! What kind of text a field takes, by the HTML names (`docs/ui.md` §2.3):
//! `type` (text, password, number, email, url, tel, search) masks a
//! password, filters what a number field accepts and says when the text is
//! a type mismatch (the `:invalid` state); `inputmode` picks the virtual
//! keyboard; `autocomplete` is the page's autofill hint. A host reads all
//! three from [`crate::TextTarget`].
//!
//! The validity rules are the HTML Living Standard's (§4.10.5.1 "States of
//! the type attribute"): a valid floating-point number (§2.3.4.3), a valid
//! email address (§4.10.5.1.5, its regular expression), a valid absolute URL
//! (a scheme, `:`, and for the special schemes an authority after `//`).
//! An empty field is never a mismatch (that is `required`'s job, which a
//! game does with `invalid`).

/// A field's `type` ([`crate::FieldDesc::input_type`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum InputType {
    #[default]
    Text,
    /// Drawn masked, never copied or cut, with a reveal toggle.
    Password,
    /// A decimal number: other characters are not typed; the decimal keypad.
    Number,
    Email,
    Url,
    /// A telephone number: the phone keypad; no validity rule (HTML has none).
    Tel,
    /// A search box: the keyboard's Enter key reads "search", Escape clears it.
    Search,
}

impl InputType {
    /// Every type's HTML name, in declaration order.
    pub const NAMES: [&'static str; 7] = [
        "text", "password", "number", "email", "url", "tel", "search",
    ];

    /// The type an HTML `type` value names.
    pub fn from_name(name: &str) -> Option<InputType> {
        Some(match name {
            "text" => InputType::Text,
            "password" => InputType::Password,
            "number" => InputType::Number,
            "email" => InputType::Email,
            "url" => InputType::Url,
            "tel" => InputType::Tel,
            "search" => InputType::Search,
            _ => return None,
        })
    }

    /// Its HTML name.
    pub fn name(self) -> &'static str {
        Self::NAMES[self as usize]
    }

    /// The virtual keyboard the type asks for when `inputmode` is unset
    /// (what browsers give the same `<input type>`).
    pub fn default_mode(self) -> InputMode {
        match self {
            InputType::Text | InputType::Password => InputMode::Text,
            InputType::Number => InputMode::Decimal,
            InputType::Email => InputMode::Email,
            InputType::Url => InputMode::Url,
            InputType::Tel => InputMode::Tel,
            InputType::Search => InputMode::Search,
        }
    }

    /// Whether a typed character may go in (a number field takes digits,
    /// a sign, a decimal point and an exponent; every other type takes all).
    pub fn accepts(self, c: char) -> bool {
        match self {
            InputType::Number => c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e' | 'E'),
            _ => true,
        }
    }

    /// Whether `value` is a type mismatch: not empty, and not what the type
    /// takes (HTML's `typeMismatch`, and for a number its `badInput`).
    pub fn mismatch(self, value: &str) -> bool {
        if value.is_empty() {
            return false;
        }
        match self {
            InputType::Number => !valid_number(value),
            InputType::Email => !valid_email(value),
            InputType::Url => !valid_url(value),
            InputType::Text | InputType::Password | InputType::Tel | InputType::Search => false,
        }
    }
}

/// A field's `inputmode`: the virtual keyboard a touch device shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum InputMode {
    /// No virtual keyboard: the game shows its own.
    None,
    #[default]
    Text,
    Decimal,
    Numeric,
    Tel,
    Search,
    Email,
    Url,
}

impl InputMode {
    /// Every mode's HTML name, in declaration order.
    pub const NAMES: [&'static str; 8] = [
        "none", "text", "decimal", "numeric", "tel", "search", "email", "url",
    ];

    /// The mode an HTML `inputmode` value names.
    pub fn from_name(name: &str) -> Option<InputMode> {
        Self::NAMES
            .iter()
            .position(|n| *n == name)
            .map(|i| match i {
                0 => InputMode::None,
                1 => InputMode::Text,
                2 => InputMode::Decimal,
                3 => InputMode::Numeric,
                4 => InputMode::Tel,
                5 => InputMode::Search,
                6 => InputMode::Email,
                _ => InputMode::Url,
            })
    }

    /// Its HTML name.
    pub fn name(self) -> &'static str {
        Self::NAMES[self as usize]
    }
}

/// The character a password's text is drawn with (U+2022 BULLET; `*`
/// where the face lacks it).
pub(crate) const MASK: char = '\u{2022}';

/// HTML's valid floating-point number: `-`? then digits with an optional
/// fraction, or a fraction alone, then an optional exponent.
fn valid_number(s: &str) -> bool {
    let b = s.as_bytes();
    let mut i = usize::from(b.first() == Some(&b'-'));
    let digits = |i: &mut usize| {
        let start = *i;
        while *i < b.len() && b[*i].is_ascii_digit() {
            *i += 1;
        }
        *i - start
    };
    let whole = digits(&mut i);
    let mut fraction = 0;
    if b.get(i) == Some(&b'.') {
        i += 1;
        fraction = digits(&mut i);
        if fraction == 0 {
            return false;
        }
    }
    if whole + fraction == 0 {
        return false;
    }
    if matches!(b.get(i), Some(b'e' | b'E')) {
        i += 1;
        if matches!(b.get(i), Some(b'+' | b'-')) {
            i += 1;
        }
        if digits(&mut i) == 0 {
            return false;
        }
    }
    i == b.len()
}

/// HTML's valid email address:
/// `[a-zA-Z0-9.!#$%&'*+/=?^_\`{|}~-]+@label(.label)*`, each label 1 to 63
/// letters, digits and hyphens, not starting or ending with a hyphen.
fn valid_email(s: &str) -> bool {
    let Some((local, domain)) = s.split_once('@') else {
        return false;
    };
    let atext = |c: char| c.is_ascii_alphanumeric() || ".!#$%&'*+/=?^_`{|}~-".contains(c);
    !local.is_empty()
        && local.chars().all(atext)
        && domain.split('.').all(|label| {
            let b = label.as_bytes();
            (1..=63).contains(&b.len())
                && b.iter().all(|c| c.is_ascii_alphanumeric() || *c == b'-')
                && b[0] != b'-'
                && b[b.len() - 1] != b'-'
        })
}

/// A valid absolute URL: a scheme (a letter, then letters, digits, `+`,
/// `-` and `.`), `:`, then no white space; the special schemes (`http`,
/// `https`, `ws`, `wss`, `ftp`) need a host, after any slashes (the URL
/// parser reads `https:example.com` as `https://example.com/`).
fn valid_url(s: &str) -> bool {
    let Some((scheme, rest)) = s.split_once(':') else {
        return false;
    };
    let mut cs = scheme.chars();
    if !cs.next().is_some_and(|c| c.is_ascii_alphabetic())
        || !cs.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
        || rest.chars().any(char::is_whitespace)
    {
        return false;
    }
    let special =
        ["http", "https", "ws", "wss", "ftp"].contains(&scheme.to_ascii_lowercase().as_str());
    if !special {
        return true;
    }
    let host = rest.trim_start_matches(['/', '\\']);
    let host = host.split(['/', '?', '#']).next().unwrap_or_default();
    let host = host.rsplit_once('@').map_or(host, |(_, h)| h);
    !host.is_empty() && !host.starts_with(':')
}

#[cfg(test)]
mod tests {
    use super::*;

    // Cases from the HTML standard's prose and the WPT
    // `html/semantics/forms/constraints` type-mismatch tests.
    #[test]
    fn numbers_follow_the_html_grammar() {
        for ok in ["0", "-1", "1.5", ".5", "1e3", "1E-3", "-0.25e+2", "007"] {
            assert!(!InputType::Number.mismatch(ok), "{ok}");
        }
        for bad in [
            "+1", "1.", ".", "-", "1e", "1e+", "1,5", " 1", "1 ", "--1", "1.2.3", "e3",
        ] {
            assert!(InputType::Number.mismatch(bad), "{bad}");
        }
    }

    #[test]
    fn emails_follow_the_html_expression() {
        for ok in ["a@b", "first.last+tag@example.co.uk", "x@1.2", "a@a-b.c"] {
            assert!(!InputType::Email.mismatch(ok), "{ok}");
        }
        for bad in [
            "a", "a@", "@b", "a@b.", "a@-b.c", "a@b-.c", "a b@c", "a@b@c", "a@b..c",
        ] {
            assert!(InputType::Email.mismatch(bad), "{bad}");
        }
    }

    #[test]
    fn urls_need_a_scheme_and_a_host_where_special() {
        for ok in [
            "https://example.com",
            "http://a/b?c#d",
            "mailto:x@y",
            "og:thing",
            "ftp://u@h",
        ] {
            assert!(!InputType::Url.mismatch(ok), "{ok}");
        }
        for bad in [
            "example.com",
            "https://",
            "http:///",
            "1http://a",
            "https://a b",
            "://a",
        ] {
            assert!(InputType::Url.mismatch(bad), "{bad}");
        }
    }

    #[test]
    fn empty_is_never_a_mismatch_and_text_types_never_are() {
        for t in [InputType::Number, InputType::Email, InputType::Url] {
            assert!(!t.mismatch(""));
        }
        for t in [
            InputType::Text,
            InputType::Password,
            InputType::Tel,
            InputType::Search,
        ] {
            assert!(!t.mismatch("anything at all"));
        }
    }

    #[test]
    fn names_round_trip_and_types_pick_their_keyboard() {
        for n in InputType::NAMES {
            assert_eq!(InputType::from_name(n).map(InputType::name), Some(n));
        }
        for n in InputMode::NAMES {
            assert_eq!(InputMode::from_name(n).map(InputMode::name), Some(n));
        }
        assert_eq!(InputType::Number.default_mode(), InputMode::Decimal);
        assert_eq!(InputType::Tel.default_mode(), InputMode::Tel);
        assert_eq!(InputType::Password.default_mode(), InputMode::Text);
        assert!(InputType::Number.accepts('7') && !InputType::Number.accepts('x'));
    }
}
