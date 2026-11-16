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
use og_ui::{DrawCmd, InputFrame, Srgba, UiKey};
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

/// A link's text fragment (`#:~:text=[prefix-,]start[,end][,-suffix]`)
/// finds its passage in the laid-out text, marks it with `::target-text`'s
/// background and ink, scrolls it to the middle of its scroll container
/// and makes the element holding it `:target`; with no match, the id
/// before `:~:` is `:target` and nothing is marked.
///
/// Ways it could fail: the first "is" marked though its context does not
/// fit (a prefix or suffix ignored, or case read), a range stopping at
/// its first element, the marks or the red ink covering the whole line,
/// the passage left out of view, `:target` staying on the id when the
/// text matched or the marks staying after a link that matches nothing.
#[test]
fn a_text_fragment_marks_scrolls_to_and_targets_its_passage() {
    let doc = r##"<meta name="component" content="tt">
<column id="Root">
  <a href="#:~:text=Gate-,IS,-shut" id="Go">go</a>
  <a href="#:~:text=gate,open" id="Range">range</a>
  <a href="#Near:~:text=nowhere" id="Miss">miss</a>
  <div id="Box">
    <p id="Near">this is the gate</p>
    <div class="pad"></div>
    <p id="Far">the gate is open and the gate is shut</p>
    <div class="pad"></div>
  </div>
</column>
<style>
a { width: 80px; height: 30px; }
#Box { width: 700px; height: 100px; overflow: auto; scrollbar-width: none; }
.pad { height: 300px; }
p { width: max-content; margin: 0; }
p:target { margin-left: 7px; }
#Box:target { margin-left: 3px; }
::target-text { background-color: #00ff00; color: #ff0000; }
</style>"##;
    let mut d = Doc::with_font(doc, &[], test_font());
    let near = |a: [f32; 4], b: Srgba| {
        let b = b.linear();
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.02)
    };
    // The marks drawn: green bands, and the clips of red and other ink.
    let drawn = |d: &Doc| {
        let (mut bands, mut red, mut other) = (vec![], vec![], vec![]);
        for item in &d.ui.frame().items {
            let cmd = match &item.cmd {
                DrawCmd::StyledText { text, .. } => text.as_ref(),
                c => c,
            };
            match cmd {
                DrawCmd::Quad { rect, color, .. } if near(*color, Srgba::rgb(0, 255, 0)) => {
                    bands.push(*rect)
                }
                DrawCmd::Text { color, clip, .. } if near(*color, Srgba::rgb(255, 0, 0)) => {
                    red.push(*clip)
                }
                DrawCmd::Text { clip, bounds, .. } => other.push((*clip, *bounds)),
                _ => {}
            }
        }
        (bands, red, other)
    };
    let follow = |d: &mut Doc, id: &str| {
        let a = d.get(id);
        d.click_at(Vec2::new(a.x + 3.0, a.y + a.height / 2.0));
        for _ in 0..3 {
            d.step(InputFrame::default());
        }
    };
    let base = d.get("Near").x;
    assert!(drawn(&d).0.is_empty(), "nothing marked yet");

    // "is" after "Gate" and before "shut": the second "is" of Far, not
    // Near's or Far's first.
    follow(&mut d, "Go");
    let (far, boxed) = (d.get("Far"), d.get("Box"));
    // (The probe lists what is on screen: Near has scrolled away.)
    assert_eq!(far.x, base + 7.0, "Far is :target");
    let (bands, red, other) = drawn(&d);
    assert_eq!(bands.len(), 1, "{bands:?}");
    let band = bands[0];
    assert!(
        band.x > far.x + far.width * 0.7 && band.right() < far.x + far.width * 0.9,
        "the second \"is\": {band:?} in {far:?}"
    );
    assert!((band.y - far.y).abs() < 1.0, "{band:?} {far:?}");
    // Its glyphs in red, cut to the band; the rest of the line in its own ink.
    assert!(!red.is_empty(), "no target-text ink");
    for c in &red {
        assert!(
            c.x >= band.x - 0.5 && c.right() <= band.right() + 0.5,
            "{c:?} {band:?}"
        );
    }
    assert!(
        other
            .iter()
            .any(|(c, b)| (b.y - far.y).abs() < 1.0 && c.right() <= band.x + 0.5)
            && other
                .iter()
                .any(|(c, b)| (b.y - far.y).abs() < 1.0 && c.x >= band.right() - 0.5),
        "Far's unmarked text on both sides"
    );
    // Scrolled so its line stands in the middle of the box.
    let (mid, view) = (far.y + far.height / 2.0, boxed.y + boxed.height / 2.0);
    assert!((mid - view).abs() < 2.0, "{mid} vs {view}");

    // "gate" to the first "open" after it: Near's last word on to Far's
    // "the gate is open", their box the element holding it. Scrolled to
    // Near's line, Far is below the view.
    follow(&mut d, "Range");
    let n = d.get("Near");
    assert_eq!(
        (n.x, d.get("Box").x),
        (base + 3.0, base + 3.0),
        "Box is :target, not either paragraph"
    );
    let (bands, ..) = drawn(&d);
    assert_eq!(bands.len(), 1, "Far is below the view: {bands:?}");
    let a = bands[0];
    assert!(
        (a.y - n.y).abs() < 1.0
            && a.x > n.x + n.width * 0.6
            && (a.right() - (n.x + n.width)).abs() < 1.0,
        "Near's \"gate\": {a:?} in {n:?}"
    );
    // Scrolled down to Far: its first four words marked.
    let b = d.get("Box");
    d.step(InputFrame {
        pointer: Vec2::new(b.x + 50.0, b.y + 50.0),
        pointer_inside: true,
        wheel: Vec2::new(0.0, 280.0),
        ..InputFrame::default()
    });
    d.step(InputFrame::default());
    let far = d.get("Far");
    let (bands, ..) = drawn(&d);
    let b = *bands.last().expect("Far's band");
    assert!(
        (b.y - far.y).abs() < 1.0
            && (b.x - far.x).abs() < 1.0
            && b.width > far.width * 0.3
            && b.width < far.width * 0.5,
        "Far's \"the gate is open\": {b:?} in {far:?}"
    );

    // No match: the id is :target, and the marks are gone.
    let b = d.get("Box");
    d.step(InputFrame {
        pointer: Vec2::new(b.x + 50.0, b.y + 50.0),
        pointer_inside: true,
        wheel: Vec2::new(0.0, -1000.0),
        ..InputFrame::default()
    });
    follow(&mut d, "Miss");
    assert_eq!(d.get("Near").x, base + 7.0);
    assert!(drawn(&d).0.is_empty(), "marks cleared");
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
