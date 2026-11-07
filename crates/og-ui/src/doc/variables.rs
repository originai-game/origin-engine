//! CSS Variables Level 1: compute on the declaring element, then inherit.
//! Dependency edges include fallback references even when the fallback is unused.
use cssparser::{Delimiter, ParseError, Parser, Token};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq)]
enum Piece {
    Text(String),
    Block(String, Value, char),
    Var(String, Option<Value>),
}

/// A token stream, not concatenated identifiers: `var(--n)px` is not a length.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Value(Vec<Piece>);

/// None is the guaranteed-invalid value; an empty string is a valid value.
pub type Computed = BTreeMap<String, Option<String>>;

impl Value {
    pub fn parse(text: &str) -> Result<Self, String> {
        parse(&mut Parser::new(text)).map_err(|e| format!("invalid CSS value: {e:?}"))
    }

    pub fn dependencies(&self, out: &mut BTreeSet<String>) {
        for piece in &self.0 {
            match piece {
                Piece::Var(name, fallback) => {
                    out.insert(name.clone());
                    if let Some(fallback) = fallback {
                        fallback.dependencies(out);
                    }
                }
                Piece::Block(_, value, _) => value.dependencies(out),
                Piece::Text(_) => {}
            }
        }
    }

    pub fn has_variables(&self) -> bool {
        let mut names = BTreeSet::new();
        self.dependencies(&mut names);
        !names.is_empty()
    }

    /// Unresolved engine tokens can be supplied by the active style. The caller
    /// must not supply a token for a locally declared guaranteed-invalid value.
    pub fn substitute(
        &self,
        lookup: &mut impl FnMut(&str) -> Option<String>,
    ) -> Result<String, String> {
        let mut out = String::new();
        for piece in &self.0 {
            match piece {
                Piece::Text(text) => out.push_str(text),
                Piece::Block(open, value, close) => {
                    out.push_str(open);
                    out.push_str(&value.substitute(lookup)?);
                    out.push(*close);
                }
                Piece::Var(name, fallback) => {
                    let value = match lookup(name) {
                        Some(value) => value,
                        None => match fallback {
                            Some(value) => value.substitute(lookup)?,
                            None => {
                                return Err(format!("{name} is invalid at computed-value time"));
                            }
                        },
                    };
                    out.push(' ');
                    out.push_str(&value);
                    out.push(' ');
                }
            }
            if out.len() > 1_048_576 {
                return Err("custom property expansion exceeds 1 MiB".into());
            }
        }
        Ok(out)
    }
}

pub(super) fn declarations(text: &str) -> Result<Vec<super::StyleDeclaration>, String> {
    let mut input = Parser::new(text);
    let mut result = Vec::new();
    while !input.is_exhausted() {
        if input.try_parse(|p| p.expect_semicolon()).is_ok() {
            continue;
        }
        let declaration = input
            .parse_until_before(Delimiter::Semicolon, |input| {
                let name = input.expect_ident_cloned()?.to_string();
                input.expect_colon()?;
                let start = input.position();
                let value = input.parse_until_before(Delimiter::Bang, parse)?;
                let source = input.slice_from(start).trim().to_string();
                let important = if input.is_exhausted() {
                    false
                } else {
                    input.expect_delim('!')?;
                    input.expect_ident_matching("important")?;
                    true
                };
                Ok(super::StyleDeclaration {
                    name,
                    value,
                    source,
                    important,
                })
            })
            .map_err(|error: ParseError<String>| format!("invalid CSS declaration: {error:?}"))?;
        result.push(declaration);
    }
    Ok(result)
}

