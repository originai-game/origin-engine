//! CSS Values 4 math functions: `calc()`, `min()`, `max()` and `clamp()`,
//! reduced once their `var()`s are substituted (at cook for a constant
//! value, when the element resolves for one that reads a variable, such as
//! a bound `style:--fill`).
//!
//! As in browsers, a length reduces to a linear sum of its `%`, `px` and
//! `em` terms: `calc(100% - 12px)` stays `calc(100% - 12px)`, and layout
//! resolves the percentage against the basis it already has. Viewport,
//! root and font-relative units become px first ([`absolute`]), when the
//! element resolves. The other functions of CSS Values 4 §10 (stepped
//! values, trigonometry, exponents and signs) evaluate here; `min()`,
//! `max()`, `clamp()`, `round()`, `mod()`, `rem()`, `abs()` and `hypot()`
//! whose length arguments mix `%` with px stay functions for layout.

/// A sum of terms, each a coefficient of a canonical unit: lengths in
/// `%`, `px` and `em`, times in `ms`, angles in `deg`, `""` for a plain
/// number. Terms are kept in [`ORDER`] and are never empty.
#[derive(Clone, Debug, PartialEq)]
enum Dim {
    Sum(Vec<(&'static str, f64)>),
    /// A `min()`, `max()` or `clamp()` of length sums in more than one unit,
    /// left for layout.
    Fn(&'static str, Vec<Vec<(&'static str, f64)>>),
}

const UNITS: &[(&str, &str, f64)] = &[
    ("px", "px", 1.0),
    ("%", "%", 1.0),
    ("em", "em", 1.0),
    // CSS Values 4 §6.2: the physical units at 96px to the inch.
    ("cm", "px", 96.0 / 2.54),
    ("mm", "px", 96.0 / 25.4),
    ("Q", "px", 96.0 / 101.6),
    ("in", "px", 96.0),
    ("pt", "px", 96.0 / 72.0),
    ("pc", "px", 16.0),
    ("ms", "ms", 1.0),
    ("s", "ms", 1000.0),
    ("deg", "deg", 1.0),
    ("grad", "deg", 0.9),
    ("rad", "deg", 180.0 / std::f64::consts::PI),
    ("turn", "deg", 360.0),
    ("x", "dppx", 1.0),
    ("dppx", "dppx", 1.0),
    ("dpi", "dppx", 1.0 / 96.0),
    ("dpcm", "dppx", 2.54 / 96.0),
    ("", "", 1.0),
];

/// What one physical unit is worth in px (`None`: not a physical unit).
pub fn physical(unit: &str) -> Option<f64> {
    UNITS
        .iter()
        .find(|(w, c, _)| *w == unit && *c == "px" && *w != "px")
        .map(|&(_, _, k)| k)
}

/// The order a sum's terms are written in.
const ORDER: [&str; 7] = ["%", "px", "em", "ms", "deg", "dppx", ""];

/// The units resolved before reduction: the viewport's (with the small,
/// large and dynamic forms of CSS Values 4 §6.1.2, one viewport here), the
/// root's and the element's font's but `em` (which a margin keeps to the
/// element's resolve, [`keeps_em`]).
const ABSOLUTE: [&str; 41] = [
    "vw", "vh", "vi", "vb", "vmin", "vmax", "svw", "svh", "svi", "svb", "svmin", "svmax", "lvw",
    "lvh", "lvi", "lvb", "lvmin", "lvmax", "dvw", "dvh", "dvi", "dvb", "dvmin", "dvmax", "rem",
    "ch", "ex", "cap", "ic", "lh", "rlh", "rch", "rex", "rcap", "ric", "cqw", "cqh", "cqi", "cqb",
    "cqmin", "cqmax",
];

const FUNCTIONS: [&str; 23] = [
    "calc", "min", "max", "clamp", "round", "mod", "rem", "abs", "sign", "sin", "cos", "tan",
    "asin", "acos", "atan2", "atan", "pow", "sqrt", "hypot", "log", "exp", "progress", "random",
];

/// Math functions over lengths whose result layout can still evaluate
/// when their arguments mix `%` with px (`Size::Calc`).
const LAYOUT_FUNCTIONS: [&str; 11] = [
    "min",
    "max",
    "clamp",
    "round",
    "round-up",
    "round-down",
    "round-to-zero",
    "mod",
    "rem",
    "abs",
    "hypot",
];

fn is_length(unit: &str) -> bool {
    matches!(unit, "%" | "px" | "em" | "size")
}

/// What the units [`absolute`] resolves are worth for one element.
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Basis {
    /// The viewport's dynamic size (the surface's now), and its small and
    /// large ones (CSS Values 4 §6.1.2).
    pub width: f32,
    pub height: f32,
    pub small: [f32; 2],
    pub large: [f32; 2],
    /// The root font size (the body type step).
    pub rem: f32,
    /// The element's font size and line height (its type step's), px; 0:
    /// the root's.
    pub em: f32,
    pub lh: f32,
    /// The root's line height, px.
    pub rlh: f32,
    /// The body face's `0` advance, cap height and CJK water ideograph
    /// advance, and its x-height, each in em (CSS Values 4 §6.1.1).
    pub ch: f32,
    pub cap: f32,
    pub ic: f32,
    pub ex: f32,
    /// The safe area's insets (top, right, bottom, left), for `env()`.
    pub safe: [f32; 4],
    /// The HUD band's insets (top, right, bottom, left), for
    /// `env(-og-hud-inset-*)` ([`crate::Viewport::hud_insets`]).
    pub hud: [f32; 4],
    /// The nearest query container eligible on each axis (width, height,
    /// its own inline and block axes: `container::unit_basis`) and its
    /// settled size along it (0 until measured); none, and the container
    /// units fall back to the viewport's.
    pub containers: [Option<(u64, f32)>; 4],
    /// The root element's writing mode is vertical: `vi` and `vb` are the
    /// viewport's height and width (CSS Values 4 §6.1.2).
    pub vertical: bool,
    /// The element's identity, stable while it stays where it is: what
    /// sets a `random()` apart from the same one on another element
    /// (CSS Values 5 §9.4).
    pub element: u64,
    /// The document's identity (its mount's): the rest of a random
    /// function's cache name, so a shared `--key` draws apart in another
    /// document.
    pub document: u64,
    /// The element's index among its parent's element children, from 1,
    /// and their count, for `sibling-index()` and `sibling-count()` (CSS
    /// Values 5 §8.1); 0: not known, read as 1.
    pub siblings: [u32; 2],
}

impl Basis {
    /// The element's em and line height, the root's when it has none.
    fn em(&self) -> (f32, f32) {
        if self.em > 0.0 {
            (self.em, self.lh)
        } else {
            (self.rem, self.rlh)
        }
    }

    /// The font ratios a face lacking them takes (CSS Values 4 §6.1.1:
    /// 0.5em for `ch` and `ex`, the ascent for `cap`, 1em for `ic`).
    fn ratio(v: f32, fallback: f32) -> f32 {
        if v > 0.0 { v } else { fallback }
    }
}

/// Which viewport axes and query containers a value read through its units.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Read {
    pub width: bool,
    pub height: bool,
    pub containers: [Option<u64>; 4],
}

/// Whether `text` has a number in a unit [`absolute`] resolves, an
/// `env()`, a tree-counting function, a `random-item()` or a `random()`
/// whose key reads its element or document.
pub fn has_absolute(text: &str) -> bool {
    text.contains("env(")
        || text.contains("random-item(")
        || text.contains("sibling-index(")
        || text.contains("sibling-count(")
        || text
            .match_indices("random(")
            .any(|(at, f)| !text[at + f.len()..].trim_start().starts_with("fixed"))
        || numbers(text).any(|(_, _, unit)| ABSOLUTE.contains(&unit))
}

/// A property that keeps its `em` terms to the element's resolve (a
/// margin's em reads the element's own type step there).
pub fn keeps_em(name: &str) -> bool {
    name.starts_with("margin")
}

/// Whether property `name`'s `text` resolves only when its element does:
/// it reads the viewport, the safe area, the element's font or an
/// attribute.
pub fn per_element(name: &str, text: &str) -> bool {
    has_absolute(text)
        || text.contains("attr(")
        || (!keeps_em(name) && numbers(text).any(|(_, _, unit)| unit == "em"))
}

