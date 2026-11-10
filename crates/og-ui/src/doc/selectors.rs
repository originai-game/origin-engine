//! Parsed scoped selectors. Functional pseudo-classes contribute their argument
//! specificity (`:where` contributes zero), not punctuation counts. Sibling
//! combinators and structural pseudo-classes depend on the instantiated tree, so
//! [`Selector::structural`] selectors are matched at runtime (`super::matching`)
//! instead of being lowered to a reactive predicate here.
use super::{NONE, UiDocument, syntax::Attr};
use crate::bind::{AttrOp, AttrTest, Expr, InteractionState, Op};
use cssparser::{ParseError, Parser, Token};

/// The authored attribute as it exists in HTML, shared by compiled
/// predicates and the retained selector tree. Bound boolean attributes
/// are present with an empty value when true and absent when false.
pub(super) fn attribute_value(attr: &Attr) -> Result<Expr, super::syntax::Error> {
    if !attr.bound && !attr.name.starts_with("class:") {
        return Ok(Expr::Lit(attr.value.clone().into()));
    }
    let value = super::svelte::expression(&attr.value, attr.at)?;
    Ok(
        if matches!(
            attr.name.as_str(),
            "allowfullscreen"
                | "async"
                | "autofocus"
                | "autoplay"
                | "checked"
                | "controls"
                | "default"
                | "defer"
                | "disabled"
                | "formnovalidate"
                | "hidden"
                | "inert"
                | "ismap"
                | "loop"
                | "multiple"
                | "muted"
                | "nomodule"
                | "novalidate"
                | "open"
                | "playsinline"
                | "readonly"
                | "required"
                | "reversed"
                | "selected"
        ) {
            Expr::If(
                Box::new(value),
                Box::new(Expr::Lit(String::new().into())),
                Box::new(Expr::Lit(serde_json::Value::Null)),
            )
        } else {
            value
        },
    )
}

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
    /// `+`: the immediately preceding element sibling.
    Next,
    /// `~`: any preceding element sibling.
    Subsequent,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Simple {
    Type(String),
    Id(String),
    Class(String),
    /// `[name]`, or `[name op "value" i]` with a value test.
    Attribute {
        name: String,
        test: Option<AttrTest>,
    },
    State(InteractionState),
    Root,
    Not(Vec<Selector>),
    Is(Vec<Selector>),
    Where(Vec<Selector>),
    Part(String),
    /// `:first-child`, `:last-child`, `:only-child`, `:nth-child()` and
    /// `:nth-last-child()`: the element's 1-based index among its instantiated
    /// element siblings (those matching `of` when given) is `a·n + b` for some
    /// n ≥ 0. `:first-child` is `nth(0, 1)`; `:only-child` is both ends.
    Nth(Nth),
    /// `:only-child`, or `:only-of-type` among siblings of its own type.
    Only {
        of_type: bool,
    },
    /// `:empty`: no instantiated element children and no text.
    Empty,
    /// `:has(<relative selectors>)`: each argument starts at
    /// [`Simple::Anchor`], the element being matched.
    Has(Vec<Selector>),
    /// The `:has()` anchor in a relative selector's first compound.
    Anchor,
    /// `:lang(…)`: the nearest `lang` of the element or an ancestor, else
    /// the locale.
    Lang(Vec<String>),
    /// `:dir(rtl)` (true) or `:dir(ltr)`: the nearest `dir`, else the
    /// locale's direction.
    Dir(bool),
    /// `:state(name)`: the element's `state:name={…}` is truthy.
    Custom(String),
    /// `:defined`: every element is.
    Defined,
    /// `:scope`: the root outside `@scope`.
    Scope,
    /// A pseudo-class that matches as `:is(list)` with a pseudo-class's
    /// specificity (`:any-link`).
    Pseudo(Vec<Selector>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Nth {
    pub a: i32,
    pub b: i32,
    pub last: bool,
    /// `of S`: compound selectors that filter the counted siblings.
    pub of: Option<Vec<Selector>>,
    /// `:nth-of-type()`: count only siblings of the element's own type.
    pub of_type: bool,
}

impl Nth {
    /// Whether the 1-based `index` is `a·n + b` for some integer n ≥ 0.
    pub fn matches(&self, index: usize) -> bool {
        let (a, k) = (i64::from(self.a), index as i64 - i64::from(self.b));
        if a == 0 {
            k == 0
        } else {
            k % a == 0 && k / a >= 0
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Compound(pub Vec<Simple>);

#[derive(Clone, Debug, PartialEq)]
pub struct Selector {
    pub source: String,
    /// A global stylesheet may match across component document boundaries.
    pub global: bool,
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
                    | Simple::Root
                    | Simple::Only { .. }
                    | Simple::Empty
                    | Simple::Lang(_)
                    | Simple::Dir(_)
                    | Simple::Custom(_)
                    | Simple::Defined
                    | Simple::Scope
                    | Simple::Pseudo(_) => Specificity(0, 1, 0),
                    Simple::Anchor => Specificity::default(),
                    Simple::Nth(nth) => Specificity(0, 1, 0).add(
                        nth.of
                            .iter()
                            .flatten()
                            .map(Self::specificity)
                            .max()
                            .unwrap_or_default(),
                    ),
                    Simple::Type(_) | Simple::Part(_) => Specificity(0, 0, 1),
                    Simple::Not(list) | Simple::Is(list) | Simple::Has(list) => {
                        list.iter().map(Self::specificity).max().unwrap_or_default()
                    }
                    Simple::Where(_) => Specificity::default(),
                })
            },
        )
    }

    /// Whether matching needs the instantiated sibling order: a sibling
    /// combinator or a structural pseudo-class anywhere, including arguments.
    pub fn structural(&self) -> bool {
        self.combinators
            .iter()
            .any(|c| matches!(c, Combinator::Next | Combinator::Subsequent))
            || self
                .compounds
                .iter()
                .flat_map(|compound| &compound.0)
                .any(Simple::structural)
    }

    /// One compound, recursively, that reads only the element's own facts.
    pub(super) fn compound_only(&self) -> bool {
        self.compounds.len() == 1
            && !self.structural()
            && self.compounds[0].0.iter().all(|simple| match simple {
                Simple::Part(_) | Simple::Lang(_) | Simple::Dir(_) => false,
                Simple::Not(list)
                | Simple::Is(list)
                | Simple::Where(list)
                | Simple::Pseudo(list) => list.iter().all(Self::compound_only),
                _ => true,
            })
    }

    /// The subject compound without what needs the instantiated tree: used at
    /// cook to skip elements that can never be the subject.
    pub(super) fn subject_filter(&self) -> Self {
        let subject = self.compounds.last().expect("a selector has a compound");
        Self {
            source: self.source.clone(),
            global: self.global,
            compounds: vec![Compound(
                subject
                    .0
                    .iter()
                    .filter(|simple| !simple.structural())
                    .cloned()
                    .collect(),
            )],
            combinators: Vec::new(),
        }
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
                    Simple::Attribute { name, test } => {
                        match attrs.iter().find(|a| a.name == *name) {
                            None => Expr::Lit(false.into()),
                            Some(attr) if !attr.bound => {
                                Expr::Lit(test.as_ref().is_none_or(|t| t.test(&attr.value)).into())
                            }
                            Some(attr) => {
                                let value = attribute_value(attr).map_err(|(_, error)| error)?;
                                match test {
                                    None => Expr::Bin(
                                        Op::Ne,
                                        Box::new(value),
                                        Box::new(Expr::Lit(serde_json::Value::Null)),
                                    ),
                                    Some(test) => Expr::AttrTest(Box::new(value), test.clone()),
                                }
                            }
                        }
                    }
                    Simple::Lang(ranges) => match inherited(doc, elements, index, "lang") {
                        Some(attr) if !attr.bound => Expr::Lit(
                            ranges
                                .iter()
                                .any(|r| crate::bind::lang_matches(r, &attr.value))
                                .into(),
                        ),
                        Some(attr) => Expr::Lang(Some(Box::new(expression(attr)?)), ranges.clone()),
                        None => Expr::Lang(None, ranges.clone()),
                    },
                    Simple::Dir(rtl) => match inherited(doc, elements, index, "dir") {
                        Some(attr)
                            if !attr.bound && matches!(attr.value.as_str(), "ltr" | "rtl") =>
                        {
                            Expr::Lit(((attr.value == "rtl") == *rtl).into())
                        }
                        Some(attr) if attr.bound => {
                            Expr::Dir(Some(Box::new(expression(attr)?)), *rtl)
                        }
                        _ => Expr::Dir(None, *rtl),
                    },
                    Simple::Custom(name) => {
                        match attrs.iter().find(|a| a.name == format!("state:{name}")) {
                            Some(attr) if attr.bound => expression(attr)?,
                            Some(attr) => Expr::Lit((attr.value != "false").into()),
                            None => Expr::Lit(false.into()),
                        }
                    }
                    Simple::Defined => Expr::Lit(true.into()),
                    Simple::Scope => Expr::Lit((index == doc.root).into()),
                    Simple::State(state) => Expr::InteractionAt {
                        node: index,
                        state: *state,
                    },
                    Simple::Not(list)
                    | Simple::Is(list)
                    | Simple::Where(list)
                    | Simple::Pseudo(list) => {
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
                    // Parts are matched against a part's Facts at runtime,
                    // never against its host as an ordinary document node.
                    Simple::Part(_) => Expr::Lit(false.into()),
                    Simple::Nth(_)
                    | Simple::Only { .. }
                    | Simple::Empty
                    | Simple::Has(_)
                    | Simple::Anchor => {
                        return Err(format!(
                            "{} needs the instantiated tree; it is matched at runtime",
                            selector.source
                        ));
                    }
                };
                own = combine(Op::And, own, test);
            }
            if part == 0 {
                return Ok(own);
            }
            let combinator = selector.combinators[part - 1];
            if matches!(combinator, Combinator::Next | Combinator::Subsequent) {
                return Err(format!(
                    "{} needs the instantiated tree; it is matched at runtime",
                    selector.source
                ));
            }
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

impl Simple {
    pub(super) fn structural(&self) -> bool {
        match self {
            Simple::Nth(_) | Simple::Only { .. } | Simple::Empty | Simple::Has(_) => true,
            Simple::Not(list) | Simple::Is(list) | Simple::Where(list) | Simple::Pseudo(list) => {
                list.iter().any(Selector::structural)
            }
            _ => false,
        }
    }
}

/// The `name` attribute of the element or its nearest ancestor that has
/// one, within the document.
fn inherited<'a>(
    doc: &UiDocument,
    elements: &'a [(usize, String, Vec<Attr>)],
    mut index: usize,
    name: &str,
) -> Option<&'a Attr> {
    while index != NONE {
        if let Some((_, _, attrs)) = elements.iter().find(|(i, _, _)| *i == index)
            && let Some(attr) = attrs.iter().find(|a| a.name == name)
        {
            return Some(attr);
        }
        index = doc.nodes[index].parent;
    }
    None
}

