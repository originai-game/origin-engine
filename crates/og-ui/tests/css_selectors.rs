//! Selectors and the cascade (docs/ui.md §3.2), each proved by what it
//! sizes: a probe the selector matches is 50px wide, one it must not match
//! keeps its 10px.
//!
//! Ways these could fail, each covered below:
//! - a logical or relational pseudo-class (`:is()`, `:where()`, `:not()`,
//!   `:has()`) matching the wrong element, or `:where()` keeping its
//!   argument's specificity → `selectors_select_as_css_does`
//! - `:nth-of-type()` counting every sibling, `:root` or `:scope` missing
//!   the root, `@scope` reaching past its limit → the same test
//! - an attribute's `i` or `s` flag ignored, `:lang()` not matching a
//!   region subtag, `:dir()` missing an element's own direction, `:state()` or `:any-link` reading the wrong attribute,
//!   nesting's `&` not standing for its parent → the same test
//! - a form pseudo-class (`:checked`, `:disabled`, `:enabled`, `:default`,
//!   `:indeterminate`, `:read-only`, `:placeholder-shown`,
//!   `:out-of-range`) or `details:open` matching an element in the other
//!   state, or `~` reaching an earlier sibling → the same test

use crate::support::*;
use og_core::glam::Vec2;
use og_ui::{InputFrame, UiKey};
use serde_json::json;

const DOC: &str = r##"<meta name="component" content="sel">
<div id="Root" class="root">
  <div class="probe" id="Has"><span class="mark"></span></div>
  <div class="probe" id="HasNot"><span></span></div>
  <div class="probe" id="Is"></div>
  <div class="probe" id="Where"></div>
  <div class="probe skip" id="NotSkip"></div>
  <div class="probe" id="Not"></div>
  <div id="Types"><span></span><p class="probe" id="Type1"></p><span></span><p class="probe" id="Type2"></p></div>
  <div class="probe" id="Root2"></div>
  <div class="probe" id="Scope"></div>
  <div class="card"><div class="probe" id="InScope"></div><div class="inner"><div class="probe" id="PastLimit"></div></div></div>
  <div class="probe" id="Caseless" data-k="AbC"></div>
  <div class="probe" id="CaseExact" data-k="abc"></div>
  <div class="probe" id="CaseWrong" data-k="AbC" data-s="AbC"></div>
  <p lang="fr-CA" class="probe" id="Lang">x</p>
  <p dir="rtl" class="probe" id="Dir">x</p>
  <div class="probe" id="State" state:on={$on}></div>
  <div class="probe" id="StateOff" state:on={$off}></div>
  <a href="#x" class="probe" id="Link">x</a>
  <div class="nest"><div class="probe" id="Nested"></div></div>
  <div class="probe" id="Sib"></div><div></div><div class="probe" id="SibLater"></div>
  <input id="Off" disabled><div class="probe" id="AfterDisabled"></div>
  <input id="On"><div class="probe" id="AfterEnabled"></div>
  <input type="checkbox" id="Checked" checked><div class="probe" id="AfterChecked"></div>
  <input type="checkbox" id="Unchecked"><div class="probe" id="AfterUnchecked"></div>
  <input type="checkbox" id="Mixed" indeterminate><div class="probe" id="AfterMixed"></div>
  <input id="ReadOnly" readonly value="r"><div class="probe" id="AfterReadOnly"></div>
  <input id="Empty" placeholder="name"><div class="probe" id="AfterEmpty"></div>
  <input type="number" id="Over" min="0" max="5" value="9"><div class="probe" id="AfterOver"></div>
  <details open id="Open"><summary>s</summary></details><div class="probe" id="AfterOpen"></div>
  <details id="Shut"><summary>s</summary></details><div class="probe" id="AfterShut"></div>