/// `text` with every viewport, root and font-relative number written in px
/// (`em` too unless property `name` [`keeps_em`]), every `env()` replaced,
/// and the viewport axes it read.
pub fn absolute(text: &str, basis: Basis, name: &str) -> (String, Read) {
    let mut read = Read::default();
    let text = env(text, basis, &mut read);
    let text = tree_counts(&text, basis);
    let text = randoms(&text, basis, name);
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    let (em, lh) = basis.em();
    let root = basis.rem;
    let ch = Basis::ratio(basis.ch, 0.5);
    let ex = Basis::ratio(basis.ex, 0.5);
    let cap = Basis::ratio(basis.cap, 0.7);
    let ic = Basis::ratio(basis.ic, 1.0);
    for (start, end, unit) in numbers(&text) {
        // `s`, `l` and `d` read the small, large and dynamic viewport; a
        // bare `v` unit the dynamic one, as browsers' do (§6.1.2).
        let size = |v: [f32; 2]| {
            if v[0] > 0.0 && v[1] > 0.0 {
                v
            } else {
                [basis.width, basis.height]
            }
        };
        let (viewport, [w, h]) = match unit.split_at_checked(1) {
            Some(("s", u)) if u.starts_with('v') => (u, size(basis.small)),
            Some(("l", u)) if u.starts_with('v') => (u, size(basis.large)),
            Some(("d", u)) if u.starts_with('v') => (u, [basis.width, basis.height]),
            _ => (unit, [basis.width, basis.height]),
        };
        let (w, h) = (w as f64 / 100.0, h as f64 / 100.0);
        let viewport = match (viewport, basis.vertical) {
            ("vi", true) => "vh",
            ("vb", true) => "vw",
            ("vi", false) => "vw",
            ("vb", false) => "vh",
            (v, _) => v,
        };
        let per = match viewport {
            "vw" => {
                read.width = true;
                w
            }
            "vh" => {
                read.height = true;
                h
            }
            "vmin" | "vmax" => {
                read.width = true;
                read.height = true;
                if viewport == "vmin" {
                    w.min(h)
                } else {
                    w.max(h)
                }
            }
            "rem" => root as f64,
            "em" if !keeps_em(name) => em as f64,
            "lh" => lh as f64,
            "rlh" => basis.rlh as f64,
            "ch" => (ch * em) as f64,
            "ex" => (ex * em) as f64,
            "cap" => (cap * em) as f64,
            "ic" => (ic * em) as f64,
            "rch" => (ch * root) as f64,
            "rex" => (ex * root) as f64,
            "rcap" => (cap * root) as f64,
            "ric" => (ic * root) as f64,
            "cqw" | "cqi" | "cqh" | "cqb" | "cqmin" | "cqmax" => {
                let mut axis = |i: usize| match basis.containers[i] {
                    Some((key, size)) => {
                        read.containers[i] = Some(key);
                        size as f64 / 100.0
                    }
                    None => {
                        if i % 2 == 0 {
                            read.width = true;
                            w
                        } else {
                            read.height = true;
                            h
                        }
                    }
                };
                match unit {
                    "cqw" => axis(0),
                    "cqh" => axis(1),
                    "cqi" => axis(2),
                    "cqb" => axis(3),
                    "cqmin" => axis(2).min(axis(3)),
                    _ => axis(2).max(axis(3)),
                }
            }
            _ => continue,
        };
        let n: f64 = text[start..end - unit.len()].parse().unwrap_or(0.0);
        out.push_str(&text[last..start]);
        out.push_str(&number(n * per));
        out.push_str("px");
        last = end;
    }
    out.push_str(&text[last..]);
    (out, read)
}

/// `text` with each `sibling-index()` and `sibling-count()` written as the
/// element's integer (CSS Values 5 §8.1).
fn tree_counts(text: &str, basis: Basis) -> String {
    if !text.contains("sibling-") {
        return text.into();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("sibling-") {
        let boundary = at == 0 || {
            let p = rest.as_bytes()[at - 1];
            !(p.is_ascii_alphanumeric() || p == b'-' || p == b'_')
        };
        let value = [("sibling-index(", 0), ("sibling-count(", 1)]
            .into_iter()
            .filter(|_| boundary)
            .find_map(|(name, i)| {
                let after = rest[at..].strip_prefix(name)?;
                let close = after.find(')')?;
                after[..close]
                    .trim()
                    .is_empty()
                    .then(|| (basis.siblings[i].max(1), at + name.len() + close + 1))
            });
        match value {
            Some((n, end)) => {
                out.push_str(&rest[..at]);
                out.push_str(&n.to_string());
                rest = &rest[end..];
            }
            None => {
                out.push_str(&rest[..at + 1]);
                rest = &rest[at + 1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// `text`'s random functions resolved for an element (CSS Values 5 §9):
/// each key still naming its property simplified ([`random_keys`]), each
/// `random-item()` replaced by the item it picks and each `random()`'s key
/// written as its random base value, `random(fixed <base>, …)`.
pub fn randoms(text: &str, basis: Basis, property: &str) -> String {
    if !text.contains("random") {
        return text.into();
    }
    let text = random_keys(text, property);
    let text = random_items(&text, basis);
    random_bases(&text, basis)
}

/// The start of each `random(` or `random-item(` in `text` (not a longer
/// name ending in one), its name and its closing `)`.
fn random_calls(text: &str) -> impl Iterator<Item = (usize, &'static str, usize)> + '_ {
    let mut from = 0;
    std::iter::from_fn(move || {
        while let Some(at) = text[from..].find("random").map(|at| from + at) {
            from = at + 1;
            let name = if text[at..].starts_with("random(") {
                "random"
            } else if text[at..].starts_with("random-item(") {
                "random-item"
            } else {
                continue;
            };
            let boundary = at == 0 || {
                let p = text.as_bytes()[at - 1];
                !(p.is_ascii_alphanumeric() || p == b'-' || p == b'_')
            };
            if let Some(close) = closing(text, at + name.len()).filter(|_| boundary) {
                return Some((at, name, close));
            }
        }
        None
    })
}

/// `text` (a declaration of `property`) with each random function's key
/// simplified as at parse time (CSS Values 5 §9.4.1): `auto` (or none)
/// is `element-scoped property-index-scoped`, `property-scoped` becomes
/// `ua-<property>` and `property-index-scoped` `ua-<property>-<index>`,
/// the index counting the declaration's random functions from 1 in the
/// order written, before a shorthand expands. A key already simplified
/// (where the declaration was parsed) is kept, and still counted.
pub fn random_keys(text: &str, property: &str) -> String {
    fn walk(text: &str, property: &str, index: &mut u32) -> String {
        let mut out = String::with_capacity(text.len());
        let mut last = 0;
        for (at, name, close) in random_calls(text) {
            if at < last {
                continue;
            }
            *index += 1;
            let place = *index;
            let body = &text[at + name.len() + 1..close];
            let (key, args) = match split_args(body).split_first() {
                Some((k, rest)) if random_key(k).is_some() => (random_key(k), rest.to_vec()),
                _ if name == "random-item" => (None, Vec::new()),
                _ => (None, split_args(body)),
            };
            out.push_str(&text[last..at]);
            last = close + 1;
            let Some(key) = key.or((name == "random").then_some(RandomKey::AUTO)) else {
                // A random-item() whose key is no key: left to fail.
                out.push_str(&text[at..=close]);
                continue;
            };
            let key = match key {
                RandomKey::Fixed(_) => split_args(body)[0].trim().to_string(),
                RandomKey::Name(mut n) => {
                    let ua = match n.scope {
                        Scope::None => n.ua.map(str::to_string),
                        Scope::Property => Some(format!("ua-{property}")),
                        Scope::Index => Some(format!("ua-{property}-{place}")),
                    };
                    n.scope = Scope::None;
                    [n.dashed, if n.element { "element-scoped" } else { "" }]
                        .into_iter()
                        .filter(|w| !w.is_empty())
                        .map(str::to_string)
                        .chain(ua)
                        .collect::<Vec<_>>()
                        .join(" ")
                }
            };
            let args: Vec<String> = args
                .iter()
                .map(|a| walk(a.trim(), property, index))
                .collect();
            out.push_str(&format!("{name}({}, {})", key.trim(), args.join(", ")));
        }
        out.push_str(&text[last..]);
        out
    }
    walk(text, property, &mut 0)
}

/// `text`'s top-level comma-separated arguments: commas inside brackets,
/// braces or a string do not split.
fn split_args(text: &str) -> Vec<&str> {
    let (mut out, mut depth, mut quote, mut from) = (Vec::new(), 0i32, None, 0);
    for (i, c) in text.char_indices() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '(' | '[' | '{') => depth += 1,
            (None, ')' | ']' | '}') => depth -= 1,
            (None, ',') if depth == 0 => {
                out.push(&text[from..i]);
                from = i + 1;
            }
            _ => {}
        }
    }
    out.push(&text[from..]);
    out
}

/// The random base value (CSS Values 5 §9.4) of a simplified key on the
/// element and document `basis` names: `fixed`'s number, else a uniform
/// [0, 1) hashed from the random cache name, the tuple of its idents, the
/// element when it is element-scoped, and the document.
fn random_base(key: &RandomKey<'_>, basis: Basis) -> f64 {
    let n = match key {
        RandomKey::Fixed(r) => return *r,
        RandomKey::Name(n) => n,
    };
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    let mut eat = |bytes: &[u8]| {
        for &b in bytes {
            h = (h ^ b as u64).wrapping_mul(0x100_0000_01b3);
        }
        h = (h ^ 0xff).wrapping_mul(0x100_0000_01b3);
    };
    eat(n.dashed.as_bytes());
    eat(n.ua.unwrap_or("").as_bytes());
    eat(&if n.element { basis.element } else { 0 }.to_le_bytes());
    eat(&basis.document.to_le_bytes());
    // SplitMix64's finaliser, then 53 bits: a uniform [0, 1).
    let mut z = h.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^= z >> 31;
    (z >> 11) as f64 / (1u64 << 53) as f64
}

/// `text` with each `random-item(<key>, <item>#)` replaced by its item
/// `floor(R × N)` (CSS Values 5 §9.2, §9.3), braces around an item
/// dropped and the item's own random-items replaced in turn. One whose
/// key is no key is kept, and fails where the value is read.
fn random_items(text: &str, basis: Basis) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for (at, name, close) in random_calls(text) {
        if at < last || name != "random-item" {
            continue;
        }
        let args = split_args(&text[at + name.len() + 1..close]);
        let Some(key) = random_key(args[0]).filter(|_| args.len() > 1) else {
            continue;
        };
        let items = &args[1..];
        let i = ((random_base(&key, basis) * items.len() as f64) as usize).min(items.len() - 1);
        let item = items[i].trim();
        let item = item
            .strip_prefix('{')
            .and_then(|s| s.strip_suffix('}'))
            .unwrap_or(item)
            .trim();
        out.push_str(&text[last..at]);
        out.push_str(&random_items(item, basis));
        last = close + 1;
    }
    out.push_str(&text[last..]);
    out
}

/// `text` with each `random()`'s simplified key written as its random
/// base value, `random(fixed <base>, …)` ([`random_base`]).
fn random_bases(text: &str, basis: Basis) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for (at, name, close) in random_calls(text) {
        if at < last || name != "random" {
            continue;
        }
        let body = &text[at + name.len() + 1..close];
        let (key, args) = match split_top(body) {
            Some((k, a)) if random_key(k).is_some() => (random_key(k), a),
            _ => (None, body),
        };
        let base = random_base(&key.unwrap_or(RandomKey::AUTO), basis);
        out.push_str(&text[last..at]);
        out.push_str(&format!(
            "random(fixed {base}, {})",
            random_bases(args, basis).trim()
        ));
        last = close + 1;
    }
    out.push_str(&text[last..]);
    out
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Scope {
    None,
    Property,
    Index,
}

/// A `<random-key>`'s cache name parts (CSS Values 5 §9.4): its
/// `<dashed-ident>` (`""`: none), its `<random-ua-ident>` once simplified,
/// whether it is element-scoped and how it still reads its property.
#[derive(Clone, Copy)]
struct KeyName<'a> {
    dashed: &'a str,
    ua: Option<&'a str>,
    element: bool,
    scope: Scope,
}

/// A random function's `<random-key>`: `fixed <number>` or a cache name.
enum RandomKey<'a> {
    Fixed(f64),
    Name(KeyName<'a>),
}

impl RandomKey<'_> {
    /// `auto`: `element-scoped property-index-scoped`.
    const AUTO: RandomKey<'static> = RandomKey::Name(KeyName {
        dashed: "",
        ua: None,
        element: true,
        scope: Scope::Index,
    });
}

