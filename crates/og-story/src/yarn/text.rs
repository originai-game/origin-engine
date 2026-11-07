//! Text/interpolation boundaries shared by Yarn lines and commands.
pub(super) enum Part {
    Text(String),
    Expression(String),
}

/// Keep Yarn literals literal when the string table parses ICU MessageFormat.
pub(super) fn icu_literal(text: &str) -> String {
    let escaped = text.replace('\'', "''");
    if let Some(first) = escaped.find(['{', '}']) {
        let last = escaped.rfind(['{', '}']).unwrap() + 1;
        format!(
            "{}'{}'{}",
            &escaped[..first],
            &escaped[first..last],
            &escaped[last..]
        )
    } else {
        escaped
    }
}

pub(super) fn parts(input: &str) -> Result<Vec<Part>, String> {
    let mut parts = Vec::new();
    let mut text = String::new();
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                let next = chars.next().ok_or("trailing text escape")?;
                // The markup parser, after localization, owns these escapes.
                if matches!(next, '[' | ']' | ':') {
                    text.push('\\');
                }
                text.push(next);
            }
            '{' => {
                parts.push(Part::Text(std::mem::take(&mut text)));
                let mut expression = String::new();
                let mut quoted = false;
                let mut escaped = false;
                let mut closed = false;
                for c in chars.by_ref() {
                    if c == '}' && !quoted {
                        closed = true;
                        break;
                    }
                    expression.push(c);
                    if escaped {
                        escaped = false;
                    } else if c == '\\' {
                        escaped = true;
                    } else if c == '"' {
                        quoted = !quoted;
                    }
                }
                if !closed {
                    return Err("close the inline expression with }".into());
                }
                parts.push(Part::Expression(expression));
            }
            '}' => return Err("unmatched } in Yarn text".into()),
            c => text.push(c),
        }
    }
    parts.push(Part::Text(text));
    Ok(parts)
}
