//! Parsed scoped selectors. Functional pseudo-classes contribute their argument
//! specificity (`:where` contributes zero), not punctuation counts.
use super::{NONE, UiDocument, syntax::Attr};
use crate::bind::{Expr, InteractionState, Op};
use cssparser::{ParseError, Parser, Token};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Specificity(pub u32, pub u32, pub u32);

impl Specificity {
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Combinator {
    Descendant,
    Child,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Simple {
    Type(String),
    Id(String),
    Class(String),
    Attribute { name: String, value: Option<String> },
    State(InteractionState),
    Root,
    Not(Vec<Selector>),
    Is(Vec<Selector>),
    Where(Vec<Selector>),
    Part(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Compound(pub Vec<Simple>);

#[derive(Clone, Debug, PartialEq)]
pub struct Selector {
    pub source: String,
    pub compounds: Vec<Compound>,
    pub combinators: Vec<Combinator>,
}

impl Selector {
    pub fn parse_list(text: &str) -> Result<Vec<Self>, String> {
        Parser::new(text)
            .parse_comma_separated(parse)
            .map_err(|error| format!("invalid selector `{text}`: {error:?}"))
    }

    pub fn specificity(&self) -> Specificity {
        self.compounds.iter().flat_map(|compound| &compound.0).fold(
            Specificity::default(),
            |sum, simple| {
                sum.add(match simple {
                    Simple::Id(_) => Specificity(1, 0, 0),
                    Simple::Class(_)
                    | Simple::Attribute { .. }
                    | Simple::State(_)
                    | Simple::Root => Specificity(0, 1, 0),
                    Simple::Type(_) | Simple::Part(_) => Specificity(0, 0, 1),
                    Simple::Not(list) | Simple::Is(list) => {
                        list.iter().map(Self::specificity).max().unwrap_or_default()
                    }
                    Simple::Where(_) => Specificity::default(),
                })
            },
        )
    }

    pub(super) fn condition(
        &self,
        doc: &UiDocument,
        elements: &[(usize, String, Vec<Attr>)],
        index: usize,
    ) -> Result<Expr, String> {
        fn chain(
            selector: &Selector,
            doc: &UiDocument,
            elements: &[(usize, String, Vec<Attr>)],
            index: usize,
            part: usize,
        ) -> Result<Expr, String> {
            let Some((_, tag, attrs)) = elements.iter().find(|(i, _, _)| *i == index) else {
                return Ok(Expr::Lit(false.into()));
            };
            let mut own = Expr::Lit(true.into());
            let expression = |attr: &Attr| {
                super::svelte::expression(&attr.value, attr.at).map_err(|(_, error)| error)
            };
            for simple in &selector.compounds[part].0 {
                let test = match simple {
                    Simple::Type(name) => Expr::Lit((name == tag).into()),
                    Simple::Root => Expr::Lit((index == doc.root).into()),
                    Simple::Id(name) => Expr::Lit(
                        attrs
                            .iter()
                            .any(|a| a.name == "id" && a.value == *name)
                            .into(),
                    ),
                    Simple::Class(name) => {
                        let mut matches = Expr::Lit(false.into());
                        if let Some(attr) = attrs.iter().find(|a| a.name == "class") {
                            matches = if attr.bound {
                                Expr::HasToken(Box::new(expression(attr)?), name.clone())
                            } else {
                                Expr::Lit(
                                    attr.value
                                        .split_ascii_whitespace()
                                        .any(|v| v == name)
                                        .into(),
                                )
                            };
                        }
                        if let Some(attr) = attrs.iter().find(|a| a.name == format!("class:{name}"))
                        {
                            matches = combine(Op::Or, matches, expression(attr)?);
                        }
                        matches
                    }
                    Simple::Attribute { name, value } => {
                        match attrs.iter().find(|a| a.name == *name) {
                            None => Expr::Lit(false.into()),
                            Some(attr) => match value {
                                None => {
                                    if attr.bound
                                        && matches!(
                                            name.as_str(),
                                            "disabled" | "checked" | "selected"
                                        )
                                    {
                                        expression(attr)?
                                    } else if attr.bound {
                                        Expr::Bin(
                                            Op::Ne,
                                            Box::new(expression(attr)?),
                                            Box::new(Expr::Lit(serde_json::Value::Null)),
                                        )
                                    } else {
                                        Expr::Lit(true.into())
                                    }
                                }
                                Some(expected) => {
                                    if attr.bound {
                                        Expr::Bin(
                                            Op::Eq,
                                            Box::new(Expr::Bin(
                                                Op::Add,
                                                Box::new(Expr::Lit("".into())),
                                                Box::new(expression(attr)?),
                                            )),
                                            Box::new(Expr::Lit(expected.clone().into())),
                                        )
                                    } else {
                                        Expr::Lit((attr.value == *expected).into())
                                    }
                                }
                            },
                        }
                    }
                    Simple::State(state) => Expr::InteractionAt {
                        node: index,
                        state: *state,
                    },
                    Simple::Not(list) | Simple::Is(list) | Simple::Where(list) => {
                        let mut matches = Expr::Lit(false.into());
                        for inner in list {
                            matches =
                                combine(Op::Or, matches, inner.condition(doc, elements, index)?);
                        }
                        if matches!(simple, Simple::Not(_)) {
                            match matches {
                                Expr::Lit(serde_json::Value::Bool(value)) => {
                                    Expr::Lit((!value).into())
                                }
                                value => Expr::Not(Box::new(value)),
                            }
                        } else {
                            matches
                        }
                    }
                    Simple::Part(_) => {
                        return Err("::part must be the final selector's named text span".into());
                    }
                };
                own = combine(Op::And, own, test);
            }
            if part == 0 {
                return Ok(own);
            }
            let combinator = selector.combinators[part - 1];
            let mut candidates = Vec::new();
            let mut parent = doc.nodes[index].parent;
            while parent != NONE {
                if elements.iter().any(|(i, _, _)| *i == parent) {
                    candidates.push(parent);
                    if combinator == Combinator::Child {
                        break;
                    }
                }
                parent = doc.nodes[parent].parent;
            }
            let mut matches = Expr::Lit(false.into());
            for candidate in candidates {
                matches = combine(
                    Op::Or,
                    matches,
                    chain(selector, doc, elements, candidate, part - 1)?,
                );
            }
            Ok(combine(Op::And, own, matches))
        }
        chain(self, doc, elements, index, self.compounds.len() - 1)
    }
}

fn combine(op: Op, a: Expr, b: Expr) -> Expr {
    let identity = op == Op::And;
    match (&a, &b) {
        (Expr::Lit(serde_json::Value::Bool(value)), _) => {
            if *value == identity {
                b
            } else {
                a
            }
        }
        (_, Expr::Lit(serde_json::Value::Bool(value))) => {
            if *value == identity {
                a
            } else {
                b
            }
        }
        _ => Expr::Bin(op, Box::new(a), Box::new(b)),
    }
}

fn parse(input: &mut Parser<'_>) -> Result<Selector, ParseError<String>> {
    let start = input.position();
    let mut selector = Selector {
        source: String::new(),
        compounds: Vec::new(),
        combinators: Vec::new(),
    };
    let mut compound = Vec::new();
    let mut started = false;
    let mut whitespace = false;
    while !input.is_exhausted() {
        let token = input.next_including_whitespace_and_comments()?.clone();
        if matches!(token, Token::Comment(_)) {
            continue;
        }
        if matches!(token, Token::WhiteSpace(_)) {
            whitespace |= started;
            continue;
        }
        if let Token::Delim(c @ ('>' | '+' | '~')) = token {
            if c != '>' {
                return Err(ParseError::custom(format!(
                    "unsupported sibling combinator '{c}'; use a descendant or child selector"
                )));
            }
            if !started {
                return Err(ParseError::custom(
                    "a combinator needs a selector on both sides",
                ));
            }
            selector
                .compounds
                .push(Compound(std::mem::take(&mut compound)));
            selector.combinators.push(Combinator::Child);
            started = false;
            whitespace = false;
            continue;
        }
        if whitespace {
            selector
                .compounds
                .push(Compound(std::mem::take(&mut compound)));
            selector.combinators.push(Combinator::Descendant);
            started = false;
            whitespace = false;
        }
        let simple = match token {
            Token::Ident(name) if !started => Some(Simple::Type(name.to_string())),
            Token::Delim('*') if !started => None,
            Token::IDHash(name) => Some(Simple::Id(name.to_string())),
            Token::Delim('.') => match input.next_including_whitespace()? {
                Token::Ident(name) => Some(Simple::Class(name.to_string())),
                _ => {
                    return Err(ParseError::custom(
                        "write a class name immediately after '.'",
                    ));
                }
            },
            Token::SquareBracketBlock => Some(input.parse_nested_block(|input| {
                let name = input.expect_ident_cloned()?.to_string();
                let value = if input.is_exhausted() {
                    None
                } else {
                    input.expect_delim('=')?;
                    Some(match input.next()? {
                        Token::Ident(value) | Token::QuotedString(value) => value.to_string(),
                        Token::Number { value, .. }
                            if compound.iter().any(|s| matches!(s, Simple::Part(_))) =>
                        {
                            value.to_string()
                        }
                        _ => {
                            return Err(ParseError::custom(
                                "quote the attribute value (numeric values are allowed on ::part)",
                            ));
                        }
                    })
                };
                Ok(Simple::Attribute { name, value })
            })?),
            Token::Colon => {
                let next = input.next_including_whitespace()?.clone();
                if matches!(next, Token::Colon) {
                    let Token::Function(name) = input.next_including_whitespace()?.clone() else {
                        return Err(ParseError::custom("write ::part(name) without spaces"));
                    };
                    if name != "part" {
                        return Err(ParseError::custom(format!(
                            "unsupported pseudo-element ::{name}; use ::part(name)"
                        )));
                    }
                    Some(Simple::Part(input.parse_nested_block(|input| {
                        Ok(input.expect_ident_cloned()?.to_string())
                    })?))
                } else {
                    match next {
                        Token::Function(name) => {
                            if !matches!(&*name, "not" | "is" | "where") {
                                return Err(ParseError::custom(format!(
                                    "unsupported pseudo-class :{name}(); :has() is not supported"
                                )));
                            }
                            let list = input
                                .parse_nested_block(|input| input.parse_comma_separated(parse))?;
                            Some(match &*name {
                                "not" => Simple::Not(list),
                                "is" => Simple::Is(list),
                                _ => Simple::Where(list),
                            })
                        }
                        Token::Ident(name) => Some(match &*name {
                            "root" => Simple::Root,
                            "hover" => Simple::State(InteractionState::Hover),
                            "active" => Simple::State(InteractionState::Active),
                            "focus" => Simple::State(InteractionState::Focus),
                            "focus-visible" => Simple::State(InteractionState::FocusVisible),
                            "focus-within" => Simple::State(InteractionState::FocusWithin),
                            "disabled" => Simple::State(InteractionState::Disabled),
                            "checked" => Simple::State(InteractionState::Checked),
                            "selected" => Simple::State(InteractionState::Selected),
                            "invalid" => Simple::State(InteractionState::Invalid),
                            "placeholder-shown" => {
                                Simple::State(InteractionState::PlaceholderShown)
                            }
                            _ => {
                                return Err(ParseError::custom(format!(
                                    "unsupported pseudo-class :{name}"
                                )));
                            }
                        }),
                        _ => return Err(ParseError::custom("a colon needs a pseudo-class name")),
                    }
                }
            }
            _ => {
                return Err(ParseError::custom(format!(
                    "unsupported selector token {token:?}"
                )));
            }
        };
        if let Some(simple) = simple {
            compound.push(simple);
        }
        started = true;
    }
    if !started {
        return Err(ParseError::custom(
            "expected a selector after the combinator",
        ));
    }
    selector.compounds.push(Compound(compound));
    selector.source = input.slice_from(start).trim().to_string();
    Ok(selector)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn functions_and_quoted_attributes_do_not_split_selector_lists() {
        let selectors = Selector::parse_list(".card:hover>.title:not([label='a, b'], :is(#x, .other)), :where(#zero) text::part(emphasis)").unwrap();
        assert_eq!(selectors.len(), 2);
        assert_eq!(selectors[0].specificity(), Specificity(1, 3, 0));
        assert_eq!(selectors[1].specificity(), Specificity(0, 0, 2));
        assert_eq!(selectors[0].combinators, vec![Combinator::Child]);
        assert_eq!(
            Selector::parse_list("* > *").unwrap()[0].specificity(),
            Specificity::default()
        );
        for bad in [
            ":has(.x)",
            ".x >",
            ".x + ~ .y",
            ".x + .y",
            ".x ~ .y",
            "text:unknown",
            "[x*=y]",
            ". x",
            ": hover",
            ": :part(x)",
            ":: part(x)",
        ] {
            assert!(Selector::parse_list(bad).is_err(), "accepted {bad}");
        }
    }
}
