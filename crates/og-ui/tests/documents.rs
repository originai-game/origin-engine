//! Documents (docs/ui.md §10), bindings (§9), the layer stack and UI
//! actions (§8), through og-ui's public API: a shop screen parsed and
//! validated, mounted over a HUD with a viewmodel, its bindings formatted,
//! its controls writing back, Back and focus memory, a probe edit saved
//! back into the text, and the stack carried by a snapshot.
//!
//! Ways it could fail: a binding read once and never again (the gold
//! label stays at its first value), a list item's write landing on the
//! item variable instead of the list entry, Back reaching the HUD under a
//! menu, focus lost when the menu over a screen pops, a write-back that
//! rewrites lines it did not touch, a snapshot that drops the stack.

use crate::support;

use og_ui::*;
use serde_json::{Value, json};
use support::*;

pub const SHOP: &str = r#"type screen
version 1
---
name "shop"
layer menu
pause
under dim
focus "Footer/Buy"
props {
  gold number "The player's gold."
  items list "What the shop sells."
  selected number "The selected item."
  sound bool "Purchase sounds."
  volume number "Volume, 0 to 1."
}
parts frame
actions {
  confirm "Buy"
  back "Leave"
}
timeline bought {
  track "" scale {
    key 0 1
    key 120 1.08 out
    key 260 1 out
  }
}
fixture rich "{\"gold\": 1250000, \"items\": [{\"name\": \"Sword\", \"price\": 1200, \"owned\": false}, {\"name\": \"Shield\", \"price\": 800, \"owned\": true}], \"selected\": 0, \"sound\": true, \"volume\": 0.5}"
panel Shop {
  width 520
  padding 20
  gap 12
  part frame
  row Header {
    cross center
    gap 8
    label Title {
      text "Shop"
      step xl
    }
    spacer
    label Gold {
      text bind gold compact
    }
  }
  column Items {
    gap 4
    for item in items {
      row Item {
        gap 8
        label Name {
          text bind item.name
        }
        label Price {
          text "{item.price, grouped} gold"
        }
        toggle Owned {
          text "Owned"
          checked bind item.owned
        }
        state selection bind "$index == selected" {
          off {
            opacity 0.7
          }
          on {
            opacity 1
          }
          transition any any spring 260 0.2
        }
      }
    }
  }
  slot Preview {
    height 40
  }
  slider Volume {
    value bind volume
  }
  row Footer {
    gap 8
    button Leave {
      text "Leave"
      on click pop
    }
    button Buy {
      text "Buy"
      variant primary
      disabled bind "gold < items.0.price"
      on click emit buy
      on click play bought
    }
  }
}
"#;

const HUD: &str = r#"type screen
version 1
---
name "hud"
layer hud
props {
  gold number "Gold."
}
row Bar {
  padding 8
  button Menu {
    text "Menu"
    on click emit menu
  }
  label Gold {
    text bind gold grouped
  }
}
"#;

fn ui() -> Ui {
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(UiDocument::parse("ui/shop.ogt", SHOP).expect("shop parses"));
    ui.define_document(UiDocument::parse("ui/hud.ogt", HUD).expect("hud parses"));
    ui
}

fn tick(ui: &mut Ui, input: InputFrame) {
    ui.begin(input, viewport());
    ui.end();
}

fn el<'a>(ui: &'a Ui, path: &str) -> &'a UiElement {
    ui.frame()
        .elements
        .iter()
        .find(|e| e.path == path)
        .unwrap_or_else(|| {
            panic!(
                "no element '{path}': {:?}",
                ui.frame()
                    .elements
                    .iter()
                    .map(|e| &e.path)
                    .collect::<Vec<_>>()
            )
        })
}

fn click(ui: &mut Ui, path: &str) {
    let (x, y) = el(ui, path).center();
    let p = og_core::glam::Vec2::new(x, y);
    let at = |pressed: bool, down: bool, released: bool| InputFrame {
        pointer: p,
        pointer_inside: true,
        pressed,
        down,
        released,
        ..InputFrame::default()
    };
    tick(ui, at(false, false, false));
    tick(ui, at(true, true, false));
    tick(ui, at(false, false, true));
}

fn key(code: &'static str) -> InputFrame {
    let ui_key = UiKey::ALL.into_iter().find(|k| k.code() == code);
    InputFrame {
        keys: ui_key.into_iter().collect(),
        keys_held: vec![code],
        keys_pressed: vec![code],
        ..InputFrame::default()
    }
}

