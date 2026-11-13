//! Generated content (CSS Generated Content 3, CSS Lists 3 counters, CSS
//! Counter Styles 3): `content`, `quotes`, `counter-reset`,
//! `counter-increment`, `counter-set`, `counter()` and `counters()`.
//!
//! Counters and quote depth follow document order. While a tick builds, a
//! [`Run`] goes through each element: its counter properties at its start,
//! then its `::before`, its children and its `::after`. A retained subtree
//! that is not built again replays the [`Event`]s it recorded, so an idle
//! frame touches no element. A subtree whose generated text reads the run
//! is built again only when the run it enters differs from last time.
use super::at_rules::{CounterStyle, Sheet, System};
use cssparser::{Parser, Token};
use std::collections::BTreeMap;
use std::sync::Arc;

/// The element-backed pseudo-elements a rule may select, as written after
/// `::`. CSS 2's single-colon spellings of the first four are accepted too.
pub const PSEUDOS: [&str; 8] = [
    "before",
    "after",
    "marker",
    "first-letter",
    "first-line",
    "placeholder",
    "selection",
    "file-selector-button",
];

/// The text properties generated text and the text pseudo-elements take,
/// as a styled run of their text ([`crate::doc::spans`]).
const TEXT: [&str; 12] = [
    "color",
    "font",
    "font-family",
    "font-size",
    "font-weight",
    "font-style",
    "line-height",
    "letter-spacing",
    "text-decoration",
    "text-decoration-line",
    "-og-text-motion",
    "display",
];

/// Whether `property` styles `pseudo` (an entry of [`PSEUDOS`]).
pub fn allows(pseudo: &str, property: &str) -> bool {
    let text = TEXT.contains(&property)
        && !(property == "display" && pseudo != "before" && pseudo != "after");
    match pseudo {
        "before" | "after" => {
            text || matches!(
                property,
                "content" | "quotes" | "counter-reset" | "counter-increment" | "counter-set"
            )
        }
        "marker" => text || property == "content",
        "first-letter" | "first-line" => text,
        "placeholder" => property == "color",
        "selection" => matches!(
            property,
            "background-color" | "background" | "color" | "text-decoration" | "text-shadow"
        ),
        // Its control does not exist (a file input needs a host picker), so
        // any property is accepted and styles nothing.
        _ => true,
    }
}

/// The declaration name a `::pseudo { property }` rule becomes on its
/// originating element (an identifier, so it parses as a declaration).
pub fn encode(pseudo: &str, property: &str) -> String {
    format!("-og-pe-{pseudo}__{property}")
}

/// `(pseudo, property)` of an [`encode`]d name.
pub fn decode(name: &str) -> Option<(&str, &str)> {
    name.strip_prefix("-og-pe-")?.split_once("__")
}

/// The element properties of generated content that cascade per element
/// and leave the declarations [`super::css`] maps.
pub fn longhand(name: &str) -> bool {
    decode(name).is_some()
        || matches!(
            name,
            "counter-reset" | "counter-increment" | "counter-set" | "quotes"
        )
}

/// Checks a generated-content declaration as written.
pub fn check(name: &str, value: &str) -> Result<(), String> {
    let (pseudo, property) = match decode(name) {
        Some((pseudo, property)) => (Some(pseudo), property),
        None => (None, name),
    };
    match property {
        "content" => parse_content(value).map(|_| ()),
        "quotes" => parse_quotes(value).map(|_| ()),
        "counter-reset" | "counter-increment" | "counter-set" => {
            parse_ops(property, value).map(|_| ())
        }
        _ if pseudo == Some("file-selector-button") => Ok(()),
        "text-decoration" if pseudo == Some("selection") => selection_decoration(value).map(|_| ()),
        "text-shadow" if pseudo == Some("selection") => {
            super::paint_css::text_shadows(value, crate::paint::ColorRef::token("ink")).map(|_| ())
        }
        // The text properties check as a span's do, when the element
        // resolves; colours check here.
        "color" | "background-color" | "background" => super::css::declarations(
            &format!(
                "{}: {value};",
                if property == "color" {
                    "color"
                } else {
                    "background"
                }
            ),
            0,
        )
        .map(|_| ()),
        _ => Ok(()),
    }
}

/// `::selection`'s style as the draw paints it: the highlight under the
/// selected text, and the selected glyphs' ink, decoration lines and
/// shadows (`None`: the text's own).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Selection {
    pub background: Option<crate::style::Srgba>,
    pub color: Option<crate::style::Srgba>,
    pub decoration: Option<super::text_css::Decoration>,
    /// `(offset, blur, colour)` each; `Some(empty)` for `text-shadow: none`.
    pub shadows: Option<Vec<([f32; 2], f32, crate::style::Srgba)>>,
}

