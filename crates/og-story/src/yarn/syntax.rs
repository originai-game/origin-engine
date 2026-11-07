//! Expression grammar from YarnSpinnerParser.g4 at the revision in third_party.
//! Kept separate from the quest expression grammar, whose precedence is older.
use crate::expr::{Expr, Op, Val};

// YarnSpinnerLexer.g4 v3.2.2 IDENTIFIER_HEAD / IDENTIFIER_CHARACTER (MIT).
pub(super) fn identifier_head(c: char) -> bool {
    matches!(c as u32,
        0x41..=0x5a | 0x61..=0x7a | 0x5f | 0xa8 | 0xaa | 0xad | 0xaf |
        0xb2..=0xb5 | 0xb7..=0xba | 0xbc..=0xbe | 0xc0..=0xd6 | 0xd8..=0xf6 |
        0xf8..=0x2ff | 0x370..=0x167f | 0x1681..=0x180d | 0x180f..=0x1dbf |
        0x1e00..=0x1fff | 0x200b..=0x200d | 0x202a..=0x202e | 0x203f..=0x2040 |
        0x2054 | 0x2060..=0x206f | 0x2070..=0x20cf | 0x2100..=0x218f |
        0x2460..=0x24ff | 0x2776..=0x2793 | 0x2c00..=0x2dff | 0x2e80..=0x2fff |
        0x3004..=0x3007 | 0x3021..=0x302f | 0x3031..=0xd7ff | 0xf900..=0xfd3d |
        0xfd40..=0xfdcf | 0xfdf0..=0xfe1f | 0xfe30..=0xfe44 | 0xfe47..=0xfffd)
        || ((0x10000..=0xefffd).contains(&(c as u32)) && c as u32 & 0xffff <= 0xfffd)
}

pub(super) fn identifier_continue(c: char) -> bool {
    identifier_head(c)
        || c.is_ascii_digit()
        || matches!(c as u32,
        0x300..=0x36f | 0x1dc0..=0x1dff | 0x20d0..=0x20ff | 0xfe20..=0xfe2f)
}

pub(super) fn identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(identifier_head) && chars.all(identifier_continue)
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(f64),
    String(String),
    Variable(String),
    Name(String),
    Operator(Op),
    Not,
    Open,
    Close,
    Comma,
    Dot,
}

fn lex(text: &str) -> Result<Vec<Token>, String> {
    let mut chars = text.char_indices().peekable();
    let mut out = Vec::new();
    while let Some((start, ch)) = chars.next() {
        if ch.is_whitespace() {
            continue;
        }
        if ch == '"' {
            let mut value = String::new();
            let mut closed = false;
            while let Some((_, c)) = chars.next() {
                match c {
                    '"' => {
                        closed = true;
                        break;
                    }
                    '\\' => match chars.next() {
                        Some((_, c @ ('"' | '\\'))) => value.push(c),
                        _ => return Err("Yarn strings escape only quotes and backslashes".into()),
                    },
                    '\n' | '\r' => return Err("newline in a Yarn expression string".into()),
                    c => value.push(c),
                }
            }
            if !closed {
                return Err("unclosed Yarn string".into());
            }
            out.push(Token::String(value));
            continue;
        }
        if ch.is_ascii_digit()
            || (ch == '.' && chars.peek().is_some_and(|(_, c)| c.is_ascii_digit()))
        {
            while chars.peek().is_some_and(|(_, c)| c.is_ascii_digit()) {
                chars.next();
            }
            if ch != '.' && chars.peek().is_some_and(|(_, c)| *c == '.') {
                chars.next();
                while chars.peek().is_some_and(|(_, c)| c.is_ascii_digit()) {
                    chars.next();
                }
            }
            let end = chars.peek().map_or(text.len(), |(i, _)| *i);
            let number = text[start..end]
                .parse::<f32>()
                .map_err(|_| "invalid Yarn number")?;
            if !number.is_finite() {
                return Err("Yarn number is outside the finite number range".into());
            }
            out.push(Token::Number(f64::from(number)));
            continue;
        }
        if ch == '$' || identifier_head(ch) {
            while chars.peek().is_some_and(|(_, c)| identifier_continue(*c)) {
                chars.next();
            }
            let end = chars.peek().map_or(text.len(), |(i, _)| *i);
            let word = &text[start..end];
            if let Some(name) = word.strip_prefix('$') {
                if !identifier(name) {
                    return Err("a Yarn variable needs a name after $".into());
                }
                out.push(Token::Variable(name.into()));
                continue;
            }
            out.push(match word {
                "and" => Token::Operator(Op::And),
                "or" => Token::Operator(Op::Or),
                "xor" => Token::Operator(Op::Xor),
                "not" => Token::Not,
                "is" | "eq" => Token::Operator(Op::Eq),
                "neq" => Token::Operator(Op::Ne),
                "lt" => Token::Operator(Op::Lt),
                "lte" => Token::Operator(Op::Le),
                "gt" => Token::Operator(Op::Gt),
                "gte" => Token::Operator(Op::Ge),
                _ => Token::Name(word.into()),
            });
            continue;
        }
        let equal = chars.peek().is_some_and(|(_, c)| *c == '=');
        let token = match ch {
            '(' => Token::Open,
            ')' => Token::Close,
            ',' => Token::Comma,
            '.' => Token::Dot,
            '+' => Token::Operator(Op::Add),
            '-' => Token::Operator(Op::Sub),
            '*' => Token::Operator(Op::Mul),
            '/' => Token::Operator(Op::Div),
            '%' => Token::Operator(Op::Rem),
            '^' => Token::Operator(Op::Xor),
            '!' if !equal => Token::Not,
            '<' if !equal => Token::Operator(Op::Lt),
            '>' if !equal => Token::Operator(Op::Gt),
            '=' | '!' | '<' | '>' if equal => {
                chars.next();
                Token::Operator(match ch {
                    '=' => Op::Eq,
                    '!' => Op::Ne,
                    '<' => Op::Le,
                    _ => Op::Ge,
                })
            }
            '&' | '|' if chars.peek().is_some_and(|(_, c)| *c == ch) => {
                chars.next();
                Token::Operator(if ch == '&' { Op::And } else { Op::Or })
            }
            _ => return Err(format!("unexpected '{ch}' in Yarn expression")),
        };
        out.push(token);
    }
    Ok(out)
}

