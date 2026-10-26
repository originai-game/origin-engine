//! The UI inside a world, driven only through og-probe's command channel:
//! the UI system reads `og.Input` in the tick, the probe resolves
//! `touch_move_to` / `move_to_text` / `dump_ui` against the live tree.

use crate::support;

use og_ecs::{World, WorldDesc};
use og_probe::{Probe, UiElement};
use og_ui::*;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use support::*;

struct Rig {
    world: World,
    probe: Probe,
    ui: SharedUi,
    sample: Arc<Mutex<Sample>>,
}

impl Rig {
    fn new() -> Rig {
        let mut world = World::new(&WorldDesc {
            tick_rate: 60,
            seed: 1,
        })
        .expect("world");
        // The probe's default sink is og-input's layer: og.Input as the UI reads it.
        let mut probe = Probe::new(&mut world).expect("probe");
        let ui = SharedUi::new(ui_with_font(test_font_metrics()));
        let sample = Arc::new(Mutex::new(Sample::default()));
        let state = sample.clone();
        let (desc, run) = ui.system("ui", viewport, move |ui, _| {
            sample_screen(ui, &mut state.lock().expect("sample"));
            Ok(())
        });
        world.add_system(desc, run).expect("ui system");
        probe.set_ui(ui.probe());
        let mut rig = Rig {
            world,
            probe,
            ui,
            sample,
        };
        rig.ticks(1);
        rig
    }

    fn ticks(&mut self, n: u32) {
        self.probe.frame(&mut self.world, n).expect("ticks");
    }

    /// Schedules a script from now and runs ticks until it has finished.
    fn run(&mut self, script: &str) -> Vec<String> {
        let report = self
            .probe
            .schedule(&self.world, script)
            .unwrap_or_else(|e| panic!("{script}: {e:?}"));
        let now = self.world.time().tick;
        self.ticks((report.until_tick - now) as u32 + 2);
        self.probe
            .take_log()
            .into_iter()
            .filter(|l| l.warning)
            .map(|l| l.text)
            .collect()
    }

    fn element(&self, path: &str) -> UiElement {
        self.ui
            .lock()
            .frame()
            .element(path)
            .cloned()
            .unwrap_or_else(|| panic!("no element {path}"))
    }

    fn elements(&self) -> Vec<UiElement> {
        self.ui.lock().frame().elements.clone()
    }

    fn pointer(&self) -> (f32, f32) {
        let p = self
            .world
            .resource::<og_input::Input>()
            .expect("og.Input")
            .pointer;
        (p.x, p.y)
    }

    fn sample(&self) -> Sample {
        self.sample.lock().expect("sample").clone()
    }
}

#[test]
fn a_button_clicks_only_on_a_press_and_a_release_inside_it() {
    let mut rig = Rig::new();
    let warnings = rig.run("0:touch_move_to:Start;0.05:touch_start;0.1:touch_end");
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(rig.sample().clicks, ["Start"]);

    // Pressed inside, released outside: no click.
    rig.run("0:touch_move_to:Options;0.05:touch_start;0.1:touch_move:790,490;0.15:touch_end");
    // Pressed outside, released inside: no click.
    rig.run("0:touch_move:790,490;0.05:touch_start;0.1:touch_move_to:Options;0.15:touch_end");
    // Held without a release: no click yet; the release then clicks.
    rig.run("0:touch_move_to:Quit;0.05:touch_start");
    assert_eq!(rig.sample().clicks, ["Start"]);
    rig.run("0:touch_end");
    assert_eq!(rig.sample().clicks, ["Start", "Quit"]);

    // A path picks the Buy of one row among twelve.
    rig.run("0:touch_move_to:Row#1/Buy;0.05:touch_start;0.1:touch_end");
    assert_eq!(rig.sample().clicks, ["Start", "Quit", "Row#1/Buy"]);
}

