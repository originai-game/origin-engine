//! Selectors, parsed and matched by Servo's `selectors` crate
//! (third_party/selectors) over og's [`OgImpl`]: its interaction states,
//! `:lang()`, `:dir()`, `:state()`, `:defined` and `:any-link`. Specificity
//! is the crate's (Selectors 4 §16). Sibling combinators and structural
//! pseudo-classes depend on the instantiated tree, so [`Selector::structural`]
//! selectors are matched at runtime (`super::matching`) instead of being
//! lowered to a reactive predicate here.
use super::{NONE, UiDocument, syntax::Attr};
use crate::bind::{AttrOp, AttrTest, Expr, InteractionState, Op};
use cssparser::{ParseError, ParseErrorKind, Parser as CssParser, ToCss, Token};
use selectors::attr::{AttrSelectorOperator, ParsedAttrSelectorOperation, ParsedCaseSensitivity};
use selectors::parser::{self as sp, SelectorParseErrorKind};
use std::fmt;

/// A document's elements with their tags and attributes, found by node
/// index in constant time: matching a rule's selector against every
/// element asks for elements by index (each one's ancestors too), and a
/// search of the list made that rules × elements².
pub(crate) struct Elements<'a> {
    list: &'a [(usize, String, Vec<Attr>)],
    /// Each node's place in `list`; `usize::MAX` for a node not in it.
    at: Vec<usize>,
}

impl<'a> Elements<'a> {
    pub(crate) fn new(list: &'a [(usize, String, Vec<Attr>)], nodes: usize) -> Self {
        let mut at = vec![usize::MAX; nodes];
        for (k, (i, _, _)) in list.iter().enumerate() {
            // The first entry for a node, as a search of the list finds.
            if at[*i] == usize::MAX {
                at[*i] = k;
            }
        }
        Self { list, at }
    }

    /// The element at node `index`, when it is one.
    fn get(&self, index: usize) -> Option<&'a (usize, String, Vec<Attr>)> {
        self.at
            .get(index)
            .filter(|&&k| k != usize::MAX)
            .map(|&k| &self.list[k])
    }
}

pub use selectors::parser::Combinator;
/// One simple selector or combinator of a parsed selector.
pub type Component = sp::Component<OgImpl>;
/// A selector as the crate holds it.
pub type Inner = sp::Selector<OgImpl>;

/// An identifier, attribute value or tag name in a selector.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Atom(pub String);

impl From<&str> for Atom {
    fn from(s: &str) -> Self {
        Atom(s.into())
    }
}

impl AsRef<str> for Atom {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<str> for Atom {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl ToCss for Atom {
    fn to_css<W: fmt::Write>(&self, dest: &mut W) -> fmt::Result {
        cssparser::serialize_identifier(&self.0, dest)
    }
}

impl precomputed_hash::PrecomputedHash for Atom {
    fn precomputed_hash(&self) -> u32 {
        // FNV-1a: stable across runs, so matching stays deterministic.
        self.0.bytes().fold(0x811c_9dc5, |h: u32, b| {
            (h ^ u32::from(b)).wrapping_mul(0x0100_0193)
        })
    }
}

/// og's pseudo-classes beside the crate's own (`:root`, `:empty`, `:scope`,
/// the `:nth-*` family, `:not()`, `:is()`, `:where()` and `:has()`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PseudoClass {
    State(InteractionState),
    /// `:lang(…)`: the nearest `lang` of the element or an ancestor, else
    /// the locale.
    Lang(Box<[String]>),
    /// `:dir(rtl)` (true) or `:dir(ltr)`: the nearest `dir`, else the
    /// locale's direction.
    Dir(bool),
    /// `:state(name)`: the element's `state:name={…}` is truthy.
    Custom(Atom),
    /// `:size(narrow|short|compact)` and `:touch`: a class of the UI's own
    /// box or input (og-ui-next).
    Screen(crate::bind::Screen),
    /// `:defined`: every element is.
    Defined,
    /// `:any-link` and `:link`: an `<a>` or `<area>` with an `href`.
    AnyLink,
    /// `:active-view-transition-type(…)`: a screen's root while its
    /// layer's view transition runs, when the screen's `@view-transition`
    /// declares one of the types.
    ViewTransitionType(Box<[Atom]>),
    /// `:heading` (no levels) and `:heading(1, 2)`: an `<h1>` to `<h6>`,
    /// of one of the levels.
    Heading(Box<[u8]>),
    /// `:has-slotted`: an element that holds a `{@render}` site where
    /// content passed to its document is rendered. The site has no box,
    /// so the element around it answers for the slot.
    HasSlotted,
}

/// The heading level of a tag: `h1` to `h6`.
pub(crate) fn heading_level(tag: &str) -> Option<u8> {
    match tag.as_bytes() {
        [b'h' | b'H', level @ b'1'..=b'6'] => Some(level - b'0'),
        _ => None,
    }
}

impl PseudoClass {
    /// Whether a `:heading` of `levels` matches an element of `tag`.
    pub(crate) fn heading_matches(levels: &[u8], tag: &str) -> bool {
        heading_level(tag).is_some_and(|l| levels.is_empty() || levels.contains(&l))
    }
}

impl ToCss for PseudoClass {
    fn to_css<W: fmt::Write>(&self, dest: &mut W) -> fmt::Result {
        match self {
            PseudoClass::State(state) => write!(dest, ":{}", state_name(*state)),
            PseudoClass::Lang(ranges) => {
                dest.write_str(":lang(")?;
                for (i, range) in ranges.iter().enumerate() {
                    if i > 0 {
                        dest.write_str(", ")?;
                    }
                    cssparser::serialize_string(range, dest)?;
                }
                dest.write_str(")")
            }
            PseudoClass::Dir(rtl) => dest.write_str(if *rtl { ":dir(rtl)" } else { ":dir(ltr)" }),
            PseudoClass::Screen(s) => dest.write_str(match s {
                crate::bind::Screen::Narrow => ":size(narrow)",
                crate::bind::Screen::Short => ":size(short)",
                crate::bind::Screen::Compact => ":size(compact)",
                crate::bind::Screen::Touch => ":touch",
            }),
            PseudoClass::Custom(name) => {
                dest.write_str(":state(")?;
                name.to_css(dest)?;
                dest.write_str(")")
            }
            PseudoClass::Defined => dest.write_str(":defined"),
            PseudoClass::AnyLink => dest.write_str(":any-link"),
            PseudoClass::HasSlotted => dest.write_str(":has-slotted"),
            PseudoClass::Heading(levels) => {
                dest.write_str(":heading")?;
                if !levels.is_empty() {
                    let list: Vec<String> = levels.iter().map(u8::to_string).collect();
                    write!(dest, "({})", list.join(", "))?;
                }
                Ok(())
            }
            PseudoClass::ViewTransitionType(types) => {
                dest.write_str(":active-view-transition-type(")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        dest.write_str(", ")?;
                    }
                    t.to_css(dest)?;
                }
                dest.write_str(")")
            }
        }
    }
}