/// `::selection`'s `text-decoration`: its lines, style and thickness, and
/// its colour as written.
pub fn selection_decoration(
    value: &str,
) -> Result<(super::text_css::Decoration, Option<String>), String> {
    use super::text_css::DecorationStyle as S;
    let mut d = super::text_css::Decoration::default();
    let mut color = None;
    for word in super::text_css::split_top(value) {
        match word.to_ascii_lowercase().as_str() {
            "none" => {}
            "underline" => d.underline = true,
            "overline" => d.overline = true,
            "line-through" => d.through = true,
            "solid" => d.style = S::Solid,
            "double" => d.style = S::Double,
            "dotted" => d.style = S::Dotted,
            "dashed" => d.style = S::Dashed,
            "wavy" => d.style = S::Wavy,
            w => match w.strip_suffix("px").and_then(|n| n.parse::<f32>().ok()) {
                Some(px) => d.thickness = Some(px),
                None => {
                    crate::css_color::parse_str(&word).map_err(|_| {
                        format!("::selection text-decoration: '{word}' is not a value")
                    })?;
                    color = Some(word);
                }
            },
        }
    }
    Ok((d, color))
}

/// One element's generated content as its cascade computed it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Generated {
    pub ops: Arc<Ops>,
    /// `::before` and `::after`: their content, counter properties and
    /// text style declarations (`name: value;`).
    pub before: Option<PseudoBox>,
    pub after: Option<PseudoBox>,
    /// `::marker`'s content (`Normal`: the list style's) and style.
    pub marker: Option<PseudoBox>,
    pub first_letter: Option<String>,
    pub first_line: Option<String>,
    pub placeholder: Option<String>,
    pub selection: Option<String>,
}

/// A pseudo-element's computed content and text style.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PseudoBox {
    pub content: Content,
    pub ops: Arc<Ops>,
    pub style: String,
}

impl Generated {
    /// From the cascade's generated-content winners, substituted.
    pub fn from_values(values: &[(String, String)], diagnostics: &mut Vec<String>) -> Self {
        let mut out = Self::default();
        let mut ops = Ops::default();
        let mut boxes: BTreeMap<&str, (PseudoBox, Ops)> = BTreeMap::new();
        let mut styles: BTreeMap<&str, String> = BTreeMap::new();
        for (name, value) in values {
            let (pseudo, property) = match decode(name) {
                Some((pseudo, property)) => (Some(pseudo), property),
                None => (None, name.as_str()),
            };
            let parsed: Result<(), String> = (|| {
                match (pseudo, property) {
                    (_, "quotes") => {}
                    (p, "counter-reset" | "counter-increment" | "counter-set") => {
                        let list = parse_ops(property, value)?;
                        let target = match p {
                            Some(p) => {
                                &mut boxes
                                    .entry(
                                        PSEUDOS
                                            .iter()
                                            .find(|x| **x == p)
                                            .copied()
                                            .unwrap_or("before"),
                                    )
                                    .or_default()
                                    .1
                            }
                            None => &mut ops,
                        };
                        match property {
                            "counter-reset" => target.reset = list,
                            "counter-increment" => target.increment = list,
                            _ => target.set = list,
                        }
                    }
                    (Some(p), "content") => {
                        let p = PSEUDOS
                            .iter()
                            .find(|x| **x == p)
                            .copied()
                            .unwrap_or("before");
                        boxes.entry(p).or_default().0.content = parse_content(value)?;
                    }
                    (Some(p), property) => {
                        let p = PSEUDOS
                            .iter()
                            .find(|x| **x == p)
                            .copied()
                            .unwrap_or("before");
                        styles
                            .entry(p)
                            .or_default()
                            .push_str(&format!("{property}: {value};"));
                    }
                    (None, _) => {}
                }
                Ok(())
            })();
            if let Err(error) = parsed {
                diagnostics.push(format!("{name} invalid at computed-value time: {error}"));
            }
        }
        out.ops = Arc::new(ops);
        let mut take = |p: &str| {
            let (mut b, ops) = boxes.remove(p).unwrap_or_default();
            b.ops = Arc::new(ops);
            b.style = styles.remove(p).unwrap_or_default();
            (b.content != Content::Normal || !b.style.is_empty() || !b.ops.is_empty()).then_some(b)
        };
        out.before = take("before");
        out.after = take("after");
        out.marker = take("marker");
        out.first_letter = styles.remove("first-letter");
        out.first_line = styles.remove("first-line");
        out.placeholder = styles.remove("placeholder");
        out.selection = styles.remove("selection");
        out
    }

