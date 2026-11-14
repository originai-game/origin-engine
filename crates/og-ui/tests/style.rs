//! Retained style metadata and tick-driven widget motion. Counters and
//! hover springs replay after restoration; token aliases reject cycles.
//! Part painting and CSS state transitions are tested in dressing.rs.

use crate::support;

use og_draw::TextureHandle;
use og_ui::*;
use serde_json::json;
use support::*;

fn atlas() -> TextureHandle {
    Handles::default().texture()
}

const MENU: &str = r#"<meta name="component" content="t">
<panel id="Menu">
  <text id="Title">Menu</text>
  <button id="Go">Go</button>
</panel>"#;

const COUNT: &str = r#"<meta name="component" content="t">
<div><og-counter id="Score" value={$score} from="0" duration="900">0</og-counter></div>"#;

/// The number the counter shows now (its grouping commas dropped).
fn shown(d: &Doc) -> f64 {
    let text = d.get("Score").text.replace(',', "");
    text.parse().unwrap_or_else(|_| panic!("a number: {text}"))
}

/// One tick of the counter; what it shows after it.
fn count_step(d: &mut Doc) -> f64 {
    d.step(InputFrame::default());
    shown(d)
}

#[test]
fn a_counter_counts_by_the_tick_and_replays_after_a_restore() {
    let mut d = Doc::new(COUNT, &[("score", json!(12840.0))]);
    d.viewport = Viewport::new(320.0, 200.0);
    // The mount's first build is the first tick.
    let mut shown_by_tick = vec![shown(&d)];
    shown_by_tick.extend((1..60).map(|_| count_step(&mut d)));
    assert_eq!(shown_by_tick[0], 0.0, "it starts at `from`");
    assert!(
        shown_by_tick.windows(2).all(|w| w[1] >= w[0]),
        "it never goes back"
    );
    assert_eq!(shown_by_tick[54], 12840.0, "it lands at 900 ms (54 ticks)");
    assert!(
        shown_by_tick[10] > 4000.0 && shown_by_tick[10] < 12840.0,
        "ease-out: fast first"
    );
    assert_eq!(d.get("Score").text, "12,840");
    // A new value counts on from where it is.
    let mut d = Doc::new(COUNT, &[("score", json!(1000.0))]);
    d.viewport = Viewport::new(320.0, 200.0);
    for _ in 1..20 {
        count_step(&mut d);
    }
    let saved = d.ui.save_state();
    let a: Vec<f64> = (0..20).map(|_| count_step(&mut d)).collect();
    d.ui.restore_state(&saved).unwrap();
    let b: Vec<f64> = (0..20).map(|_| count_step(&mut d)).collect();
    assert_eq!(a, b, "a restored count replays exactly");
    assert_eq!(format_count(1234567.891, 2), "1,234,567.89");
}

/// The `Go` button's top edge over `n` ticks of `input`.
fn tops(d: &mut Doc, input: &InputFrame, n: usize) -> Vec<f32> {
    (0..n)
        .map(|_| {
            d.step(input.clone());
            d.get("Go").y
        })
        .collect()
}

#[test]
fn the_hover_lift_moves_on_springs_and_replays() {
    // Ways it could fail: the lift jumps to its end in a tick (no motion),
    // eases on a curve that ignores the style's spring (no overshoot at a
    // low damping), never settles exactly, settles somewhere else, or
    // differs after a restore.
    for bounces in [false, true] {
        let mut d = Doc::new(MENU, &[]);
        d.viewport = Viewport::new(800.0, 600.0);
        let mut theme = Theme::default();
        theme.art.atlas = Some(atlas());
        theme.art.hover = Some(style_file::Hover {
            dy: -2.0,
            scale: 1.02,
        });
        theme.art.motion.hover_spring = bounces.then_some(Spring {
            damping: 0.3,
            response_ms: 300.0,
        });
        d.ui.set_theme(theme);
        for _ in 0..30 {
            d.step(InputFrame::default());
        }
        let rest = d.get("Go");
        let (x, y) = rest.center();
        let over = InputFrame::hover(x, y);
        let ys = tops(&mut d, &over, 150);
        let end = ys[149];
        assert!(
            end < rest.y - 1.5,
            "it lifts by the style's 2 px: {} to {end}",
            rest.y
        );
        assert_eq!(ys[148], end, "and settles exactly: {ys:?}");
        assert!(
            ys[1] > end + 0.5 && ys[1] < rest.y,
            "moving over several ticks, not at once: {ys:?}"
        );
        let lowest = ys.iter().copied().fold(f32::INFINITY, f32::min);
        if bounces {
            assert!(
                lowest < end - 0.3,
                "a damping of 0.3 overshoots the lift: {ys:?}"
            );
        } else {
            assert!(lowest >= end - 1e-4, "damping 1 never overshoots: {ys:?}");
        }
        // A restore mid-motion replays the same positions.
        d.step(InputFrame::hover(0.0, 0.0));
        let saved = d.ui.save_state();
        let a = tops(&mut d, &over, 12);
        d.ui.restore_state(&saved).unwrap();
        let b = tops(&mut d, &over, 12);
        assert_eq!(a, b, "the spring is UI state");
    }
}
