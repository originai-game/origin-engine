//! The accessibility tree (`Ui::a11y`, and the probe's `dump_ui` fields):
//! every widget's role, name, value, states and bounds, and the focus
//! order keyboard Tab walks.
//!
//! Ways it could fail, each covered below:
//! - a widget has no role, or the wrong one (a switch read as a button);
//! - a control has no name (an icon button, a stepper's − and +, a field
//!   that shows only its text, a slider beside its row's label), a text
//!   field is named by its placeholder or its text when a label names it
//!   (the placeholder is its description), or a value or state goes stale (a checked
//!   box not `checked`, a slider's number missing, the chosen tab not
//!   `selected`);
//! - a range carries no bounds: a slider's, a stepper's spinbutton's, a
//!   progress's or a meter's `value_min`, `value_max` or `value_now`
//!   missing or wrong, a stepper without bounds given some, a control
//!   that is no range given a value number →
//!   `ranges_carry_their_bounds_and_value`
//! - the probe's elements and the tree disagree;
//! - Tab visits controls in another order than `focus_order`, skips one or
//!   reaches one the tree does not list; an open dialog's tree lets focus
//!   order leave it.

use crate::support;
use og_ui::prelude::*;

use og_ui::*;
use support::*;

#[path = "a11y/form.rs"]
mod form;
use form::*;

#[test]
fn every_widget_has_a_role_a_name_and_its_state() {
    let mut ui = ui_with_font(test_font_metrics());
    let mut f = Form {
        on: true,
        volume: 35.0,
        name: "Ada".into(),
        nick: "Mei".into(),
        tab: 1,
        count: 3.0,
        ..Form::default()
    };
    for _ in 0..2 {
        tick(&mut ui, &mut f, InputFrame::default());
    }
    let tree = ui.a11y().clone();
    let has = |path: &str, role: &str, name: &str| {
        let n = node(&tree, path);
        assert_eq!(
            (n.role.as_str(), n.name.as_str()),
            (role, name),
            "{path}'s role and name"
        );
        n
    };
    has("Form/Title", "heading", "Settings");
    has("Form/Note", "text", "Changes save at once.");
    has("Form/Save", "button", "Save");
    assert!(
        has("Form/Music", "switch", "Music")
            .states
            .contains(&"checked".to_owned())
    );
    assert!(
        !has("Form/Hints", "checkbox", "Show hints")
            .states
            .contains(&"checked".to_owned())
    );
    assert_eq!(has("Form/Volume/Track", "slider", "Volume").value, "35");
    assert_eq!(
        has("Form/Gain/Slider/Track", "slider", "Master").value,
        "0.50",
        "a control without text takes the label before it"
    );
    // A field alone is named by its placeholder; one after its row's label
    // by the label, the placeholder its description, whatever it holds.
    let alone = has("Form/Name", "textbox", "Your name");
    assert_eq!(
        (alone.value.as_str(), alone.description.as_str()),
        ("Ada", "")
    );
    let labelled = has("Form/Player/Field", "textbox", "Lamplighter");
    assert_eq!(
        (labelled.value.as_str(), labelled.description.as_str()),
        ("Mei", "Your name"),
        "the label names it, the placeholder describes it"
    );
    // An area after a field is not labelled by it: its placeholder names it.
    let area = has("Form/Player/Note", "textbox", "A note");
    assert_eq!(area.description, "");
    has("Form/Mode", "radiogroup", "Mode");
    assert!(
        has("Form/Mode/Strip/Track/Item#1", "radio", "Hard")
            .states
            .contains(&"checked".to_owned())
    );
    assert!(
        !node(&tree, "Form/Mode/Strip/Track/Item#0")
            .states
            .contains(&"checked".to_owned())
    );
    let q = has("Form/Quality/Trigger", "combobox", "Quality");
    assert_eq!(q.value, "Low");
    assert!(!q.states.contains(&"expanded".to_owned()));
    assert_eq!(
        has("Form/Count/Frame/Value", "spinbutton", "Count").value,
        "3"
    );
    has("Form/Count/Frame/Dec", "button", "Decrease");
    has("Form/Count/Frame/Inc", "button", "Increase");
    has("Form/Size", "radiogroup", "Size");
    assert!(
        has("Form/Size/Options/Option#0", "radio", "Small")
            .states
            .contains(&"checked".to_owned())
    );
    assert_eq!(has("Form/Load", "progressbar", "Load").value, "40%");
    has("Form/Help", "text", "Read the rules.");
    let link = has("Form/Help/rules", "link", "rules");
    assert_eq!(link.parent, "Form/Help", "a link hangs under its paragraph");
    // Every control keys reach is named; every node has bounds.
    for n in &tree.nodes {
        assert!(n.width > 0.0 && n.height > 0.0, "{} has bounds", n.path);
        if n.states.contains(&"focusable".to_owned()) {
            assert!(!n.name.is_empty(), "{} ({}) is named", n.path, n.role);
        }
    }
    // The probe reads the same: dump_ui's elements carry role, name, value
    // and states.
    let e = ui.element("Form/Volume/Track").expect("the slider element");
    assert_eq!(
        (e.role.as_str(), e.name.as_str(), e.value.as_str()),
        ("slider", "Volume", "35")
    );
    let e = ui.element("Form/Music").expect("the switch element");
    assert!(e.states.contains(&"checked".to_owned()) && e.states.contains(&"focusable".to_owned()));
    // A layout column has no role and is not in the tree.
    assert!(tree.nodes.iter().all(|n| n.path != "Form"));
}

