//! MessageFormat 2 markup in translated strings (docs/ui.md §2): a string
//! table's `Press {#b}{$key}{/b} to {#i}jump{/i}` styles its runs as a
//! document's inline `<b>` and `<i>` do. Markup is `{#name}` opening,
//! `{/name}` closing and `{#name/}` standalone (`{#br/}`); `\{`, `\}` and
//! `\\` escape. Names other than the formatting ones keep their text
//! unstyled.

/// One run of a translated string: its text and the formatting around it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Run {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
}

/// Whether `text` holds markup (so it needs runs, not plain text).
pub(crate) fn has_markup(text: &str) -> bool {
    text.contains("{#") || text.contains("{/")
}

/// `text`'s runs in order; an unclosed open runs to the end.
pub(crate) fn runs(text: &str) -> Vec<Run> {
    let mut out: Vec<Run> = Vec::new();
    let mut open: Vec<String> = Vec::new();
    let mut cur = String::new();
    let style = |open: &[String], text: String| {
        let on = |names: &[&str]| open.iter().any(|o| names.contains(&o.as_str()));
        Run {
            text,
            bold: on(&["b", "strong"]),
            italic: on(&["i", "em"]),
            underline: on(&["u"]),
            strike: on(&["s", "del"]),
        }
    };
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(&next) = chars.peek() {
                    cur.push(next);
                    chars.next();
                }
            }
            '{' if matches!(chars.peek(), Some('#' | '/')) => {
                let closing = chars.next() == Some('/');
                // Its name, then options up to `}` (a `/` before it: standalone).
                let mut name = String::new();
                let mut in_name = true;
                let mut standalone = false;
                for c in chars.by_ref() {
                    match c {
                        '}' => break,
                        '/' => standalone = true,
                        c if c.is_whitespace() => in_name = false,
                        c if in_name => name.push(c),
                        _ => standalone = false,
                    }
                }
                if !cur.is_empty() {
                    out.push(style(&open, std::mem::take(&mut cur)));
                }
                if standalone {
                    if name == "br" {
                        out.push(style(&open, "\n".into()));
                    }
                } else if closing {
                    if let Some(k) = open.iter().rposition(|o| *o == name) {
                        open.remove(k);
                    }
                } else {
                    open.push(name);
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(style(&open, cur));
    }
    out
}

/// `text` without its markup.
pub(crate) fn plain(text: &str) -> String {
    runs(text).into_iter().map(|r| r.text).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markup_styles_its_runs_and_escapes_stay_text() {
        let r = runs("Press {#b}E{/b} to {#i}{#u}jump{/u}{/i}{#br/}\\{ok\\}");
        let texts: Vec<_> = r.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(texts, ["Press ", "E", " to ", "jump", "\n", "{ok}"]);
        assert!(r[1].bold && !r[0].bold && !r[2].bold);
        assert!(r[3].italic && r[3].underline && !r[3].bold);
        assert_eq!(plain("a {#b}b{/b} c"), "a b c");
        assert!(!has_markup("100% {name}"));
    }
}