#[test]
fn a_document_reports_every_error_with_its_line() {
    let bad = r#"type screen
version 1
---
name "bad"
props {
  gold number "Gold."
}
column Root {
  label A {
    txt "x"
    on click play missing
  }
  label A {
    text bind wallet.gold
  }
  button B {
    step xl
  }
}
"#;
    let errors = UiDocument::parse("ui/bad.ogt", bad).expect_err("refused");
    let has = |line: usize, what: &str| {
        assert!(
            errors
                .iter()
                .any(|e| e.contains(&format!("line {line}:")) && e.contains(what)),
            "no error at line {line} with '{what}': {errors:#?}"
        )
    };
    has(10, "'txt' is not a prop");
    has(11, "the timeline 'missing' is not declared");
    has(13, "the id 'A' repeats");
    has(14, "'wallet.gold' reads 'wallet'");
    has(17, "'step' is not a prop of button");
}

#[test]
fn a_screen_binds_its_viewmodel_and_formats_it() {
    let mut ui = ui();
    let vm = UiDocument::parse("x", SHOP)
        .unwrap()
        .fixture("rich")
        .cloned()
        .unwrap();
    ui.push_screen("shop", vm).unwrap();
    tick(&mut ui, InputFrame::default());
    tick(&mut ui, InputFrame::default());
    assert_eq!(el(&ui, "shop/Shop/Header/Gold").text, "1.25M");
    assert_eq!(el(&ui, "shop/Shop/Items/Item#1/Price").text, "800 gold");
    assert_eq!(el(&ui, "shop/Shop/Items/Item#0/Name").text, "Sword");
    // Bindings are read every tick: the vm changes, the label follows.
    let mut vm = ui.screen_vm("shop").unwrap().clone();
    vm["gold"] = json!(999);
    vm["items"][0]["price"] = json!(1234567);
    ui.set_screen_vm("shop", vm);
    tick(&mut ui, InputFrame::default());
    assert_eq!(el(&ui, "shop/Shop/Header/Gold").text, "999");
    assert_eq!(
        el(&ui, "shop/Shop/Items/Item#0/Price").text,
        "1,234,567 gold"
    );
    // `gold < items.0.price` now: Buy is disabled.
    assert!(el(&ui, "shop/Shop/Footer/Buy").disabled);
    // The selection group follows its binding per item.
    let b = ui
        .doc_built()
        .iter()
        .find(|b| b.path == "shop/Shop/Items/Item#1")
        .unwrap();
    assert_eq!(b.states, vec![("selection".to_string(), "off".to_string())]);
}

#[test]
fn a_list_items_control_writes_into_its_list_entry() {
    let mut ui = ui();
    let vm = UiDocument::parse("x", SHOP)
        .unwrap()
        .fixture("rich")
        .cloned()
        .unwrap();
    ui.push_screen("shop", vm).unwrap();
    tick(&mut ui, InputFrame::default());
    click(&mut ui, "shop/Shop/Items/Item#0/Owned");
    let vm = ui.screen_vm("shop").unwrap();
    assert_eq!(
        vm["items"][0]["owned"],
        json!(true),
        "the first item's entry took the write"
    );
    assert_eq!(
        vm["items"][1]["owned"],
        json!(true),
        "the second one kept its value"
    );
    assert!(
        vm.get("item").is_none(),
        "nothing was written at the item variable's name"
    );
    let events = ui.take_doc_events();
    assert!(
        events
            .iter()
            .any(|e| e.kind == "set" && e.name == "items.0.owned" && e.value == json!(true)),
        "{events:?}"
    );
}

#[test]
fn document_tabs_bind_their_labels_and_write_selection() {
    let mut ui = ui();
    let doc = UiDocument::parse(
        "ui/tabs.ogt",
        r#"type screen
version 1
---
name "tabs"
layer menu
props {
  labels list "Tab labels."
  current number "Selected index."
}
tabs Categories {
  text "Categories"
  items bind labels
  selected bind current
  width 400
  fill true
  on changed emit category
}
"#,
    )
    .unwrap();
    ui.define_document(doc);
    ui.push_screen(
        "tabs",
        json!({"labels": ["Gear", "Skills", "Quests"], "current": 0}),
    )
    .unwrap();
    tick(&mut ui, InputFrame::default());
    assert_eq!(el(&ui, "tabs/Categories/Tab#2").text, "Quests");
    click(&mut ui, "tabs/Categories/Tab#2");
    assert_eq!(ui.screen_vm("tabs").unwrap()["current"], 2);
    assert!(ui.take_doc_events().iter().any(|e| e.name == "category"));
    ui.set_screen_vm(
        "tabs",
        json!({"labels": ["Inventory", "Journal"], "current": 1}),
    );
    tick(&mut ui, InputFrame::default());
    assert_eq!(el(&ui, "tabs/Categories/Tab#1").text, "Journal");
    assert!(
        !ui.frame()
            .elements
            .iter()
            .any(|e| e.path == "tabs/Categories/Tab#2")
    );
}

