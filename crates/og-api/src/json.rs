//! A tiny ordered JSON value and pretty printer, so generated JSON is
//! byte-stable and `og-api` needs no dependencies.

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    /// Source text of a number, written as is.
    Num(String),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn str(s: impl Into<String>) -> Json {
        Json::Str(s.into())
    }

    pub fn num(n: impl std::fmt::Display) -> Json {
        Json::Num(n.to_string())
    }

    pub fn obj() -> Json {
        Json::Obj(Vec::new())
    }

    /// Append a key (builder style). Panics on a non-object, which is a generator bug.
    pub fn with(mut self, key: &str, value: Json) -> Json {
        self.set(key, value);
        self
    }

    pub fn set(&mut self, key: &str, value: Json) {
        match self {
            Json::Obj(pairs) => pairs.push((key.to_string(), value)),
            _ => panic!("Json::set on a non-object"),
        }
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        self.write(&mut out, 0);
        out.push('\n');
        out
    }

    fn write(&self, out: &mut String, depth: usize) {
        let pad = |out: &mut String, d: usize| {
            for _ in 0..d {
                out.push_str("  ");
            }
        };
        match self {
            Json::Null => out.push_str("null"),
            Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Json::Num(n) => out.push_str(n),
            Json::Str(s) => quote(out, s),
            Json::Arr(items) if items.is_empty() => out.push_str("[]"),
            Json::Obj(pairs) if pairs.is_empty() => out.push_str("{}"),
            Json::Arr(items) => {
                // Short arrays of scalars stay on one line.
                if items.len() <= 16
                    && items
                        .iter()
                        .all(|i| matches!(i, Json::Num(_) | Json::Bool(_) | Json::Null))
                {
                    out.push('[');
                    for (i, item) in items.iter().enumerate() {
                        if i > 0 {
                            out.push_str(", ");
                        }
                        item.write(out, depth);
                    }
                    out.push(']');
                    return;
                }
                out.push_str("[\n");
                for (i, item) in items.iter().enumerate() {
                    pad(out, depth + 1);
                    item.write(out, depth + 1);
                    if i + 1 < items.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                pad(out, depth);
                out.push(']');
            }
            Json::Obj(pairs) => {
                out.push_str("{\n");
                for (i, (k, v)) in pairs.iter().enumerate() {
                    pad(out, depth + 1);
                    quote(out, k);
                    out.push_str(": ");
                    v.write(out, depth + 1);
                    if i + 1 < pairs.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                pad(out, depth);
                out.push('}');
            }
        }
    }
}

fn quote(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}
