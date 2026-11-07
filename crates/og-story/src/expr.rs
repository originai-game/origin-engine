//! The expressions a dialogue or quest writes in quotes: conditions
//! (`if "gold >= 10 and not seen(intro)"`) and values (`set gold "gold - 10"`).
//!
//! Numbers (`f64`), booleans and strings (`'Kai'`); variables by name;
//! `+ - * / %` on numbers (`+` also joins two strings), comparisons
//! (`== != < <= > >=`), `and`, `or`, `not` and parentheses; and the story's
//! functions: `seen(node)` (a node shown before; `other.node` names a node
//! of the dialogue `other`), `status(quest)` (`'none'`, `'active'`,
//! `'completed'` or `'failed'`), `stage(quest)` (its stage id, `''` before
//! it starts) and `count(quest, objective)`. Arithmetic is IEEE `f64`
//! addition, subtraction, multiplication, division and remainder only, so
//! every host computes the same bits.

use std::fmt;

/// A story value.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Val {
    Num(f64),
    Bool(bool),
    Str(String),
}

impl Val {
    /// Whether it counts as true: `true`, a number other than 0, a string
    /// that is not empty.
    pub fn truthy(&self) -> bool {
        match self {
            Val::Num(n) => *n != 0.0,
            Val::Bool(b) => *b,
            Val::Str(s) => !s.is_empty(),
        }
    }

    /// Its kind's name, for errors.
    pub fn kind(&self) -> &'static str {
        match self {
            Val::Num(_) => "a number",
            Val::Bool(_) => "a boolean",
            Val::Str(_) => "a string",
        }
    }
}

impl fmt::Display for Val {
    /// Numbers that are whole print without a fraction (`3`, not `3.0`).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Val::Num(n) if n.fract() == 0.0 && n.abs() < 1e15 => write!(f, "{}", *n as i64),
            Val::Num(n) => write!(f, "{n}"),
            Val::Bool(b) => write!(f, "{b}"),
            Val::Str(s) => f.write_str(s),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Xor,
}

/// A story function.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Func {
    /// `seen(node)`.
    Seen(String),
    /// `status(quest)`.
    Status(String),
    /// `stage(quest)`.
    Stage(String),
    /// `count(quest, objective)`.
    Count(String, String),
}

/// A parsed expression.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Expr {
    Lit(Val),
    Var(String),
    /// Whether the stable identity has already consumed its Yarn once content.
    Once(String),
    Not(Box<Expr>),
    Neg(Box<Expr>),
    Bin(Op, Box<Expr>, Box<Expr>),
    Call(Func),
    Invoke(String, Vec<Expr>),
    Member(String, String),
    EnumValue(String, Val),
    /// Yarn uses single-precision numbers and eager boolean operands.
    Yarn(Box<Expr>),
}