#[test]
fn dump_ui_lists_every_visible_element_with_bounds_and_text() {
    let mut rig = Rig::new();
    let warnings = rig.run("0:dump_ui:dump-ui.json");
    assert!(warnings.is_empty(), "{warnings:?}");
    let outputs = rig.probe.take_outputs();
    let out = outputs
        .iter()
        .find(|o| o.action == "dump_ui")
        .expect("a dump_ui output");
    let bytes = out.bytes();
    std::fs::write(out_dir().join("dump-ui.json"), &bytes).expect("write dump");
    let dump: Value = serde_json::from_slice(&bytes).expect("dump is JSON");
    assert_eq!(dump["kind"], "ui");
    // The surface the CSS-pixel bounds are on, so a frame of another size maps onto them.
    let vp = viewport();
    assert_eq!(
        dump["surface"],
        serde_json::json!({ "width": vp.width, "height": vp.height, "dpr": vp.dpr })
    );
    let listed = dump["elements"].as_array().expect("elements");
    // The store's list cuts its last row: that row and what it holds say so.
    let clipped: Vec<&str> = listed
        .iter()
        .filter(|e| e["clipped"] == true)
        .map(|e| e["id"].as_str().unwrap_or_default())
        .collect();
    assert!(clipped.contains(&"Row#6"), "{clipped:?}");
    assert!(
        listed.iter().any(|e| e["kind"] == "button") && listed.iter().all(|e| e["kind"] != ""),
        "every element names its kind"
    );
    let live = rig.elements();
    assert_eq!(listed.len(), live.len());
    assert!(listed.len() > 40, "{} elements", listed.len());

    let by_id = |path: &str| rig.element(path);
    for (e, json) in live.iter().zip(listed) {
        assert_eq!(json["id"], e.id.as_str());
        assert_eq!(json["text"], e.text.as_str());
        assert!(e.width > 0.0 && e.height > 0.0, "{e:?}");
        assert!(e.x >= 0.0 && e.y >= 0.0, "{e:?}");
        assert!(
            e.x + e.width <= vp.width && e.y + e.height <= vp.height,
            "{e:?}"
        );
        if !e.parent.is_empty() {
            assert!(
                live.iter().any(|p| p.id == e.parent),
                "{} names a parent '{}' that is not listed",
                e.id,
                e.parent
            );
        }
    }
    // What was built is what is listed, with its text.
    for (path, text) in [
        ("Menu/Title", "Origin"),
        ("Start", "Start"),
        ("Music", "Music"),
        ("Search", "Search items"),
        ("Row#0/Name", "Item 0"),
        ("Header/Engine", "原点引擎"),
        ("Tags/Tag#7", "Map"),
    ] {
        assert_eq!(by_id(path).text, text, "{path}");
    }
    for path in ["Volume", "Loading", "List", "Screen", "Store/Header"] {
        by_id(path);
    }
    // Rows scrolled out of the list are not on screen, so not listed.
    assert!(rig.ui.lock().frame().element("Row#11").is_none());
}

#[test]
fn move_to_text_resolves_exact_substring_and_nth_matches() {
    let mut rig = Rig::new();
    let center = |e: UiElement| e.center();

    // Exact text.
    assert!(rig.run("0:move_to_text:=Shield").is_empty());
    assert_eq!(rig.pointer(), center(rig.element("Tag#1")));
    // A substring: "Sh" is only in "Shield"; "=Sh" is no element's whole text.
    assert!(rig.run("0:move_to_text:Sh").is_empty());
    assert_eq!(rig.pointer(), center(rig.element("Tag#1")));
    let missed = rig.run("0:move_to_text:=Sh;0.05:touch_start;0.1:touch_end");
    assert!(
        missed.iter().any(|w| w.contains("target not found")),
        "{missed:?}"
    );
    assert!(
        rig.sample().clicks.is_empty(),
        "a missed target clicks nothing"
    );

    // The Nth match counts topmost (last drawn) first; negative from the end.
    let buys: Vec<UiElement> = rig
        .elements()
        .into_iter()
        .filter(|e| e.text == "Buy")
        .collect();
    assert!(buys.len() >= 6, "{} Buy buttons visible", buys.len());
    // Elements are listed in draw order: the list's rows top to bottom.
    assert_eq!(buys[0], rig.element("Row#0/Buy"));
    assert_eq!(
        buys[buys.len() - 1],
        rig.element(&format!("Row#{}/Buy", buys.len() - 1))
    );
    assert!(rig.run("0:move_to_text:Buy").is_empty());
    assert_eq!(rig.pointer(), buys[buys.len() - 1].center());
    assert!(rig.run("0:move_to_text:Buy@2").is_empty());
    assert_eq!(rig.pointer(), buys[buys.len() - 3].center());
    assert!(rig.run("0:move_to_text:Buy@-1").is_empty());
    assert_eq!(rig.pointer(), buys[0].center());
    let past = rig.run(&format!("0:move_to_text:Buy@{}", buys.len()));
    assert!(
        past.iter().any(|w| w.contains("target not found")),
        "{past:?}"
    );

    // And the pointer it placed clicks what it found.
    rig.run("0:move_to_text:=Item 1;0.05:touch_start;0.1:touch_end");
    assert_eq!(rig.pointer(), center(rig.element("Row#1/Name")));
    rig.run("0:move_to_text:Buy@-2;0.05:touch_start;0.1:touch_end");
    assert_eq!(rig.sample().clicks, ["Row#1/Buy"]);
}

