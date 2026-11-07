//! Data binding and formatting (docs/ui.md §2.1): what a document reads from
//! its viewmodel and how it shows it.
//!
//! - A **viewmodel** is JSON ([`serde_json::Value`]): an object passed from
//!   code, or a reflected resource or component the host resolves by its
//!   name ([`Env::lookup`]). A **path** is dotted (`hero.health`,
//!   `items.3.name`, `items.length`).
//! - A **binding** is an [`Expr`]: paths, numbers, strings, `true`,
//!   `false`, `null`, `!`, unary `-`, `* / %`, `+ -` (`+` also joins
//!   strings), `< <= > >=`, `== !=`, `&&`, `||`, `a ? b : c` and
//!   parentheses (`hero.health / hero.maxHealth`, `!item.owned && price >
//!   wallet.gold`).
//! - A **template** is a text with arguments: an og-i18n message (`{name}`,
//!   `{n, number}`, `{n, plural, one {# coin} other {# coins}}`, `{k,
//!   select, …}`) whose argument names are paths, plus the engine's formats
//!   as argument types: `{gold, compact}`, `{t, clock}`, `{rank,
//!   ordinal}`, `{n, scientific, 4}` ([`Format`]).
//! - **Formats** ([`Format`], [`format_value`]): grouping, fixed decimals,
//!   compact (1.2K, 3.4M; 万/億 in Japanese, 万/亿 in Chinese, 만/억 in
//!   Korean), engineering and scientific notation, letter notation for idle
//!   games (K M B T, then aa ab …), percentages, durations, clocks and
//!   countdowns, ordinals (og-i18n). Every number goes through [`Num`], a
//!   mantissa and a power of ten, so a value past 1e308 (`{"m": 1.5, "e":
//!   400}` or the string `"1.5e400"` in a viewmodel) formats like any other.
//!
//! Formatting uses Rust's exact float formatting only (no logarithms), so
//! both hosts write the same text for the same value.

use serde_json::Value;
use std::collections::BTreeMap;

/// Internal CSS selector inputs, resolved against a mounted node's input state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InteractionState {
    Hover,
    Active,
    FocusVisible,
    Disabled,
}

/// What the host gives a binding beyond its viewmodel.
pub trait Env {
    /// A node-scoped CSS selector read; never an authored store root.
    fn interaction(&self, state: InteractionState) -> bool {
        let _ = state;
        false
    }
    /// A viewmodel root the viewmodel itself lacks: a reflected resource
    /// by name (`Wallet.gold` reads the `Wallet` resource).
    fn lookup(&self, root: &str) -> Option<Value> {
        let _ = root;
        None
    }
    /// The full locale tag formats and plurals follow, including script and region.
    fn language(&self) -> &str {
        "en"
    }
    /// A string table's message for `key` (og.i18n), `None` when absent.
    fn translate(&self, key: &str) -> Option<String> {
        let _ = key;
        None
    }
    /// The texture of an asset handle a document's `texture` binds (`og.load.texture`).
    fn texture(&self, handle: f64) -> Option<og_render::TextureHandle> {
        let _ = handle;
        None
    }
}

/// No host: viewmodels alone, English.
pub struct NoEnv;
impl Env for NoEnv {}

// ── paths ──────────────────────────────────────────────────────────────────

/// The value at a dotted path of `root`; `items.length` is a list's (or a
/// string's) length.
pub fn get_path<'a>(root: &'a Value, path: &str) -> Option<std::borrow::Cow<'a, Value>> {
    use std::borrow::Cow;
    let mut at: &Value = root;
    let mut parts = path.split('.').filter(|p| !p.is_empty()).peekable();
    while let Some(p) = parts.next() {
        let next = match at {
            Value::Object(o) => o.get(p),
            Value::Array(a) => {
                if p == "length" && parts.peek().is_none() {
                    return Some(Cow::Owned(Value::from(a.len())));
                }
                p.parse::<usize>().ok().and_then(|i| a.get(i))
            }
            Value::String(s) if p == "length" && parts.peek().is_none() => {
                return Some(Cow::Owned(Value::from(s.chars().count())));
            }
            _ => None,
        }?;
        at = next;
    }
    Some(Cow::Borrowed(at))
}

/// Writes `value` at a dotted path of `root`, making objects on the way.
/// False when the path goes through something that is not an object or a
/// list (or a list index past its end).
pub fn set_path(root: &mut Value, path: &str, value: Value) -> bool {
    let parts: Vec<&str> = path.split('.').filter(|p| !p.is_empty()).collect();
    let Some((last, head)) = parts.split_last() else {
        *root = value;
        return true;
    };
    let mut at = root;
    for p in head {
        if at.is_null() {
            *at = Value::Object(Default::default());
        }
        at = match at {
            Value::Object(o) => o
                .entry(p.to_string())
                .or_insert(Value::Object(Default::default())),
            Value::Array(a) => match p.parse::<usize>().ok().and_then(|i| a.get_mut(i)) {
                Some(v) => v,
                None => return false,
            },
            _ => return false,
        };
    }
    if at.is_null() {
        *at = Value::Object(Default::default());
    }
    match at {
        Value::Object(o) => {
            o.insert(last.to_string(), value);
            true
        }
        Value::Array(a) => match last.parse::<usize>().ok().and_then(|i| a.get_mut(i)) {
            Some(v) => {
                *v = value;
                true
            }
            None => false,
        },
        _ => false,
    }
}

/// JavaScript's truthiness: false, null, 0, NaN and "" are false.
pub fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|x| x != 0.0 && !x.is_nan()),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// A value as a number: numbers, numeric strings, booleans (1, 0) and big
/// numbers (`{m, e}`, at f64's range).
pub fn number(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::Bool(b) => Some(f64::from(u8::from(*b))),
        Value::String(s) => s.trim().parse().ok(),
        Value::Object(_) => Num::from_json(v).map(|n| n.to_f64()),
        _ => None,
    }
}

