//! The recipe's JSON, read into values with JavaScript's meaning: a key
//! present with `null` is there but nullish, the last of duplicate keys
//! wins, numbers are the nearest f64 to their text, and reading a value as
//! a number or a truth value follows ECMAScript (`ToNumber`, truthiness).

/// A JSON value.
#[derive(Clone, Debug, PartialEq)]
pub enum Val {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Val>),
    Obj(Vec<(String, Val)>),
}

impl Val {
    /// The member `key` as written (`null` included); `None` when absent.
    pub fn get(&self, key: &str) -> Option<&Val> {
        match self {
            Val::Obj(m) => m.iter().rev().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// The member `key` unless absent or `null` (what `??` falls through).
    pub fn opt(&self, key: &str) -> Option<&Val> {
        self.get(key).filter(|v| !matches!(v, Val::Null))
    }

    /// `L.key ?? d`, read as a number.
    pub fn n(&self, key: &str, d: f64) -> f64 {
        self.opt(key).map_or(d, Val::num)
    }

    /// Whether `L.key` is truthy.
    pub fn t(&self, key: &str) -> bool {
        self.get(key).is_some_and(Val::truthy)
    }

    /// `L.key` when it is a string.
    pub fn s(&self, key: &str) -> Option<&str> {
        match self.get(key) {
            Some(Val::Str(s)) => Some(s),
            _ => None,
        }
    }

    /// ECMAScript `ToNumber`.
    pub fn num(&self) -> f64 {
        match self {
            Val::Null => 0.0,
            Val::Bool(b) => f64::from(u8::from(*b)),
            Val::Num(n) => *n,
            Val::Str(s) => str_to_number(s),
            Val::Arr(a) => match a.as_slice() {
                [] => 0.0,
                [one] => str_to_number(&one.js_string()),
                _ => f64::NAN,
            },
            Val::Obj(_) => f64::NAN,
        }
    }

    /// ECMAScript truthiness.
    pub fn truthy(&self) -> bool {
        match self {
            Val::Null => false,
            Val::Bool(b) => *b,
            Val::Num(n) => super::jsmath::truthy(*n),
            Val::Str(s) => !s.is_empty(),
            Val::Arr(_) | Val::Obj(_) => true,
        }
    }

    /// ECMAScript `String(value)`.
    pub fn js_string(&self) -> String {
        match self {
            Val::Null => "null".into(),
            Val::Bool(b) => b.to_string(),
            Val::Num(n) => number_string(*n),
            Val::Str(s) => s.clone(),
            Val::Arr(a) => a
                .iter()
                .map(|v| match v {
                    Val::Null => String::new(),
                    v => v.js_string(),
                })
                .collect::<Vec<_>>()
                .join(","),
            Val::Obj(_) => "[object Object]".into(),
        }
    }
}

/// A number as JavaScript prints it (integers without a fraction).
pub fn number_string(n: f64) -> String {
    if n.is_nan() {
        "NaN".into()
    } else if n.is_infinite() {
        if n > 0.0 { "Infinity" } else { "-Infinity" }.into()
    } else if n == n.trunc() && n.abs() < 1e21 {
        format!("{n:.0}")
    } else {
        format!("{n}")
    }
}

fn str_to_number(s: &str) -> f64 {
    let t = s.trim();
    if t.is_empty() {
        return 0.0;
    }
    if let Some(h) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        return u64::from_str_radix(h, 16).map_or(f64::NAN, |v| v as f64);
    }
    match t.trim_start_matches(['+', '-']) {
        "Infinity" => {
            return if t.starts_with('-') {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            };
        }
        r if r.is_empty()
            || !r
                .bytes()
                .all(|b| b.is_ascii_digit() || b"eE.+-".contains(&b)) =>
        {
            return f64::NAN;
        }
        _ => {}
    }
    t.parse().unwrap_or(f64::NAN)
}

/// JavaScript's `parseFloat`: the longest number at the start of `s`.
pub fn parse_float(s: &str) -> f64 {
    let t = s.trim_start();
    let b = t.as_bytes();
    let mut i = usize::from(matches!(b.first(), Some(b'+' | b'-')));
    if t[i..].starts_with("Infinity") {
        return if b[0] == b'-' {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
    }
    let digits = |i: &mut usize| {
        let s = *i;
        while *i < b.len() && b[*i].is_ascii_digit() {
            *i += 1;
        }
        *i > s
    };
    let mut any = digits(&mut i);
    if i < b.len() && b[i] == b'.' {
        let mut j = i + 1;
        if digits(&mut j) || any {
            any = true;
            i = j;
        }
    }
    if !any {
        return f64::NAN;
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        let mut j = i + 1;
        if j < b.len() && (b[j] == b'+' || b[j] == b'-') {
            j += 1;
        }
        if digits(&mut j) {
            i = j;
        }
    }
    t[..i].trim_end_matches('.').parse().unwrap_or(f64::NAN)
}

/// Parses JSON text.
pub fn parse(text: &str) -> Result<Val, String> {
    let mut p = Parser {
        b: text.as_bytes(),
        i: 0,
    };
    let v = p.value(0)?;
    p.ws();
    if p.i != p.b.len() {
        return Err(p.err("text after the JSON value"));
    }
    Ok(v)
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn err(&self, what: &str) -> String {
        let line = 1 + self.b[..self.i.min(self.b.len())]
            .iter()
            .filter(|&&c| c == b'\n')
            .count();
        format!("recipe JSON, line {line}: {what}")
    }

    fn ws(&mut self) {
        while self.i < self.b.len() && matches!(self.b[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }

    fn eat(&mut self, lit: &str) -> bool {
        if self.b[self.i..].starts_with(lit.as_bytes()) {
            self.i += lit.len();
            true
        } else {
            false
        }
    }

    fn value(&mut self, depth: usize) -> Result<Val, String> {
        if depth > 64 {
            return Err(self.err("nested more than 64 deep"));
        }
        self.ws();
        match self.b.get(self.i) {
            None => Err(self.err("a value is missing")),
            Some(b'{') => {
                self.i += 1;
                let mut m = Vec::new();
                self.ws();
                if self.eat("}") {
                    return Ok(Val::Obj(m));
                }
                loop {
                    self.ws();
                    if self.b.get(self.i) != Some(&b'"') {
                        return Err(self.err("a key is a string"));
                    }
                    let k = self.string()?;
                    self.ws();
                    if !self.eat(":") {
                        return Err(self.err("`:` after a key"));
                    }
                    let v = self.value(depth + 1)?;
                    m.push((k, v));
                    self.ws();
                    if self.eat(",") {
                        continue;
                    }
                    if self.eat("}") {
                        return Ok(Val::Obj(m));
                    }
                    return Err(self.err("`,` or `}` in an object"));
                }
            }
            Some(b'[') => {
                self.i += 1;
                let mut a = Vec::new();
                self.ws();
                if self.eat("]") {
                    return Ok(Val::Arr(a));
                }
                loop {
                    a.push(self.value(depth + 1)?);
                    self.ws();
                    if self.eat(",") {
                        continue;
                    }
                    if self.eat("]") {
                        return Ok(Val::Arr(a));
                    }
                    return Err(self.err("`,` or `]` in an array"));
                }
            }
            Some(b'"') => Ok(Val::Str(self.string()?)),
            Some(b't') if self.eat("true") => Ok(Val::Bool(true)),
            Some(b'f') if self.eat("false") => Ok(Val::Bool(false)),
            Some(b'n') if self.eat("null") => Ok(Val::Null),
            Some(c) if *c == b'-' || c.is_ascii_digit() => {
                let s = self.i;
                self.i += 1;
                while self.i < self.b.len()
                    && (self.b[self.i].is_ascii_digit() || b".eE+-".contains(&self.b[self.i]))
                {
                    self.i += 1;
                }
                let t = std::str::from_utf8(&self.b[s..self.i]).expect("ASCII");
                og_core::ogt::Number::parse(t)
                    .ok()
                    .and_then(|_| t.parse::<f64>().ok())
                    .map(Val::Num)
                    .ok_or_else(|| self.err(&format!("`{t}` is not a number")))
            }
            Some(_) => Err(self.err("not a JSON value")),
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.i += 1;
        let mut out = String::new();
        loop {
            let Some(&c) = self.b.get(self.i) else {
                return Err(self.err("an unterminated string"));
            };
            self.i += 1;
            match c {
                b'"' => return Ok(out),
                b'\\' => {
                    let Some(&e) = self.b.get(self.i) else {
                        return Err(self.err("an unterminated string"));
                    };
                    self.i += 1;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let mut unit = self.hex4()?;
                            if (0xd800..0xdc00).contains(&unit) && self.eat("\\u") {
                                let lo = self.hex4()?;
                                unit = 0x10000
                                    + ((unit - 0xd800) << 10)
                                    + (lo.wrapping_sub(0xdc00) & 0x3ff);
                            }
                            out.push(char::from_u32(unit).unwrap_or('\u{fffd}'));
                        }
                        _ => return Err(self.err("an unknown string escape")),
                    }
                }
                _ => {
                    // Copy the rest of a UTF-8 sequence whole.
                    let s = self.i - 1;
                    let len = match c {
                        0..0x80 => 1,
                        0xc0..0xe0 => 2,
                        0xe0..0xf0 => 3,
                        _ => 4,
                    };
                    let e = (s + len).min(self.b.len());
                    out.push_str(&String::from_utf8_lossy(&self.b[s..e]));
                    self.i = e;
                }
            }
        }
    }

    fn hex4(&mut self) -> Result<u32, String> {
        let h = self
            .b
            .get(self.i..self.i + 4)
            .and_then(|h| std::str::from_utf8(h).ok())
            .and_then(|h| u32::from_str_radix(h, 16).ok())
            .ok_or_else(|| self.err("`\\u` takes four hex digits"))?;
        self.i += 4;
        Ok(h)
    }
}