    /// Whether building it reads or moves the counters or quotes.
    pub fn uses_run(&self) -> bool {
        !self.ops.is_empty()
            || [&self.before, &self.after, &self.marker]
                .into_iter()
                .flatten()
                .any(|b| !b.ops.is_empty() || b.content.reads_run())
    }
}

/// One part of a `content` value.
#[derive(Clone, Debug, PartialEq)]
pub enum Piece {
    Text(String),
    /// `attr(name)`: the originating element's attribute, or "".
    Attr(String),
    /// `counter(name, style)`.
    Counter(String, String),
    /// `counters(name, "sep", style)`.
    Counters(String, String, String),
    OpenQuote,
    CloseQuote,
    NoOpenQuote,
    NoCloseQuote,
    /// `url(…)` or `image(…)`: a picture the game registered.
    Image(String),
}

/// A computed `content` value.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Content {
    /// `normal`: none on `::before`/`::after`, the marker on `::marker`.
    #[default]
    Normal,
    None,
    Pieces(Vec<Piece>),
}

impl Content {
    /// Whether evaluating it reads the counters or the quote depth.
    pub fn reads_run(&self) -> bool {
        matches!(self, Self::Pieces(p) if p.iter().any(|p| !matches!(p, Piece::Text(_) | Piece::Attr(_) | Piece::Image(_))))
    }
}

fn unexpected<T>(what: &str, value: &str) -> Result<T, String> {
    Err(format!("{what}: cannot read `{}`", value.trim()))
}

/// A counter style name as written (`decimal` when omitted).
fn style_name(p: &mut Parser<'_>) -> Result<String, ()> {
    if p.try_parse(|p| p.expect_comma()).is_ok() {
        Ok(p.expect_ident_cloned().map_err(|_| ())?.to_string())
    } else {
        Ok("decimal".into())
    }
}

/// Parses `content`: `normal | none | [<string> | <counter> | <quote> |
/// attr() | <image>]+ [/ <string>+]?` (the alternative text is for
/// assistive technology and is dropped).
pub fn parse_content(value: &str) -> Result<Content, String> {
    let trimmed = value.trim();
    match trimmed.to_ascii_lowercase().as_str() {
        "normal" => return Ok(Content::Normal),
        "none" => return Ok(Content::None),
        _ => {}
    }
    let mut p = Parser::new(trimmed);
    let mut pieces = Vec::new();
    while !p.is_exhausted() {
        let token = p
            .next()
            .map_err(|_| "content: unexpected end".to_string())?
            .clone();
        let piece = match token {
            Token::QuotedString(s) => Piece::Text(s.to_string()),
            Token::Delim('/') => break,
            Token::Ident(w) => match w.to_ascii_lowercase().as_str() {
                "open-quote" => Piece::OpenQuote,
                "close-quote" => Piece::CloseQuote,
                "no-open-quote" => Piece::NoOpenQuote,
                "no-close-quote" => Piece::NoCloseQuote,
                _ => return unexpected("content", value),
            },
            Token::UnquotedUrl(u) => Piece::Image(u.to_string()),
            Token::Function(f) => {
                let f = f.to_ascii_lowercase();
                let piece: Result<Piece, cssparser::ParseError<()>> = p.parse_nested_block(|p| {
                    Ok(match f.as_str() {
                        "counter" => {
                            let name = p.expect_ident_cloned()?.to_string();
                            let style =
                                style_name(p).map_err(|_| cssparser::ParseError::custom(()))?;
                            Piece::Counter(name, style)
                        }
                        "counters" => {
                            let name = p.expect_ident_cloned()?.to_string();
                            p.expect_comma()?;
                            let sep = p.expect_string_cloned()?.to_string();
                            let style =
                                style_name(p).map_err(|_| cssparser::ParseError::custom(()))?;
                            Piece::Counters(name, sep, style)
                        }
                        "attr" => Piece::Attr(p.expect_ident_cloned()?.to_string()),
                        "url" | "image" => Piece::Image(p.expect_string_cloned()?.to_string()),
                        _ => return Err(cssparser::ParseError::custom(())),
                    })
                });
                piece.or_else(|_| unexpected("content", value))?
            }
            _ => return unexpected("content", value),
        };
        pieces.push(piece);
    }
    if pieces.is_empty() {
        return unexpected("content", value);
    }
    Ok(Content::Pieces(pieces))
}

/// One element's (or pseudo-element's) counter properties.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ops {
    pub reset: Vec<(String, i64)>,
    pub increment: Vec<(String, i64)>,
    pub set: Vec<(String, i64)>,
}

impl Ops {
    pub fn is_empty(&self) -> bool {
        self.reset.is_empty() && self.increment.is_empty() && self.set.is_empty()
    }
}

