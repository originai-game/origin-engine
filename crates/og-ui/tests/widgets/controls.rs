use super::*;
use serde_json::{Value, json};

/// Row `key`'s content height: 31, 54 or 77 px by the number in its key.
fn row(key: &str) -> Value {
    let n: usize = key.trim_start_matches("item").parse().unwrap_or(0);
    json!({ "id": key, "h": format!("{}px", 31 + (n % 3) * 23) })
}

const ROWS: &str = r#"<meta name="component" content="t">
<column id="Root">
<list id="Rows" row-height="40" width="240" height="150">
{#each $rows as r (r.id)}<div id="Content" style:height={r.h}></div>{/each}
</list>
</column>
<style>#Rows { gap: 3px; }</style>"#;

/// The rows built under list `list` (its window), as probe elements.
fn built(d: &Doc, list: &str, item: &str) -> Vec<UiElement> {
    d.ui.probe_elements()
        .into_iter()
        .filter(|e| e.parent == list && e.id.starts_with(item))
        .collect()
}

#[test]
fn virtual_rows_leave_focus_selection_and_padding_to_their_controls() {
    let rows = json!([
        {"id":"a", "name":"Ember Blade"},
        {"id":"b", "name":"Health Potion"},
        {"id":"c", "name":"Iron Shield"}
    ]);
    let text = r#"<meta name="component" content="t">
<div id="Root" role="grid">
  <div id="Head"><text id="Heading">Item</text></div>
  <list id="Rows" row-height="32" height="120">
    {#each $rows as r (r.id)}
      <div id="Content">{#if $controls}<button id="Row" role="row" aria-selected={r.id == "b"} onclick={choose}><div id="Name">{r.name}</div></button>{:else}<text>{r.name}</text>{/if}</div>
    {/each}
  </list>
</div>
<style>
#Root { width: 240px; }
#Head, #Row { width: stretch; height: 32px; padding: 0 12px; border: 0; box-sizing: border-box; }
#Head { display: flex; align-items: center; }
#Row::part(base) { justify-content: start; padding: 0 12px; }
#Rows, #Content { width: stretch; }
</style>"#;
    for (tag, prefix) in [("list", "Row"), ("cells", "Cell")] {
        let text = text
            .replace("<list ", &format!("<{tag} "))
            .replace("</list>", &format!("</{tag}>"));
        let text = if tag == "cells" {
            text.replace(
                "row-height=\"32\"",
                "cell-height=\"32\" cell-width=\"240\" cell-columns=\"1\"",
            )
        } else {
            text
        };
        let mut d = mount(&text, &[("rows", rows.clone()), ("controls", json!(true))]);
        let a = format!("Rows/{prefix}#a/Content#a/Row");
        let b = format!("Rows/{prefix}#b/Content#b/Row");
        let c = format!("Rows/{prefix}#c/Content#c/Row");
        assert_eq!(
            d.get(&format!("{a}/Name")).x,
            d.get("Heading").x,
            "cells align with the heading"
        );
        assert!(d.ui.elements().iter().all(|n| n.role != "option"));
        assert!(d.a11y(&b).states.contains(&"selected".to_string()));
        d.ui.navigate(Nav::Next);
        idle(&mut d, 1);
        assert_eq!(focus(&d, 0), a);
        d.step(InputFrame {
            keys: vec![UiKey::Down],
            ..Default::default()
        });
        assert_eq!(focus(&d, 0), b);
        d.step(InputFrame {
            gamepad: GamepadFrame {
                pressed: GamepadFrame::DOWN,
                ..Default::default()
            },
            ..Default::default()
        });
        assert_eq!(focus(&d, 0), c);
        let events = d.click(&b);
        assert!(events.iter().any(|e| e.name == "choose"));
        assert!(
            events.iter().all(|e| e.kind != "select"),
            "no generated option selection"
        );
        d.edit("rows", json!({"op":"move", "key":"b", "before":"a"}));
        assert_eq!(
            focus(&d, 0),
            b,
            "focus remains on the authored keyed control"
        );
        assert!(d.get(&b).y < d.get(&a).y);
        d.set("controls", json!(false));
        assert_eq!(d.a11y(&format!("Rows/{prefix}#b")).role, "option");
        d.set("controls", json!(true));
        assert_eq!(d.get(&format!("{a}/Name")).x, d.get("Heading").x);
        for index in 0..12 {
            d.edit(
                "rows",
                json!({"op":"insert", "row":{"id":format!("tail{index}"), "name":"More"}}),
            );
        }
        d.focus = Some(format!("test:t/{b}"));
        idle(&mut d, 1);
        for key in ["a".to_string(), "c".to_string()]
            .into_iter()
            .chain((0..12).map(|i| format!("tail{i}")))
        {
            d.step(InputFrame {
                keys: vec![UiKey::Down],
                ..Default::default()
            });
            assert_eq!(
                focus(&d, 0),
                format!("Rows/{prefix}#{key}/Content#{key}/Row")
            );
        }
        let at = center(&d, "Rows");
        d.step(InputFrame {
            wheel: Vec2::new(0.0, 5000.0),
            ..InputFrame::hover(at.x, at.y)
        });
        idle(&mut d, 2);
        let last = d.get(&format!("Rows/{prefix}#tail11/Content#tail11/Row"));
        let viewport = d.get("Rows");
        assert!((last.y + last.height - viewport.y - viewport.height).abs() < 0.5);
        assert!(d.ui.elements().iter().all(|n| n.role != "option"));
        d.step(InputFrame {
            wheel: Vec2::new(0.0, -5000.0),
            ..InputFrame::hover(at.x, at.y)
        });
        idle(&mut d, 2);
        let start = d.get(&b).y;
        let pointer = center(&d, &a);
        d.events.clear();
        let mut fingers = Fingers::new();
        d.step(fingers.at(&[(0, pointer)]));
        d.step(fingers.at(&[(0, pointer), (1, pointer + Vec2::new(40.0, 0.0))]));
        for k in 1..=3 {
            let moved = pointer - Vec2::new(0.0, 12.0 * k as f32);
            d.step(fingers.at(&[(0, moved), (1, moved + Vec2::new(40.0, 0.0))]));
        }
        d.step(fingers.lift(1));
        d.step(fingers.lift(0));
        assert!(
            d.el(&b).is_none_or(|row| row.y < start),
            "two fingers scroll the authored controls ({tag})"
        );
        assert!(
            d.events
                .iter()
                .all(|e| e.name != "choose" && e.kind != "select")
        );
    }
}

#[test]
fn keyed_rows_measure_anchor_navigate_and_restore() {
    let keys: Vec<String> = (0..100).map(|n| format!("item{n}")).collect();
    let rows: Vec<Value> = keys.iter().map(|k| row(k)).collect();
    let mut d = mount(ROWS, &[("rows", json!(rows))]);
    idle(&mut d, 2);
    assert!(
        built(&d, "Rows", "Row#").len() < 15,
        "only a window is built"
    );
    assert_ne!(
        d.get("Rows/Row#item0").height,
        d.get("Rows/Row#item1").height
    );
    d.ui.navigate(Nav::Next);
    idle(&mut d, 1);
    for i in 1..25 {
        let input = if i % 2 == 0 {
            InputFrame {
                keys: vec![UiKey::Down],
                ..Default::default()
            }
        } else {
            InputFrame {
                gamepad: GamepadFrame {
                    pressed: GamepadFrame::DOWN,
                    ..Default::default()
                },
                ..Default::default()
            }
        };
        d.step(input);
        assert_eq!(focus(&d, 0), format!("Rows/Row#item{i}"));
    }
    idle(&mut d, 3);
    let anchor = built(&d, "Rows", "Row#")
        .into_iter()
        .filter(|e| e.y >= 0.0)
        .min_by(|a, b| a.y.total_cmp(&b.y))
        .unwrap();
    // Above the viewport: an insert at the start, rows 1 and 3 swapped,
    // row 5 removed.
    d.edit(
        "rows",
        json!({"op": "insert", "row": row("inserted"), "before": "item0"}),
    );
    d.edit(
        "rows",
        json!({"op": "move", "key": "item2", "before": "item0"}),
    );
    d.edit(
        "rows",
        json!({"op": "move", "key": "item0", "before": "item3"}),
    );
    d.edit("rows", json!({"op": "remove", "key": "item4"}));
    idle(&mut d, 3);
    assert!(
        (d.get(anchor.path.strip_prefix("test:t/").unwrap()).y - anchor.y).abs() < 0.01,
        "edits above the viewport preserve the visible item's position"
    );
    assert_eq!(focus(&d, 0), "Rows/Row#item24", "focus belongs to the key");
    let saved = d.ui.save_state();
    idle(&mut d, 1);
    let expected = d.ui.frame().items.clone();
    let mut now: Vec<&str> = vec!["inserted", "item2", "item1", "item0", "item3"];
    now.extend(keys[5..].iter().map(String::as_str));
    let rows: Vec<Value> = now.iter().map(|k| row(k)).collect();
    let mut restored = mount(ROWS, &[("rows", json!(rows))]);
    restored.ui.restore_state(&saved).unwrap();
    idle(&mut restored, 1);
    assert_eq!(
        restored.ui.frame().items,
        expected,
        "fresh UI restores measured extents"
    );
}

#[test]
fn keyed_grid_uses_tallest_cells_and_reaches_unbuilt_end() {
    let cells: Vec<Value> = (0..41)
        .map(|n| json!({ "id": n.to_string(), "h": if n % 2 == 0 { "35px" } else { "81px" } }))
        .collect();
    let mut d = mount(
        r#"<meta name="component" content="t">
<column id="Root">
<cells id="Cells" cell-columns="2" cell-width="100" cell-height="60" height="180">
{#each $cells as c (c.id)}<div id="Content" style:height={c.h}></div>{/each}
</cells>
</column>
<style>#Cells { gap: 4px; width: fit-content; }</style>"#,
        &[("cells", json!(cells))],
    );
    idle(&mut d, 2);
    let short = d.get("Cells/Cell#0");
    let tall = d.get("Cells/Cell#1");
    let next = d.get("Cells/Cell#2");
    assert_eq!(short.y, tall.y);
    assert!((next.y - (tall.y + tall.height + 4.0)).abs() < 0.01);
    assert!(d.el("Cells/Cell#40").is_none());
    d.ui.navigate(Nav::Next);
    idle(&mut d, 1);
    d.step(InputFrame {
        keys: vec![UiKey::End],
        ..Default::default()
    });
    idle(&mut d, 3);
    assert_eq!(focus(&d, 0), "Cells/Cell#40");
    assert!(d.get("Cells/Cell#40").y < 180.0);
    d.step(InputFrame {
        keys: vec![UiKey::Home],
        ..Default::default()
    });
    assert_eq!(focus(&d, 0), "Cells/Cell#0");
}

/// A grid below a scroll view's fold builds the rows of its own box, not a
/// screenful: out of view it has no hit, so it reads last frame's layout for
/// its height and width (it guessed the viewport's: a dozen columns of rows,
/// laid out four across, three times its height).
#[test]
fn a_grid_out_of_view_builds_only_its_own_rows() {
    let bag: Vec<Value> = (0..120).map(|n| json!({ "id": n.to_string() })).collect();
    let d = mount(
        r#"<meta name="component" content="t">
<column id="Root">
<scroll id="Page">
  <div id="Spacer"></div>
  <column id="Frame">
    <cells id="Bag" cell-width="64">{#each $bag as c (c.id)}<div id="Item"></div>{/each}</cells>
  </column>
</scroll>
</column>
<style>
#Page { width: 400px; height: 300px; }
#Spacer { height: 900px; }
#Frame { width: 300px; height: 150px; }
#Bag { width: 100%; height: 100%; }
#Item { height: 64px; }
</style>"#,
        &[("bag", json!(bag))],
    );
    // Out of view the cells are not shown, but every one built is a node.
    let built = (0..d.ui.tree.nodes().len())
        .filter(|&i| d.ui.tree.in_tree(i))
        .map(|i| d.ui.tree.path_of(i))
        .filter(|p| p.contains("/Bag/Cell#") && !p.contains("/Item"))
        .count();
    assert!(built > 0, "the grid is built");
    // 300 px holds four 64 px cells across; 150 px shows two or three rows,
    // plus one past each edge: at most five rows of four.
    assert!(built <= 20, "built {built} cells");
}

/// Small controls' reach (docs/ui.md §2.1): on a touch screen a control's
/// box grows to the 44 px touch target, and where two reaches overlap the
/// nearest box wins; a box around them (a panel behind) never takes a
/// press from a reach, a sibling's own box does; `data-hit-slop` reaches
/// on any pointer, the touch minimum only on a coarse one.
#[test]
fn a_small_controls_reach_grows_to_the_touch_target_and_the_nearest_box_wins() {
    const BOXES: &str = r#"<meta name="component" content="t">
<div id="Root">
  <div id="Panel" interactive></div>
  <div id="A" interactive></div>
  <div id="B" interactive></div>
  <div id="C" interactive data-hit-slop="8"></div>
</div>
<style>
#Panel, #A, #B, #C { position: fixed; }
#Panel { left: 40px; top: 40px; width: 400px; height: 300px; }
#A, #B, #C { top: 100px; width: 20px; height: 20px; }
#A { left: 100px; }
#B { left: 126px; }
#C { left: 300px; }
</style>"#;
    let tap = |touch: bool, x: f32, y: f32| {
        let mut vp = Viewport::new(800.0, 600.0);
        vp.coarse_pointer = touch;
        let mut d = mount_at(BOXES, &[], vp);
        clicks(&mut d);
        let p = Vec2::new(x, y);
        d.step(press(p));
        d.step(release(p));
        clicks(&mut d).first().cloned().unwrap_or_default()
    };
    // Between A (100–120) and B (126–146): the nearer box.
    assert_eq!(tap(true, 124.0, 110.0), "B");
    assert_eq!(tap(true, 121.0, 110.0), "A");
    // 10 px over A: in its 44 px reach on touch (the panel behind does not take it), the panel's with a mouse.
    assert_eq!(tap(true, 110.0, 90.0), "A");
    assert_eq!(tap(false, 110.0, 90.0), "Panel");
    // Past the 44 px reach: the panel.
    assert_eq!(tap(true, 110.0, 86.0), "Panel");
    // C's hitSlop reaches with a mouse too.
    assert_eq!(tap(false, 310.0, 94.0), "C");
}