/// A value as text: a string as is, a number as Rust's shortest form
/// (integers without a fraction), true/false, empty for null.
pub fn text(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Number(n) => match n.as_f64() {
            Some(x) if x.fract() == 0.0 && x.abs() < 1e15 => format!("{}", x as i64),
            Some(x) => format!("{x}"),
            None => n.to_string(),
        },
        Value::Bool(b) => b.to_string(),
        other => other.to_string(),
    }
}

// ── expressions ────────────────────────────────────────────────────────────

/// A binding expression (see the module docs).
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Lit(Value),
    Path(String),
    Interaction(InteractionState),
    Translate(Box<Expr>, Vec<(String, Expr)>),
    Format(Format, Box<Expr>),
    Not(Box<Expr>),
    Neg(Box<Expr>),
    Bin(Op, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Mul,
    Div,
    Rem,
    Add,
    Sub,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    And,
    Or,
}

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f64),
    Str(String),
    Ident(String),
    Sym(&'static str),
}

fn lex(src: &str) -> Result<Vec<Tok>, String> {
    let c: Vec<char> = src.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    const SYMS: [&str; 21] = [
        "&&", "||", "==", "!=", "<=", ">=", "<", ">", "!", "+", "-", "*", "/", "%", "(", ")", "?",
        ":", ",", "{", "}",
    ];
    while i < c.len() {
        let ch = c[i];
        if ch.is_whitespace() {
            i += 1;
        } else if ch.is_ascii_digit()
            || (ch == '.' && c.get(i + 1).is_some_and(char::is_ascii_digit))
        {
            let s = i;
            while i < c.len() && (c[i].is_ascii_digit() || c[i] == '.') {
                i += 1;
            }
            if i < c.len() && (c[i] == 'e' || c[i] == 'E') {
                let mut j = i + 1;
                if j < c.len() && (c[j] == '-' || c[j] == '+') {
                    j += 1;
                }
                if j < c.len() && c[j].is_ascii_digit() {
                    i = j;
                    while i < c.len() && c[i].is_ascii_digit() {
                        i += 1;
                    }
                }
            }
            let t: String = c[s..i].iter().collect();
            out.push(Tok::Num(
                t.parse().map_err(|_| format!("'{t}' is not a number"))?,
            ));
        } else if ch == '"' || ch == '\'' {
            let q = ch;
            i += 1;
            let mut s = String::new();
            while i < c.len() && c[i] != q {
                if c[i] == '\\' && i + 1 < c.len() {
                    i += 1;
                }
                s.push(c[i]);
                i += 1;
            }
            if i >= c.len() {
                return Err(format!("a string opened with {q} is not closed"));
            }
            i += 1;
            out.push(Tok::Str(s));
        } else if ch.is_alphabetic() || ch == '_' || ch == '$' {
            let s = i;
            while i < c.len() && (c[i].is_alphanumeric() || matches!(c[i], '_' | '$' | '.' | '#')) {
                i += 1;
            }
            out.push(Tok::Ident(c[s..i].iter().collect()));
        } else {
            let rest: String = c[i..c.len().min(i + 2)].iter().collect();
            let Some(sym) = SYMS.iter().find(|s| rest.starts_with(**s)) else {
                return Err(format!(
                    "'{ch}' is not part of a binding (paths, numbers, strings, ! && || == != < <= > >= + - * / % ?: and parentheses)"
                ));
            };
            i += sym.chars().count();
            out.push(Tok::Sym(sym));
        }
    }
    Ok(out)
}

struct P {
    t: Vec<Tok>,
    i: usize,
}

impl P {
    fn peek(&self) -> Option<&Tok> {
        self.t.get(self.i)
    }
    fn eat(&mut self, s: &str) -> bool {
        if matches!(self.peek(), Some(Tok::Sym(x)) if *x == s) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn expr(&mut self) -> Result<Expr, String> {
        let c = self.bin(0)?;
        if self.eat("?") {
            let a = self.expr()?;
            if !self.eat(":") {
                return Err("'?' needs its ':' (a ? b : c)".into());
            }
            let b = self.expr()?;
            return Ok(Expr::If(Box::new(c), Box::new(a), Box::new(b)));
        }
        Ok(c)
    }
    fn bin(&mut self, level: usize) -> Result<Expr, String> {
        const LEVELS: [&[(&str, Op)]; 6] = [
            &[("||", Op::Or)],
            &[("&&", Op::And)],
            &[("==", Op::Eq), ("!=", Op::Ne)],
            &[("<=", Op::Le), (">=", Op::Ge), ("<", Op::Lt), (">", Op::Gt)],
            &[("+", Op::Add), ("-", Op::Sub)],
            &[("*", Op::Mul), ("/", Op::Div), ("%", Op::Rem)],
        ];
        if level == LEVELS.len() {
            return self.unary();
        }
        let mut left = self.bin(level + 1)?;
        'outer: loop {
            for (s, op) in LEVELS[level] {
                if self.eat(s) {
                    let right = self.bin(level + 1)?;
                    left = Expr::Bin(*op, Box::new(left), Box::new(right));
                    continue 'outer;
                }
            }
            return Ok(left);
        }
    }
    fn unary(&mut self) -> Result<Expr, String> {
        if self.eat("!") {
            return Ok(Expr::Not(Box::new(self.unary()?)));
        }
        if self.eat("-") {
            return Ok(Expr::Neg(Box::new(self.unary()?)));
        }
        if self.eat("(") {
            let e = self.expr()?;
            if !self.eat(")") {
                return Err("a '(' is not closed".into());
            }
            return Ok(e);
        }
        let t = self.t.get(self.i).cloned();
        self.i += 1;
        match t {
            Some(Tok::Num(n)) => Ok(Expr::Lit(Value::from(n))),
            Some(Tok::Str(s)) => Ok(Expr::Lit(Value::String(s))),
            Some(Tok::Ident(w)) if w.starts_with("format") && self.eat("(") => {
                let format = match w.as_str() {
                    "formatNumber" => Format::Grouped(None),
                    "formatCompact" => Format::Compact(None),
                    "formatDuration" => Format::Duration,
                    "formatPercent" => Format::Percent(None),
                    "formatOrdinal" => Format::Ordinal,
                    "formatCountdown" => Format::Countdown,
                    _ => {
                        return Err(format!(
                            "unknown formatter '{w}'; use formatNumber, formatCompact, formatDuration, formatPercent, formatOrdinal or formatCountdown"
                        ));
                    }
                };
                let value = self.expr()?;
                if !self.eat(")") {
                    return Err(format!(
                        "close {w}(expression) with ')'; formatters take one value"
                    ));
                }
                Ok(Expr::Format(format, Box::new(value)))
            }
            Some(Tok::Ident(w)) if w == "t" && self.eat("(") => {
                let key = self.expr()?;
                let mut args = Vec::new();
                if self.eat(",") {
                    if !self.eat("{") {
                        return Err("t(key, { name: expression }) takes named parameters".into());
                    }
                    while !self.eat("}") {
                        let name = match self.t.get(self.i).cloned() {
                            Some(Tok::Ident(n) | Tok::Str(n))
                                if !n.is_empty()
                                    && n.chars().all(|c| c.is_alphanumeric() || c == '_') =>
                            {
                                n
                            }
                            _ => return Err("a translation parameter needs a name".into()),
                        };
                        self.i += 1;
                        let value = if self.eat(":") {
                            self.expr()?
                        } else {
                            Expr::Path(name.clone())
                        };
                        if args.iter().any(|(n, _)| n == &name) {
                            return Err(format!(
                                "translation parameter '{name}' is repeated; keep one value"
                            ));
                        }
                        args.push((name, value));
                        if self.eat("}") {
                            break;
                        }
                        if !self.eat(",") {
                            return Err(
                                "separate translation parameters with ',' and close '}'".into()
                            );
                        }
                    }
                }
                if !self.eat(")") {
                    return Err("close t(key, { name: expression }) with ')'".into());
                }
                Ok(Expr::Translate(Box::new(key), args))
            }
            Some(Tok::Ident(w)) => Ok(match w.as_str() {
                "true" => Expr::Lit(Value::Bool(true)),
                "false" => Expr::Lit(Value::Bool(false)),
                "null" => Expr::Lit(Value::Null),
                _ => Expr::Path(w),
            }),
            Some(Tok::Sym(s)) => Err(format!("'{s}' where a value was expected")),
            None => Err("the binding ends where a value was expected".into()),
        }
    }
}

