//! Bound CSS custom properties (`style:--name={…}`) and the math functions
//! that read them (`calc()`, `min()`, `max()`, `clamp()`), built as a
//! document is (mounted, retained).
//!
//! Ways these could fail, each covered below:
//! - a bound value not reaching var(), read once and never again when its
//!   store changes, or not inherited by a child that reads it; a literal
//!   one ignored; an `!important` rule losing to it; a null value set
//!   instead of leaving var()'s fallback; a keyed insert keeping stale
//!   values → `bound_custom_properties_feed_the_cascade`
//! - calc() left unreduced (or reduced wrongly) where a rule reads a
//!   variable, or a mixed-unit calc() accepted → `calc_reduces_once_its_variables_are_known`
//! - env(safe-area-inset-*) blind to the host's covered control, folding it
//!   into the wrong edge for the surface's shape, or not re-read when the
//!   surface or the control changes; env(-og-hud-inset-*) not narrowing
//!   to the HUD band on an ultrawide (or narrowing a 16:9 surface), or not
//!   following UiDesc.hudMaxAspect → `safe_area_insets_clear_the_hosts_covered_control`

use crate::support::{Doc, with_doc};
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
    with_doc(
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
            );
            assert_eq!(heights(d), [6.0, 10.0, 14.0]);
        },
    );
}

/// Sets signal store `name`.
fn set(d: &mut Doc, name: &str, value: serde_json::Value) {
    d.edit(name, json!({"op": "set", "value": value}));
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
    with_doc(text, &[], |d| {
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
    with_doc(text, &[("w", json!(2))], |d| {
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

#[test]
fn safe_area_insets_clear_the_hosts_covered_control() {
    // A HUD at the insets' corners, as a document places one: the page's
    // Back (40 CSS px at 12, 12, top left) widens the left inset on a wide
    // surface and deepens the top one on a tall one (a portrait phone's
    // over its notch's 47), and a change of surface re-resolves it.
    let text = r#"<meta name="component" content="t">
<div><div id="Hud">
  <div id="TL"></div><div id="BL"></div><div id="HR"></div>
</div><div id="Flow"><div id="Plain"><div id="PlainChild"></div></div><div id="Padded"><div id="PaddedChild"></div></div></div><div id="FixedPage"><div id="FixedChild"></div></div></div>
<style>
#Flow { display: flex; }
#Plain, #Padded { width: 100px; }
#Plain { padding-top: 8px; }
#Padded { padding-top: calc(env(safe-area-inset-top) + 8px); }
#PlainChild, #PaddedChild { height: 10px; }
#FixedPage { position: fixed; inset: 0; padding-top: calc(env(safe-area-inset-top) + 8px); }
#FixedChild { width: 10px; height: 10px; }
#TL { position: fixed; left: env(safe-area-inset-left); top: env(safe-area-inset-top); width: 100px; height: 20px; }
#BL { position: fixed; left: max(8px, env(safe-area-inset-left)); bottom: calc(env(safe-area-inset-bottom) + 4px); width: 100px; height: 20px; }
#HR { position: fixed; right: calc(env(-og-hud-inset-right) + 24px); top: env(-og-hud-inset-top); width: 10px; height: 10px; }
</style>"#;
    let back = Rect::new(12.0, 12.0, 40.0, 40.0);
    with_doc(text, &[], |d| {
        assert_eq!(
            (d.get("TL").x, d.get("TL").y),
            (0.0, 0.0),
            "no control: the corner"
        );
        d.viewport.covered = back;
        d.step(InputFrame::default());
        assert_eq!(
            (d.get("TL").x, d.get("TL").y),
            (0.0, 52.0),
            "800 × 900: below it"
        );
        d.viewport = Viewport::new(1280.0, 720.0);
        d.viewport.covered = back;
        d.step(InputFrame::default());
        assert_eq!(
            (d.get("TL").x, d.get("TL").y),
            (52.0, 0.0),
            "a wide surface: beside it"
        );
        let mut phone = Viewport::new(390.0, 844.0);
        phone.safe_area = Insets {
            left: 0.0,
            top: 47.0,
            right: 0.0,
            bottom: 34.0,
        };
        d.viewport = phone;
        d.step(InputFrame::default());
        assert_eq!(
            d.get("Flow").y,
            47.0,
            "the host root already pads normal flow"
        );
        assert_eq!(d.get("PlainChild").y, 55.0);
        assert_eq!(
            d.get("FixedChild").y,
            55.0,
            "a viewport page applies the inset once"
        );
        assert_eq!(
            d.get("PaddedChild").y,
            102.0,
            "env is viewport-relative, not the remaining inset after ancestor padding"
        );
        assert_eq!(
            d.get("TL").y,
            47.0,
            "fixed positioning starts at the viewport"
        );
        phone.covered = Rect::new(12.0, 59.0, 40.0, 40.0);
        d.viewport = phone;
        d.step(InputFrame::default());
        assert_eq!(
            (d.get("TL").x, d.get("TL").y),
            (0.0, 99.0),
            "portrait: below it"
        );
        assert_eq!(
            d.get("FixedChild").y,
            107.0,
            "the viewport page also clears the covered control"
        );
        assert_eq!(
            (d.get("BL").x, d.get("BL").y),
            (8.0, 844.0 - 34.0 - 4.0 - 20.0)
        );
        let mut landscape = Viewport::new(844.0, 390.0);
        landscape.safe_area = Insets {
            left: 47.0,
            top: 0.0,
            right: 47.0,
            bottom: 21.0,
        };
        landscape.covered = Rect::new(59.0, 12.0, 40.0, 40.0);
        d.viewport = landscape;
        d.step(InputFrame::default());
        assert_eq!(
            (d.get("TL").x, d.get("TL").y),
            (99.0, 0.0),
            "landscape: beside it"
        );
        assert_eq!(d.get("BL").x, 99.0, "the whole edge, as a notch's inset");
        d.viewport.covered = Rect::default();
        d.step(InputFrame::default());
        assert_eq!(d.get("TL").x, 47.0, "cleared: the notch's inset alone");
        let right = |d: &Doc| d.get("HR").x + d.get("HR").width;
        assert_eq!(right(d), 844.0 - 47.0 - 24.0, "under 21:9: the safe area");
        // 32:9: the band is 21/9 × 720 = 1680 wide, centred.
        d.viewport = Viewport::new(2560.0, 720.0);
        d.step(InputFrame::default());
        assert_eq!(right(d), 2560.0 - 440.0 - 24.0, "an ultrawide: the band");
        assert_eq!(d.get("HR").y, 0.0);
        d.ui.set_hud_max_aspect(0.0);
        d.step(InputFrame::default());
        assert_eq!(right(d), 2560.0 - 24.0, "hudMaxAspect 0: the whole width");
    });
}
