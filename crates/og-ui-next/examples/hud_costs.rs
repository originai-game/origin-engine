//! First Light's HUD on the new core, headless: the full build (compile,
//! instantiate, first layout and paint), an unchanged frame, and one store
//! changed (the clock), each the median of N runs with its allocations.
//! Text is measured with fixed advances; nothing is drawn.
//!
//!     cargo run --release -p og-ui-next --example hud_costs [N]

use og_ui::UiDocument;
use og_ui_next::look::{Layer, Look};
use og_ui_next::queue;
use og_ui_next::tree::Tree;
use serde_json::json;
use std::rc::Rc;
use std::time::Instant;

#[global_allocator]
static HEAP: og_core::alloc::Counting<std::alloc::System> =
    og_core::alloc::Counting(std::alloc::System);

fn allocations() -> usize {
    og_core::alloc::heap().map_or(0, |h| h.allocations)
}

fn read(p: &str) -> String {
    std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{p}: {e}"))
}

fn library() -> Vec<std::sync::Arc<UiDocument>> {
    let mut docs = vec![
        UiDocument::parse(
            "ui/hud.svelte",
            &read("showcase/first-light/assets/ui/hud.svelte"),
        )
        .unwrap(),
    ];
    for dir in ["components/hud", "components/rpg", "controls"] {
        let root = format!("packages/engine/src/ui/{dir}");
        for f in std::fs::read_dir(&root).unwrap().filter_map(|e| e.ok()) {
            let name = f.file_name().into_string().unwrap();
            if name.ends_with(".svelte")
                && let Ok(d) = UiDocument::parse(
                    &format!("@ui/{dir}/{name}"),
                    &read(&format!("{root}/{name}")),
                )
            {
                docs.push(d);
            }
        }
    }
    docs.into_iter().map(std::sync::Arc::new).collect()
}

fn look() -> Look {
    let mut tokens = String::new();
    for f in [
        "seed", "color", "role", "derived", "contrast", "state", "space", "density", "type",
        "effect", "motion",
    ] {
        tokens += &read(&format!("packages/design/tokens/{f}.css"));
    }
    tokens += og_ui::DEFAULT_CSS;
    let surfaces = read("crates/og-ui-next/default/surfaces.css");
    Look::load(
        &[Layer {
            tokens: &tokens,
            surfaces: &surfaces,
        }],
        "dark",
    )
}

fn slots(name: &str) -> serde_json::Value {
    let slot = |id: &str, icon: &str, name: &str, key: &str| json!({"id": id, "icon": icon, "name": name, "action": id, "hotkey": key, "cooldown": 0, "cooldownMax": 0, "cost": "", "unaffordable": false, "locked": false, "active": false, "count": 0});
    json!([
        slot("weather", "drop", name, "1"),
        slot("season", "leaf", "Season: Summer", "2"),
        slot("time", "clock", "Time: Dawn", "3")
    ])
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(200);
    let lib = library();
    let fonts = og_ui::Fonts {
        body: og_ui::UiFont::unrendered(0.55),
        ..Default::default()
    };
    let icons = og_ui::IconAtlas::default();
    let t = Instant::now();
    let look = Rc::new(look());
    println!(
        "look: {} tokens, loaded in {:.2} ms",
        look.tokens.len(),
        t.elapsed().as_secs_f64() * 1e3
    );
    let steps = json!([
        {"id": "dawn", "text": "Watch the sun come up over the lake", "done": true},
        {"id": "weather", "text": "See the weather turn", "done": false},
        {"id": "season", "text": "Follow the oak through a season", "done": false},
        {"id": "night", "text": "Stay until the village lights its lamps", "done": false},
    ]);
    let (mut build, mut build_alloc, mut idle, mut idle_alloc, mut one, mut one_alloc) =
        (Vec::new(), 0, Vec::new(), 0, Vec::new(), 0);
    let mut shown = (0, 0, 0);
    for run in 0..n {
        let stores: Vec<(&str, u32)> = vec![
            ("shown", queue::signal(true.into())),
            ("clock", queue::signal("06:54".into())),
            ("day", queue::signal("Summer · Dawn".into())),
            ("sky", queue::signal("Clear".into())),
            ("quest", queue::signal("Morning rounds".into())),
            ("steps", queue::signal(steps.clone())),
            ("slots", queue::signal(slots("Weather: Clear"))),
        ];
        let a0 = allocations();
        let t = Instant::now();
        let mut tree = Tree::new(look.clone(), lib.iter().cloned());
        tree.resize(1280.0, 720.0);
        tree.mount("hud", &stores).unwrap();
        let f = tree.frame(&fonts, &icons);
        build.push(t.elapsed().as_secs_f64() * 1e3);
        build_alloc = allocations() - a0;
        tree.frame(&fonts, &icons);
        let a0 = allocations();
        let t = Instant::now();
        let idle_frame = tree.frame(&fonts, &icons);
        idle.push(t.elapsed().as_secs_f64() * 1e3);
        idle_alloc = allocations() - a0;
        assert_eq!(
            idle_frame,
            Default::default(),
            "an unchanged frame does nothing"
        );
        let value: serde_json::Value = format!("06:{:02}", 55 + run % 4).into();
        let a0 = allocations();
        let t = Instant::now();
        queue::set(stores[1].1, value);
        let a1 = allocations();
        let mut changed = tree.commit();
        let a2 = allocations();
        let (l, p) = tree.present(&fonts, &icons);
        (changed.laid_out, changed.painted) = (l, p);
        if run == 0 {
            println!(
                "one change: write {} commit {} present {}",
                a1 - a0,
                a2 - a1,
                allocations() - a2
            );
        }
        one.push(t.elapsed().as_secs_f64() * 1e3);
        one_alloc = allocations() - a0;
        if run == 0 {
            shown = (tree.len(), f.ops, changed.ops);
            println!(
                "first frame {f:?}; one change {changed:?}; {} items",
                tree.items.len()
            );
            for id in ["Clock", "Time", "Tasks", "Hotbar", "AbilityBar"] {
                println!("  #{id}: {:?} {:?}", tree.find(id), tree.text(id));
            }
        }
    }
    println!(
        "{} elements, {} ops to build, {} for the clock",
        shown.0, shown.1, shown.2
    );
    println!(
        "full build      {:.3} ms, {build_alloc} allocations",
        median(build)
    );
    println!(
        "unchanged frame {:.4} ms, {idle_alloc} allocations",
        median(idle)
    );
    println!(
        "one change      {:.4} ms, {one_alloc} allocations",
        median(one)
    );
}
