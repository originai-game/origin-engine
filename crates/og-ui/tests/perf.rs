//! The CPU budgets of docs/ui.md §6.3 for menus, measured per tick on the
//! reference machines in release (the perf lane: `scripts/perf.sh`, or
//! `cargo test -p og-ui --release --test tests perf_ -- --ignored
//! --nocapture`). A tick is what a host pays: the game's store writes,
//! `begin`, building the mounted documents, `end` (layout, motion, the
//! accessibility tree) and recording the commands into an overlay. Each test
//! prints its median and 95th percentile with the frame's element and
//! command counts, so the numbers can be reported whether or not they meet
//! the budget. Every surface is a document over named stores, as a game
//! builds it.
//!
//! - `perf_dense_menu_300_elements_40_animating`: the budget's own screen:
//!   a shop of 49 cards, 40 of them in a spring at every tick (retargeted
//!   by a store write before they settle), ≤ 0.5 ms, or the machine's own
//!   budget where it has one (`dense_menu_budget_ms`).
//! - `perf_every_library_and_kit_screen`: each screen document of the
//!   library and the kits mounted with its default fixture, ≤ 0.5 ms each;
//!   the fullest is named with its count.
//! - `perf_scaling_inventory_table_and_raid_hud`: a 2,000-row table and a
//!   40-frame raid HUD, ms a tick against the element count; no budget.
//! - `perf_ui_retained_change_budgets`: a large surface idle, one label
//!   changing, and 1,000-row lists scrolled.
//! - `perf_hud_2000_mass_and_100_anchors_within_a_millisecond`: a HUD, 2,000
//!   mass elements and 100 world anchors, ≤ 1 ms.

use crate::support::*;
use og_core::glam::Vec3;
use og_draw::{Overlay, UiMassInstance, UiMassKind, UiMassSpace, UiMassTemplate};
use og_ui::*;
use serde_json::{Value, json};
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
    /// The medians of the build (the writes, `begin` and the documents),
    /// `end` and recording.
    parts: [f64; 3],
    elements: u32,
    commands: usize,
}