impl Expr {
    /// Parses a binding.
    pub fn parse(src: &str) -> Result<Expr, String> {
        let mut p = P { t: lex(src)?, i: 0 };
        let e = p.expr()?;
        if p.i < p.t.len() {
            return Err(format!(
                "'{src}' has more after its end (a missing operator?)"
            ));
        }
        Ok(e)
    }

    /// Every path it reads, in order (a document's static structure).
    pub fn paths(&self, out: &mut Vec<String>) {
        match self {
            Expr::Lit(_) | Expr::Interaction(_) => {}
            Expr::Path(p) => out.push(p.clone()),
            Expr::Translate(key, args) => {
                key.paths(out);
                for (_, value) in args {
                    value.paths(out);
                }
            }
            Expr::Not(a) | Expr::Neg(a) | Expr::Format(_, a) => a.paths(out),
            Expr::Bin(_, a, b) => {
                a.paths(out);
                b.paths(out);
            }
            Expr::If(c, a, b) => {
                c.paths(out);
                a.paths(out);
                b.paths(out);
            }
        }
    }

    /// Its value against a scope ([`Scope`]).
    pub fn eval(&self, scope: &Scope<'_>) -> Value {
        match self {
            Expr::Lit(v) => v.clone(),
            Expr::Path(p) => scope.get(p),
            Expr::Interaction(state) => Value::Bool(scope.env.interaction(*state)),
            Expr::Translate(key, params) => {
                let key = text(&key.eval(scope));
                let message = scope.env.translate(&key).unwrap_or_else(|| key.clone());
                let Ok(message) = og_i18n::Message::parse(&message) else {
                    return Value::String(key);
                };
                let args = params
                    .iter()
                    .map(|(name, expr)| {
                        let value = expr.eval(scope);
                        let arg = match value {
                            Value::Number(n) => og_i18n::Arg::Num(n.as_f64().unwrap_or(0.0)),
                            other => og_i18n::Arg::Str(text(&other)),
                        };
                        (name.clone(), arg)
                    })
                    .collect();
                Value::String(message.format(scope.env.language(), &args))
            }
            Expr::Format(format, value) => Value::String(format_value(
                &value.eval(scope),
                *format,
                scope.env.language(),
            )),
            Expr::Not(a) => Value::Bool(!truthy(&a.eval(scope))),
            Expr::Neg(a) => num(-number(&a.eval(scope)).unwrap_or(f64::NAN)),
            Expr::If(c, a, b) => {
                if truthy(&c.eval(scope)) {
                    a.eval(scope)
                } else {
                    b.eval(scope)
                }
            }
            Expr::Bin(op, a, b) => {
                let x = a.eval(scope);
                match op {
                    Op::And => {
                        return if truthy(&x) { b.eval(scope) } else { x };
                    }
                    Op::Or => {
                        return if truthy(&x) { x } else { b.eval(scope) };
                    }
                    _ => {}
                }
                let y = b.eval(scope);
                let (nx, ny) = (number(&x), number(&y));
                match op {
                    Op::Eq => Value::Bool(loose_eq(&x, &y)),
                    Op::Ne => Value::Bool(!loose_eq(&x, &y)),
                    Op::Add if x.is_string() || y.is_string() => {
                        Value::String(format!("{}{}", text(&x), text(&y)))
                    }
                    _ => {
                        let (Some(p), Some(q)) = (nx, ny) else {
                            return match op {
                                Op::Lt | Op::Le | Op::Gt | Op::Ge => Value::Bool(false),
                                _ => Value::Null,
                            };
                        };
                        match op {
                            Op::Mul => num(p * q),
                            Op::Div => num(p / q),
                            Op::Rem => num(p % q),
                            Op::Add => num(p + q),
                            Op::Sub => num(p - q),
                            Op::Lt => Value::Bool(p < q),
                            Op::Le => Value::Bool(p <= q),
                            Op::Gt => Value::Bool(p > q),
                            Op::Ge => Value::Bool(p >= q),
                            _ => unreachable!("handled above"),
                        }
                    }
                }
            }
        }
    }
}

