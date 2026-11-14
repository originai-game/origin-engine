use super::*;

/// Two scrollers: a page (vertical) holding a horizontal strip, and a
/// board scrolling both ways. Tiles are plain (non-interactive) so a press
/// on them holds the scroller.
fn scrollers() -> String {
    let cards: String = (0..10)
        .map(|k| format!(r#"<div id="Card{k}" class="card"></div>"#))
        .collect();
    let lines: String = (0..10)
        .map(|k| format!(r#"<div id="Line{k}" class="line"></div>"#))
        .collect();
    let rows: String = (0..10)
        .map(|r| {
            let tiles: String = (0..10)
                .map(|c| format!(r#"<div id="Tile{}" class="tile"></div>"#, r * 10 + c))
                .collect();
            format!(r#"<row id="Row{r}">{tiles}</row>"#)
        })
        .collect();
    format!(
        r#"<meta name="component" content="t">
<row id="Root">
  <scroll id="Page">
    <scroll id="Strip"><row id="Cards">{cards}</row></scroll>
    {lines}
  </scroll>
  <scroll id="Board">{rows}</scroll>
</row>
<style>
#Root {{ gap: 20px; }}
#Page {{ width: 220px; height: 200px; gap: 10px; }}
#Strip {{ width: 100%; height: 60px; overflow-x: auto; overflow-y: hidden; }}
#Cards {{ gap: 10px; }}
#Board {{ width: 200px; height: 200px; overflow: auto; }}
.card, .line, .tile {{ background: rgb(80, 80, 90); flex-shrink: 0; }}
.card {{ width: 80px; height: 50px; }}
.line {{ width: 200px; height: 40px; }}
.tile {{ width: 50px; height: 50px; }}
</style>"#
    )
}

fn wheel(at: Vec2, d: Vec2) -> InputFrame {
    InputFrame {
        wheel: d,
        ..InputFrame::hover(at.x, at.y)
    }
}

/// Where an element is now, or None when it is scrolled out of view.
fn pos(d: &Doc, path: &str) -> Option<Vec2> {
    d.el(path).map(|e| Vec2::new(e.x, e.y))
}

/// Scrolling both ways: the wheel moves a board on both axes and stops at
/// its content's ends; a wheel the inner strip cannot take (it scrolls
/// sideways only) scrolls the page around it; a drag moves both axes and,
/// released while moving, keeps going and slows to rest (a press stops it);
/// a press on a scrollbar grabs it; the offsets replay after a restore.
/// Ways it could fail: one axis only, no clamp at the end, a nested
/// container swallowing what it cannot use, a fling that never stops or
/// does not decay, a scrollbar that cannot be grabbed, or offsets outside
/// the UI state.
#[test]
fn scroll_views_scroll_both_ways_hand_off_fling_and_grab() {
    let mut d = mount(&scrollers(), &[]);
    let board = center(&d, "Board");
    let t0 = pos(&d, "Board/Row0/Tile0").expect("the first tile shows");
    // Both axes by the wheel.
    // (The probe reports what shows: a tile well inside the view is whole.)
    let t22 = pos(&d, "Board/Row2/Tile22").expect("a middle tile");
    d.step(wheel(board, Vec2::new(30.0, 40.0)));
    idle(&mut d, 1);
    let t = pos(&d, "Board/Row2/Tile22").expect("still in view");
    assert_eq!(
        t,
        t22 - Vec2::new(30.0, 40.0),
        "the wheel moved the board on both axes"
    );
    // Past the end: the content is 500×500 in a 200×200 view, so 300 each way.
    d.step(wheel(board, Vec2::new(5000.0, 5000.0)));
    idle(&mut d, 1);
    let last = pos(&d, "Board/Row9/Tile99").expect("the last tile shows");
    let view = d.get("Board");
    assert!(
        (last.x + 50.0 - (view.x + view.width)).abs() < 0.5
            && (last.y + 50.0 - (view.y + view.height)).abs() < 0.5,
        "it stops with the last tile at the view's corner: {last:?} in {view:?}"
    );
    // Handoff: a vertical wheel over the strip (horizontal only) scrolls the page.
    let strip = center(&d, "Strip");
    let line0 = pos(&d, "Page/Line0").expect("a line");
    d.step(wheel(strip, Vec2::new(0.0, 25.0)));
    idle(&mut d, 1);
    assert_eq!(
        pos(&d, "Page/Line0").expect("a line").y,
        line0.y - 25.0,
        "what the strip cannot take scrolls the page"
    );
    let card0 = pos(&d, "Strip/Cards/Card1").expect("a card");
    let strip_now = center(&d, "Strip");
    d.step(wheel(strip_now, Vec2::new(45.0, 0.0)));
    idle(&mut d, 1);
    assert_eq!(
        pos(&d, "Strip/Cards/Card1").expect("a card").x,
        card0.x - 45.0,
        "a sideways wheel scrolls the strip"
    );
    // A drag on the board back towards the start, released while moving.
    d.step(wheel(board, Vec2::new(-5000.0, -5000.0)));
    idle(&mut d, 1);
    let start = center(&d, "Board");
    let mut p = start;
    d.step(press(p));
    for _ in 0..4 {
        p -= Vec2::new(12.0, 9.0);
        d.step(InputFrame {
            pointer: p,
            pointer_inside: true,
            down: true,
            ..InputFrame::default()
        });
    }
    let dragged = pos(&d, "Board/Row0/Tile0");
    d.step(release(p));
    // The horizontal scrollbar's position is the board's horizontal offset.
    let bar_x = |d: &Doc| -> f32 {
        let r = d.get("Board");
        d.ui.frame()
            .items
            .iter()
            .find_map(|i| match i.cmd.filled() {
                Some((rect, _))
                    if rect.y > r.y + r.height - 12.0
                        && rect.width > 20.0
                        && rect.height < 8.0
                        && rect.x >= r.x - 0.5 =>
                {
                    Some(rect.x)
                }
                _ => None,
            })
            .unwrap_or(f32::NAN)
    };
    let mut xs = Vec::new();
    for _ in 0..120 {
        d.step(InputFrame::hover(p.x, p.y));
        xs.push(bar_x(&d));
    }
    assert!(dragged.is_some(), "the drag scrolled the board");
    let steps: Vec<f32> = xs.windows(2).map(|w| w[0] - w[1]).collect();
    let steps: Vec<f32> = steps.iter().map(|d| -d).collect();
    assert!(
        steps[0] > 0.3,
        "released while moving, it keeps going: {xs:?}"
    );
    assert!(
        steps[0] > steps[20] && steps[20] >= 0.0,
        "and slows: {steps:?}"
    );
    assert_eq!(xs[118], xs[119], "and comes to rest");
    // Back to the start.
    d.step(wheel(board, Vec2::new(-5000.0, -5000.0)));
    let saved = d.ui.save_state();
    // The vertical scrollbar: a press near the bottom of the board's right edge grabs it.
    let r = d.get("Board");
    let grab = Vec2::new(r.x + r.width - 3.0, r.y + r.height - 6.0);
    d.step(press(grab));
    idle(&mut d, 1);
    let y = pos(&d, "Board/Row0/Tile0").map_or(f32::NEG_INFINITY, |p| p.y);
    assert!(
        d.el("Board/Row0/Tile0").is_none() || y < r.y - 100.0,
        "pressing the track near its end moved the board far down ({y})"
    );
    // A restore replays the offsets.
    d.ui.restore_state(&saved).expect("restore");
    idle(&mut d, 1);
    assert_eq!(
        pos(&d, "Board/Row0/Tile0"),
        Some(t0),
        "the offsets are UI state"
    );
}

fn gestures() -> String {
    let rows: String = (0..20)
        .map(|k| format!(r#"<button id="Row{k}">Row {k}</button>"#))
        .collect();
    format!(
        r#"<meta name="component" content="t">
<column id="Screen">
  <row id="Top"><scroll id="List">{rows}</scroll></row>
  <column id="Pictures"><img id="Wide" alt=""/><img id="Tall" alt=""/></column>
</column>
<style>
#List {{ width: 200px; height: 200px; }}
#Pictures {{ width: 400px; }}
#Wide {{ width: 100%; aspect-ratio: 2; }}
#Tall {{ height: 90px; aspect-ratio: 0.5; }}
</style>"#
    )
}

/// Two fingers (og.Pointers): moved together over a list of buttons they
/// scroll it (one finger would press a row) and the row the first finger
/// landed on is not clicked. An image with an aspect ratio is as tall as
/// its width over it, or as wide as its height times it.
///
/// Ways it could fail: the two touches fight the press (the row clicks,
/// the list does not move), or the aspect is ignored.
#[test]
fn two_fingers_scroll_a_list_without_clicking_its_row() {
    let mut d = mount(&gestures(), &[]);
    // The aspect ratios.
    let wide = d.get("Pictures/Wide");
    assert_eq!(
        (wide.width, wide.height),
        (400.0, 200.0),
        "grow 400 wide at 2:1"
    );
    let tall = d.get("Pictures/Tall");
    assert_eq!((tall.width, tall.height), (45.0, 90.0), "90 tall at 1:2");
    clicks(&mut d);
    // Two-finger scroll over the list, starting on a row.
    let row3 = center(&d, "Top/List/Row3");
    let y3 = pos(&d, "Top/List/Row3").expect("row 3").y;
    let mut f = Fingers::new();
    d.step(f.at(&[(0, row3)]));
    d.step(f.at(&[(0, row3), (1, row3 + Vec2::new(40.0, 0.0))]));
    for k in 1..=3 {
        let up = Vec2::new(0.0, -20.0 * k as f32);
        d.step(f.at(&[(0, row3 + up), (1, row3 + Vec2::new(40.0, 0.0) + up)]));
    }
    // Held still a tick (lifted, they would fling it on).
    let up = Vec2::new(0.0, -60.0);
    d.step(f.at(&[(0, row3 + up), (1, row3 + Vec2::new(40.0, 0.0) + up)]));
    assert_eq!(
        pos(&d, "Top/List/Row3").expect("row 3").y,
        y3 - 60.0,
        "the list followed the two fingers up by 60"
    );
    d.step(f.lift(1));
    d.step(f.lift(0));
    idle(&mut d, 1);
    assert_eq!(
        clicks(&mut d),
        Vec::<String>::new(),
        "the row the first finger landed on is not clicked"
    );
}