/// HTML attributes whose values selectors compare ASCII case-insensitively
/// (HTML §4.16.2), unless the selector has the `s` flag.
const CASELESS: &[&str] = &[
    "accept",
    "accept-charset",
    "align",
    "alink",
    "axis",
    "bgcolor",
    "charset",
    "checked",
    "clear",
    "codetype",
    "color",
    "compact",
    "declare",
    "defer",
    "dir",
    "direction",
    "disabled",
    "enctype",
    "face",
    "frame",
    "hreflang",
    "http-equiv",
    "lang",
    "language",
    "link",
    "media",
    "method",
    "multiple",
    "nohref",
    "noresize",
    "noshade",
    "nowrap",
    "readonly",
    "rel",
    "rev",
    "rules",
    "scope",
    "scrolling",
    "selected",
    "shape",
    "target",
    "text",
    "type",
    "valign",
    "valuetype",
    "vlink",
];

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
    parse_selector(input, false)
}

/// A `:has()` argument: a selector that starts at the anchor, with an
/// optional leading combinator (a descendant one when it has none).
fn parse_relative(input: &mut Parser<'_>) -> Result<Selector, ParseError<String>> {
    parse_selector(input, true)
}

fn nth(a: i32, b: i32, last: bool, of_type: bool) -> Simple {
    Simple::Nth(Nth {
        a,
        b,
        last,
        of: None,
        of_type,
    })
}

