//! An FBX file's node tree: properties and nodes, parsed from the binary and the ASCII forms.

use super::*;

/// One property of an FBX node.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Prop {
    Int(i64),
    Num(f64),
    Str(String),
    Bytes(Vec<u8>),
    Arr(Vec<f64>),
}

impl Prop {
    pub(crate) fn num(&self) -> Option<f64> {
        match self {
            Prop::Int(v) => Some(*v as f64),
            Prop::Num(v) => Some(*v),
            _ => None,
        }
    }
    pub(crate) fn int(&self) -> Option<i64> {
        match self {
            Prop::Int(v) => Some(*v),
            Prop::Num(v) if v.fract() == 0.0 => Some(*v as i64),
            _ => None,
        }
    }
    pub(crate) fn str(&self) -> Option<&str> {
        match self {
            Prop::Str(s) => Some(s),
            _ => None,
        }
    }
    pub(crate) fn arr(&self) -> Option<&[f64]> {
        match self {
            Prop::Arr(v) => Some(v),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Node {
    pub(crate) name: String,
    pub(crate) props: Vec<Prop>,
    pub(crate) children: Vec<Node>,
}

impl Node {
    pub(crate) fn child(&self, name: &str) -> Option<&Node> {
        self.children.iter().find(|c| c.name == name)
    }
    pub(crate) fn all<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Node> + 'a {
        self.children.iter().filter(move |c| c.name == name)
    }
    /// The values of a `Properties70` entry named `key`.
    pub(crate) fn p70(&self, key: &str) -> Option<&[Prop]> {
        self.child("Properties70")?
            .all("P")
            .find(|p| p.props.first().and_then(Prop::str) == Some(key))
            .map(|p| p.props.get(4..).unwrap_or(&[]))
    }
    pub(crate) fn p70_vec(&self, key: &str, default: DVec3) -> DVec3 {
        match self.p70(key) {
            Some(v) if v.len() >= 3 => DVec3::new(
                v[0].num().unwrap_or(default.x),
                v[1].num().unwrap_or(default.y),
                v[2].num().unwrap_or(default.z),
            ),
            _ => default,
        }
    }
    pub(crate) fn p70_num(&self, key: &str) -> Option<f64> {
        self.p70(key)?.first()?.num()
    }
    /// A child's array: `Vertices: *n { a: … }` or a binary array.
    pub(crate) fn array(&self, name: &str) -> Option<&[f64]> {
        self.child(name)?.props.first()?.arr()
    }
    pub(crate) fn text(&self, name: &str) -> Option<&str> {
        self.child(name)?.props.first()?.str()
    }
}

pub(crate) struct Bin<'a> {
    pub(crate) b: &'a [u8],
    pub(crate) wide: bool,
}

impl Bin<'_> {
    pub(crate) fn u(&self, at: usize, n: usize) -> Result<u64, ImportError> {
        let s = self
            .b
            .get(at..at + n)
            .ok_or_else(|| fmt_err("FBX is cut short"))?;
        let mut v = 0u64;
        for (k, &x) in s.iter().enumerate() {
            v |= u64::from(x) << (8 * k);
        }
        Ok(v)
    }

    pub(crate) fn word(&self, at: usize) -> Result<(u64, usize), ImportError> {
        if self.wide {
            Ok((self.u(at, 8)?, at + 8))
        } else {
            Ok((self.u(at, 4)?, at + 4))
        }
    }

