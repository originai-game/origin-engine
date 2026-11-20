//! A style's tokens: its custom properties, resolved once when the style
//! loads. A sheet's declarations refer to them by id, so swapping a pack is
//! swapping this table and re-resolving the sheet's records, never a cascade
//! per element.

use rustc_hash::FxHashMap;

/// The token table: each name's value with every `var()` substituted.
#[derive(Clone, Debug, Default)]
pub struct Tokens {
    ids: FxHashMap<Box<str>, u32>,
    raw: Vec<Box<str>>,
    resolved: Vec<Option<Box<str>>>,
}

impl Tokens {
    /// The custom properties declared in `css` (every block's, the last
    /// declaration of a name winning), resolved.
    pub fn parse(css: &str) -> Tokens {
        let mut t = Tokens::default();
        for (_, body) in crate::sheet::blocks(css) {
            for (name, value) in crate::sheet::declarations(body) {
                if name.starts_with("--") {
                    t.set_raw(name, value);
                }
            }
        }
        t.resolve();
        t
    }

    fn set_raw(&mut self, name: &str, value: &str) {
        match self.ids.get(name) {
            Some(&i) => self.raw[i as usize] = value.into(),
            None => {
                self.ids.insert(name.into(), self.raw.len() as u32);
                self.raw.push(value.into());
            }
        }
    }

    /// Lays `over`'s tokens over these (a pack's over the defaults).
    pub fn over(&mut self, over: &str) {
        for (_, body) in crate::sheet::blocks(over) {
            for (name, value) in crate::sheet::declarations(body) {
                if name.starts_with("--") {
                    self.set_raw(name, value);
                }
            }
        }
        self.resolve();
    }

    fn resolve(&mut self) {
        self.resolved = vec![None; self.raw.len()];
        for i in 0..self.raw.len() {
            self.resolve_one(i, 0);
        }
    }

    fn resolve_one(&mut self, i: usize, depth: u32) -> Option<Box<str>> {
        if let Some(v) = &self.resolved[i] {
            return Some(v.clone());
        }
        if depth > 64 {
            return None;
        }
        let raw = self.raw[i].clone();
        let v = self.substitute(&raw, depth + 1)?;
        self.resolved[i] = Some(v.clone().into());
        Some(v.into())
    }

    /// `text` with each `var(--name, fallback)` replaced by the token's value.
    pub fn substitute_str(&self, text: &str) -> Option<String> {
        substitute(text, &mut |name| self.get(name).map(String::from))
    }

    fn substitute(&mut self, text: &str, depth: u32) -> Option<String> {
        substitute(text, &mut |name| {
            let id = self.ids.get(name).copied()?;
            self.resolve_one(id as usize, depth).map(String::from)
        })
    }
}

fn substitute(text: &str, lookup: &mut dyn FnMut(&str) -> Option<String>) -> Option<String> {
    let Some(at) = text.find("var(") else {
        return Some(text.to_owned());
    };
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut at = Some(at);
    while let Some(a) = at {
        out.push_str(&rest[..a]);
        let inner_start = a + 4;
        let close = matching(rest, inner_start)?;
        let inner = &rest[inner_start..close];
        let (name, fallback) = match top_comma(inner) {
            Some(c) => (inner[..c].trim(), Some(inner[c + 1..].trim())),
            None => (inner.trim(), None),
        };
        let value = match (lookup(name), fallback) {
            (Some(v), _) => v,
            (None, Some(f)) => substitute(f, lookup)?,
            (None, None) => return None,
        };
        out.push_str(&value);
        rest = &rest[close + 1..];
        at = rest.find("var(");
    }
    out.push_str(rest);
    Some(out)
}

impl Tokens {
    pub fn get(&self, name: &str) -> Option<&str> {
        let i = *self.ids.get(name)?;
        self.resolved[i as usize].as_deref()
    }

    pub fn len(&self) -> usize {
        self.raw.len()
    }

    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }
}

/// The index of the `)` closing the parenthesis opened before `from`.
pub(crate) fn matching(text: &str, from: usize) -> Option<usize> {
    let mut depth = 1;
    for (i, b) in text.as_bytes()[from..].iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(from + i);
                }
            }
            _ => {}
        }
    }
    None
}

fn top_comma(text: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, b) in text.bytes().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => depth -= 1,
            b',' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// The index of the `}` closing the brace opened before `from`.
pub(crate) fn matching_brace(text: &str, from: usize) -> Option<usize> {
    let mut depth = 1;
    for (i, b) in text.as_bytes()[from..].iter().enumerate() {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(from + i);
                }
            }
            _ => {}
        }
    }
    None
}
