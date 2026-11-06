//! The CPU budgets of docs/ui.md §15.1 for menus, measured per tick on the
//! reference machines in release (the perf lane: `scripts/perf.sh`, or
//! `cargo test -p og-ui --release --test tests perf_ -- --ignored
//! --nocapture`). A tick is what a host pays: `begin`, the build, `end`
//! (layout, motion, the accessibility tree) and recording the commands into
//! an overlay. Each test prints its median and 95th percentile with the
//! frame's element and command counts, so the numbers can be reported
//! whether or not they meet the budget.
//!
//! - `perf_dense_menu_300_elements_40_animating`: the budget's own screen,
//!   built by code: a shop of 48 cards, 40 of them in a spring at every
//!   tick (retargeted before they settle), ≤ 0.5 ms.
//! - `perf_every_library_and_kit_screen`: each screen document of the
//!   library and the kits mounted with its default fixture, ≤ 0.5 ms each;
//!   the fullest is named with its count.
//! - `perf_scaling_inventory_table_and_raid_hud`: a 2,000-row table and a
//!   40-frame raid HUD, ms a tick against the element count; no budget.

use crate::support::*;
use og_render::Overlay;
use og_ui::*;
use std::path::PathBuf;

const BUDGET_MS: f64 = 0.5;
const WARM: u32 = 30;
const TICKS: u32 = 240;

fn vp() -> Viewport {
    Viewport::new(1920.0, 1080.0)
}

fn record(ui: &Ui, seconds: f32) -> usize {
    let mut o = Overlay::new(1920.0, 1080.0, 1.0);
    ui.frame().emit(
        &EmitDesc {
            layer: 1000,
            time: seconds,
        },
        &mut o,
    );
    std::hint::black_box(&o);
    ui.frame().items.len()
}

struct Measured {
    median: f64,
    p95: f64,
    /// The medians of the build (`begin` and the calls), `end` and recording.
    parts: [f64; 3],
    elements: u32,
    commands: usize,
}

/// `TICKS` ticks of `build` after `WARM`, timed from `begin` to the recorded
/// overlay.
fn measure(ui: &mut Ui, mut build: impl FnMut(&mut Ui, u32)) -> Measured {
    let mut times = Vec::new();
    let mut parts: [Vec<f64>; 3] = Default::default();
    let mut commands = 0;
    let ms = |t: std::time::Instant| t.elapsed().as_secs_f64() * 1000.0;
    for tick in 0..WARM + TICKS {
        let start = std::time::Instant::now();
        ui.begin(InputFrame::default(), vp());
        build(ui, tick);
        let built = ms(start);
        let t = std::time::Instant::now();
        ui.end();
        let ended = ms(t);
        let t = std::time::Instant::now();
        commands = record(ui, tick as f32 / 60.0);
        let recorded = ms(t);
        if tick >= WARM {
            times.push(ms(start));
            for (p, v) in parts.iter_mut().zip([built, ended, recorded]) {
                p.push(v);
            }
        }
    }
    let median = |v: &mut Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let [b, e, r] = &mut parts;
    let parts = [median(b), median(e), median(r)];
    times.sort_by(f64::total_cmp);
    Measured {
        median: times[times.len() / 2],
        p95: times[times.len() * 95 / 100],
        parts,
        elements: ui.frame().stats.elements,
        commands,
    }
}

fn release_only() {
    if cfg!(debug_assertions) {
        panic!(
            "perf tests measure release builds: cargo test -p og-ui --release --test tests perf_ -- --ignored"
        );
    }
}

/// One card of the shop: its frame, art, name, price, a rarity badge and a
/// stock bar.
fn card(ui: &mut Ui, k: usize, tick: u32) {
    let animating = k < 40;
    // Retargeted every 12 ticks, before a 300 ms spring settles: always in flight.
    let lifted = animating && (tick / 12 + k as u32) % 2 == 0;
    ui.open_with(
        format!("Card{k}"),
        Element {
            width: Size::Fixed(200.0),
            height: Size::Fixed(150.0),
            padding: Insets::all(8.0),
            gap: 4.0,
            background: Some(Srgba::rgb(30, 34, 44)),
            radius: 6.0,
            interactive: true,
            ..Element::default()
        },
        Props {
            transform: Transform {
                scale_x: if lifted { 1.05 } else { 1.0 },
                scale_y: if lifted { 1.05 } else { 1.0 },
                ..Transform::IDENTITY
            },
            opacity: if lifted { 1.0 } else { 0.9 },
            motion: Some(Transition::Spring(SpringSpec::new(300.0, 0.1))),
            ..Props::default()
        },
    );
    ui.icon("Art", "sword", IconDesc::default());
    ui.label("Name", &format!("Item {k}"), LabelDesc::default());
    let caption = ui.theme().type_ramp.caption;
    ui.label(
        "Price",
        &format!("{} gold", 100 + k * 25),
        LabelDesc {
            step: Some(caption),
            ..LabelDesc::default()
        },
    );
    ui.badge("Rarity", ["Common", "Rare", "Epic"][k % 3], Tone::Primary);
    ui.progress("Stock", (k % 10) as f32 / 10.0, ProgressDesc::default());
    ui.close();
}

