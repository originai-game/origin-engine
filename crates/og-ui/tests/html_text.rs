//! Ordinary HTML in a document (docs/ui.md §2): text, generic blocks and
//! landmarks, lists with CSS markers, `<a href>` links, and checkboxes
//! joined to their labels by `for`. Each is built both ways a document is:
//! immediately (`Ui::component`, a viewmodel) and mounted (`mount_document`,
//! named stores, retained).
//!
//! Ways these could fail, each covered below:
//! - a text tag kept out of the text kind, a heading's level lost on its way
//!   to the accessibility tree, or a type selector matching the internal kind
//!   instead of the tag written → `text_tags_are_text_and_headings_have_levels`
//! - a `<div>` that ignores `display` (or is a row by default), a landmark
//!   without its role, an unnamed `<section>` announced as a region →
//!   `blocks_lay_out_by_display_and_landmarks_have_roles`
//! - list markers missing or wrong for a type, a nested `<ul>` not stepping
//!   disc→circle→square, `start`, `reversed` or `<li value>` ignored, or a
//!   keyed reorder or a middle insert leaving stale numbers →
//!   `list_markers_follow_css_and_renumber_on_reorder`
//! - an `<a href>` not answering a click with a `link` event of its href, or
//!   an inline `<a>` claiming the whole paragraph →
//!   `links_send_their_href`
//! - HTML's UA block margins missing, or vertical margins that add where
//!   CSS 2 collapses them (adjacent siblings, a parent with its first or
//!   last child), or collapse through padding, a set height or a flex
//!   column, or stale after a keyed insert →
//!   `block_margins_follow_the_ua_stylesheet_and_collapse`
//! - horizontal margins refused or laid out wrong: `margin: 0 auto` not
//!   centring a block, a percentage not reading the parent's content width,
//!   a negative one not pulling the box out, `auto` ignored in a flex line
//!   or a grid area, or a mixed-unit `calc()`/`min()`/`clamp()` width, a
//!   store-driven `var()` term or a viewport unit not resolved against its
//!   basis (or stale after a resize) →
//!   `horizontal_margins_and_mixed_unit_lengths_resolve_at_layout`
//! - an empty block (or an empty `<p>`) taking room instead of letting its
//!   margins collapse through it with its neighbours' →
//!   `an_empty_block_collapses_through_itself`
//! - a plain checkbox refused, `<label for>` not joining its checkbox (or
//!   left drawn twice), a field not named by its label, a bare `<label>`
//!   accepted → `checkboxes_and_labels_join_by_for`
//! - a form sent while a field is invalid, the focus not moving to the
//!   first invalid one, a disabled field validated, or `formnovalidate`
//!   ignored → `a_form_submits_only_when_its_fields_are_valid`
//! - a date part stepping past its month or year, `min`/`max` or `step`
//!   ignored, or an empty required date input not holding a form back →
//!   `date_and_time_inputs_step_their_parts_within_min_max_and_step`

use crate::support;

use og_core::glam::Vec2;
use og_ui::reactive::{SourceId, wire};
use og_ui::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use support::*;

/// One document, built either way (other document tests share it).
pub(crate) struct Doc {
    pub(crate) ui: Ui,
    name: String,
    /// Mounted with these stores; `None`: built immediately from `vm`.
    pub(crate) mounted: bool,
    vm: Value,
    /// An immediate build's events, collected tick by tick.
    pub(crate) events: Vec<DocEvent>,
    /// The viewport each step lays out in (800 × 900 unless a test resizes it).
    pub(crate) viewport: Viewport,
}

impl Doc {
    pub(crate) fn new(text: &str, stores: &[(&str, Value)], mounted: bool) -> Doc {
        let mut ui = ui_with_font(test_font_metrics());
        let doc = UiDocument::parse("ui/t.svelte", text).unwrap_or_else(|e| panic!("{e:?}"));
        let name = doc.name.clone();
        ui.define_document(doc);
        let mut d = Doc {
            ui,
            name,
            mounted,
            events: Vec::new(),
            viewport: Viewport::new(800.0, 900.0),
            // An immediate build reads the stores as `$name` from its viewmodel.
            vm: Value::Object(
                stores
                    .iter()
                    .map(|(k, v)| (format!("${k}"), v.clone()))
                    .collect(),
            ),
        };
        if mounted {
            let mut inputs = BTreeMap::new();
            for (k, v) in stores {
                let source = SourceId::new("test", k);
                d.store(k, v.clone());
                inputs.insert(k.to_string(), source);
            }
            d.ui.mount_document(
                SourceId::new("test", "t"),
                &d.name.clone(),
                inputs,
                Default::default(),
                &bind::NoEnv,
            )
            .unwrap_or_else(|e| panic!("mounts: {e}"));
        }
        d.step(InputFrame::default());
        d
    }

    fn apply(&mut self, command: Value) -> Value {
        let reply = wire::decode(
            &self
                .ui
                .apply_reactive(&wire::encode(&json!([command])).unwrap()),
        )
        .unwrap();
        assert_eq!(reply[0]["ok"], true, "{reply}");
        reply[0]["value"].clone()
    }

