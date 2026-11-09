//! Key codes: the `KeyboardEvent.code` names the input layer knows, in the
//! bit order of [`crate::Input`]'s key sets, and the US-layout table `type`
//! uses.

/// The `KeyboardEvent.code` names the input layer knows, in bit order of
/// [`crate::Input`]'s key sets (bit `i` of the 128-bit set is word `i / 32`,
/// bit `i % 32`). A code outside this table is refused. The order is part of
/// the `og.Input` layout's meaning: append, never reorder.
pub const KEY_CODES: &[&str] = &[
    "KeyA",
    "KeyB",
    "KeyC",
    "KeyD",
    "KeyE",
    "KeyF",
    "KeyG",
    "KeyH",
    "KeyI",
    "KeyJ",
    "KeyK",
    "KeyL",
    "KeyM",
    "KeyN",
    "KeyO",
    "KeyP",
    "KeyQ",
    "KeyR",
    "KeyS",
    "KeyT",
    "KeyU",
    "KeyV",
    "KeyW",
    "KeyX",
    "KeyY",
    "KeyZ",
    "Digit0",
    "Digit1",
    "Digit2",
    "Digit3",
    "Digit4",
    "Digit5",
    "Digit6",
    "Digit7",
    "Digit8",
    "Digit9",
    "F1",
    "F2",
    "F3",
    "F4",
    "F5",
    "F6",
    "F7",
    "F8",
    "F9",
    "F10",
    "F11",
    "F12",
    "ArrowUp",
    "ArrowDown",
    "ArrowLeft",
    "ArrowRight",
    "Space",
    "Enter",
    "Escape",
    "Tab",
    "Backspace",
    "CapsLock",
    "ShiftLeft",
    "ShiftRight",
    "ControlLeft",
    "ControlRight",
    "AltLeft",
    "AltRight",
    "MetaLeft",
    "MetaRight",
    "Minus",
    "Equal",
    "BracketLeft",
    "BracketRight",
    "Backslash",
    "Semicolon",
    "Quote",
    "Backquote",
    "Comma",
    "Period",
    "Slash",
    "IntlBackslash",
    "Home",
    "End",
    "PageUp",
    "PageDown",
    "Insert",
    "Delete",
    "Numpad0",
    "Numpad1",
    "Numpad2",
    "Numpad3",
    "Numpad4",
    "Numpad5",
    "Numpad6",
    "Numpad7",
    "Numpad8",
    "Numpad9",
    "NumpadAdd",
    "NumpadSubtract",
    "NumpadMultiply",
    "NumpadDivide",
    "NumpadDecimal",
    "NumpadEnter",
    "NumpadEqual",
    "NumLock",
    "ScrollLock",
    "PrintScreen",
    "Pause",
    "ContextMenu",
];

/// The number of bits in a key set: 128, so [`KEY_CODES`] has room to grow.
pub const KEY_SET_BITS: usize = 128;

const _: () = assert!(KEY_CODES.len() <= KEY_SET_BITS);

/// The bit of a `KeyboardEvent.code` in [`crate::Input`]'s key sets: its
/// place in [`KEY_CODES`], found by a binary search of the codes in name
/// order (made once). og-ui asks for every code on every tick (the keys held
/// and pressed that UI actions bind) and for the US layout's characters, and
/// a scan of the table per code was most of reading a tick's input on the
/// web (engine s111).
pub fn key_index(code: &str) -> Option<usize> {
    static BY_NAME: std::sync::OnceLock<Vec<(&'static str, u8)>> = std::sync::OnceLock::new();
    let by_name = BY_NAME.get_or_init(|| {
        let mut by_name: Vec<(&'static str, u8)> = (0u8..)
            .zip(KEY_CODES.iter().copied())
            .map(|(i, k)| (k, i))
            .collect();
        by_name.sort_unstable_by_key(|(k, _)| *k);
        by_name
    });
    let at = by_name.binary_search_by(|(k, _)| (*k).cmp(code)).ok()?;
    Some(usize::from(by_name[at].1))
}

/// The key a character is typed with on a US layout, and whether Shift is
/// held: `a` is `KeyA`, `A` is Shift+`KeyA`, `!` is Shift+`Digit1`.
pub fn key_for_char(c: char) -> Option<(String, bool)> {
    if c.is_ascii_alphabetic() {
        return Some((
            format!("Key{}", c.to_ascii_uppercase()),
            c.is_ascii_uppercase(),
        ));
    }
    if c.is_ascii_digit() {
        return Some((format!("Digit{c}"), false));
    }
    let (code, shift) = match c {
        ' ' => ("Space", false),
        '\n' => ("Enter", false),
        '\t' => ("Tab", false),
        '-' => ("Minus", false),
        '_' => ("Minus", true),
        '=' => ("Equal", false),
        '+' => ("Equal", true),
        '[' => ("BracketLeft", false),
        '{' => ("BracketLeft", true),
        ']' => ("BracketRight", false),
        '}' => ("BracketRight", true),
        '\\' => ("Backslash", false),
        '|' => ("Backslash", true),
        ';' => ("Semicolon", false),
        ':' => ("Semicolon", true),
        '\'' => ("Quote", false),
        '"' => ("Quote", true),
        '`' => ("Backquote", false),
        '~' => ("Backquote", true),
        ',' => ("Comma", false),
        '<' => ("Comma", true),
        '.' => ("Period", false),
        '>' => ("Period", true),
        '/' => ("Slash", false),
        '?' => ("Slash", true),
        '!' => ("Digit1", true),
        '@' => ("Digit2", true),
        '#' => ("Digit3", true),
        '$' => ("Digit4", true),
        '%' => ("Digit5", true),
        '^' => ("Digit6", true),
        '&' => ("Digit7", true),
        '*' => ("Digit8", true),
        '(' => ("Digit9", true),
        ')' => ("Digit0", true),
        _ => return None,
    };
    Some((code.to_owned(), shift))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A name's bit is what a scan of the table answers: each code's place
    /// (the codes are distinct), and none for a name the table does not
    /// have, a code's prefix that is no code, a code with more after it,
    /// another case, nothing.
    #[test]
    fn a_key_s_bit_is_its_place_in_the_table() {
        let scan = |name: &str| KEY_CODES.iter().position(|k| *k == name);
        let mut seen = std::collections::BTreeSet::new();
        for (i, code) in KEY_CODES.iter().enumerate() {
            assert!(seen.insert(*code), "{code} twice");
            assert_eq!(key_index(code), Some(i), "{code}");
            for name in [
                &code[..code.len() - 1],
                &format!("{code}x"),
                &code.to_lowercase(),
            ] {
                assert_eq!(key_index(name), scan(name), "{name}");
            }
        }
        for name in ["", "Key", "KEYA", "Digit10", "Numpad", "Zzz", "A", "\u{0}"] {
            assert_eq!(key_index(name), None, "{name:?}");
        }
        assert_eq!(key_index("F1"), scan("F1"));
    }
}
