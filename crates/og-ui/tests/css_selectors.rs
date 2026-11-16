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
  <input type="checkbox" id="TypeA"><div class="probe" id="TypeCaseless"></div>
  <input type="checkbox" id="TypeB"><div class="probe" id="TypeExact"></div>
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
  <input type="checkbox" id="Def" checked><div class="probe" id="AfterDef"></div>
  <input type="checkbox" id="Bound" bind:checked={$on}><div class="probe" id="AfterBound"></div>
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
#TypeA[type="CHECKBOX"] + .probe { width: 50px; }
#TypeB[type="CHECKBOX" s] + .probe { width: 50px; }
.probe:lang(fr) { width: 50px; }
.probe:dir(rtl) { width: 50px; }
.probe:state(on) { width: 50px; }
.probe:any-link { width: 50px; }
.nest { & > .probe { width: 50px; } }
input:checked + .probe, input:indeterminate + .probe { width: 50px; }
input:default + .probe { height: 9px; }
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
            "TypeCaseless",
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
            "TypeExact",
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
        // `:default` (height) is the checkbox its markup checks, apart from
        // `:checked` (width): Bound is checked by its binding, not its
        // markup.
        let size = |id: &str| (d.get(id).width, d.get(id).height);
        assert_eq!(size("AfterDef"), (50.0, 9.0), "checked and default");
        assert_eq!(size("AfterBound"), (50.0, 4.0), "checked, not default");
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