    /// Store `name`'s value now: what a control wrote back to it.
    pub(crate) fn read(&mut self, name: &str) -> Value {
        if self.mounted {
            self.apply(json!({"op": "read", "source": SourceId::new("test", name)}))["value"]
                .clone()
        } else {
            self.vm[format!("${name}")].clone()
        }
    }

    /// A store: a list is a keyed collection (by `id`).
    pub(crate) fn store(&mut self, name: &str, value: Value) {
        let source = SourceId::new("test", name);
        self.apply(match &value {
            Value::Array(items) => {
                json!({"op": "collection", "source": source, "key": "id", "items": items})
            }
            _ => json!({"op": "signal", "source": source, "initial": value}),
        });
    }

    /// A new value for scalar store `name`, then two steps.
    pub(crate) fn set(&mut self, name: &str, value: Value) {
        if self.mounted {
            self.apply(json!({"op": "set", "source": SourceId::new("test", name), "value": value}));
        } else {
            self.vm[format!("${name}")] = value;
        }
        self.step(InputFrame::default());
        self.step(InputFrame::default());
    }

    /// A keyed edit of list store `name`: `op` on a mounted collection,
    /// the whole list `after` in an immediate viewmodel.
    pub(crate) fn edit(&mut self, name: &str, mut op: Value, after: Value) {
        if self.mounted {
            op["source"] = json!(SourceId::new("test", name));
            self.apply(op);
        } else {
            self.vm[format!("${name}")] = after;
        }
        self.step(InputFrame::default());
        self.step(InputFrame::default());
    }

    pub(crate) fn step(&mut self, input: InputFrame) {
        self.ui.begin(input, self.viewport);
        if self.mounted {
            self.ui
                .build_retained_documents(&bind::NoEnv)
                .expect("builds");
        } else {
            let name = self.name.clone();
            let r = self.ui.component("T", &name, &mut self.vm, &bind::NoEnv);
            self.events.extend(r.events);
        }
        self.ui.end();
        self.events.extend(self.ui.take_doc_events());
    }

    pub(crate) fn el(&self, id: &str) -> Option<UiElement> {
        self.ui
            .probe_elements()
            .into_iter()
            .find(|e| e.id == id || e.path.ends_with(&format!("/{id}")))
    }

    pub(crate) fn get(&self, id: &str) -> UiElement {
        self.el(id)
            .unwrap_or_else(|| panic!("no {id} (mounted: {})", self.mounted))
    }

    pub(crate) fn a11y(&self, path_end: &str) -> A11yNode {
        self.ui
            .a11y()
            .nodes
            .iter()
            .find(|n| n.path.ends_with(&format!("/{path_end}")) || n.path == path_end)
            .cloned()
            .unwrap_or_else(|| panic!("no a11y node {path_end}"))
    }

    pub(crate) fn click_at(&mut self, at: Vec2) -> Vec<DocEvent> {
        let mut events = std::mem::take(&mut self.events);
        events.clear();
        for (pressed, released) in [(true, false), (false, true)] {
            self.step(InputFrame {
                pointer: at,
                pointer_inside: true,
                down: pressed,
                pressed,
                released,
                ..InputFrame::default()
            });
        }
        for _ in 0..30 {
            self.step(InputFrame::default());
        }
        events.append(&mut self.events);
        events
    }

    pub(crate) fn click(&mut self, id: &str) -> Vec<DocEvent> {
        let e = self.get(id);
        self.click_at(Vec2::new(e.x + e.width / 2.0, e.y + e.height / 2.0))
    }

    /// The markers drawn under `list`, top to bottom: a number's text, or
    /// the shape a bullet is drawn as (`disc`, `circle`, `square`).
    fn markers(&self, list: &str) -> Vec<String> {
        let elements = self.ui.probe_elements();
        let mut m: Vec<_> = elements
            .iter()
            .filter(|e| e.id == "Marker" && e.path.contains(&format!("/{list}/")))
            .collect();
        m.sort_by(|a, b| a.y.total_cmp(&b.y));
        m.into_iter()
            .map(|e| {
                if !e.text.is_empty() {
                    return e.text.clone();
                }
                elements
                    .iter()
                    .find(|c| c.path.starts_with(&format!("{}/", e.path)))
                    .map_or_else(String::new, |c| c.id.clone())
            })
            .collect()
    }
}

fn id_text(id: &str) -> &str {
    match id {
        "Title" => "Title",
        "Sub" => "Sub",
        "Para" => "Para",
        "Note" => "Note",
        _ => "Core",
    }
}

pub(crate) fn both(text: &str, stores: &[(&str, Value)], check: impl Fn(&mut Doc)) {
    for mounted in [false, true] {
        check(&mut Doc::new(text, stores, mounted));
    }
}

