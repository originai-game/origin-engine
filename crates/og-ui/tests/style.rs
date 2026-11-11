//! Retained style metadata and tick-driven widget motion. Counters and
//! hover springs replay after restoration; token aliases reject cycles.
//! Part painting and CSS state transitions are tested in dressing.rs.

use crate::support;

use og_draw::TextureHandle;
use og_ui::*;
use support::*;

fn atlas() -> TextureHandle {
    Handles::default().texture()
}

fn screen(ui: &mut Ui, disabled: bool) {
    ui.panel("Menu", PanelDesc::default(), |ui| {
        ui.label("Title", "Menu", LabelDesc::default());
        ui.button(
            "Go",
            "Go",
            ButtonDesc {
                disabled,
                ..ButtonDesc::default()
            },
        );
    });
}

fn step(ui: &mut Ui, input: InputFrame, disabled: bool) {
    ui.begin(input, Viewport::new(800.0, 600.0));
    screen(ui, disabled);
    ui.end();
}

fn count_step(ui: &mut Ui, value: f64) -> f64 {
    ui.begin(InputFrame::default(), Viewport::new(320.0, 200.0));
    let v = ui.counter(
        "Score",
        value,
        &CounterDesc {
            from: Some(0.0),
            ..CounterDesc::default()
        },
    );
    ui.end();
    v
}

#[test]
fn a_counter_counts_by_the_tick_and_replays_after_a_restore() {
    let mut ui = ui_with_font(test_font_metrics());
    let shown: Vec<f64> = (0..60).map(|_| count_step(&mut ui, 12840.0)).collect();
    assert_eq!(shown[0], 0.0, "it starts at `from`");
    assert!(shown.windows(2).all(|w| w[1] >= w[0]), "it never goes back");
    assert_eq!(shown[54], 12840.0, "it lands at 900 ms (54 ticks)");
    assert!(
        shown[10] > 4000.0 && shown[10] < 12840.0,
        "ease-out: fast first"
    );
    assert_eq!(ui.element("Score").unwrap().text, "12,840");
    // A new value counts on from where it is.
    let mut ui = ui_with_font(test_font_metrics());
    for _ in 0..20 {
        count_step(&mut ui, 1000.0);
    }
    let saved = ui.save_state();
    let a: Vec<f64> = (0..20).map(|_| count_step(&mut ui, 1000.0)).collect();
    ui.restore_state(&saved).unwrap();
    let b: Vec<f64> = (0..20).map(|_| count_step(&mut ui, 1000.0)).collect();
    assert_eq!(a, b, "a restored count replays exactly");
    assert_eq!(format_count(1234567.891, 2), "1,234,567.89");
}

/// The `Go` button's top edge over `n` ticks of `input` under `style`.
fn tops(ui: &mut Ui, input: &InputFrame, n: usize) -> Vec<f32> {
    (0..n)
        .map(|_| {
            step(ui, input.clone(), false);
            ui.element("Go").expect("the button").y
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
        let mut ui = ui_with_font(test_font_metrics());
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
        ui.set_theme(theme);
        for _ in 0..30 {
            step(&mut ui, InputFrame::default(), false);
        }
        let rest = ui.element("Go").unwrap().clone();
        let (x, y) = rest.center();
        let over = InputFrame::hover(x, y);
        let ys = tops(&mut ui, &over, 150);
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
        step(&mut ui, InputFrame::hover(0.0, 0.0), false);
        let saved = ui.save_state();
        let a = tops(&mut ui, &over, 12);
        ui.restore_state(&saved).unwrap();
        let b = tops(&mut ui, &over, 12);
        assert_eq!(a, b, "the spring is UI state");
    }
}