/// `text` as a `<random-key>`; `None`: it is not one.
fn random_key(text: &str) -> Option<RandomKey<'_>> {
    let words: Vec<&str> = text.split_whitespace().collect();
    match words.as_slice() {
        ["auto"] => return Some(RandomKey::AUTO),
        ["fixed", n] => return n.parse().ok().map(RandomKey::Fixed),
        [] => return None,
        _ => {}
    }
    let mut n = KeyName {
        dashed: "",
        ua: None,
        element: false,
        scope: Scope::None,
    };
    for w in words {
        let unscoped = n.scope == Scope::None && n.ua.is_none();
        match w {
            "element-scoped" if !n.element => n.element = true,
            "property-scoped" if unscoped => n.scope = Scope::Property,
            "property-index-scoped" if unscoped => n.scope = Scope::Index,
            w if w.starts_with("ua-") && w.len() > 3 && unscoped => n.ua = Some(w),
            w if w.starts_with("--") && w.len() > 2 && n.dashed.is_empty() => n.dashed = w,
            _ => return None,
        }
    }
    Some(RandomKey::Name(n))
}

/// `text` with each `env(<name>[, <fallback>])` replaced (CSS Environment
/// Variables): the safe-area insets as px (the `max` ones too, one safe
/// area here) and the HUD band's (`-og-hud-inset-*`), any other name by
/// its fallback, else nothing.
fn env(text: &str, basis: Basis, read: &mut Read) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("env(") {
        let boundary = at == 0 || {
            let p = rest.as_bytes()[at - 1];
            !(p.is_ascii_alphanumeric() || p == b'-')
        };
        let Some(close) = closing(rest, at + 3).filter(|_| boundary) else {
            out.push_str(&rest[..at + 4]);
            rest = &rest[at + 4..];
            continue;
        };
        out.push_str(&rest[..at]);
        let body = &rest[at + 4..close];
        let (name, fallback) = match split_top(body) {
            Some((n, f)) => (n.trim(), Some(f.trim())),
            None => (body.trim(), None),
        };
        let side = name
            .strip_prefix("safe-area-inset-")
            .or_else(|| name.strip_prefix("safe-area-max-inset-"))
            .map(|s| (s, &basis.safe))
            .or_else(|| name.strip_prefix("-og-hud-inset-").map(|s| (s, &basis.hud)));
        let value = side.and_then(|(s, insets)| {
            ["top", "right", "bottom", "left"]
                .iter()
                .position(|w| *w == s)
                .map(|k| {
                    read.width = true;
                    read.height = true;
                    format!("{}px", number(insets[k] as f64))
                })
        });
        out.push_str(&value.unwrap_or_else(|| fallback.unwrap_or("").to_string()));
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    out
}

/// `text` with each `attr(<name> [<type>][, <fallback>])` (CSS Values 5
/// §7.7) replaced by the element's attribute `name`: as a string with no
/// type (or `raw-string`), as written for `type(…)`, and a number with a
/// unit keyword (`px`, `%`, `deg`…) or `number`. An absent attribute, or
/// one that is no number where a number is asked, takes the fallback;
/// without one the declaration is invalid.
pub fn attr(text: &str, lookup: &dyn Fn(&str) -> Option<String>) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("attr(") {
        let boundary = at == 0 || {
            let p = rest.as_bytes()[at - 1];
            !(p.is_ascii_alphanumeric() || p == b'-')
        };
        let Some(close) = closing(rest, at + 4).filter(|_| boundary) else {
            out.push_str(&rest[..at + 5]);
            rest = &rest[at + 5..];
            continue;
        };
        out.push_str(&rest[..at]);
        let body = &rest[at + 5..close];
        let (head, fallback) = match split_top(body) {
            Some((h, f)) => (h.trim(), Some(f.trim())),
            None => (body.trim(), None),
        };
        let (name, ty) = match head.split_once(char::is_whitespace) {
            Some((n, t)) => (n, t.trim()),
            None => (head, ""),
        };
        let value = lookup(name).and_then(|v| {
            let v = v.trim().to_string();
            let numeric = || v.parse::<f64>().ok().map(number);
            match ty {
                "" | "raw-string" => Some(format!(
                    "\"{}\"",
                    v.replace('\\', "\\\\").replace('"', "\\\"")
                )),
                "number" => numeric(),
                t if t.starts_with("type(") => (!v.is_empty()).then_some(v.clone()),
                unit if UNITS.iter().any(|(w, ..)| w.eq_ignore_ascii_case(unit))
                    || ABSOLUTE.contains(&unit) =>
                {
                    numeric().map(|n| format!("{n}{unit}"))
                }
                _ => None,
            }
        });
        match value.or_else(|| fallback.map(str::to_string)) {
            Some(v) => out.push_str(&v),
            None => {
                return Err(format!(
                    "attr({head}) has no value: the element has no {name} attribute of that type, and no fallback"
                ));
            }
        }
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// `s` split at its first comma outside parentheses.
fn split_top(s: &str) -> Option<(&str, &str)> {
    let mut depth = 0;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => return Some((&s[..i], &s[i + 1..])),
            _ => {}
        }
    }
    None
}

/// Every number token in `text` outside strings and colours: its start, its
/// end and its unit.
fn numbers(text: &str) -> impl Iterator<Item = (usize, usize, &str)> {
    let b = text.as_bytes();
    let mut i = 0;
    let mut quote = None;
    std::iter::from_fn(move || {
        while i < b.len() {
            let c = b[i];
            if let Some(q) = quote {
                if c == q {
                    quote = None;
                }
                i += 1;
                continue;
            }
            if c == b'"' || c == b'\'' {
                quote = Some(c);
                i += 1;
                continue;
            }
            let prev = if i == 0 { b' ' } else { b[i - 1] };
            let starts =
                c.is_ascii_digit() || (c == b'.' && b.get(i + 1).is_some_and(u8::is_ascii_digit));
            if !starts || prev.is_ascii_alphanumeric() || prev == b'#' || prev == b'_' {
                i += 1;
                continue;
            }
            let start = if prev == b'-' || prev == b'+' {
                i - 1
            } else {
                i
            };
            while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'.') {
                i += 1;
            }
            let us = i;
            while i < b.len() && (b[i].is_ascii_alphabetic() || b[i] == b'%') {
                i += 1;
            }
            return Some((start, i, &text[us..i]));
        }
        None
    })
}