/// Cascade layers (CSS Cascade 5 §6.4) order a document's own rules: the
/// layer declared later wins whatever the specificity or source order
/// (`@layer a, b;` then a's `#Order` loses to b's `.probe`), unlayered
/// rules beat every layer, `!important` reverses both (a's important beats
/// b's, and a layer's beats an unlayered one), an anonymous layer takes
/// its place where it first appears (after a, b and c), a layer's own
/// rules beat its sublayers', and sublayers order among themselves as
/// declared (`c.x, c.y`).
///
/// Ways it could fail: layers ignored (specificity or source order
/// deciding), the order taken from the blocks rather than the first
/// declaration, unlayered rules losing, important declarations not
/// reversed, a nested layer ranked above its parent's own rules.
#[test]
fn cascade_layers_order_a_documents_rules() {
    let doc = r##"<meta name="component" content="layers">
<column id="Root">
  <div class="probe" id="Order"></div>
  <div class="probe" id="Unlayered"></div>
  <div class="probe" id="Important"></div>
  <div class="probe" id="ImportantUnlayered"></div>
  <div class="probe" id="Anon"></div>
  <div class="probe" id="Nested"></div>
  <div class="probe" id="Sub"></div>
</column>
<style>
@layer a, b, c;
.probe { height: 4px; }
[id="Unlayered"] { width: 30px; }
#ImportantUnlayered { width: 51px !important; }
@layer { #Anon { width: 60px; } }
@layer c.x, c.y;
@layer b {
  .probe { width: 20px; }
  #Unlayered.probe { width: 21px; }
  #Important { width: 41px !important; }
  #Anon { width: 61px; }
}
@layer a {
  #Order.probe { width: 10px; }
  #Important { width: 40px !important; }
  #ImportantUnlayered { width: 50px !important; }
}
@layer c {
  #Nested { width: 70px; }
  @layer inner { #Nested.probe { width: 71px; } }
}
@layer c.y { #Sub { width: 75px; } }
@layer c.x { #Sub.probe { width: 76px; } }
</style>"##;
    let d = Doc::new(doc, &[]);
    let widths = [
        "Order",
        "Unlayered",
        "Important",
        "ImportantUnlayered",
        "Anon",
        "Nested",
        "Sub",
    ]
    .map(|id| d.get(id).width);
    assert_eq!(widths, [20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 75.0]);
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

/// A `#id` link scrolls its element's scroll-margin box to the start of
/// the scroller's view, inside its scroll-padding, as HTML's "scroll to the
/// fragment" does, and makes it `:target`. Text directives are not read:
/// `#Near:~:text=gate` names no element.
///
/// Ways it could fail: an `#id` left out of view, centred, or placed
/// without its scroll-margin and the scroller's scroll-padding; `:target`
/// left on the last element; the old text-fragment syntax still finding
/// Near.
#[test]
fn a_fragment_link_scrolls_its_element_into_view() {
    let doc = r##"<meta name="component" content="tt">
<column id="Root">
  <a href="#Far" id="Plain">plain</a>
  <a href="#Near" id="Back">back</a>
  <a href="#Near:~:text=gate" id="Text">text</a>
  <div id="Box">
    <p id="Near">this is the gate</p>
    <div class="pad"></div>
    <p id="Far">the gate is open and the gate is shut</p>
    <div class="pad"></div>
  </div>
</column>
<style>
a { width: 80px; height: 30px; }
#Box { width: 700px; height: 100px; overflow: auto; scrollbar-width: none; scroll-padding-top: 6px; }
#Far { scroll-margin-top: 10px; }
.pad { height: 300px; }
p { width: max-content; margin: 0; }
p:target { margin-left: 7px; }
</style>"##;
    let mut d = Doc::with_font(doc, &[], test_font());
    let follow = |d: &mut Doc, id: &str| {
        let a = d.get(id);
        d.click_at(Vec2::new(a.x + 3.0, a.y + a.height / 2.0));
        for _ in 0..3 {
            d.step(InputFrame::default());
        }
    };
    let base = d.get("Near").x;

    // Far's scroll-margin box at the start of the view inside its
    // scroll-padding (10 + 6 px above its top), not centred.
    follow(&mut d, "Plain");
    let (far, boxed) = (d.get("Far"), d.get("Box"));
    assert_eq!(far.x, base + 7.0, "Far is :target");
    assert!((far.y - boxed.y - 16.0).abs() < 0.5, "{far:?} in {boxed:?}");

    // Back to Near, the scroller's first box, at offset 0.
    follow(&mut d, "Back");
    let near = d.get("Near");
    assert!((near.y - d.get("Box").y).abs() < 0.5, "{near:?}");
    assert_eq!(near.x, base + 7.0, "Near is :target");

    // A text directive is part of the id: nothing is :target.
    follow(&mut d, "Text");
    assert_eq!(d.get("Near").x, base, "Near is not :target");
}

/// Custom highlights (CSS Custom Highlight API 1) paint as the spec's
/// §4.2.5 example does. Each glyph of the test font's CJK text is 1em
/// wide, so character k spans [20k, 20k + 20] px. `Foo` covers characters
/// 0 to 6, blue on yellow, and `bar` covers 3 to 9 on orange; both are
/// styled once at the root and reach the paragraph by highlight inheritance
/// (CSS Pseudo-Elements 4 §3.5). Tied at priority 0, `bar`, registered
/// later, is above: yellow under orange, with the glyphs blue over 0 to 6
/// (orange sets no color). Once `Foo` has priority 1 it is above. A range
/// that crosses into the next element paints only where that element
/// styles its highlight, a collapsed range paints nothing, overlapping
/// ranges paint as one band, and removing a highlight clears it.
///
/// Ways it could fail: the overlays stacked by registration alone, by name,
/// or with the earlier registered above; `::highlight()` reaching only
/// the element its rule matched; the name's case lost in the cascade; a
/// range stopping at its first element; a union drawn twice; a collapsed
/// range drawn; the marks staying after the highlight is removed.
#[test]
fn custom_highlights_paint_their_ranges_in_priority_order() {
    let doc = r##"<meta name="component" content="hl">
<column id="Root">
  <p id="A">原字引擎文染渲点原</p>
  <p id="B">字引擎</p>
</column>
<style>
p { font-size: 20px; margin: 0; width: max-content; }
:root::highlight(Foo) { color: #0000ff; background-color: #ffff00; }
:root::highlight(bar) { background-color: #ffa500; }
#B::highlight(baz) { background-color: #ff0000; }
</style>"##;
    let mut d = Doc::with_font(doc, &[], test_font());
    let near = |a: [f32; 4], b: Srgba| {
        let b = b.linear();
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.02)
    };
    let (yellow, orange, red, blue) = (
        Srgba::rgb(255, 255, 0),
        Srgba::rgb(255, 165, 0),
        Srgba::rgb(255, 0, 0),
        Srgba::rgb(0, 0, 255),
    );
    // Each quad's colour and rect in drawing order, and the clips of the
    // blue and the other glyphs.
    let drawn = |d: &Doc| {
        let (mut quads, mut ink, mut other) = (vec![], vec![], vec![]);
        for item in &d.ui.frame().items {
            let cmd = match &item.cmd {
                DrawCmd::StyledText { text, .. } => text.as_ref(),
                c => c,
            };
            match cmd {
                DrawCmd::Quad { rect, color, .. } => {
                    for (name, c) in [("yellow", yellow), ("orange", orange), ("red", red)] {
                        if near(*color, c) {
                            quads.push((name, *rect));
                        }
                    }
                }
                DrawCmd::Text { color, clip, .. } if near(*color, blue) => ink.push(*clip),
                DrawCmd::Text { clip, .. } => other.push(*clip),
                _ => {}
            }
        }
        (quads, ink, other)
    };
    let (a, b) = (d.get("A"), d.get("B"));
    assert_eq!((a.width, b.width), (180.0, 60.0), "1em glyphs");
    let at = |k: u32| a.x + 20.0 * k as f32;
    let range =
        |el: &og_ui::UiElement, s: u32, e: &og_ui::UiElement, t: u32| og_ui::HighlightRange {
            start: el.path.clone(),
            start_offset: s,
            end: e.path.clone(),
            end_offset: t,
        };
    let set = |d: &mut Doc, name: &str, ranges: Vec<og_ui::HighlightRange>, priority: i32| {
        d.ui.set_highlight(name, Some(og_ui::Highlight { ranges, priority }));
        d.step(InputFrame::default());
    };
    assert!(drawn(&d).0.is_empty(), "nothing registered");

    // Foo's two overlapping ranges and a collapsed one, then bar.
    set(
        &mut d,
        "Foo",
        vec![
            range(&a, 0, &a, 4),
            range(&a, 2, &a, 6),
            range(&a, 8, &a, 8),
        ],
        0,
    );
    set(&mut d, "bar", vec![range(&a, 3, &a, 9)], 0);
    let span = |r: &og_ui::Rect| (r.x, r.right());
    let (quads, ink, other) = drawn(&d);
    let names: Vec<_> = quads.iter().map(|q| q.0).collect();
    assert_eq!(names, ["yellow", "orange"], "bar above Foo: {quads:?}");
    assert_eq!(span(&quads[0].1), (at(0), at(6)), "one band for the union");
    assert_eq!(span(&quads[1].1), (at(3), at(9)));
    assert!((quads[0].1.y - a.y).abs() < 1.0, "{quads:?} {a:?}");
    assert!(!ink.is_empty(), "no Foo ink");
    for c in &ink {
        assert!(c.x >= at(0) - 0.5 && c.right() <= at(6) + 0.5, "{c:?}");
    }
    assert!(
        other
            .iter()
            .any(|c| c.x >= at(6) - 0.5 && c.x < at(9) && (c.y - a.y).abs() < a.height),
        "the default ink past Foo: {other:?}"
    );

    // Foo at priority 1 rises above bar, keeping its ink.
    set(
        &mut d,
        "Foo",
        vec![range(&a, 0, &a, 4), range(&a, 2, &a, 6)],
        1,
    );
    let (quads, ink, _) = drawn(&d);
    let names: Vec<_> = quads.iter().map(|q| q.0).collect();
    assert_eq!(names, ["orange", "yellow"], "Foo above bar: {quads:?}");
    assert!(ink.iter().all(|c| c.right() <= at(6) + 0.5) && !ink.is_empty());

    // A range from A's eighth character into B's second: only B styles
    // baz, so only B's two characters are red.
    set(&mut d, "baz", vec![range(&a, 7, &b, 2)], 0);
    let (quads, ..) = drawn(&d);
    let reds: Vec<_> = quads.iter().filter(|q| q.0 == "red").collect();
    assert_eq!(reds.len(), 1, "{quads:?}");
    assert_eq!(span(&reds[0].1), (b.x, b.x + 40.0));
    assert!((reds[0].1.y - b.y).abs() < 1.0);

    // Removed, a highlight leaves no mark.
    for name in ["Foo", "bar", "baz"] {
        d.ui.set_highlight(name, None);
    }
    d.step(InputFrame::default());
    assert!(drawn(&d).0.is_empty(), "{:?}", drawn(&d).0);
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

/// `:heading` matches every `<h1>` to `<h6>` and `:heading(1, 3)` those of
/// the listed levels, in a rule lowered to a condition and in one matched
/// on the instantiated tree.
#[test]
fn heading_matches_headings_by_level() {
    let doc = r##"<meta name="component" content="heads">
<column id="Root">
  <h1 id="H1">a</h1><div class="probe" id="NextH1"></div>
  <h2 id="H2">b</h2><div class="probe" id="NextH2"></div>
  <h3 id="H3">c</h3><div class="probe" id="NextH3"></div>
  <p id="P">d</p><div class="probe" id="NextP"></div>
</column>
<style>
.probe { width: 10px; height: 4px; }
h1, h2, h3, p { width: 100px; margin-left: 0; }
:heading { margin-left: 20px; }
:heading(1, 3) { width: 150px; }
:heading + .probe { width: 30px; }
:heading(2) + .probe { width: 50px; }
</style>"##;
    with_doc(doc, &[], |d| {
        let of = |id: &str| (d.get(id).x, d.get(id).width);
        assert_eq!(
            [of("H1"), of("H2"), of("H3"), of("P")],
            [(20.0, 150.0), (20.0, 100.0), (20.0, 150.0), (0.0, 100.0)]
        );
        let next = ["NextH1", "NextH2", "NextH3", "NextP"].map(|id| d.get(id).width);
        assert_eq!(next, [30.0, 50.0, 30.0, 10.0]);
    });
    let refused = "<meta name=\"component\" content=\"h\">\n<h1>a</h1>\n<style>:heading(0) { width: 1px; }</style>";
    assert!(og_ui::UiDocument::parse("ui/h.svelte", refused).is_err());
}

/// `revert-rule` rolls a property back to what it is without the
/// keyword's own rule (CSS Cascade 5 §7.3): the next rule's declaration,
/// whatever its layer, where `revert-layer` would drop the whole layer.
///
/// Ways it could fail: the keyword read as `revert-layer` or `unset` (an
/// equal-layer rule below lost too), an earlier declaration of the same
/// rule surviving, a custom property or a `var()` fallback not rolled back.
#[test]
fn revert_rule_rolls_back_one_rule() {
    let doc = r##"<meta name="component" content="rr">
<column id="Root">
  <div class="probe on" id="Same"></div>
  <div class="probe on" id="Earlier"></div>
  <div class="probe on" id="Layered"></div>
  <div class="probe on" id="Custom"></div>
  <div class="probe on" id="Fallback"></div>
  <div class="probe" id="Off"></div>
  <div class="probe on" id="Text"></div>
  <div class="on" id="Alone"></div><div id="Plain"></div>
</column>
<style>
@layer low { #Layered { width: 70px; } }
.probe { width: 10px; height: 4px; }
#Same { width: 50px; }
#Same.on { width: revert-rule; }
#Earlier { width: 30px; }
#Earlier.on { width: 90px; width: revert-rule; }
#Layered.on { width: revert-rule; }
#Custom { --w: 50px; width: var(--w); }
#Custom.on { --w: revert-rule; }
#Fallback { width: 50px; }
#Fallback.on { width: var(--none, revert-rule); }
#Off { width: 50px; }
#Off.on { width: revert-rule; }
#Text { font-size: 20px; width: 2em; }
#Text.on { font-size: revert-rule; }
#Alone.on { margin-left: revert-rule; width: 20px; height: 6px; }
#Plain { margin-left: 8px; width: 20px; height: 6px; }
</style>"##;
    with_doc(doc, &[], |d| {
        let w =
            ["Same", "Earlier", "Layered", "Custom", "Fallback", "Off"].map(|id| d.get(id).width);
        assert_eq!(w, [50.0, 30.0, 10.0, 50.0, 50.0, 50.0]);
        assert_eq!(d.get("Text").width, 40.0, "an inherited text property");
        // With no rule below, the property is as if never declared; the
        // rule's other declarations stay.
        let (alone, plain) = (d.get("Alone"), d.get("Plain"));
        assert_eq!(
            (alone.x, alone.width, alone.height),
            (plain.x - 8.0, 20.0, 6.0)
        );
    });
}

/// The media pseudo-classes follow a `<video>`'s own state (HTML
/// §4.16.3): `:paused` until it plays and `:playing` after, `:muted` with
/// its `muted`, `:seeking` while a seek is on its way. `:buffering`,
/// `:stalled` and `:volume-locked` parse and never match: the engine's
/// players read local assets and mix their own sound.
#[test]
fn media_pseudo_classes_follow_the_element() {
    let doc = r##"<meta name="component" content="media">
<column id="Root">
  <video id="Clip" src={$player} bind:paused={$paused} bind:muted={$mute} bind:currentTime={$t}></video>
  <div class="probe" id="Playing"></div><div class="probe" id="Paused"></div>
  <div class="probe" id="Muted"></div><div class="probe" id="Seeking"></div>
  <div class="probe" id="Never"></div>
</column>
<style>
.probe { width: 10px; height: 4px; }
video { width: 100px; height: 50px; }
video:playing ~ #Playing, video:paused ~ #Paused, video:muted ~ #Muted, video:seeking ~ #Seeking { width: 50px; }
video:buffering ~ #Never, video:stalled ~ #Never, video:volume-locked ~ #Never { width: 50px; }
</style>"##;
    let stores = [
        ("player", json!(7)),
        ("paused", json!(true)),
        ("mute", json!(false)),
        ("t", json!(0)),
    ];
    let mut d = Doc::new(doc, &stores);
    d.step(InputFrame::default());
    let w = |d: &Doc| ["Playing", "Paused", "Muted", "Seeking", "Never"].map(|id| d.get(id).width);
    assert_eq!(w(&d), [10.0, 50.0, 10.0, 10.0, 10.0], "paused");
    d.set("paused", json!(false));
    d.step(InputFrame::default());
    assert_eq!(w(&d), [50.0, 10.0, 10.0, 10.0, 10.0], "playing");
    d.set("mute", json!(true));
    d.step(InputFrame::default());
    assert_eq!(w(&d), [50.0, 10.0, 50.0, 10.0, 10.0], "muted");
    d.set("t", json!(5));
    assert_eq!(w(&d)[3], 50.0, "seeking");
    for _ in 0..4 {
        d.step(InputFrame::default());
    }
    assert_eq!(w(&d), [50.0, 10.0, 50.0, 10.0, 10.0], "the seek arrived");
}

const CARD: &str = r##"<meta name="component" content="Card">
<meta name="prop:tone" content="string" data-default="">
<meta name="prop:wide" content="bool" data-default="false">
<meta name="prop:class" content="string" data-default="">
<div id="Frame" class="frame">
  <div class="probe" id="Any"></div>
  <div class="probe" id="Warn"></div>
  <div class="probe" id="Wide"></div>
  <div class="probe" id="Narrow"></div>
  <div class="probe" id="Hover"></div>
  <div class="probe" id="Class"></div>
  <div class="probe" id="Tag"></div>
  <div class="probe" id="OtherTag"></div>
  <div class="probe" id="Second"></div>
  <div class="frame" id="Inner"><div class="probe" id="Deep"></div></div>
  <ul id="List"><li class="probe" id="A"></li><li class="probe" id="B"></li></ul>
</div>
<style>
.probe { width: 10px; height: 4px; }
:host { padding-left: 7px; --accent: 50px; }
:host #Any { width: var(--accent); }
:host([tone=warn]) #Warn { width: 50px; }
:host([wide]) > #Wide { width: 50px; }
:host(:not([wide])) #Narrow { width: 50px; }
:host(:hover) #Hover { width: 50px; }
:host(.featured) #Class { width: 50px; }
:host(Card) #Tag, :host(Other) #OtherTag { width: 50px; }
:host(:nth-child(2)) #Second { width: 50px; }
:host > #Deep { width: 50px; }
:host([tone=warn]) li:nth-child(2) { width: 50px; }
</style>"##;

/// `:host` selects a component's instance, whose box is the component's
/// root: its declarations style that box, `:host(<compound>)` reads the
/// attributes written on the instance (the component's props, `class`
/// among them), the root's state and the instance's place among its
/// siblings, and a rule that also reads sibling order inside the
/// component holds only under the same host.
///
/// Ways it could fail: `:host` reading the root element's own class or
/// tag instead of the instance's, a `false` or unpassed prop counting as
/// present, the host's condition not following its store, `:host >`
/// reaching past the root's children, one instance's host deciding for
/// another, or the tree-matched rule ignoring the host.
#[test]
fn host_selects_the_instance() {
    let page = r##"<meta name="component" content="page">
<column id="Root">
  <Card id="One" class="big featured" tone={$tone} wide={$wide} />
  <Card id="Two" />
</column>"##;
    let stores = [("tone", json!("warn")), ("wide", json!(true))];
    let mut d = Doc::with_components(&[CARD], page, &stores);
    let ids = [
        "Any",
        "Warn",
        "Wide",
        "Narrow",
        "Class",
        "Tag",
        "OtherTag",
        "Second",
        "Inner/Deep",
        "List/A",
        "List/B",
    ];
    let w = |d: &Doc, card: &str| ids.map(|id| d.get(&format!("{card}/{id}")).width);
    assert_eq!(
        w(&d, "One"),
        [
            50.0, 50.0, 50.0, 10.0, 50.0, 50.0, 10.0, 10.0, 10.0, 10.0, 50.0
        ]
    );
    assert_eq!(
        w(&d, "Two"),
        [
            50.0, 10.0, 10.0, 50.0, 10.0, 50.0, 10.0, 50.0, 10.0, 10.0, 10.0
        ]
    );
    // The declarations are the root's box's: its content starts 7px in.
    for card in ["One", "Two"] {
        assert_eq!(d.get(&format!("{card}/Any")).x - d.get(card).x, 7.0);
    }
    d.set("tone", json!("calm"));
    d.set("wide", json!(false));
    assert_eq!(
        w(&d, "One"),
        [
            50.0, 10.0, 10.0, 50.0, 50.0, 50.0, 10.0, 10.0, 10.0, 10.0, 10.0
        ]
    );
}

/// `:host(<state>)` reads the host's live state, not an attribute: a
/// component whose root is a button is `:host(:hover)` under the pointer
/// and `:host(:disabled)` while its root is disabled, and only that
/// instance.
#[test]
fn host_reads_the_instances_state() {
    let chip = r##"<meta name="component" content="Chip">
<meta name="prop:off" content="bool" data-default="false">
<button id="Face" disabled={off}>x</button>
<style>
:host { width: 40px; height: 20px; padding: 0; }
:host(:hover) { width: 90px; }
:host(:disabled) { height: 30px; }
</style>"##;
    let page = r##"<meta name="component" content="page">
<column id="Root"><Chip id="One" /><Chip id="Two" off={$off} /></column>"##;
    let mut d = Doc::with_components(&[chip], page, &[("off", json!(true))]);
    let size = |d: &Doc| ["One", "Two"].map(|id| (d.get(id).width, d.get(id).height));
    assert_eq!(size(&d), [(40.0, 20.0), (40.0, 30.0)]);
    let one = d.get("One");
    let over = InputFrame {
        pointer: Vec2::new(one.x + 5.0, one.y + 5.0),
        pointer_inside: true,
        ..InputFrame::default()
    };
    d.step(over.clone());
    d.step(over);
    assert_eq!(size(&d), [(90.0, 20.0), (40.0, 30.0)]);
    d.set("off", json!(false));
    assert_eq!(size(&d)[1], (40.0, 20.0));
}

/// What `:host` cannot mean is refused with its fix: a feature beside
/// `:host` (a host is featureless inside its component), an instance
/// attribute the component declares no prop for, an id, and a component
/// whose root is itself an instance.
#[test]
fn host_refusals_name_their_fix() {
    let component = |css: &str, body: &str| {
        let text = format!(
            "<meta name=\"component\" content=\"Card\">\n<meta name=\"prop:tone\" content=\"string\" data-default=\"\">\n{body}\n<style>{css}</style>"
        );
        og_ui::UiDocument::parse("ui/card.svelte", &text)
            .err()
            .map(|e| e.join("\n"))
            .unwrap_or_default()
    };
    let div = "<div id=\"Frame\"></div>";
    for (css, said) in [
        (":host.wide { width: 1px; }", "write `:host(.wide)`"),
        (":host:hover { width: 1px; }", "write `:host(:hover)`"),
        (":host([size=big]) { width: 1px; }", "prop:size"),
        (":host(.wide) { width: 1px; }", "prop:class"),
        (
            ":host(#One) { width: 1px; }",
            "an instance's id names its root's path",
        ),
    ] {
        let error = component(css, div);
        assert!(error.contains(said), "{css}: {error}");
    }
    assert_eq!(component(":host([tone=warn]) { width: 1px; }", div), "");
    let error = component(":host { width: 1px; }", "<Other />");
    assert!(
        error.contains("this component's root is a <Other> instance"),
        "{error}"
    );
}

const PANEL: &str = r##"<meta name="component" content="Panel">
<meta name="prop:children" content="content">
<meta name="prop:footer" content="content">
<meta name="prop:open" content="bool" data-default="false">
<div id="Frame">
  <div id="Body" class="body" class:open={open}>{@render children?.()}</div>
  <div id="Foot" class="foot">{@render footer()}</div>
  <div id="Own" class="own"><Chip id="Mine" /></div>
</div>
{#snippet footer()}<p id="Default">d</p>{/snippet}
<style>
.own::slotted(p) { margin-left: 21px; }
.own:has-slotted { min-height: 40px; }
::slotted(p) { margin-left: 9px; width: 50px; }
.body::slotted(.lead) { margin-top: 5px; }
.body.open::slotted(p) { margin-left: 30px; }
.foot::slotted(p) { margin-left: 13px; height: 8px !important; }
.body:has-slotted { padding-left: 3px; min-height: 40px; }
.foot:has-slotted { padding-left: 4px; }
</style>"##;

/// A component with rules of its own for its root.
const CHIP: &str = r##"<meta name="component" content="Chip">
<p id="Chip" class="chip">x</p>
<style>.chip { margin-left: 2px; width: 30px; height: 6px; }</style>"##;

/// A `{@render}` site is the slot. `::slotted(<compound>)` selects the
/// top-level elements of the content rendered there, from the element
/// around the site, and ranks under the content's own document's rules
/// unless it is `!important`; `:has-slotted` matches that element while
/// passed content, not the component's own fallback, is rendered in it.
/// A component instance passed as content is slotted like an element, and
/// the rule wins over its component's rules for its root; an instance the
/// component wrote itself is not.
///
/// Ways it could fail: `::slotted()` reaching a nested element, the
/// component's own fallback or content rendered at another site; winning
/// over the content's document by specificity or order; losing to a
/// user-agent rule; not following the holder's class; `:has-slotted`
/// counting the fallback, or not following content that comes and goes;
/// a passed instance read as the component's own (its root is another
/// document's root either way), or the reverse.
#[test]
fn slotted_selects_content_rendered_at_a_site() {
    let page = r##"<meta name="component" content="page">
<column id="Root">
  <Panel id="One" open={$open}><p id="Lead" class="lead">a</p><div id="Rest"><p id="Deep">b</p></div></Panel>
  <Panel id="Two">{#snippet footer()}<p id="Sig">c</p>{/snippet}</Panel>
  <Panel id="Three">{#if $show}<p id="Late">e</p>{/if}</Panel>
  <Panel id="Four"><Chip id="C" /></Panel>
</column>
<style>p { width: 20px; height: 4px; } #Rest { height: 4px; }</style>"##;
    let stores = [("open", json!(false)), ("show", json!(false))];
    let mut d = Doc::with_components(&[CHIP, PANEL], page, &stores);
    // Left edge within its instance, and size.
    let at = |d: &Doc, card: &str, id: &str| {
        let e = d.get(&format!("{card}/{id}"));
        (e.x - d.get(card).x, e.width, e.height)
    };
    // The holder's 3px, the rule's 9px; the page's own width stands.
    assert_eq!(at(&d, "One", "Body/children/Lead"), (12.0, 20.0, 4.0));
    let (lead, body) = (d.get("One/Body/children/Lead"), d.get("One"));
    assert_eq!(lead.y - body.y, 5.0, ".body::slotted(.lead)");
    // A nested element is not slotted, nor is the component's fallback.
    assert_eq!(at(&d, "One", "Body/children/Rest/Deep").0, 3.0);
    assert_eq!(at(&d, "One", "Foot/footer/Default").0, 0.0);
    // The other site's rule, later in the sheet, and `!important` over
    // the page's own height.
    assert_eq!(at(&d, "Two", "Foot/footer/Sig"), (17.0, 20.0, 8.0));
    d.set("open", json!(true));
    assert_eq!(
        at(&d, "One", "Body/children/Lead").0,
        33.0,
        "the holder's class"
    );
    // `:has-slotted` follows what is rendered at the site.
    let height = |d: &Doc, card: &str| d.get(&format!("{card}/Body")).height;
    assert_eq!(
        [height(&d, "One"), height(&d, "Two"), height(&d, "Three")],
        [40.0, 0.0, 0.0]
    );
    d.set("show", json!(true));
    assert_eq!(at(&d, "Three", "Body/children/Late").0, 12.0);
    assert_eq!(height(&d, "Three"), 40.0);
    d.set("show", json!(false));
    assert_eq!(height(&d, "Three"), 0.0);
    // A passed instance is slotted: the holder's 3px and the rule's 9px
    // and width over its component's own, whose height stands.
    assert_eq!(at(&d, "Four", "Body/children/C"), (12.0, 50.0, 6.0));
    assert_eq!(height(&d, "Four"), 40.0);
    // The panel's own instance is not: its component's margin and width.
    assert_eq!(at(&d, "Four", "Own/Mine"), (2.0, 30.0, 6.0));
    assert_eq!(d.get("Four/Own").height, 6.0);
}

/// What `::slotted()` does not read is refused with what to write.
#[test]
fn slotted_refusals_name_their_fix() {
    let said = |css: &str| {
        let text = format!(
            "<meta name=\"component\" content=\"Panel\">\n<meta name=\"prop:open\" content=\"bool\" data-default=\"false\">\n<div class=\"b\">{{@render children?.()}}</div>\n<style>{css}</style>"
        );
        og_ui::UiDocument::parse("ui/panel.svelte", &text)
            .err()
            .map(|e| e.join("\n"))
            .unwrap_or_default()
    };
    for (css, why) in [
        ("::slotted(p) span { width: 1px; }", "ends the selector"),
        (
            "::slotted(p):hover { width: 1px; }",
            "as ::slotted(p:hover)",
        ),
        (
            "::slotted(div p) { width: 1px; }",
            "takes one compound selector",
        ),
        (
            "::slotted(p)::before { content: 'x'; }",
            "a pseudo-element after ::slotted()",
        ),
        (
            ":host([open]) .b::slotted(p) { width: 1px; }",
            ":host() before ::slotted()",
        ),
    ] {
        let error = said(css);
        assert!(error.contains(why), "{css}: {error}");
    }
    assert_eq!(said(".b::slotted(p:hover) { width: 1px; }"), "");
}
