//! Recipe values with ECMAScript numeric coercion and truthiness. Checked TS
//! data enters directly; this module has no author text parser.

/// A recipe value.
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