/// Tabs with `values` keep the selected tab's name in `current` (a screen's
/// mode, which its `when`s compare), and `keys` label them from the string
/// table, falling back to the item where the table has no message.
#[test]
fn document_tabs_write_their_value_and_translate_their_keys() {
    struct Host;
    impl og_ui::bind::Env for Host {
        fn translate(&self, key: &str) -> Option<String> {
            (key == "screen.results.graphs").then(|| "Graphen".to_string())
        }
    }
    const MODES: &str = r#"type component
version 1
---
name "modes"
props {
  tab string "The mode shown."
}
tabs Modes {
  items Summary Graphs Board
  keys screen.results.summary screen.results.graphs screen.results.board
  values summary graphs board
  current bind tab
  width 400
}
"#;
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(UiDocument::parse("ui/modes.ogt", MODES).expect("modes parses"));
    let mut vm = json!({ "tab": "board" });
    let frame = |ui: &mut Ui, vm: &mut serde_json::Value, input: InputFrame| {
        ui.begin(input, viewport());
        ui.component("Modes", "modes", vm, &Host);
        ui.end();
    };
    frame(&mut ui, &mut vm, InputFrame::default());
    assert_eq!(el(&ui, "Modes/Tab#1").text, "Graphen");
    assert_eq!(el(&ui, "Modes/Tab#0").text, "Summary");
    assert!(
        el(&ui, "Modes/Tab#2")
            .states
            .iter()
            .any(|s| s == "selected"),
        "the bound name selects its tab"
    );
    let (x, y) = el(&ui, "Modes/Tab#1").center();
    let at = |pressed: bool, down: bool, released: bool| InputFrame {
        pointer: og_core::glam::Vec2::new(x, y),
        pointer_inside: true,
        pressed,
        down,
        released,
        ..InputFrame::default()
    };
    frame(&mut ui, &mut vm, at(false, false, false));
    frame(&mut ui, &mut vm, at(true, true, false));
    frame(&mut ui, &mut vm, at(false, false, true));
    assert_eq!(vm["tab"], json!("graphs"), "the click wrote the tab's name");
}

#[test]
fn a_menu_over_the_hud_takes_input_back_and_focus() {
    let mut ui = ui();
    ui.push_screen("hud", json!({ "gold": 1500 })).unwrap();
    tick(&mut ui, InputFrame::default());
    tick(&mut ui, InputFrame::default());
    assert_eq!(el(&ui, "hud/Bar/Gold").text, "1,500");
    assert!(ui.game_input(), "a HUD leaves input to gameplay");
    // Focus on the HUD's button, then a menu over it.
    tick(&mut ui, key("Tab"));
    tick(&mut ui, InputFrame::default());
    assert_eq!(ui.focus_path(0), "hud/Bar/Menu");
    let vm = UiDocument::parse("x", SHOP)
        .unwrap()
        .fixture("rich")
        .cloned()
        .unwrap();
    ui.push_screen("shop", vm).unwrap();
    tick(&mut ui, InputFrame::default());
    tick(&mut ui, InputFrame::default());
    assert!(!ui.game_input(), "the menu holds input");
    assert!(ui.paused());
    assert_eq!(
        ui.focus_path(0),
        "shop/Shop/Footer/Buy",
        "the declared focus"
    );
    let stack = ui.layer_stack();
    assert!(stack[0].inert && !stack[1].inert, "{stack:?}");
    // Back pops the menu (not the game's Back); focus returns to the HUD's button.
    tick(&mut ui, key("Escape"));
    tick(&mut ui, InputFrame::default());
    tick(&mut ui, InputFrame::default());
    assert_eq!(ui.layer_stack().len(), 1);
    assert!(!ui.back(), "the menu took Back");
    assert_eq!(ui.focus_path(0), "hud/Bar/Menu", "focus memory");
    assert_eq!(ui.action_bar().len(), 0);
}