    /// The record at `at`: the node (or `None` for the closing null
    /// record) and where the next record starts.
    pub(crate) fn node(
        &self,
        at: usize,
        depth: usize,
    ) -> Result<(Option<Node>, usize), ImportError> {
        if depth > 64 {
            return Err(fmt_err("FBX nodes nest too deep"));
        }
        let (end, p) = self.word(at)?;
        let (count, p) = self.word(p)?;
        let (_list_len, p) = self.word(p)?;
        let name_len = self.u(p, 1)? as usize;
        let p = p + 1;
        if end == 0 {
            return Ok((None, p + name_len));
        }
        let end = end as usize;
        if end > self.b.len() || end <= at {
            return Err(fmt_err("an FBX record ends outside the file"));
        }
        let name = String::from_utf8_lossy(
            self.b
                .get(p..p + name_len)
                .ok_or_else(|| fmt_err("FBX is cut short"))?,
        )
        .into_owned();
        let mut p = p + name_len;
        let mut props = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let (prop, next) = self.prop(p)?;
            props.push(prop);
            p = next;
        }
        let mut children = Vec::new();
        while p < end {
            let (child, next) = self.node(p, depth + 1)?;
            p = next;
            match child {
                Some(c) => children.push(c),
                None => break,
            }
        }
        Ok((
            Some(Node {
                name,
                props,
                children,
            }),
            end,
        ))
    }

    pub(crate) fn prop(&self, at: usize) -> Result<(Prop, usize), ImportError> {
        let kind = self.u(at, 1)? as u8;
        let p = at + 1;
        let f = |bits: u64, n: usize| -> f64 {
            if n == 4 {
                f64::from(f32::from_bits(bits as u32))
            } else {
                f64::from_bits(bits)
            }
        };
        let signed = |v: u64, n: usize| -> i64 {
            let shift = 64 - 8 * n as u32;
            ((v << shift) as i64) >> shift
        };
        Ok(match kind {
            b'Y' => (Prop::Int(signed(self.u(p, 2)?, 2)), p + 2),
            b'C' => (Prop::Int(self.u(p, 1)? as i64), p + 1),
            b'I' => (Prop::Int(signed(self.u(p, 4)?, 4)), p + 4),
            b'L' => (Prop::Int(self.u(p, 8)? as i64), p + 8),
            b'F' => (Prop::Num(f(self.u(p, 4)?, 4)), p + 4),
            b'D' => (Prop::Num(f(self.u(p, 8)?, 8)), p + 8),
            b'S' | b'R' => {
                let n = self.u(p, 4)? as usize;
                let data = self
                    .b
                    .get(p + 4..p + 4 + n)
                    .ok_or_else(|| fmt_err("an FBX string runs past the end"))?;
                let prop = if kind == b'S' {
                    Prop::Str(String::from_utf8_lossy(data).into_owned())
                } else {
                    Prop::Bytes(data.to_vec())
                };
                (prop, p + 4 + n)
            }
            b'f' | b'd' | b'l' | b'i' | b'b' => {
                let count = self.u(p, 4)? as usize;
                let encoding = self.u(p + 4, 4)?;
                let len = self.u(p + 8, 4)? as usize;
                let data = self
                    .b
                    .get(p + 12..p + 12 + len)
                    .ok_or_else(|| fmt_err("an FBX array runs past the end"))?;
                let size = match kind {
                    b'f' | b'i' => 4,
                    b'b' => 1,
                    _ => 8,
                };
                let raw: Vec<u8> = match encoding {
                    0 => data.to_vec(),
                    1 => {
                        miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(data, count * size)
                            .map_err(|_| fmt_err("an FBX array does not inflate"))?
                    }
                    e => return Err(fmt_err(format!("FBX array encoding {e}"))),
                };
                if raw.len() < count * size {
                    return Err(fmt_err("an FBX array is shorter than its count"));
                }
                let mut v = Vec::with_capacity(count);
                for k in 0..count {
                    let mut bits = 0u64;
                    for j in 0..size {
                        bits |= u64::from(raw[k * size + j]) << (8 * j);
                    }
                    v.push(match kind {
                        b'f' | b'd' => f(bits, size),
                        b'b' => bits as f64,
                        _ => signed(bits, size) as f64,
                    });
                }
                (Prop::Arr(v), p + 12 + len)
            }
            k => return Err(fmt_err(format!("FBX property type {:?}", k as char))),
        })
    }
}