/// What an expression reads while it is evaluated.
pub trait Scope {
    fn once(&self, _id: &str) -> bool {
        false
    }
    /// A variable's value; `None` for one the story does not have.
    fn var(&self, name: &str) -> Option<Val>;
    /// `seen(node)`: the node, as written.
    fn seen(&self, node: &str) -> bool;
    /// `status(quest)`.
    fn quest_status(&self, quest: &str) -> &'static str;
    /// `stage(quest)`.
    fn quest_stage(&self, quest: &str) -> String;
    /// `count(quest, objective)`.
    fn quest_count(&self, quest: &str, objective: &str) -> f64;
    fn invoke(&self, name: &str, args: &[Val]) -> Result<Val, String> {
        crate::yarn::functions::pure(name, args)?
            .ok_or_else(|| format!("function '{name}' is not registered"))
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f64),
    Str(String),
    Ident(String),
    Sym(&'static str),
}

fn tokenize(src: &str) -> Result<Vec<Tok>, String> {
    let b = src.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();
    while i < b.len() {
        let c = b[i] as char;
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if c.is_ascii_digit() {
            let start = i;
            while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'.') {
                i += 1;
            }
            let t = &src[start..i];
            let n: f64 = t.parse().map_err(|_| format!("'{t}' is not a number"))?;
            out.push(Tok::Num(n));
            continue;
        }
        if c == '"' {
            let start = i;
            i += 1;
            while i < b.len() && b[i] != b'"' {
                if b[i] == b'\\' {
                    i += 1;
                }
                i += 1;
            }
            if i >= b.len() {
                return Err("a double-quoted string is not closed".into());
            }
            i += 1;
            let value = serde_json::from_str(&src[start..i])
                .map_err(|e| format!("invalid string escape: {e}"))?;
            out.push(Tok::Str(value));
            continue;
        }
        if c == '\'' {
            let start = i + 1;
            i = start;
            while i < b.len() && b[i] != b'\'' {
                i += 1;
            }
            if i >= b.len() {
                return Err("a string is not closed: write 'text'".into());
            }
            out.push(Tok::Str(src[start..i].to_string()));
            i += 1;
            continue;
        }
        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b'.') {
                i += 1;
            }
            out.push(Tok::Ident(src[start..i].to_string()));
            continue;
        }
        let two = src.get(i..i + 2).unwrap_or("");
        let sym = match two {
            "==" => Some("=="),
            "!=" => Some("!="),
            "<=" => Some("<="),
            ">=" => Some(">="),
            "&&" => Some("&&"),
            "||" => Some("||"),
            _ => None,
        };
        if let Some(s) = sym {
            out.push(Tok::Sym(s));
            i += 2;
            continue;
        }
        let one = match c {
            '+' => "+",
            '-' => "-",
            '*' => "*",
            '/' => "/",
            '%' => "%",
            '<' => "<",
            '>' => ">",
            '(' => "(",
            ')' => ")",
            ',' => ",",
            '!' => "!",
            '=' => return Err("'=' compares nothing: write '==' to compare".into()),
            _ => return Err(format!("'{c}' is not part of an expression")),
        };
        out.push(Tok::Sym(one));
        i += 1;
    }
    Ok(out)
}