fn num(x: f64) -> Value {
    serde_json::Number::from_f64(x).map_or(Value::Null, Value::Number)
}

fn loose_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Null, Value::Null) => true,
        (Value::Null, _) | (_, Value::Null) => false,
        _ => match (number(a), number(b)) {
            (Some(x), Some(y)) => x == y,
            _ => a == b,
        },
    }
}

/// What a binding reads: the item variables of the list templates it is
/// in (innermost first), then the viewmodel, then the host's roots.
pub struct Scope<'a> {
    pub vm: &'a Value,
    /// `(name, value)` of the enclosing `for` loops, innermost last.
    pub locals: Vec<(&'a str, Value)>,
    pub env: &'a dyn Env,
}

impl Scope<'_> {
    /// The value at `path` (null when nothing has it).
    pub fn get(&self, path: &str) -> Value {
        let (root, rest) = path.split_once('.').unwrap_or((path, ""));
        for (name, v) in self.locals.iter().rev() {
            if *name == root {
                return get_path(v, rest).map_or(Value::Null, |c| c.into_owned());
            }
        }
        if let Some(v) = get_path(self.vm, path) {
            return v.into_owned();
        }
        if self.vm.get(root).is_none()
            && let Some(r) = self.env.lookup(root)
        {
            return get_path(&r, rest).map_or(Value::Null, |c| c.into_owned());
        }
        Value::Null
    }
}

// ── numbers past f64 ──────────────────────────────────────────────────────

/// A number as a mantissa and a power of ten: `m × 10^e`, `1 ≤ |m| < 10`
/// (0 is `m = 0`). Idle games count past 1e308 with it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Num {
    pub m: f64,
    pub e: i64,
}

impl Num {
    pub const ZERO: Num = Num { m: 0.0, e: 0 };

    pub fn from_f64(v: f64) -> Num {
        if v == 0.0 || !v.is_finite() {
            return Num { m: v, e: 0 };
        }
        let s = format!("{v:e}");
        let (m, e) = s.split_once('e').unwrap_or((&s, "0"));
        Num {
            m: m.parse().unwrap_or(v),
            e: e.parse().unwrap_or(0),
        }
    }

    /// From a viewmodel value: a number, a numeric string (`"1.5e400"`
    /// too) or `{m, e}` / `{mantissa, exponent}`.
    pub fn from_json(v: &Value) -> Option<Num> {
        match v {
            Value::Number(n) => n.as_f64().map(Num::from_f64),
            Value::String(s) => Num::parse(s),
            Value::Object(o) => {
                let m = o.get("m").or_else(|| o.get("mantissa"))?.as_f64()?;
                let e = o.get("e").or_else(|| o.get("exponent"))?.as_f64()?;
                Some(Num::normal(m, e as i64))
            }
            Value::Bool(b) => Some(Num::from_f64(f64::from(u8::from(*b)))),
            _ => None,
        }
    }

    /// `"1.5e400"`, `"-2E10"`, `"12345"`.
    pub fn parse(s: &str) -> Option<Num> {
        let s = s.trim();
        let (m, e) = match s.find(['e', 'E']) {
            Some(i) => (
                &s[..i],
                s[i + 1..].trim_start_matches('+').parse::<i64>().ok()?,
            ),
            None => (s, 0),
        };
        let m: f64 = m.parse().ok()?;
        m.is_finite().then(|| Num::normal(m, e))
    }

    /// `m × 10^e` brought to `1 ≤ |m| < 10`.
    pub fn normal(m: f64, e: i64) -> Num {
        if m == 0.0 || !m.is_finite() {
            return Num { m, e: 0 };
        }
        let n = Num::from_f64(m);
        Num { m: n.m, e: n.e + e }
    }

    /// As f64 (infinite past its range).
    pub fn to_f64(self) -> f64 {
        if self.e > 308 {
            return self.m.signum() * f64::INFINITY;
        }
        if self.e < -330 {
            return 0.0;
        }
        format!("{}e{}", self.m, self.e).parse().unwrap_or(0.0)
    }

    /// `sig` significant digits of |self| as digit characters, and the
    /// power of ten of the first (rounding may carry into a new digit).
    fn digits(self, sig: usize) -> (String, i64) {
        let sig = sig.clamp(1, 17);
        let s = format!("{:.*e}", sig - 1, self.m.abs());
        let (m, e) = s.split_once('e').unwrap_or((&s, "0"));
        let d: String = m.chars().filter(char::is_ascii_digit).collect();
        (d, self.e + e.parse::<i64>().unwrap_or(0))
    }
}

// ── formats ────────────────────────────────────────────────────────────────

/// How a value is shown (see the module docs). The digits are significant
/// digits for the notations, decimals for `grouped`, `fixed` and
/// `percent`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// Rust's shortest text of the value (no grouping).
    Plain,
    /// Thousands grouped by the language, `digits` decimals (default 0).
    Grouped(Option<u8>),
    /// Always `digits` decimals, grouped.
    Fixed(u8),
    /// 1.2K, 3.45M (significant digits, default 3).
    Compact(Option<u8>),
    /// 12.3e6: an exponent that is a multiple of three.
    Engineering(Option<u8>),
    /// 1.23e7.
    Scientific(Option<u8>),
    /// Idle notation: K M B T, then aa, ab … zz, aaa.
    Letters(Option<u8>),
    /// A fraction as a percentage (0.45 → 45%), `digits` decimals.
    Percent(Option<u8>),
    /// Seconds as two units of words: 1h 2m, 2m 5s, 45s.
    Duration,
    /// Seconds as a clock: 1:02:03, 2:05.
    Clock,
    /// A clock that shows tenths under ten seconds (9.4).
    Countdown,
    /// 1st, 2nd (og-i18n).
    Ordinal,
}

impl Format {
    /// Every format's name, as documents and templates write them.
    pub const NAMES: [&'static str; 12] = [
        "plain",
        "grouped",
        "fixed",
        "compact",
        "engineering",
        "scientific",
        "letters",
        "percent",
        "duration",
        "clock",
        "countdown",
        "ordinal",
    ];