fn parse(input: &mut Parser<'_>) -> Result<Value, ParseError<String>> {
    let mut pieces = Vec::new();
    while !input.is_exhausted() {
        let start = input.position();
        let token = input.next_including_whitespace_and_comments()?.clone();
        let open = input.slice_from(start).to_string();
        match token {
            Token::Function(name) if name.eq_ignore_ascii_case("var") => {
                let (name, fallback) = input.parse_nested_block(|input| {
                    let name = input.expect_ident_cloned()?.to_string();
                    if !name.starts_with("--") || name.len() == 2 {
                        return Err(ParseError::custom("var() requires a --name"));
                    }
                    let fallback = if input.is_exhausted() {
                        None
                    } else {
                        input.expect_comma()?;
                        Some(parse(input)?)
                    };
                    Ok((name, fallback))
                })?;
                pieces.push(Piece::Var(name, fallback));
            }
            Token::Function(_)
            | Token::ParenthesisBlock
            | Token::SquareBracketBlock
            | Token::CurlyBracketBlock => {
                let close = match token {
                    Token::SquareBracketBlock => ']',
                    Token::CurlyBracketBlock => '}',
                    _ => ')',
                };
                let body = input.parse_nested_block(parse)?;
                pieces.push(Piece::Block(open, body, close));
            }
            Token::BadUrl(_)
            | Token::BadString(_)
            | Token::CloseParenthesis
            | Token::CloseSquareBracket
            | Token::CloseCurlyBracket => {
                return Err(ParseError::custom("unbalanced CSS value"));
            }
            Token::Comment(_) => pieces.push(Piece::Text(" ".into())),
            _ => pieces.push(Piece::Text(open)),
        }
    }
    Ok(Value(pieces))
}

