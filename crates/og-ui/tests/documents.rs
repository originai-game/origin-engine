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

fn key(code: &'static str) -> InputFrame {
    let ui_key = UiKey::ALL.into_iter().find(|k| k.code() == code);
    InputFrame {
        keys: ui_key.into_iter().collect(),
        keys_held: vec![code],
        keys_pressed: vec![code],
        ..InputFrame::default()
    }
}

/// The stack is ordered by layer kind, as CommonUI's layer groups are, and
/// the active layer is the top one that takes input, not the topmost: a
/// toast over a menu leaves Back, focus and pop with the menu, a modal
/// stays over a menu pushed after it, and a layer hears when it becomes
/// and stops being the active one.
#[test]
fn the_active_layer_is_the_top_input_taker_of_a_kind_ordered_stack() {
    let mut ui = ui();
    let screen = |name: &str, layer: &str| {
        format!(
            "type screen\nversion 1\n---\nname \"{name}\"\nlayer {layer}\ncolumn Root {{\n  button Ok {{\n    text \"Ok\"\n  }}\n}}\n"
        )
    };
    for (n, l) in [("notice", "toast"), ("confirm", "modal")] {
        let src = screen(n, l);
        ui.define_document(UiDocument::parse(&format!("ui/{n}.ogt"), &src).unwrap());
    }
    let names =
        |ui: &Ui| -> Vec<String> { ui.layer_stack().into_iter().map(|l| l.screen).collect() };
    ui.push_screen("hud", json!({ "gold": 1 })).unwrap();
    ui.push_screen("shop", Value::Null).unwrap();
    tick(&mut ui, InputFrame::default());
    tick(&mut ui, InputFrame::default());
    ui.take_doc_events();
    // A toast over the menu: the menu keeps Back and focus.
    ui.push_screen("notice", Value::Null).unwrap();
    tick(&mut ui, InputFrame::default());
    tick(&mut ui, InputFrame::default());
    assert_eq!(ui.focus_path(0), "shop/Shop/Footer/Buy");
    // A modal pushed now sits under the toast and over the menu.
    ui.push_screen("confirm", Value::Null).unwrap();
    assert_eq!(names(&ui), ["hud", "shop", "confirm", "notice"]);
    tick(&mut ui, InputFrame::default());
    tick(&mut ui, InputFrame::default());
    assert_eq!(ui.focus_path(0), "confirm/Root/Ok", "the modal is active");
    let events = ui.take_doc_events();
    let fired = |kind: &str| -> Vec<String> {
        events
            .iter()
            .filter(|e| e.kind == kind)
            .map(|e| e.mount.clone())
            .collect()
    };
    assert_eq!(fired("deactivate"), ["shop"], "{events:?}");
    assert_eq!(fired("activate"), ["confirm"], "{events:?}");
    // A menu pushed after the modal goes under it.
    let src = screen("options", "menu");
    ui.define_document(UiDocument::parse("ui/options.ogt", &src).unwrap());
    ui.push_screen("options", Value::Null).unwrap();
    assert_eq!(names(&ui), ["hud", "shop", "options", "confirm", "notice"]);
    tick(&mut ui, InputFrame::default());
    tick(&mut ui, InputFrame::default());
    assert_eq!(
        ui.focus_path(0),
        "confirm/Root/Ok",
        "the modal stays active"
    );
    // Back pops the modal, not the toast, then the menu under it; the shop
    // returns with its focus.
    ui.take_doc_events();
    for _ in 0..2 {
        tick(&mut ui, key("Escape"));
        tick(&mut ui, InputFrame::default());
        tick(&mut ui, InputFrame::default());
        assert!(!ui.back(), "a menu layer took Back");
    }
    assert_eq!(names(&ui), ["hud", "shop", "notice"]);
    assert_eq!(ui.focus_path(0), "shop/Shop/Footer/Buy", "focus memory");
    let activated: Vec<String> = ui
        .take_doc_events()
        .into_iter()
        .filter(|e| e.kind == "activate")
        .map(|e| e.mount)
        .collect();
    assert_eq!(activated, ["options", "shop"]);
    // pop_screen pops the active layer too.
    assert_eq!(ui.pop_screen().as_deref(), Some("shop"));
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