pub(crate) fn state(name: &str) -> Option<InteractionState> {
    use InteractionState::*;
    Some(match name {
        "hover" => Hover,
        "active" => Active,
        "focus" => Focus,
        "focus-visible" => FocusVisible,
        "focus-within" => FocusWithin,
        "disabled" => Disabled,
        "enabled" => Enabled,
        "checked" => Checked,
        "selected" => Selected,
        "invalid" => Invalid,
        "valid" => Valid,
        "user-invalid" => UserInvalid,
        "placeholder-shown" => PlaceholderShown,
        "required" => Required,
        "optional" => Optional,
        "read-only" => ReadOnly,
        "read-write" => ReadWrite,
        "in-range" => InRange,
        "out-of-range" => OutOfRange,
        "indeterminate" => Indeterminate,
        "default" => Default,
        "open" => Open,
        "popover-open" => PopoverOpen,
        "modal" => Modal,
        "target" => Target,
        _ => return None,
    })
}

fn attribute(input: &mut Parser<'_>, on_part: bool) -> Result<Simple, ParseError<String>> {
    let name = input.expect_ident_cloned()?.to_string();
    if input.is_exhausted() {
        return Ok(Simple::Attribute { name, test: None });
    }
    let op = match input.next()? {
        Token::Delim('=') => AttrOp::Equals,
        Token::IncludeMatch => AttrOp::Includes,
        Token::DashMatch => AttrOp::Dash,
        Token::PrefixMatch => AttrOp::Prefix,
        Token::SuffixMatch => AttrOp::Suffix,
        Token::SubstringMatch => AttrOp::Substring,
        token => {
            return Err(ParseError::custom(format!(
                "expected =, ~=, |=, ^=, $= or *= after [{name}, found {token:?}"
            )));
        }
    };
    let value = match input.next()? {
        Token::Ident(value) | Token::QuotedString(value) => value.to_string(),
        Token::Number { value, .. } if on_part => value.to_string(),
        _ => {
            return Err(ParseError::custom(
                "quote the attribute value (numeric values are allowed on ::part)",
            ));
        }
    };
    let insensitive = match input.try_parse(|p| p.expect_ident_cloned()) {
        Ok(flag) if flag.eq_ignore_ascii_case("i") => true,
        Ok(flag) if flag.eq_ignore_ascii_case("s") => false,
        Ok(flag) => {
            return Err(ParseError::custom(format!(
                "unknown attribute selector flag {flag}; use i or s"
            )));
        }
        Err(_) => CASELESS.contains(&name.to_ascii_lowercase().as_str()),
    };
    input.expect_exhausted()?;
    Ok(Simple::Attribute {
        name,
        test: Some(AttrTest {
            op,
            value,
            insensitive,
        }),
    })
}