impl sp::NonTSPseudoClass for PseudoClass {
    fn is_active_or_hover(&self) -> bool {
        matches!(
            self,
            PseudoClass::State(InteractionState::Hover | InteractionState::Active)
        )
    }

    fn is_user_action_state(&self) -> bool {
        use InteractionState::*;
        matches!(
            self,
            PseudoClass::State(Hover | Active | Focus | FocusVisible | FocusWithin)
        )
    }
}

/// The standard pseudo-elements a control document gives an element of its
/// own (`pseudo="details-content"`, docs/ui.md §2.4): a rule written for
/// one in the host's document (`details::details-content`) reaches that
/// element as `::part()` reaches a part. `::part()` the crate parses
/// itself; `::before` and the other pseudo-elements og draws are read off
/// a rule before its selector is parsed (`stylesheet::pseudo_element`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PseudoElement {
    Backdrop,
    DetailsContent,
    Picker,
    PickerIcon,
    Checkmark,
    FileSelectorButton,
}

impl PseudoElement {
    pub const ALL: [PseudoElement; 6] = [
        PseudoElement::Backdrop,
        PseudoElement::DetailsContent,
        PseudoElement::Picker,
        PseudoElement::PickerIcon,
        PseudoElement::Checkmark,
        PseudoElement::FileSelectorButton,
    ];

    /// Its `pseudo=` name.
    pub fn name(self) -> &'static str {
        match self {
            PseudoElement::Backdrop => "backdrop",
            PseudoElement::DetailsContent => "details-content",
            PseudoElement::Picker => "picker",
            PseudoElement::PickerIcon => "picker-icon",
            PseudoElement::Checkmark => "checkmark",
            PseudoElement::FileSelectorButton => "file-selector-button",
        }
    }

    pub fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.name() == name)
    }

    /// The host it belongs to, as a control's host selector names it:
    /// HTML gives a `<dialog>` and a popover a backdrop, a `<select>` its
    /// picker and picker icon, an `<option>` its checkmark and a file
    /// input its button.
    pub fn belongs_to(self, host: &super::ControlHost) -> bool {
        let tests = |name: &str, value: Option<&str>| {
            host.attributes
                .iter()
                .any(|(n, v)| n == name && (value.is_none() || v.as_deref() == value))
        };
        match self {
            PseudoElement::Backdrop => host.tag == "dialog" || tests("popover", None),
            PseudoElement::DetailsContent => host.tag == "details",
            PseudoElement::Picker | PseudoElement::PickerIcon => host.tag == "select",
            PseudoElement::Checkmark => host.tag == "option",
            PseudoElement::FileSelectorButton => host.tag == "input" && tests("type", Some("file")),
        }
    }
}

impl ToCss for PseudoElement {
    fn to_css<W: fmt::Write>(&self, dest: &mut W) -> fmt::Result {
        match self {
            PseudoElement::Picker => dest.write_str("::picker(select)"),
            p => write!(dest, "::{}", p.name()),
        }
    }
}

/// Each is an element of the control's document, so it takes state
/// pseudo-classes of its own as a part does (`dialog::backdrop:hover`).
impl sp::PseudoElement for PseudoElement {
    fn accepts_state_pseudo_classes(&self) -> bool {
        true
    }

    fn parses_as_element_backed(&self) -> bool {
        true
    }
}

/// og's selector vocabulary for the crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OgImpl;

impl sp::SelectorImpl for OgImpl {
    type ExtraMatchingData<'a> = ();
    type AttrValue = Atom;
    type Identifier = Atom;
    type LocalName = Atom;
    type NamespaceUrl = Atom;
    type NamespacePrefix = Atom;
    type BorrowedNamespaceUrl = str;
    type BorrowedLocalName = str;
    type NonTSPseudoClass = PseudoClass;
    type PseudoElement = PseudoElement;
}

/// A parse error: the crate's, or og's own message.
#[derive(Clone, Debug)]
pub enum OgError {
    Kind(SelectorParseErrorKind),
    Message(String),
}

impl From<SelectorParseErrorKind> for OgError {
    fn from(kind: SelectorParseErrorKind) -> Self {
        OgError::Kind(kind)
    }
}

fn message(text: impl Into<String>) -> ParseError<OgError> {
    ParseError::custom(OgError::Message(text.into()))
}

struct OgParser;

impl<'i> sp::Parser<'i> for OgParser {
    type Impl = OgImpl;
    type Error = OgError;

    fn parse_part(&self) -> bool {
        true
    }

    fn parse_host(&self) -> bool {
        true
    }

    fn parse_slotted(&self) -> bool {
        true
    }

    fn parse_nth_child_of(&self) -> bool {
        true
    }

    fn parse_is_and_where(&self) -> bool {
        true
    }

    fn parse_has(&self) -> bool {
        true
    }

    /// An argument that does not parse is an error, not a dropped
    /// alternative: an author sees the typo.
    fn allow_forgiving_selectors(&self) -> bool {
        false
    }

    fn parse_non_ts_pseudo_class(&self, name: &str) -> Result<PseudoClass, ParseError<OgError>> {
        let name = name.to_ascii_lowercase();
        if let Some(state) = state(&name) {
            return Ok(PseudoClass::State(state));
        }
        match name.as_str() {
            "defined" => Ok(PseudoClass::Defined),
            "heading" => Ok(PseudoClass::Heading(Box::default())),
            "has-slotted" => Ok(PseudoClass::HasSlotted),
            "any-link" | "link" => Ok(PseudoClass::AnyLink),
            "touch" => Ok(PseudoClass::Screen(crate::bind::Screen::Touch)),
            _ => Err(message(format!(
                "unsupported pseudo-class :{name}; see the selector list in docs/ui.md §2.4"
            ))),
        }
    }