/// Parses `counter-reset`, `counter-increment` or `counter-set`: `none |
/// [<name> <integer>?]+`, the integer defaulting to 1 for an increment and
/// 0 otherwise. `reversed(name)` resets to 0 here (a reversed list counts
/// down from its item count, which documents give with `ol reversed`).
pub fn parse_ops(property: &str, value: &str) -> Result<Vec<(String, i64)>, String> {
    let default = i64::from(property == "counter-increment");
    if value.trim().eq_ignore_ascii_case("none") {
        return Ok(Vec::new());
    }
    let mut p = Parser::new(value);
    let mut out: Vec<(String, i64)> = Vec::new();
    while !p.is_exhausted() {
        let name = match p
            .next()
            .map_err(|_| format!("{property}: unexpected end"))?
            .clone()
        {
            Token::Ident(n) if !matches!(n.to_ascii_lowercase().as_str(), "none") => n.to_string(),
            Token::Function(f)
                if f.eq_ignore_ascii_case("reversed") && property == "counter-reset" =>
            {
                let r: Result<String, cssparser::ParseError<()>> =
                    p.parse_nested_block(|p| Ok(p.expect_ident_cloned()?.to_string()));
                r.or_else(|_| unexpected(property, value))?
            }
            _ => return unexpected(property, value),
        };
        let n = p
            .try_parse(|p| p.expect_integer())
            .map(i64::from)
            .unwrap_or(default);
        out.push((name, n));
    }
    Ok(out)
}

/// A computed `quotes` value: pairs by nesting level.
#[derive(Clone, Debug, PartialEq)]
pub enum Quotes {
    /// `auto`: the typographic quotes of the content language (English
    /// curly quotes, then single ones).
    Auto,
    None,
    Pairs(Vec<(String, String)>),
}

pub fn parse_quotes(value: &str) -> Result<Quotes, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" | "match-parent" => return Ok(Quotes::Auto),
        "none" => return Ok(Quotes::None),
        _ => {}
    }
    let mut p = Parser::new(value);
    let mut strings = Vec::new();
    while !p.is_exhausted() {
        match p.next() {
            Ok(Token::QuotedString(s)) => strings.push(s.to_string()),
            _ => return unexpected("quotes", value),
        }
    }
    if strings.is_empty() || strings.len() % 2 != 0 {
        return Err("quotes takes auto, none or pairs of strings".into());
    }
    Ok(Quotes::Pairs(
        strings
            .chunks(2)
            .map(|c| (c[0].clone(), c[1].clone()))
            .collect(),
    ))
}

impl Quotes {
    fn pair(&self, depth: usize) -> (String, String) {
        match self {
            Self::None => (String::new(), String::new()),
            Self::Auto => {
                if depth == 0 {
                    ("\u{201C}".into(), "\u{201D}".into())
                } else {
                    ("\u{2018}".into(), "\u{2019}".into())
                }
            }
            Self::Pairs(p) => p[depth.min(p.len() - 1)].clone(),
        }
    }
}

/// A change to the run, as an element applies it; replayed for a subtree
/// that is not built again.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Enter,
    /// Enters an element with style containment.
    EnterContained,
    Exit,
    Ops(Arc<Ops>),
    /// The quote depth moves by this.
    Quote(i32),
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Contained {
    level: u32,
    counters: BTreeMap<String, Vec<(i64, u32)>>,
    quote_depth: u32,
}

/// Counters in scope and the quote depth, in document order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Run {
    /// Each counter's instances, outermost first: (value, the nesting
    /// level of the element whose siblings it is scoped to).
    counters: BTreeMap<String, Vec<(i64, u32)>>,
    quote_depth: u32,
    level: u32,
    /// The style-contained elements entered (CSS Containment 2 §3.3), each
    /// with its level, the counters and quote depth before it, and how
    /// many instances each counter had then: what its subtree changes is
    /// undone when it is left, and an increment or set inside it makes a
    /// counter of its own rather than move one from outside.
    contained: Vec<Contained>,
    /// What happened, in order, for [`Run::since`].
    log: Vec<Event>,
    /// How many generated texts read the run so far this tick.
    pub reads: u64,
}

impl Run {
    /// The run's state without its log, to compare with a later one.
    pub fn snapshot(&self) -> Arc<Run> {
        Arc::new(Run {
            log: Vec::new(),
            ..self.clone()
        })
    }

    pub fn same_state(&self, other: &Run) -> bool {
        self.counters == other.counters
            && self.quote_depth == other.quote_depth
            && self.level == other.level
            && self.contained == other.contained
    }

    /// Where the log stands, to take a subtree's events with [`Run::since`].
    pub fn mark(&self) -> usize {
        self.log.len()
    }