    /// A format by its name and an optional digit count (`compact`,
    /// `scientific` with `4`).
    pub fn new(name: &str, digits: Option<u8>) -> Result<Format, String> {
        Ok(match name {
            "plain" => Format::Plain,
            "grouped" => Format::Grouped(digits),
            "fixed" => Format::Fixed(digits.unwrap_or(2)),
            "compact" => Format::Compact(digits),
            "engineering" => Format::Engineering(digits),
            "scientific" => Format::Scientific(digits),
            "letters" => Format::Letters(digits),
            "percent" => Format::Percent(digits),
            "duration" => Format::Duration,
            "clock" => Format::Clock,
            "countdown" => Format::Countdown,
            "ordinal" => Format::Ordinal,
            other => {
                return Err(format!(
                    "'{other}' is not a format: {}",
                    Format::NAMES.join(", ")
                ));
            }
        })
    }

    /// `compact`, `compact:2`, `scientific 4`.
    pub fn parse(spec: &str) -> Result<Format, String> {
        let spec = spec.trim();
        let (name, d) = match spec.split_once([':', ' ', ',']) {
            Some((n, d)) => (
                n.trim(),
                Some(
                    d.trim()
                        .parse::<u8>()
                        .map_err(|_| format!("'{d}' is not a digit count (0 to 17)"))?,
                ),
            ),
            None => (spec, None),
        };
        Format::new(name, d)
    }

    /// Its text form (what [`Format::parse`] reads back).
    pub fn to_text(self) -> String {
        let (n, d) = match self {
            Format::Plain => ("plain", None),
            Format::Grouped(d) => ("grouped", d),
            Format::Fixed(d) => ("fixed", Some(d)),
            Format::Compact(d) => ("compact", d),
            Format::Engineering(d) => ("engineering", d),
            Format::Scientific(d) => ("scientific", d),
            Format::Letters(d) => ("letters", d),
            Format::Percent(d) => ("percent", d),
            Format::Duration => ("duration", None),
            Format::Clock => ("clock", None),
            Format::Countdown => ("countdown", None),
            Format::Ordinal => ("ordinal", None),
        };
        match d {
            Some(d) => format!("{n}:{d}"),
            None => n.into(),
        }
    }
}

fn separators(lang: &str) -> (char, char) {
    match lang {
        "de" | "es" | "it" | "pt" | "nl" | "id" | "tr" | "da" | "el" => ('.', ','),
        "fr" | "ru" | "pl" | "uk" | "cs" | "sk" | "sv" | "fi" | "nb" | "no" | "bg" => {
            ('\u{202f}', ',')
        }
        _ => (',', '.'),
    }
}

fn group_digits(int: &str, sep: char) -> String {
    let mut out = String::new();
    let n = int.len();
    for (k, c) in int.chars().enumerate() {
        if k > 0 && (n - k).is_multiple_of(3) {
            out.push(sep);
        }
        out.push(c);
    }
    out
}

/// `digits` placed with `int` digits before the decimal point (zeros
/// padded), trailing zeros of the fraction dropped unless `keep`.
fn place(digits: &str, int: usize, dec: char, keep: bool) -> String {
    let mut d = digits.to_string();
    while d.len() < int {
        d.push('0');
    }
    let (a, b) = d.split_at(int);
    let b = if keep { b } else { b.trim_end_matches('0') };
    if b.is_empty() {
        a.to_string()
    } else {
        format!("{a}{dec}{b}")
    }
}

/// A number with `dp` decimals, grouped (f64 range; past it, scientific).
fn grouped(n: Num, dp: usize, lang: &str, keep: bool) -> String {
    let v = n.to_f64();
    if !v.is_finite() || n.e >= 21 {
        return scientific(n, 4, lang);
    }
    let (th, dec) = separators(lang);
    let s = format!("{:.*}", dp.min(17), v.abs());
    let (int, frac) = s.split_once('.').unwrap_or((&s, ""));
    let frac = if keep {
        frac
    } else {
        frac.trim_end_matches('0')
    };
    let mut out = group_digits(int, th);
    if !frac.is_empty() {
        out.push(dec);
        out.push_str(frac);
    }
    out
}

fn scientific(n: Num, sig: usize, lang: &str) -> String {
    if n.m == 0.0 {
        return "0".into();
    }
    let (_, dec) = separators(lang);
    let (d, e) = n.digits(sig);
    format!("{}e{e}", place(&d, 1, dec, false))
}

/// The tier scaled into `1..base^group` and its index: `group` digits per tier.
fn tiered(n: Num, sig: usize, group: i64, dec: char) -> (String, i64) {
    let (d, e) = n.digits(sig);
    let tier = e.div_euclid(group);
    let int = (e.rem_euclid(group) + 1) as usize;
    (place(&d, int, dec, false), tier)
}

/// The letters of idle notation's tier `t` past T (0 → aa).
fn letters_of(t: i64) -> String {
    let mut k = t;
    let mut width = 2;
    let mut span = 26i64 * 26;
    while k >= span && width < 8 {
        k -= span;
        width += 1;
        span *= 26;
    }
    let mut out = vec!['a'; width];
    for slot in out.iter_mut().rev() {
        *slot = (b'a' + (k % 26) as u8) as char;
        k /= 26;
    }
    out.into_iter().collect()
}

/// Formats a value: a number (or a [`Num`]-shaped value) by `format` in
/// `lang`; a value that is not a number shows as its text.
pub fn format_value(v: &Value, format: Format, lang: &str) -> String {
    match Num::from_json(v) {
        Some(n) => format_num(n, format, lang),
        None => text(v),
    }
}