</div>
<style>
.probe { width: 10px; height: 4px; }
.probe:has(> .mark) { width: 50px; }
.probe:is(#Is, #Nowhere) { width: 50px; }
:where(#Where) { width: 50px; min-height: 9px; }
.probe:not(.skip):is(#Not, #NotSkip) { width: 50px; }
p.probe:nth-of-type(2) { width: 50px; }
:root { --wide: 50px; }
#Root2 { width: var(--wide); }
:scope > #Scope { width: 50px; }
@scope (.card) to (.inner) { .probe { width: 50px; } }
#Caseless[data-k="abc" i] { width: 50px; }
.probe[data-k="abc" s] { width: 50px; }
.probe:lang(fr) { width: 50px; }
.probe:dir(rtl) { width: 50px; }
.probe:state(on) { width: 50px; }
.probe:any-link { width: 50px; }
.nest { & > .probe { width: 50px; } }
input:checked + .probe, input:default + .probe, input:indeterminate + .probe { width: 50px; }
input:read-only:not([type]) + .probe { width: 50px; }
input:placeholder-shown + .probe { width: 50px; }
input:out-of-range + .probe { width: 50px; }
details:open + .probe { width: 50px; }
#Sib ~ #SibLater { width: 50px; }
input:disabled + .probe { width: 50px; }
#On:enabled + .probe { width: 50px; }
</style>"##;

#[test]
fn selectors_select_as_css_does() {
    with_doc(DOC, &[("on", json!(true)), ("off", json!(false))], |d| {
        let width = |id: &str| d.get(id).width;
        for id in [
            "Has",
            "Is",
            "Not",
            "Type2",
            "Root2",
            "Scope",
            "InScope",
            "Caseless",
            "CaseExact",
            "Lang",
            "Dir",
            "State",
            "Link",
            "Nested",
            "AfterChecked",
            "AfterMixed",
            "AfterReadOnly",
            "AfterEmpty",
            "AfterOver",
            "AfterOpen",
            "SibLater",
            "AfterDisabled",
            "AfterEnabled",
        ] {
            assert_eq!(width(id), 50.0, "{id} is selected");
        }
        for id in [
            "HasNot",
            "NotSkip",
            "Type1",
            "PastLimit",
            "CaseWrong",
            "StateOff",
            "AfterUnchecked",
            "AfterShut",
            "Sib",
        ] {
            assert_eq!(width(id), 10.0, "{id} is not selected");
        }
        // `:where()` matches with no specificity: its min-height holds, its
        // width loses to `.probe`.
        assert_eq!((width("Where"), d.get("Where").height), (10.0, 9.0));
    });
}

/// `:active-view-transition` matches a screen's root from the frame its
/// layer's view transition starts until it ends, and
/// `:active-view-transition-type()` only when the screen's
/// `@view-transition` declares one of the types.
///
/// Ways it could fail: the state never set or never cleared, set on every
/// element and not only the root, a type the screen does not declare
/// matching, or a sibling rule (matched on the instantiated tree) not
/// seeing the types.
#[test]
fn active_view_transition_matches_the_root_while_it_runs() {
    let doc = r##"<meta name="screen" content="t">
<column id="Page">
  <div class="probe" id="Any"></div>
  <div class="probe" id="Typed"></div>
  <div class="probe" id="Listed"></div>
  <div class="probe" id="Other"></div>
  <div class="probe" id="Own"></div>
  <div class="probe" id="Sib"></div>
</column>
<style>
@view-transition { navigation: auto; types: slide; }
.probe { width: 10px; height: 4px; }
:active-view-transition #Any { width: 50px; }
:root:active-view-transition-type(slide) #Typed { width: 50px; }
:active-view-transition-type(fade, slide) #Listed { width: 50px; }
:active-view-transition-type(fade) #Other { width: 50px; }
.probe:active-view-transition { width: 50px; }
:active-view-transition-type(slide) #Own + #Sib { width: 50px; }
</style>"##;
    let mut d = Doc::new(doc, &[]);
    let widths =
        |d: &Doc| ["Any", "Typed", "Listed", "Other", "Own", "Sib"].map(|id| d.get(id).width);
    let running = [50.0, 50.0, 50.0, 10.0, 10.0, 50.0];
    assert_eq!(widths(&d), running, "mid-transition");
    let ticks = d.ui.ticks_for(250.0) as usize;
    for _ in 0..ticks + 3 {
        d.step(InputFrame::default());
    }
    assert_eq!(widths(&d), [10.0; 6], "after it ends");
}

/// The user-action, focus, user-validity and `:target` pseudo-classes
/// follow the pointer and the keys as a browser's do: `:hover` under the
/// pointer, `:active` while it is down, `:focus` on a clicked control but
/// `:focus-visible` only for keyboard focus, `:user-invalid` and
/// `:user-valid` only once the player has typed, and `:target` on the
/// element a `#fragment` link names.
///
/// Ways it could fail: a state set on the wrong element or never cleared,
/// `:focus-visible` matching a pointer focus, an untouched required field
/// counting as `:user-invalid`, `:target` matching before the link is
/// followed.
#[test]
fn interaction_pseudo_classes_follow_the_pointer_and_keys() {
    let doc = r##"<meta name="component" content="ix">
<column id="Root">
  <button id="A">A</button><div class="probe" id="Hover"></div><div class="probe" id="Down"></div>
  <button id="B">B</button><div class="probe" id="Focus"></div><div class="probe" id="Ring"></div>
  <input id="Mail" type="email" required bind:value={$mail}><div class="probe" id="Bad"></div><div class="probe" id="Good"></div>
  <a href="#Goal" id="Go">go</a>
  <div class="probe" id="Goal"></div><div class="probe" id="NotGoal"></div>
</column>
<style>
.probe { width: 10px; height: 4px; }
button, input, a { width: 80px; height: 30px; }
#A:hover + #Hover, #A:active ~ #Down { width: 50px; }
#B:focus + #Focus, #B:focus-visible ~ #Ring { width: 50px; }
#Mail:user-invalid + #Bad, #Mail:user-valid ~ #Good { width: 50px; }
.probe:target { width: 50px; }
</style>"##;
    let mut d = Doc::new(doc, &[("mail", json!(""))]);
    let w = |d: &Doc, ids: &[&str]| ids.iter().map(|id| d.get(id).width).collect::<Vec<_>>();
    let all = [
        "Hover", "Down", "Focus", "Ring", "Bad", "Good", "Goal", "NotGoal",
    ];
    assert_eq!(w(&d, &all), [10.0; 8], "nothing yet");
    let a = d.get("A");
    let over = Vec2::new(a.x + 5.0, a.y + 5.0);
    let at = |down: bool, pressed: bool| InputFrame {
        pointer: over,
        pointer_inside: true,
        down,
        pressed,
        ..InputFrame::default()
    };
    d.step(at(false, false));
    d.step(at(false, false));
    assert_eq!(w(&d, &["Hover", "Down"]), [50.0, 10.0], "hovered");
    d.step(at(true, true));
    d.step(at(true, false));
    assert_eq!(w(&d, &["Hover", "Down"]), [50.0, 50.0], "pressed");
    d.click("B");
    assert_eq!(
        w(&d, &["Hover", "Focus", "Ring"]),
        [10.0, 50.0, 10.0],
        "clicked: focus, no ring"
    );
    d.click("Mail");
    assert_eq!(
        w(&d, &["Focus", "Bad", "Good"]),
        [10.0, 10.0, 10.0],
        "untouched"
    );
    let typed = |d: &mut Doc, text: &str| {
        d.step(InputFrame {
            text: text.into(),
            ..InputFrame::default()
        });
        d.step(InputFrame::default());
        d.step(InputFrame::default());
    };
    typed(&mut d, "x");
    assert_eq!(w(&d, &["Bad", "Good"]), [50.0, 10.0], "refused text");
    typed(&mut d, "@y.z");
    assert_eq!(w(&d, &["Bad", "Good"]), [10.0, 50.0], "taken text");
    // Shift+Tab back from the field reaches B by the keyboard: the ring.
    d.step(InputFrame {
        keys: vec![UiKey::Tab],
        shift: true,
        ..InputFrame::default()
    });
    d.step(InputFrame::default());
    d.step(InputFrame::default());
    assert_eq!(w(&d, &["Focus", "Ring"]), [50.0, 50.0], "keyboard focus");
    // A link is its text run, not its box: click on the glyphs.
    let go = d.get("Go");
    d.click_at(Vec2::new(go.x + 3.0, go.y + go.height / 2.0));
    assert_eq!(
        w(&d, &["Goal", "NotGoal"]),
        [50.0, 10.0],
        "followed the link"
    );
}

/// `:modal` matches a dialog opened modally (it makes the rest of the page
/// inert) and not one open in place.
#[test]
fn modal_matches_only_a_modal_dialog() {
    let doc = r##"<meta name="component" content="dlg">
<column id="Root">
  <dialog id="Plain" open><div class="probe" id="InPlain"></div></dialog>
  <dialog id="Ask" modal open aria-label="Quit?"><div class="probe" id="InModal"></div></dialog>
</column>
<style>
.probe { width: 10px; height: 4px; }
:modal .probe { width: 50px; }
</style>"##;
    let mut d = Doc::new(doc, &[]);
    for _ in 0..30 {
        d.step(InputFrame::default());
    }
    assert_eq!(
        (d.get("InModal").width, d.get("InPlain").width),
        (50.0, 10.0)
    );
}

/// A field without `bind:value` keeps what the player types as its own
/// value, as an HTML input does: it shows it, its validity pseudo-classes
/// read it, and a snapshot carries it.
///
/// Ways it could fail: the typed text dropped each frame (the field
/// re-read its `value` attribute), validity read from the attribute and
/// not the typed text, the value lost on restore.
#[test]
fn an_unbound_field_keeps_its_own_value() {
    let doc = r##"<meta name="component" content="own">
<column id="Root">
  <input id="Mail" type="email" required value="">
  <div class="probe" id="Invalid"></div><div class="probe" id="Bad"></div><div class="probe" id="Good"></div>
</column>
<style>
.probe { width: 10px; height: 4px; }
input { width: 160px; height: 30px; }
#Mail:invalid ~ #Invalid, #Mail:user-invalid ~ #Bad, #Mail:user-valid ~ #Good { width: 50px; }
</style>"##;
    let mut d = Doc::new(doc, &[]);
    let w = |d: &Doc| ["Invalid", "Bad", "Good"].map(|id| d.get(id).width);
    assert_eq!(w(&d), [50.0, 10.0, 10.0], "empty and required, untouched");
    d.click("Mail");
    let typed = |d: &mut Doc, text: &str| {
        d.step(InputFrame {
            text: text.into(),
            ..InputFrame::default()
        });
        d.step(InputFrame::default());
        d.step(InputFrame::default());
    };
    typed(&mut d, "x");
    assert_eq!(d.a11y("Mail").value, "x");
    assert_eq!(w(&d), [50.0, 50.0, 10.0], "refused text");
    typed(&mut d, "@y.z");
    assert_eq!(d.a11y("Mail").value, "x@y.z");
    assert_eq!(w(&d), [10.0, 10.0, 50.0], "taken text");
    let saved = d.ui.save_state();
    typed(&mut d, "!");
    assert_eq!(w(&d), [50.0, 50.0, 10.0], "refused again");
    d.ui.restore_state(&saved).unwrap();
    d.step(InputFrame::default());
    d.step(InputFrame::default());
    assert_eq!(d.a11y("Mail").value, "x@y.z", "restored");
    assert_eq!(w(&d), [10.0, 10.0, 50.0], "restored validity");
}

/// `[open]` on a `<details>` or `<dialog>` follows its open state however
/// it changes, as HTML reflects it: the player toggling an unbound one, a
/// `bind:open` store, and the player closing a literal `open`. Both the
/// lowered rule (`details[open] { width }`) and the one matched on the
/// instantiated tree (`details[open] + .probe`) read it.
///
/// Ways it could fail: `[open]` read from the written attribute only (a
/// bound or a toggled element never matching, a closed literal one still
/// matching), the toggled state lost on restore.
#[test]
fn the_open_attribute_reflects_the_open_state() {
    let doc = r##"<meta name="component" content="open">
<column id="Root">
  <details id="Own"><summary>a</summary><p>x</p></details><div class="probe" id="OwnNext"></div>
  <details id="Bound" bind:open={$b}><summary>b</summary><p>x</p></details><div class="probe" id="BoundNext"></div>
  <details id="Lit" open><summary>c</summary><p>x</p></details><div class="probe" id="LitNext"></div>
  <dialog id="Dlg" bind:open={$d}><p>d</p></dialog><div class="probe" id="DlgNext"></div>
</column>
<style>
.probe { width: 10px; height: 4px; }
details { width: 100px; }
details[open] { width: 140px; }
details[open] + .probe, dialog[open] + .probe { width: 50px; }
</style>"##;
    let mut d = Doc::new(doc, &[("b", json!(false)), ("d", json!(false))]);
    let w = |d: &Doc, id: &str| (d.get(id).width, d.get(&format!("{id}Next")).width);
    assert_eq!(
        [w(&d, "Own"), w(&d, "Bound"), w(&d, "Lit")],
        [(100.0, 10.0), (100.0, 10.0), (140.0, 50.0)]
    );
    assert_eq!(d.get("DlgNext").width, 10.0);
    d.click("Own/Summary");
    assert_eq!(w(&d, "Own"), (140.0, 50.0), "the player opened it");
    d.click("Bound/Summary");
    assert_eq!(
        (d.read("b"), w(&d, "Bound")),
        (json!(true), (140.0, 50.0)),
        "bind:open"
    );
    d.click("Lit/Summary");
    assert_eq!(
        w(&d, "Lit"),
        (100.0, 10.0),
        "the player closed a literal open"
    );
    d.set("d", json!(true));
    assert_eq!(d.get("DlgNext").width, 50.0, "a bound dialog");
    let saved = d.ui.save_state();
    d.click("Own/Summary");
    assert_eq!(w(&d, "Own"), (100.0, 10.0), "closed again");
    d.ui.restore_state(&saved).unwrap();
    d.step(InputFrame::default());
    d.step(InputFrame::default());
    assert_eq!(w(&d, "Own"), (140.0, 50.0), "restored open");
}

/// A state pseudo-class after a box pseudo-element reads the
/// pseudo-element's own state (Selectors 4 §3.6.4): `dialog::backdrop:hover`
/// while the pointer is on the scrim and not while it is on the dialog,
/// and a state set for the pseudo-element from outside
/// (`Ui::set_selector_state`), as the scroll markers' are.
///
/// Ways it could fail: the pseudo-class read on the originating element
/// (the dialog hovered counting as its backdrop), the rule applying with
/// no state at all, the state never cleared, a state set by the
/// pseudo-element's key not reaching its rules.
#[test]
fn a_pseudo_class_after_a_box_pseudo_element_reads_its_own_state() {
    let doc = r##"<meta name="component" content="pe">
<column id="Root">
  <dialog id="Ask" modal open aria-label="Quit?"><button id="Yes">Yes</button></dialog>
</column>
<style>
dialog { width: 200px; }
button { width: 80px; height: 30px; }
dialog::backdrop { padding: 0; }
dialog::backdrop:hover { padding-left: 200px; }
dialog::backdrop:target-current { padding-left: 400px; }
</style>"##;
    let mut d = Doc::new(doc, &[]);
    for _ in 0..30 {
        d.step(InputFrame::default());
    }
    let middle = |d: &Doc| {
        let p = d.get("Panel");
        p.x + p.width / 2.0
    };
    let rest = middle(&d);
    let at = |p: Vec2| InputFrame {
        pointer: p,
        pointer_inside: true,
        ..InputFrame::default()
    };
    let settle = |d: &mut Doc, input: InputFrame| {
        for _ in 0..4 {
            d.step(input.clone());
        }
    };
    settle(&mut d, at(Vec2::new(5.0, 5.0)));
    assert_eq!(middle(&d), rest + 100.0, "the scrim hovered");
    // A point on the dialog both where it stood and where the hovered
    // backdrop's padding put it: leaving the backdrop does not re-enter it.
    let panel = d.get("Panel");
    settle(
        &mut d,
        at(Vec2::new(panel.x + 20.0, panel.y + panel.height / 2.0)),
    );
    assert_eq!(middle(&d), rest, "the dialog hovered, not its backdrop");
    let backdrop = og_ui::id::key_of_path("test:t/Ask/::backdrop");
    let current = og_ui::bind::InteractionState::TargetCurrent;
    d.ui.set_selector_state(backdrop, current, true);
    settle(&mut d, InputFrame::default());
    assert_eq!(middle(&d), rest + 200.0, "a state set by its key");
    d.ui.set_selector_state(backdrop, current, false);
    settle(&mut d, InputFrame::default());
    assert_eq!(middle(&d), rest, "and set off");
    // A select's picker takes one too; a pseudo-class that is no state of
    // a pseudo-element's own is still refused.
    let parse = |rule: &str| {
        og_ui::UiDocument::parse(
            "ui/p.svelte",
            &format!(
                "<meta name=\"component\" content=\"p\">\n<column><select><option>a</option></select><dialog modal></dialog></column>\n<style>{rule}</style>"
            ),
        )
    };
    assert!(parse("select::picker(select):hover { height: 90px; }").is_ok());
    assert!(parse("dialog::backdrop:checked { padding: 0; }").is_err());
}

/// `:target-current`, `:target-before` and `:target-after` match the
/// elements the scroll-marker machinery marks (CSS Overflow 5 §3.1), in a
/// rule lowered to a condition and in one matched on the instantiated tree.
///
/// Ways it could fail: a state set by key not reaching the element, the
/// three states confused, a state surviving being set off.
#[test]
fn target_current_before_and_after_match_marked_elements() {
    let doc = r##"<meta name="component" content="marks">
<column id="Root">
  <a href="#one" id="One">one</a><div class="probe" id="NextOne"></div>
  <a href="#two" id="Two">two</a><div class="probe" id="NextTwo"></div>
  <a href="#three" id="Three">three</a><div class="probe" id="NextThree"></div>
</column>
<style>
.probe { width: 10px; height: 4px; }
a { width: 20px; height: 20px; }
a:target-before { width: 30px; }
a:target-current { width: 50px; }
a:target-after { width: 70px; }
a:target-current + .probe { width: 50px; }
</style>"##;
    let mut d = Doc::new(doc, &[]);
    let w = |d: &Doc| {
        ["One", "Two", "Three", "NextOne", "NextTwo", "NextThree"].map(|id| d.get(id).width)
    };
    assert_eq!(w(&d), [20.0, 20.0, 20.0, 10.0, 10.0, 10.0], "unmarked");
    use og_ui::bind::InteractionState::{TargetAfter, TargetBefore, TargetCurrent};
    let key = |id: &str| og_ui::id::key_of_path(&format!("test:t/{id}"));
    let mark = |d: &mut Doc, states: [(&str, og_ui::bind::InteractionState); 3]| {
        for id in ["One", "Two", "Three"] {
            for state in [TargetBefore, TargetCurrent, TargetAfter] {
                let on = states.contains(&(id, state));
                d.ui.set_selector_state(key(id), state, on);
            }
        }
        d.step(InputFrame::default());
        d.step(InputFrame::default());
    };
    mark(
        &mut d,
        [
            ("One", TargetBefore),
            ("Two", TargetCurrent),
            ("Three", TargetAfter),
        ],
    );
    assert_eq!(
        w(&d),
        [30.0, 50.0, 70.0, 10.0, 50.0, 10.0],
        "the second is current"
    );
    mark(
        &mut d,
        [
            ("One", TargetCurrent),
            ("Two", TargetAfter),
            ("Three", TargetAfter),
        ],
    );
    assert_eq!(
        w(&d),
        [50.0, 70.0, 70.0, 50.0, 10.0, 10.0],
        "the first is current"
    );
}