fn pseudo_function(
    name: &str,
    input: &mut Parser<'_>,
    relative: bool,
) -> Result<Simple, ParseError<String>> {
    match name {
        "nth-child" | "nth-last-child" => {
            let last = name == "nth-last-child";
            Ok(Simple::Nth(input.parse_nested_block(|input| {
                let (a, b) = cssparser::parse_nth(input)?;
                let of = if input.is_exhausted() {
                    None
                } else {
                    input.expect_ident_matching("of")?;
                    let list = input.parse_comma_separated(parse)?;
                    if let Some(bad) = list.iter().find(|s| !s.compound_only()) {
                        return Err(ParseError::custom(format!(
                            "`of {}` in :{name}() takes compound selectors without combinators or structural pseudo-classes",
                            bad.source
                        )));
                    }
                    Some(list)
                };
                Ok(Nth {
                    a,
                    b,
                    last,
                    of,
                    of_type: false,
                })
            })?))
        }
        "nth-of-type" | "nth-last-of-type" => {
            let (a, b) = input.parse_nested_block(|input| Ok(cssparser::parse_nth(input)?))?;
            Ok(nth(a, b, name == "nth-last-of-type", true))
        }
        "has" => {
            if relative {
                return Err(ParseError::custom(":has() cannot nest inside :has()"));
            }
            let list =
                input.parse_nested_block(|input| input.parse_comma_separated(parse_relative))?;
            Ok(Simple::Has(list))
        }
        "lang" => {
            let ranges = input.parse_nested_block(|input| {
                input.parse_comma_separated(|input| {
                    Ok(match input.next()? {
                        Token::Ident(v) | Token::QuotedString(v) => v.to_string(),
                        _ => return Err(ParseError::custom(":lang() takes language ranges")),
                    })
                })
            })?;
            Ok(Simple::Lang(ranges))
        }
        "dir" => {
            let word = input.parse_nested_block(|input| {
                Ok(input.expect_ident_cloned()?.to_ascii_lowercase())
            })?;
            match word.as_str() {
                "rtl" => Ok(Simple::Dir(true)),
                "ltr" => Ok(Simple::Dir(false)),
                _ => Err(ParseError::custom(format!(
                    ":dir() takes ltr or rtl, not {word}"
                ))),
            }
        }
        "state" => Ok(Simple::Custom(input.parse_nested_block(|input| {
            Ok(input.expect_ident_cloned()?.to_string())
        })?)),
        "not" | "is" | "where" => {
            let list = input.parse_nested_block(|input| {
                input.parse_comma_separated(|input| parse_selector(input, false))
            })?;
            if relative
                && list
                    .iter()
                    .flat_map(|s| s.compounds.iter().flat_map(|c| &c.0))
                    .any(|s| matches!(s, Simple::Has(_)))
            {
                return Err(ParseError::custom(":has() cannot nest inside :has()"));
            }
            Ok(match name {
                "not" => Simple::Not(list),
                "is" => Simple::Is(list),
                _ => Simple::Where(list),
            })
        }
        _ => Err(ParseError::custom(format!(
            "unsupported pseudo-class :{name}(); supported functions are :not(), :is(), :where(), :has(), :nth-child(), :nth-last-child(), :nth-of-type(), :nth-last-of-type(), :lang(), :dir() and :state()"
        ))),
    }
}

