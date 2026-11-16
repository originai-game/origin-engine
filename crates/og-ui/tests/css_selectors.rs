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