/// Formats a [`Num`].
pub fn format_num(n: Num, format: Format, lang: &str) -> String {
    if !n.m.is_finite() {
        return if n.m.is_nan() {
            "–".into()
        } else if n.m > 0.0 {
            "∞".into()
        } else {
            "-∞".into()
        };
    }
    let neg = n.m < 0.0;
    let a = Num {
        m: n.m.abs(),
        e: n.e,
    };
    let (_, dec) = separators(lang);
    let sig = |d: Option<u8>| usize::from(d.unwrap_or(3)).max(1);
    let body = match format {
        Format::Plain => {
            let v = a.to_f64();
            if v.is_finite() {
                text(&num(v))
            } else {
                scientific(a, 6, lang)
            }
        }
        Format::Grouped(d) => grouped(a, usize::from(d.unwrap_or(0)), lang, false),
        Format::Fixed(d) => grouped(a, usize::from(d), lang, true),
        Format::Compact(d) => {
            let cjk = match lang {
                "ja" => Some(&["", "万", "億", "兆", "京"][..]),
                "zh" => Some(&["", "万", "亿", "万亿"][..]),
                "ko" => Some(&["", "만", "억", "조", "경"][..]),
                _ => None,
            };
            let (group, units): (i64, &[&str]) = match cjk {
                Some(u) => (4, u),
                None => (
                    3,
                    &[
                        "", "K", "M", "B", "T", "Qa", "Qi", "Sx", "Sp", "Oc", "No", "Dc",
                    ],
                ),
            };
            if a.m == 0.0 {
                "0".into()
            } else if a.e < group {
                grouped(
                    a,
                    sig(d).saturating_sub(1 + a.e.max(0) as usize).min(2),
                    lang,
                    false,
                )
            } else {
                let (t, tier) = tiered(a, sig(d), group, dec);
                match units.get(tier as usize) {
                    Some(u) => format!("{t}{u}"),
                    None => scientific(a, sig(d), lang),
                }
            }
        }
        Format::Engineering(d) => {
            if a.m == 0.0 {
                "0".into()
            } else {
                let (t, tier) = tiered(a, sig(d), 3, dec);
                if tier == 0 {
                    t
                } else {
                    format!("{t}e{}", tier * 3)
                }
            }
        }
        Format::Scientific(d) => scientific(a, sig(d), lang),
        Format::Letters(d) => {
            if a.m == 0.0 {
                "0".into()
            } else if a.e < 3 {
                grouped(a, 0, lang, false)
            } else {
                let (t, tier) = tiered(a, sig(d), 3, dec);
                let u = match tier {
                    1 => "K".to_string(),
                    2 => "M".to_string(),
                    3 => "B".to_string(),
                    4 => "T".to_string(),
                    t => letters_of(t - 5),
                };
                format!("{t}{u}")
            }
        }
        Format::Percent(d) => {
            let p = grouped(
                Num { m: a.m, e: a.e + 2 },
                usize::from(d.unwrap_or(0)),
                lang,
                true,
            );
            match lang {
                "fr" | "de" | "sv" | "nb" | "no" | "fi" | "cs" | "sk" | "ru" | "uk" | "pl" => {
                    format!("{p}\u{a0}%")
                }
                "tr" => format!("%{p}"),
                _ => format!("{p}%"),
            }
        }
        Format::Duration | Format::Clock | Format::Countdown => {
            let secs = a.to_f64();
            if !secs.is_finite() {
                return "∞".into();
            }
            match format {
                Format::Countdown if secs < 10.0 => {
                    // Tenths, rounded down: a countdown never shows time it does not have.
                    let t = (secs * 10.0).floor() / 10.0;
                    let s = format!("{t:.1}");
                    s.replace('.', &dec.to_string())
                }
                Format::Clock | Format::Countdown => {
                    let s = if format == Format::Countdown {
                        secs.ceil() as u64
                    } else {
                        secs.floor() as u64
                    };
                    let (h, m, s) = (s / 3600, s / 60 % 60, s % 60);
                    if h > 0 {
                        format!("{h}:{m:02}:{s:02}")
                    } else {
                        format!("{m}:{s:02}")
                    }
                }
                _ => {
                    let s = secs.floor() as u64;
                    let units: [(u64, &str); 4] = match lang {
                        "ja" => [(86400, "日"), (3600, "時間"), (60, "分"), (1, "秒")],
                        "zh" => [(86400, "天"), (3600, "小时"), (60, "分"), (1, "秒")],
                        "ko" => [(86400, "일 "), (3600, "시간 "), (60, "분 "), (1, "초 ")],
                        _ => [(86400, "d "), (3600, "h "), (60, "m "), (1, "s ")],
                    };
                    // The first unit it has, and the next when it is not zero.
                    let first = units.iter().position(|(size, _)| s >= *size).unwrap_or(3);
                    let (size, u) = units[first];
                    let mut parts = vec![format!("{}{u}", s / size)];
                    if first < 3 {
                        let (next, nu) = units[first + 1];
                        let q = s % size / next;
                        if q > 0 {
                            parts.push(format!("{q}{nu}"));
                        }
                    }
                    parts.concat().trim_end().to_string()
                }
            }
        }
        // Past what a place can be, the number itself.
        Format::Ordinal if a.to_f64() >= 1e15 => scientific(a, 3, lang),
        Format::Ordinal => og_i18n::format_ordinal(a.to_f64(), lang),
    };
    if neg && body != "0" {
        format!("-{body}")
    } else {
        body
    }
}

// ── templates ──────────────────────────────────────────────────────────────

/// A text with arguments (see the module docs): parsed once, formatted
/// against a scope every tick.
#[derive(Clone, Debug, PartialEq)]
pub struct Template {
    message: og_i18n::Message,
    /// The engine-format arguments, replaced in the message by `__f<k>`.
    formats: Vec<(String, Format)>,
    /// The message's other arguments: paths.
    args: Vec<String>,
}