    fn parse_non_ts_functional_pseudo_class(
        &self,
        name: &str,
        input: &mut CssParser<'i>,
        _after_part: bool,
    ) -> Result<PseudoClass, ParseError<OgError>> {
        match name.to_ascii_lowercase().as_str() {
            "lang" => {
                let ranges = input.parse_comma_separated(|input| {
                    Ok(match input.next()? {
                        Token::Ident(v) | Token::QuotedString(v) => v.to_string(),
                        _ => return Err(message(":lang() takes language ranges")),
                    })
                })?;
                Ok(PseudoClass::Lang(ranges.into()))
            }
            "dir" => {
                let word = input.expect_ident()?.to_ascii_lowercase();
                match word.as_str() {
                    "rtl" => Ok(PseudoClass::Dir(true)),
                    "ltr" => Ok(PseudoClass::Dir(false)),
                    _ => Err(message(format!(":dir() takes ltr or rtl, not {word}"))),
                }
            }
            "state" => Ok(PseudoClass::Custom(input.expect_ident()?.as_ref().into())),
            "size" => {
                use crate::bind::Screen;
                let word = input.expect_ident()?.to_ascii_lowercase();
                Ok(PseudoClass::Screen(match word.as_str() {
                    "narrow" => Screen::Narrow,
                    "short" => Screen::Short,
                    "compact" => Screen::Compact,
                    _ => {
                        return Err(message(format!(
                            ":size() takes narrow, short or compact, not {word}"
                        )));
                    }
                }))
            }
            "heading" => {
                let levels = input.parse_comma_separated(|input| {
                    let level = input.expect_integer()?;
                    u8::try_from(level).ok().filter(|l| *l > 0).ok_or_else(|| {
                        message(":heading() takes heading levels, as :heading(1, 2)")
                    })
                })?;
                Ok(PseudoClass::Heading(levels.into()))
            }
            "active-view-transition-type" => {
                let types = input
                    .parse_comma_separated(|input| Ok(input.expect_ident()?.as_ref().into()))?;
                Ok(PseudoClass::ViewTransitionType(types.into()))
            }
            name => Err(message(format!(
                "unsupported pseudo-class :{name}(); supported functions are :not(), :is(), :where(), :has(), :nth-child(), :nth-last-child(), :nth-of-type(), :nth-last-of-type(), :lang(), :dir(), :state(), :heading() and :active-view-transition-type()"
            ))),
        }
    }

    fn parse_pseudo_element(&self, name: &str) -> Result<PseudoElement, ParseError<OgError>> {
        PseudoElement::named(&name.to_ascii_lowercase())
            .filter(|p| *p != PseudoElement::Picker)
            .ok_or_else(|| {
                message(format!(
                    "unsupported pseudo-element ::{name}; use ::part(name)"
                ))
            })
    }

    fn parse_functional_pseudo_element(
        &self,
        name: &str,
        input: &mut CssParser<'i>,
    ) -> Result<PseudoElement, ParseError<OgError>> {
        if name.eq_ignore_ascii_case("picker")
            && input.expect_ident()?.eq_ignore_ascii_case("select")
        {
            return Ok(PseudoElement::Picker);
        }
        Err(message(format!(
            "unsupported pseudo-element ::{name}(); a select's picker is ::picker(select)"
        )))
    }
}

const STATES: &[(&str, InteractionState)] = {
    use InteractionState::*;
    &[
        ("hover", Hover),
        ("active", Active),
        ("focus", Focus),
        ("focus-visible", FocusVisible),
        ("focus-within", FocusWithin),
        ("disabled", Disabled),
        ("enabled", Enabled),
        ("checked", Checked),
        ("selected", Selected),
        ("invalid", Invalid),
        ("valid", Valid),
        ("user-invalid", UserInvalid),
        ("user-valid", UserValid),
        ("placeholder-shown", PlaceholderShown),
        ("required", Required),
        ("optional", Optional),
        ("read-only", ReadOnly),
        ("read-write", ReadWrite),
        ("in-range", InRange),
        ("out-of-range", OutOfRange),
        ("indeterminate", Indeterminate),
        ("default", Default),
        ("open", Open),
        ("popover-open", PopoverOpen),
        ("modal", Modal),
        ("target", Target),
        ("active-view-transition", ActiveViewTransition),
        ("target-current", TargetCurrent),
        ("target-before", TargetBefore),
        ("target-after", TargetAfter),
        ("playing", Playing),
        ("paused", Paused),
        ("seeking", Seeking),
        ("muted", Muted),
        ("buffering", Buffering),
        ("stalled", Stalled),
        ("volume-locked", VolumeLocked),
    ]
};

pub(crate) fn state(name: &str) -> Option<InteractionState> {
    STATES.iter().find(|(n, _)| *n == name).map(|(_, s)| *s)
}

fn state_name(state: InteractionState) -> &'static str {
    STATES
        .iter()
        .find(|(_, s)| *s == state)
        .map_or("", |(n, _)| n)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Specificity(pub u32, pub u32, pub u32);

