//! The CSS a sheet is written in, split once at load: its rule blocks and
//! their declarations. Nothing here runs per element or per frame.

/// Each top-level rule's prelude and body, `@media` and other at-rule
/// blocks flattened into their inner rules (their condition is the
/// compiler's, `crate::style`). Comments are skipped.
pub fn blocks(css: &str) -> Vec<(&str, &str)> {
    let mut out = Vec::new();
    collect(css, &mut out);
    out
}

fn collect<'a>(css: &'a str, out: &mut Vec<(&'a str, &'a str)>) {
    let bytes = css.as_bytes();
    let mut i = 0;
    let mut start = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                let end = css[i + 2..]
                    .find("*/")
                    .map_or(bytes.len(), |e| i + 2 + e + 2);
                if css[start..i].trim().is_empty() {
                    start = end;
                }
                i = end;
                continue;
            }
            b'{' => {
                let Some(close) = crate::tokens::matching_brace(css, i + 1) else {
                    return;
                };
                let prelude = css[start..i].trim();
                let body = &css[i + 1..close];
                if prelude.starts_with('@') && body.contains('{') {
                    collect(body, out);
                } else {
                    out.push((prelude, body));
                }
                i = close + 1;
                start = i;
                continue;
            }
            b';' => start = i + 1,
            _ => {}
        }
        i += 1;
    }
}

/// A block's declarations as name and value, in order; `!important` and
/// surrounding space trimmed.
pub fn declarations(body: &str) -> impl Iterator<Item = (&str, &str)> {
    split_top(body, b';').filter_map(|d| {
        let colon = d.find(':')?;
        let name = d[..colon].trim();
        let value = d[colon + 1..].trim();
        let value = value
            .strip_suffix("!important")
            .map_or(value, str::trim_end);
        (!name.is_empty()).then_some((name, value))
    })
}

/// `text` split on `sep` outside parentheses and quotes.
pub fn split_top(text: &str, sep: u8) -> impl Iterator<Item = &str> {
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    let mut quote = 0u8;
    let mut cuts = vec![0usize];
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'"' | b'\'' if quote == 0 => quote = b,
            q if q == quote && quote != 0 => quote = 0,
            b'(' if quote == 0 => depth += 1,
            b')' if quote == 0 => depth -= 1,
            s if s == sep && depth == 0 && quote == 0 => cuts.push(i + 1),
            _ => {}
        }
    }
    cuts.push(bytes.len() + 1);
    (0..cuts.len() - 1).map(move |k| {
        let a = cuts[k];
        let b = (cuts[k + 1] - 1).min(text.len());
        &text[a..b]
    })
}