#[test]
#[ignore = "perf: run on the reference machines in release"]
fn perf_dense_menu_300_elements_40_animating() {
    release_only();
    let mut ui = ui_with_font(test_font_metrics());
    let m = measure(&mut ui, |ui, tick| {
        ui.panel("Shop", PanelDesc::default(), |ui| {
            ui.row("Header", Element::row(), |ui| {
                ui.label("Title", "Shop", LabelDesc::default());
                for t in ["Weapons", "Armour", "Potions", "Gems"] {
                    ui.button(t, t, ButtonDesc::default());
                }
            });
            ui.open_with(
                "Cards",
                Element {
                    gap: 8.0,
                    ..Element::default()
                },
                Props {
                    layout: LayoutMode::Grid,
                    grid: GridSpec {
                        columns: vec![Track::Fr(1.0); 8],
                        ..GridSpec::default()
                    },
                    ..Props::default()
                },
            );
            for k in 0..49 {
                card(ui, k, tick);
            }
            ui.close();
        });
    });
    let animating = ui
        .frame()
        .motion
        .iter()
        .filter(|d| !d.moving.is_empty())
        .count();
    println!(
        "perf ui dense menu: median {:.3} ms, p95 {:.3} ms a tick (build {:.3}, end {:.3}, record {:.3}); {} elements, {animating} animating, {} commands",
        m.median, m.p95, m.parts[0], m.parts[1], m.parts[2], m.elements, m.commands
    );
    assert!(m.elements >= 300, "{} elements", m.elements);
    assert!(animating >= 40, "{animating} animating");
    assert!(
        m.median <= BUDGET_MS,
        "median {:.3} ms a tick (budget {BUDGET_MS} ms)",
        m.median
    );
}

/// Every screen document of the library and the kits (a kit's `.ogt`
/// beside its `extension.json`), as `(asset id, text)`.
fn screens() -> Vec<(String, String)> {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut dirs = vec![repo.join("packages/engine/src/ui/screens")];
    for kit in std::fs::read_dir(repo.join("packages/engine/extensions")).unwrap() {
        let kit = kit.unwrap().path();
        if kit.join("extension.json").is_file() {
            dirs.push(kit);
        }
    }
    let mut out = Vec::new();
    for d in dirs {
        for f in std::fs::read_dir(&d).unwrap() {
            let p = f.unwrap().path();
            if p.extension().is_some_and(|e| e == "ogt") {
                let text = std::fs::read_to_string(&p).unwrap();
                if text.starts_with("type screen\n") {
                    let stem = p.file_stem().unwrap().to_str().unwrap();
                    out.push((format!("ui/{stem}.ogt"), text));
                }
            }
        }
    }
    out.sort();
    out
}

#[test]
#[ignore = "perf: run on the reference machines in release"]
fn perf_every_library_and_kit_screen() {
    release_only();
    let mut rows = Vec::new();
    for (id, text) in screens() {
        let doc = UiDocument::parse(&id, &text).unwrap_or_else(|e| panic!("{}", e.join("\n")));
        let name = doc.name.clone();
        let vm = doc
            .fixtures
            .iter()
            .find(|(n, _)| n == "default")
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        let mut ui = ui_with_font(test_font_metrics());
        ui.define_document(doc);
        ui.push_screen(&name, vm).unwrap();
        let m = measure(&mut ui, |_, _| {});
        rows.push((id, m));
    }
    rows.sort_by(|a, b| b.1.median.total_cmp(&a.1.median));
    for (id, m) in &rows {
        println!(
            "perf ui screen {id}: median {:.3} ms, p95 {:.3} ms a tick (build {:.3}, end {:.3}, record {:.3}); {} elements, {} commands",
            m.median, m.p95, m.parts[0], m.parts[1], m.parts[2], m.elements, m.commands
        );
    }
    let fullest = rows.iter().max_by_key(|(_, m)| m.elements).unwrap();
    println!(
        "perf ui fullest screen: {} ({} elements, median {:.3} ms)",
        fullest.0, fullest.1.elements, fullest.1.median
    );
    let over: Vec<String> = rows
        .iter()
        .filter(|(_, m)| m.median > BUDGET_MS)
        .map(|(id, m)| format!("{id} {:.3} ms", m.median))
        .collect();
    assert!(over.is_empty(), "over {BUDGET_MS} ms a tick: {over:?}");
}