impl From<u32> for Specificity {
    fn from(v: u32) -> Self {
        Self(v >> 20, (v >> 10) & 0x3ff, v & 0x3ff)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Selector {
    pub source: String,
    /// A global stylesheet may match across component document boundaries.
    pub global: bool,
    pub inner: Inner,
    /// Attribute tests after `::part()`: a text span's properties
    /// (`text::part(name)[rarity="rare"]`, `super::spans`), which the
    /// crate does not parse. Each counts as an attribute selector.
    pub span: u32,
    /// For a `::part()` selector whose host is matched in its own tree
    /// (`.shop button::part(base)`, `li:first-child > button::part(base)`):
    /// the host's compound as the part's shadow tree sees it, without what
    /// reads the host's tree ([`Selector::host_marker`]).
    pub local: Option<std::sync::Arc<Selector>>,
}

/// The selector without the attribute tests that follow its `::part()`,
/// and how many there were ([`Selector::span`]).
fn span_tests(item: &str) -> (String, u32) {
    let mut input = CssParser::new(item);
    let mut colons = 0;
    let mut after = None;
    while let Ok(token) = input.next_including_whitespace() {
        match token {
            Token::Colon => colons += 1,
            Token::Function(name) if colons == 2 && name.eq_ignore_ascii_case("part") => {
                let _ = input.parse_nested_block(|i| {
                    while i.next().is_ok() {}
                    Ok::<(), ParseError<()>>(())
                });
                after = Some(input.position().byte_index());
                break;
            }
            _ => colons = 0,
        }
    }
    let Some(after) = after else {
        return (item.to_string(), 0);
    };
    let (mut text, mut tests) = (item[..after].to_string(), 0);
    loop {
        let start = input.position().byte_index();
        match input.next_including_whitespace() {
            Ok(Token::SquareBracketBlock) => {
                let _ = input.parse_nested_block(|i| {
                    while i.next().is_ok() {}
                    Ok::<(), ParseError<()>>(())
                });
                tests += 1;
            }
            Ok(_) => text.push_str(&item[start..input.position().byte_index()]),
            Err(_) => break,
        }
    }
    (if tests == 0 { item.to_string() } else { text }, tests)
}

/// A selector's compounds in source order, each with its components in
/// source order and the combinator after it (the crate stores them in match
/// order: the rightmost compound first).
fn segments(selector: &Inner) -> Vec<(Vec<&Component>, Option<Combinator>)> {
    let mut out: Vec<(Vec<&Component>, Option<Combinator>)> = vec![(Vec::new(), None)];
    for c in selector.iter_raw_match_order() {
        match c {
            Component::Combinator(combinator) => out.push((Vec::new(), Some(*combinator))),
            c => out.last_mut().expect("a compound").0.push(c),
        }
    }
    // A combinator is stored before the compound on its left in source.
    out.reverse();
    out
}

/// Whether a combinator joins a compound to a pseudo-element of it
/// (`::part()`) rather than to another element.
fn joins(combinator: Combinator) -> bool {
    matches!(
        combinator,
        Combinator::Part | Combinator::PseudoElement | Combinator::SlotAssignment
    )
}

/// A selector's compounds in source order, a `::part()` kept in the compound
/// of its host (`button.primary::part(base):hover` is one), and the
/// combinators between them.
pub fn compounds(selector: &Inner) -> (Vec<Vec<&Component>>, Vec<Combinator>) {
    let mut compounds = vec![Vec::new()];
    let mut combinators = Vec::new();
    for (components, after) in segments(selector) {
        compounds.last_mut().expect("a compound").extend(components);
        if let Some(combinator) = after.filter(|c| !joins(*c)) {
            combinators.push(combinator);
            compounds.push(Vec::new());
        }
    }
    (compounds, combinators)
}

/// The nested selector lists a component holds.
pub fn nested(c: &Component) -> &[Inner] {
    match c {
        Component::Negation(list) | Component::Is(list) | Component::Where(list) => list.slice(),
        Component::NthOf(nth) => nth.selectors(),
        Component::Host(Some(argument)) | Component::Slotted(argument) => {
            std::slice::from_ref(argument)
        }
        _ => &[],
    }
}

/// The element `:host` selects in `doc`. A component instance and its root
/// are one element, so a component's host is its root: `:host(…)` reads the
/// instance's attributes (the component's props) and the root's state. A
/// document attached under a host element of its own answers that element
/// here, bound in [`crate::bind::BindingContext::selector_nodes`].
pub(crate) fn host_node(doc: &UiDocument) -> usize {
    match doc.control {
        Some(_) => super::HOST_NODE,
        None => doc.root,
    }
}

/// What `:host(<compound>)` asks of the host, as a condition: `features`
/// are the compound's simple selectors without those that need the
/// instantiated tree. An attribute is the prop of that name as the
/// instance passes it (absent while it is null or false), a class one of
/// the words of its `class`, a state the host's own.
fn host_features<'a>(
    features: impl IntoIterator<Item = &'a Component>,
    doc: &UiDocument,
    runtime: &dyn Fn() -> String,
) -> Result<Expr, String> {
    // An instance's attributes are its component's props: one the
    // component does not declare is never passed.
    let prop = |name: &str, c: &Component| {
        if doc.props.is_empty() || doc.props.iter().any(|p| p.name == name) {
            return Ok(Box::new(Expr::Path(name.to_owned())));
        }
        Err(format!(
            "`:host({})` reads the instance's {name}, which <{}> does not declare: add <meta name=\"prop:{name}\" content=\"string\">",
            c.to_css_string(),
            doc.name
        ))
    };
    let mut all = Expr::Lit(true.into());
    for c in features {
        let test = match c {
            // A control's host is its element; a component's its instance.
            Component::LocalName(name) => Expr::Lit(
                name.name
                    .0
                    .eq_ignore_ascii_case(doc.control.as_ref().map_or(&doc.name, |c| &c.tag))
                    .into(),
            ),
            Component::ExplicitUniversalType
            | Component::ExplicitAnyNamespace
            | Component::ExplicitNoNamespace
            | Component::DefaultNamespace(_)
            | Component::Namespace(..)
            | Component::ImplicitScope
            | Component::NonTSPseudoClass(PseudoClass::Defined) => Expr::Lit(true.into()),
            Component::ID(_) => {
                return Err(format!(
                    "`:host({})`: an instance's id names its root's path and is not passed to the component; select by a prop, as :host([name=value])",
                    c.to_css_string()
                ));
            }
            Component::Class(name) => Expr::HasToken(prop("class", c)?, name.0.clone()),
            Component::AttributeInNoNamespaceExists { .. }
            | Component::AttributeInNoNamespace { .. }
            | Component::AttributeOther(_) => {
                let (name, test) = attribute(c).expect("an attribute selector");
                match test {
                    None => combine(
                        Op::And,
                        Expr::Bin(
                            Op::Ne,
                            prop(name, c)?,
                            Box::new(Expr::Lit(serde_json::Value::Null)),
                        ),
                        Expr::Bin(Op::Ne, prop(name, c)?, Box::new(Expr::Lit(false.into()))),
                    ),
                    Some(test) => Expr::AttrTest(prop(name, c)?, test),
                }
            }
            Component::NonTSPseudoClass(PseudoClass::State(state)) => Expr::InteractionAt {
                node: host_node(doc),
                state: *state,
            },
            Component::NonTSPseudoClass(PseudoClass::Lang(ranges)) => {
                Expr::Lang(None, ranges.to_vec())
            }
            Component::NonTSPseudoClass(PseudoClass::Dir(rtl)) => Expr::Dir(None, *rtl),
            Component::NonTSPseudoClass(PseudoClass::Screen(s)) => Expr::Screen(*s),
            Component::Negation(_) | Component::Is(_) | Component::Where(_) => {
                let mut any = Expr::Lit(false.into());
                for inner in nested(c) {
                    let inner = host_features(inner.iter_raw_parse_order_from(0), doc, runtime)?;
                    any = combine(Op::Or, any, inner);
                }
                if matches!(c, Component::Negation(_)) {
                    match any {
                        Expr::Lit(serde_json::Value::Bool(value)) => Expr::Lit((!value).into()),
                        value => Expr::Not(Box::new(value)),
                    }
                } else {
                    any
                }
            }
            Component::Nth(_) | Component::NthOf(_) | Component::Empty | Component::Has(_) => {
                return Err(runtime());
            }
            _ => Expr::Lit(false.into()),
        };
        all = combine(Op::And, all, test);
    }
    Ok(all)
}

/// Whether `c` or a selector nested in it holds a component `f` accepts.
fn walk_any(c: &Component, f: &dyn Fn(&Component) -> bool) -> bool {
    f(c) || nested(c)
        .iter()
        .any(|s| s.iter_raw_parse_order_from(0).any(|c| walk_any(c, f)))
}

/// An attribute selector's name and value test.
pub fn attribute(c: &Component) -> Option<(&str, Option<AttrTest>)> {
    match c {
        Component::AttributeInNoNamespaceExists { local_name, .. } => Some((&local_name.0, None)),
        Component::AttributeInNoNamespace {
            local_name,
            operator,
            value,
            case_sensitivity,
        } => Some((
            &local_name.0,
            Some(attr_test(*operator, value, *case_sensitivity)),
        )),
        Component::AttributeOther(attr) => Some((
            &attr.local_name.0,
            match &attr.operation {
                ParsedAttrSelectorOperation::Exists => None,
                ParsedAttrSelectorOperation::WithValue {
                    operator,
                    case_sensitivity,
                    value,
                } => Some(attr_test(*operator, value, *case_sensitivity)),
            },
        )),
        _ => None,
    }
}

fn attr_test(op: AttrSelectorOperator, value: &Atom, case: ParsedCaseSensitivity) -> AttrTest {
    AttrTest {
        op: match op {
            AttrSelectorOperator::Equal => AttrOp::Equals,
            AttrSelectorOperator::Includes => AttrOp::Includes,
            AttrSelectorOperator::DashMatch => AttrOp::Dash,
            AttrSelectorOperator::Prefix => AttrOp::Prefix,
            AttrSelectorOperator::Substring => AttrOp::Substring,
            AttrSelectorOperator::Suffix => AttrOp::Suffix,
        },
        value: value.0.clone(),
        insensitive: matches!(
            case,
            ParsedCaseSensitivity::AsciiCaseInsensitive
                | ParsedCaseSensitivity::AsciiCaseInsensitiveIfInHtmlElementInHtmlDocument
        ),
    }
}

/// Whether an element reflects its open state as its `open` attribute, as
/// HTML's `<details>` and `<dialog>` do whoever opens them.
pub(crate) fn reflects_open(tag: &str) -> bool {
    matches!(tag, "details" | "dialog")
}

/// Whether a component needs the instantiated sibling order or children.
pub fn structural_component(c: &Component) -> bool {
    // Placement feedback exists only on the instantiated anchor, not in
    // authored markup, so these selectors must read the retained tree.
    if attribute(c).is_some_and(|(name, _)| {
        matches!(
            name,
            "data-clamped" | "data-edge" | "data-lod" | "data-blocked"
        )
    }) {
        return true;
    }
    match c {
        Component::Nth(_)
        | Component::NthOf(_)
        | Component::Empty
        | Component::Has(_)
        | Component::Slotted(_)
        | Component::NonTSPseudoClass(PseudoClass::HasSlotted) => true,
        c => nested(c).iter().any(structural),
    }
}

/// Whether matching needs the instantiated sibling order: a sibling
/// combinator or a structural pseudo-class anywhere, including arguments.
pub fn structural(selector: &Inner) -> bool {
    selector.iter_raw_parse_order_from(0).any(|c| match c {
        Component::Combinator(c) => c.is_sibling(),
        c => structural_component(c),
    })
}

/// One compound, recursively, that reads only the element's own facts.
pub fn compound_only(selector: &Inner) -> bool {
    !structural(selector)
        && selector.iter_raw_parse_order_from(0).all(|c| match c {
            Component::Combinator(_)
            | Component::Part(_)
            | Component::NonTSPseudoClass(PseudoClass::Lang(_) | PseudoClass::Dir(_)) => false,
            c => nested(c).iter().all(compound_only),
        })
}

/// Serializes components as one compound (`*` when none), parsed again.
pub fn compound_of<'a>(components: impl IntoIterator<Item = &'a Component>) -> Selector {
    let mut text = String::new();
    for c in components {
        if !matches!(c, Component::Combinator(_)) {
            let _ = c.to_css(&mut text);
        }
    }
    if text.is_empty() {
        text.push('*');
    }
    let mut list = Selector::parse_list(&text).expect("a serialized compound parses");
    list.swap_remove(0)
}