pub(crate) fn parse_binary(b: &[u8]) -> Result<(u32, Node), ImportError> {
    let version = u32::from_le_bytes(
        b.get(23..27)
            .ok_or_else(|| fmt_err("FBX header is cut short"))?
            .try_into()
            .map_err(|_| fmt_err("FBX header"))?,
    );
    let r = Bin {
        b,
        wide: version >= 7500,
    };
    let mut root = Node::default();
    let mut p = 27;
    let header = if r.wide { 25 } else { 13 };
    while p + header <= b.len() {
        let (node, next) = r.node(p, 0)?;
        match node {
            Some(n) => root.children.push(n),
            None => break,
        }
        p = next;
    }
    Ok((version, root))
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Tok {
    Key(String),
    Val(Prop),
    Count,
    Open,
    Close,
    Comma,
}

pub(crate) fn lex(text: &str) -> Result<Vec<Tok>, ImportError> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        match c {
            b' ' | b'\t' | b'\r' | b'\n' => i += 1,
            b';' => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            b'{' => {
                out.push(Tok::Open);
                i += 1;
            }
            b'}' => {
                out.push(Tok::Close);
                i += 1;
            }
            b',' => {
                out.push(Tok::Comma);
                i += 1;
            }
            b'*' => {
                out.push(Tok::Count);
                i += 1;
                while i < b.len() && b[i].is_ascii_digit() {
                    i += 1;
                }
            }
            b'"' => {
                let start = i + 1;
                i = start;
                while i < b.len() && b[i] != b'"' {
                    i += 1;
                }
                let s = text[start..i.min(b.len())].replace("&quot;", "\"");
                out.push(Tok::Val(Prop::Str(s)));
                i += 1;
            }
            _ => {
                let start = i;
                while i < b.len() && !b" \t\r\n,{};\"".contains(&b[i]) {
                    i += 1;
                }
                if i == start {
                    return Err(fmt_err(format!("FBX text: unexpected {:?}", c as char)));
                }
                let word = &text[start..i];
                if let Some(key) = word.strip_suffix(':') {
                    out.push(Tok::Key(key.to_string()));
                } else if let Ok(v) = word.parse::<i64>() {
                    out.push(Tok::Val(Prop::Int(v)));
                } else if let Ok(v) = word.parse::<f64>() {
                    out.push(Tok::Val(Prop::Num(v)));
                } else {
                    out.push(Tok::Val(Prop::Str(word.to_string())));
                }
            }
        }
    }
    Ok(out)
}

pub(crate) fn parse_ascii(text: &str) -> Result<Node, ImportError> {
    let toks = lex(text)?;
    let mut i = 0;
    let mut root = Node::default();
    fn node(toks: &[Tok], i: &mut usize, depth: usize) -> Result<Node, ImportError> {
        if depth > 64 {
            return Err(fmt_err("FBX nodes nest too deep"));
        }
        let Some(Tok::Key(name)) = toks.get(*i) else {
            return Err(fmt_err("FBX text: expected a key"));
        };
        *i += 1;
        let mut n = Node {
            name: name.clone(),
            ..Default::default()
        };
        let mut counted = false;
        loop {
            match toks.get(*i) {
                Some(Tok::Val(v)) => {
                    n.props.push(v.clone());
                    *i += 1;
                    if toks.get(*i) == Some(&Tok::Comma) {
                        *i += 1;
                    } else {
                        break;
                    }
                }
                Some(Tok::Count) => {
                    counted = true;
                    *i += 1;
                    break;
                }
                _ => break,
            }
        }
        if toks.get(*i) == Some(&Tok::Open) {
            *i += 1;
            while let Some(t) = toks.get(*i) {
                if *t == Tok::Close {
                    *i += 1;
                    break;
                }
                n.children.push(node(toks, i, depth + 1)?);
            }
        }
        if counted {
            // `Name: *n { a: v, v, … }` is an array.
            let values: Vec<f64> = n
                .child("a")
                .map(|a| a.props.iter().filter_map(Prop::num).collect())
                .unwrap_or_default();
            n.props = vec![Prop::Arr(values)];
            n.children.clear();
        }
        Ok(n)
    }
    while i < toks.len() {
        root.children.push(node(&toks, &mut i, 0)?);
    }
    Ok(root)
}

/// An object's name without its class: binary `Name\0\x01Class`, ASCII
/// `Class::Name`.
pub(crate) fn object_name(raw: &str) -> String {
    if let Some(i) = raw.find("\u{0}\u{1}") {
        return raw[..i].to_string();
    }
    match raw.find("::") {
        Some(i) => raw[i + 2..].to_string(),
        None => raw.to_string(),
    }
}
