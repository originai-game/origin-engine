//! The arena as an AccessKit tree (port of rune's `a11y/tree.rs`, MIT,
//! `third_party/rune-a11y`). What an element is for assistive technology
//! (its role, name, states and range) is the [`Semantics`] the op applier
//! writes from a node's tag, its a11y props and its listeners
//! (CONVENTIONS.md §Accessibility); its box and text are the arena's.
//!
//! An update that read nothing changed costs one stamp comparison. Else one
//! walk over the arena finds the tree's nodes and a fingerprint of all each
//! shows (its semantics, its role, its box, its children, the text it is
//! named by); only the nodes whose fingerprint moved are built and sent.
//! A node the walk no longer finds leaves with its parent's new children,
//! as AccessKit removes it. Node ids are arena ids: the arena's root is the
//! tree's root, a window the size of the viewport.

pub use accesskit;
use accesskit::{
    Action, Affine, AriaCurrent, Live, Node, NodeId, Rect, Role, Toggled, TreeId, TreeInfo,
    TreeUpdate,
};
use rustc_hash::FxHashMap;
use std::hash::{Hash, Hasher};

use crate::arena::{Arena, Id};

/// The tree's root: the arena's root element, the viewport.
pub const ROOT: NodeId = NodeId(0);

/// Text a node is named by from its content stops here, in chars.
const NAME_MAX: usize = 400;

/// What an element is, by the tag it was created with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Kind {
    #[default]
    View,
    Text,
    Image,
    Icon,
    Scroll,
    Input,
    /// A `<Line>` (tag 6).
    Line,
    /// A `<Particles>` (tag 7): decoration, an image when labelled.
    Particles,
    /// A `Show`/`For` anchor: never a node.
    Placeholder,
}

impl Kind {
    /// A `create` op's tag code (CONVENTIONS.md §Ops: 0 View, 1 Text,
    /// 2 Image, 3 Scroll, 4 Input, 5 Icon, 6 Line, 7 Particles).
    pub fn of(tag: u8) -> Kind {
        match tag {
            1 => Kind::Text,
            2 => Kind::Image,
            3 => Kind::Scroll,
            4 => Kind::Input,
            5 => Kind::Icon,
            6 => Kind::Line,
            7.. => Kind::Particles,
            0 => Kind::View,
        }
    }
}

/// One element's semantics, parsed once from its props.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sem {
    pub kind: Kind,
    /// `role`: `Some(None)` for `none`/`presentation` (left out, its
    /// children join its parent).
    pub role: Option<Option<Role>>,
    pub label: Option<Box<str>>,
    pub description: Option<Box<str>>,
    /// An input's text (`value`).
    pub value: Option<Box<str>>,
    /// A range's value as words (`valueText`).
    pub value_text: Option<Box<str>>,
    pub checked: Option<Toggled>,
    pub pressed: Option<Toggled>,
    pub selected: Option<bool>,
    pub expanded: Option<bool>,
    pub busy: bool,
    /// Its value fails the game's checks (`invalid`).
    pub invalid: bool,
    pub hidden: bool,
    pub modal: bool,
    pub current: bool,
    /// On its way out (an exit animation plays): no longer in the tree.
    pub exiting: bool,
    /// An anchored node's placed top left on the surface, and whether its
    /// target is off the screen (`away`): then no longer in the tree.
    pub at: Option<[f32; 2]>,
    pub away: bool,
    pub disabled: bool,
    /// `tabIndex`.
    pub tab_index: Option<i32>,
    /// It listens to `press` or `tap`.
    pub press: bool,
    pub level: Option<u8>,
    pub live: Option<Live>,
    /// `valueMin`, `valueMax`, `valueNow`.
    pub range: [Option<f64>; 3],
    /// A hash of all the above, kept as it changes.
    print: u64,
}

impl Sem {
    /// The keyboard reaches it.
    pub fn focusable(&self) -> bool {
        match self.tab_index {
            Some(t) => t >= 0,
            None => self.press || self.kind == Kind::Input,
        }
    }