/// A `::part()` selector's host compound with the part and what follows it,
/// when the selector also reads the host's tree: its ancestors, siblings or
/// position. A built-in control's parts are a shadow tree under the host,
/// so that part of the selector is matched on the host where it stands in
/// its document ([`Selector::host_marker`]), and this, on the part.
fn part_local(selector: &Inner) -> Option<Selector> {
    let (compounds, _) = compounds(selector);
    let host = compounds.last()?;
    if !host.iter().any(|c| matches!(c, Component::Part(_))) {
        return None;
    }
    if compounds.len() == 1 && !host.iter().any(|c| structural_component(c)) {
        return None;
    }
    let mut text = String::new();
    for c in host.iter().filter(|c| !structural_component(c)) {
        let _ = c.to_css(&mut text);
    }
    if text.starts_with("::") {
        text.insert(0, '*');
    }
    let mut list = Selector::parse_list(&text).ok()?;
    Some(list.swap_remove(0))
}

/// A parse error's message.
fn explain(error: &ParseError<OgError>) -> String {
    match &error.kind {
        ParseErrorKind::Custom(OgError::Message(m)) => m.clone(),
        ParseErrorKind::Custom(OgError::Kind(kind)) => format!("{kind:?}"),
        ParseErrorKind::Basic(kind) => format!("{kind:?}"),
    }
}

/// og's own refusals of what the crate parses.
fn check(selector: &Inner) -> Result<(), String> {
    // Inside its component a host has no features of its own (CSS Scoping
    // 1 §3.2.1): `:host.wide` and `:host:hover` never match, so they are
    // an author's slip for `:host(.wide)` and `:host(:hover)`.
    for (compound, _) in segments(selector) {
        let featured = compound.iter().find(|c| {
            !matches!(
                c,
                Component::Host(_)
                    | Component::Part(_)
                    | Component::PseudoElement(_)
                    | Component::ImplicitScope
                    | Component::ExplicitUniversalType
            )
        });
        if let Some(feature) = featured
            && compound.iter().any(|c| matches!(c, Component::Host(_)))
        {
            let feature = feature.to_css_string();
            return Err(format!(
                "`:host{feature}` never matches: inside its component a host has no features of its own; write `:host({feature})`"
            ));
        }
    }
    for c in selector.iter_raw_parse_order_from(0) {
        if let Component::NthOf(nth) = c
            && let Some(bad) = nth.selectors().iter().find(|s| !compound_only(s))
        {
            return Err(format!(
                "`of {}` takes compound selectors without combinators or structural pseudo-classes",
                bad.to_css_string()
            ));
        }
        for inner in nested(c) {
            check(inner)?;
        }
        if let Component::Has(list) = c {
            for relative in list.iter() {
                check(&relative.selector)?;
            }
        }
    }
    Ok(())
}

impl Selector {
    pub fn parse_list(text: &str) -> Result<Vec<Self>, String> {
        let fail = |e: String| format!("invalid selector `{text}`: {e}");
        let mut input = CssParser::new(text);
        let items = input
            .parse_comma_separated(|input| {
                let start = input.position();
                while input.next().is_ok() {}
                Ok::<_, ParseError<OgError>>(input.slice_from(start).trim())
            })
            .map_err(|e| fail(explain(&e)))?;
        let mut list = Vec::with_capacity(items.len());
        for item in items {
            let (selector, span) = span_tests(item);
            let mut input = CssParser::new(&selector);
            let inner = input
                .parse_entirely(|input| Inner::parse(&OgParser, input))
                .map_err(|e| {
                    let hint = match item.contains("::slotted(") {
                        true => "; ::slotted() takes one compound selector and ends the selector: write a state inside it, as ::slotted(p:hover)",
                        false => "",
                    };
                    fail(format!("{}{hint}", explain(&e)))
                })?;
            check(&inner).map_err(fail)?;
            let local = part_local(&inner).map(std::sync::Arc::new);
            list.push(Selector {
                source: item.to_string(),
                global: false,
                inner,
                span,
                local,
            });
        }
        Ok(list)
    }