fn pseudo_class(name: &str) -> Result<Simple, ParseError<String>> {
    if let Some(state) = state(name) {
        return Ok(Simple::State(state));
    }
    Ok(match name {
        "root" => Simple::Root,
        "scope" => Simple::Scope,
        "defined" => Simple::Defined,
        "first-child" => nth(0, 1, false, false),
        "last-child" => nth(0, 1, true, false),
        "only-child" => Simple::Only { of_type: false },
        "first-of-type" => nth(0, 1, false, true),
        "last-of-type" => nth(0, 1, true, true),
        "only-of-type" => Simple::Only { of_type: true },
        "empty" => Simple::Empty,
        "any-link" | "link" => {
            Simple::Pseudo(Selector::parse_list("a[href], area[href]").map_err(ParseError::custom)?)
        }
        _ => {
            return Err(ParseError::custom(format!(
                "unsupported pseudo-class :{name}; see the selector list in docs/ui.md §2.4"
            )));
        }
    })
}

fn parse_selector(input: &mut Parser<'_>, relative: bool) -> Result<Selector, ParseError<String>> {
    let start = input.position();
    let mut selector = Selector {
        source: String::new(),
        global: false,
        compounds: Vec::new(),
        combinators: Vec::new(),
    };
    let mut compound = Vec::new();
    let mut started = false;
    let mut whitespace = false;
    if relative {
        compound.push(Simple::Anchor);
        let leading = input.try_parse(|input| {
            input.skip_whitespace();
            match input.next()? {
                Token::Delim(c @ ('>' | '+' | '~')) => Ok(*c),
                _ => Err(ParseError::<String>::custom("no combinator")),
            }
        });
        selector
            .compounds
            .push(Compound(std::mem::take(&mut compound)));
        selector.combinators.push(match leading {
            Ok('>') => Combinator::Child,
            Ok('+') => Combinator::Next,
            Ok('~') => Combinator::Subsequent,
            _ => Combinator::Descendant,
        });
        started = false;
        input.skip_whitespace();
    }
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
            if !started {
                return Err(ParseError::custom(
                    "a combinator needs a selector on both sides",
                ));
            }
            selector
                .compounds
                .push(Compound(std::mem::take(&mut compound)));
            selector.combinators.push(match c {
                '>' => Combinator::Child,
                '+' => Combinator::Next,
                _ => Combinator::Subsequent,
            });
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
            Token::SquareBracketBlock => {
                let on_part = compound.iter().any(|s| matches!(s, Simple::Part(_)));
                Some(input.parse_nested_block(|input| attribute(input, on_part))?)
            }
            Token::Colon => {
                let next = input.next_including_whitespace()?.clone();
                if matches!(next, Token::Colon) {
                    if relative {
                        return Err(ParseError::custom(":has() takes no pseudo-elements"));
                    }
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
                        Token::Function(name) => Some(pseudo_function(
                            &name.to_ascii_lowercase(),
                            input,
                            relative,
                        )?),
                        Token::Ident(name) => Some(pseudo_class(&name.to_ascii_lowercase())?),
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
        let structural = Selector::parse_list(
            ".row:nth-child(2n+1 of .shown, [kind='a']) + .row ~ :not(:last-child) > text:empty",
        )
        .unwrap();
        assert!(structural[0].structural());
        assert_eq!(
            structural[0].combinators,
            vec![Combinator::Next, Combinator::Subsequent, Combinator::Child]
        );
        assert_eq!(structural[0].specificity(), Specificity(0, 6, 1));
        let Simple::Nth(nth) = &structural[0].compounds[0].0[1] else {
            panic!("nth-child")
        };
        assert_eq!((nth.a, nth.b, nth.last), (2, 1, false));
        assert_eq!(nth.of.as_ref().unwrap().len(), 2);
        assert!(!selectors[0].structural());
        for (text, index, expected) in [
            (":nth-child(2n+1)", 5, true),
            (":nth-child(2n+1)", 4, false),
            (":nth-child(-n+3)", 3, true),
            (":nth-child(-n+3)", 4, false),
            (":nth-last-child(3)", 3, true),
            (":first-child", 1, true),
            (":first-child", 2, false),
            (":nth-child(even)", 2, true),
            (":nth-child(n+4)", 3, false),
        ] {
            let Simple::Nth(nth) = &Selector::parse_list(text).unwrap()[0].compounds[0].0[0] else {
                panic!("{text}")
            };
            assert_eq!(nth.matches(index), expected, "{text} at {index}");
        }
        let has = &Selector::parse_list(".card:has(> .x + .y, .z)").unwrap()[0];
        assert!(has.structural());
        assert_eq!(has.specificity(), Specificity(0, 3, 0));
        let Simple::Has(relative) = &has.compounds[0].0[1] else {
            panic!("has")
        };
        assert_eq!(
            relative[0].combinators,
            vec![Combinator::Child, Combinator::Next]
        );
        assert_eq!(relative[1].combinators, vec![Combinator::Descendant]);
        for (text, actual, expected) in [
            ("[a~='b']", "x b y", true),
            ("[a~='b']", "xb", false),
            ("[a|='en']", "en-GB", true),
            ("[a|='en']", "english", false),
            ("[a^='pre']", "prefix", true),
            ("[a$='fix']", "prefix", true),
            ("[a*='efi']", "prefix", true),
            ("[a*='']", "prefix", false),
            ("[a='X' i]", "x", true),
            ("[a='X']", "x", false),
            ("[type='TEXT']", "text", true),
            ("[type='TEXT' s]", "text", false),
        ] {
            let Simple::Attribute {
                test: Some(test), ..
            } = &Selector::parse_list(text).unwrap()[0].compounds[0].0[0]
            else {
                panic!("{text}")
            };
            assert_eq!(test.test(actual), expected, "{text} on {actual}");
        }
        for bad in [
            ":has(:has(.x))",
            ":has(::part(x))",
            "[a=b x]",
            ".x >",
            ".x + ~ .y",
            ".x +",
            "~ .y",
            ":nth-child(2n of .a .b)",
            ":nth-child(odd of :first-child)",
            ":nth-child(odd of :is(.a .b))",
            ":nth-child(x)",
            "text:unknown",
            ":dir(up)",
            ". x",
            ": hover",
            ": :part(x)",
            ":: part(x)",
        ] {
            assert!(Selector::parse_list(bad).is_err(), "accepted {bad}");
        }
    }
}