    fn reprint(&mut self) {
        let mut h = rustc_hash::FxHasher::default();
        let s = &*self;
        (
            s.kind,
            s.role,
            &s.label,
            &s.description,
            &s.value,
            &s.value_text,
        )
            .hash(&mut h);
        (
            s.checked, s.pressed, s.selected, s.expanded, s.level, s.live,
        )
            .hash(&mut h);
        (
            s.busy,
            s.invalid,
            s.hidden,
            s.exiting,
            s.modal,
            s.current,
            s.disabled,
            s.tab_index,
            s.press,
        )
            .hash(&mut h);
        s.range.map(|v| v.map(f64::to_bits)).hash(&mut h);
        self.print = h.finish();
    }
}

static PLAIN: Sem = Sem {
    kind: Kind::View,
    role: None,
    label: None,
    description: None,
    value: None,
    value_text: None,
    checked: None,
    pressed: None,
    selected: None,
    expanded: None,
    busy: false,
    invalid: false,
    hidden: false,
    modal: false,
    current: false,
    exiting: false,
    at: None,
    away: false,
    disabled: false,
    tab_index: None,
    press: false,
    level: None,
    live: None,
    range: [None; 3],
    print: 0,
};

/// Every element's semantics by arena id, as the op applier writes them.
#[derive(Clone, Debug, Default)]
pub struct Semantics {
    nodes: Vec<Sem>,
    /// Counts changes, for the tree's stamp.
    rev: u64,
    /// How many elements are modal: the walk looks for the topmost only
    /// when one is.
    modals: u32,
    /// The box each transformed node is drawn in on screen (x, y, w, h),
    /// where it is not its layout box: a node scaling in is where it shows.
    drawn: FxHashMap<Id, [f32; 4]>,
}

impl Semantics {
    pub fn get(&self, id: Id) -> &Sem {
        self.nodes.get(id as usize).unwrap_or(&PLAIN)
    }

    /// The boxes the transformed nodes are drawn in after a paint (a
    /// motion's scale, a translate): a node's bounds are where it is drawn,
    /// so a reader's ring lands on it. Changes the stamp only when they did.
    pub fn set_drawn(&mut self, drawn: FxHashMap<Id, [f32; 4]>) {
        if drawn != self.drawn {
            self.drawn = drawn;
            self.rev += 1;
        }
    }

    fn at(&mut self, id: Id) -> &mut Sem {
        let i = id as usize;
        if i >= self.nodes.len() {
            self.nodes.resize(i + 1, Sem::default());
        }
        &mut self.nodes[i]
    }

    /// `create`, `createText` (a [`Kind::Text`]) and `placeholder`.
    pub fn create(&mut self, id: Id, kind: Kind) {
        if self.get(id).modal {
            self.modals -= 1;
        }
        let s = self.at(id);
        *s = Sem {
            kind,
            ..Sem::default()
        };
        s.reprint();
        self.rev += 1;
    }

    /// `release`: the element is gone.
    pub fn release(&mut self, id: Id) {
        if (id as usize) < self.nodes.len() {
            self.create(id, Kind::View);
        }
    }

    /// Whether it is on its way out (removed or released while its exit
    /// plays): the tree leaves it out at once.
    pub fn exiting(&mut self, id: Id, on: bool) {
        if self.get(id).exiting != on {
            let s = self.at(id);
            s.exiting = on;
            s.reprint();
            self.rev += 1;
        }
    }

    /// Where an anchored node was placed (`Some`), off the screen (`None`
    /// while anchored), or back in flow: [`Tree::place_anchors`].
    pub fn place(&mut self, id: Id, at: Option<[f32; 2]>) {
        let s = self.at(id);
        let away = at.is_none();
        if s.away != away || (at.is_some() && s.at != at) {
            s.away = away;
            if at.is_some() {
                s.at = at;
            }
            self.rev += 1;
        }
    }

    /// The node comes back into flow: no longer placed.
    pub fn unplace(&mut self, id: Id) {
        let s = self.at(id);
        if s.at.is_some() || s.away {
            (s.at, s.away) = (None, false);
            self.rev += 1;
        }
    }