    pub fn specificity(&self) -> Specificity {
        let Specificity(ids, classes, types) = self.inner.specificity().into();
        Specificity(ids, classes + self.span, types)
    }

    /// Its compounds in parse order and the combinators between them
    /// ([`compounds`]).
    pub fn compounds(&self) -> (Vec<Vec<&Component>>, Vec<Combinator>) {
        compounds(&self.inner)
    }

    /// Every component, nested lists included, in parse order.
    pub fn any_component(&self, f: &mut impl FnMut(&Component) -> bool) -> bool {
        fn walk(s: &Inner, f: &mut impl FnMut(&Component) -> bool) -> bool {
            s.iter_raw_parse_order_from(0).any(|c| {
                f(c) || nested(c).iter().any(|s| walk(s, f))
                    || matches!(c, Component::Has(list) if list.iter().any(|r| walk(&r.selector, f)))
            })
        }
        walk(&self.inner, f)
    }

    /// The custom property a part's host declares on itself while the part of
    /// this selector before `::part()` matches it where it stands in its
    /// document ([`Selector::local`]): the part's cascade reads the host's
    /// tree through it, as a shadow tree's part is matched through its host.
    pub fn host_marker(&self) -> String {
        use std::hash::{Hash, Hasher};
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        self.source.hash(&mut hash);
        format!("--og-part-host-{:016x}", hash.finish())
    }

    /// For a selector with `:host` that is matched on the instantiated
    /// tree: the selector with each `:host(…)` written as the element it
    /// selects (`:root`, keeping what its argument reads of the tree, as
    /// `:nth-child()`), and what the arguments ask of the host beside
    /// that, as the condition the rule holds under.
    pub(super) fn without_host(
        &self,
        doc: &UiDocument,
    ) -> Result<Option<(Selector, Expr)>, String> {
        let host = |c: &Component| matches!(c, Component::Host(_));
        if !self.any_component(&mut |c| host(c)) {
            return Ok(None);
        }
        let runtime = || unreachable!("structural simple selectors stay in the selector");
        let mut text = String::new();
        let mut condition = Expr::Lit(true.into());
        for (compound, after) in segments(&self.inner) {
            for c in compound {
                match c {
                    Component::Host(argument) => {
                        text.push_str(":root");
                        let simple = argument.iter().flat_map(|a| a.iter_raw_parse_order_from(0));
                        let (tree, own): (Vec<_>, Vec<_>) =
                            simple.partition(|c| structural_component(c));
                        for c in tree {
                            let _ = c.to_css(&mut text);
                        }
                        condition = combine(Op::And, condition, host_features(own, doc, &runtime)?);
                    }
                    c if nested(c)
                        .iter()
                        .any(|s| s.iter_raw_parse_order_from(0).any(|c| walk_any(c, &host))) =>
                    {
                        return Err(format!(
                            "{}: write :host(…) outside :is(), :not(), :where() and :nth-child(… of) in a selector that reads sibling order or :has()",
                            self.source
                        ));
                    }
                    c => {
                        let _ = c.to_css(&mut text);
                    }
                }
            }
            if let Some(combinator) = after {
                let _ = combinator.to_css(&mut text);
            }
        }
        let mut list = Selector::parse_list(&text)?;
        let mut selector = list.swap_remove(0);
        selector.global = self.global;
        Ok(Some((selector, condition)))
    }

    /// Whether its subject is content rendered at a `{@render}` site
    /// (`.body::slotted(p)`): the selector is its document's, the subject
    /// another's.
    pub fn slotted(&self) -> bool {
        self.inner
            .iter_raw_match_order()
            .take_while(|c| !matches!(c, Component::Combinator(_)))
            .any(|c| matches!(c, Component::Slotted(_)))
    }

    /// The control document element its subject is (`details::details-content`),
    /// which the host's document reaches as it reaches a part.
    pub fn pseudo_element(&self) -> Option<PseudoElement> {
        self.inner
            .iter_raw_match_order()
            .take_while(|c| !matches!(c, Component::Combinator(_)))
            .find_map(|c| match c {
                Component::PseudoElement(p) => Some(*p),
                _ => None,
            })
    }

    /// Whether matching needs the instantiated sibling order ([`structural`]).
    pub fn structural(&self) -> bool {
        structural(&self.inner)
    }

    /// The subject compound without what needs the instantiated tree: used at
    /// cook to skip elements that can never be the subject.
    pub(super) fn subject_filter(&self) -> Self {
        let (compounds, _) = self.compounds();
        let subject = compounds.last().expect("a selector has a compound");
        let mut filter = compound_of(subject.iter().copied().filter(|c| !structural_component(c)));
        filter.global = self.global;
        filter
    }

    /// The selector with `f`'s text in place of the components it names;
    /// `f` sees each component with the compound it is in.
    pub(super) fn rewritten(
        &self,
        f: impl Fn(&[&Component], &Component) -> Option<&'static str>,
    ) -> Self {
        let (compounds, _) = self.compounds();
        let mut text = String::new();
        let mut k = 0;
        for (components, after) in segments(&self.inner) {
            for c in components {
                match f(&compounds[k], c) {
                    Some(replacement) => text.push_str(replacement),
                    None => {
                        let _ = c.to_css(&mut text);
                    }
                }
            }
            if let Some(combinator) = after {
                let _ = combinator.to_css(&mut text);
                k += usize::from(!joins(combinator));
            }
        }
        let mut list = Self::parse_list(&text).expect("a rewritten selector parses");
        let mut out = list.swap_remove(0);
        out.source = self.source.clone();
        out.global = self.global;
        out.span = self.span;
        out
    }

    pub(super) fn condition(
        &self,
        doc: &UiDocument,
        elements: &Elements<'_>,
        index: usize,
    ) -> Result<Expr, String> {
        condition(&self.inner, &self.source, doc, elements, index)
    }
}

