//! Built-in exposed parts. Each declaration is also a constructor: widgets
//! can use `PartSpec::reference`, while reflection consumers enumerate the
//! same declarations. Author-defined parts are not restricted to this catalogue.

/// One exposed box, addressed by a markup host selector and `::part(name)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartSpec {
    pub host: &'static str,
    pub name: &'static str,
    pub selector: &'static str,
    /// Standard state pseudo-classes emitted by the built-in widget.
    /// Variant classes and ARIA attributes are not pseudo-classes.
    pub states: &'static [&'static str],
}

impl PartSpec {
    pub fn reference(self, class: Option<crate::tokens::ElementClass>) -> crate::dress::PartRef {
        crate::dress::PartRef::parse(self.selector, class)
            .expect("reflected part has a concrete host selector")
    }
}

macro_rules! parts {
    ($($id:ident: $host:literal, $name:literal, [$($state:literal),*];)*) => {
        $(pub const $id: PartSpec = PartSpec {
            host: $host, name: $name, selector: concat!($host, "::part(", $name, ")"),
            states: &[$($state),*],
        };)*
        pub const PARTS: &[PartSpec] = &[$($id),*];
    };
}

parts! {
    BUTTON_BASE: "button", "base", ["enabled", "disabled", "hover", "active", "focus-visible"];
    BUTTON_FOCUS: "button", "focus", ["enabled", "disabled", "hover", "active", "focus-visible"];
    SWITCH_ROW: "input[type=checkbox][switch]", "row", ["enabled", "disabled", "hover", "active", "focus-visible", "checked"];
    SWITCH_TRACK: "input[type=checkbox][switch]", "track", ["enabled", "disabled", "hover", "active", "focus-visible", "checked"];
    SWITCH_KNOB: "input[type=checkbox][switch]", "knob", ["enabled", "checked"];
    CHECKBOX_ROW: "input[type=checkbox]", "row", ["enabled", "disabled", "hover", "active", "focus-visible", "checked", "invalid"];
    CHECKBOX_BOX: "input[type=checkbox]", "box", ["enabled", "disabled", "hover", "active", "focus-visible", "checked", "invalid"];
    CHECKBOX_MARK: "input[type=checkbox]", "mark", ["enabled"];
    RADIO_OPTION: "input[type=radio]", "option", ["enabled", "disabled", "hover", "active", "focus-visible", "checked"];
    RADIO_CARD: "input[type=radio]", "card", ["enabled", "disabled", "hover", "active", "focus-visible", "checked"];
    RADIO_RING: "input[type=radio]", "ring", ["enabled", "disabled", "hover", "active", "focus-visible", "checked"];
    RADIO_DOT: "input[type=radio]", "dot", ["enabled"];
    RADIO_CHOSEN: "input[type=radio]", "chosen", ["enabled"];
    TABS_TRACK: "tabs", "track", ["enabled", "disabled"];
    TABS_INDICATOR: "tabs", "indicator", ["enabled"];
    TABS_TAB: "tabs", "tab", ["enabled", "disabled", "hover", "active", "focus-visible", "checked"];
    TABS_FOCUS: "tabs", "focus", ["enabled", "disabled", "hover", "active", "focus-visible", "checked"];
    RANGE_TRACK: "input[type=range]", "track", ["enabled"];
    RANGE_FILL: "input[type=range]", "fill", ["enabled"];
    RANGE_TICK: "input[type=range]", "tick", ["enabled"];
    RANGE_CORE: "input[type=range]", "core", ["enabled"];
    RANGE_BUBBLE: "input[type=range]", "bubble", ["enabled"];
    RANGE_REACH: "input[type=range]", "reach", ["enabled", "disabled", "hover", "active", "focus-visible"];
    RANGE_KNOB: "input[type=range]", "knob", ["enabled", "disabled", "hover", "active", "focus-visible"];
    SELECT_TRIGGER: "select", "trigger", ["enabled", "disabled", "hover", "active", "focus-visible", "open", "invalid"];
    SELECT_LIST: "select", "list", ["enabled"];
    SELECT_OPTION: "select", "option", ["enabled", "disabled", "hover", "active", "focus-visible"];
    SELECT_GROUP: "select", "group", ["enabled"];
    DETAILS_ROOT: "details", "root", ["enabled"];
    DETAILS_SUMMARY: "details", "summary", ["enabled", "disabled", "hover", "active", "focus-visible", "open"];
    DETAILS_CONTENT: "details", "content", ["enabled"];
    STEPPER_FRAME: "stepper", "frame", ["enabled"];
    STEPPER_VALUE: "stepper", "value", ["enabled", "disabled", "hover", "active", "focus-visible"];
    STEPPER_BUTTON: "stepper", "button", ["enabled", "disabled", "hover", "active", "focus-visible"];
    KEYBOARD_PANEL: "keyboard", "panel", ["enabled"];
    KEYBOARD_PREVIEW: "keyboard", "preview", ["enabled"];
    KEYBOARD_CANDIDATES: "keyboard", "candidates", ["enabled"];
    KEYBOARD_CANDIDATE: "keyboard", "candidate", ["enabled", "disabled", "hover", "active", "focus-visible"];
    KEYBOARD_KEY: "keyboard", "key", ["enabled", "disabled", "hover", "active", "focus-visible"];
    INPUT_FRAME: "input", "frame", ["enabled", "disabled", "focus-visible", "invalid"];
    INPUT_TEXT: "input", "text", ["enabled", "disabled", "focus-visible", "invalid"];
    INPUT_PLACEHOLDER: "input", "placeholder", ["enabled", "disabled", "focus-visible", "invalid"];
    INPUT_REVEAL: "input", "reveal", ["enabled", "disabled", "hover", "active", "focus-visible", "invalid"];
    INPUT_REVEAL_ICON: "input", "reveal-icon", ["enabled", "disabled", "focus-visible", "invalid"];
    INPUT_CLEAR: "input", "clear", ["enabled", "disabled", "hover", "active", "focus-visible", "invalid"];
    INPUT_CLEAR_ICON: "input", "clear-icon", ["enabled", "disabled", "focus-visible", "invalid"];
    TEXTAREA_FRAME: "textarea", "frame", ["enabled", "disabled", "focus-visible", "invalid"];
    TEXTAREA_TEXT: "textarea", "text", ["enabled", "disabled", "focus-visible", "invalid"];
    TEXTAREA_PLACEHOLDER: "textarea", "placeholder", ["enabled", "disabled", "focus-visible", "invalid"];
    TEXTAREA_SELECTION: "textarea", "selection", ["enabled"];
    PANEL_FRAME: "panel", "frame", ["enabled"];
    SLOT_FRAME: "slot", "frame", ["enabled", "hover", "active", "focus-visible"];
    TOOLTIP_FRAME: "tooltip", "frame", ["enabled"];
    HUD_FRAME: "hud", "frame", ["enabled"];
    PROGRESS_TRACK: "progress", "track", ["enabled"];
    PROGRESS_FILL: "progress", "fill", ["enabled"];
    METER_TRACK: "meter", "track", ["enabled"];
    METER_FILL: "meter", "fill", ["enabled"];
    METER_TRAIL: "meter", "trail", ["enabled"];
    METER_SEPARATOR: "meter", "separator", ["enabled"];
    METER_LABEL: "meter", "label", ["enabled"];
    CHIP_BASE: "chip", "base", ["enabled", "disabled", "hover", "active", "focus-visible"];
    BADGE_BASE: "badge", "base", ["enabled"];
    SKELETON_FRAME: "skeleton", "frame", ["enabled"];
    SKELETON_SHEEN: "skeleton", "sheen", ["enabled"];
    SKELETON_LINE: "skeleton", "line", ["enabled"];
    SPINNER_BASE: "spinner", "base", ["enabled"];
    SPINNER_DOT: "spinner", "dot", ["enabled"];
    ICON_IMAGE: "og-icon", "image", ["enabled"];
    ICON_SECONDARY: "og-icon", "secondary", ["enabled"];
    ARTSLOT_ICON: "artslot", "icon", ["enabled"];
    ARTSLOT_PLACEHOLDER: "artslot", "placeholder", ["enabled"];
    ARTSLOT_LABEL: "artslot", "label", ["enabled"];
    DIALOG_BACKDROP: "dialog", "backdrop", ["enabled"];
    DIALOG_PANEL: "dialog", "panel", ["enabled"];
    POPOVER_ARROW: "popover", "arrow", ["enabled"];
    TOAST_INDICATOR: "toast", "indicator", ["enabled"];
    SPOTLIGHT_EDGE: "og-spotlight", "edge", ["enabled"];
    COOLDOWN_TRACK: "cooldown", "track", ["enabled"];
    COOLDOWN_SWEEP: "cooldown", "sweep", ["enabled"];
    COOLDOWN_MASK: "cooldown", "mask", ["enabled"];
    COOLDOWN_CORE: "cooldown", "core", ["enabled", "active"];
    COOLDOWN_ICON: "cooldown", "icon", ["enabled"];
    COOLDOWN_LABEL: "cooldown", "label", ["enabled"];
    MINIMAP_FRAME: "minimap", "frame", ["enabled"];
    MINIMAP_IMAGE: "minimap", "image", ["enabled"];
    MINIMAP_MARKER_OUTLINE: "minimap", "markerOutline", ["enabled"];
    MINIMAP_MARKER: "minimap", "marker", ["enabled"];
    MINIMAP_NORTH: "minimap", "north", ["enabled"];
    DIALOGUE_PORTRAIT_ICON: "dialogue", "portraitIcon", ["enabled"];
    DIALOGUE_INITIAL: "dialogue", "initial", ["enabled"];
    DIALOGUE_CONTINUE: "dialogue", "continue", ["enabled"];
    JOYSTICK_BASE: "joystick", "base", ["enabled", "disabled"];
    JOYSTICK_RIM: "joystick", "rim", ["enabled", "disabled", "active"];
    JOYSTICK_KNOB: "joystick", "knob", ["enabled", "disabled", "active"];
    JOYSTICK_DIRECTION: "joystick", "direction", ["enabled", "disabled", "active"];
    DPAD_BASE: "dpad", "base", ["enabled", "disabled"];
    DPAD_INDICATOR: "dpad", "indicator", ["enabled", "active"];
    DPAD_CHEVRON: "dpad", "chevron", ["enabled", "active"];
    ACTIONBUTTON_FACE: "actionbutton", "face", ["enabled", "disabled", "active"];
    ACTIONBUTTON_ICON: "actionbutton", "icon", ["enabled", "disabled", "active"];
    ACTIONBUTTON_TRACK: "actionbutton", "track", ["enabled"];
    ACTIONBUTTON_SWEEP: "actionbutton", "sweep", ["enabled"];
    ACTIONBUTTON_MASK: "actionbutton", "mask", ["enabled"];
    ACTIONBUTTON_CORE: "actionbutton", "core", ["enabled", "disabled", "active"];
    ACTIONBUTTON_LABEL: "actionbutton", "label", ["enabled", "disabled", "active"];
    HEADING_FLOURISH: "heading", "flourish", ["enabled"];
    SCROLL_BAR: "scroll", "bar", ["enabled"];
    TEXTAREA_CARET: "textarea", "caret", ["enabled"];
    TEXTAREA_COMPOSITION: "textarea", "composition", ["enabled"];
    INPUT_SELECTION: "input", "selection", ["enabled"];
    INPUT_CARET: "input", "caret", ["enabled"];
    INPUT_COMPOSITION: "input", "composition", ["enabled"];
    INPUT_FOCUS: "input", "focus", ["enabled", "disabled", "hover", "active", "focus-visible", "invalid"];
    CHECKBOX_FOCUS: "input[type=checkbox]", "focus", ["enabled", "disabled", "hover", "active", "focus-visible", "checked", "invalid"];
    SWITCH_FOCUS: "input[type=checkbox][switch]", "focus", ["enabled", "disabled", "hover", "active", "focus-visible", "checked"];
    RADIO_FOCUS: "input[type=radio]", "focus", ["enabled", "disabled", "hover", "active", "focus-visible", "checked"];
    RANGE_FOCUS: "input[type=range]", "focus", ["enabled", "disabled", "hover", "active", "focus-visible"];
    TEXTAREA_FOCUS: "textarea", "focus", ["enabled", "disabled", "focus-visible", "invalid"];
    SELECT_FOCUS: "select", "focus", ["enabled", "disabled", "hover", "active", "focus-visible", "open", "invalid"];
    DETAILS_FOCUS: "details", "focus", ["enabled", "disabled", "hover", "active", "focus-visible", "open"];
    STEPPER_FOCUS: "stepper", "focus", ["enabled", "disabled", "hover", "active", "focus-visible"];
    KEYBOARD_FOCUS: "keyboard", "focus", ["enabled", "disabled", "hover", "active", "focus-visible"];
    SLOT_FOCUS: "slot", "focus", ["enabled", "hover", "active", "focus-visible"];
    ART_ICON: "art", "icon", ["enabled"];
    ART_PLACEHOLDER: "art", "placeholder", ["enabled"];
    ART_LABEL: "art", "label", ["enabled"];
    H1_FLOURISH: "h1", "flourish", ["enabled"];
    H2_FLOURISH: "h2", "flourish", ["enabled"];
    H3_FLOURISH: "h3", "flourish", ["enabled"];
    H4_FLOURISH: "h4", "flourish", ["enabled"];
    H5_FLOURISH: "h5", "flourish", ["enabled"];
    H6_FLOURISH: "h6", "flourish", ["enabled"];
    NUMBER_FRAME: "input[type=number]", "frame", ["enabled"];
    NUMBER_VALUE: "input[type=number]", "value", ["enabled", "disabled", "hover", "active", "focus-visible"];
    NUMBER_BUTTON: "input[type=number]", "button", ["enabled", "disabled", "hover", "active", "focus-visible"];
}