/// `text` with every math function reduced to its value (`calc(0.5 *
/// 100%)` → `50%`, `calc(100% - 2 * 6px)` → `calc(100% - 12px)`). Other
/// text is unchanged.
pub fn reduce(text: &str) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some((at, name)) = next_function(rest) {
        // A `calc-size()` keeps its calculation, which reads `size`, for
        // layout ([`calc_size`]).
        if let Some(start) = rest[..at].rfind("calc-size(")
            && let Some(close) = closing(rest, start + 9)
            && close > at
        {
            out.push_str(&rest[..=close]);
            rest = &rest[close + 1..];
            continue;
        }
        // A relative colour's channels read its origin's (`calc(h + 180)`
        // in `oklch(from var(--accent) …)`): the colour reduces them.
        if let Some((start, close)) = relative_colour(rest)
            && start < at
        {
            out.push_str(&rest[..=close]);
            rest = &rest[close + 1..];
            continue;
        }
        out.push_str(&rest[..at]);
        let open = at + name.len();
        let close = closing(rest, open).ok_or_else(|| format!("{name}( is not closed"))?;
        let body = &rest[open + 1..close];
        // `anchor()` and `anchor-size()` read other elements' boxes:
        // layout resolves their math (CSS Anchor Positioning 1 §4).
        if body.contains("anchor(") || body.contains("anchor-size(") {
            out.push_str(&rest[at..=close]);
            rest = &rest[close + 1..];
            continue;
        }
        let mut p = Parser {
            s: body.as_bytes(),
            i: 0,
            size: false,
        };
        let value = p.function(name)?;
        out.push_str(&format(&value));
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// The first relative colour in `s` (`oklch(from …)`): where its name
/// starts, and its `)`.
fn relative_colour(s: &str) -> Option<(usize, usize)> {
    const COLOURS: [&str; 11] = [
        "rgb", "rgba", "hsl", "hsla", "hwb", "lab", "lch", "oklab", "oklch", "color", "alpha",
    ];
    let mut from = 0;
    while let Some(open) = s[from..].find('(').map(|i| from + i) {
        let name = s[..open]
            .bytes()
            .rev()
            .take_while(|b| b.is_ascii_alphanumeric() || *b == b'-')
            .count();
        let start = open - name;
        let origin = s[open + 1..].trim_start().strip_prefix("from");
        if COLOURS
            .iter()
            .any(|c| s[start..open].eq_ignore_ascii_case(c))
            && origin.is_some_and(|r| r.starts_with(char::is_whitespace))
        {
            return Some((start, closing(s, open)?));
        }
        from = open + 1;
    }
    None
}

/// A `calc-size()` (CSS Values 5 §8.1) as layout evaluates it:
/// `scale × size + pct × the percentage basis + px`, where `size` is the
/// box's size for `basis`, an intrinsic size keyword. A length basis (or
/// none, `scale` 0) and a nested `calc-size()` fold into the sum.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalcSize {
    /// `auto`, `min-content`, `max-content`, `fit-content` or `stretch`;
    /// `None` when the basis was a length.
    pub basis: Option<&'static str>,
    pub scale: f64,
    /// A fraction (0–1) of the percentage basis.
    pub pct: f64,
    pub px: f64,
}

/// `text`, a whole `calc-size(<basis>, <calculation>)` with its font and
/// viewport units already px, as a [`CalcSize`]. The calculation must be
/// linear in `size` (a sum of `size` times a number and lengths), which is
/// what interpolating between sizes produces.
pub fn calc_size(text: &str) -> Result<CalcSize, String> {
    const HOW: &str = "calc-size(<basis>, <calculation>): an intrinsic size keyword (auto, min-content, max-content, fit-content, stretch), a length or a calc-size(), then a calculation over size, such as calc-size(auto, size * 0.5)";
    let text = text.trim();
    let open = "calc-size".len();
    if !text.starts_with("calc-size(") || closing(text, open) != Some(text.len() - 1) {
        return Err(HOW.into());
    }
    let (basis, calculation) = split_top(&text[open + 1..text.len() - 1]).ok_or(HOW)?;
    let basis = basis.trim();
    let lengths = |terms: Vec<(&'static str, f64)>, out: &mut CalcSize| -> Result<(), String> {
        for (unit, v) in terms {
            match unit {
                "size" => out.scale += v,
                "%" => out.pct += v / 100.0,
                "px" => out.px += v,
                "" if v == 0.0 => {}
                u => return Err(format!("calc-size(): a {u} is not a length here; {HOW}")),
            }
        }
        Ok(())
    };
    let inner = match basis {
        "auto" | "min-content" | "max-content" | "fit-content" | "stretch" => CalcSize {
            basis: [
                "auto",
                "min-content",
                "max-content",
                "fit-content",
                "stretch",
            ]
            .into_iter()
            .find(|k| *k == basis),
            scale: 1.0,
            pct: 0.0,
            px: 0.0,
        },
        b if b.starts_with("calc-size(") => calc_size(b)?,
        b => {
            let mut p = Parser {
                s: b.as_bytes(),
                i: 0,
                size: false,
            };
            let mut out = CalcSize {
                basis: None,
                scale: 0.0,
                pct: 0.0,
                px: 0.0,
            };
            lengths(terms(p.sum()?).map_err(|_| HOW.to_string())?, &mut out)?;
            p.space();
            if p.i < p.s.len() {
                return Err(HOW.into());
            }
            out
        }
    };
    let mut p = Parser {
        s: calculation.as_bytes(),
        i: 0,
        size: true,
    };
    let linear =
        "calc-size(): its calculation is linear in size (size times a number, plus lengths)";
    let sum = terms(p.sum()?).map_err(|_| linear.to_string())?;
    p.space();
    if p.i < p.s.len() {
        return Err(format!(
            "'{}' is not part of calc-size()'s calculation",
            String::from_utf8_lossy(&p.s[p.i..])
        ));
    }
    let mut outer = CalcSize {
        basis: None,
        scale: 0.0,
        pct: 0.0,
        px: 0.0,
    };
    lengths(sum, &mut outer)?;
    // `outer` reads `size`, the inner one's result.
    Ok(CalcSize {
        basis: inner.basis,
        scale: outer.scale * inner.scale,
        pct: outer.scale * inner.pct + outer.pct,
        px: outer.scale * inner.px + outer.px,
    })
}

/// Whether `text` holds a math function.
pub fn has_math(text: &str) -> bool {
    next_function(text).is_some()
}

/// The first math function in `s`: where its name starts, and the name.
fn next_function(s: &str) -> Option<(usize, &'static str)> {
    let b = s.as_bytes();
    // Every value the cascade resolves is asked, each frame it resolves:
    // most hold no call at all, and a name starts only at an ASCII letter.
    if !b.contains(&b'(') {
        return None;
    }
    let mut quote = None;
    let mut escaped = false;
    for (i, &c) in b.iter().enumerate() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == b'\\' {
            escaped = true;
            continue;
        }
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
            continue;
        }
        if c == b'"' || c == b'\'' {
            quote = Some(c);
            continue;
        }
        if !c.is_ascii_lowercase()
            || (i > 0
                && (b[i - 1].is_ascii_alphanumeric()
                    || matches!(b[i - 1], b'-' | b'_' | 128..=255)))
        {
            continue;
        }
        // The name runs to the first byte that could not continue it; only
        // a name called there can be a math function.
        let end = b[i..]
            .iter()
            .position(|&c| !(c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | 128..=255)))
            .map_or(b.len(), |n| i + n);
        if b.get(end) != Some(&b'(') {
            continue;
        }
        if let Some(f) = FUNCTIONS.iter().find(|f| f.as_bytes() == &b[i..end]) {
            return Some((i, f));
        }
    }
    None
}