fn condition(
    selector: &Inner,
    source: &str,
    doc: &UiDocument,
    elements: &Elements<'_>,
    index: usize,
) -> Result<Expr, String> {
    let runtime = || format!("{source} needs the instantiated tree; it is matched at runtime");
    let (compounds, combinators) = compounds(selector);
    #[allow(clippy::too_many_arguments)]
    fn chain(
        compounds: &[Vec<&Component>],
        combinators: &[Combinator],
        source: &str,
        runtime: &dyn Fn() -> String,
        doc: &UiDocument,
        elements: &Elements<'_>,
        index: usize,
        part: usize,
    ) -> Result<Expr, String> {
        // A control document's host, above its root: featureless to the
        // document but for `:host`, with nothing of its own tree beyond
        // it (CSS Scoping 1 §3.2.1).
        if index == super::HOST_NODE {
            let mut own = Expr::Lit(true.into());
            for c in &compounds[part] {
                let test = match c {
                    Component::Host(None) => continue,
                    Component::Host(Some(argument)) => {
                        host_features(argument.iter_raw_parse_order_from(0), doc, runtime)?
                    }
                    _ => return Ok(Expr::Lit(false.into())),
                };
                own = combine(Op::And, own, test);
            }
            return Ok(if part == 0 {
                own
            } else {
                Expr::Lit(false.into())
            });
        }
        let Some((_, tag, attrs)) = elements.get(index) else {
            return Ok(Expr::Lit(false.into()));
        };
        let mut own = Expr::Lit(true.into());
        let expression = |attr: &Attr| {
            super::syntax::expression(&attr.value, attr.at).map_err(|(_, error)| error)
        };
        let attribute_test = |name: &str, test: Option<AttrTest>| -> Result<Expr, String> {
            Ok(match attrs.iter().find(|a| a.name == *name) {
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
                        Some(test) => Expr::AttrTest(Box::new(value), test),
                    }
                }
            })
        };
        for c in &compounds[part] {
            let test = match c {
                Component::LocalName(name) => Expr::Lit(type_matches(tag, &name.name.0).into()),
                Component::ExplicitUniversalType
                | Component::ExplicitAnyNamespace
                | Component::ExplicitNoNamespace
                | Component::DefaultNamespace(_)
                | Component::Namespace(..)
                | Component::ImplicitScope
                | Component::NonTSPseudoClass(PseudoClass::Defined) => Expr::Lit(true.into()),
                // A control document's root stands under its host: it is
                // no `:root`, so the style's root tokens are not declared
                // again between a control and its parts.
                Component::Root => Expr::Lit((index == doc.root && doc.control.is_none()).into()),
                Component::Scope => Expr::Lit((index == doc.root).into()),
                Component::ID(name) => Expr::Lit(
                    attrs
                        .iter()
                        .any(|a| a.name == "id" && a.value == name.0)
                        .into(),
                ),
                Component::Class(name) => {
                    let name = &name.0;
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
                    if let Some(attr) = attrs.iter().find(|a| a.name == format!("class:{name}")) {
                        matches = combine(Op::Or, matches, expression(attr)?);
                    }
                    // Only a variant's word can name one: a bound variant
                    // is one of them, so another class never matches it.
                    if variant_is_class(Some(tag))
                        && button_variant(name)
                        && let Some(attr) = attrs.iter().find(|a| a.name == "variant")
                    {
                        let variant = if attr.bound {
                            Expr::HasToken(Box::new(expression(attr)?), name.clone())
                        } else {
                            Expr::Lit((attr.value == *name).into())
                        };
                        matches = combine(Op::Or, matches, variant);
                    }
                    matches
                }
                Component::AttributeInNoNamespaceExists { .. }
                | Component::AttributeInNoNamespace { .. }
                | Component::AttributeOther(_) => {
                    let (name, test) = attribute(c).expect("an attribute selector");
                    if name == "open" && reflects_open(tag) {
                        // HTML reflects a disclosure's or a dialog's open
                        // state as its `open` attribute (value "").
                        if test.is_none_or(|t| t.test("")) {
                            Expr::InteractionAt {
                                node: index,
                                state: InteractionState::Open,
                            }
                        } else {
                            Expr::Lit(false.into())
                        }
                    } else {
                        attribute_test(name, test)?
                    }
                }
                Component::NonTSPseudoClass(PseudoClass::AnyLink) => {
                    if matches!(tag.as_str(), "a" | "area") {
                        attribute_test("href", None)?
                    } else {
                        Expr::Lit(false.into())
                    }
                }
                Component::NonTSPseudoClass(PseudoClass::Lang(ranges)) => {
                    match inherited(doc, elements, index, "lang") {
                        Some(attr) if !attr.bound => Expr::Lit(
                            ranges
                                .iter()
                                .any(|r| crate::bind::lang_matches(r, &attr.value))
                                .into(),
                        ),
                        Some(attr) => {
                            Expr::Lang(Some(Box::new(expression(attr)?)), ranges.to_vec())
                        }
                        None => Expr::Lang(None, ranges.to_vec()),
                    }
                }
                Component::NonTSPseudoClass(PseudoClass::Dir(rtl)) => {
                    match inherited(doc, elements, index, "dir") {
                        Some(attr)
                            if !attr.bound && matches!(attr.value.as_str(), "ltr" | "rtl") =>
                        {
                            Expr::Lit(((attr.value == "rtl") == *rtl).into())
                        }
                        Some(attr) if attr.bound => {
                            Expr::Dir(Some(Box::new(expression(attr)?)), *rtl)
                        }
                        _ => Expr::Dir(None, *rtl),
                    }
                }
                Component::NonTSPseudoClass(PseudoClass::Screen(s)) => Expr::Screen(*s),
                Component::NonTSPseudoClass(PseudoClass::Custom(name)) => {
                    match attrs.iter().find(|a| a.name == format!("state:{}", name.0)) {
                        Some(attr) if attr.bound => {
                            let value = expression(attr)?;
                            combine(
                                Op::And,
                                value.clone(),
                                Expr::Bin(
                                    Op::Ne,
                                    Box::new(value),
                                    Box::new(Expr::Lit("false".into())),
                                ),
                            )
                        }
                        Some(attr) => Expr::Lit(custom_state_on(&attr.value.clone().into()).into()),
                        None => Expr::Lit(false.into()),
                    }
                }
                Component::NonTSPseudoClass(PseudoClass::State(state)) => Expr::InteractionAt {
                    node: index,
                    state: *state,
                },
                Component::NonTSPseudoClass(PseudoClass::Heading(levels)) => {
                    Expr::Lit(PseudoClass::heading_matches(levels, tag).into())
                }
                Component::NonTSPseudoClass(PseudoClass::ViewTransitionType(types)) => {
                    if doc
                        .layer
                        .transition
                        .as_ref()
                        .is_some_and(|t| t.types.iter().any(|d| types.iter().any(|n| n.0 == *d)))
                    {
                        Expr::InteractionAt {
                            node: index,
                            state: InteractionState::ActiveViewTransition,
                        }
                    } else {
                        Expr::Lit(false.into())
                    }
                }
                Component::Negation(_) | Component::Is(_) | Component::Where(_) => {
                    let mut matches = Expr::Lit(false.into());
                    for inner in nested(c) {
                        matches = combine(
                            Op::Or,
                            matches,
                            condition(inner, source, doc, elements, index)?,
                        );
                    }
                    if matches!(c, Component::Negation(_)) {
                        match matches {
                            Expr::Lit(serde_json::Value::Bool(value)) => Expr::Lit((!value).into()),
                            value => Expr::Not(Box::new(value)),
                        }
                    } else {
                        matches
                    }
                }
                Component::Host(_) if index != host_node(doc) => Expr::Lit(false.into()),
                Component::Host(None) => Expr::Lit(true.into()),
                Component::Host(Some(argument)) => {
                    host_features(argument.iter_raw_parse_order_from(0), doc, runtime)?
                }
                // Parts are matched against a part's Facts at runtime,
                // never against its host as an ordinary document node.
                Component::Part(_) => Expr::Lit(false.into()),
                Component::Nth(_)
                | Component::NthOf(_)
                | Component::Empty
                | Component::Has(_)
                | Component::Slotted(_)
                | Component::NonTSPseudoClass(PseudoClass::HasSlotted)
                | Component::RelativeSelectorAnchor => return Err(runtime()),
                _ => Expr::Lit(false.into()),
            };
            own = combine(Op::And, own, test);
        }
        if part == 0 {
            return Ok(own);
        }
        let combinator = combinators[part - 1];
        if combinator.is_sibling() {
            return Err(runtime());
        }
        let mut candidates = Vec::new();
        let mut parent = doc.nodes[index].parent;
        while parent != NONE {
            if elements.get(parent).is_some() {
                candidates.push(parent);
                if combinator == Combinator::Child {
                    break;
                }
            }
            parent = doc.nodes[parent].parent;
        }
        // Above a control document's root stands its host.
        if doc.control.is_some()
            && parent == NONE
            && (candidates.is_empty() || combinator != Combinator::Child)
        {
            candidates.push(super::HOST_NODE);
        }
        let mut matches = Expr::Lit(false.into());
        for candidate in candidates {
            matches = combine(
                Op::Or,
                matches,
                chain(
                    compounds,
                    combinators,
                    source,
                    runtime,
                    doc,
                    elements,
                    candidate,
                    part - 1,
                )?,
            );
        }
        Ok(combine(Op::And, own, matches))
    }
    chain(
        &compounds,
        &combinators,
        source,
        &runtime,
        doc,
        elements,
        index,
        compounds.len() - 1,
    )
}