#[test]
fn text_tags_are_text_and_headings_have_levels() {
    let text = r#"<meta name="component" content="t">
<div><div id="Page">
  <h1 id="Title" i18n="t">Title</h1>
  <h3 id="Sub" i18n="s">Sub</h3>
  <p id="Para" i18n="p">Para</p>
  <span id="Note" i18n="n">Note</span>
  <text id="Core" i18n="c">Core</text>
  <span id="Gone" i18n="g">Gone</span>
</div></div>
<style>
p { padding: 0 0 0 30px; }
span#Gone { display: none; }
text { padding: 0 0 0 50px; }
</style>"#;
    both(text, &[], |d| {
        for id in ["Title", "Sub", "Para", "Note", "Core"] {
            assert_eq!(d.get(id).kind, "label", "{id}");
            assert_eq!(d.get(id).text, id_text(id), "{id}");
        }
        assert_eq!(d.a11y("Title").role, "heading");
        assert_eq!(d.a11y("Title").level, Some(1));
        assert_eq!(d.a11y("Sub").level, Some(3));
        assert_eq!(d.a11y("Para").role, "text");
        assert_eq!(d.a11y("Para").level, None);
        // HTML's UA steps: an h1 is taller than an h3, which is taller
        // than a paragraph at the body step.
        assert!(d.get("Title").height > d.get("Sub").height);
        assert!(d.get("Sub").height >= d.get("Para").height);
        // A type selector matches the tag written, not the kind: `p`
        // pads only the paragraph, `text` only the core text.
        let page = d.get("Page");
        assert_eq!(d.get("Para").x - page.x, 0.0);
        assert!(d.get("Para").width > d.get("Note").width);
        assert!(d.el("Gone").is_none(), "span#Gone matches the written tag");
        assert!(d.get("Core").width >= 50.0);
    });
}

#[test]
fn blocks_lay_out_by_display_and_landmarks_have_roles() {
    let text = r#"<meta name="component" content="t">
<div><main id="Main">
  <header id="Head"><span id="H1" i18n="a">A</span><span id="H2" i18n="b">B</span></header>
  <nav id="Nav" aria-label="Site"><span id="N1" i18n="a">A</span><span id="N2" i18n="b">B</span></nav>
  <div id="Grid"><span id="G1" i18n="a">A</span><span id="G2" i18n="b">B</span><span id="G3" i18n="c">C</span></div>
  <section id="Plain"><span i18n="a">A</span></section>
  <section id="Named" aria-label="News"><span i18n="a">A</span></section>
  <aside id="Side"><span i18n="a">A</span></aside>
  <article id="Art"><span i18n="a">A</span></article>
  <footer id="Foot"><span i18n="a">A</span></footer>
  <div id="Hidden"><span i18n="a">A</span></div>
</main></div>
<style>
nav { display: flex; gap: 10px; }
#Grid { display: grid; grid-template-columns: 100px 100px; }
div#Hidden { display: none; }
</style>"#;
    both(text, &[], |d| {
        // Block by default: children top to bottom.
        let (h1, h2) = (d.get("H1"), d.get("H2"));
        assert!(
            h2.y > h1.y && h2.x == h1.x,
            "a header stacks: {h1:?} {h2:?}"
        );
        // display: flex is a row, through a type selector on the tag.
        let (n1, n2) = (d.get("N1"), d.get("N2"));
        assert_eq!(n2.y, n1.y);
        assert_eq!(n2.x, n1.x + n1.width + 10.0);
        // display: grid takes tracks.
        let (g1, g2, g3) = (d.get("G1"), d.get("G2"), d.get("G3"));
        assert_eq!((g2.x - g1.x, g2.y), (100.0, g1.y));
        assert_eq!((g3.x, g3.y > g1.y), (g1.x, true));
        assert!(d.el("Hidden").is_none());
        for (id, role) in [
            ("Main", "main"),
            ("Head", "banner"),
            ("Nav", "navigation"),
            ("Named", "region"),
            ("Side", "complementary"),
            ("Art", "article"),
            ("Foot", "contentinfo"),
        ] {
            assert_eq!(d.a11y(id).role, role, "{id}");
        }
        assert!(
            !d.ui.a11y().nodes.iter().any(|n| n.path.ends_with("/Plain")),
            "an unnamed section is no region"
        );
    });
    let refused = UiDocument::parse(
        "ui/t.svelte",
        "<meta name=\"component\" content=\"t\"><column><button id=\"B\">x</button></column><style>button { display: flex; }</style>",
    )
    .unwrap_err()
    .join("\n");
    assert!(
        refused.contains("display: flex lays out a <div>"),
        "{refused}"
    );
    let internal = UiDocument::parse(
        "ui/t.svelte",
        "<meta name=\"component\" content=\"t\"><box></box>",
    )
    .unwrap_err()
    .join("\n");
    assert!(
        internal.contains("<box> is not a core element; write <div>"),
        "{internal}"
    );
}