struct Parser {
    tokens: std::vec::IntoIter<Token>,
    next: Option<Token>,
}

impl Parser {
    fn take(&mut self) -> Option<Token> {
        std::mem::replace(&mut self.next, self.tokens.next())
    }

    fn expect(&mut self, token: Token) -> Result<(), String> {
        if self.take() == Some(token.clone()) {
            Ok(())
        } else {
            Err(format!("expected {token:?} in Yarn expression"))
        }
    }

    fn name(&mut self) -> Result<String, String> {
        match self.take() {
            Some(Token::Name(name)) => Ok(name),
            _ => Err("expected enum member name".into()),
        }
    }

    fn expression(&mut self, min: u8) -> Result<Expr, String> {
        let mut left = match self.take().ok_or("Yarn expression ends early")? {
            Token::Number(n) => Expr::Lit(Val::Num(n)),
            Token::String(s) => Expr::Lit(Val::Str(s)),
            Token::Variable(v) => Expr::Var(v),
            Token::Name(n) if n == "true" || n == "false" => Expr::Lit(Val::Bool(n == "true")),
            Token::Name(n) if self.next == Some(Token::Open) => {
                self.take();
                let mut args = Vec::new();
                if self.next != Some(Token::Close) {
                    loop {
                        args.push(self.expression(0)?);
                        if self.next != Some(Token::Comma) {
                            break;
                        }
                        self.take();
                    }
                }
                self.expect(Token::Close)?;
                Expr::Invoke(n, args)
            }
            Token::Name(n) if self.next == Some(Token::Dot) => {
                self.take();
                Expr::Member(n, self.name()?)
            }
            Token::Dot => Expr::Member(String::new(), self.name()?),
            Token::Name(n) => return Err(format!("bare name '{n}': variables start with $")),
            Token::Open => {
                let inner = self.expression(0)?;
                self.expect(Token::Close)?;
                inner
            }
            Token::Not => Expr::Not(Box::new(self.expression(6)?)),
            Token::Operator(Op::Sub) => Expr::Neg(Box::new(self.expression(7)?)),
            token => return Err(format!("unexpected {token:?} in Yarn expression")),
        };
        while let Some(Token::Operator(op)) = self.next {
            let priority = match op {
                Op::And | Op::Or | Op::Xor => 1,
                Op::Eq | Op::Ne => 2,
                Op::Lt | Op::Le | Op::Gt | Op::Ge => 3,
                Op::Add | Op::Sub => 4,
                Op::Mul | Op::Div | Op::Rem => 5,
            };
            if priority < min {
                break;
            }
            self.take();
            left = Expr::Bin(op, Box::new(left), Box::new(self.expression(priority + 1)?));
        }
        Ok(left)
    }
}

pub(super) fn parse(text: &str) -> Result<Expr, String> {
    let mut tokens = lex(text)?.into_iter();
    let next = tokens.next();
    let mut parser = Parser { tokens, next };
    let expression = parser.expression(0)?;
    if parser.next.is_some() {
        return Err("unexpected trailing Yarn expression tokens".into());
    }
    Ok(Expr::Yarn(Box::new(expression)))
}