/// The authored attribute as it exists in HTML, shared by compiled
/// predicates and the retained selector tree. Bound boolean attributes
/// are present with an empty value when true and absent when false.
pub fn attribute_value(attr: &Attr) -> Result<Expr, super::syntax::Error> {
    if !attr.bound && !attr.name.starts_with("class:") {
        return Ok(Expr::Lit(attr.value.clone().into()));
    }
    let value = super::syntax::expression(&attr.value, attr.at)?;
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

/// The `name` attribute of the element or its nearest ancestor that has
/// one, within the document.
fn inherited<'a>(
    doc: &UiDocument,
    elements: &Elements<'a>,
    mut index: usize,
    name: &str,
) -> Option<&'a Attr> {
    while index != NONE {
        if let Some((_, _, attrs)) = elements.get(index)
            && let Some(attr) = attrs.iter().find(|a| a.name == name)
        {
            return Some(attr);
        }
        index = doc.nodes[index].parent;
    }
    None
}

// What a simple selector asks of an element's own facts, read the same way
// where a selector is lowered at cook ([`Selector::condition`]) and where it
// is matched on the instantiated tree ([`super::matching`]): one predicate
// each, so the two cannot disagree.

/// Whether an element of `tag` is of type `name`: an HTML element's type
/// matches caselessly (Selectors 4 §5.1, HTML §4.16.2).
pub(crate) fn type_matches(tag: &str, name: &str) -> bool {
    tag == name || tag.eq_ignore_ascii_case(name)
}

/// Whether a `state:name` value turns `:state(name)` on: a true value
/// other than the word "false".
pub(crate) fn custom_state_on(value: &serde_json::Value) -> bool {
    crate::bind::truthy(value) && value.as_str() != Some("false")
}

/// Whether `name` is one of a button's variants (`variant="primary"`).
fn button_variant(name: &str) -> bool {
    matches!(
        super::schema::prop("variant").map(|(_, p)| &p.ty),
        Some(super::schema::Ty::Word(words)) if words.contains(&name)
    )
}

/// Whether an element of `tag` counts its `variant` among its classes, as
/// a button does (`button.primary`, `button.primary::part(base)`).
pub(crate) fn variant_is_class(tag: Option<&str>) -> bool {
    tag == Some("button")
}

pub(crate) fn combine(op: Op, a: Expr, b: Expr) -> Expr {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn functions_and_quoted_attributes_do_not_split_selector_lists() {
        let selectors = Selector::parse_list(".card:hover>.title:not([label='a, b'], :is(#x, .other)), :where(#zero) text::part(emphasis)").unwrap();
        assert_eq!(selectors.len(), 2);
        assert_eq!(selectors[0].specificity(), Specificity(1, 3, 0));
        assert_eq!(selectors[1].specificity(), Specificity(0, 0, 2));
        assert_eq!(selectors[0].compounds().1, vec![Combinator::Child]);
        assert_eq!(
            selectors[0].source,
            ".card:hover>.title:not([label='a, b'], :is(#x, .other))"
        );
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
            structural[0].compounds().1,
            vec![
                Combinator::NextSibling,
                Combinator::LaterSibling,
                Combinator::Child
            ]
        );
        assert_eq!(structural[0].specificity(), Specificity(0, 6, 1));
        assert!(!selectors[0].structural());
        let has = &Selector::parse_list(".card:has(> .x + .y, .z)").unwrap()[0];
        assert!(has.structural());
        assert_eq!(has.specificity(), Specificity(0, 3, 0));
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
            let selector = &Selector::parse_list(text).unwrap()[0];
            let (compounds, _) = selector.compounds();
            let Some((_, Some(test))) = attribute(compounds[0][0]) else {
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
            "::before",
        ] {
            assert!(Selector::parse_list(bad).is_err(), "accepted {bad}");
        }
        // A part list names a part by every name (CSS Shadow Parts §4.1).
        let parts = &Selector::parse_list("button.primary::part(base face):hover").unwrap()[0];
        let (compounds, combinators) = parts.compounds();
        assert_eq!((compounds.len(), combinators.len()), (1, 0));
        assert_eq!(parts.specificity(), Specificity(0, 2, 2));
    }

    #[test]
    fn serialized_compounds_and_rewrites_parse_back() {
        let s = &Selector::parse_list(":root[data-scheme=light] .a:lang(en):state(open):hover")
            .unwrap()[0];
        let rewritten = s.rewritten(|compound, c| {
            (compound.iter().any(|c| matches!(c, Component::Root))
                && attribute(c).is_some_and(|(name, _)| name == "data-scheme"))
            .then_some(":root")
        });
        assert_eq!(
            rewritten.inner.to_css_string(),
            ":root:root .a:lang(\"en\"):state(open):hover"
        );
        assert_eq!(rewritten.specificity(), s.specificity());
        assert_eq!(rewritten.source, s.source);
        let filter = Selector::parse_list(".a:first-child:hover").unwrap()[0].subject_filter();
        assert_eq!(filter.inner.to_css_string(), ".a:hover");
    }
}