    /// Whether it listens to `press` or `tap`.
    pub fn pressable(&mut self, id: Id, on: bool) {
        if self.get(id).press != on {
            let s = self.at(id);
            s.press = on;
            s.reprint();
            self.rev += 1;
        }
    }

    /// `setProp` with its value as JSON text (`null` or empty unsets it;
    /// unquoted text is a string). Answers
    /// whether the name is one of §Accessibility's; others are left to
    /// their owners.
    pub fn prop(&mut self, id: Id, name: &str, value: &str) -> bool {
        let v = Scalar::parse(value);
        let was = self.get(id).clone();
        let s = self.at(id);
        match name {
            "role" => s.role = v.text().map(role_of),
            "label" => s.label = v.text().map(Into::into),
            "description" => s.description = v.text().map(Into::into),
            "value" => s.value = v.words().map(Into::into),
            "valueText" => s.value_text = v.text().map(Into::into),
            "checked" => s.checked = v.toggled(),
            "pressed" => s.pressed = v.toggled(),
            "selected" => s.selected = v.flag(),
            "expanded" => s.expanded = v.flag(),
            "busy" => s.busy = v.flag() == Some(true),
            "invalid" => s.invalid = v.flag() == Some(true),
            "hidden" => s.hidden = v.flag() == Some(true),
            "modal" => s.modal = v.flag() == Some(true),
            "current" => s.current = v.flag() == Some(true),
            "disabled" => s.disabled = v.flag() == Some(true),
            "tabIndex" => s.tab_index = v.number().map(|n| n as i32),
            "level" => {
                s.level = v
                    .number()
                    .filter(|n| (1.0..=6.0).contains(n))
                    .map(|n| n as u8)
            }
            "live" => {
                s.live = match v.text() {
                    Some("polite") => Some(Live::Polite),
                    Some("assertive") => Some(Live::Assertive),
                    Some("off") => Some(Live::Off),
                    _ => None,
                }
            }
            "valueMin" => s.range[0] = v.number(),
            "valueMax" => s.range[1] = v.number(),
            "valueNow" => s.range[2] = v.number(),
            _ => return false,
        }
        s.reprint();
        let modal = s.modal;
        if *s != was {
            self.rev += 1;
            match (was.modal, modal) {
                (false, true) => self.modals += 1,
                (true, false) => self.modals -= 1,
                _ => {}
            }
        }
        true
    }
}

/// A prop's JSON scalar.
enum Scalar<'a> {
    Null,
    Bool(bool),
    Number(f64),
    Text(std::borrow::Cow<'a, str>),
}

impl<'a> Scalar<'a> {
    fn parse(json: &'a str) -> Scalar<'a> {
        let t = json.trim();
        match t {
            "null" | "" => Scalar::Null,
            "true" => Scalar::Bool(true),
            "false" => Scalar::Bool(false),
            _ if t.starts_with('"') => match serde_json::from_str::<std::borrow::Cow<'a, str>>(t) {
                Ok(s) => Scalar::Text(s),
                Err(_) => Scalar::Null,
            },
            _ => t.parse().map_or(Scalar::Text(t.into()), Scalar::Number),
        }
    }

    fn text(&self) -> Option<&str> {
        match self {
            Scalar::Text(s) => Some(s),
            _ => None,
        }
    }

    /// A string, or a number or bool as its words (an input's value).
    fn words(&self) -> Option<String> {
        match self {
            Scalar::Text(s) => Some(s.to_string()),
            Scalar::Number(n) => Some(n.to_string()),
            Scalar::Bool(b) => Some(b.to_string()),
            Scalar::Null => None,
        }
    }

    fn flag(&self) -> Option<bool> {
        match self {
            Scalar::Bool(b) => Some(*b),
            _ => None,
        }
    }

    fn number(&self) -> Option<f64> {
        match self {
            Scalar::Number(n) if n.is_finite() => Some(*n),
            _ => None,
        }
    }

    fn toggled(&self) -> Option<Toggled> {
        match self {
            Scalar::Bool(b) => Some(Toggled::from(*b)),
            Scalar::Text(s) if s == "mixed" => Some(Toggled::Mixed),
            _ => None,
        }
    }
}

