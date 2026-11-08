//! Bound CSS custom properties (`style:--name={…}`) and the math functions
//! that read them (`calc()`, `min()`, `max()`, `clamp()`), built both ways
//! a document is (immediate and mounted, retained).
//!
//! Ways these could fail, each covered below:
//! - a bound value not reaching var(), read once and never again when its
//!   store changes, or not inherited by a child that reads it; a literal
//!   one ignored; an `!important` rule losing to it; a null value set
//!   instead of leaving var()'s fallback; a keyed insert keeping stale
//!   values → `bound_custom_properties_feed_the_cascade`
//! - calc() left unreduced (or reduced wrongly) where a rule reads a
//!   variable, or a mixed-unit calc() accepted → `calc_reduces_once_its_variables_are_known`

use crate::html_text::{Doc, both};
use og_ui::*;
use serde_json::json;

#[test]
fn bound_custom_properties_feed_the_cascade() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="Bar" style:--fill={$hp / $max}><div id="Fill"></div></div>
  <div id="Fixed" style:--w="40px"></div>
  <div id="Imp" style:--w="10px"></div>
  <div id="Unset" style:--fill={$missing}></div>
  {#each $stars as s (s.id)}<div class="star" style:--rank={s.rank}></div>{/each}
</div></div>
<style>
#Bar { width: 200px; height: 10px; }
#Fill { width: calc(var(--fill) * 100%); height: 10px; }
#Fixed { width: var(--w); height: 5px; }
#Imp { --w: 30px !important; width: var(--w); height: 5px; }
#Unset { width: calc(var(--fill, 0.25) * 100px); height: 5px; }
.star { width: 10px; height: calc(var(--rank) * 4px + 2px); }
</style>"#;
    let stars = json!([{"id": "a", "rank": 1}, {"id": "c", "rank": 3}]);
    both(
        text,
        &[
            ("hp", json!(50)),
            ("max", json!(100)),
            ("missing", json!(null)),
            ("stars", stars),
        ],
        |d| {
            // Bar sets it, Fill inherits it: half of Bar's 200px.
            assert_eq!(d.get("Fill").width, 100.0);
            assert_eq!(d.get("Fixed").width, 40.0);
            assert_eq!(d.get("Imp").width, 30.0, "!important beats style:");
            assert_eq!(d.get("Unset").width, 25.0, "null leaves the fallback");
            let heights = |d: &Doc| -> Vec<f32> {
                let mut s: Vec<_> =
                    d.ui.probe_elements()
                        .into_iter()
                        .filter(|e| e.path.contains("/Root/") && e.width == 10.0)
                        .collect();
                s.sort_by(|a, b| a.y.total_cmp(&b.y));
                s.iter().map(|e| e.height).collect()
            };
            assert_eq!(heights(d), [6.0, 14.0]);
            // A change re-reads it, and the child that reads it follows.
            set(d, "hp", json!(25));
            assert_eq!(d.get("Fill").width, 50.0);
            set(d, "hp", json!(130));
            assert_eq!(d.get("Fill").width, 260.0);
            d.edit(
                "stars",
                json!({"op": "insert", "row": {"id": "b", "rank": 2}, "before": "c"}),
                json!([{"id": "a", "rank": 1}, {"id": "b", "rank": 2}, {"id": "c", "rank": 3}]),
            );
            assert_eq!(heights(d), [6.0, 10.0, 14.0]);
        },
    );
}

/// Sets signal store `name` (a viewmodel entry when immediate).
fn set(d: &mut Doc, name: &str, value: serde_json::Value) {
    d.edit(name, json!({"op": "set", "value": value}), value.clone());
}

#[test]
fn calc_reduces_once_its_variables_are_known() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="A" style:--n="3"></div>
  <div id="B"></div>
  <div id="C" style:--n="0.5"></div>
</div></div>
<style>
#Root { width: 300px; }
#A { width: clamp(20px, calc(var(--n) * 10px), 100px); height: max(4px, calc(var(--n) * 1px)); }
#B { width: calc((10px + 6px) / 2); height: min(9px, 7px); }
#C { width: calc(var(--n) * 100%); height: 3px; }
</style>"#;
    both(text, &[], |d| {
        assert_eq!((d.get("A").width, d.get("A").height), (30.0, 4.0));
        assert_eq!((d.get("B").width, d.get("B").height), (8.0, 7.0));
        assert_eq!(d.get("C").width, 150.0);
    });
    // CSS Values 4 and 5: physical and font-relative units, the stepped
    // functions layout evaluates against a percentage's basis, env() and
    // attr() (bound, so a change re-resolves its element).
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="D" data-w="42"></div>
  <div id="G" data-w="wide"></div>
  <div id="H" data-w={$w}></div>
  <div id="E"></div>
  <p id="F">x</p>
</div></div>
<style>
#Root { width: 300px; }
#D { width: attr(data-w px, 5px); height: calc(env(safe-area-inset-top) + 2px); }
#G { width: attr(data-w px, 5px); height: 1lh; }
#H { width: calc(attr(data-w number, 1) * 1in); height: 4px; }
#E { width: round(down, 50%, 40px); height: calc(1in / 8); }
#F { font: var(--text-caption); width: 2em; height: calc(3pt * 4); }
</style>"#;
    both(text, &[("w", json!(2))], |d| {
        assert_eq!((d.get("D").width, d.get("D").height), (42.0, 2.0));
        assert_eq!(d.get("G").width, 5.0, "not a number: the fallback");
        assert_eq!(d.get("G").height, 20.0, "1lh is the body step's line");
        assert_eq!(d.get("H").width, 192.0);
        set(d, "w", json!(1));
        assert_eq!(d.get("H").width, 96.0);
        assert_eq!((d.get("E").width, d.get("E").height), (120.0, 12.0));
        assert_eq!((d.get("F").width, d.get("F").height), (26.0, 16.0));
    });
    let mixed = UiDocument::parse(
        "ui/t.svelte",
        "<meta name=\"component\" content=\"t\"><div id=\"A\"></div><style>#A { width: calc(100% - 12ms); }</style>",
    )
    .unwrap_err()
    .join("\n");
    assert!(mixed.contains("calc() mixes % and ms"), "{mixed}");
}