/// `TICKS` ticks after `WARM`, timed from the game's store writes
/// (`write`, before `begin`) to the recorded overlay.
fn measure(ui: &mut Ui, mut write: impl FnMut(&mut Ui, u32)) -> Measured {
    let mut times = Vec::new();
    let mut parts: [Vec<f64>; 3] = Default::default();
    let mut commands = 0;
    let ms = |t: std::time::Instant| t.elapsed().as_secs_f64() * 1000.0;
    for tick in 0..WARM + TICKS {
        let start = std::time::Instant::now();
        write(ui, tick);
        ui.begin(InputFrame::default(), vp());
        ui.build_layers(&bind::NoEnv);
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
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
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

/// A UI with `text` defined and mounted over the test stores `stores`.
fn mounted(text: &str, stores: &[(&str, Value)]) -> Ui {
    mounted_with(text, stores, &[])
}

/// [`mounted`] with the library's control documents `controls` defined
/// (`progress`: `packages/engine/src/ui/controls/progress.svelte`), as a
/// game's load defines those its elements use.
fn mounted_with(text: &str, stores: &[(&str, Value)], controls: &[&str]) -> Ui {
    let mut ui = ui_with_font(test_font_metrics());
    for name in controls {
        let path = engine_dir(&format!("src/ui/controls/{name}.svelte"));
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        let control = ui
            .parse_document(&format!("@ui/controls/{name}.svelte"), &text, Vec::new())
            .unwrap_or_else(|e| panic!("{e:?}"));
        ui.define_document(control);
    }
    let doc = UiDocument::parse("ui/perf.svelte", text).unwrap_or_else(|e| panic!("{e:?}"));
    let name = doc.name.clone();
    ui.define_document(doc);
    mount_stores(&mut ui, source("screen"), &name, stores);
    ui
}

/// The shop: a header, then 49 cards of art, name, price, a rarity badge and
/// a stock bar. A card springs its scale and opacity when it is lifted; the
/// first 40 are lifted on alternate phases, so each phase change retargets
/// them.
const SHOP: &str = r#"<meta name="component" content="shop">
<div id="Shop">
  <div id="Header">
    <h2 id="Title">Shop</h2>
    {#each $tabs as tab (tab.id)}<button id="Tab">{tab.name}</button>{/each}
  </div>
  <div id="Cards">
    {#each $cards as c (c.id)}
      <button id="Card" class:lifted={c.animated && c.parity == $phase}><div id="Body">
        <og-icon id="Art" icon="sword"></og-icon>
        <p id="Name">{c.name}</p>
        <small id="Price">{c.price}</small>
        <span id="Rarity" class="badge">{c.rarity}</span>
        <progress id="Stock" value={c.stock}></progress>
      </div></button>
    {/each}
  </div>
</div>
<style>
#Shop { display: flex; flex-direction: column; gap: 8px; padding: 16px; height: 100%; background: var(--panel); }
#Header { display: flex; gap: 8px; }
#Cards { display: grid; grid-template-columns: repeat(8, 1fr); gap: 8px; flex: 1; min-height: 0; overflow-y: auto; }
#Card {
  width: 140px; height: 150px; padding: 8px;
  background: #1e222c; border-radius: 6px; opacity: 0.9;
  transition: scale 300ms -og-spring(300ms, 0.1), opacity 300ms -og-spring(300ms, 0.1);
}
#Card.lifted { scale: 1.05; opacity: 1; }
#Body { display: flex; flex-direction: column; gap: 4px; }
</style>"#;

#[test]
#[ignore = "perf: run on the reference machines in release"]
fn perf_dense_menu_300_elements_40_animating() {
    release_only();
    let (m, animating) = dense_menu(&[]);
    assert!(m.elements >= 300, "{} elements", m.elements);
    assert!(animating >= 40, "{animating} animating");
    let budget = dense_menu_budget_ms();
    assert!(
        m.median <= budget,
        "median {:.3} ms a tick (budget {budget} ms)",
        m.median
    );
}

/// The dense menu with its 49 stock bars drawn by the progress control
/// document (docs/ui.md §2.4) instead of og-ui's fallback: what a control
/// document costs on the budget's own screen. It reports and holds no
/// budget until the controls have all moved.
#[test]
#[ignore = "perf lane: run in release with --ignored"]
fn perf_dense_menu_with_control_documents() {
    release_only();
    let (m, animating) = dense_menu(&["progress"]);
    assert!(animating >= 40, "{animating} animating");
    assert!(m.elements >= 300 + 2 * 49, "{} elements", m.elements);
}

/// The dense menu's measurement and how many of its elements are in
/// motion, with the control documents `controls` defined.
fn dense_menu(controls: &[&str]) -> (Measured, usize) {
    let tabs: Vec<Value> = ["Weapons", "Armour", "Potions", "Gems"]
        .iter()
        .map(|t| json!({"id": t, "name": t}))
        .collect();
    let cards: Vec<Value> = (0..49)
        .map(|k| {
            json!({
                "id": format!("c{k}"),
                "animated": k < 40,
                "parity": k % 2,
                "name": format!("Item {k}"),
                "price": format!("{} gold", 100 + k * 25),
                "rarity": (["Common", "Rare", "Epic"][k % 3]),
                "stock": (k % 10) as f32 / 10.0,
            })
        })
        .collect();
    let mut ui = mounted_with(
        SHOP,
        &[
            ("tabs", Value::Array(tabs)),
            ("cards", Value::Array(cards)),
            ("phase", json!(0)),
        ],
        controls,
    );
    // Retargeted every 12 ticks, before a 300 ms spring settles: always in flight.
    let m = measure(&mut ui, |ui, tick| {
        if tick % 12 == 0 {
            set_store(ui, "phase", json!((tick / 12) % 2));
        }
    });
    let animating = ui
        .motion_dump()
        .iter()
        .filter(|d| !d.moving.is_empty())
        .count();
    println!(
        "perf ui dense menu{}: median {:.3} ms, p95 {:.3} ms a tick (build {:.3}, end {:.3}, record {:.3}); {} elements, {animating} animating, {} commands",
        if controls.is_empty() {
            String::new()
        } else {
            format!(" with control documents ({})", controls.join(", "))
        },
        m.median,
        m.p95,
        m.parts[0],
        m.parts[1],
        m.parts[2],
        m.elements,
        m.commands
    );
    (m, animating)
}

/// The dense menu's budget on this machine, by its CPU as og-runtime keeps
/// its perf baselines (`cpu_slug`): `BUDGET_MS`, except on the L4, which
/// runs this scene about 1.8 times slower than the Windows reference (a
/// Ryzen 5 5600G: 0.92 against 0.51 ms a tick at 26d6d66df).
fn dense_menu_budget_ms() -> f64 {
    match cpu_slug().as_str() {
        "intel-r-xeon-r-cpu-2-20ghz" => 0.85,
        _ => BUDGET_MS,
    }
}

/// The CPU's model name as a path segment (`intel-r-xeon-r-cpu-2-20ghz`),
/// as og-runtime's perf tests name their baselines; `unknown` where it is
/// not read (Windows).
fn cpu_slug() -> String {
    let brand = if cfg!(target_os = "macos") {
        std::process::Command::new("sysctl")
            .args(["-n", "machdep.cpu.brand_string"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    } else {
        std::fs::read_to_string("/proc/cpuinfo").ok().and_then(|t| {
            t.lines()
                .find(|l| l.starts_with("model name"))
                .and_then(|l| l.split(':').nth(1))
                .map(|s| s.trim().to_string())
        })
    }
    .unwrap_or_else(|| "unknown".into());
    let mut out = String::new();
    for c in brand.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
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
    // The built-in components, for a screen that mounts one (a film's
    // flowchart its NodeGraph).
    let (components, _) = engine_documents(&["src/ui/components"]);
    for (id, text) in screens() {
        let doc = UiDocument::parse(&id, &text).unwrap_or_else(|e| panic!("{}", e.join("\n")));
        // A component is measured where a screen mounts it; only a screen
        // is pushed.
        if doc.kind != og_ui::doc::DocKind::Screen {
            continue;
        }
        let name = doc.name.clone();
        let vm = doc
            .fixtures
            .iter()
            .find(|(n, _)| n == "default")
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        let mut ui = ui_with_font(test_font_metrics());
        for component in &components {
            ui.define_document(component.doc.clone());
        }
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

/// An inventory table in a scroller (every row built, as a table of plain
/// boxes is) beside a 40-frame raid HUD in a fight: each member's health
/// moves every tick and a hit dips its opacity through a spring.
const RAID: &str = r#"<meta name="component" content="raid">
<div id="Screen">
  <div id="Inventory">
    {#each $items as item (item.id)}
      <div id="Item">
        <og-icon id="Icon" icon="gem"></og-icon>
        <p id="Name">{item.name}</p>
        <p id="Count">{item.count}</p>
        <p id="Weight">{item.weight}</p>
        <p id="Value">{item.value}</p>
      </div>
    {/each}
  </div>
  <div id="Raid">
    {#each $members as m (m.id)}
      <div id="Frame" class:hit={m.hit}>
        <div id="Top"><og-icon id="Class" icon={m.class}></og-icon><p id="Member">{m.name}</p></div>
        <progress id="Health" value={m.hp}></progress>
        <progress id="Mana" value={m.mana}></progress>
      </div>
    {/each}
  </div>
</div>
<style>
#Screen { display: flex; width: 100%; height: 100%; }
#Inventory { display: flex; flex-direction: column; gap: 2px; width: 900px; height: 100%; overflow-y: auto; }
#Item { display: flex; gap: 8px; }
#Raid { display: grid; grid-template-columns: repeat(5, 1fr); gap: 4px; }
#Frame {
  display: flex; flex-direction: column; width: 120px; height: 44px; padding: 4px; gap: 2px;
  background: #14161c; border-radius: 3px; opacity: 1;
  transition: opacity 200ms -og-spring(200ms, 0.1);
}
#Frame.hit { opacity: 0.7; }
#Top { display: flex; }
</style>"#;

/// The raid's members at `tick`: health moves every tick, a few are hit.
fn members(tick: u32) -> Vec<Value> {
    (0..40usize)
        .map(|k| {
            json!({
                "id": format!("m{k}"),
                "class": (["shield", "sword", "heart"][k % 3]),
                "name": format!("Member {k}"),
                "hp": ((tick as usize * 3 + k * 37) % 100) as f32 / 100.0,
                "mana": ((k * 13) % 100) as f32 / 100.0,
                "hit": (tick as usize + k * 7) % 23 == 0,
            })
        })
        .collect()
}

/// Well past today's screens: the inventory table from no rows to 2,000
/// beside the raid HUD, its 40 members patched every tick. Prints ms a tick
/// against the elements built, how the cost scales; no budget is asserted.
#[test]
#[ignore = "perf: run on the reference machines in release"]
fn perf_scaling_inventory_table_and_raid_hud() {
    release_only();
    for rows in [0usize, 250, 500, 1000, 2000] {
        let items: Vec<Value> = (0..rows)
            .map(|k| {
                json!({
                    "id": format!("i{k}"),
                    "name": format!("Item {k}"),
                    "count": format!("x{}", k % 99 + 1),
                    "weight": format!("{}.{} kg", k % 20, k % 10),
                    "value": format!("{} gold", k * 5),
                })
            })
            .collect();
        let mut ui = mounted(
            RAID,
            &[
                ("items", Value::Array(items)),
                ("members", Value::Array(members(0))),
            ],
        );
        let m = measure(&mut ui, |ui, tick| {
            let patches: Vec<Value> = members(tick)
                .into_iter()
                .map(|m| {
                    json!({"op": "patch", "source": source("members"), "key": m["id"].clone(),
                        "fields": {"hp": m["hp"].clone(), "hit": m["hit"].clone()}})
                })
                .collect();
            let reply = og_ui::reactive::wire::decode(
                &ui.apply_reactive(&og_ui::reactive::wire::encode(&Value::Array(patches)).unwrap()),
            )
            .unwrap();
            assert!(
                reply.as_array().unwrap().iter().all(|r| r["ok"] == true),
                "{reply}"
            );
        });
        let per_k = m.median * 1000.0 / f64::from(m.elements);
        println!(
            "perf ui scaling: {rows} rows + 40-frame raid: {} elements, median {:.3} ms, p95 {:.3} ms a tick (build {:.3}, end {:.3}, record {:.3}), {per_k:.2} µs an element, {} commands",
            m.elements, m.median, m.p95, m.parts[0], m.parts[1], m.parts[2], m.commands
        );
    }
}

/// A large surface (20 columns of 50 labels) and a 1,000-row list as
/// Svelte documents over named stores, mounted once and driven only by store
/// writes and input, with the tick's executed work. Idle writes nothing; one
/// label patches one row's field each tick; the list scrolls by the wheel.
/// `fling-1000` scrolls that list by a trackpad's momentum, fractional
/// steps that decay, at a 1.5 device pixel ratio. `row-999` scrolls a
/// 1,000-row list to its last row and back with the focus held on row 999
/// by key.
#[test]
#[ignore = "perf: run on the reference machines in release"]
fn perf_ui_retained_change_budgets() {
    use og_ui::reactive::{SourceId, wire};
    use std::collections::BTreeMap;
    release_only();
    let apply = |ui: &mut Ui, commands: serde_json::Value| {
        let reply = wire::decode(&ui.apply_reactive(&wire::encode(&commands).unwrap())).unwrap();
        assert!(
            reply.as_array().unwrap().iter().all(|r| r["ok"] == true),
            "{reply}"
        );
    };
    for scenario in [
        "idle",
        "idle-animated",
        "one-label",
        "scroll-1000",
        "fling-1000",
        "row-999",
    ] {
        let mut ui = ui_with_font(test_font());
        let mut inputs = BTreeMap::new();
        let list = matches!(scenario, "scroll-1000" | "fling-1000" | "row-999");
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
            // row-999 fits the screen (its last row on it).
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
            // Every label animates its opacity: settled, none is visited.
            if scenario == "idle-animated" {
                doc.push_str("<style>text { transition: opacity 200ms; }</style>");
            }
            doc
        };
        ui.define_document(UiDocument::parse("perf.svelte", &text).unwrap());
        ui.mount_document(
            SourceId::new("perf", "screen"),
            "perf",
            inputs,
            Default::default(),
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
                // A trackpad's momentum: fractional steps that decay, again
                // and again, on a 1.5 device pixel ratio.
                "fling-1000" if tick > 0 => 46.3 * 0.94f32.powi((tick % 40) as i32),
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
                Viewport {
                    dpr: if scenario == "fling-1000" { 1.5 } else { 1.0 },
                    ..Viewport::new(2560.0, 1440.0)
                },
            );
            ui.build_retained_documents(&bind::NoEnv).unwrap();
            let built = start.elapsed().as_secs_f64() * 1000.0;
            // The last frame showed row 999: hold the focus on it by key.
            if scenario == "row-999" && ui.element_bounds(&far).is_some() {
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
            "scroll-1000" | "fling-1000" => {
                assert!(max_first > 100, "the measured list must actually scroll")
            }
            "row-999" => {
                assert!(max_first >= 999, "reached row {max_first}");
                assert_eq!(ui.focus_path(0), far, "focus held on row 999 by key");
            }
            "idle" | "idle-animated" => {
                let busy = work.iter().find(|s| {
                    (s.bindings, s.reconciled, s.laid_out, s.painted, s.animated) != (0, 0, 0, 0, 0)
                });
                assert!(busy.is_none(), "an idle tick worked: {busy:?}");
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

/// A HUD of 12 readouts and a menu button over 100 world anchors (name
/// plates around the camera's target that stack, fade behind geometry and
/// scale with distance).
const PLATES: &str = r#"<meta name="component" content="plates">
<div id="Root">
  <div id="Hud">
    {#each $stats as s (s.id)}<p id="Stat">{s.text}</p>{/each}
    <button id="Menu">Menu</button>
  </div>
  {#each $plates as p (p.id)}
    <anchor id="Plate" at={p.at} declutter="stack" occlusion="fade" scale-at="5" priority={p.priority}>
      <p id="Name">Brenna</p>
    </anchor>
  {/each}
</div>
<style>
#Hud { display: flex; flex-direction: column; gap: 4px; padding: 12px; background: var(--panel); }
</style>"#;

/// §15.1: a HUD plus 2,000 mass elements and 100 anchored elements in
/// ≤ 1 ms of CPU a tick (build, lay out, place and record).
#[test]
#[ignore = "perf: run on the reference machines in release"]
fn perf_hud_2000_mass_and_100_anchors_within_a_millisecond() {
    const W: f32 = 1280.0;
    const H: f32 = 720.0;
    release_only();
    let stats: Vec<Value> = (0..12)
        .map(|k| json!({"id": format!("s{k}"), "text": "Health 100"}))
        .collect();
    let plates: Vec<Value> = (0..100)
        .map(|k| {
            let a = k as f32 * 0.37;
            json!({
                "id": format!("p{k}"),
                "at": format!(
                    "{} 1 {}",
                    og_core::math::cos(a) * 4.0,
                    og_core::math::sin(a) * 4.0 - 4.0
                ),
                "priority": k % 3,
            })
        })
        .collect();
    let mut ui = mounted(
        PLATES,
        &[
            ("stats", Value::Array(stats)),
            ("plates", Value::Array(plates)),
        ],
    );
    let h = ui.define_mass(
        "damage",
        UiMassTemplate {
            kind: UiMassKind::Number {
                decimals: 0,
                plus: false,
                compact: false,
            },
            lifetime: 48,
            ..UiMassTemplate::default()
        },
        UiMassSpace::World,
    );
    let view = WorldView::look_at(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::ZERO,
        Vec3::Y,
        50.0,
        W / H,
        0.1,
        100.0,
    );
    let mut times = Vec::new();
    for tick in 0..240u32 {
        let start = std::time::Instant::now();
        ui.begin(InputFrame::default(), Viewport::new(W, H));
        ui.set_world_view(Some(view));
        ui.build_layers(&bind::NoEnv);
        // About 2,000 alive: 42 a tick over a 48-tick life.
        for i in 0..42 {
            ui.spawn_mass(
                h,
                UiMassInstance {
                    position: [(i as f32 - 21.0) * 0.2, 1.0, -((tick % 10) as f32)],
                    value: (tick * 42 + i) as f32,
                    color: [1.0; 4],
                    scale: 1.0,
                    ..Default::default()
                },
            );
        }
        ui.end();
        let mut o = Overlay::new(W, H, 1.0);
        ui.frame().emit(
            &EmitDesc {
                layer: 1000,
                time: 0.0,
            },
            &mut o,
        );
        ui.world_frame().emit(1000, &mut o);
        std::hint::black_box(&o);
        if tick >= 48 {
            times.push(start.elapsed().as_secs_f64() * 1000.0);
        }
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let plates = ui
        .probe_elements()
        .iter()
        .filter(|e| e.kind == "world")
        .count();
    assert_eq!(plates, 100, "every plate placed");
    assert!(ui.mass_count(h) >= 1900, "{}", ui.mass_count(h));
    times.sort_by(f64::total_cmp);
    let median = times[times.len() / 2];
    let p95 = times[times.len() * 95 / 100];
    println!(
        "perf ui hud: median {median:.3} ms, p95 {p95:.3} ms a tick; {} elements",
        ui.frame().stats.elements
    );
    assert!(median <= 1.0, "median {median:.3} ms a tick");
}