#[test]
fn block_margins_follow_the_ua_stylesheet_and_collapse() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="A"></div>
  <div id="B"></div>
  <div id="C"></div>
  <div id="Outer"><div id="Inner"></div></div>
  <div id="Padded"><div id="Kid"></div></div>
  <div id="Tall"><div id="Low"></div></div>
  <div id="Flex"><div id="F1"></div><div id="F2"></div></div>
  <p id="Reset" i18n="r">Reset</p>
  <p id="P1" i18n="p">One</p>
  <h1 id="H" i18n="h">Head</h1>
  {#each $items as it (it.id)}<p>{it.id}</p>{/each}
  <ul id="L"><li i18n="x">X</li></ul>
</div></div>
<style>
#Root { padding: 10px; }
#Root div { height: 20px; }
#Root #Outer, #Root #Padded, #Root #Flex { height: fit-content; }
#A { margin-bottom: 30px; }
#B { margin-top: 10px; margin-bottom: -5px; }
#C { margin-top: 15px; }
#Outer { margin-top: 5px; }
#Inner { margin: 25px 0 12px; }
#Padded { padding: 4px 0 0 0; }
#Kid { margin-top: 25px; }
#Root #Tall { height: 60px; margin-top: 0; }
#Low { margin-bottom: 40px; }
#Flex { display: flex; flex-direction: column; margin-top: 7px; }
#F1 { margin-bottom: 10px; }
#F2 { margin-top: 10px; }
#Reset { margin: 0; }
</style>"#;
    let items = json!([{"id": "a"}, {"id": "c"}]);
    both(text, &[("items", items)], |d| {
        let em = d.ui.theme().type_ramp.body.size;
        let h1_em = d.ui.theme().type_ramp.xl3.size;
        let bottom = |e: &UiElement| e.y + e.height;
        let gap = |a: &str, b: &str| d.get(b).y - bottom(&d.get(a));
        let root = d.get("Root");
        // Root's padding keeps A's margin inside it; A has none on top.
        assert_eq!(d.get("A").y - root.y, 10.0);
        // Adjacent siblings: the larger positive margin wins…
        assert_eq!(gap("A", "B"), 30.0);
        // …and a negative one is added to it.
        assert_eq!(gap("B", "C"), 10.0);
        // A first child's margin collapses through its parent's top: Inner
        // sits at Outer's edge, and Outer stands max(5, 25) below C.
        assert_eq!(gap("C", "Outer"), 25.0);
        assert_eq!(d.get("Inner").y, d.get("Outer").y);
        // …and a last child's through an auto-height parent's bottom: Outer
        // is Inner's height, and Inner's 12px sits below it.
        assert_eq!(d.get("Outer").height, 20.0);
        assert_eq!(gap("Outer", "Padded"), 12.0);
        // Padding separates them: Kid's margin stays inside Padded.
        assert_eq!(d.get("Kid").y - d.get("Padded").y, 4.0 + 25.0);
        assert_eq!(d.get("Padded").height, 4.0 + 25.0 + 20.0);
        // A set height stops the bottom collapse: Low's 40px stays inside.
        assert_eq!(gap("Tall", "Flex"), 7.0);
        // A flex column adds its children's margins and lets none out.
        assert_eq!(gap("F1", "F2"), 20.0);
        assert_eq!(d.get("F1").y, d.get("Flex").y);
        // The UA gives <p> 1em top and bottom; `margin: 0` resets it.
        assert_eq!(gap("Flex", "Reset"), 0.0);
        assert_eq!(gap("Reset", "P1"), em);
        // An h1's 0.67em is of its own (xl3) step, collapsed with P1's 1em.
        // (Edges snap to whole pixels.)
        assert!(
            (gap("P1", "H") - 0.67 * h1_em).abs() <= 0.5 && 0.67 * h1_em > em,
            "{} for 0.67 × {h1_em}",
            gap("P1", "H")
        );
        // Adjacent paragraphs collapse to one em, a list's 1em with them,
        // including after a keyed insert in the middle.
        let paras = |d: &Doc| -> Vec<UiElement> {
            d.ui.probe_elements()
                .into_iter()
                .filter(|e| ["a", "b", "c"].contains(&e.text.as_str()))
                .collect()
        };
        assert_eq!(paras(d).len(), 2);
        d.edit(
            "items",
            json!({"op": "insert", "row": {"id": "b"}, "before": "c"}),
            json!([{"id": "a"}, {"id": "b"}, {"id": "c"}]),
        );
        let p = paras(d);
        assert_eq!(
            p.iter().map(|e| e.text.as_str()).collect::<Vec<_>>(),
            ["a", "b", "c"]
        );
        assert!((p[0].y - bottom(&d.get("H")) - 0.67 * h1_em).abs() <= 0.5);
        for w in p.windows(2) {
            assert_eq!(
                w[1].y - bottom(&w[0]),
                em,
                "{:?} → {:?}",
                w[0].text,
                w[1].text
            );
        }
        assert_eq!(d.get("L").y - bottom(&p[2]), em);
    });
}

#[test]
fn horizontal_margins_and_mixed_unit_lengths_resolve_at_layout() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="Centre"></div>
  <div id="Right"></div>
  <div id="Pct"></div>
  <div id="Neg"></div>
  <div id="Calc"></div>
  <div id="Min"></div>
  <div id="Clamp"></div>
  <div id="Var" style:--w={$frac}></div>
  <div id="Vw"></div>
  <div id="Row"><div id="A1"></div><div id="A2"></div><div id="A3"></div></div>
  <div id="Row2"><div id="B1"></div></div>
  <div id="Col"><div id="C1"></div><div id="C2"></div></div>
  <div id="Grid"><div id="G1"></div><div id="G2"></div></div>