#[test]
fn dump_ui_names_each_elements_layer_source_bindings_and_states() {
    let mut ui = ui();
    ui.push_screen("hud", json!({ "gold": 1500 })).unwrap();
    let vm = UiDocument::parse("x", SHOP)
        .unwrap()
        .fixture("rich")
        .cloned()
        .unwrap();
    ui.push_screen("shop", vm).unwrap();
    tick(&mut ui, InputFrame::default());
    tick(&mut ui, InputFrame::default());
    ui.set_doc_prop("shop/Shop/Header/Title", "text", "\"Bazaar\"")
        .unwrap();
    tick(&mut ui, InputFrame::default());
    let els = ui.probe_elements();
    let find = |p: &str| els.iter().find(|e| e.path == p).unwrap().to_json();
    // Same id under two layers: each answers with its own layer and line.
    let shop_gold = find("shop/Shop/Header/Gold");
    let hud_gold = find("hud/Bar/Gold");
    assert_eq!(shop_gold["layer"], json!("shop"));
    assert_eq!(hud_gold["layer"], json!("hud"));
    assert_eq!(shop_gold["source"], json!("ui/shop.ogt:42"), "{shop_gold}");
    assert_eq!(hud_gold["source"], json!("ui/hud.ogt:15"), "{hud_gold}");
    assert_eq!(
        shop_gold["bindings"],
        json!([{ "prop": "text", "expr": "bind gold compact", "value": "1.25M" }]),
    );
    assert_eq!(
        find("shop/Shop/Header/Title")["edits"],
        json!({ "text": "\"Bazaar\"" })
    );
    assert_eq!(find("shop/Shop/Preview")["slot"]["kind"], json!("code"));
    // The second item is not selected: its state group says so.
    let second = find("shop/Shop/Items/Item#1");
    assert_eq!(
        second["visualStates"]["selection"],
        json!("off"),
        "{second}"
    );
    let extra = ui.probe_extra();
    let layers = extra["layers"].as_array().unwrap();
    assert_eq!(layers.len(), 2);
    assert_eq!(layers[1]["screen"], json!("shop"));
    assert_eq!(layers[1]["focused"], json!("shop/Shop/Footer/Buy"));
    assert_eq!(
        extra["edits"],
        json!([{ "path": "shop/Shop/Header/Title", "prop": "text", "value": "\"Bazaar\"" }])
    );
}

#[test]
fn a_probe_edit_is_saved_into_its_line_alone() {
    let mut ui = ui();
    ui.push_screen("shop", Value::Null).unwrap();
    tick(&mut ui, InputFrame::default());
    ui.set_doc_prop("shop/Shop/Header/Title", "text", "Bazaar")
        .unwrap();
    ui.set_doc_prop("shop/Shop", "gap", "20").unwrap();
    ui.set_doc_prop("shop/Shop/Header/Title", "color", "token primary")
        .unwrap();
    tick(&mut ui, InputFrame::default());
    assert_eq!(el(&ui, "shop/Shop/Header/Title").text, "Bazaar");
    let built = ui
        .doc_built()
        .iter()
        .find(|b| b.path == "shop/Shop/Header/Title")
        .unwrap();
    assert_eq!(built.edits.len(), 2, "{:?}", built.edits);
    let saved = ui.save_doc_edits().unwrap();
    assert_eq!(saved.len(), 1);
    let (_, source, text) = &saved[0];
    assert_eq!(source, "ui/shop.ogt");
    let old: Vec<&str> = SHOP.lines().collect();
    let new: Vec<&str> = text.lines().collect();
    assert_eq!(
        new.len(),
        old.len() + 1,
        "one line added (color), two replaced"
    );
    let changed: Vec<(usize, &str)> = new
        .iter()
        .enumerate()
        .filter(|(i, l)| old.get(*i) != Some(*l))
        .map(|(i, l)| (i, *l))
        .collect();
    assert_eq!(changed[0].1, "  gap 20", "{changed:#?}");
    assert!(new.contains(&"      text \"Bazaar\""), "{text}");
    assert!(new.contains(&"      color token primary"), "{text}");
    assert!(
        ui.doc_edits().is_empty(),
        "saved edits are the document's now"
    );
    tick(&mut ui, InputFrame::default());
    assert_eq!(el(&ui, "shop/Shop/Header/Title").text, "Bazaar");
}