/// An ARIA role name's AccessKit role; `None` for `none` and
/// `presentation`, a group for one it does not know.
fn role_of(name: &str) -> Option<Role> {
    Some(match name {
        "none" | "presentation" => return None,
        "button" => Role::Button,
        "link" => Role::Link,
        "heading" => Role::Heading,
        "image" | "img" => Role::Image,
        "text" => Role::Label,
        "checkbox" => Role::CheckBox,
        "radio" => Role::RadioButton,
        "switch" => Role::Switch,
        "slider" => Role::Slider,
        "spinbutton" => Role::SpinButton,
        "textbox" => Role::TextInput,
        "combobox" => Role::ComboBox,
        "listbox" => Role::ListBox,
        "option" => Role::ListBoxOption,
        "tab" => Role::Tab,
        "tablist" => Role::TabList,
        "tabpanel" => Role::TabPanel,
        "radiogroup" => Role::RadioGroup,
        "progressbar" => Role::ProgressIndicator,
        "meter" => Role::Meter,
        "timer" => Role::Timer,
        "status" => Role::Status,
        "alert" => Role::Alert,
        "log" => Role::Log,
        "dialog" => Role::Dialog,
        "alertdialog" => Role::AlertDialog,
        "tooltip" => Role::Tooltip,
        "separator" => Role::Splitter,
        "list" => Role::List,
        "listitem" => Role::ListItem,
        "menu" => Role::Menu,
        "menubar" => Role::MenuBar,
        "menuitem" => Role::MenuItem,
        "menuitemcheckbox" => Role::MenuItemCheckBox,
        "menuitemradio" => Role::MenuItemRadio,
        "toolbar" => Role::Toolbar,
        "grid" => Role::Grid,
        "table" => Role::Table,
        "row" => Role::Row,
        "columnheader" => Role::ColumnHeader,
        "rowheader" => Role::RowHeader,
        "gridcell" | "cell" => Role::GridCell,
        "tree" => Role::Tree,
        "treeitem" => Role::TreeItem,
        "searchbox" => Role::SearchInput,
        "scrollbar" => Role::ScrollBar,
        "article" => Role::Article,
        "banner" => Role::Banner,
        "complementary" => Role::Complementary,
        "contentinfo" => Role::ContentInfo,
        "form" => Role::Form,
        "search" => Role::Search,
        "region" => Role::Region,
        "navigation" => Role::Navigation,
        "main" => Role::Main,
        _ => Role::Group,
    })
}

/// A node whose content is one leaf a reader hears as one thing (ARIA's
/// children presentational): named or valued by the text inside it, which
/// is not a node of its own.
fn leaf(role: Role) -> bool {
    matches!(
        role,
        Role::Label
            | Role::Heading
            | Role::Button
            | Role::CheckBox
            | Role::RadioButton
            | Role::Switch
            | Role::Tab
            | Role::ListBoxOption
            | Role::Link
            | Role::MenuItem
            | Role::MenuItemCheckBox
            | Role::MenuItemRadio
            | Role::Image
            | Role::ProgressIndicator
            | Role::Meter
            | Role::Slider
            | Role::SpinButton
            | Role::TextInput
            | Role::Splitter
    )
}

/// Element `id`'s role in the tree, or `None` when it joins its parent:
/// its `role`, else its tag's (rune's `roles::map` and `group`).
fn role(s: &Sem, run: bool) -> Option<Role> {
    if let Some(r) = s.role {
        return r;
    }
    if run {
        return Some(Role::Label);
    }
    match s.kind {
        Kind::Input => Some(Role::TextInput),
        Kind::Text => Some(Role::Label),
        Kind::Scroll => Some(Role::ScrollView),
        Kind::Image | Kind::Icon | Kind::Line | Kind::Particles => {
            s.label.is_some().then_some(Role::Image)
        }
        Kind::Placeholder => None,
        Kind::View if s.modal => Some(Role::Dialog),
        Kind::View if s.press => Some(Role::Button),
        Kind::View => (s.focusable() || s.label.is_some()).then_some(Role::Group),
    }
}

mod export;
mod tree;
pub use export::{Facts, facts, snapshot, target, web};
pub use tree::{Tree, Viewport};