/// A screen reader reads only what changed: an idle tick reads no record
/// and sends nothing, and one bound label reads a handful of records and
/// sends that label, however long the list beside it is. Rows that enter,
/// leave or move in a small group read that group's children and the rows
/// themselves, not the 300 rows beside it. Fails if the bridge falls back
/// to the whole tree on every change, or if a change is missed
/// (`OG_UI_CHECK_BOUNDED=1` also compares every update with the whole
/// tree).
#[test]
fn native_tree_reads_only_the_changed_nodes() {
    use og_ui::reactive::{SourceId, wire};
    use serde_json::{Value, json};
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(
        UiDocument::parse(
            "ui/hud.svelte",
            r#"<meta name="component" content="hud">
<column id="Hud">
  <h2 id="Title">Hud</h2>
  <text id="Score">{$score}</text>
  {#each $rows as row (row.id)}<button id="Row">{row.id}</button>{/each}
  <scroll id="Small" height="160">{#each $small as item (item.id)}<button id="Item">{item.id}</button>{/each}</scroll>
</column>"#,
        )
        .unwrap(),
    );
    let apply = |ui: &mut Ui, command: Value| {
        let reply =
            wire::decode(&ui.apply_reactive(&wire::encode(&json!([command])).unwrap())).unwrap();
        assert_eq!(reply[0]["ok"], true, "{reply}");
    };
    let rows: Vec<Value> = (0..300).map(|i| json!({ "id": format!("r{i}") })).collect();
    let mut inputs = std::collections::BTreeMap::new();
    let (score, list) = (
        SourceId::new("test", "score"),
        SourceId::new("test", "rows"),
    );
    apply(
        &mut ui,
        json!({"op": "signal", "source": score, "initial": "10"}),
    );
    apply(
        &mut ui,
        json!({"op": "collection", "source": list, "key": "id", "items": rows}),
    );
    let small = SourceId::new("test", "small");
    apply(
        &mut ui,
        json!({"op": "collection", "source": small, "key": "id",
               "items": [{"id": "a"}, {"id": "b"}, {"id": "c"}]}),
    );
    inputs.insert("score".to_string(), score);
    inputs.insert("rows".to_string(), list);
    inputs.insert("small".to_string(), small.clone());
    ui.mount_document(
        SourceId::new("test", "hud"),
        "hud",
        inputs,
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    let step = |ui: &mut Ui| {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 900.0));
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
    };
    step(&mut ui);
    step(&mut ui);
    let mut bridge = native_a11y::Bridge::default();
    let first = bridge.update(&ui, [1.0, 1.0]).expect("the whole tree");
    assert!(bridge.read() > 300, "the first update reads the whole tree");
    let heading = first
        .nodes
        .iter()
        .find(|(_, n)| n.label() == Some("Hud"))
        .expect("the heading");
    assert_eq!(
        (heading.1.role(), heading.1.level()),
        (native_a11y::accesskit::Role::Heading, Some(2)),
        "an <h2> reaches the screen reader as a level 2 heading"
    );
    step(&mut ui);
    assert!(bridge.update(&ui, [1.0, 1.0]).is_none());
    assert_eq!(bridge.read(), 0, "an idle tick reads nothing");
    apply(
        &mut ui,
        json!({"op": "set", "source": SourceId::new("test", "score"), "value": "11"}),
    );
    step(&mut ui);
    let update = bridge.update(&ui, [1.0, 1.0]).expect("the label changed");
    let labels: Vec<_> = update.nodes.iter().map(|(_, n)| n.label()).collect();
    assert_eq!(labels, [Some("11")], "only the label is sent");
    assert!(bridge.read() < 8, "read {} records", bridge.read());
    step(&mut ui);
    assert!(bridge.update(&ui, [1.0, 1.0]).is_none());
    // In the small group: an item enters, one moves, one leaves. The
    // group's children are sent again, and only the items that moved with
    // them; the order is the whole tree's.
    let names = |ui: &Ui| -> Vec<String> {
        ui.a11y()
            .nodes
            .iter()
            .filter(|n| n.path.contains("Item"))
            .map(|n| n.name.clone())
            .collect()
    };
    for (command, expect) in [
        (
            json!({"op": "insert", "source": small, "row": {"id": "n"}, "before": "b"}),
            ["a", "n", "b", "c"].as_slice(),
        ),
        (
            json!({"op": "move", "source": small, "key": "c", "before": "a"}),
            ["c", "a", "n", "b"].as_slice(),
        ),
        (
            json!({"op": "remove", "source": small, "key": "a"}),
            ["c", "n", "b"].as_slice(),
        ),
    ] {
        apply(&mut ui, command);
        step(&mut ui);
        let update = bridge.update(&ui, [1.0, 1.0]).expect("the group changed");
        assert!(bridge.read() < 30, "read {} records", bridge.read());
        assert!(update.nodes.len() < 8, "sent {} nodes", update.nodes.len());
        let group = update
            .nodes
            .iter()
            .find(|(_, n)| n.role() == native_a11y::accesskit::Role::Group)
            .expect("the group's children are sent again");
        assert_eq!(group.1.children().len(), expect.len());
        assert_eq!(names(&ui), expect);
    }
}

/// Live regions (`aria-live`, `role="status"`, `role="alert"`): a screen
/// reader hears what changed in one, once, from the tree the bridge last
/// sent. Ways it could fail, each covered below: the first tree, an idle
/// tick or a restore announces; a text outside any region announces; a
/// change announces twice or not at all; an atomic region says only what
/// changed, or a plain one says all of it; an alert row's entry is not
/// assertive, or a removal or a hidden region speaks; the same words
/// twice are heard once (the voice did not change); the page's mirror
/// (`A11yTree::announcements`) hears other than the native bridge; the
/// queue grows without a host taking it. `OG_UI_CHECK_BOUNDED=1` also has
/// every update's whole-tree twin announce the same.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn live_regions_announce_each_change_once() {
    use native_a11y::accesskit::{Live, Role};
    use og_ui::reactive::{SourceId, wire};
    use serde_json::{Value, json};
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(
        UiDocument::parse(
            "ui/news.svelte",
            r#"<meta name="component" content="news">
<column id="News">
  <text id="Status" role="status">{$status}</text>
  <column id="Feed" aria-live="polite">{#each $feed as row (row.id)}<text id="Line">{row.text}</text>{/each}</column>
  <column id="Purse" aria-live="polite" aria-atomic="true"><text id="Label">Gold</text><text id="Gold">{$gold}</text></column>
  <column id="Alerts">{#each $alerts as a (a.id)}<text id="Alert" role="alert">{a.text}</text>{/each}</column>
  <column id="Shut" aria-live="assertive" hidden={$shut}><text id="Secret">{$secret}</text></column>
  <text id="Plain">{$gold} coins</text>
  {#if $unread}<text id="Unread" role="status">{$count}</text>{/if}
  <dialog id="Ask" modal bind:open={$ask} aria-label="Quit?"><button id="Yes">Yes</button></dialog>
  {#each $rows as row (row.id)}<button id="Row">{row.id}</button>{/each}
</column>"#,
        )
        .unwrap(),
    );
    let apply = |ui: &mut Ui, command: Value| {
        let reply =
            wire::decode(&ui.apply_reactive(&wire::encode(&json!([command])).unwrap())).unwrap();
        assert_eq!(reply[0]["ok"], true, "{reply}");
    };
    let source = |name: &str| SourceId::new("test", name);
    let mut inputs = std::collections::BTreeMap::new();
    for (name, initial) in [
        ("status", json!("Ready")),
        ("gold", json!("5")),
        ("secret", json!("s0")),
        ("shut", json!(false)),
        ("ask", json!(false)),
        ("unread", json!(false)),
        ("count", json!("3 unread")),
    ] {
        apply(
            &mut ui,
            json!({"op": "signal", "source": source(name), "initial": initial}),
        );
        inputs.insert(name.to_string(), source(name));
    }
    let rows: Vec<Value> = (0..200).map(|i| json!({ "id": format!("r{i}") })).collect();
    for (name, items) in [
        ("feed", json!([{"id": "a", "text": "Ada joined"}])),
        ("alerts", json!([])),
        ("rows", json!(rows)),
    ] {
        apply(
            &mut ui,
            json!({"op": "collection", "source": source(name), "key": "id", "items": items}),
        );
        inputs.insert(name.to_string(), source(name));
    }
    ui.mount_document(
        source("news"),
        "news",
        inputs,
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    let step = |ui: &mut Ui| {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 900.0));
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
    };
    let set = |ui: &mut Ui, name: &str, value: Value| {
        apply(
            ui,
            json!({"op": "set", "source": source(name), "value": value}),
        )
    };
    let said = |text: &str, assertive: bool| Announcement {
        text: text.into(),
        assertive,
    };
    step(&mut ui);
    step(&mut ui);
    let mut bridge = native_a11y::Bridge::default();
    // The page's mirror, as og-runtime keeps it: the tree it last sent and
    // the frame it last looked at.
    let mut page: (Option<u64>, A11yTree) = (None, ui.a11y().clone());
    // One update: the native bridge's announcements, checked against what
    // the page's mirror hears from the tree it was last sent.
    // The voice of each priority that spoke last: every announcement is
    // spoken by the other one, so AccessKit hears a change even when the
    // words repeat.
    let mut last_voice: [Option<u64>; 2] = [None, None];
    let mut heard = |ui: &Ui, bridge: &mut native_a11y::Bridge| {
        let update = bridge.update(ui, [1.0, 1.0]);
        let native = bridge.take_announcements();
        for (id, n) in update.iter().flat_map(|u| &u.nodes) {
            if id.0 > u64::MAX - 4 && n.value().is_some_and(|v| !v.is_empty()) {
                let p = usize::from(n.live() == Some(Live::Assertive));
                assert_ne!(
                    last_voice[p],
                    Some(id.0),
                    "the voice that spoke last speaks again"
                );
                last_voice[p] = Some(id.0);
            }
        }
        let tree = ui.a11y().clone();
        let on_page = if tree != page.1 && ui.semantic_news(page.0) {
            tree.announcements(&page.1)
        } else {
            Vec::new()
        };
        assert_eq!(
            on_page, native,
            "the page's mirror hears what the native tree does"
        );
        page = (Some(ui.semantic_changes(None).1), tree);
        (update, native)
    };
    let (first, said_first) = heard(&ui, &mut bridge);
    assert!(said_first.is_empty(), "the first tree announces nothing");
    let first = first.expect("the whole tree");
    let status = first
        .nodes
        .iter()
        .find(|(_, n)| n.label() == Some("Ready"))
        .expect("the status");
    assert_eq!(status.1.role(), Role::Status);
    assert_eq!(
        status.1.live(),
        None,
        "content nodes carry no live setting of their own: the voices speak"
    );
    let voices: Vec<_> = first
        .nodes
        .iter()
        .filter(|(id, n)| id.0 > u64::MAX - 4 && n.live().is_some() && n.value().is_none())
        .collect();
    assert_eq!(
        voices.len(),
        4,
        "two polite and two assertive voices, silent"
    );
    step(&mut ui);
    let (idle, nothing) = heard(&ui, &mut bridge);
    assert!(
        idle.is_none() && nothing.is_empty(),
        "an idle tick announces nothing"
    );
    assert_eq!(bridge.read(), 0, "and reads no record");

    // A bound status changes once and announces once, politely; the text
    // outside any region is not heard.
    set(&mut ui, "status", json!("Saved"));
    step(&mut ui);
    let (update, news) = heard(&ui, &mut bridge);
    assert_eq!(news, [said("Saved", false)]);
    assert!(bridge.read() < 12, "read {} records", bridge.read());
    let voice = |update: &native_a11y::accesskit::TreeUpdate, live: Live| -> Vec<String> {
        update
            .nodes
            .iter()
            .filter(|(id, n)| id.0 > u64::MAX - 4 && n.live() == Some(live))
            .filter_map(|(_, n)| n.value().map(str::to_owned))
            .collect()
    };
    assert_eq!(
        voice(&update.unwrap(), Live::Polite),
        ["Saved"],
        "a voice speaks it"
    );
    step(&mut ui);
    assert!(heard(&ui, &mut bridge).1.is_empty(), "once");

    // Atomic: the whole region; plain text beside it says nothing.
    set(&mut ui, "gold", json!("6"));
    step(&mut ui);
    assert_eq!(heard(&ui, &mut bridge).1, [said("Gold 6", false)]);
    // Not atomic: only the line that entered, not the lines already there.
    apply(
        &mut ui,
        json!({"op": "insert", "source": source("feed"), "row": {"id": "b", "text": "Bo joined"}}),
    );
    step(&mut ui);
    assert_eq!(heard(&ui, &mut bridge).1, [said("Bo joined", false)]);
    // The same words twice in a row are heard twice.
    apply(
        &mut ui,
        json!({"op": "insert", "source": source("feed"), "row": {"id": "c", "text": "Bo joined"}}),
    );
    step(&mut ui);
    assert_eq!(heard(&ui, &mut bridge).1, [said("Bo joined", false)]);
    // An alert row entering is assertive; its removal says nothing.
    apply(
        &mut ui,
        json!({"op": "insert", "source": source("alerts"), "row": {"id": "x", "text": "Low health"}}),
    );
    step(&mut ui);
    let (update, news) = heard(&ui, &mut bridge);
    assert_eq!(news, [said("Low health", true)]);
    assert_eq!(voice(&update.unwrap(), Live::Assertive), ["Low health"]);
    apply(
        &mut ui,
        json!({"op": "remove", "source": source("alerts"), "key": "x"}),
    );
    step(&mut ui);
    assert!(
        heard(&ui, &mut bridge).1.is_empty(),
        "a removal says nothing"
    );
    // The same words again are heard again: the other voice speaks them.
    set(&mut ui, "status", json!("Ready"));
    step(&mut ui);
    assert_eq!(heard(&ui, &mut bridge).1, [said("Ready", false)]);
    set(&mut ui, "status", json!("Saved"));
    step(&mut ui);
    let (update, news) = heard(&ui, &mut bridge);
    assert_eq!(news, [said("Saved", false)]);
    let update = update.unwrap();
    let spoken: Vec<_> = update
        .nodes
        .iter()
        .filter(|(id, _)| id.0 > u64::MAX - 4)
        .map(|(id, n)| (id.0, n.value().map(str::to_owned)))
        .collect();
    assert_eq!(
        spoken.len(),
        2,
        "one voice speaks, the other falls silent: {spoken:?}"
    );
    // A hidden region says nothing while hidden.
    set(&mut ui, "shut", json!(true));
    step(&mut ui);
    assert!(heard(&ui, &mut bridge).1.is_empty());
    set(&mut ui, "secret", json!("s1"));
    step(&mut ui);
    assert!(
        heard(&ui, &mut bridge).1.is_empty(),
        "a hidden region says nothing"
    );
    set(&mut ui, "shut", json!(false));
    set(&mut ui, "secret", json!("s2"));
    step(&mut ui);
    assert!(
        heard(&ui, &mut bridge).1.is_empty(),
        "a region shown again arrives with its content: not news"
    );
    set(&mut ui, "secret", json!("s3"));
    step(&mut ui);
    assert_eq!(
        heard(&ui, &mut bridge).1,
        [said("s3", true)],
        "its next change is"
    );

    // A modal dialog opening and closing changes the scope: the page's
    // regions say nothing for it, but a status set as it closes is heard
    // (a toast shown as a dialog closes).
    for open in [true, false, true] {
        set(&mut ui, "ask", json!(open));
        step(&mut ui);
        assert!(
            heard(&ui, &mut bridge).1.is_empty(),
            "the dialog open {open} is not news"
        );
    }
    set(&mut ui, "ask", json!(false));
    set(&mut ui, "status", json!("Quit cancelled"));
    step(&mut ui);
    assert_eq!(heard(&ui, &mut bridge).1, [said("Quit cancelled", false)]);

    // A region that arrives already holding content says nothing (a
    // pushed screen's status); its later changes are heard. An addition
    // inside a region already there is heard (the feed, above).
    set(&mut ui, "unread", json!(true));
    step(&mut ui);
    assert!(
        heard(&ui, &mut bridge).1.is_empty(),
        "a region's arrival is not news"
    );
    set(&mut ui, "count", json!("4 unread"));
    step(&mut ui);
    assert_eq!(heard(&ui, &mut bridge).1, [said("4 unread", false)]);

    // A restore announces nothing, though what nodes say changed with it.
    let saved = ui.save_state();
    set(&mut ui, "status", json!("Loading"));
    set(&mut ui, "gold", json!("9"));
    step(&mut ui);
    assert_eq!(
        heard(&ui, &mut bridge).1,
        [said("Loading", false), said("Gold 9", false)]
    );
    ui.restore_state(&saved).unwrap();
    step(&mut ui);
    let (update, news) = heard(&ui, &mut bridge);
    assert!(update.is_some(), "the tree changed");
    assert!(news.is_empty(), "a restore is not news: {news:?}");
    set(&mut ui, "status", json!("Loaded"));
    step(&mut ui);
    assert_eq!(
        heard(&ui, &mut bridge).1,
        [said("Loaded", false)],
        "news again after it"
    );

    // A host that never takes them holds the newest few.
    for i in 0..ANNOUNCEMENTS + 4 {
        set(&mut ui, "status", json!(format!("n{i}")));
        step(&mut ui);
        bridge.update(&ui, [1.0, 1.0]);
    }
    let kept = bridge.take_announcements();
    assert_eq!(kept.len(), ANNOUNCEMENTS);
    assert_eq!(
        kept.last(),
        Some(&said(&format!("n{}", ANNOUNCEMENTS + 3), false))
    );
    assert_eq!(kept[0], said("n4", false), "the oldest are dropped first");
}

/// The hidden state silences a live region, its own or an ancestor's
/// (`A11yTree::announcements`, which the native bridge shares): a region
/// that `visible false` takes out of the tree says nothing because its
/// nodes left, but a host's tree can also carry `hidden` on a node.
#[test]
fn hidden_live_regions_say_nothing() {
    let node = |path: &str, parent: &str, name: &str, live: &str, hidden: bool| A11yNode {
        path: path.into(),
        parent: parent.into(),
        role: if live.is_empty() { "text" } else { "group" }.into(),
        name: name.into(),
        live: live.into(),
        notify: false,
        states: if hidden {
            vec!["hidden".into()]
        } else {
            Vec::new()
        },
        ..Default::default()
    };
    let tree = |text: &str, region_hidden: bool, top_hidden: bool| A11yTree {
        nodes: vec![
            node("Top", "", "", "", top_hidden),
            node("Top/R", "Top", "", "polite", region_hidden),
            node("Top/R/T", "Top/R", text, "", false),
        ],
        ..Default::default()
    };
    let said = |after: &A11yTree, before: &A11yTree| -> Vec<String> {
        after
            .announcements(before)
            .into_iter()
            .map(|a| a.text)
            .collect()
    };
    assert_eq!(
        said(&tree("b", false, false), &tree("a", false, false)),
        ["b"]
    );
    assert!(
        said(&tree("b", true, false), &tree("a", true, false)).is_empty(),
        "hidden region"
    );
    assert!(
        said(&tree("b", false, true), &tree("a", false, true)).is_empty(),
        "hidden ancestor"
    );
}