#[test]
fn a_snapshot_carries_the_stack_and_its_viewmodels() {
    let mut ui = ui();
    ui.push_screen("hud", json!({ "gold": 7 })).unwrap();
    ui.push_screen("shop", Value::Null).unwrap();
    tick(&mut ui, InputFrame::default());
    let bytes = ui.save_state();
    let mut other = self::ui();
    other.restore_state(&bytes).unwrap();
    let names: Vec<String> = other
        .layer_stack()
        .iter()
        .map(|l| l.screen.clone())
        .collect();
    assert_eq!(names, ["hud", "shop"]);
    assert_eq!(other.screen_vm("hud"), Some(&json!({ "gold": 7 })));
}

#[test]
fn a_hold_has_a_toggle_and_glyphs_follow_the_device() {
    let mut ui = ui();
    ui.define_action(ActionDesc {
        name: "sprint".into(),
        keys: vec!["ShiftLeft".into()],
        buttons: vec!["leftStick".into()],
        trigger: ActionTrigger::Hold(500.0),
        toggle: true,
        label: "Sprint".into(),
        action: String::new(),
    });
    let held = |pressed: bool| InputFrame {
        keys_held: vec!["ShiftLeft"],
        keys_pressed: if pressed { vec!["ShiftLeft"] } else { vec![] },
        ..InputFrame::default()
    };
    // Held 0.5 s at 60 Hz: fires once, on the 31st tick.
    let mut fired = Vec::new();
    tick(&mut ui, held(true));
    for k in 0..40 {
        if ui.action("sprint") {
            fired.push(k);
        }
        tick(&mut ui, held(false));
    }
    assert_eq!(fired, [30]);
    assert!(ui.action_held("sprint"));
    tick(&mut ui, InputFrame::default());
    assert!(!ui.action_held("sprint"));
    // A tap is the toggle: on until the next tap.
    tick(&mut ui, held(true));
    tick(&mut ui, InputFrame::default());
    assert!(ui.action_held("sprint"), "a tap turned the toggle on");
    tick(&mut ui, held(true));
    tick(&mut ui, InputFrame::default());
    assert!(!ui.action_held("sprint"), "the next tap turned it off");
    assert_eq!(ui.glyph("confirm").unwrap().text, "Enter");
    ui.set_pad_family(Device::PlayStation);
    let pad = InputFrame {
        gamepad: GamepadFrame {
            pressed: GamepadFrame::A,
            held: GamepadFrame::A,
            ..Default::default()
        },
        ..InputFrame::default()
    };
    tick(&mut ui, pad);
    assert_eq!(ui.device(), Device::PlayStation);
    assert_eq!(ui.glyph("confirm").unwrap().icon, "ps.cross");
    assert_eq!(ui.glyph("back").unwrap().text, "○");
    tick(&mut ui, key("KeyA"));
    assert_eq!(ui.device(), Device::KeyboardMouse);
}

#[test]
fn the_readme_lists_the_schema() {
    let readme =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md")).unwrap();
    assert!(
        readme.contains(&og_ui::doc::schema::markdown()),
        "the README's prop table differs from og_ui::doc::schema::markdown(): paste it in"
    );
}

#[test]
fn a_ui_action_follows_a_gameplay_action_and_its_rebinding() {
    let mut ui = ui();
    ui.define_action(ActionDesc {
        name: "inventory".into(),
        label: "Inventory".into(),
        action: "openBag".into(),
        ..ActionDesc::default()
    });
    let feed = |bindings: &[&str], pressed: bool| {
        vec![og_input::ActionFeed {
            name: "openBag".into(),
            state: og_input::ActionState {
                down: pressed,
                pressed,
                ..Default::default()
            },
            bindings: bindings
                .iter()
                .map(|b| og_input::Binding::control(b))
                .collect(),
        }]
    };
    // Its own keys and buttons are not read: Tab alone does not fire it.
    tick(
        &mut ui,
        InputFrame {
            keys_held: vec!["Tab"],
            keys_pressed: vec!["Tab"],
            ..InputFrame::default()
        }
        .with_actions(feed(&["KeyI", "pad.y"], false)),
    );
    assert!(!ui.action("inventory"));
    tick(
        &mut ui,
        InputFrame::default().with_actions(feed(&["KeyI", "pad.y"], true)),
    );
    assert!(
        ui.action("inventory"),
        "the gameplay action's press fires it"
    );
    assert_eq!(ui.glyph("inventory").unwrap().text, "I");
    // The player rebinds the gameplay action: the prompt follows.
    tick(
        &mut ui,
        InputFrame::default().with_actions(feed(&["KeyB", "pad.y"], false)),
    );
    assert_eq!(ui.glyph("inventory").unwrap().text, "B");
    ui.set_device(Device::PlayStation);
    assert_eq!(ui.glyph("inventory").unwrap().icon, "ps.triangle");
}