impl Template {
    /// Parses `src`. A text with no `{` is a template of one literal.
    pub fn parse(src: &str) -> Result<Template, String> {
        let mut out = String::new();
        let mut formats = Vec::new();
        let chars: Vec<char> = src.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c == '\'' {
                // Quoting is the message's: copy the quoted run untouched.
                out.push(c);
                i += 1;
                if i < chars.len() && matches!(chars[i], '{' | '}' | '#' | '|') {
                    while i < chars.len() {
                        out.push(chars[i]);
                        i += 1;
                        if chars[i - 1] == '\'' {
                            break;
                        }
                    }
                }
                continue;
            }
            if c == '{' {
                let end = chars[i + 1..].iter().position(|&x| x == '{' || x == '}');
                if let Some(k) = end
                    && chars[i + 1 + k] == '}'
                {
                    let inner: String = chars[i + 1..i + 1 + k].iter().collect();
                    let mut parts = inner.split(',').map(str::trim);
                    if let (Some(name), Some(kind)) = (parts.next(), parts.next())
                        && Format::NAMES.contains(&kind)
                    {
                        let digits = match parts.next() {
                            Some(d) => Some(d.parse::<u8>().map_err(|_| {
                                format!("in {{{inner}}}: '{d}' is not a digit count")
                            })?),
                            None => None,
                        };
                        formats.push((name.to_string(), Format::new(kind, digits)?));
                        out.push_str(&format!("{{__f{}}}", formats.len() - 1));
                        i += k + 2;
                        continue;
                    }
                }
            }
            out.push(c);
            i += 1;
        }
        let message = og_i18n::Message::parse(&out).map_err(|e| format!("'{src}': {e}"))?;
        let args = message
            .arguments()
            .into_iter()
            .filter(|a| !a.starts_with("__f"))
            .collect();
        Ok(Template {
            message,
            formats,
            args,
        })
    }

    /// Every path it reads.
    pub fn paths(&self, out: &mut Vec<String>) {
        out.extend(self.formats.iter().map(|(p, _)| p.clone()));
        out.extend(self.args.iter().cloned());
    }

    /// Its text in the scope.
    pub fn format(&self, scope: &Scope<'_>) -> String {
        let lang = scope.env.language();
        let mut args = BTreeMap::new();
        for a in &self.args {
            let v = scope.get(a);
            let arg = match &v {
                Value::Number(n) => og_i18n::Arg::Num(n.as_f64().unwrap_or(0.0)),
                Value::Bool(b) => og_i18n::Arg::Str(b.to_string()),
                Value::Null => continue,
                other => og_i18n::Arg::Str(text(other)),
            };
            args.insert(a.clone(), arg);
        }
        for (k, (p, f)) in self.formats.iter().enumerate() {
            let v = scope.get(p);
            args.insert(
                format!("__f{k}"),
                og_i18n::Arg::Str(format_value(&v, *f, lang)),
            );
        }
        self.message.format(lang, &args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn f(v: Value, spec: &str, lang: &str) -> String {
        format_value(&v, Format::parse(spec).unwrap(), lang)
    }

    #[test]
    fn compact_rounds_and_carries_into_the_next_tier() {
        assert_eq!(f(json!(999), "compact", "en"), "999");
        assert_eq!(f(json!(1234), "compact", "en"), "1.23K");
        assert_eq!(f(json!(12_345), "compact", "en"), "12.3K");
        assert_eq!(
            f(json!(999_999), "compact", "en"),
            "1M",
            "999.999K rounds into M"
        );
        assert_eq!(f(json!(1_500_000), "compact:2", "en"), "1.5M");
        assert_eq!(f(json!(-2_000_000_000.0), "compact", "en"), "-2B");
        assert_eq!(f(json!(12_345), "compact", "de"), "12,3K");
        // CJK counts in myriads.
        assert_eq!(f(json!(123_456), "compact", "ja"), "12.3万");
        assert_eq!(f(json!(250_000_000), "compact", "zh"), "2.5亿");
    }

    #[test]
    fn numbers_past_f64_keep_their_mantissa_and_exponent() {
        assert_eq!(
            f(json!({"m": 1.5, "e": 400}), "scientific", "en"),
            "1.5e400"
        );
        assert_eq!(f(json!("2.5e400"), "engineering", "en"), "25e399");
        // 1e15 is the first two-letter tier: aa; 1e18 ab; 1e93 az + 1 = ba.
        assert_eq!(f(json!(1e15), "letters", "en"), "1aa");
        assert_eq!(f(json!(4.2e18), "letters", "en"), "4.2ab");
        assert_eq!(f(json!(1e93), "letters", "en"), "1ba");
        assert_eq!(f(json!({"m": 3, "e": 2043}), "letters", "en"), "3aaa");
        assert_eq!(f(json!(1.234e7), "scientific", "en"), "1.23e7");
        assert_eq!(f(json!(9.9996e7), "scientific", "en"), "1e8", "carry");
        assert_eq!(f(json!({"m": 1.5, "e": 400}), "grouped", "en"), "1.5e400");
    }

    #[test]
    fn grouping_percent_and_time() {
        assert_eq!(f(json!(1234567.891), "grouped", "en"), "1,234,568");
        assert_eq!(f(json!(1234567.891), "fixed:2", "de"), "1.234.567,89");
        assert_eq!(f(json!(0.456), "percent", "en"), "46%");
        assert_eq!(f(json!(0.456), "percent:1", "fr"), "45,6\u{a0}%");
        assert_eq!(f(json!(3723), "clock", "en"), "1:02:03");
        assert_eq!(f(json!(125.9), "clock", "en"), "2:05");
        assert_eq!(f(json!(9.47), "countdown", "en"), "9.4");
        assert_eq!(f(json!(61.2), "countdown", "en"), "1:02");
        assert_eq!(f(json!(3723), "duration", "en"), "1h 2m");
        assert_eq!(f(json!(125), "duration", "en"), "2m 5s");
        assert_eq!(f(json!(3600), "duration", "en"), "1h");
        assert_eq!(f(json!(45), "duration", "ja"), "45秒");
        assert_eq!(f(json!(22), "ordinal", "en"), "22nd");
    }

    #[test]
    fn expressions_read_locals_then_the_viewmodel_then_the_host() {
        struct Host;
        impl Env for Host {
            fn lookup(&self, root: &str) -> Option<Value> {
                (root == "Wallet").then(|| json!({ "gold": 40 }))
            }
        }
        let vm = json!({ "price": 50, "items": [{ "owned": false }], "hero": { "health": 30, "maxHealth": 120 } });
        let item = json!({ "owned": false, "name": "Sword" });
        let scope = Scope {
            vm: &vm,
            locals: vec![("item", item)],
            env: &Host,
        };
        let e = Expr::parse("!item.owned && price > Wallet.gold").unwrap();
        assert_eq!(e.eval(&scope), json!(true));
        assert_eq!(
            Expr::parse("hero.health / hero.maxHealth")
                .unwrap()
                .eval(&scope),
            json!(0.25)
        );
        assert_eq!(Expr::parse("items.length").unwrap().eval(&scope), json!(1));
        assert_eq!(
            Expr::parse("price >= 50 ? 'dear' : 'cheap'")
                .unwrap()
                .eval(&scope),
            json!("dear")
        );
        assert_eq!(
            Expr::parse("'x' + item.name").unwrap().eval(&scope),
            json!("xSword")
        );
        assert!(Expr::parse("price >").is_err());
        assert!(Expr::parse("price $$ 2").is_err());
    }

    #[test]
    fn pure_formatters_read_values_and_the_current_locale() {
        struct Host(&'static str);
        impl Env for Host {
            fn language(&self) -> &str {
                self.0
            }
            fn lookup(&self, root: &str) -> Option<Value> {
                (root == "$seconds").then(|| json!(3723))
            }
        }
        let vm = json!({"amount":12345,"fraction":0.456});
        let scope = Scope {
            vm: &vm,
            locals: vec![],
            env: &Host("en"),
        };
        for (source, expected) in [
            ("formatNumber(amount)", "12,345"),
            ("formatCompact(amount)", "12.3K"),
            ("formatPercent(fraction)", "46%"),
            ("formatDuration($seconds)", "1h 2m"),
        ] {
            assert_eq!(Expr::parse(source).unwrap().eval(&scope), json!(expected));
        }
        let expr = Expr::parse("formatNumber(amount + 2)").unwrap();
        let mut paths = Vec::new();
        expr.paths(&mut paths);
        assert_eq!(paths, ["amount"]);
        assert_eq!(
            expr.eval(&Scope {
                env: &Host("de"),
                ..scope
            }),
            json!("12.347")
        );
        for bad in [
            "formatNumber(amount++)",
            "formatDuration(set(1))",
            "formatNumber()",
            "formatNumber(1, 2)",
            "formatAnything(1)",
        ] {
            assert!(Expr::parse(bad).is_err(), "accepted {bad}");
        }
    }

    #[test]
    fn translations_use_named_expressions_and_locale_plurals() {
        struct Host;
        impl Env for Host {
            fn language(&self) -> &str {
                "de"
            }
            fn translate(&self, key: &str) -> Option<String> {
                (key == "taps").then(|| "{who}: {count, plural, one {# Schlag} other {# Schläge}} / {offset, number} ms".into())
            }
        }
        let expr = Expr::parse("t('taps', { who: item.name, count: taps + 1, offset })").unwrap();
        for (taps, expected) in [
            (0, "Ada: 1 Schlag / 1.234,5 ms"),
            (2, "Ada: 3 Schläge / 1.234,5 ms"),
        ] {
            let vm = json!({"taps": taps, "offset": 1234.5});
            let scope = Scope {
                vm: &vm,
                locals: vec![("item", json!({"name":"Ada"}))],
                env: &Host,
            };
            assert_eq!(expr.eval(&scope), json!(expected));
        }
        let mut paths = Vec::new();
        expr.paths(&mut paths);
        assert_eq!(paths, ["item.name", "taps", "offset"]);
        for bad in [
            "t('x', { n: taps++ })",
            "t('x', { n: set(1) })",
            "t('x', { n: 1, n: 2 })",
            "t('x', { n: { nested: 1 } })",
            "{ n: 1 }",
            "t('x', 'fallback')",
            "t('x', { n: 1 }",
            "t('x', { n: 1 n: 2 })",
        ] {
            assert!(Expr::parse(bad).is_err(), "accepted {bad}");
        }
    }

    #[test]
    fn interaction_inputs_are_internal_and_node_scoped() {
        struct NodeEnv(InteractionState);
        impl Env for NodeEnv {
            fn interaction(&self, state: InteractionState) -> bool {
                state == self.0
            }
        }
        let vm = Value::Null;
        let env = NodeEnv(InteractionState::FocusVisible);
        let mut scope = Scope {
            vm: &vm,
            locals: vec![],
            env: &env,
        };
        assert_eq!(
            Expr::Interaction(InteractionState::FocusVisible).eval(&scope),
            Value::Bool(true)
        );
        assert_eq!(
            Expr::Interaction(InteractionState::Hover).eval(&scope),
            Value::Bool(false)
        );
        scope.env = &NoEnv;
        assert_eq!(
            Expr::Interaction(InteractionState::FocusVisible).eval(&scope),
            Value::Bool(false)
        );
        assert!(matches!(Expr::parse("hover").unwrap(), Expr::Path(_)));
        assert!(Expr::parse("interaction('hover')").is_err());
    }

    #[test]
    fn templates_mix_engine_formats_with_plurals() {
        let vm = json!({ "gold": 12345, "n": 1, "rank": 3, "name": "Ada" });
        let scope = Scope {
            vm: &vm,
            locals: vec![],
            env: &NoEnv,
        };
        let t = Template::parse("{name}: {gold, compact} gold, {n, plural, one {# gem} other {# gems}}, {rank, ordinal}").unwrap();
        assert_eq!(t.format(&scope), "Ada: 12.3K gold, 1 gem, 3rd");
        let mut paths = Vec::new();
        t.paths(&mut paths);
        paths.sort();
        assert_eq!(paths, ["gold", "n", "name", "rank"]);
        assert!(Template::parse("{gold, compact, x}").is_err());
    }

    #[test]
    fn set_path_writes_through_objects_and_lists() {
        let mut vm = json!({ "settings": { "volume": 0.5 }, "items": [1, 2] });
        assert!(set_path(&mut vm, "settings.volume", json!(0.8)));
        assert!(set_path(&mut vm, "items.1", json!(5)));
        assert!(set_path(&mut vm, "new.deep", json!(true)));
        assert!(!set_path(&mut vm, "items.9", json!(0)));
        assert_eq!(
            vm,
            json!({ "settings": { "volume": 0.8 }, "items": [1, 5], "new": { "deep": true } })
        );
    }
}