</div></div>
<style>
#Root { width: 400px; margin-left: 40px; }
#Root div { height: 10px; }
#Centre { width: 200px; margin: 0 auto; }
#Right { width: 100px; margin-left: auto; }
#Pct { margin: 0 10% 0 25px; }
#Neg { margin-left: -10px; margin-right: 30px; }
#Calc { width: calc(50% - 12px); margin-left: calc(25% + 3px); }
#Min { width: min(100%, 150px); }
#Clamp { width: clamp(50px, 10%, 300px); }
#Var { width: calc(var(--w) * 100% - 20px); }
#Root #Vw { width: calc(50vw - 40px); height: 2vh; }
#Root #Row { display: flex; height: 100px; }
#A1 { width: 50px; margin: auto 20px 0 10px; }
#A2 { width: 60px; margin-left: auto; }
#A3 { width: calc(25% - 10px); margin-right: 5%; }
#Root #Row2 { display: flex; height: 60px; }
#B1 { width: 100px; margin: auto; }
#Root #Col { display: flex; flex-direction: column; height: 100px; }
#C2 { margin-top: auto; }
#Root #Grid { display: grid; grid-template-columns: 200px 200px; height: fit-content; }
#G1 { width: 100px; margin: 0 auto; }
#G2 { margin-left: 30px; }
</style>"#;
    both(text, &[("frac", json!(0.5))], |d| {
        let x0 = d.get("Root").x;
        assert_eq!(x0, 40.0);
        let at = |d: &Doc, id: &str| {
            let e = d.get(id);
            (e.x - x0, e.width)
        };
        // Block flow (CSS 2 §10.3.3): auto margins centre or push, a
        // percentage reads the parent's 400px, a negative one pulls out.
        assert_eq!(at(d, "Centre"), (100.0, 200.0));
        assert_eq!(at(d, "Right"), (300.0, 100.0));
        assert_eq!(at(d, "Pct"), (25.0, 400.0 - 40.0 - 25.0));
        assert_eq!(at(d, "Neg"), (-10.0, 400.0 + 10.0 - 30.0));
        // Mixed units are a sum layout resolves against the same 400px.
        assert_eq!(at(d, "Calc"), (103.0, 188.0));
        assert_eq!(at(d, "Min").1, 150.0);
        assert_eq!(at(d, "Clamp").1, 50.0);
        assert_eq!(at(d, "Var").1, 180.0);
        // 50vw of the 800px viewport, less 40px; 2vh of 900px.
        assert_eq!((at(d, "Vw").1, d.get("Vw").height), (360.0, 18.0));
        // A flex row: margins take room, the auto one takes what is left
        // (400 − 80 − 60 − 110 = 150), and an auto top sends A1 down.
        let row = d.get("Row");
        assert_eq!(at(d, "A1"), (10.0, 50.0));
        assert_eq!(at(d, "A2"), (80.0 + 150.0, 60.0));
        assert_eq!(at(d, "A3"), (290.0, 90.0));
        assert_eq!(d.get("A1").y - row.y, 90.0);
        assert_eq!(d.get("A2").y, row.y);
        // `margin: auto` centres on both axes.
        let b1 = d.get("B1");
        assert_eq!((b1.x - x0, b1.y - d.get("Row2").y), (150.0, 25.0));
        // A flex column's auto top margin takes its free space.
        assert_eq!(d.get("C2").y - d.get("Col").y, 90.0);
        assert_eq!(d.get("C1").y, d.get("Col").y);
        // A grid area: auto centres in the 200px column, a margin insets
        // the stretched item.
        assert_eq!(at(d, "G1"), (50.0, 100.0));
        assert_eq!(at(d, "G2"), (230.0, 170.0));

        // A store-driven var() term and a resize re-resolve their elements.
        d.set("frac", json!(0.25));
        assert_eq!(at(d, "Var").1, 80.0);
        d.viewport = Viewport::new(600.0, 900.0);
        d.step(InputFrame::default());
        d.step(InputFrame::default());
        assert_eq!(at(d, "Vw").1, 260.0);
        assert_eq!(at(d, "Centre"), (100.0, 200.0));
    });
}

#[test]
fn an_empty_block_collapses_through_itself() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="A"></div>
  <div id="Empty"></div>
  <div id="B"></div>
  <div id="C"></div>
  <p id="EmptyP"></p>
  <div id="D"></div>
  <div id="E"></div>
  <div id="Kept"></div>
  <div id="F"></div>
</div></div>
<style>
#Root { width: 400px; padding: 10px; }
#A, #B, #C, #D, #E, #F { height: 10px; }
#A { margin-bottom: 20px; }
#Empty { margin: 30px 0 10px; }
#B { margin-top: 5px; }
#Kept { margin: 30px 0 10px; padding: 1px 0 0 0; }
</style>"#;
    both(text, &[], |d| {
        let em = d.ui.theme().type_ramp.body.size;
        let gap = |a: &str, b: &str| d.get(b).y - (d.get(a).y + d.get(a).height);
        // A, an empty block and B: one set of margins, max(20, 30, 10, 5).
        assert_eq!(d.get("Empty").height, 0.0);
        assert_eq!(gap("A", "B"), 30.0);
        // An empty <p>: no line, and its 1em margins collapse into one.
        assert_eq!(d.get("EmptyP").height, 0.0);
        assert_eq!(gap("C", "D"), em);
        // Padding gives a block an edge: its margins no longer adjoin.
        assert_eq!(d.get("Kept").height, 1.0);
        assert_eq!(gap("E", "Kept"), 30.0);
        assert_eq!(gap("Kept", "F"), 10.0);
    });
}

fn steps(ids: &[&str]) -> Value {
    json!(ids.iter().map(|id| json!({ "id": id })).collect::<Vec<_>>())
}