/// Values have already won the cascade. Inherited values are computed, so
/// changing --a on this element cannot change an inherited --b: var(--a).
pub fn compute(
    inherited: &Computed,
    specified: &BTreeMap<String, Value>,
    tokens: &mut impl FnMut(&str) -> Option<String>,
) -> (Computed, Vec<String>, BTreeSet<String>) {
    let graph: BTreeMap<_, _> = specified
        .iter()
        .map(|(name, value)| {
            let mut dependencies = BTreeSet::new();
            value.dependencies(&mut dependencies);
            (name.clone(), dependencies)
        })
        .collect();
    fn visit(
        name: &str,
        graph: &BTreeMap<String, BTreeSet<String>>,
        stack: &mut Vec<String>,
        seen: &mut BTreeSet<String>,
        cycles: &mut BTreeSet<String>,
    ) {
        if let Some(start) = stack.iter().position(|n| n == name) {
            cycles.extend(stack[start..].iter().cloned());
            return;
        }
        if !seen.insert(name.into()) {
            return;
        }
        stack.push(name.into());
        if let Some(edges) = graph.get(name) {
            for edge in edges {
                visit(edge, graph, stack, seen, cycles);
            }
        }
        stack.pop();
    }
    let mut cycles = BTreeSet::new();
    for name in specified.keys() {
        visit(
            name,
            &graph,
            &mut Vec::new(),
            &mut BTreeSet::new(),
            &mut cycles,
        );
    }
    let mut out = inherited.clone();
    for name in specified.keys() {
        out.remove(name);
    }
    for name in &cycles {
        out.insert(name.clone(), None);
    }
    let mut errors: Vec<_> = cycles
        .iter()
        .map(|name| {
            format!(
                "custom property {name} participates in a cycle; invalid at computed-value time"
            )
        })
        .collect();
    fn resolve(
        name: &str,
        specified: &BTreeMap<String, Value>,
        inherited: &Computed,
        out: &mut Computed,
        tokens: &mut impl FnMut(&str) -> Option<String>,
        errors: &mut Vec<String>,
        inherited_reads: &mut BTreeSet<String>,
    ) -> Option<String> {
        if !specified.contains_key(name) {
            inherited_reads.insert(name.into());
        }
        if let Some(value) = out.get(name) {
            return value.clone();
        }
        let Some(value) = specified.get(name) else {
            return tokens(name);
        };
        let mut significant = value.0.iter().filter(|piece| {
            !matches!(piece,
                Piece::Text(text) if text.trim().is_empty() || text.starts_with("/*")
            )
        });
        let keyword = match (significant.next(), significant.next()) {
            (Some(Piece::Text(word)), None) => word.to_ascii_lowercase(),
            _ => String::new(),
        };
        let result = match keyword.as_str() {
            "initial" => None,
            "inherit" | "unset" => {
                inherited_reads.insert(name.into());
                inherited.get(name).cloned().flatten()
            }
            _ => match value.substitute(&mut |dependency| {
                resolve(
                    dependency,
                    specified,
                    inherited,
                    out,
                    tokens,
                    errors,
                    inherited_reads,
                )
            }) {
                Ok(value) => Some(value),
                Err(error) => {
                    errors.push(format!("{name}: {error}"));
                    None
                }
            },
        };
        out.insert(name.into(), result.clone());
        result
    }
    let mut inherited_reads = BTreeSet::new();
    for name in specified.keys() {
        resolve(
            name,
            specified,
            inherited,
            &mut out,
            tokens,
            &mut errors,
            &mut inherited_reads,
        );
    }
    (out, errors, inherited_reads)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn declarations(values: &[(&str, &str)]) -> BTreeMap<String, Value> {
        values
            .iter()
            .map(|(name, value)| (name.to_string(), Value::parse(value).unwrap()))
            .collect()
    }

    #[test]
    fn inheritance_uses_computed_values_and_cycles_include_unused_fallbacks() {
        let (parent, errors, _) = compute(
            &Computed::new(),
            &declarations(&[("--a", "12px"), ("--b", "var(--a)")]),
            &mut |_| None,
        );
        assert!(errors.is_empty());
        let (child, errors, reads) = compute(
            &parent,
            &declarations(&[
                ("--a", "var(--b)"),
                ("--safe", "green"),
                ("--cycle", "var(--safe, var(--cycle))"),
                ("--use", "var(--cycle, 7px)"),
            ]),
            &mut |_| None,
        );
        assert_eq!(child["--a"].as_deref().unwrap().trim(), "12px");
        assert_eq!(child["--cycle"], None);
        assert_eq!(child["--use"].as_deref().unwrap().trim(), "7px");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("--cycle"));
        assert_eq!(reads, BTreeSet::from(["--b".into()]));
    }

    #[test]
    fn empty_fallback_and_token_boundaries_are_not_string_concatenation() {
        let (values, errors, reads) = compute(
            &Computed::new(),
            &declarations(&[
                ("--empty", "var(--missing,)"),
                ("--n", "20"),
                ("--dimension", "var(--n)px"),
                ("--quoted", "'var(--missing)'"),
                ("--comment", "18px /* note */"),
            ]),
            &mut |_| None,
        );
        assert!(errors.is_empty());
        assert_eq!(values["--empty"].as_deref().unwrap().trim(), "");
        assert_eq!(values["--dimension"].as_deref().unwrap().trim(), "20 px");
        assert_eq!(values["--quoted"].as_deref(), Some("'var(--missing)'"));
        assert_eq!(values["--comment"].as_deref().unwrap().trim(), "18px");
        assert_eq!(reads, BTreeSet::from(["--missing".into()]));
        for invalid in ["var(color)", "var(--a red)", "var()"] {
            assert!(Value::parse(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn dependency_reads_follow_only_evaluated_inherited_lookups() {
        let inherited = Computed::from([
            ("--parent".into(), Some("18px".into())),
            ("--reset".into(), Some("4px".into())),
            ("--invalid".into(), None),
        ]);
        let (values, errors, reads) = compute(
            &inherited,
            &declarations(&[
                ("--local", "32px"),
                ("--uses-local", "var(--local, var(--unused))"),
                ("--uses-fallback", "var(--invalid, var(--parent))"),
                ("--reset", " /* comment */ initial "),
                ("--reset-use", "var(--reset, 6px)"),
            ]),
            &mut |_| None,
        );
        assert!(errors.is_empty());
        assert_eq!(values["--reset"], None);
        assert_eq!(values["--reset-use"].as_deref().unwrap().trim(), "6px");
        assert_eq!(
            reads,
            BTreeSet::from(["--invalid".into(), "--parent".into()])
        );
    }
}