    pub fn since(&self, mark: usize) -> Arc<Vec<Event>> {
        Arc::new(self.log[mark..].to_vec())
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn enter(&mut self) {
        self.level += 1;
        self.log.push(Event::Enter);
    }

    /// Enters an element with style containment (`contain: style`).
    pub fn enter_contained(&mut self) {
        self.level += 1;
        self.contained.push(Contained {
            level: self.level,
            counters: self.counters.clone(),
            quote_depth: self.quote_depth,
        });
        self.log.push(Event::EnterContained);
    }

    /// Leaves an element: the counters its children reset go out of scope;
    /// a style-contained one's subtree leaves nothing changed behind it.
    pub fn exit(&mut self) {
        let level = self.level;
        if let Some(c) = self.contained.pop_if(|c| c.level == level) {
            self.counters = c.counters;
            self.quote_depth = c.quote_depth;
            self.level = self.level.saturating_sub(1);
            self.log.push(Event::Exit);
            return;
        }
        let inner = self.level + 1;
        self.counters.retain(|_, stack| {
            stack.retain(|&(_, level)| level < inner);
            !stack.is_empty()
        });
        self.level = self.level.saturating_sub(1);
        self.log.push(Event::Exit);
    }

    /// Applies an element's counter properties: resets, then increments,
    /// then sets (CSS Lists 3 §4.5). A counter used but not in scope is
    /// instantiated at the element.
    pub fn apply(&mut self, ops: &Arc<Ops>) {
        if ops.is_empty() {
            return;
        }
        let level = self.level;
        for (name, n) in &ops.reset {
            let outside = self.outside(name);
            let stack = self.counters.entry(name.clone()).or_default();
            // A sibling's instance is replaced, not nested (one outside a
            // style-contained element is kept).
            if stack.len() > outside && stack.last().is_some_and(|&(_, l)| l == level) {
                stack.pop();
            }
            stack.push((*n, level));
        }
        for (name, n) in &ops.increment {
            self.instance(name).0 += n;
        }
        for (name, n) in &ops.set {
            self.instance(name).0 = *n;
        }
        self.log.push(Event::Ops(ops.clone()));
    }

    /// How many of counter `name`'s instances are outside the innermost
    /// style-contained element entered (0: none is).
    fn outside(&self, name: &str) -> usize {
        self.contained
            .last()
            .and_then(|c| c.counters.get(name))
            .map_or(0, Vec::len)
    }

    fn instance(&mut self, name: &str) -> &mut (i64, u32) {
        let level = self.level;
        let outside = self.outside(name);
        let stack = self.counters.entry(name.to_string()).or_default();
        if stack.len() <= outside {
            stack.push((0, level));
        }
        stack.last_mut().unwrap()
    }

    fn quote(&mut self, delta: i32) {
        self.quote_depth = self.quote_depth.saturating_add_signed(delta);
        self.log.push(Event::Quote(delta));
    }

    /// Replays a subtree's recorded events.
    pub fn replay(&mut self, events: &[Event]) {
        for e in events {
            match e {
                Event::Enter => self.enter(),
                Event::EnterContained => self.enter_contained(),
                Event::Exit => self.exit(),
                Event::Ops(ops) => self.apply(ops),
                Event::Quote(d) => self.quote(*d),
            }
        }
    }

    /// The text `content` generates here, moving the quote depth.
    /// `list_item` is an item's number for the implicit `list-item`
    /// counter when no author counter of that name is in scope.
    pub fn evaluate(
        &mut self,
        pieces: &[Piece],
        quotes: &Quotes,
        sheet: &Sheet,
        attr: &dyn Fn(&str) -> Option<String>,
        list_item: Option<i64>,
    ) -> String {
        self.reads += 1;
        let mut out = String::new();
        for piece in pieces {
            match piece {
                Piece::Text(t) => out.push_str(t),
                Piece::Attr(name) => out.push_str(&attr(name).unwrap_or_default()),
                Piece::Image(_) => {}
                Piece::Counter(name, style) => {
                    let n = match (self.counters.get(name), list_item) {
                        (Some(stack), _) => stack.last().map_or(0, |c| c.0),
                        (None, Some(n)) if name == "list-item" => n,
                        _ => 0,
                    };
                    out.push_str(&format(n, style, sheet));
                }
                Piece::Counters(name, sep, style) => {
                    let values: Vec<i64> = match (self.counters.get(name), list_item) {
                        (Some(stack), _) => stack.iter().map(|c| c.0).collect(),
                        (None, Some(n)) if name == "list-item" => vec![n],
                        _ => vec![0],
                    };
                    let parts: Vec<String> =
                        values.iter().map(|&n| format(n, style, sheet)).collect();
                    out.push_str(&parts.join(sep));
                }
                Piece::OpenQuote => {
                    out.push_str(&quotes.pair(self.quote_depth as usize).0);
                    self.quote(1);
                }
                Piece::CloseQuote => {
                    if self.quote_depth > 0 {
                        out.push_str(&quotes.pair(self.quote_depth as usize - 1).1);
                        self.quote(-1);
                    }
                }
                Piece::NoOpenQuote => self.quote(1),
                Piece::NoCloseQuote => {
                    if self.quote_depth > 0 {
                        self.quote(-1);
                    }
                }
            }
        }
        out
    }
}

/// The predefined counter styles (CSS Counter Styles 3 §6), as
/// `@counter-style` would declare them.
fn predefined(name: &str) -> Option<CounterStyle> {
    let chars = |s: &str| s.chars().map(String::from).collect::<Vec<_>>();
    let style = |system, symbols: Vec<String>, suffix: &str| CounterStyle {
        system,
        symbols,
        additive: Vec::new(),
        negative: ("-".into(), String::new()),
        prefix: String::new(),
        suffix: suffix.into(),
        range: Vec::new(),
        pad: None,
        fallback: "decimal".into(),
    };
    let roman = |upper: bool| {
        let table = [
            (1000, "M"),
            (900, "CM"),
            (500, "D"),
            (400, "CD"),
            (100, "C"),
            (90, "XC"),
            (50, "L"),
            (40, "XL"),
            (10, "X"),
            (9, "IX"),
            (5, "V"),
            (4, "IV"),
            (1, "I"),
        ];
        CounterStyle {
            additive: table
                .iter()
                .map(|&(w, s)| (w, if upper { s.into() } else { s.to_lowercase() }))
                .collect(),
            range: vec![(1, 3999)],
            ..style(System::Additive, Vec::new(), ". ")
        }
    };
    Some(match name {
        "decimal" => style(System::Numeric, chars("0123456789"), ". "),
        "decimal-leading-zero" => CounterStyle {
            pad: Some((2, "0".into())),
            ..style(System::Numeric, chars("0123456789"), ". ")
        },
        "arabic-indic" => style(System::Numeric, chars("٠١٢٣٤٥٦٧٨٩"), ". "),
        "cjk-decimal" => CounterStyle {
            range: vec![(0, i64::MAX)],
            ..style(System::Numeric, chars("〇一二三四五六七八九"), "、")
        },
        "lower-roman" => roman(false),
        "upper-roman" => roman(true),
        "lower-alpha" | "lower-latin" => style(
            System::Alphabetic,
            chars("abcdefghijklmnopqrstuvwxyz"),
            ". ",
        ),
        "upper-alpha" | "upper-latin" => style(
            System::Alphabetic,
            chars("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
            ". ",
        ),
        "lower-greek" => style(System::Alphabetic, chars("αβγδεζηθικλμνξοπρστυφχψω"), ". "),
        "hiragana" => style(
            System::Alphabetic,
            chars(
                "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわゐゑをん",
            ),
            "、",
        ),
        "katakana" => style(
            System::Alphabetic,
            chars(
                "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヰヱヲン",
            ),
            "、",
        ),
        "disc" => style(System::Cyclic, vec!["\u{2022}".into()], " "),
        "circle" => style(System::Cyclic, vec!["\u{25E6}".into()], " "),
        "square" => style(System::Cyclic, vec!["\u{25AA}".into()], " "),
        "disclosure-open" => style(System::Cyclic, vec!["\u{25BE}".into()], " "),
        "disclosure-closed" => style(System::Cyclic, vec!["\u{25B8}".into()], " "),
        _ => return None,
    })
}

/// The counter style `name` declares: the document's `@counter-style`,
/// else a predefined one, else `decimal`.
fn lookup(name: &str, sheet: &Sheet) -> CounterStyle {
    let style = sheet
        .counter(name)
        .cloned()
        .or_else(|| predefined(name))
        .unwrap_or_else(|| predefined("decimal").expect("decimal is predefined"));
    match &style.system {
        // `extends`: the other style's algorithm and symbols with this
        // one's other descriptors.
        System::Extends(base) if base != name => {
            let base = lookup(base, sheet);
            CounterStyle {
                system: base.system,
                symbols: base.symbols,
                additive: base.additive,
                ..style
            }
        }
        _ => style,
    }
}

/// `n` as counter style `name` represents it in `counter()` (without its
/// prefix and suffix); `none` is empty.
pub fn format(n: i64, name: &str, sheet: &Sheet) -> String {
    if name == "none" {
        return String::new();
    }
    represent(n, name, sheet, 0)
}

/// `n` as a list marker: the representation with the style's prefix and
/// suffix.
pub fn marker(n: i64, name: &str, sheet: &Sheet) -> String {
    if name == "none" {
        return String::new();
    }
    let style = lookup(name, sheet);
    format!(
        "{}{}{}",
        style.prefix,
        represent(n, name, sheet, 0),
        style.suffix
    )
}

fn represent(n: i64, name: &str, sheet: &Sheet, depth: u8) -> String {
    let style = lookup(name, sheet);
    let fallback = |n: i64| {
        if depth > 4 || style.fallback == name {
            n.to_string()
        } else {
            represent(n, &style.fallback, sheet, depth + 1)
        }
    };
    let in_range = if style.range.is_empty() {
        match style.system {
            System::Alphabetic | System::Symbolic => n >= 1,
            System::Additive => n >= 0,
            _ => true,
        }
    } else {
        style.range.iter().any(|&(lo, hi)| (lo..=hi).contains(&n))
    };
    if !in_range {
        return fallback(n);
    }
    let negative = n < 0
        && matches!(
            style.system,
            System::Symbolic | System::Alphabetic | System::Numeric | System::Additive
        );
    let Some(mut body) = algorithm(&style, if negative { -n } else { n }) else {
        return fallback(n);
    };
    if let Some((len, pad)) = &style.pad {
        let sign = if negative {
            style.negative.0.chars().count() + style.negative.1.chars().count()
        } else {
            0
        };
        let have = body.chars().count() + sign;
        if have < *len {
            body = pad.repeat(len - have) + &body;
        }
    }
    if negative {
        format!("{}{body}{}", style.negative.0, style.negative.1)
    } else {
        body
    }
}

/// The counter system's representation of nonnegative `n` (any `n` for
/// cyclic and fixed), `None` where it has none.
fn algorithm(style: &CounterStyle, n: i64) -> Option<String> {
    let s = &style.symbols;
    let len = s.len() as i64;
    match style.system {
        System::Cyclic => (len > 0).then(|| s[(n - 1).rem_euclid(len) as usize].clone()),
        System::Fixed(first) => {
            (n >= first && n - first < len).then(|| s[(n - first) as usize].clone())
        }
        System::Symbolic => (n >= 1 && len > 0).then(|| {
            let symbol = &s[((n - 1) % len) as usize];
            symbol.repeat(((n - 1) / len + 1) as usize)
        }),
        System::Alphabetic => (n >= 1 && len >= 2).then(|| {
            let mut out = Vec::new();
            let mut n = n;
            while n > 0 {
                n -= 1;
                out.push(s[(n % len) as usize].clone());
                n /= len;
            }
            out.reverse();
            out.concat()
        }),
        System::Numeric => (len >= 2).then(|| {
            if n == 0 {
                return s[0].clone();
            }
            let mut out = Vec::new();
            let mut n = n;
            while n > 0 {
                out.push(s[(n % len) as usize].clone());
                n /= len;
            }
            out.reverse();
            out.concat()
        }),
        System::Additive => {
            if n == 0 {
                return style
                    .additive
                    .iter()
                    .find(|(w, _)| *w == 0)
                    .map(|(_, s)| s.clone());
            }
            let mut out = String::new();
            let mut n = n;
            for (weight, symbol) in &style.additive {
                if *weight <= 0 {
                    continue;
                }
                while n >= *weight {
                    out.push_str(symbol);
                    n -= weight;
                }
            }
            (n == 0).then_some(out)
        }
        System::Extends(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_styles_follow_css_counter_styles() {
        let sheet = Sheet::default();
        let f = |n, s| format(n, s, &sheet);
        assert_eq!(f(7, "decimal"), "7");
        assert_eq!(f(-7, "decimal"), "-7");
        assert_eq!(f(7, "decimal-leading-zero"), "07");
        assert_eq!(f(-7, "decimal-leading-zero"), "-7");
        assert_eq!(f(1994, "upper-roman"), "MCMXCIV");
        assert_eq!(f(4000, "lower-roman"), "4000");
        assert_eq!(f(28, "lower-alpha"), "ab");
        assert_eq!(f(0, "lower-alpha"), "0");
        assert_eq!(f(2, "lower-greek"), "β");
        assert_eq!(f(12, "cjk-decimal"), "一二");
        assert_eq!(f(3, "unknown-style"), "3");
        assert_eq!(marker(3, "lower-roman", &sheet), "iii. ");
        let mut sheet = Sheet::default();
        sheet
            .counter_style("stars", "system: symbolic; symbols: '*' '+'; suffix: ' ';")
            .unwrap();
        sheet
            .counter_style(
                "dice",
                "system: fixed; symbols: ⚀ ⚁ ⚂; fallback: lower-alpha;",
            )
            .unwrap();
        sheet
            .counter_style(
                "paren",
                "system: extends lower-roman; prefix: '('; suffix: ') ';",
            )
            .unwrap();
        assert_eq!(format(3, "stars", &sheet), "**");
        assert_eq!(format(2, "dice", &sheet), "⚁");
        assert_eq!(format(5, "dice", &sheet), "e");
        assert_eq!(marker(4, "paren", &sheet), "(iv) ");
    }

    #[test]
    fn counters_scope_to_siblings_and_quotes_nest() {
        let sheet = Sheet::default();
        let ops = |reset: &[&str], inc: &[&str]| {
            Arc::new(Ops {
                reset: reset.iter().map(|n| (n.to_string(), 0)).collect(),
                increment: inc.iter().map(|n| (n.to_string(), 1)).collect(),
                set: Vec::new(),
            })
        };
        let content = |c: &str| match parse_content(c).unwrap() {
            Content::Pieces(p) => p,
            other => panic!("{other:?}"),
        };
        let mut run = Run::default();
        let text = |run: &mut Run, c: &str| {
            run.evaluate(&content(c), &Quotes::Auto, &sheet, &|_| None, None)
        };
        // <ol reset=item> <li inc/> <li inc> <ol reset=item><li inc/></ol> </li> </ol>
        run.enter();
        run.apply(&ops(&["item"], &[]));
        run.enter();
        run.apply(&ops(&[], &["item"]));
        assert_eq!(text(&mut run, "counter(item) '.'"), "1.");
        run.exit();
        run.enter();
        run.apply(&ops(&[], &["item"]));
        let mark = run.mark();
        run.enter();
        run.apply(&ops(&["item"], &[]));
        run.enter();
        run.apply(&ops(&[], &["item"]));
        assert_eq!(text(&mut run, "counters(item, '.', upper-roman)"), "II.I");
        run.exit();
        run.exit();
        let nested = run.since(mark);
        // After the nested list, still inside its item: the nested
        // instance (its following siblings are in its scope).
        assert_eq!(text(&mut run, "counter(item)"), "1");
        run.exit();
        // Out of the item: the nested counter is out of scope.
        assert_eq!(text(&mut run, "counter(item)"), "2");
        run.exit();
        // After the list: its following siblings are in its scope too.
        assert_eq!(text(&mut run, "counter(item)"), "2");
        // Replaying the nested list's events moves the state the same way.
        let mut a = Run::default();
        a.enter();
        a.apply(&ops(&["item"], &["item"]));
        let mut b = a.clone();
        a.replay(&nested);
        b.replay(&nested);
        assert!(a.same_state(&b));
        // Style containment: a counter incremented inside is its own,
        // and nothing inside changes the one outside (Containment 2 §3.3).
        let mut c = Run::default();
        c.enter();
        c.apply(&ops(&["item"], &["item"]));
        c.enter_contained();
        c.apply(&ops(&[], &["item"]));
        assert_eq!(text(&mut c, "counters(item, '.')"), "1.1");
        c.apply(&ops(&["item"], &[]));
        assert_eq!(text(&mut c, "counters(item, '.')"), "1.0");
        c.exit();
        assert_eq!(text(&mut c, "counter(item)"), "1");
        // Quotes nest by depth; a close at depth 0 shows nothing.
        let mut q = Run::default();
        assert_eq!(
            text(
                &mut q,
                "open-quote 'a' open-quote 'b' close-quote close-quote close-quote"
            ),
            "\u{201C}a\u{2018}b\u{2019}\u{201D}"
        );
        assert_eq!(
            q.evaluate(
                &content("open-quote"),
                &parse_quotes("'<' '>' '[' ']'").unwrap(),
                &sheet,
                &|_| None,
                None
            ),
            "<"
        );
    }

    #[test]
    fn content_and_counter_properties_parse_to_spec() {
        assert_eq!(parse_content("normal").unwrap(), Content::Normal);
        assert_eq!(parse_content("none").unwrap(), Content::None);
        assert_eq!(
            parse_content("\"#\" attr(data-n) counter(c, lower-alpha) / \"alt\"").unwrap(),
            Content::Pieces(vec![
                Piece::Text("#".into()),
                Piece::Attr("data-n".into()),
                Piece::Counter("c".into(), "lower-alpha".into()),
            ])
        );
        assert!(parse_content("12px").is_err());
        assert_eq!(
            parse_ops("counter-increment", "a b 2").unwrap(),
            vec![("a".into(), 1), ("b".into(), 2)]
        );
        assert_eq!(
            parse_ops("counter-reset", "a").unwrap(),
            vec![("a".into(), 0)]
        );
        assert!(parse_ops("counter-set", "3").is_err());
        assert!(parse_quotes("'a'").is_err());
    }
}