struct Parser {
    toks: Vec<Tok>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.at)
    }
    fn sym(&mut self, s: &str) -> bool {
        if matches!(self.peek(), Some(Tok::Sym(x)) if *x == s) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn word(&mut self, w: &str) -> bool {
        if matches!(self.peek(), Some(Tok::Ident(x)) if x == w) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn or(&mut self) -> Result<Expr, String> {
        let mut l = self.and()?;
        while self.word("or") || self.sym("||") {
            l = Expr::Bin(Op::Or, Box::new(l), Box::new(self.and()?));
        }
        Ok(l)
    }
    fn and(&mut self) -> Result<Expr, String> {
        let mut l = self.not()?;
        while self.word("and") || self.sym("&&") {
            l = Expr::Bin(Op::And, Box::new(l), Box::new(self.not()?));
        }
        Ok(l)
    }
    fn not(&mut self) -> Result<Expr, String> {
        if self.word("not") || self.sym("!") {
            return Ok(Expr::Not(Box::new(self.not()?)));
        }
        self.cmp()
    }
    fn cmp(&mut self) -> Result<Expr, String> {
        let l = self.sum()?;
        for (s, op) in [
            ("==", Op::Eq),
            ("!=", Op::Ne),
            ("<=", Op::Le),
            (">=", Op::Ge),
            ("<", Op::Lt),
            (">", Op::Gt),
        ] {
            if self.sym(s) {
                return Ok(Expr::Bin(op, Box::new(l), Box::new(self.sum()?)));
            }
        }
        Ok(l)
    }
    fn sum(&mut self) -> Result<Expr, String> {
        let mut l = self.prod()?;
        loop {
            let op = if self.sym("+") {
                Op::Add
            } else if self.sym("-") {
                Op::Sub
            } else {
                return Ok(l);
            };
            l = Expr::Bin(op, Box::new(l), Box::new(self.prod()?));
        }
    }
    fn prod(&mut self) -> Result<Expr, String> {
        let mut l = self.unary()?;
        loop {
            let op = if self.sym("*") {
                Op::Mul
            } else if self.sym("/") {
                Op::Div
            } else if self.sym("%") {
                Op::Rem
            } else {
                return Ok(l);
            };
            l = Expr::Bin(op, Box::new(l), Box::new(self.unary()?));
        }
    }
    fn unary(&mut self) -> Result<Expr, String> {
        if self.sym("-") {
            return Ok(Expr::Neg(Box::new(self.unary()?)));
        }
        self.atom()
    }
    fn ident_arg(&mut self, f: &str) -> Result<String, String> {
        match self.toks.get(self.at).cloned() {
            Some(Tok::Ident(s)) => {
                self.at += 1;
                Ok(s)
            }
            _ => Err(format!("{f}(...) takes a name, such as {f}(intro)")),
        }
    }
    fn atom(&mut self) -> Result<Expr, String> {
        let t = self
            .toks
            .get(self.at)
            .cloned()
            .ok_or("the expression ends early")?;
        self.at += 1;
        match t {
            Tok::Num(n) => Ok(Expr::Lit(Val::Num(n))),
            Tok::Str(s) => Ok(Expr::Lit(Val::Str(s))),
            Tok::Sym("(") => {
                let e = self.or()?;
                if !self.sym(")") {
                    return Err("a '(' is not closed".into());
                }
                Ok(e)
            }
            Tok::Ident(w) if w == "true" => Ok(Expr::Lit(Val::Bool(true))),
            Tok::Ident(w) if w == "false" => Ok(Expr::Lit(Val::Bool(false))),
            Tok::Ident(w) if matches!(w.as_str(), "and" | "or" | "not") => {
                Err(format!("'{w}' needs something on its left and right"))
            }
            Tok::Ident(w) if self.sym("(") => {
                if !matches!(w.as_str(), "seen" | "status" | "stage" | "count") {
                    return Err(format!(
                        "'{w}' is not a story function: seen, status, stage or count"
                    ));
                }
                let a = self.ident_arg(&w)?;
                let f = match w.as_str() {
                    "seen" => Func::Seen(a),
                    "status" => Func::Status(a),
                    "stage" => Func::Stage(a),
                    "count" => {
                        if !self.sym(",") {
                            return Err("count(quest, objective) takes two names".into());
                        }
                        Func::Count(a, self.ident_arg("count")?)
                    }
                    _ => unreachable!("checked above"),
                };
                if !self.sym(")") {
                    return Err(format!("{w}(...) is not closed"));
                }
                Ok(Expr::Call(f))
            }
            Tok::Ident(w) => Ok(Expr::Var(w)),
            Tok::Sym(s) => Err(format!("'{s}' is out of place")),
        }
    }
}

impl Expr {
    /// Parses an expression; the error says what to write instead.
    pub fn parse(src: &str) -> Result<Expr, String> {
        let toks = tokenize(src)?;
        if toks.is_empty() {
            return Err("the expression is empty".into());
        }
        let mut p = Parser { toks, at: 0 };
        let e = p.or()?;
        if p.at != p.toks.len() {
            return Err(format!(
                "'{}' is left over: join conditions with and / or",
                describe(&p.toks[p.at])
            ));
        }
        Ok(e)
    }

    /// Every variable it reads, in order.
    pub fn vars(&self, out: &mut Vec<String>) {
        match self {
            Expr::Var(v) => out.push(v.clone()),
            Expr::Not(e) | Expr::Neg(e) | Expr::Yarn(e) => e.vars(out),
            Expr::Bin(_, a, b) => {
                a.vars(out);
                b.vars(out);
            }
            Expr::Invoke(_, args) => {
                for arg in args {
                    arg.vars(out);
                }
            }
            Expr::Once(_)
            | Expr::Lit(_)
            | Expr::Call(_)
            | Expr::Member(..)
            | Expr::EnumValue(..) => {}
        }
    }

    /// Every function call, in order.
    pub fn calls<'a>(&'a self, out: &mut Vec<&'a Func>) {
        match self {
            Expr::Call(f) => out.push(f),
            Expr::Not(e) | Expr::Neg(e) | Expr::Yarn(e) => e.calls(out),
            Expr::Bin(_, a, b) => {
                a.calls(out);
                b.calls(out);
            }
            Expr::Invoke(_, args) => {
                for arg in args {
                    arg.calls(out);
                }
            }
            Expr::Once(_)
            | Expr::Lit(_)
            | Expr::Var(_)
            | Expr::Member(..)
            | Expr::EnumValue(..) => {}
        }
    }

    /// Its value in `scope`.
    pub fn eval(&self, scope: &dyn Scope) -> Result<Val, String> {
        self.eval_in(scope, false)
    }

    fn eval_in(&self, scope: &dyn Scope, yarn: bool) -> Result<Val, String> {
        Ok(match self {
            Expr::Once(id) => Val::Bool(scope.once(id)),
            Expr::Lit(v) | Expr::EnumValue(_, v) => v.clone(),
            Expr::Yarn(e) => e.eval_in(scope, true)?,
            Expr::Member(ty, member) => {
                return Err(format!("unresolved enum member {ty}.{member}"));
            }
            Expr::Invoke(name, args) => {
                let args = args
                    .iter()
                    .map(|a| a.eval_in(scope, yarn))
                    .collect::<Result<Vec<_>, _>>()?;
                scope.invoke(name, &args)?
            }
            Expr::Var(n) => scope
                .var(n)
                .ok_or_else(|| format!("the story has no variable '{n}'"))?,
            Expr::Not(e) => Val::Bool(!e.eval_in(scope, yarn)?.truthy()),
            Expr::Neg(e) => Val::Num(-num(&e.eval_in(scope, yarn)?, "-")?),
            Expr::Call(f) => match f {
                Func::Seen(n) => Val::Bool(scope.seen(n)),
                Func::Status(q) => Val::Str(scope.quest_status(q).to_string()),
                Func::Stage(q) => Val::Str(scope.quest_stage(q)),
                Func::Count(q, o) => Val::Num(scope.quest_count(q, o)),
            },
            Expr::Bin(Op::And, a, b) if !yarn => {
                Val::Bool(a.eval(scope)?.truthy() && b.eval(scope)?.truthy())
            }
            Expr::Bin(Op::Or, a, b) if !yarn => {
                Val::Bool(a.eval(scope)?.truthy() || b.eval(scope)?.truthy())
            }
            Expr::Bin(op @ (Op::And | Op::Or | Op::Xor), a, b) => {
                let (a, b) = (
                    a.eval_in(scope, yarn)?.truthy(),
                    b.eval_in(scope, yarn)?.truthy(),
                );
                Val::Bool(match op {
                    Op::And => a && b,
                    Op::Or => a || b,
                    _ => a ^ b,
                })
            }
            Expr::Bin(op, a, b) => {
                let (a, b) = (a.eval_in(scope, yarn)?, b.eval_in(scope, yarn)?);
                match (&a, &b) {
                    (Val::Num(x), Val::Num(y))
                        if yarn
                            && matches!(op, Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Rem) =>
                    {
                        let (x, y) = (*x as f32, *y as f32);
                        Val::Num(f64::from(match op {
                            Op::Add => x + y,
                            Op::Sub => x - y,
                            Op::Mul => x * y,
                            Op::Div => x / y,
                            _ => x % y,
                        }))
                    }
                    _ => binary(*op, a, b)?,
                }
            }
        })
    }
}

