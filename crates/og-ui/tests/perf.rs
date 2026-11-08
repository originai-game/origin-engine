//! The CPU budgets of docs/ui.md §6.3 for menus, measured per tick on the
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

/// Every core screen document, as `(asset id, text)`.
fn screens() -> Vec<(String, String)> {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut out = Vec::new();
    for f in std::fs::read_dir(repo.join("packages/engine/src/ui/screens")).unwrap() {
        let p = f.unwrap().path();
        if p.extension().is_some_and(|e| e == "svelte") {
            let text = std::fs::read_to_string(&p).unwrap();
            let stem = p.file_stem().unwrap().to_str().unwrap();
            out.push((format!("ui/{stem}.svelte"), text));
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

/// The same large surface with no change, one text mutation, and a real
/// wheel-driven virtual list. Include authoring and overlay recording:
/// skipping layout alone must not look like an idle-frame improvement.
#[test]
#[ignore = "perf: run on the reference machines in release"]
fn perf_ui_change_budgets() {
    release_only();
    for scenario in ["idle", "one-label", "scroll-1000"] {
        let mut scene = og_render::RenderScene::new();
        let mut ui = ui_with_font(add_test_font(&mut scene));
        let mut samples = Vec::new();
        let mut max_first = 0;
        let mut commands = 0;
        for tick in 0..WARM + TICKS {
            let start = std::time::Instant::now();
            ui.begin(
                InputFrame {
                    pointer_inside: true,
                    pointer: og_core::glam::Vec2::new(50.0, 80.0),
                    wheel: og_core::glam::Vec2::new(
                        0.0,
                        if scenario == "scroll-1000" && tick > 0 {
                            24.0
                        } else {
                            0.0
                        },
                    ),
                    ..InputFrame::default()
                },
                Viewport::new(2560.0, 1440.0),
            );
            if scenario == "scroll-1000" {
                let range = ui.open_list(
                    "Rows",
                    ListDesc {
                        count: 1000,
                        row_height: 24.0,
                        width: Size::Fixed(2560.0),
                        height: Some(Size::Fixed(1440.0)),
                        ..ListDesc::default()
                    },
                );
                max_first = max_first.max(range.first);
                for row in range.first..range.last {
                    ui.list_item(row, false);
                    ui.label(
                        "Name",
                        &format!("Inventory item {row}"),
                        LabelDesc::default(),
                    );
                    ui.close();
                }
                ui.close_list();
            } else {
                ui.row("Screen", Element::row(), |ui| {
                    for column in 0..20 {
                        ui.open(
                            format!("Column{column}"),
                            Element {
                                width: Size::Fixed(128.0),
                                height: Size::Fixed(1440.0),
                                ..Element::column()
                            },
                        );
                        for row in 0..50 {
                            let label = if scenario == "one-label" && column == 7 && row == 19 {
                                format!("Value {tick}")
                            } else {
                                format!("Cell {column}:{row}")
                            };
                            ui.label(format!("Label{row}"), &label, LabelDesc::default());
                        }
                        ui.close();
                    }
                });
            }
            ui.end();
            let mut overlay = Overlay::new(2560.0, 1440.0, 1.0);
            ui.frame().emit(&EmitDesc::default(), &mut overlay);
            std::hint::black_box(&overlay);
            commands = ui.frame().items.len();
            if tick >= WARM {
                samples.push(start.elapsed().as_secs_f64() * 1000.0);
            }
        }
        assert!(commands > 0);
        if scenario == "scroll-1000" {
            assert!(max_first > 100, "the measured list must actually scroll");
        } else {
            let text = ui
                .probe_elements()
                .into_iter()
                .find(|e| e.path == "Screen/Column7/Label19")
                .unwrap()
                .text;
            assert_eq!(
                text,
                if scenario == "one-label" {
                    format!("Value {}", WARM + TICKS - 1)
                } else {
                    "Cell 7:19".to_owned()
                }
            );
        }
        samples.sort_by(f64::total_cmp);
        println!(
            "perf ui change {scenario}: median {:.3} ms, p95 {:.3} ms; {} elements, {} laid out, {commands} commands, first row {max_first}",
            samples[samples.len() / 2],
            samples[samples.len() * 95 / 100],
            ui.frame().stats.elements,
            ui.frame().stats.laid_out,
        );
    }
}

/// The authored path of `perf_ui_change_budgets`: the same three surfaces as
/// Svelte documents over named stores, mounted once and driven only by store
/// writes and input, with the tick's executed work. Idle writes nothing; one
/// label patches one row's field each tick; the list scrolls by the wheel.
/// `row-999` scrolls a 1,000-row list to its last row and back with the
/// focus held on row 999 by key.
#[test]
#[ignore = "perf: run on the reference machines in release"]
fn perf_ui_retained_change_budgets() {
    use og_ui::reactive::{SourceId, wire};
    use serde_json::json;
    use std::collections::BTreeMap;
    release_only();
    let apply = |ui: &mut Ui, commands: serde_json::Value| {
        let reply = wire::decode(&ui.apply_reactive(&wire::encode(&commands).unwrap())).unwrap();
        assert!(
            reply.as_array().unwrap().iter().all(|r| r["ok"] == true),
            "{reply}"
        );
    };
    for scenario in ["idle", "one-label", "scroll-1000", "row-999"] {
        let mut scene = og_render::RenderScene::new();
        let mut ui = ui_with_font(add_test_font(&mut scene));
        let mut inputs = BTreeMap::new();
        let list = scenario == "scroll-1000" || scenario == "row-999";
        let text = if list {
            let rows: Vec<_> = (0..1000)
                .map(|row| json!({"id": format!("r{row}"), "label": format!("Inventory item {row}")}))
                .collect();
            let source = SourceId::new("perf", "items");
            apply(
                &mut ui,
                json!([{"op":"collection","source":source,"key":"id","items":rows}]),
            );
            inputs.insert("items".to_owned(), source);
            // row-999 fits the screen (its last row on it); scroll-1000 is
            // perf_ui_change_budgets' list.
            let (width, height) = if scenario == "row-999" {
                (1200, 600)
            } else {
                (2560, 1440)
            };
            format!(
                r#"<meta name="component" content="perf">
<list id="Rows" row-height="24" width="{width}" height="{height}">
{{#each $items as item (item.id)}}<button id="Name" height="24">{{item.label}}</button>{{/each}}
</list>"#
            )
        } else {
            let mut doc =
                String::from("<meta name=\"component\" content=\"perf\">\n<row id=\"Screen\">\n");
            for column in 0..20 {
                let rows: Vec<_> = (0..50)
                    .map(|row| json!({"id": format!("Label{row}"), "text": format!("Cell {column}:{row}")}))
                    .collect();
                let source = SourceId::new("perf", &format!("c{column}"));
                apply(
                    &mut ui,
                    json!([{"op":"collection","source":source,"key":"id","items":rows}]),
                );
                inputs.insert(format!("c{column}"), source);
                doc.push_str(&format!(
                    "<column id=\"Column{column}\" width=\"128\" height=\"1440\">{{#each $c{column} as cell (cell.id)}}<text id=\"Cell{column}\">{{cell.text}}</text>{{/each}}</column>\n"
                ));
            }
            doc.push_str("</row>");
            doc
        };
        ui.define_document(UiDocument::parse("perf.svelte", &text).unwrap());
        ui.mount_document(
            SourceId::new("perf", "screen"),
            "perf",
            inputs,
            &bind::NoEnv,
        )
        .unwrap();
        let mut samples = Vec::new();
        let mut max_first = 0usize;
        let mut commands = 0;
        let mut work = Vec::new();
        let mut parts: [Vec<f64>; 3] = Default::default();
        let far = "perf:screen/Row#r999/Name#r999".to_owned();
        let total = WARM + TICKS;
        for tick in 0..total {
            let start = std::time::Instant::now();
            if scenario == "one-label" && tick > 0 {
                apply(
                    &mut ui,
                    json!([{"op":"patch","source":SourceId::new("perf","c7"),"key":"Label19","fields":{"text":format!("Value {tick}")}}]),
                );
            }
            let wheel = match scenario {
                "scroll-1000" if tick > 0 => 24.0,
                // Down to the end in the first half, back in the second.
                "row-999" if tick > 0 => {
                    if tick < total / 2 {
                        960.0
                    } else {
                        -960.0
                    }
                }
                _ => 0.0,
            };
            ui.begin(
                InputFrame {
                    pointer_inside: true,
                    pointer: og_core::glam::Vec2::new(50.0, 80.0),
                    wheel: og_core::glam::Vec2::new(0.0, wheel),
                    ..InputFrame::default()
                },
                Viewport::new(2560.0, 1440.0),
            );
            ui.build_retained_documents(&bind::NoEnv).unwrap();
            let built = start.elapsed().as_secs_f64() * 1000.0;
            if scenario == "row-999" && ui.probe_elements().iter().any(|e| e.path == far) {
                ui.set_focus(&far);
            }
            let t = std::time::Instant::now();
            ui.end();
            let ended = t.elapsed().as_secs_f64() * 1000.0;
            let t = std::time::Instant::now();
            let mut overlay = Overlay::new(2560.0, 1440.0, 1.0);
            ui.frame().emit(&EmitDesc::default(), &mut overlay);
            std::hint::black_box(&overlay);
            let recorded = t.elapsed().as_secs_f64() * 1000.0;
            commands = ui.frame().items.len();
            if list {
                max_first = max_first.max(
                    ui.probe_elements()
                        .iter()
                        .filter_map(|e| e.path.rsplit_once("/Name#r").map(|(_, r)| r))
                        .filter_map(|r| r.parse::<usize>().ok())
                        .max()
                        .unwrap_or(0),
                );
            }
            if tick >= WARM {
                samples.push(built + ended + recorded);
                for (p, v) in parts.iter_mut().zip([built, ended, recorded]) {
                    p.push(v);
                }
                work.push(ui.frame().stats);
            }
        }
        assert!(commands > 0);
        match scenario {
            "scroll-1000" => assert!(max_first > 100, "the measured list must actually scroll"),
            "row-999" => {
                assert!(max_first >= 999, "reached row {max_first}");
                assert_eq!(ui.focus_path(0), far, "focus held on row 999 by key");
            }
            _ => {}
        }
        samples.sort_by(f64::total_cmp);
        for p in &mut parts {
            p.sort_by(f64::total_cmp);
        }
        let last = work.last().copied().unwrap_or_default();
        let max = |f: fn(&LayoutStats) -> u32| work.iter().map(f).max().unwrap_or(0);
        println!(
            "perf ui retained {scenario}: median {:.3} ms, p95 {:.3} ms (build {:.3}, end {:.3}, record {:.3}); {} elements, {commands} commands, deepest row {max_first}; last tick {last:?}; max laid out {}, max rebuilt {}",
            samples[samples.len() / 2],
            samples[samples.len() * 95 / 100],
            parts[0][parts[0].len() / 2],
            parts[1][parts[1].len() / 2],
            parts[2][parts[2].len() / 2],
            last.elements,
            max(|s| s.laid_out),
            max(|s| s.rebuilt),
        );
    }
}
