//! A style's tokens: its custom properties, resolved once when the style
//! loads. A sheet's declarations refer to them by id, so swapping a pack is
//! swapping this table and re-resolving the sheet's records, never a cascade
//! per element.

use rustc_hash::FxHashMap;

/// The token table: each name's value with every `var()` substituted.
/// A scheme's blocks (`:root.light`, `:root[data-scheme=dark]`) are kept
/// apart as that scheme's overlay; [`Tokens::scheme`] lays them over the
/// base, so a scheme is a second table, never a cascade.
#[derive(Clone, Debug, Default)]
pub struct Tokens {
    ids: FxHashMap<Box<str>, u32>,
    raw: Vec<Box<str>>,
    resolved: Vec<Option<Box<str>>>,
    schemes: Vec<(Box<str>, Overlay)>,
}

/// A scheme's declarations, in order, raw.
type Overlay = Vec<(Box<str>, Box<str>)>;

/// The scheme a block's prelude names: `light` for `:root.light` or
/// `:root[data-scheme=light]`, none for a plain block.
fn scheme_of(prelude: &str) -> Option<&str> {
    let rest = prelude.trim().strip_prefix(":root")?;
    if let Some(name) = rest.strip_prefix('.') {
        return Some(name.trim());
    }
    let inner = rest.strip_prefix("[data-scheme=")?.strip_suffix(']')?;
    Some(inner.trim_matches(|c| c == '"' || c == '\''))
}

impl Tokens {
    /// The custom properties declared in `css` (every block's, the last
    /// declaration of a name winning), resolved; a scheme's blocks become
    /// its overlay.
    pub fn parse(css: &str) -> Tokens {
        let mut t = Tokens::default();
        t.read(css);
        t.resolve();
        t
    }

    fn read(&mut self, css: &str) {
        for (prelude, body) in crate::sheet::blocks(css) {
            let decls = crate::sheet::declarations(body).filter(|(n, _)| n.starts_with("--"));
            match scheme_of(prelude) {
                Some(scheme) => {
                    let at = match self.schemes.iter().position(|(s, _)| &**s == scheme) {
                        Some(at) => at,
                        None => {
                            self.schemes.push((scheme.into(), Vec::new()));
                            self.schemes.len() - 1
                        }
                    };
                    self.schemes[at]
                        .1
                        .extend(decls.map(|(n, v)| (n.into(), v.into())));
                }
                None => {
                    for (name, value) in decls {
                        self.set_raw(name, value);
                    }
                }
            }
        }
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

    /// Lays `over`'s tokens over these (a pack's over the defaults), its
    /// scheme blocks over each scheme's overlay.
    pub fn over(&mut self, over: &str) {
        self.read(over);
        self.resolve();
    }

    /// The schemes the table has an overlay for, in the order first read.
    pub fn schemes(&self) -> impl Iterator<Item = &str> {
        self.schemes.iter().map(|(s, _)| &**s)
    }

    /// The table in `scheme`: every layer's base, then every layer's
    /// overlay for it (as `:root.light` outranks `:root` whatever the
    /// order), resolved. A scheme with no overlay is the base.
    pub fn scheme(&self, scheme: &str) -> Tokens {
        let mut t = Tokens {
            schemes: Vec::new(),
            ..self.clone()
        };
        if let Some((_, decls)) = self.schemes.iter().find(|(s, _)| &**s == scheme) {
            for (name, value) in decls {
                t.set_raw(name, value);
            }
        }
        t.resolve();
        t
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

    /// `text` substituted with `local`'s values (an element's own custom
    /// properties, innermost last) over these.
    pub fn substitute_with(&self, text: &str, local: &[(Box<str>, Box<str>)]) -> Option<String> {
        substitute(text, &mut |name| {
            local
                .iter()
                .rev()
                .find(|(n, _)| &**n == name)
                .map(|(_, v)| v.to_string())
                .or_else(|| self.get(name).map(String::from))
        })
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

    /// The resolved table as the blob keeps it: `name value` lines (a
    /// value holds no newline once read). Raw values are the cook's only.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut names: Vec<(&str, u32)> = self.ids.iter().map(|(n, &i)| (&**n, i)).collect();
        names.sort_unstable_by_key(|&(_, i)| i);
        let mut out = String::new();
        for (name, i) in names {
            if let Some(v) = &self.resolved[i as usize] {
                out.push_str(name);
                out.push(' ');
                out.push_str(&v.replace('\n', " "));
                out.push('\n');
            }
        }
        out.into_bytes()
    }

    /// A table [`Tokens::to_bytes`] wrote.
    pub fn from_bytes(bytes: &[u8]) -> Option<Tokens> {
        let mut t = Tokens::default();
        for line in std::str::from_utf8(bytes).ok()?.lines() {
            let (name, value) = line.split_once(' ')?;
            t.ids.insert(name.into(), t.raw.len() as u32);
            t.raw.push(value.into());
            t.resolved.push(Some(value.into()));
        }
        Some(t)
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

#[cfg(test)]
mod tests {
    use super::*;

    const PACK: &str = "@font-face { font-family: \"X\"; src: url(x.ttf); }\n\
        :root {\n  /* the pack's palette: dark */\n  --seed: #102030;\n  --ink: var(--seed);\n}\n\
        :root.light {\n  /* by day */\n  --seed: #f0e0d0;\n}";

    #[test]
    fn a_comment_with_an_apostrophe_loses_no_token() {
        let t = Tokens::parse(PACK);
        assert_eq!(t.get("--seed"), Some("#102030"));
        assert_eq!(t.get("--ink"), Some("#102030"));
    }

    #[test]
    fn a_light_block_is_its_own_table_over_every_layer() {
        let mut t = Tokens::parse(
            ":root { --seed: #fff; --rim: #111; --ground: #eee; }\n\
             :root[data-scheme=dark] { --rim: #999; }",
        );
        t.over(PACK);
        assert_eq!(t.get("--seed"), Some("#102030"), "the base stays dark");
        assert_eq!(t.get("--ground"), Some("#eee"));
        let light = t.scheme("light");
        assert_eq!(light.get("--seed"), Some("#f0e0d0"));
        assert_eq!(light.get("--ink"), Some("#f0e0d0"), "its users re-resolve");
        assert_eq!(t.scheme("dark").get("--rim"), Some("#999"));
        assert_eq!(t.scheme("dark").get("--seed"), Some("#102030"));
        assert_eq!(t.schemes().collect::<Vec<_>>(), ["dark", "light"]);
    }
}
