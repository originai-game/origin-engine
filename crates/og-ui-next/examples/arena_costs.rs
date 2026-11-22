//! The new core's cost on a HUD-sized tree (First Light's shape: a clock
//! panel, a four-step quest list and a three-slot hotbar, 58 elements):
//! built and laid out from scratch, an unchanged frame, and one text
//! changed. Medians of N runs (default 2000), with the allocations each
//! made.
//!
//!     cargo run --release -p og-ui-next --example arena_costs [N]

use og_ui_next::arena::{Arena, Id};
use rune_taffy::CompactStyle;
use std::time::Instant;
use taffy::FlexDirection;

#[global_allocator]
static HEAP: og_core::alloc::Counting<std::alloc::System> =
    og_core::alloc::Counting(std::alloc::System);

fn allocations() -> usize {
    og_core::alloc::heap().map_or(0, |h| h.allocations)
}

fn measure(
    _: og_ui_next::arena::Id,
    text: &str,
    run: &og_ui_next::arena::Run,
    _: og_ui_next::arena::Avail,
) -> (f32, f32) {
    let size = run.size;
    (text.chars().count() as f32 * size * 0.5, size * 1.25)
}

fn style(dir: FlexDirection, gap: f32, pad: f32) -> CompactStyle {
    let mut s = CompactStyle::default();
    s.set_flex_direction(dir);
    s.set_gap(gap, gap);
    s.set_padding_all(pad);
    s
}

/// The HUD; answers the arena and the clock's text leaf.
fn build() -> (Arena, Id) {
    let mut a = Arena::new();
    let col = style(FlexDirection::Column, 2.0, 8.0);
    let row = style(FlexDirection::Row, 4.0, 0.0);
    let clock = a.element(0, col.clone());
    let now = a.element(clock, row.clone());
    a.element(now, CompactStyle::default());
    let time = a.text(now, "06:54", 28.0);
    a.text(clock, "Summer · Dawn", 16.0);
    a.text(clock, "Clear", 13.0);
    let tasks = a.element(0, col.clone());
    a.text(tasks, "Main quest", 12.0);
    let head = a.element(tasks, row.clone());
    a.text(head, "Morning rounds", 18.0);
    let steps = a.element(tasks, col.clone());
    for t in [
        "Watch the sun come up over the lake",
        "See the weather turn",
        "Follow the oak through a season",
        "Stay until the village lights its lamps",
    ] {
        let li = a.element(steps, row.clone());
        a.element(li, CompactStyle::default());
        a.text(li, t, 14.0);
    }
    let bar = a.element(0, row.clone());
    for (name, key) in [
        ("Weather: Clear", "1"),
        ("Season: Summer", "2"),
        ("Time: Dawn", "3"),
    ] {
        let item = a.element(bar, col.clone());
        let slot = a.element(item, col.clone());
        let face = a.element(slot, CompactStyle::default());
        let art = a.element(face, CompactStyle::default());
        a.element(art, CompactStyle::default());
        a.element(face, row.clone());
        a.element(face, row.clone());
        let label = a.element(item, CompactStyle::default());
        a.text(label, key, 12.0);
        let tip = a.element(item, CompactStyle::default());
        a.text(tip, name, 12.0);
    }
    (a, time)
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(2000);
    let (mut full, mut clean, mut one) = (Vec::new(), Vec::new(), Vec::new());
    let (mut full_a, mut clean_a, mut one_a, mut elements) = (0, 0, 0, 0);
    for i in 0..n {
        let a0 = allocations();
        let t = Instant::now();
        let (mut a, time) = build();
        a.layout(1280.0, 720.0, &mut measure);
        full.push(t.elapsed().as_secs_f64() * 1e3);
        full_a = allocations() - a0;
        elements = a.len();
        let a0 = allocations();
        let t = Instant::now();
        a.layout(1280.0, 720.0, &mut measure);
        clean.push(t.elapsed().as_secs_f64() * 1e3);
        clean_a = allocations() - a0;
        let a0 = allocations();
        let t = Instant::now();
        a.set_text(time, if i % 2 == 0 { "06:55" } else { "06:56" });
        a.layout(1280.0, 720.0, &mut measure);
        one.push(t.elapsed().as_secs_f64() * 1e3);
        one_a = allocations() - a0;
    }
    println!(
        "{{\"elements\": {elements}, \"runs\": {n}, \"fullMs\": {:.4}, \"fullAllocations\": {full_a}, \"unchangedMs\": {:.5}, \"unchangedAllocations\": {clean_a}, \"oneValueMs\": {:.4}, \"oneValueAllocations\": {one_a}}}",
        median(full),
        median(clean),
        median(one)
    );
}
