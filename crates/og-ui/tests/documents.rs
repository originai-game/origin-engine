//! Documents (docs/ui.md §2.1), bindings (§2.1), the layer stack and UI
//! actions (§2.1), through og-ui's public API: a shop screen parsed and
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

pub const SHOP: &str = r#"<meta name="screen" content="shop" data-layer="menu" data-pause="true" data-under="dim" data-focus="Footer/Buy">
<meta name="prop:gold" content="number" data-doc="The player's gold.">
<meta name="prop:items" content="list" data-doc="What the shop sells.">
<meta name="prop:selected" content="string" data-doc="The selected item.">
<meta name="prop:sound" content="bool" data-doc="Purchase sounds.">
<meta name="prop:volume" content="number" data-doc="Volume, 0 to 1.">
<meta name="action:confirm" content="Buy">
<meta name="action:back" content="Leave">
<meta name="fixture:rich" content='{"gold":1250000,"items":[{"id":"sword","name":"Sword","price":1200,"owned":false},{"id":"shield","name":"Shield","price":800,"owned":true}],"selected":"sword","sound":true,"volume":0.5}'>
<panel id="Shop">
  <row id="Header">
    <text id="Title">Shop</text>
    <spacer />
    <text id="Gold">{formatCompact(gold)}</text>
  </row>
  <column id="Items">
    {#each items as item (item.id)}
      <row id="Item" class:selected={item.id == selected}>
        <text id="Name">{item.name}</text>
        <text id="Price">{formatNumber(item.price)} gold</text>
        <toggle id="Owned" bind:checked={item.owned}>Owned</toggle>
      </row>
    {/each}
  </column>
  <column id="Preview">{@render preview?.()}</column>
  <slider id="Volume" bind:value={volume} />
  <row id="Footer">
    <button id="Leave" onclick={leave}>Leave</button>
    <button id="Buy" variant="primary" disabled={gold < items.0.price} onclick={buy}>Buy</button>
  </row>
</panel>
<style>
#Shop { width: 520px; padding: 20px; gap: 12px; }
#Header { align-items: center; gap: 8px; }
#Title { font: var(--text-xl); }
#Items { gap: 4px; }
#Item { gap: 8px; opacity: 0.7; }
#Item.selected { opacity: 1; }
#Preview { height: 40px; }
#Footer { gap: 8px; }
</style>
"#;

const HUD: &str = r#"<meta name="screen" content="hud" data-layer="hud">
<meta name="prop:gold" content="number" data-doc="Gold.">
<row id="Bar">
  <button id="Menu" onclick={menu}>Menu</button>
  <text id="Gold">{formatNumber(gold)}</text>
</row>
<style>#Bar { padding: 8px; }</style>
"#;

fn ui() -> Ui {
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(UiDocument::parse("ui/shop.svelte", SHOP).expect("shop parses"));
    ui.define_document(UiDocument::parse("ui/hud.svelte", HUD).expect("hud parses"));
    ui
}

fn tick(ui: &mut Ui, input: InputFrame) {
    ui.begin(input, viewport());
    ui.end();
}

#[test]
fn scoped_css_motion_uses_shared_curves_emits_sound_and_restores() {
    let mut ui = Ui::new(UiDesc::default());
    let source = r#"<meta name="screen" content="motion" data-layer="hud">
<panel id="Card"/>
<style>
#Card { width: 40px; height: 20px; animation: reveal 800ms cubic-bezier(0,1,0,1) forwards; }
@keyframes reveal {
  from { translate: 0 0; -og-sound: error; }
  to { translate: 80px 0; }
}
</style>"#;
    ui.define_document(UiDocument::parse("motion.svelte", source).unwrap());
    ui.push_screen("motion", Value::Null).unwrap();
    let mut sounds = 0;
    for _ in 0..7 {
        tick(&mut ui, InputFrame::default());
        sounds += ui
            .events()
            .iter()
            .filter(|e| e.kind == "sound" && e.name == "error")
            .count();
    }
    // x=u³ and y=1-(1-u)³: at 1/8 duration, y=7/8.
    assert!((ui.frame().element("motion/Card").unwrap().x - 70.0).abs() < 0.001);
    assert_eq!(sounds, 1);
    let saved = ui.save_state();
    let continued: Vec<_> = (0..12)
        .map(|_| {
            tick(&mut ui, InputFrame::default());
            ui.frame().element("motion/Card").unwrap().x
        })
        .collect();
    ui.restore_state(&saved).unwrap();
    for expected in continued {
        tick(&mut ui, InputFrame::default());
        assert_eq!(ui.frame().element("motion/Card").unwrap().x, expected);
        assert!(
            ui.events().is_empty(),
            "restoring does not replay the entrance sound"
        );
    }
}

#[test]
fn keyed_document_rows_keep_identity_after_reorder_and_build_the_empty_branch() {
    let document = UiDocument::parse(
        "rows.svelte",
        r#"<meta name="screen" content="rows">
<column id="Root">
  {#each items as item (item.id)}
    <button id="Row" onclick={choose}>{item.name}</button>
  {:else}
    <text id="Empty">No rows</text>
  {/each}
</column>
"#,
    )
    .unwrap();

    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(document);
    let a = json!({"id": "sword", "name": "Sword"});
    let b = json!({"id": "shield", "name": "Shield"});
    ui.push_screen("rows", json!({"items": [a, b]})).unwrap();
    tick(&mut ui, InputFrame::default());
    assert!(
        ui.doc_built()
            .iter()
            .any(|e| e.path == "rows/Root/Row#sword")
    );
    assert!(!ui.doc_built().iter().any(|e| e.path.ends_with("/Empty")));

    ui.set_screen_vm("rows", json!({"items": [b, a]}));
    tick(&mut ui, InputFrame::default());
    let paths: Vec<_> = ui
        .doc_built()
        .iter()
        .map(|e| e.path.as_str())
        .filter(|path| path.contains("Row#"))
        .collect();
    assert_eq!(paths, ["rows/Root/Row#shield", "rows/Root/Row#sword"]);

    ui.set_screen_vm("rows", json!({"items": []}));
    tick(&mut ui, InputFrame::default());
    assert!(ui.doc_built().iter().any(|e| e.path == "rows/Root/Empty"));
    assert!(!ui.doc_built().iter().any(|e| e.path.contains("Row#")));
}

#[test]
fn length_tokens_follow_the_active_style_in_layout_and_states() {
    let source = r#"<meta name="screen" content="tokens" data-layer="hud">
<column id="Root">
  <button id="Child">Token</button>
  <grid id="Tracks" />
</column>
<style>
#Root {
  width: stretch; min-width: var(--size-sm); max-width: var(--size-xl);
  padding: var(--space-xs) 3px var(--space-md) 7px;
  gap: var(--space-sm);
  background: var(--panel);
  border-radius: var(--radius-lg);
  translate: var(--space-xs) var(--space-sm);
}
  #Child {
    height: var(--size-lg);
    border-radius: var(--radius-sm);
  }
    #Child:hover {
      border-radius: var(--radius-xl);
      translate: var(--space-md) 0;
    }
  #Tracks {
    grid-template-columns: var(--size-sm) minmax(var(--size-xs), var(--size-lg));
    grid-template-rows: var(--size-md);
  }