/// The index of the `)` closing the `(` at `open`.
fn closing(s: &str, open: usize) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in s.bytes().enumerate().skip(open) {
        match c {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

pub fn number(v: f64) -> String {
    let mut n = format!("{v:.6}");
    while n.ends_with('0') {
        n.pop();
    }
    if n.ends_with('.') {
        n.pop();
    }
    if n == "-0" {
        n = "0".into();
    }
    n
}

/// A sum as CSS: one term bare (`12px`), more in a `calc()`
/// (`calc(100% - 12px)`).
pub fn sum_text(terms: &[(&str, f64)]) -> String {
    match terms {
        [] => "0".into(),
        [(u, v)] => format!("{}{u}", number(*v)),
        _ => {
            let mut s = String::from("calc(");
            for (k, (u, v)) in terms.iter().enumerate() {
                match k {
                    0 => s.push_str(&format!("{}{u}", number(*v))),
                    _ if *v < 0.0 => s.push_str(&format!(" - {}{u}", number(-v))),
                    _ => s.push_str(&format!(" + {}{u}", number(*v))),
                }
            }
            s.push(')');
            s
        }
    }
}

fn format(d: &Dim) -> String {
    match d {
        Dim::Sum(terms) => sum_text(terms),
        Dim::Fn(name, args) if name.starts_with("progress") => {
            let clamp = if *name == "progress" { "" } else { "no-clamp " };
            let f = format!(
                "progress({clamp}{}, {}, {})",
                sum_text(&args[0]),
                sum_text(&args[1]),
                sum_text(&args[2])
            );
            match args[3].as_slice() {
                [("", k)] if *k == 1.0 => f,
                by => format!("calc({f} * {})", sum_text(by)),
            }
        }
        Dim::Fn(name, args) => {
            let list = args
                .iter()
                .map(|a| sum_text(a))
                .collect::<Vec<_>>()
                .join(", ");
            match name.strip_prefix("round-") {
                Some(strategy) => format!("round({strategy}, {list})"),
                None => format!("{name}({list})"),
            }
        }
    }
}

/// `a + k·b`, term by term: units of one kind add, a bare 0 adds to any.
fn add(
    a: Vec<(&'static str, f64)>,
    b: Vec<(&'static str, f64)>,
    k: f64,
) -> Result<Vec<(&'static str, f64)>, String> {
    let zero = |t: &[(&str, f64)]| matches!(t, [("", v)] if *v == 0.0);
    if zero(&b) {
        return Ok(a);
    }
    if zero(&a) {
        return Ok(b.into_iter().map(|(u, v)| (u, k * v)).collect());
    }
    let kind = |t: &[(&'static str, f64)]| -> &'static str {
        if t.iter().all(|(u, _)| is_length(u)) {
            "length"
        } else {
            t[0].0
        }
    };
    let (ka, kb) = (kind(&a), kind(&b));
    if ka != kb {
        return Err(mixed(a[0].0, b[0].0));
    }
    let mut out = a;
    for (u, v) in b {
        match out.iter_mut().find(|(w, _)| *w == u) {
            Some((_, w)) => *w += k * v,
            None => out.push((u, k * v)),
        }
    }
    out.sort_by_key(|(u, _)| ORDER.iter().position(|o| o == u));
    // Keep one term for a sum that cancels out, so its unit survives.
    if out.len() > 1 {
        let first = out[0];
        out.retain(|(_, v)| *v != 0.0);
        if out.is_empty() {
            out.push((first.0, 0.0));
        }
    }
    Ok(out)
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
    /// It reads a `calc-size()` calculation, where `size` is the basis's
    /// size (CSS Values 5 §8.1).
    size: bool,
}

impl Parser<'_> {
    fn space(&mut self) {
        while self.s.get(self.i).is_some_and(u8::is_ascii_whitespace) {
            self.i += 1;
        }
    }

    fn eat(&mut self, c: u8) -> bool {
        self.space();
        if self.s.get(self.i) == Some(&c) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    /// A word at the cursor (`nearest`, `pi`), consumed.
    fn word(&mut self) -> Option<&str> {
        self.space();
        let start = self.i;
        while self
            .s
            .get(self.i)
            .is_some_and(|c| c.is_ascii_alphabetic() || *c == b'-')
        {
            self.i += 1;
        }
        if self.i == start || self.s.get(self.i) == Some(&b'(') {
            self.i = start;
            return None;
        }
        std::str::from_utf8(&self.s[start..self.i]).ok()
    }

    /// A function's arguments up to the end of its body.
    fn function(&mut self, name: &'static str) -> Result<Dim, String> {
        // `round()`'s rounding strategy (CSS Values 4 §10.3).
        let mut name = name;
        if name == "round" {
            let at = self.i;
            match self.word() {
                Some("nearest") => name = "round",
                Some("up") => name = "round-up",
                Some("down") => name = "round-down",
                Some("to-zero") => name = "round-to-zero",
                _ => self.i = at,
            }
            if self.i != at && !self.eat(b',') {
                return Err("round(<strategy>, A, B): a comma follows the strategy".into());
            }
        }
        // `progress(no-clamp? …)` (CSS Values 5 §5); `random(fixed R, …)`
        // (§9.1; [`random_bases`] writes any other key as its base value,
        // once the element is known).
        if name == "progress" {
            let at = self.i;
            if self.word() == Some("no-clamp") {
                name = "progress-no-clamp";
            } else {
                self.i = at;
            }
        }
        let mut base = None;
        if name == "random" {
            if self.word() != Some("fixed") {
                return Err("random() resolves on its element".into());
            }
            match self.value()? {
                Dim::Sum(t) if matches!(t.as_slice(), [("", r)] if (0.0..=1.0).contains(r)) => {
                    // A base is below 1 (§9.4).
                    base = Some(t[0].1.min(1.0 - f64::EPSILON));
                }
                _ => return Err("random(fixed <number>, …) takes a number from 0 to 1".into()),
            }
            if !self.eat(b',') {
                return Err("random(fixed <number>, A, B): a comma follows the key".into());
            }
        }
        let mut args = vec![self.sum()?];
        while self.eat(b',') {
            args.push(self.sum()?);
        }
        self.space();
        if self.i < self.s.len() {
            return Err(format!(
                "'{}' is not part of {name}()",
                String::from_utf8_lossy(&self.s[self.i..])
            ));
        }
        let (count, how) = match name {
            "calc" => (1..=1, "one expression"),
            "clamp" => (3..=3, "a minimum, a preferred value and a maximum"),
            "mod" | "rem" | "atan2" | "pow" => (2..=2, "two values"),
            n if n.starts_with("round") => (1..=2, "a value and its step"),
            "log" => (1..=2, "a value and its base"),
            n if n.starts_with("progress") => (3..=3, "a value, a start and an end"),
            "random" => (2..=3, "a minimum, a maximum and an optional step"),
            "min" | "max" | "hypot" => (1..=usize::MAX, "one or more values"),
            _ => (1..=1, "one value"),
        };
        if !count.contains(&args.len()) {
            return Err(format!("{name}() takes {how}"));
        }
        if name == "calc" {
            return Ok(match args.remove(0) {
                Dim::Sum(t) => Dim::Sum(t.into_iter().map(|(u, v)| (u, censor(v))).collect()),
                d => d,
            });
        }
        let mut sums: Vec<Vec<(&'static str, f64)>> = args
            .into_iter()
            .map(|a| match a {
                Dim::Sum(t) => Ok(t),
                Dim::Fn(inner, _) => Err(nested(inner)),
            })
            .collect::<Result<_, _>>()?;
        // `round(A)` steps by 1 of A's unit.
        if name.starts_with("round") && sums.len() == 1 {
            let unit = sums[0][0].0;
            sums.push(vec![(unit, 1.0)]);
        }
        if let Some(r) = base {
            return random(r, sums);
        }
        // One unit throughout: the value now.
        let unit = sums[0][0].0;
        if sums.iter().all(|t| t.len() == 1 && t[0].0 == unit) {
            let v: Vec<f64> = sums.iter().map(|t| t[0].1).collect();
            return evaluate(name, unit, &v).map(|(u, v)| Dim::Sum(vec![(u, censor(v))]));
        }
        if let Some(t) = sums.iter().find(|t| t.iter().any(|(u, _)| !is_length(u))) {
            let other = sums
                .iter()
                .find(|s| s[0].0 != t[0].0)
                .map_or("", |s| s[0].0);
            return Err(mixed(t[0].0, other));
        }
        // A progress() of lengths that mix % with px is a number layout
        // knows (`Size::Calc`), here times 1.
        if name.starts_with("progress") {
            sums.push(vec![("", 1.0)]);
            return Ok(Dim::Fn(name, sums));
        }
        if !LAYOUT_FUNCTIONS.contains(&name) {
            return Err(format!(
                "{name}() of lengths in more than one unit ({}) has no value before layout; give it lengths in one unit",
                sums.iter()
                    .map(|s| sum_text(s))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        Ok(Dim::Fn(name, sums))
    }

    fn sum(&mut self) -> Result<Dim, String> {
        let mut a = self.product()?;
        loop {
            // CSS needs spaces around + and -, so `-2px` stays a number.
            self.space();
            let op = match self.s.get(self.i) {
                Some(&c @ (b'+' | b'-'))
                    if self.s.get(self.i + 1).is_some_and(u8::is_ascii_whitespace) =>
                {
                    c
                }
                _ => return Ok(a),
            };
            self.i += 1;
            let b = self.product()?;
            a = Dim::Sum(add(
                terms(a)?,
                terms(b)?,
                if op == b'+' { 1.0 } else { -1.0 },
            )?);
        }
    }

    fn product(&mut self) -> Result<Dim, String> {
        let mut a = self.value()?;
        loop {
            if self.eat(b'*') {
                let b = self.value()?;
                // A progress() that mixes % with px scales by the other side.
                if is_progress(&a) || is_progress(&b) {
                    let (p, by) = if is_progress(&a) { (a, b) } else { (b, a) };
                    a = scale_progress(p, terms(by)?)?;
                    continue;
                }
                let (x, y) = (terms(a)?, terms(b)?);
                let (k, t) = match (x.as_slice(), y.as_slice()) {
                    ([("", k)], _) => (*k, y),
                    (_, [("", k)]) => (*k, x),
                    _ => {
                        return Err(format!(
                            "calc() multiplies a {} by a {}; one side is a plain number",
                            x[0].0, y[0].0
                        ));
                    }
                };
                a = Dim::Sum(t.into_iter().map(|(u, v)| (u, k * v)).collect());
            } else if self.eat(b'/') {
                let b = terms(self.value()?)?;
                let k = match b.as_slice() {
                    [("", k)] => *k,
                    _ => {
                        return Err(format!(
                            "calc() divides by a {}; divide by a plain number",
                            b[0].0
                        ));
                    }
                };
                if k == 0.0 {
                    return Err("calc() divides by zero".into());
                }
                if is_progress(&a) {
                    a = scale_progress(a, vec![("", 1.0 / k)])?;
                    continue;
                }
                a = Dim::Sum(terms(a)?.into_iter().map(|(u, v)| (u, v / k)).collect());
            } else {
                return Ok(a);
            }
        }
    }

    fn value(&mut self) -> Result<Dim, String> {
        self.space();
        if self.eat(b'(') {
            let v = self.sum()?;
            if !self.eat(b')') {
                return Err("a '(' in calc() is not closed".into());
            }
            return Ok(v);
        }
        let rest = std::str::from_utf8(&self.s[self.i..]).unwrap_or("");
        if let Some((_, name)) = next_function(rest).filter(|(at, _)| *at == 0) {
            let open = self.i + name.len();
            let text = std::str::from_utf8(self.s).unwrap_or("");
            let close = closing(text, open).ok_or_else(|| format!("{name}( is not closed"))?;
            let mut inner = Parser {
                s: &self.s[open + 1..close],
                i: 0,
                size: self.size,
            };
            self.i = close + 1;
            return inner.function(name);
        }
        let at = self.i;
        let negative = self.s.get(self.i) == Some(&b'-');
        if negative {
            self.i += 1;
        }
        let reads_size = self.size;
        if let Some(w) = self.word() {
            // CSS Values 4 §10.7.1's constants.
            let v = match w.to_ascii_lowercase().as_str() {
                "size" if reads_size => {
                    return Ok(Dim::Sum(vec![("size", if negative { -1.0 } else { 1.0 })]));
                }
                "e" => std::f64::consts::E,
                "pi" => std::f64::consts::PI,
                "infinity" => f64::INFINITY,
                "nan" => f64::NAN,
                _ => return Err(format!("calc() does not take '{w}'")),
            };
            return Ok(Dim::Sum(vec![("", if negative { -v } else { v })]));
        }
        self.i = at;
        let start = self.i;
        if matches!(self.s.get(self.i), Some(b'-' | b'+')) {
            self.i += 1;
        }
        while self
            .s
            .get(self.i)
            .is_some_and(|c| c.is_ascii_digit() || *c == b'.')
        {
            self.i += 1;
        }
        let number = std::str::from_utf8(&self.s[start..self.i]).unwrap_or("");
        let v: f64 = number.parse().map_err(|_| {
            format!(
                "'{}' is not a number in calc()",
                rest.chars().take(12).collect::<String>()
            )
        })?;
        let us = self.i;
        while self
            .s
            .get(self.i)
            .is_some_and(|c| c.is_ascii_alphabetic() || *c == b'%')
        {
            self.i += 1;
        }
        let written = std::str::from_utf8(&self.s[us..self.i]).unwrap_or("");
        // Units are ASCII case-insensitive (CSS Values 4 §6).
        let &(_, unit, k) = UNITS
            .iter()
            .find(|(w, ..)| w.eq_ignore_ascii_case(written))
            .ok_or_else(|| format!("calc() does not take {written}"))?;
        Ok(Dim::Sum(vec![(unit, v * k)]))
    }
}

/// `random()`'s value from its base `r` (CSS Values 5 §9.3): `min + r ×
/// (max − min)`, a sum in any units of one type; with a step, in one unit,
/// the `min + N × step` within `max` that `r` picks among them uniformly.
/// A maximum below the minimum is the minimum (§9.1.1), as is a step of 0
/// or less.
fn random(r: f64, sums: Vec<Vec<(&'static str, f64)>>) -> Result<Dim, String> {
    let unit = sums[0][0].0;
    let one: Option<Vec<f64>> = sums
        .iter()
        .map(|t| (t.len() == 1 && t[0].0 == unit).then_some(t[0].1))
        .collect();
    if let Some(v) = one {
        let (min, max) = (v[0], v[1].max(v[0]));
        let x = match v.get(2) {
            Some(&step) if step > 0.0 && ((max - min) / step).is_finite() => {
                // The epsilon is a thousandth of the step.
                let n = ((max - min) / step + 0.001).floor();
                min + (r * (n + 1.0)).floor().min(n) * step
            }
            _ => min + r * (max - min),
        };
        return Ok(Dim::Sum(vec![(unit, censor(x))]));
    }
    if sums.len() == 3 {
        return Err(format!(
            "random() with a step of lengths in more than one unit ({}) has no value before layout; give it lengths in one unit",
            sums.iter()
                .map(|s| sum_text(s))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let mut sums = sums.into_iter();
    let (min, max) = (sums.next().unwrap(), sums.next().unwrap());
    let span = add(max, min.clone(), -1.0)?;
    Ok(Dim::Sum(add(
        min,
        span.into_iter().map(|(u, v)| (u, v * r)).collect(),
        1.0,
    )?))
}

/// A math function of arguments in one `unit`: its unit and its value
/// (CSS Values 4 §10.3 to §10.7).
fn evaluate(name: &str, unit: &'static str, v: &[f64]) -> Result<(&'static str, f64), String> {
    let number = |what: &str| -> Result<(), String> {
        if unit.is_empty() {
            Ok(())
        } else {
            Err(format!("{name}() takes {what}, not a {unit}"))
        }
    };
    let radians = |x: f64| -> Result<f64, String> {
        match unit {
            "" => Ok(x),
            "deg" => Ok(x.to_radians()),
            u => Err(format!("{name}() takes an angle or a number, not a {u}")),
        }
    };
    let (a, b) = (v[0], v.get(1).copied().unwrap_or(0.0));
    Ok(match name {
        "min" => (unit, v.iter().copied().fold(f64::INFINITY, f64::min)),
        "max" => (unit, v.iter().copied().fold(f64::NEG_INFINITY, f64::max)),
        "clamp" => (unit, v[1].min(v[2]).max(v[0])),
        "round" | "round-up" | "round-down" | "round-to-zero" => (unit, round(name, a, b)),
        "mod" => (unit, a - b * (a / b).floor()),
        "rem" => (unit, a - b * (a / b).trunc()),
        "abs" => (unit, a.abs()),
        "sign" => ("", if a == 0.0 { a } else { a.signum() }),
        "hypot" => (unit, v.iter().map(|x| x * x).sum::<f64>().sqrt()),
        "sin" => ("", radians(a)?.sin()),
        "cos" => ("", radians(a)?.cos()),
        "tan" => ("", radians(a)?.tan()),
        "asin" | "acos" | "atan" => {
            number("a number")?;
            let r = match name {
                "asin" => a.asin(),
                "acos" => a.acos(),
                _ => a.atan(),
            };
            ("deg", r.to_degrees())
        }
        "atan2" => ("deg", a.atan2(b).to_degrees()),
        "pow" => {
            number("numbers")?;
            ("", a.powf(b))
        }
        "sqrt" => {
            number("a number")?;
            ("", a.sqrt())
        }
        "exp" => {
            number("a number")?;
            ("", a.exp())
        }
        // CSS Values 5 §5: clamped to [0, 1] unless no-clamp; 0 where the
        // start is the end, and clamped.
        "progress" | "progress-no-clamp" => {
            let (start, end) = (v[1], v[2]);
            let p = (a - start) / (end - start);
            match name {
                "progress" if start == end => ("", 0.0),
                "progress" => ("", p.clamp(0.0, 1.0)),
                _ => ("", p),
            }
        }
        "log" => {
            number("numbers")?;
            (
                "",
                if v.len() == 2 {
                    a.ln() / b.ln()
                } else {
                    a.ln()
                },
            )
        }
        _ => return Err(format!("{name}() is not a math function")),
    })
}

/// `round()`'s nearest multiple of `b` to `a` by its strategy: the
/// nearest (a tie upwards), up, down, or towards zero; a zero step gives
/// NaN, as CSS's does.
pub fn round(strategy: &str, a: f64, b: f64) -> f64 {
    let b = b.abs();
    if b == 0.0 {
        return f64::NAN;
    }
    let q = a / b;
    b * match strategy {
        "round-up" => q.ceil(),
        "round-down" => q.floor(),
        "round-to-zero" => q.trunc(),
        _ => (q + 0.5).floor(),
    }
}

/// A top-level result as CSS censors it (CSS Values 4 §10.9): NaN is 0
/// and an infinity the largest finite value.
fn censor(v: f64) -> f64 {
    if v.is_nan() {
        0.0
    } else {
        v.clamp(-(f32::MAX as f64), f32::MAX as f64)
    }
}

/// A value's terms; a `min()`, `max()` or `clamp()` left for layout takes
/// part in no arithmetic.
fn terms(d: Dim) -> Result<Vec<(&'static str, f64)>, String> {
    match d {
        Dim::Sum(t) => Ok(t),
        Dim::Fn(name, _) => Err(nested(name)),
    }
}

fn is_progress(d: &Dim) -> bool {
    matches!(d, Dim::Fn(name, _) if name.starts_with("progress"))
}

/// A progress() left for layout times `by`: its scale (the last argument,
/// a number until a length multiplies it) times `by`.
fn scale_progress(p: Dim, by: Vec<(&'static str, f64)>) -> Result<Dim, String> {
    let Dim::Fn(name, mut args) = p else {
        unreachable!("a progress()")
    };
    let scale = args.pop().expect("a progress() keeps its scale");
    let scale = match (scale.as_slice(), by.as_slice()) {
        (_, [("", k)]) => scale.iter().map(|&(u, v)| (u, v * k)).collect(),
        ([("", k)], _) if by.iter().all(|(u, _)| is_length(u)) => {
            by.iter().map(|&(u, v)| (u, v * k)).collect()
        }
        _ => {
            return Err(
                "a progress() of lengths in more than one unit scales by one length or number"
                    .into(),
            );
        }
    };
    args.push(scale);
    Ok(Dim::Fn(name, args))
}

fn nested(name: &str) -> String {
    format!(
        "this {name}() mixes units, so layout evaluates it; it stands alone as a value, not inside other math"
    )
}

fn mixed(a: &str, b: &str) -> String {
    let name = |u: &str| {
        if u.is_empty() {
            "a plain number".to_string()
        } else {
            u.to_string()
        }
    };
    format!(
        "calc() mixes {} and {}; only lengths (%, px, em, rem and viewport units) add to each other",
        name(a),
        name(b)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn math_functions_reduce_to_one_value() {
        for (text, want) in [
            ("calc(0.25 * 100%)", "25%"),
            ("calc(3 * 80ms)", "240ms"),
            ("calc(0.5s + 20ms)", "520ms"),
            ("calc((1 + 2) * 4px)", "12px"),
            ("calc(10px - 2 * 3px)", "4px"),
            ("calc(10px - -3px)", "13px"),
            ("calc(100px / 4)", "25px"),
            ("min(10px, 4px, 7px)", "4px"),
            ("max(1, 2)", "2"),
            ("clamp(0%, 130%, 100%)", "100%"),
            ("clamp(0%, calc(-0.2 * 100%), 100%)", "0%"),
            ("calc(0.5turn)", "180deg"),
            ("translate(calc(2 * 3px), 4px)", "translate(6px, 4px)"),
            ("12px minmax(80px, 1fr)", "12px minmax(80px, 1fr)"),
            // A relative colour's channels are the colour's to reduce.
            (
                "oklch(from var(--a) l c calc(h + 180)) calc(2 * 3px)",
                "oklch(from var(--a) l c calc(h + 180)) 6px",
            ),
            ("rgb(calc(255 / 5) 0 0)", "rgb(51 0 0)"),
        ] {
            assert_eq!(reduce(text).as_deref(), Ok(want), "{text}");
        }
    }

    #[test]
    fn lengths_in_several_units_reduce_to_a_linear_sum() {
        for (text, want) in [
            ("calc(100% - 12px)", "calc(100% - 12px)"),
            ("calc(12px + 100%)", "calc(100% + 12px)"),
            ("calc((100% - 2 * 8px) / 2)", "calc(50% - 8px)"),
            ("calc(1em + 50% - 4px + 1em)", "calc(50% - 4px + 2em)"),
            ("calc(100% - 12px + 12px)", "100%"),
            ("calc(50% - 50%)", "0%"),
            ("min(100%, 600px)", "min(100%, 600px)"),
            (
                "clamp(200px, calc(50% + 0px), 40em)",
                "clamp(200px, 50%, 40em)",
            ),
            (
                "max(calc(100% - 2 * 10px), 3em)",
                "max(calc(100% - 20px), 3em)",
            ),
            // Reducing a reduced value changes nothing.
            ("calc(50% - 8px)", "calc(50% - 8px)"),
        ] {
            assert_eq!(reduce(text).as_deref(), Ok(want), "{text}");
        }
        for bad in [
            "calc(2px * 3px)",
            "calc(4px / 2px)",
            "calc(1px / 0)",
            "clamp(1px, 2px)",
            "calc(1px+2px)",
            "calc(10px + 2)",
            "calc(10px + 20ms)",
            "min(10px, 20ms)",
            "calc(min(100%, 10px) + 2px)",
            "calc(2 * 3qx)",
        ] {
            assert!(reduce(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn viewport_units_and_rem_become_px_with_the_axes_they_read() {
        let basis = Basis {
            width: 1280.0,
            height: 720.0,
            rem: 16.0,
            ..Basis::default()
        };
        let (text, read) = absolute("calc(100vw - 2rem) 50vh", basis, "width");
        assert_eq!(text, "calc(1280px - 32px) 360px");
        assert_eq!(
            read,
            Read {
                width: true,
                height: true,
                ..Read::default()
            }
        );
        // `vi` and `vb` follow the root's writing mode.
        let (text, _) = absolute("10vi 10vb 10svi", basis, "width");
        assert_eq!(text, "128px 72px 128px");
        let vertical = Basis {
            vertical: true,
            ..basis
        };
        let (text, read) = absolute("10vi 10vb", vertical, "width");
        assert_eq!(text, "72px 128px");
        assert_eq!((read.width, read.height), (true, true));
        let (text, read) = absolute("-10vmin 1.5vmax #12vw0a 'a 3vw'", basis, "width");
        assert_eq!(text, "-72px 19.2px #12vw0a 'a 3vw'");
        assert_eq!(
            read,
            Read {
                width: true,
                height: true,
                ..Read::default()
            }
        );
        let (text, read) = absolute(
            "calc(50% - 1vw) 2em",
            Basis {
                width: 390.0,
                ..basis
            },
            "margin-left",
        );
        assert_eq!(text, "calc(50% - 3.9px) 2em");
        assert_eq!(
            read,
            Read {
                width: true,
                height: false,
                ..Read::default()
            }
        );
        assert!(has_absolute("1rem") && !has_absolute("1em 10px"));
        // The small, large and dynamic viewports are each their own (a
        // phone's browser bar shown, hidden, and as it is now); the
        // font-relative units read the element's step, a root one the
        // body's, and the face's ratios (0.5em for a face without them).
        let (text, _) = absolute(
            "100svh 100lvh 100dvh 100vh 10svmax 50lvw",
            Basis {
                height: 700.0,
                small: [1280.0, 600.0],
                large: [1300.0, 760.0],
                ..basis
            },
            "height",
        );
        assert_eq!(text, "600px 760px 700px 700px 128px 650px");
        let font = Basis {
            em: 20.0,
            lh: 30.0,
            rlh: 24.0,
            ch: 0.5,
            safe: [44.0, 0.0, 34.0, 0.0],
            ..basis
        };
        let (text, _) = absolute(
            "100dvh 50svw 10lvmax 2em 2ch 3ex 1lh 1rlh 2rch",
            font,
            "width",
        );
        assert_eq!(text, "720px 640px 128px 40px 20px 30px 30px 24px 16px");
        let (text, read) = absolute(
            "calc(env(safe-area-inset-top) + 8px) env(safe-area-inset-left, 3px) env(nope, 5px)",
            font,
            "padding",
        );
        assert_eq!(text, "calc(44px + 8px) 0px 5px");
        assert!(read.width && read.height);
        assert!(per_element("width", "2em") && !per_element("margin-top", "2em"));
        // Container units: an inline-size container answers the width axis
        // only; the height falls back to the viewport's.
        let (text, read) = absolute(
            "10cqw 10cqi 10cqh 10cqmin 10cqmax",
            Basis {
                containers: [Some((7, 300.0)), None, Some((7, 300.0)), None],
                ..basis
            },
            "width",
        );
        assert_eq!(text, "30px 30px 72px 30px 72px");
        assert_eq!(
            read,
            Read {
                width: false,
                height: true,
                containers: [Some(7), None, Some(7), None],
            }
        );
        // A vertical inline-size container: its inline axis is its height,
        // so it answers `cqi` and `cqh`, and `cqw` reads the viewport.
        let (text, _) = absolute(
            "10cqi 10cqh 10cqb",
            Basis {
                containers: [None, Some((8, 200.0)), Some((8, 200.0)), None],
                ..basis
            },
            "width",
        );
        assert_eq!(text, "20px 20px 72px");
    }

    #[test]
    fn stepped_trigonometric_and_exponential_functions_evaluate() {
        for (text, want) in [
            ("round(7px, 5px)", "5px"),
            ("round(7.5px, 5px)", "10px"),
            ("round(-7.5px, 5px)", "-5px"),
            ("round(up, 6px, 5px)", "10px"),
            ("round(down, -6px, 5px)", "-10px"),
            ("round(to-zero, -6px, 5px)", "-5px"),
            ("round(2.5)", "3"),
            ("mod(-7px, 3px)", "2px"),
            ("rem(-7px, 3px)", "-1px"),
            ("mod(7, -3)", "-2"),
            ("abs(-4ms)", "4ms"),
            ("calc(sign(-3px) * 10px)", "-10px"),
            ("calc(10px * sin(90deg))", "10px"),
            ("cos(0.5turn)", "-1"),
            ("tan(45deg)", "1"),
            ("atan2(1px, 1px)", "45deg"),
            ("acos(0)", "90deg"),
            ("pow(2, 10)", "1024"),
            ("sqrt(16)", "4"),
            ("hypot(3px, 4px)", "5px"),
            ("log(e)", "1"),
            ("log(8, 2)", "3"),
            ("exp(0)", "1"),
            ("calc(2 * pi)", "6.283185"),
            ("calc(1in - 48pt)", "32px"),
            ("calc(2.54cm + 10mm + 4Q)", "137.574803px"),
            ("calc(6pc)", "96px"),
            ("calc(100grad + 0.25turn)", "180deg"),
            ("round(1px, 0px)", "0px"),
            (
                "calc(1px * infinity)",
                "340282346638528859811704183484516925440px",
            ),
            ("round(50%, 10px)", "round(50%, 10px)"),
            (
                "round(up, 50% - 3px, 8px)",
                "round(up, calc(50% - 3px), 8px)",
            ),
            ("mod(100%, 7px)", "mod(100%, 7px)"),
            ("abs(calc(50% - 100px))", "abs(calc(50% - 100px))"),
        ] {
            assert_eq!(reduce(text).as_deref(), Ok(want), "{text}");
        }
        for bad in [
            "sign(50% - 10px)",
            "sin(10px)",
            "pow(2px, 2)",
            "mod(1px)",
            "round(sideways, 1px, 2px)",
            "atan2(1px, 1deg)",
            "calc(2 * tau)",
        ] {
            assert!(reduce(bad).is_err(), "{bad}");
        }
    }

    /// CSS Values 5 §5 and §9.3, the values worked by hand.
    #[test]
    fn progress_and_random_evaluate() {
        for (text, want) in [
            ("progress(50px, 0px, 200px)", "0.25"),
            ("progress(1s, 0ms, 4000ms)", "0.25"),
            ("calc(progress(300px, 0px, 200px) * 100px)", "100px"),
            ("progress(-50px, 0px, 200px)", "0"),
            ("progress(no-clamp 300px, 0px, 200px)", "1.5"),
            ("progress(no-clamp -50px, 0px, 200px)", "-0.25"),
            ("progress(5, 10, 10)", "0"),
            ("random(fixed 0.25, 100px, 200px)", "125px"),
            // Steps of 30px from 100px within 200px: 100, 130, 160, 190.
            ("random(fixed 0.5, 100px, 200px, 30px)", "160px"),
            ("random(fixed 0.24, 100px, 200px, 30px)", "100px"),
            ("random(fixed 1, 0px, 100px, 50px)", "100px"),
            ("random(fixed 0, 0px, 100px, 50px)", "0px"),
            ("random(fixed 0.5, 100px, 50px)", "100px"),
            ("random(fixed 0.5, 0px, 10px, 0px)", "5px"),
            ("random(fixed 0.5, 0deg, 1turn)", "180deg"),
            ("random(fixed 0.25, 50px, 100%)", "calc(25% + 37.5px)"),
            // Mixing % with px: a number layout knows, by its scale.
            (
                "calc(progress(50%, 0px, 200px) * 100px)",
                "calc(progress(50%, 0px, 200px) * 100px)",
            ),
            (
                "calc(2 * progress(no-clamp 50%, 10px, 200px) * 50% / 4)",
                "calc(progress(no-clamp 50%, 10px, 200px) * 25%)",
            ),
            ("progress(50%, 0px, 100px)", "progress(50%, 0px, 100px)"),
            ("progress(50%, 0%, 200%)", "0.25"),
        ] {
            assert_eq!(reduce(text).as_deref(), Ok(want), "{text}");
        }
        for bad in [
            "progress(1px, 0deg, 2px)",
            "calc(progress(50%, 0px, 200px) * 1px * 1px)",
            "calc(progress(50%, 0px, 200px) + 1px)",
            "min(progress(50%, 0px, 200px), 1)",
            "random(0px, 10px)",
            "random(fixed 2, 0px, 1px)",
            "random(fixed 0.5, 0px, 100%, 10px)",
        ] {
            assert!(reduce(bad).is_err(), "{bad}");
        }
    }

    /// A random() reads its key on its element (CSS Values 5 §9.4): one
    /// value each time it resolves, shared where the cache names match.
    #[test]
    fn random_keys_share_and_separate_values() {
        let at = |element: u64, property: &str, text: &str| {
            let basis = Basis {
                element,
                ..Basis::default()
            };
            reduce(&absolute(text, basis, property).0).unwrap()
        };
        let auto = "random(0px, 100px)";
        assert!(has_absolute(auto) && has_absolute("random(--k, 0px, 1px)"));
        assert_eq!(at(1, "width", auto), at(1, "width", auto));
        assert_ne!(at(1, "width", auto), at(2, "width", auto));
        assert_ne!(at(1, "width", auto), at(1, "height", auto));
        assert_eq!(
            at(1, "width", auto),
            at(1, "width", "random(auto, 0px, 100px)")
        );
        // Two in one declaration differ by their index; a name shares.
        let pair = |el, t: &str| {
            let v = at(el, "margin", t);
            let (a, b) = v.split_once(' ').unwrap();
            (a.to_string(), b.to_string())
        };
        let (a, b) = pair(1, "random(0px, 100px) random(0px, 100px)");
        assert_ne!(a, b);
        let named = "random(--k, 0px, 100px) random(--k, 0px, 100px)";
        let (a, b) = pair(1, named);
        assert_eq!(a, b);
        assert_eq!(pair(1, named), pair(2, named));
        let scoped = "random(--k element-scoped, 0px, 100px)";
        assert_ne!(at(1, "width", scoped), at(2, "width", scoped));
        let by_property = "random(--k property-scoped, 0px, 100px)";
        assert_eq!(at(1, "width", by_property), at(2, "width", by_property));
        assert_ne!(at(1, "width", by_property), at(1, "height", by_property));
        assert_eq!(at(7, "width", "random(fixed 0.5, 0px, 10px)"), "5px");
        assert!(!has_absolute("random(fixed 0.5, 0px, 10px)"));
        // Uniform over elements: each step taken, and a mean near the middle.
        let mut seen = [0; 3];
        let mut sum = 0.0;
        for el in 0..900 {
            let v = at(el, "width", "random(0px, 100px, 50px)");
            seen[["0px", "50px", "100px"]
                .iter()
                .position(|w| *w == v)
                .unwrap()] += 1;
            sum += at(el, "width", "random(0, 1)").parse::<f64>().unwrap();
        }
        assert!(seen.iter().all(|&n| (250..350).contains(&n)), "{seen:?}");
        assert!((sum / 900.0 - 0.5).abs() < 0.03, "{}", sum / 900.0);
    }

    /// CSS Values 5 §9.4.1: a key names the property it was parsed for
    /// (a shorthand's, kept as it expands) and its place among the
    /// declaration's random functions; the document is part of every name.
    #[test]
    fn random_keys_name_their_property_and_document() {
        for (property, text, want) in [
            (
                "margin",
                "random(0px, 10px) random(--k property-scoped, 1px, 2px)",
                "random(element-scoped ua-margin-1, 0px, 10px) random(--k ua-margin, 1px, 2px)",
            ),
            (
                "width",
                "random(fixed 0.5, 0px, random(--a property-index-scoped element-scoped, 1px, 2px))",
                "random(fixed 0.5, 0px, random(--a element-scoped ua-width-2, 1px, 2px))",
            ),
            (
                "width",
                "random-item(--x, a, b) calc(random(1px, 2px) * 2)",
                "random-item(--x, a, b) calc(random(element-scoped ua-width-2, 1px, 2px) * 2)",
            ),
        ] {
            let simple = random_keys(text, property);
            assert_eq!(simple, want, "{text}");
            // Simplified once, a key keeps its property in a longhand.
            assert_eq!(random_keys(&simple, "margin-left"), want);
        }
        let at = |element: u64, document: u64, property: &str, text: &str| {
            let basis = Basis {
                element,
                document,
                ..Basis::default()
            };
            reduce(&absolute(text, basis, property).0).unwrap()
        };
        let shared = "random(--k, 0px, 100px)";
        assert_eq!(at(1, 1, "width", shared), at(2, 1, "width", shared));
        assert_ne!(at(1, 1, "width", shared), at(1, 2, "width", shared));
        let scoped = "random(--k property-scoped, 0px, 100px)";
        let margin = at(1, 1, "margin", scoped);
        assert_ne!(margin, at(1, 1, "margin-left", scoped));
        let expanded = random_keys(scoped, "margin");
        assert_eq!(at(1, 1, "margin-left", &expanded), margin);
    }

    /// CSS Values 5 §8.1: the tree-counting functions are integers that
    /// take part in math; an element with no known place is first of one.
    #[test]
    fn sibling_index_and_count_are_the_element_s_integers() {
        let basis = Basis {
            siblings: [3, 5],
            ..Basis::default()
        };
        let at = |basis: Basis, text: &str| reduce(&absolute(text, basis, "width").0).unwrap();
        assert!(has_absolute("sibling-count()"));
        assert_eq!(at(basis, "sibling-index()"), "3");
        assert_eq!(at(basis, "sibling-count( )"), "5");
        assert_eq!(
            at(
                basis,
                "calc(sibling-index() * 10px) calc(100% / sibling-count())"
            ),
            "30px 20%"
        );
        assert_eq!(at(basis, "-og-sibling-index()"), "-og-sibling-index()");
        assert_eq!(
            at(Basis::default(), "calc(sibling-index() + sibling-count())"),
            "2"
        );
    }

    /// CSS Values 5 §9.2 and §9.3: random-item() is its item
    /// `floor(R × N)`, braces dropped, picked uniformly per element.
    #[test]
    fn random_item_picks_its_item() {
        let pick = |element: u64, text: &str| {
            let basis = Basis {
                element,
                ..Basis::default()
            };
            randoms(text, basis, "font-family")
        };
        for (text, want) in [
            ("random-item(fixed 0.5, a, b, c)", "b"),
            ("random-item(fixed 0, a, b)", "a"),
            ("random-item(fixed 1, a, b)", "b"),
            (
                "random-item(fixed 0.99, serif, {Times, serif})",
                "Times, serif",
            ),
            (
                "random-item(fixed 0.9, x, random-item(fixed 0.1, y, z))",
                "y",
            ),
            ("random-item(fixed 0.5, a, , c)", ""),
            ("1px random-item(fixed 0.7, \"a,b\", c) 2px", "1px c 2px"),
            (
                "random-item(not-a-key, a, b)",
                "random-item(not-a-key, a, b)",
            ),
        ] {
            assert_eq!(pick(0, text), want, "{text}");
        }
        assert!(has_absolute("random-item(--k, a, b)"));
        let mut seen = [0; 3];
        for el in 0..900 {
            let v = pick(el, "random-item(auto, a, b, c)");
            seen[["a", "b", "c"].iter().position(|w| *w == v).unwrap()] += 1;
        }
        assert!(seen.iter().all(|&n| (250..350).contains(&n)), "{seen:?}");
        // A name shared with random() shares its base R: 100R px, and the
        // item floor(4R).
        let r = pick(3, "random(--s, 0px, 100px) random-item(--s, a, b, c, d)");
        let (len, item) = r.rsplit_once(' ').unwrap();
        let px: f64 = reduce(len).unwrap().trim_end_matches("px").parse().unwrap();
        assert_eq!(item, ["a", "b", "c", "d"][(px / 25.0) as usize]);
    }
}