/// Exact host-selector lookup. Typed inputs have their own contracts;
/// callers must not substitute a generic `input` contract for a switch.
pub fn part(host: &str, name: &str) -> Option<&'static PartSpec> {
    PARTS.iter().find(|p| p.host == host && p.name == name)
}

/// Validate one CSS source. Imports are checked as their own sources, so
/// diagnostics retain the imported filename and its original line numbers.
/// `authored` contains (markup tag, part name) declarations from game documents.
pub fn validate_source(source: &str, css: &str, authored: &[(String, String)]) -> Vec<String> {
    crate::doc::stylesheet::check_part_selectors(source, css, &|selector| {
        validate_selector(selector, authored)
    })
}

fn validate_selector(
    selector: &crate::doc::selectors::Selector,
    authored: &[(String, String)],
) -> Result<(), String> {
    use crate::doc::selectors::Simple;
    for compound in &selector.compounds {
        for simple in &compound.0 {
            if let Simple::Is(list) | Simple::Where(list) | Simple::Not(list) | Simple::Has(list) =
                simple
            {
                for nested in list {
                    validate_selector(nested, authored)?;
                }
            }
        }
        let tag = compound.0.iter().find_map(|s| match s {
            Simple::Type(tag) => Some(tag.as_str()),
            _ => None,
        });
        // Custom-element names are an explicit author namespace, not misspelled
        // built-ins. Documents can also declare parts on standard elements.
        if tag.is_some_and(|tag| tag.contains('-')) {
            continue;
        }
        let Some((index, name)) = compound.0.iter().enumerate().find_map(|(i, s)| match s {
            Simple::Part(name) => Some((i, name.as_str())),
            _ => None,
        }) else {
            continue;
        };
        let known = tag.is_none_or(|tag| {
            PARTS.iter().any(|p| p.host.split('[').next() == Some(tag))
                || super::Kind::ALL.iter().any(|k| {
                    super::tags::of(*k)
                        .iter()
                        .any(|(t, _)| t.split_whitespace().next() == Some(tag))
                })
                || authored.iter().any(|(host, _)| host == tag)
        });
        if !known {
            return Err(format!("unknown UI host `{}`", tag.unwrap()));
        }
        if authored
            .iter()
            .any(|(host, part)| tag.is_none_or(|tag| host == tag) && part == name)
        {
            continue;
        }
        let candidates: Vec<_> = PARTS
            .iter()
            .filter(|spec| {
                spec.name == name
                    && spec.reference(None).with_facts(|facts| {
                        tag.is_none_or(|tag| facts.tag == tag)
                            && compound.0[..index].iter().all(|s| match s {
                                Simple::Attribute {
                                    name,
                                    test: Some(test),
                                } => facts
                                    .attributes
                                    .get(name)
                                    .is_none_or(|value| test.test(&crate::bind::text(value))),
                                _ => true,
                            })
                    })
            })
            .collect();
        if candidates.is_empty() {
            return Err(format!(
                "unknown part `{name}` on `{}`; declare custom parts in markup with part=\"{name}\"",
                tag.unwrap_or("*")
            ));
        }
        for simple in &compound.0[index + 1..] {
            if let Simple::State(state) = simple
                && !candidates.iter().any(|spec| {
                    spec.states
                        .iter()
                        .any(|s| crate::doc::selectors::state(s) == Some(*state))
                })
            {
                return Err(format!(
                    "state `{}` is not exposed by {}::part({name})",
                    crate::doc::css::kebab(&format!("{state:?}")).trim_start_matches('-'),
                    tag.unwrap_or("*")
                ));
            }
        }
    }
    Ok(())
}

