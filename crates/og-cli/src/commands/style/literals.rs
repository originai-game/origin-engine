//! `og style check`'s token rule: a pack's values live in `:root` (and
//! `:root[data-scheme=…]`) as custom properties; every other declaration
//! names them with `var()`, so a variant restyles the pack by setting
//! tokens alone. Outside a `:root` rule a declaration may not write a
//! colour (`#hex`, a named colour, `rgb()`/`hsl()`/… without a `var()`), a
//! length other than 0 (`px`, `em`, `rem`, viewport and print units), a
//! duration (`ms`, `s`) or a `url()`. Keywords, `0`, unitless numbers (flex,
//! z-index, line-height, opacity, `border-image` slices, sound volumes),
//! percentages (gradient stops, positions, sizes), container-query units
//! (`cqw`, `cqh`…: a share of the container, as a percentage is) and angles stay
//! structural. `@font-face`, `@property` and `@counter-style` take their
//! own descriptors, and `-og-sound` names sound files: both declare assets,
//! so their `url()`s may stand anywhere. `@media` and `@container`
//! conditions are not values.

const LENGTHS: &[&str] = &[
    "px", "em", "rem", "ex", "rex", "ch", "rch", "cap", "rcap", "ic", "ric", "lh", "rlh", "vw",
    "vh", "vi", "vb", "vmin", "vmax", "svw", "svh", "lvw", "lvh", "dvw", "dvh", "cm", "mm", "q",
    "in", "pt", "pc",
];
const DURATIONS: &[&str] = &["ms", "s"];
const COLOUR_FUNCTIONS: &[&str] = &[
    "rgb",
    "rgba",
    "hsl",
    "hsla",
    "hwb",
    "lab",
    "lch",
    "oklab",
    "oklch",
    "color",
    "color-mix",
    "light-dark",
];
/// The property that declares assets anywhere (its files and their gains).
const ASSETS: &str = "-og-sound";
/// At-rules whose blocks hold descriptors, not declarations.
const DESCRIPTORS: &[&str] = &[
    "font-face",
    "property",
    "counter-style",
    "font-feature-values",
];

/// The literals of one CSS source: `<file>:<line>: <property>: …` each.
pub fn findings(file: &str, css: &str) -> Vec<String> {
    let text = blank_comments(css);
    let mut out = Vec::new();
    rules(&text, 0, false, &mut |at, name, value| {
        // A sound set names its files, as `@font-face`'s `src` does: an
        // asset declaration, not dressing.
        if name.eq_ignore_ascii_case(ASSETS) {
            return;
        }
        if let Some(what) = literal(value) {
            let line = css[..at].bytes().filter(|b| *b == b'\n').count() + 1;
            out.push(format!(
                "{file}:{line}: {name}: literal {what} outside :root; set it as a :root token and use var()"
            ));
        }
    });
    out
}

/// `css` with each comment replaced by spaces (newlines kept), so offsets hold.
fn blank_comments(css: &str) -> String {
    let mut bytes = css.as_bytes().to_vec();
    let mut i = 0;
    while let Some(start) = css[i..].find("/*").map(|s| i + s) {
        let end = css[start + 2..]
            .find("*/")
            .map_or(css.len(), |e| start + e + 4);
        for b in &mut bytes[start..end] {
            if *b != b'\n' {
                *b = b' ';
            }
        }
        i = end;
    }
    // Only ASCII bytes inside comments changed, each to a space.
    String::from_utf8(bytes).unwrap_or_default()
}

/// One item of a block: its offset, its prelude or statement, and its `{}`
/// body with the body's offset (`None` for a `;` statement).
type Item<'a> = (usize, &'a str, Option<(usize, &'a str)>);

/// The items of `text`, offsets from its start.
fn items(text: &str) -> Vec<Item<'_>> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let (mut start, mut depth, mut quote) = (0, 0i32, None);
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if let Some(q) = quote {
            if c == b'\\' {
                i += 1;
            } else if c == q {
                quote = None;
            }
        } else {
            match c {
                b'"' | b'\'' => quote = Some(c),
                b'(' | b'[' => depth += 1,
                b')' | b']' => depth -= 1,
                b';' if depth == 0 => {
                    out.push((start, &text[start..i], None));
                    start = i + 1;
                }
                b'{' if depth == 0 => {
                    let mut level = 1;
                    let mut j = i + 1;
                    let mut q = None;
                    while j < bytes.len() && level > 0 {
                        let d = bytes[j];
                        if let Some(qq) = q {
                            if d == b'\\' {
                                j += 1;
                            } else if d == qq {
                                q = None;
                            }
                        } else {
                            match d {
                                b'"' | b'\'' => q = Some(d),
                                b'{' => level += 1,
                                b'}' => level -= 1,
                                _ => {}
                            }
                        }
                        j += 1;
                    }
                    let close = j.saturating_sub(1).max(i + 1);
                    out.push((start, &text[start..i], Some((i + 1, &text[i + 1..close]))));
                    start = j;
                    i = j;
                    continue;
                }
                _ => {}
            }
        }
        i += 1;
    }
    if !text[start..].trim().is_empty() {
        out.push((start, &text[start..], None));
    }
    out
}