#[test]
fn a_bound_texture_handle_reaches_the_host_whole() {
    // Wrong: the bound number stored as f32, so a handle ((1 << 32) | index)
    // rounds to 2³² and every image draws texture 0 (the icon atlas).
    struct Host(std::cell::RefCell<Vec<f64>>);
    impl og_ui::bind::Env for Host {
        fn texture(&self, handle: f64) -> Option<og_render::TextureHandle> {
            self.0.borrow_mut().push(handle);
            None
        }
    }
    const PIC: &str = r#"type component
version 1
---
name "pic"
props {
  art number "A texture handle."
}
image Pic {
  texture bind art
  width 64
  height 64
}
"#;
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(UiDocument::parse("ui/pic.ogt", PIC).expect("pic parses"));
    let handle = ((1u64 << 32) | 5) as f64;
    let host = Host(std::cell::RefCell::new(Vec::new()));
    let mut vm = json!({ "art": handle });
    ui.begin(InputFrame::default(), viewport());
    ui.component("Pic", "pic", &mut vm, &host);
    ui.end();
    assert_eq!(host.0.borrow().as_slice(), [handle]);
}

/// A document scroll with `fade` fades its bottom edge where its rows run
/// on past it, so a cut row reads as more rather than as a clip; one whose
/// rows fit draws no fade.
#[test]
fn a_document_scroll_fades_where_its_content_runs_on() {
    const LIST: &str = r#"type screen
version 1
---
name "list"
layer menu
props {
  rows list "Rows."
}
fixture few "{\"rows\": [1, 2]}"
fixture many "{\"rows\": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]}"
column List {
  width 300
  height 300
  scroll Page {
    width grow
    height 120
    fade 32
    for r in rows {
      label Row {
        text "Row"
      }
    }
  }
}
"#;
    let fades = |fixture: &str| {
        let mut ui = ui_with_font(test_font_metrics());
        let doc = UiDocument::parse("ui/list.ogt", LIST).expect("the list parses");
        let vm = doc.fixture(fixture).cloned().unwrap();
        ui.define_document(doc);
        ui.push_screen("list", vm).unwrap();
        for _ in 0..3 {
            tick(&mut ui, InputFrame::default());
        }
        let page = el(&ui, "list/List/Page").clone();
        ui.frame()
            .items
            .iter()
            .filter(|it| match &it.cmd {
                DrawCmd::Shape { shape, .. } => {
                    matches!(shape.paint, og_render::UiPaint::Gradient(_))
                        && (shape.center.y + shape.half.y - (page.y + page.height)).abs() < 1.0
                }
                _ => false,
            })
            .count()
    };
    assert_eq!(fades("few"), 0, "rows that fit draw no fade");
    assert_eq!(fades("many"), 1, "one fade over the bottom edge");
}

/// `direction` turns a row into a column (and back) from an `at` block: a
/// phone in landscape stacks the actions beside the content instead of
/// under it, where a row of them took the height the content needed.
#[test]
fn an_at_block_turns_a_row_into_a_column() {
    const BAR: &str = r#"type screen
version 1
---
name "bar"
layer menu
row Actions {
  gap 8
  at phoneLandscape {
    direction column
  }
  button A {
    text "Again"
  }
  button B {
    text "Continue"
  }
}
"#;
    let laid = |vp: Viewport| {
        let mut ui = ui_with_font(test_font_metrics());
        ui.define_document(UiDocument::parse("ui/bar.ogt", BAR).expect("bar parses"));
        ui.push_screen("bar", json!({})).unwrap();
        for _ in 0..2 {
            ui.begin(InputFrame::default(), vp);
            ui.end();
        }
        let (a, b) = (
            el(&ui, "bar/Actions/A").clone(),
            el(&ui, "bar/Actions/B").clone(),
        );
        (b.x > a.x + a.width - 0.5, b.y > a.y + a.height - 0.5)
    };
    assert_eq!(
        laid(Viewport::new(1280.0, 720.0)),
        (true, false),
        "a row on the desktop"
    );
    let landscape = Viewport {
        width: 844.0,
        height: 390.0,
        dpr: 3.0,
        coarse_pointer: true,
        ..Viewport::default()
    };
    assert_eq!(
        laid(landscape),
        (false, true),
        "a column on a phone in landscape"
    );
}