#[test]
fn a_scroll_container_scrolls_by_wheel_and_clips_its_content() {
    let mut rig = Rig::new();
    let list = rig.element("List");
    let row0 = rig.element("Row#0");
    let row1 = rig.element("Row#1");
    assert!(rig.ui.lock().frame().element("Row#11").is_none());

    rig.run("0:touch_move_to:List;0.05:scroll:0,30");
    let moved = rig.element("Row#1");
    assert!(
        (moved.y - (row1.y - 30.0)).abs() < 0.7,
        "Row#1 moved from {} to {}",
        row1.y,
        moved.y
    );
    // Row#0 is now cut by the list's top edge: listed with its visible part.
    let cut = rig.element("Row#0");
    assert!(cut.y >= list.y && cut.height < row0.height, "{cut:?}");
    for e in rig.elements() {
        if e.id.starts_with("Row#") || e.parent.starts_with("Row#") {
            assert!(
                e.y >= list.y - 0.01 && e.y + e.height <= list.y + list.height + 0.01,
                "{e:?} escapes {list:?}"
            );
        }
    }

    // A long scroll stops at the end: the last row's bottom meets the list's.
    rig.run("0:scroll:0,100000");
    let last = rig.element("Row#11");
    assert!(
        (last.y + last.height - (list.y + list.height)).abs() < 0.7,
        "{last:?} vs {list:?}"
    );
    assert!(rig.ui.lock().frame().element("Row#0").is_none());
    // Wheel elsewhere does not scroll the list.
    rig.run("0:touch_move_to:Start;0.05:scroll:0,-500");
    assert!(rig.ui.lock().frame().element("Row#11").is_some());
}

#[test]
fn the_text_field_toggle_and_slider_edit_game_state() {
    let mut rig = Rig::new();
    let w = rig.run(
        "0:touch_move_to:Search;0.05:touch_start;0.1:touch_end;0.2:type:pots;0.5:key_down:Backspace;0.55:key_up:Backspace",
    );
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(rig.sample().search, "pot");
    assert_eq!(rig.element("Search").text, "pot");
    rig.run("0:key_down:Enter;0.05:key_up:Enter");
    assert_eq!(rig.sample().submitted, ["pot"]);
    // Enter left the field: typing goes nowhere now.
    rig.run("0:type:x");
    assert_eq!(rig.sample().search, "pot");

    rig.run("0:touch_move_to:Music;0.05:touch_start;0.1:touch_end");
    assert!(!rig.sample().music);

    let v = rig.element("Volume");
    let (y, right) = (v.y + v.height / 2.0, v.x + v.width - 1.0);
    rig.run(&format!(
        "0:touch_move_to:Volume;0.05:touch_start;0.1:touch_move:{right},{y};0.15:touch_end"
    ));
    assert_eq!(rig.sample().volume, 1.0);
    rig.run(&format!(
        "0:touch_move:{},{y};0.05:touch_start;0.1:touch_end",
        v.x + 1.0
    ));
    assert_eq!(rig.sample().volume, 0.0);
}

#[test]
fn restoring_the_world_and_the_ui_state_replays_the_same_frames() {
    let mut rig = Rig::new();
    rig.run("0:touch_move_to:List;0.05:scroll:0,40;0.1:touch_move_to:Search;0.15:touch_start");
    let snap = rig.probe.snapshot(&rig.world);
    let id = snap["id"].as_str().expect("snapshot id").to_owned();
    let ui_state = rig.ui.lock().save_state();
    let sample = rig.sample();

    let script = "0:touch_end;0.1:type:ab;0.3:move_to_text:Buy@3;0.35:touch_start;0.4:touch_end;0.5:scroll:0,25";
    rig.run(script);
    let first = (rig.elements(), rig.sample());

    assert!(rig.probe.restore(&mut rig.world, &id));
    rig.ui.lock().restore_state(&ui_state).expect("ui state");
    *rig.sample.lock().expect("sample") = sample;
    rig.run(script);
    assert_eq!((rig.elements(), rig.sample()), first);
    assert_eq!(first.1.search, "ab");
    assert_eq!(first.1.clicks.len(), 1);

    assert!(
        rig.ui
            .lock()
            .restore_state(&ui_state[..ui_state.len() - 1])
            .is_err()
    );
}
