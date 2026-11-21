//! A style's tokens: its custom properties, resolved once when the style
//! loads. A sheet's declarations refer to them by id, so swapping a pack is
//! swapping this table and re-resolving the sheet's records, never a cascade
//! per element.

use rustc_hash::FxHashMap;

/// The token table: each name's value with every `var()` substituted.
/// A scheme's blocks (`:root.light`, `:root[data-scheme=dark]`) are kept
/// apart as that scheme's overlay; [`Tokens::scheme`] builds its table, so
/// a scheme is a second table, never a cascade.
#[derive(Clone, Debug, Default)]
pub struct Tokens {
    ids: FxHashMap<Box<str>, u32>,
    raw: Vec<Box<str>>,
    resolved: Vec<Option<Box<str>>>,
    layers: Vec<Layer>,
}

/// One sheet's tokens as read (the defaults, then each pack over them):
/// its base declarations and each scheme's, raw and in order.
#[derive(Clone, Debug, Default)]
struct Layer {
    base: Decls,
    schemes: Vec<(Box<str>, Decls)>,
}

type Decls = Vec<(Box<str>, Box<str>)>;

/// The overlay a block's prelude names: `light` for `:root.light` or
/// `:root[data-scheme=light]`, [`CONTRAST`] for `:root[data-contrast=more]`
/// and `dark more` for `:root[data-scheme=dark][data-contrast=more]`; none
/// for a plain block.
fn scheme_of(prelude: &str) -> Option<String> {
    let rest = prelude.trim().strip_prefix(":root")?;
    if let Some(name) = rest.strip_prefix('.') {
        return Some(name.trim().to_owned());
    }
    let (mut scheme, mut more) = (None, false);
    for attr in rest.split_inclusive(']') {
        let inner = attr.trim().strip_prefix('[')?.strip_suffix(']')?;
        let (key, value) = inner.split_once('=')?;
        let value = value.trim_matches(|c| c == '"' || c == '\'');
        match key.trim() {
            "data-scheme" => scheme = Some(value),
            "data-contrast" if value == CONTRAST => more = true,
            _ => return None,
        }
    }
    match (scheme, more) {
        (Some(s), false) => Some(s.to_owned()),
        (None, true) => Some(CONTRAST.to_owned()),
        (Some(s), true) => Some(format!("{s} {CONTRAST}")),
        (None, false) => None,
    }
}

/// The overlay of `prefers-contrast: more`.
pub const CONTRAST: &str = "more";

impl Tokens {
    /// The custom properties declared in `css` (every block's, the last
    /// declaration of a name winning), resolved; a scheme's blocks become
    /// its overlay.
    pub fn parse(css: &str) -> Tokens {
        let mut t = Tokens::default();
        t.over(css);
        t
    }

    /// Lays `over`'s tokens over these (a pack's over the defaults) as a
    /// layer of its own, its scheme blocks with it.
    pub fn over(&mut self, over: &str) {
        let mut layer = Layer::default();
        for (prelude, body) in crate::sheet::blocks(over) {
            let decls = crate::sheet::declarations(body)
                .filter(|(n, _)| n.starts_with("--"))
                .map(|(n, v)| (n.into(), v.into()));
            match scheme_of(prelude) {
                Some(scheme) => match layer.schemes.iter_mut().find(|(s, _)| **s == *scheme) {
                    Some((_, d)) => d.extend(decls),
                    None => layer.schemes.push((scheme.into(), decls.collect())),
                },
                None => layer.base.extend(decls),
            }
        }
        for (name, value) in &layer.base {
            self.set_raw(name, value);
        }
        self.layers.push(layer);
        self.resolve();
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

    /// The schemes some layer has an overlay for, in the order first read.
    pub fn schemes(&self) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        for (s, _) in self.layers.iter().flat_map(|l| &l.schemes) {
            if !out.contains(&&**s) {
                out.push(s);
            }
        }
        out
    }

    /// The table in `scheme`, resolved: layer by layer, its base and then
    /// its overlay for the scheme. A later layer outranks an earlier one
    /// whatever their blocks (a pack's `:root` over the defaults'
    /// `:root[data-scheme=dark]`); within a layer the scheme's block
    /// outranks `:root`. A scheme no layer names is the base, and so is any
    /// scheme of a table [`Tokens::from_bytes`] read.
    pub fn scheme(&self, scheme: &str) -> Tokens {
        self.table(scheme, false)
    }

    /// The table in `scheme`, in high contrast when `more`: within a layer
    /// its base, the scheme's overlay, then [`CONTRAST`]'s and the
    /// scheme's own contrast overlay (`dark more`).
    pub fn table(&self, scheme: &str, more: bool) -> Tokens {
        if self.layers.is_empty() {
            return self.clone();
        }
        let both = format!("{scheme} {CONTRAST}");
        let names: &[&str] = if more {
            &[scheme, CONTRAST, &both]
        } else {
            &[scheme]
        };
        let mut t = Tokens::default();
        for layer in &self.layers {
            let overlay = names
                .iter()
                .flat_map(|n| layer.schemes.iter().filter(move |(s, _)| &**s == *n));
            for (name, value) in layer.base.iter().chain(overlay.flat_map(|(_, d)| d)) {
                t.set_raw(name, value);
            }
        }
        t.layers = self.layers.clone();
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
    fn a_scheme_block_is_its_own_table_layer_by_layer() {
        let mut t = Tokens::parse(
            ":root { --seed: #fff; --rim: #111; --ground: #eee; }\n\
             :root[data-scheme=dark] { --rim: #999; --seed: #000; }",
        );
        t.over(PACK);
        assert_eq!(t.get("--seed"), Some("#102030"), "the base stays dark");
        assert_eq!(t.get("--ground"), Some("#eee"));
        let light = t.scheme("light");
        assert_eq!(light.get("--seed"), Some("#f0e0d0"));
        assert_eq!(light.get("--ink"), Some("#f0e0d0"), "its users re-resolve");
        let dark = t.scheme("dark");
        assert_eq!(dark.get("--rim"), Some("#999"), "the defaults' dark block");
        assert_eq!(
            dark.get("--seed"),
            Some("#102030"),
            "a pack's :root outranks the defaults' dark block"
        );
        assert_eq!(t.schemes(), ["dark", "light"]);
    }
}