fn describe(t: &Tok) -> String {
    match t {
        Tok::Num(n) => n.to_string(),
        Tok::Str(s) => format!("'{s}'"),
        Tok::Ident(s) => s.clone(),
        Tok::Sym(s) => s.to_string(),
    }
}

fn num(v: &Val, op: &str) -> Result<f64, String> {
    match v {
        Val::Num(n) => Ok(*n),
        other => Err(format!("'{op}' takes numbers, not {}", other.kind())),
    }
}

fn binary(op: Op, a: Val, b: Val) -> Result<Val, String> {
    let sym = match op {
        Op::Add => "+",
        Op::Sub => "-",
        Op::Mul => "*",
        Op::Div => "/",
        Op::Rem => "%",
        Op::Eq => "==",
        Op::Ne => "!=",
        Op::Lt => "<",
        Op::Le => "<=",
        Op::Gt => ">",
        Op::Ge => ">=",
        Op::And | Op::Or | Op::Xor => unreachable!("evaluated by eval"),
    };
    Ok(match op {
        Op::Eq => Val::Bool(a == b),
        Op::Ne => Val::Bool(a != b),
        Op::Add => match (a, b) {
            (Val::Str(x), Val::Str(y)) => Val::Str(x + &y),
            (a, b) => Val::Num(num(&a, sym)? + num(&b, sym)?),
        },
        Op::Lt | Op::Le | Op::Gt | Op::Ge => {
            let (x, y) = (num(&a, sym)?, num(&b, sym)?);
            Val::Bool(match op {
                Op::Lt => x < y,
                Op::Le => x <= y,
                Op::Gt => x > y,
                _ => x >= y,
            })
        }
        _ => {
            let (x, y) = (num(&a, sym)?, num(&b, sym)?);
            Val::Num(match op {
                Op::Sub => x - y,
                Op::Mul => x * y,
                Op::Div => x / y,
                _ => x % y,
            })
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    struct S(BTreeMap<&'static str, Val>);
    impl Scope for S {
        fn var(&self, n: &str) -> Option<Val> {
            self.0.get(n).cloned()
        }
        fn seen(&self, n: &str) -> bool {
            n == "intro"
        }
        fn quest_status(&self, _: &str) -> &'static str {
            "active"
        }
        fn quest_stage(&self, _: &str) -> String {
            "find".into()
        }
        fn quest_count(&self, _: &str, _: &str) -> f64 {
            3.0
        }
    }

    fn eval(src: &str) -> Val {
        let s = S(BTreeMap::from([
            ("gold", Val::Num(12.0)),
            ("name", Val::Str("Kai".into())),
        ]));
        Expr::parse(src).unwrap().eval(&s).unwrap()
    }

    #[test]
    fn precedence_and_functions() {
        assert_eq!(eval("gold - 2 * 3"), Val::Num(6.0));
        assert_eq!(eval("(gold - 2) * 3"), Val::Num(30.0));
        assert_eq!(eval("gold >= 10 and not seen(other)"), Val::Bool(true));
        assert_eq!(eval("seen(intro) or gold < 0"), Val::Bool(true));
        assert_eq!(
            eval("status(q) == 'active' and stage(q) == 'find'"),
            Val::Bool(true)
        );
        assert_eq!(eval("count(q, wolves) + 1"), Val::Num(4.0));
        assert_eq!(eval("name + '!'"), Val::Str("Kai!".into()));
        assert_eq!(eval("-gold % 5"), Val::Num(-2.0));
        assert_eq!(Val::Num(3.0).to_string(), "3");
        assert_eq!(Val::Num(2.5).to_string(), "2.5");
    }

    #[test]
    fn errors_say_what_to_write() {
        assert!(Expr::parse("gold = 3").unwrap_err().contains("=="));
        assert!(
            Expr::parse("gold >= 3 gold")
                .unwrap_err()
                .contains("and / or")
        );
        assert!(Expr::parse("roll(3)").unwrap_err().contains("seen, status"));
        assert!(Expr::parse("\"x").unwrap_err().contains("not closed"));
        let s = S(BTreeMap::new());
        assert!(
            Expr::parse("gold + 1")
                .unwrap()
                .eval(&s)
                .unwrap_err()
                .contains("'gold'")
        );
    }
}