#[test]
fn list_markers_follow_css_and_renumber_on_reorder() {
    let text = r#"<meta name="component" content="t">
<div><div id="Page">
  <ul id="Bullets"><li i18n="a">A</li><li>
    <span i18n="b">B</span>
    <ul id="Inner"><li i18n="c">C</li><li><ul id="Deep"><li i18n="d">D</li></ul></li></ul>
  </li></ul>
  <ol id="Steps">
    {#each $steps as s (s.id)}<li>{s.id}</li>{/each}
  </ol>
  <ol id="Down" start="10" reversed><li i18n="a">A</li><li i18n="b">B</li><li i18n="c">C</li></ol>
  <ol id="Valued"><li i18n="a">A</li><li value="7" i18n="b">B</li><li i18n="c">C</li></ol>
  <ol id="Roman"><li i18n="a">A</li><li i18n="b">B</li><li i18n="c">C</li><li i18n="d">D</li></ol>
  <ol id="Alpha"><li i18n="a">A</li><li i18n="b">B</li></ol>
  <ul id="Bare"><li i18n="a">A</li></ul>
  <ul id="In"><li id="InItem" i18n="a">A</li></ul>
</div></div>
<style>
#Roman { list-style-type: upper-roman; }
#Alpha { list-style: lower-alpha inside; }
#Bare { list-style: none; }
#In { list-style-position: inside; }
</style>"#;
    both(text, &[("steps", steps(&["x", "y", "z"]))], |d| {
        assert_eq!(
            d.markers("Bullets"),
            ["disc", "disc", "circle", "circle", "square"]
        );
        assert_eq!(d.markers("Inner"), ["circle", "circle", "square"]);
        assert_eq!(d.markers("Steps"), ["1.", "2.", "3."]);
        assert_eq!(d.markers("Down"), ["10.", "9.", "8."]);
        assert_eq!(d.markers("Valued"), ["1.", "7.", "8."]);
        assert_eq!(d.markers("Roman"), ["I.", "II.", "III.", "IV."]);
        assert_eq!(d.markers("Alpha"), ["a.\u{a0}", "b.\u{a0}"]);
        assert!(d.markers("Bare").is_empty());
        // Outside hangs before the item; inside sits in its row.
        let item = d.get("Bullets").x;
        let first =
            d.ui.probe_elements()
                .into_iter()
                .find(|e| e.id == "Marker" && e.path.contains("/Bullets/"))
                .unwrap();
        assert!(
            first.x + first.width <= item + 40.0 && first.x >= item,
            "{first:?}"
        );
        let inside =
            d.ui.probe_elements()
                .into_iter()
                .find(|e| e.id == "Marker" && e.path.contains("/InItem/"))
                .unwrap();
        assert!(inside.x >= d.get("InItem").x, "{inside:?}");
        // A keyed reorder renumbers by place; a middle insert shifts the
        // items after it. The numbers drawn follow the text they sit by.
        let texts = |d: &Doc| -> Vec<String> {
            let mut items: Vec<_> =
                d.ui.probe_elements()
                    .into_iter()
                    .filter(|e| e.kind == "label" && e.id != "Marker" && e.path.contains("/Steps/"))
                    .collect();
            items.sort_by(|a, b| a.y.total_cmp(&b.y));
            items.into_iter().map(|e| e.text).collect()
        };
        d.edit(
            "steps",
            json!({"op": "move", "key": "z", "before": "x"}),
            steps(&["z", "x", "y"]),
        );
        assert_eq!(texts(d), ["z", "x", "y"]);
        assert_eq!(d.markers("Steps"), ["1.", "2.", "3."]);
        d.edit(
            "steps",
            json!({"op": "insert", "row": {"id": "w"}, "before": "x"}),
            steps(&["z", "w", "x", "y"]),
        );
        assert_eq!(texts(d), ["z", "w", "x", "y"]);
        assert_eq!(d.markers("Steps"), ["1.", "2.", "3.", "4."]);
        assert_eq!(d.a11y("Steps").role, "list");
    });
    let refused = UiDocument::parse(
        "ui/t.svelte",
        "<meta name=\"component\" content=\"t\"><div><li>x</li></div>",
    )
    .unwrap_err()
    .join("\n");
    assert!(refused.contains("<li> belongs inside"), "{refused}");
}

#[test]
fn links_send_their_href() {
    let text = r#"<meta name="component" content="t">
<div><div id="Page">
  <a id="Shop" href="screen:shop">Open the shop</a>
  <span id="Same">Open the shop</span>
  <p id="Para">Read the <a href="rules">rules</a> and <b>enjoy</b>.</p>
</div></div>
<style>
a { padding: 0 0 0 40px; }
</style>"#;
    both(text, &[], |d| {
        let shop = d.get("Shop");
        assert_eq!(shop.kind, "richtext");
        assert_eq!(shop.text, "Open the shop");
        // `a` pads the link, not the span of the same text.
        let same = d.get("Same").width;
        assert!(
            (shop.width - same - 40.0).abs() < 1.0,
            "{} vs {same}",
            shop.width
        );
        let events = d.click("Shop");
        assert!(
            events
                .iter()
                .any(|e| e.kind == "link" && e.name == "screen:shop"),
            "{events:?}"
        );
        let para = d.get("Para");
        assert_eq!(para.text, "Read the rules and enjoy.");
        // A press on "Read" (the paragraph's start) is no link.
        let events = d.click_at(Vec2::new(para.x + 2.0, para.y + para.height / 2.0));
        assert!(!events.iter().any(|e| e.kind == "link"), "{events:?}");
        let link = d.a11y("Para/link0");
        assert_eq!((link.role.as_str(), link.name.as_str()), ("link", "rules"));
        let events = d.click_at(Vec2::new(
            link.x + link.width / 2.0,
            link.y + link.height / 2.0,
        ));
        assert!(
            events.iter().any(|e| e.kind == "link" && e.name == "rules"),
            "{events:?}"
        );
    });
}

#[test]
fn checkboxes_and_labels_join_by_for() {
    let text = r#"<meta name="component" content="t">
<div><div id="Page">
  <input type="checkbox" switch id="Mute" bind:checked={$mute}>
  <label for="Mute" i18n="m">Mute music</label>
  <label for="Nick" i18n="n">Nickname</label>
  <input type="text" id="Nick" bind:value={$nick}>
</div></div>
<style>
input[type="checkbox"] { width: 300px; }
</style>"#;
    both(text, &[("mute", json!(false)), ("nick", json!(""))], |d| {
        let mute = d.a11y("Mute");
        assert_eq!(
            (mute.role.as_str(), mute.name.as_str()),
            ("switch", "Mute music")
        );
        assert_eq!(d.get("Mute").width, 300.0, "input[type=checkbox] matches");
        let captions =
            d.ui.probe_elements()
                .iter()
                .filter(|e| e.kind == "label" && e.text == "Mute music")
                .count();
        assert_eq!(
            captions, 1,
            "the joined label is the switch's caption, drawn once"
        );
        assert_eq!(d.a11y("Nick").name, "Nickname");
        // Its switch track takes the press (the box is wider than it).
        let events = d.click("Mute/Track");
        assert!(
            events
                .iter()
                .any(|e| e.kind == "changed" && e.name == "checked" && e.value == json!(true)),
            "{events:?}"
        );
        if d.mounted {
            let reply = wire::decode(
                &d.ui.apply_reactive(
                    &wire::encode(&json!([{"op":"read","source":SourceId::new("test","mute")}]))
                        .unwrap(),
                ),
            )
            .unwrap();
            assert_eq!(reply[0]["value"]["value"], json!(true), "{reply}");
        }
    });
    for (doc, why) in [
        (
            "<label i18n=\"x\">x</label>",
            "<label> is not a core element",
        ),
        (
            "<label for=\"Nope\" i18n=\"x\">x</label>",
            "for=\"Nope\" names no element",
        ),
    ] {
        let e = UiDocument::parse(
            "ui/t.svelte",
            &format!("<meta name=\"component\" content=\"t\"><div>{doc}</div>"),
        )
        .unwrap_err()
        .join("\n");
        assert!(e.contains(why), "{e}");
    }
}

const PAGE: &str = r#"<meta name="component" content="t">
<main id="Page">
  <header><h1 i18n="h1">Patch notes</h1><p i18n="intro">Season two brings new maps.</p></header>
  <nav aria-label="Sections"><a href="maps" i18n="maps">Maps</a><a href="heroes" i18n="heroes">Heroes</a><a href="shop" i18n="shop">Shop</a></nav>
  <section aria-label="Changes">
    <h2 i18n="h2">Changes</h2>
    <ul>
      <li i18n="a">Faster matchmaking</li>
      <li><span i18n="b">New modes</span>
        <ul><li i18n="c">Duel</li><li><span i18n="d">Siege</span><ul><li i18n="e">Night</li></ul></li></ul>
      </li>
    </ul>
    <h3 i18n="h3">Steps</h3>
    <ol>{#each $steps as s (s.id)}<li>{s.id}</li>{/each}</ol>
    <ol id="Roman"><li i18n="r1">Prologue</li><li i18n="r2">Ascent</li><li i18n="r3">Fall</li><li i18n="r4">Epilogue</li></ol>
    <p>Read the <a href="rules">rules</a> and <b>enjoy</b> the <i>season</i>.</p>
  </section>
</main>
<style>
main { padding: 24px; gap: 12px; }
nav { display: flex; gap: 16px; }
#Roman { list-style-type: upper-roman; }
</style>"#;

#[test]
#[ignore = "requires GPU"]
fn html_text_matches_its_reference_frames() {
    use og_render::*;
    for (width, height) in [(1280u32, 720u32), (390, 844)] {
        let mut scene = RenderScene::new();
        let font = add_test_font(&mut scene);
        let mut ui = ui_with_font(font);
        ui.define_document(UiDocument::parse("ui/t.svelte", PAGE).unwrap());
        let source = SourceId::new("test", "steps");
        let reply = wire::decode(
            &ui.apply_reactive(
                &wire::encode(&json!([{"op": "collection", "source": source, "key": "id",
                "items": steps(&["Download", "Install", "Play"])}]))
                .unwrap(),
            ),
        )
        .unwrap();
        assert_eq!(reply[0]["ok"], true, "{reply}");
        ui.mount_document(
            SourceId::new("test", "t"),
            "t",
            BTreeMap::from([("steps".to_string(), source)]),
            Default::default(),
            &bind::NoEnv,
        )
        .unwrap();
        let vp = Viewport::new(width as f32, height as f32);
        for _ in 0..3 {
            ui.begin(InputFrame::default(), vp);
            ui.build_retained_documents(&bind::NoEnv).unwrap();
            ui.end();
        }
        let mut overlay = Overlay::new(width as f32, height as f32, 1.0);
        ui.frame().emit(&EmitDesc::default(), &mut overlay);
        scene.overlay = Some(overlay);
        let bg = ui.theme().palette.backdrop.linear();
        scene.environment.background =
            Background::Color(og_core::glam::Vec3::new(bg[0], bg[1], bg[2]));
        scene.post.tonemap = Tonemap::None;
        let mut renderer = Renderer::new_blocking(&RenderDesc {
            width,
            height,
            ..Default::default()
        })
        .expect("a headless adapter");
        let stats = renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
        assert_eq!(stats.missing_glyphs, 0);
        let image = renderer.read_offscreen().unwrap();
        let name = format!("html-text-{width}.png");
        image::save_buffer(
            out_dir().join(&name),
            &image.pixels,
            width,
            height,
            image::ColorType::Rgba8,
        )
        .unwrap();
        let own = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("references/metal")
            .join(&name);
        if std::env::var_os("OG_UI_CAPTURE").is_some() {
            record_reference(&own, width, height, &image.pixels);
        }
        let expected = image::open(&own)
            .unwrap_or_else(|e| panic!("{}: {e} (OG_UI_CAPTURE=1 records it)", own.display()))
            .to_rgba8();
        let (rmse, _) = compare(&image.pixels, expected.as_raw(), renderer.info(), &own)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        println!("{name}: RMSE {rmse:.6}");
    }
}

#[test]
fn a_form_submits_only_when_its_fields_are_valid() {
    let text = r#"<meta name="component" content="t">
<div><form id="Form" onsubmit={save}>
  <input type="text" id="Name" required bind:value={$name}>
  <input type="email" id="Mail" bind:value={$mail}>
  <input type="text" id="Off" required disabled bind:value={$off}>
  <button id="Save">Save</button>
  <button id="Skip" formnovalidate>Skip</button>
</form></div>"#;
    let stores = [
        ("name", json!("")),
        ("mail", json!("x")),
        ("off", json!("")),
    ];
    let saved = |events: &[DocEvent]| events.iter().any(|e| e.kind == "emit" && e.name == "save");
    both(text, &stores, |d| {
        // The first invalid field takes the focus and nothing is sent; a
        // disabled one is barred from validation.
        let events = d.click("Save");
        assert!(!saved(&events), "{events:?}");
        assert!(
            d.ui.focus_path(0).ends_with("/Name"),
            "{}",
            d.ui.focus_path(0)
        );
        assert!(saved(&d.click("Skip")));
        d.set("name", json!("Ana"));
        assert!(!saved(&d.click("Save")));
        assert!(
            d.ui.focus_path(0).ends_with("/Mail"),
            "{}",
            d.ui.focus_path(0)
        );
        d.set("mail", json!("ana@example.com"));
        assert!(saved(&d.click("Save")));
    });
}

#[test]
fn date_and_time_inputs_step_their_parts_within_min_max_and_step() {
    let text = r#"<meta name="component" content="t">
<div><form id="Form" onsubmit={book}>
  <input type="date" id="When" min="2024-02-27" max="2024-03-01" bind:value={$day}>
  <input type="time" id="At" step="900" bind:value={$at}>
  <input type="week" id="Wk" required bind:value={$wk}>
  <button id="Book">Book</button>
</form></div>"#;
    let stores = [
        ("day", json!("2024-02-28")),
        ("at", json!("09:00")),
        ("wk", json!("")),
    ];
    both(text, &stores, |d| {
        // A press on a part's +: the value it writes back (an immediate
        // build's viewmodel takes it as its game would).
        let step = |d: &mut Doc, part: &str, store: &str| -> Value {
            let events = d.click(part);
            let Some(v) = events
                .iter()
                .rfind(|e| e.kind == "changed" && e.name == "dateValue")
                .map(|e| e.value.clone())
            else {
                return d.read(store);
            };
            if d.mounted {
                assert_eq!(d.read(store), v);
            } else {
                d.set(store, v.clone());
            }
            v
        };
        assert_eq!(step(d, "When/Day/Frame/Inc", "day"), json!("2024-02-29"));
        assert_eq!(
            step(d, "When/Day/Frame/Inc", "day"),
            json!("2024-03-01"),
            "the 30th is no date"
        );
        assert_eq!(
            step(d, "When/Month/Frame/Inc", "day"),
            json!("2024-03-01"),
            "April is past max"
        );
        // The minutes step by the step's 15.
        assert_eq!(step(d, "At/Minute/Frame/Inc", "at"), json!("09:15"));
        // An empty required week holds the form back.
        let events = d.click("Book");
        assert!(!events.iter().any(|e| e.name == "book"), "{events:?}");
        step(d, "Wk/Week/Frame/Inc", "wk");
        assert!(d.click("Book").iter().any(|e| e.name == "book"));
        assert_eq!(d.a11y("When").role, "group");
    });
}