/// One frame of the raid HUD: the member's class, name, health and mana,
/// its health springing toward a value that moves every tick (a fight).
fn raid_frame(ui: &mut Ui, k: usize, tick: u32) {
    let hit = (tick as usize + k * 7) % 23 == 0;
    ui.open_with(
        format!("Member{k}"),
        Element {
            width: Size::Fixed(120.0),
            height: Size::Fixed(44.0),
            padding: Insets::all(4.0),
            gap: 2.0,
            background: Some(Srgba::rgb(20, 22, 28)),
            radius: 3.0,
            ..Element::column()
        },
        Props {
            opacity: if hit { 0.7 } else { 1.0 },
            motion: Some(Transition::Spring(SpringSpec::new(200.0, 0.1))),
            ..Props::default()
        },
    );
    ui.row("Top", Element::row(), |ui| {
        ui.icon(
            "Class",
            ["shield", "sword", "heart"][k % 3],
            IconDesc::default(),
        );
        ui.label("Name", &format!("Member {k}"), LabelDesc::default());
    });
    let hp = ((tick as usize * 3 + k * 37) % 100) as f32 / 100.0;
    ui.progress("Health", hp, ProgressDesc::default());
    ui.progress(
        "Mana",
        ((k * 13) % 100) as f32 / 100.0,
        ProgressDesc::default(),
    );
    ui.close();
}

/// One row of the inventory table: icon, name, count, weight, value.
fn inventory_row(ui: &mut Ui, k: usize) {
    ui.row(
        format!("Item{k}"),
        Element {
            gap: 8.0,
            ..Element::row()
        },
        |ui| {
            ui.icon("Icon", "gem", IconDesc::default());
            ui.label("Name", &format!("Item {k}"), LabelDesc::default());
            ui.label("Count", &format!("x{}", k % 99 + 1), LabelDesc::default());
            ui.label(
                "Weight",
                &format!("{}.{} kg", k % 20, k % 10),
                LabelDesc::default(),
            );
            ui.label("Value", &format!("{} gold", k * 5), LabelDesc::default());
        },
    );
}

/// Well past today's screens: an inventory table of `rows` rows in a
/// scroller (every row built, as an immediate-mode table is) beside a
/// 40-frame raid HUD in a fight, from no rows to 2,000. Prints ms a tick
/// against the elements built, how the cost scales; no budget is asserted.
#[test]
#[ignore = "perf: run on the reference machines in release"]
fn perf_scaling_inventory_table_and_raid_hud() {
    release_only();
    for rows in [0usize, 250, 500, 1000, 2000] {
        let mut ui = ui_with_font(test_font_metrics());
        let m = measure(&mut ui, |ui, tick| {
            ui.row(
                "Screen",
                Element {
                    width: Size::grow(),
                    height: Size::grow(),
                    ..Element::row()
                },
                |ui| {
                    ui.scroll(
                        "Inventory",
                        ScrollDesc {
                            width: Size::Fixed(900.0),
                            height: Size::grow(),
                            gap: 2.0,
                            ..ScrollDesc::default()
                        },
                        |ui| {
                            for k in 0..rows {
                                inventory_row(ui, k);
                            }
                        },
                    );
                    ui.open_with(
                        "Raid",
                        Element {
                            gap: 4.0,
                            ..Element::default()
                        },
                        Props {
                            layout: LayoutMode::Grid,
                            grid: GridSpec {
                                columns: vec![Track::Fr(1.0); 5],
                                ..GridSpec::default()
                            },
                            ..Props::default()
                        },
                    );
                    for k in 0..40 {
                        raid_frame(ui, k, tick);
                    }
                    ui.close();
                },
            );
        });
        let per_k = m.median * 1000.0 / f64::from(m.elements);
        println!(
            "perf ui scaling: {rows} rows + 40-frame raid: {} elements, median {:.3} ms, p95 {:.3} ms a tick (build {:.3}, end {:.3}, record {:.3}), {per_k:.2} µs an element, {} commands",
            m.elements, m.median, m.p95, m.parts[0], m.parts[1], m.parts[2], m.commands
        );
    }
}