/// Read explicit `part` attributes without resolving a document's imports or
/// bindings. Dynamic part names are not guessed by static pack validation.
pub fn authored_parts(source: &str, text: &str) -> Result<Vec<(String, String)>, String> {
    use crate::doc::syntax::{self, Node};
    fn collect(nodes: &[Node], out: &mut Vec<(String, String)>) {
        for node in nodes {
            match node {
                Node::Element {
                    tag,
                    attrs,
                    children,
                    ..
                } => {
                    for attr in attrs.iter().filter(|a| a.name == "part" && !a.bound) {
                        out.extend(
                            attr.value
                                .split_whitespace()
                                .map(|part| (tag.clone(), part.to_owned())),
                        );
                    }
                    collect(children, out);
                }
                Node::If { yes, no, .. } => {
                    collect(yes, out);
                    collect(no, out);
                }
                Node::Each { body, empty, .. } => {
                    collect(body, out);
                    collect(empty, out);
                }
                Node::Snippet { body, .. } | Node::Key { body, .. } => collect(body, out),
                Node::Await {
                    pending,
                    then,
                    catch,
                    ..
                } => {
                    collect(pending, out);
                    for (_, body) in then.iter().chain(catch) {
                        collect(body, out);
                    }
                }
                _ => (),
            }
        }
    }
    let nodes = syntax::parse(text).map_err(|error| syntax::diagnostic(source, text, error))?;
    let mut out = Vec::new();
    collect(&nodes, &mut out);
    out.sort();
    out.dedup();
    Ok(out)
}

pub fn to_json() -> serde_json::Value {
    serde_json::Value::Array(
        PARTS
            .iter()
            .map(|p| {
                serde_json::json!({
                    "host": p.host, "name": p.name, "states": p.states
                })
            })
            .collect(),
    )
}

pub fn markdown() -> String {
    use std::fmt::Write;
    let mut out = String::from("| Host | Part | State pseudo-classes |\n|---|---|---|\n");
    for p in PARTS {
        writeln!(
            out,
            "| `{}` | `{}` | {} |",
            p.host,
            p.name,
            p.states
                .iter()
                .map(|s| format!("`:{s}`"))
                .collect::<Vec<_>>()
                .join(", ")
        )
        .unwrap();
    }
    out
}