/// Walks the rules of `text` at offset `base`, calling `found(offset, name,
/// value)` for each declaration the rule covers. `in_style` is a style
/// rule's body (declarations and nested rules).
fn rules(text: &str, base: usize, in_style: bool, found: &mut dyn FnMut(usize, &str, &str)) {
    for (at, prelude, block) in items(text) {
        let offset = base + at + (prelude.len() - prelude.trim_start().len());
        let prelude = prelude.trim();
        match block {
            None if in_style => {
                if let Some((name, value)) = prelude.split_once(':') {
                    found(offset, name.trim(), value);
                }
            }
            None => {}
            Some((inner, body)) => {
                let inner = base + inner;
                if let Some(at_rule) = prelude.strip_prefix('@') {
                    let name = at_rule
                        .split(|c: char| c.is_whitespace() || c == '(')
                        .next()
                        .unwrap_or("")
                        .to_ascii_lowercase();
                    if DESCRIPTORS.contains(&name.as_str()) {
                        continue;
                    }
                    // Conditional and grouping rules hold rules (a keyframe's
                    // `from` and `50%` read as style rules); nested in a
                    // style rule they hold its declarations.
                    rules(body, inner, in_style, found);
                } else if !root_only(prelude) {
                    rules(body, inner, true, found);
                }
            }
        }
    }
}

/// Whether every selector of a list is `:root`, alone or with attributes.
fn root_only(selectors: &str) -> bool {
    selectors.split(',').all(|s| {
        s.trim()
            .strip_prefix(":root")
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('['))
    })
}

/// What literal `value` writes, if any.
fn literal(value: &str) -> Option<String> {
    use cssparser::{Parser, Token};
    fn walk(p: &mut Parser<'_>) -> Option<String> {
        while let Ok(token) = p.next() {
            let token = token.clone();
            match token {
                Token::Hash(h) | Token::IDHash(h) => return Some(format!("colour `#{h}`")),
                Token::UnquotedUrl(u) => return Some(format!("url `url({u})`")),
                Token::Ident(name) => {
                    let n = name.to_ascii_lowercase();
                    if n != "transparent"
                        && n != "currentcolor"
                        && og_ui::css_color::parse_str(&n).is_ok()
                    {
                        return Some(format!("colour `{name}`"));
                    }
                }
                Token::Dimension { value, unit, .. } if value != 0.0 => {
                    let u = unit.to_ascii_lowercase();
                    if LENGTHS.contains(&u.as_str()) {
                        return Some(format!("length `{value}{unit}`"));
                    }
                    if DURATIONS.contains(&u.as_str()) {
                        return Some(format!("duration `{value}{unit}`"));
                    }
                }
                Token::Function(name) => {
                    let n = name.to_ascii_lowercase();
                    if n == "url" || n == "src" {
                        return Some(format!("url `{name}()`"));
                    }
                    if n == "var" || n == "env" {
                        // The name and its fallback: a fallback is still a value.
                        let found = p.parse_nested_block(|p| {
                            let _ = p.next();
                            let found = walk(p);
                            drain(p);
                            Ok::<_, cssparser::ParseError<()>>(found)
                        });
                        if let Ok(Some(f)) = found {
                            return Some(f);
                        }
                        continue;
                    }
                    let colour = COLOUR_FUNCTIONS.contains(&n.as_str());
                    let inner = p.parse_nested_block(|p| {
                        let start = p.position();
                        let found = walk(p);
                        drain(p);
                        Ok::<_, cssparser::ParseError<()>>((found, p.slice_from(start).to_owned()))
                    });
                    if let Ok((found, text)) = inner {
                        if colour && !text.contains("var(") {
                            return Some(format!("colour `{name}({})`", text.trim()));
                        }
                        if found.is_some() {
                            return found;
                        }
                    }
                }
                Token::ParenthesisBlock | Token::SquareBracketBlock | Token::CurlyBracketBlock => {
                    if let Ok(Some(f)) = p.parse_nested_block(|p| {
                        let found = walk(p);
                        drain(p);
                        Ok::<_, cssparser::ParseError<()>>(found)
                    }) {
                        return Some(f);
                    }
                }
                _ => {}
            }
        }
        None
    }
    walk(&mut Parser::new(value))
}

/// Consumes what a block's walk left after a find, so the block parses whole
/// (a nested block that is not exhausted is an error, and the find is lost).
fn drain(p: &mut cssparser::Parser<'_>) {
    while p.next().is_ok() {}
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ways it could fail: a value in `:root` reported, a structural number
    /// (0, a percentage stop, a slice, a unitless factor, an angle) or a
    /// keyword reported, or a literal hidden in a gradient, a nested rule, a
    /// media block, a keyframe or a `var()` fallback missed.
    #[test]
    fn literals_outside_root_are_found_with_their_line_and_property() {
        let css = "/* #fff in a comment */\n:root { --a: #fff; --b: url(x.png); }\n:root[data-scheme=light] { --a: 2px; }\n\
button::part(base) { color: var(--a); border-image: var(--b) 28 24 fill / 50% var(--w) stretch; opacity: 0.5; flex: 1; padding: 0; transform: rotate(180deg); background: none; border: var(--s) solid transparent; }\n\
panel::part(frame) { background: linear-gradient(90deg, var(--a), #000 50%); }\n\
@media (width < 600px) { chip::part(base) { margin: 4px; } }\n\
@keyframes k { from { opacity: 0; } to { box-shadow: 0 0 2px red; } }\n\
slot::part(frame) { & :hover { transition: color 160ms; } color: var(--x, white); }\n\
@font-face { font-family: X; src: url(a.ttf); }\n\
button::part(cell) { -og-sound: press url(tap.ogg) volume(0.3 0.4); }\n";
        let found = findings("p.css", css);
        let lines: Vec<&str> = found
            .iter()
            .map(|f| f.split(": literal").next().unwrap())
            .collect();
        assert_eq!(
            lines,
            [
                "p.css:5: background",
                "p.css:6: margin",
                "p.css:7: box-shadow",
                "p.css:8: transition",
                "p.css:8: color",
            ],
            "{found:#?}"
        );
        assert!(found[0].contains("`#000`"), "{}", found[0]);
    }
}