</style>
"#;
    let mut symbolic = ui_with_font(test_font_metrics());
    symbolic.define_document(UiDocument::parse("tokens.ogt", source).unwrap());
    symbolic.push_screen("tokens", Value::Null).unwrap();
    for factor in [1.0, 2.0] {
        let mut theme = Theme {
            reduced_motion: true,
            ..Theme::default()
        };
        for v in &mut theme.tokens.space {
            *v *= factor;
        }
        for v in &mut theme.tokens.size {
            *v *= factor;
        }
        for v in &mut theme.tokens.radius {
            *v *= factor;
        }
        symbolic.set_theme(theme);
        let mut literal = source.to_string();
        for name in [
            "space-xs",
            "space-sm",
            "space-md",
            "size-xs",
            "size-sm",
            "size-md",
            "size-lg",
            "size-xl",
            "radius-sm",
            "radius-lg",
            "radius-xl",
        ] {
            literal = literal.replace(
                &format!("var(--{name})"),
                &format!("{}px", theme.length(name).unwrap()),
            );
        }
        let mut expected = ui_with_font(test_font_metrics());
        expected.set_theme(theme);
        expected.define_document(UiDocument::parse("tokens.ogt", &literal).unwrap());
        expected.push_screen("tokens", Value::Null).unwrap();
        for _ in 0..3 {
            tick(&mut symbolic, InputFrame::default());
            tick(&mut expected, InputFrame::default());
        }
        assert_eq!(
            symbolic.frame().items,
            expected.frame().items,
            "style factor {factor}"
        );
        let button = expected.frame().element("tokens/Root/Child").unwrap();
        let (x, y) = (
            button.x + button.width * 0.5,
            button.y + button.height * 0.5,
        );
        for _ in 0..3 {
            tick(&mut symbolic, InputFrame::hover(x, y));
            tick(&mut expected, InputFrame::hover(x, y));
        }
        assert_eq!(
            symbolic.frame().items,
            expected.frame().items,
            "hover, style factor {factor}"
        );
    }
    let bad = source.replace("--radius-lg", "--radius-missing");
    assert!(
        UiDocument::parse("bad.ogt", &bad)
            .unwrap_err()
            .join("\n")
            .contains("--radius-missing")
    );
    let bad = source.replace("--panel", "--missing-colour");
    let doc = UiDocument::parse("bad.ogt", &bad).unwrap();
    assert!(
        doc.validate_tokens(&[Theme::default()])
            .unwrap_err()
            .join("\n")
            .contains("--missing-colour")
    );
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
            "<meta name=\"screen\" content=\"{name}\" data-layer=\"{layer}\"><column id=\"Root\"><button id=\"Ok\">Ok</button></column>"
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
    assert_eq!(
        shop_gold["source"],
        json!("ui/shop.svelte:14"),
        "{shop_gold}"
    );
    assert_eq!(hud_gold["source"], json!("ui/hud.svelte:5"), "{hud_gold}");
    assert_eq!(
        shop_gold["bindings"],
        json!([{ "prop": "text", "expr": "{formatCompact(gold)}", "value": "1.25M" }]),
    );
    assert_eq!(
        find("shop/Shop/Header/Title")["edits"],
        json!({ "text": "\"Bazaar\"" })
    );
    assert!(
        ui.doc_built()
            .iter()
            .any(|e| e.path == "shop/Shop/Items/Item#shield")
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

/// The CSS dialect is a strict subset: an unknown property or a value without its unit is
/// an error that names the form to write.
#[test]
fn css_outside_the_subset_is_an_error_that_names_the_fix() {
    let err = |body: &str| {
        let t = format!(
            "<meta name=\"screen\" content=\"s\"><column id=\"Root\"/><style>#Root {{ {body} }}</style>"
        );
        UiDocument::parse("ui/s.svelte", &t).unwrap_err().join("\n")
    };
    assert!(
        err("  float: left").contains("display"),
        "{}",
        err("  float: left")
    );
    assert!(err("  margin: 4px").contains("gap"));
    assert!(err("  gap: 12").contains("12px"));
    assert!(err("  font-size: 14px").contains("var(--text-"));
    assert!(err("  padding: 4px !important").contains("!important"));
    // A Rive frame takes one `<file>.riv#<Artboard>` id.
    for bad in [
        "rive(\"ui/chest.riv\", \"Chest\")",
        "rive(\"ui/chest.riv\")",
        "rive(chest)",
    ] {
        let e = err(&format!("  -og-frame: {bad}"));
        assert!(e.contains("takes one quoted id"), "{bad}: {e}");
    }
}

/// Probe edits preserve authored structure and existing stylesheet rules.
#[test]
fn a_prop_edit_writes_a_css_declaration_into_a_svelte_document() {
    const DOC: &str = "<meta name=\"screen\" content=\"s\">\n<column id=\"Root\"><text id=\"Title\">Hi</text></column>\n<style>#Root { width: stretch; max-width: 640px; translate: 0 -4px; }</style>\n";
    let mut doc = UiDocument::parse("ui/s.svelte", DOC).unwrap();
    let root = doc.nodes.iter().position(|n| n.id == "Root").unwrap();
    doc.set_prop(root, "x", "6").unwrap();
    doc.set_prop(root, "gap", "12").unwrap();
    let title = doc.nodes.iter().position(|n| n.id == "Title").unwrap();
    doc.set_prop(title, "color", "\"#ffcc00\"").unwrap();
    let t = &doc.text;
    assert!(t.contains("translate: 6px -4px"), "{t}");
    assert!(t.contains("gap: 12px"), "{t}");
    assert!(t.contains("width: stretch; max-width: 640px"), "{t}");
    assert!(t.contains("color: #ffcc00"), "{t}");
    assert!(t.contains(">Hi</text>"), "{t}");
}

/// A Rive artboard of 100 × 50: a 40 × 20 red rectangle `Lid` centred at
/// x 25 that an animation moves to x 75 over a second, looping.
fn chest() -> std::sync::Arc<og_ui::rive::Artboard> {
    use og_ui::rive::*;
    let node = |name: &str, parent, kind, values| Node {
        name: name.into(),
        parent,
        kind,
        values,
    };
    let key = |seconds, value| Key {
        seconds,
        value,
        color: 0,
        ease: Ease::Linear,
    };
    std::sync::Arc::new(Artboard {
        name: "Chest".into(),
        size: og_core::glam::Vec2::new(100.0, 50.0),
        fit: Fit::None,
        clip: true,
        nodes: vec![
            node(
                "Chest",
                0,
                Kind::Artboard { paints: vec![] },
                Values::default(),
            ),
            node(
                "Lid",
                0,
                Kind::Shape {
                    paths: vec![2],
                    paints: vec![3],
                    hidden: false,
                    blend: og_render::UiBlend::Normal,
                },
                Values {
                    x: 25.0,
                    y: 25.0,
                    ..Values::default()
                },
            ),
            node(
                "box",
                1,
                Kind::Path {
                    outline: Outline::Rect,
                    closed: true,
                },
                Values {
                    width: 40.0,
                    height: 20.0,
                    ..Values::default()
                },
            ),
            node(
                "fill",
                1,
                Kind::Fill {
                    rule: og_render::UiFillRule::NonZero,
                    shader: 4,
                    visible: true,
                },
                Values::default(),
            ),
            node(
                "red",
                3,
                Kind::Solid,
                Values {
                    color: 0xFFFF_0000,
                    ..Values::default()
                },
            ),
        ],
        draws: vec![1],
        animations: vec![Animation {
            name: "open".into(),
            fps: 60,
            duration: 60,
            speed: 1.0,
            looping: Loop::Loop,
            work: None,
            tracks: vec![Track {
                node: 1,
                prop: Prop::X,
                keys: vec![key(0.0, 25.0), key(1.0, 75.0)],
            }],
        }],
        play: Some(0),
        ..Artboard::default()
    })
}

const CHEST_SCREEN: &str = "<meta name=\"screen\" content=\"loot\"><column id=\"Chest\"/><style>#Chest { width: 100px; height: 50px; -og-frame: rive(\"ui/chest.riv#Chest\"); }</style>";

/// A document element whose frame is a Rive artboard plays it on the tick
/// in its box: its shapes draw over its background, its objects are
/// elements with the artboard as their source, and its player is saved
/// with the UI's state and runs on after a restore as if never stopped.
///
/// Ways it could fail: a player advanced per build rather than per tick or
/// restarted each frame (the lid never moves or jumps), a restore that
/// starts it again (the restored run lags the unbroken one), refused bytes
/// silently reset, the write-back losing `rive(…)`.
#[test]
fn a_rive_frame_plays_draws_names_its_objects_and_is_saved() {
    let doc = UiDocument::parse("ui/loot.ogt", CHEST_SCREEN).expect("parses");
    assert_eq!(doc.rive_ids(), ["ui/chest.riv#Chest"]);
    let fresh = |board: Option<std::sync::Arc<og_ui::rive::Artboard>>| {
        let mut ui = ui_with_font(test_font_metrics());
        if let Some(b) = board {
            ui.define_rive("ui/chest.riv#Chest", b);
        }
        ui.define_document(doc.clone());
        ui.push_screen("loot", Value::Null).unwrap();
        ui
    };
    // Where the lid is drawn: its shape's centre, x.
    let lid = |ui: &Ui| -> Vec<f32> {
        ui.frame()
            .items
            .iter()
            .filter_map(|it| match &it.cmd {
                DrawCmd::Shape { shape, .. }
                    if matches!(shape.geometry, og_render::UiGeometry::Path(_)) =>
                {
                    Some(shape.center.x)
                }
                _ => None,
            })
            .collect()
    };
    let mut whole = fresh(Some(chest()));
    tick(&mut whole, InputFrame::default());
    let at = whole
        .probe_elements()
        .into_iter()
        .find(|e| e.path == "loot/Chest")
        .expect("the element");
    let first = lid(&whole);
    assert_eq!(first.len(), 1, "one shape, drawn once");
    for _ in 0..30 {
        tick(&mut whole, InputFrame::default());
    }
    let moved = lid(&whole);
    // 30 ticks of 1/60 s: half way, 25 → 50 in the artboard, at the box's x.
    assert!(
        (moved[0] - first[0] - 25.0).abs() < 0.6,
        "{first:?} → {moved:?}"
    );
    let els = whole.probe_elements();
    let obj = els
        .iter()
        .find(|e| e.kind == "rive")
        .expect("the lid as an element");
    assert_eq!(obj.path, "loot/Chest/Chest/Lid");
    assert_eq!(obj.detail["source"], json!("ui/chest.riv#Chest/Lid"));
    assert!(obj.x >= at.x && obj.x + obj.width <= at.x + at.width + 0.01);

    // Saved, run on, and restored into a UI that knows the artboard: the
    // next ticks match the unbroken run's.
    let saved = whole.save_state();
    for _ in 0..10 {
        tick(&mut whole, InputFrame::default());
    }
    let mut back = fresh(Some(chest()));
    back.restore_state(&saved).expect("restores");
    for _ in 0..10 {
        tick(&mut back, InputFrame::default());
    }
    assert_eq!(
        lid(&back),
        lid(&whole),
        "it runs on from where it was saved"
    );

    // Restored where the artboard is another: an error, nothing drawn, and
    // the bytes kept for the save rather than reset.
    let other = std::sync::Arc::new(og_ui::rive::Artboard {
        animations: Vec::new(),
        play: None,
        ..(*chest()).clone()
    });
    let mut wrong = fresh(Some(other));
    wrong.restore_state(&saved).expect("the UI state reads");
    tick(&mut wrong, InputFrame::default());
    assert!(lid(&wrong).is_empty(), "not drawn");
    assert!(
        wrong
            .frame()
            .warnings
            .iter()
            .any(|w| w.contains("does not fit")),
        "reported"
    );
    let mut again = fresh(Some(chest()));
    again.restore_state(&wrong.save_state()).expect("restores");
    tick(&mut again, InputFrame::default());
    let mut check = fresh(Some(chest()));
    check.restore_state(&saved).expect("restores");
    tick(&mut check, InputFrame::default());
    assert_eq!(lid(&again), lid(&check), "the refused bytes were kept");

    // No artboard of that id: reported, nothing drawn.
    let mut none = fresh(None);
    tick(&mut none, InputFrame::default());
    assert!(lid(&none).is_empty());
    assert!(
        none.frame()
            .warnings
            .iter()
            .any(|w| w.contains("names no Rive artboard"))
    );

    // An edit writes the frame back in the form it is written in.
    let mut edited = doc.clone();
    let chest_node = edited.nodes.iter().position(|n| n.id == "Chest").unwrap();
    edited
        .set_prop(chest_node, "rive", "\"ui/chest.riv#Lid\"")
        .unwrap();
    assert!(
        edited
            .text
            .contains("-og-frame: rive(\"ui/chest.riv#Lid\")"),
        "{}",
        edited.text
    );
    // The scoped override preserves the authored rule, so both assets are dependencies.
    assert_eq!(
        edited.rive_ids(),
        ["ui/chest.riv#Chest", "ui/chest.riv#Lid"]
    );
}
