//! The screens og-ui's tests drive and its reference frames show: the
//! sample screen, the game widget set, the text fields and the mixed-script
//! labels, each a document (a `.svelte` text and the stores it reads)
//! mounted once and stepped frame by frame. `scenes.rs` makes render
//! scenes of them; the browser tiers (crates/og-tests-web) compile both.
//! Nothing here reads a file: the test font comes in as bytes
//! ([`FontFiles`]).

use og_core::glam::Vec2;
use og_ui::reactive::{SourceId, wire};
use og_ui::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The scope the screens' stores are declared in.
const SCOPE: &str = "screen";

/// One host operation on `ui`'s stores; answers its value.
fn host(ui: &mut Ui, command: Value) -> Value {
    let reply =
        wire::decode(&ui.apply_reactive(&wire::encode(&json!([command])).unwrap())).unwrap();
    assert_eq!(reply[0]["ok"], true, "{reply}");
    reply[0]["value"].clone()
}

/// Screen store `name`'s value now: what a control wrote back to it.
pub fn screen_value(ui: &mut Ui, name: &str) -> Value {
    host(
        ui,
        json!({"op": "read", "source": SourceId::new(SCOPE, name)}),
    )["value"]
        .clone()
}

/// A new value for scalar screen store `name` (read at the next build).
pub fn set_screen_value(ui: &mut Ui, name: &str, value: Value) {
    host(
        ui,
        json!({"op": "set", "source": SourceId::new(SCOPE, name), "value": value}),
    );
}

/// Defines the document `text` (component `name`) in `ui` and mounts it
/// as `screen:<name>` with `stores`, declared in the scope `screen` (a list
/// of objects is a collection keyed by `id`).
pub fn mount_screen_document(ui: &mut Ui, name: &str, text: &str, stores: &[(&str, Value)]) {
    let doc = UiDocument::parse(&format!("ui/{name}.svelte"), text)
        .unwrap_or_else(|e| panic!("{name}: {e:?}"));
    assert_eq!(doc.name, name);
    ui.define_document(doc);
    let mut inputs = BTreeMap::new();
    for (store, value) in stores {
        let source = SourceId::new(SCOPE, store);
        host(
            ui,
            match value {
                Value::Array(items) if items.iter().all(Value::is_object) => {
                    json!({"op": "collection", "source": source, "key": "id", "items": items})
                }
                _ => json!({"op": "signal", "source": source, "initial": value}),
            },
        );
        inputs.insert(store.to_string(), source);
    }
    ui.mount_document(
        SourceId::new(SCOPE, name),
        name,
        inputs,
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap_or_else(|e| panic!("{name} mounts: {e}"));
}

/// One frame of `ui` at `vp` with its mounted documents built; answers the
/// document events it sent.
pub fn step_screen(ui: &mut Ui, input: InputFrame, vp: Viewport) -> Vec<DocEvent> {
    ui.begin(input, vp);
    ui.build_retained_documents(&bind::NoEnv)
        .expect("the screens build");
    ui.end();
    ui.take_doc_events()
}

/// The clicks among `events` (the actions a press sent, `onclick`), each
/// named by the clicked element's path under its screen's root
/// (`Menu/Start`, `Store/List/Row#2/Buy`).
pub fn clicks(events: &[DocEvent]) -> Vec<String> {
    events
        .iter()
        .filter(|e| e.kind == "emit")
        .map(|e| {
            e.path
                .split_once('/')
                .map_or(&*e.path, |(_, p)| p)
                .to_string()
        })
        .collect()
}

/// The game state the sample screen edits: its stores, and every click as
/// [`clicks`] names it.
#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub music: bool,
    pub volume: f32,
    pub search: String,
    pub loading: f32,
    pub clicks: Vec<String>,
}

impl Sample {
    /// The sample screen's stores in `ui` now, with `clicks`.
    pub fn read(ui: &mut Ui, clicks: Vec<String>) -> Sample {
        Sample {
            music: screen_value(ui, "music").as_bool().expect("music"),
            volume: screen_value(ui, "volume").as_f64().expect("volume") as f32,
            search: screen_value(ui, "search")
                .as_str()
                .expect("search")
                .to_string(),
            loading: screen_value(ui, "loading").as_f64().expect("loading") as f32,
            clicks,
        }
    }
}

pub const ITEMS: u32 = 12;

/// The play surface of the sample: 800×500 CSS pixels at 1.5× (1200×750
/// device pixels; compact density, rectangles snapped to 2/3 CSS px).
pub fn viewport() -> Viewport {
    Viewport {
        width: 800.0,
        height: 500.0,
        dpr: 1.5,
        ..Viewport::default()
    }
}

/// A game menu and a store: every widget kind, a scroll list, a wrapping row.
pub const SAMPLE_DOC: &str = r#"<meta name="component" content="sample">
<row id="Screen">
  <panel id="Menu">
    <text id="Title">Origin</text>
    <text id="Subtitle">A small game made with Origin Engine.</text>
    <button id="Start" variant="primary" onclick={start}>Start</button>
    <button id="Options" variant="secondary" onclick={options}>Options</button>
    <button id="Quit" variant="danger" onclick={quit}>Quit</button>
    <label>Music <input type="checkbox" switch id="Music" bind:checked={$music} onclick={music}></label>
    <input type="range" id="Volume" bind:value={$volume}>
    <progress id="Loading" value={$loading}></progress>
  </panel>
  <panel id="Store">
    <row id="Header">
      <text id="Heading">Store</text>
      <text id="Engine">原点引擎</text>
      <text id="Coins">1250 coins</text>
    </row>
    <input id="Search" placeholder="Search items" bind:value={$search}>
    <scroll id="List">
      {#each $items as item (item.id)}
        <row id="Row">
          <text id="Name">{item.name}</text>
          <button id="Buy" class="small" title={item.price} onclick={buy}>Buy</button>
        </row>
      {/each}
    </scroll>
    <row id="Tags">
      {#each $tags as tag (tag.id)}
        <button id="Tag" class="small" variant="quiet" onclick={pick}>{tag.name}</button>
      {/each}
    </row>
  </panel>
</row>
<style>
#Screen { width: stretch; height: stretch; padding: var(--space-xl); gap: var(--space-lg); background: var(--backdrop); }
#Menu { width: 220px; height: stretch; }
#Title { font: var(--text-display-sm); letter-spacing: var(--type-display-sm-tracking); }
#Subtitle { font: var(--text-meta); letter-spacing: var(--type-meta-tracking); color: var(--ink-muted); }
#Start, #Options, #Quit { width: stretch; }
#Store { width: stretch; height: stretch; }
#Header { width: stretch; align-items: center; gap: var(--space-sm); }
#Heading { font: var(--text-xl); letter-spacing: var(--type-xl-tracking); }
#Engine { width: stretch; color: var(--ink-muted); }
#Coins { font: var(--text-meta); letter-spacing: var(--type-meta-tracking); color: var(--highlight); white-space: nowrap; }
#List { width: stretch; height: stretch; gap: var(--space-xs); }
#Row { width: stretch; padding: var(--space-xs) var(--space-md); align-items: center; background: var(--control); border-radius: var(--radius-control); }
#Name { width: stretch; }
.small, .small::part(base) { height: var(--control-sm); }
.small::part(base) { padding: 0 var(--space-md); }
#Tags { width: stretch; flex-wrap: wrap; gap: var(--space-xs); }
</style>"#;

/// The sample screen's stores at the start: music on, volume 0.6, an empty
/// search, loading at 42 %, [`ITEMS`] store rows (`Row#<n>`, each with its
/// price as the Buy button's tooltip) and eight tags (`Tag#<k>`).
pub fn sample_stores() -> Vec<(&'static str, Value)> {
    let items: Vec<Value> = (0..ITEMS)
        .map(|n| json!({"id": n.to_string(), "name": format!("Item {n}"), "price": format!("{} coins", 100 + 10 * n)}))
        .collect();
    let tags: Vec<Value> = [
        "Sword", "Shield", "Potion", "Bow", "Helmet", "Ring", "Boots", "Map",
    ]
    .iter()
    .enumerate()
    .map(|(k, name)| json!({"id": k.to_string(), "name": name}))
    .collect();
    vec![
        ("music", json!(true)),
        ("volume", json!(0.6)),
        ("search", json!("")),
        ("loading", json!(0.42)),
        ("items", Value::Array(items)),
        ("tags", Value::Array(tags)),
    ]
}

/// Mounts the sample screen ([`SAMPLE_DOC`] with [`sample_stores`]) in
/// `ui`; each frame then builds it ([`step_screen`]).
pub fn sample_screen(ui: &mut Ui) {
    mount_screen_document(ui, "sample", SAMPLE_DOC, &sample_stores());
}

/// The MSDF test font's files (msdf-atlas-gen's PNG, JSON and CSV; the Noto
/// Sans SC subset og-tests commits under `assets/fonts/noto-sans-sc-msdf.*`).
pub struct FontFiles<'a> {
    pub png: &'a [u8],
    pub json: &'a str,
    pub csv: &'a str,
}

/// The MSDF test font's glyph table over `atlas`, and the UI metrics of it
/// with the font's own ascender and descender.
pub fn msdf_font(
    files: &FontFiles,
    atlas: og_draw::TextureHandle,
) -> (og_draw::MsdfFont, FontMetrics) {
    let json: serde_json::Value = serde_json::from_str(files.json).expect("font json parses");
    let number = |a: &str, b: &str| json[a][b].as_f64().unwrap_or_else(|| panic!("{a}.{b}")) as f32;
    let mut glyphs = Vec::new();
    for line in files.csv.lines().filter(|l| !l.trim().is_empty()) {
        let f: Vec<f32> = line
            .split(',')
            .map(|v| v.trim().parse().expect("number"))
            .collect();
        // -yorigin top: plane bounds left, top, right, bottom with y down.
        glyphs.push(og_draw::Glyph {
            key: char::from_u32(f[0] as u32).expect("codepoint").into(),
            advance: f[1],
            plane_bounds: [f[2], -f[5], f[4], -f[3]],
            atlas_bounds: [f[6], f[7], f[8], f[9]],
        });
    }
    let font = og_draw::MsdfFont {
        atlas,
        distance_range: number("atlas", "distanceRange"),
        foreground: false,
        silhouette: false,
        line_height: number("metrics", "lineHeight"),
        glyphs,
        kerning: Vec::new(),
    };
    // yOrigin top: the JSON's ascender is negative, its descender positive.
    let metrics = FontMetrics::from_msdf(&font).with_extents(
        -number("metrics", "ascender"),
        -number("metrics", "descender"),
    );
    (font, metrics)
}

/// A Ui with the test font's metrics (and handle when one is given).
pub fn ui_with_font(font: UiFont) -> Ui {
    Ui::new(UiDesc {
        fonts: Fonts {
            body: font,
            ..Fonts::default()
        },
        ..UiDesc::default()
    })
}

/// A press at `p` (`down`, `pressed`, `released` as given).
fn at(p: Vec2, down: bool, pressed: bool, released: bool) -> InputFrame {
    InputFrame {
        pointer: p,
        pointer_inside: true,
        down,
        pressed,
        released,
        ..InputFrame::default()
    }
}

/// The centre of `ui`'s element `path` in the last frame.
fn center_of(ui: &Ui, path: &str) -> Vec2 {
    let e = ui
        .element(path)
        .unwrap_or_else(|| panic!("no element {path}"));
    let (x, y) = e.center();
    Vec2::new(x, y)
}

/// The sample session in `ui`: mounts the sample screen, focuses the search
/// field and types, scrolls the list a little, then rests the pointer on a
/// Buy button until its tooltip shows. Answers the screen's state after it.
pub fn sample_session(ui: &mut Ui) -> Sample {
    let vp = viewport();
    sample_screen(ui);
    let mut events = step_screen(ui, InputFrame::default(), vp);
    let field = center_of(ui, "Search");
    events.extend(step_screen(ui, at(field, true, true, false), vp));
    events.extend(step_screen(ui, at(field, false, false, true), vp));
    events.extend(step_screen(
        ui,
        InputFrame {
            text: "pot".into(),
            ..InputFrame::hover(field.x, field.y)
        },
        vp,
    ));
    let list = center_of(ui, "List");
    events.extend(step_screen(
        ui,
        InputFrame {
            wheel: Vec2::new(0.0, 22.0),
            ..InputFrame::hover(list.x, list.y)
        },
        vp,
    ));
    let buy = center_of(ui, "Row#2/Buy");
    for _ in 0..40 {
        events.extend(step_screen(ui, InputFrame::hover(buy.x, buy.y), vp));
    }
    Sample::read(ui, clicks(&events))
}

/// The game widget set: tabs, a select and a number input, a radio row,
/// two checkboxes, badges, a rule, and the touch stick and d-pad.
pub const KIT_DOC: &str = r#"<meta name="component" content="kit">
<row id="Screen">
  <panel id="Left">
    <tabs id="Tabs" fill items="Overview Stats Gear" bind:selected={$tab}/>
    <row id="Pickers">
      <select id="Difficulty" bind:value={$pick}>
        <option value="easy">Easy</option>
        <option value="normal">Normal</option>
        <option value="hard">Hard</option>
      </select>
      <input type="number" id="Amount" min="0" max="9" bind:value={$amount}>
    </row>
    <row id="Mode">
      <label><input type="radio" id="Solo" value="solo" bind:group={$mode}> Solo</label>
      <label><input type="radio" id="Coop" value="coop" bind:group={$mode}> Co-op</label>
      <label><input type="radio" id="Versus" value="versus" bind:group={$mode}> Versus</label>
    </row>
    <column id="Options">
      {#each $options as option (option.id)}
        <label><input type="checkbox" id="Option" bind:checked={option.on}> {option.label}</label>
      {/each}
    </column>
    <row id="Chips">
      <badge id="New" tone="accent">New</badge>
      <badge id="Sale" tone="success">-20%</badge>
    </row>
    <hr id="Rule">
  </panel>
  <column id="Right">
    <row id="Touch">
      <joystick id="Stick" action="move"/>
      <dpad id="Dpad" action="move"/>
    </row>
  </column>
</row>
<style>
#Screen { width: stretch; height: stretch; padding: var(--space-lg); gap: var(--space-lg); background: var(--backdrop); }
#Left { width: 380px; height: stretch; }
#Pickers { gap: var(--space-md); align-items: center; }
#Difficulty, #Difficulty::part(trigger) { width: 160px; }
#Mode { gap: var(--space-xxs); flex-wrap: wrap; }
#Chips { gap: var(--space-sm); align-items: center; }
#Right { gap: var(--space-lg); }
#Touch { gap: var(--space-lg); align-items: flex-end; }
</style>"#;

/// The widget set's state: the tab, the select's option (`easy`, `normal`,
/// `hard`), the number, the radio's value (`solo`, `coop`, `versus`) and
/// the two checkboxes.
#[derive(Clone, Debug, PartialEq)]
pub struct Kit {
    pub tab: usize,
    pub pick: String,
    pub amount: f32,
    pub mode: String,
    pub checks: [bool; 2],
}

impl Default for Kit {
    fn default() -> Kit {
        Kit {
            tab: 0,
            pick: "easy".into(),
            amount: 0.0,
            mode: "solo".into(),
            checks: [false; 2],
        }
    }
}

/// Mounts the widget set ([`KIT_DOC`]) in `ui` with `s` as its stores.
pub fn kit_screen(ui: &mut Ui, s: &Kit) {
    let options: Vec<Value> = ["Show hints", "Hard mode"]
        .iter()
        .enumerate()
        .map(|(k, label)| json!({"id": k.to_string(), "label": label, "on": s.checks[k]}))
        .collect();
    mount_screen_document(
        ui,
        "kit",
        KIT_DOC,
        &[
            ("tab", json!(s.tab)),
            ("pick", json!(s.pick)),
            ("amount", json!(s.amount)),
            ("mode", json!(s.mode)),
            ("options", Value::Array(options)),
        ],
    );
}

/// A column of two fields (a double click's word, an undone edit) and a
/// three-row text area, 120 CSS px further down the surface when `low`.
pub const FIELDS_DOC: &str = r#"<meta name="component" content="fields">
<column id="Fields" class:low={$low}>
  <input id="Words" bind:value={$words}>
  <input id="Undo" bind:value={$undo}>
  <textarea id="Notes" rows="3" bind:value={$notes}/>
</column>
<style>
#Fields { width: stretch; padding: var(--space-lg) var(--space-lg) 0; gap: var(--space-sm); }
#Fields.low { padding-top: calc(var(--space-lg) + 120px); }
#Words, #Undo, #Notes { width: 320px; }
</style>"#;

/// The fields of `text-fields-2` and the text they hold.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fields {
    pub words: String,
    pub undo: String,
    pub notes: String,
}

/// Mounts [`FIELDS_DOC`] in `ui` holding `f`, `low` or not.
pub fn fields_screen(ui: &mut Ui, f: &Fields, low: bool) {
    mount_screen_document(
        ui,
        "fields",
        FIELDS_DOC,
        &[
            ("low", json!(low)),
            ("words", json!(f.words)),
            ("undo", json!(f.undo)),
            ("notes", json!(f.notes)),
        ],
    );
}

/// The surface of `text-fields-2`.
pub fn text_fields_viewport() -> Viewport {
    Viewport {
        width: 520.0,
        height: 330.0,
        dpr: 1.5,
        ..Viewport::default()
    }
}

/// The two UIs of `text-fields-2` after their input ([`text_fields_shot`]):
/// the fields' and the area's.
pub fn text_fields_uis(font: &UiFont) -> (Ui, Ui) {
    let vp = text_fields_viewport();
    let end_of = |ui: &Ui, id: &str| {
        let e = ui.element(id).expect(id);
        Vec2::new(e.x + e.width - 4.0, e.y + e.height / 2.0)
    };

    let mut ui = ui_with_font(font.clone());
    fields_screen(
        &mut ui,
        &Fields {
            words: "drag to select words".into(),
            ..Fields::default()
        },
        false,
    );
    step_screen(&mut ui, InputFrame::default(), vp);
    let p = end_of(&ui, "Undo");
    step_screen(&mut ui, at(p, true, true, false), vp);
    step_screen(&mut ui, at(p, false, false, true), vp);
    for c in "one two three".chars() {
        step_screen(
            &mut ui,
            InputFrame {
                text: c.to_string(),
                ..InputFrame::default()
            },
            vp,
        );
    }
    step_screen(
        &mut ui,
        InputFrame {
            keys: vec![UiKey::KeyZ],
            ctrl: true,
            ..InputFrame::default()
        },
        vp,
    );
    // A double click inside "select" (the test face's advances at body size).
    let words = ui.element("Words").expect("words").clone();
    let t = *ui.theme();
    let m = &font.metrics;
    let x = words.x + t.space.md + m.measure("drag to sel", t.type_ramp.body.size);
    let p = Vec2::new(x, words.y + words.height / 2.0);
    for _ in 0..2 {
        step_screen(&mut ui, at(p, true, true, false), vp);
        step_screen(&mut ui, at(p, false, false, true), vp);
    }
    step_screen(&mut ui, InputFrame::default(), vp);

    let mut area = ui_with_font(font.clone());
    fields_screen(
        &mut area,
        &Fields {
            notes: (1..=9)
                .map(|k| format!("Line {k} of the notes"))
                .collect::<Vec<_>>()
                .join("\n"),
            ..Fields::default()
        },
        true,
    );
    step_screen(&mut area, InputFrame::default(), vp);
    let notes = area.element("Notes").expect("notes").clone();
    let start = Vec2::new(notes.x + 30.0, notes.y + 14.0);
    step_screen(&mut area, at(start, true, true, false), vp);
    let below = Vec2::new(notes.x + 120.0, notes.y + notes.height + 30.0);
    for _ in 0..6 {
        step_screen(&mut area, at(below, true, false, false), vp);
    }

    (ui, area)
}

/// The labels of `text-mixed-runs`: lines that mix Latin with CJK, Arabic,
/// Hebrew and Devanagari, each drawn a run per font (`Fonts::link`).
pub const MIXED_LINES: [&str; 5] = [
    "Level 1-2 古い水車小屋",
    "Score 42: مرحبا بك",
    "Player: שלום, नमस्ते!",
    "Found 冒険 and مغامرة in one line",
    "A wrapped paragraph of Latin words with 古い水車小屋 inside it, and مرحبا بك too, broken where it meets the column's edge.",
];

/// The document of `text-mixed-runs`: [`MIXED_LINES`] as texts (`Line#<k>`,
/// the last one `Wrapped` in a 300 px column), a text with a bold Latin run
/// beside CJK and an Arabic run in the primary ink, and a field.
pub const MIXED_DOC: &str = r#"<meta name="component" content="mixed">
<column id="Mixed">
  {#each $lines as line (line.id)}
    <text id="Line">{line.text}</text>
  {/each}
  <text id="Wrapped">{$wrapped}</text>
  <text id="Rich"><b>Rank 3</b> 冒険 <span part="accent">مغامرة</span></text>
  <input id="Name" bind:value={$name}>
</column>
<style>
#Mixed { width: stretch; padding: var(--space-lg); gap: var(--space-sm); }
#Wrapped, #Name { width: 300px; }
#Rich::part(accent) { color: var(--primary); }
</style>"#;

/// The UI of `references/<tier>/text-mixed-runs.png` (og-tests-cooked's `tests/text_gfx.rs`
/// cooks its fonts and draws it at `vp`): [`MIXED_DOC`] with the field
/// holding a mixed name focused, its caret at the end, in `fonts` (a Latin
/// face first, the other scripts' faces as fallbacks).
pub fn mixed_runs_ui(fonts: Fonts, vp: Viewport) -> Ui {
    let mut ui = Ui::new(UiDesc {
        fonts,
        ..UiDesc::default()
    });
    let lines: Vec<Value> = MIXED_LINES[..4]
        .iter()
        .enumerate()
        .map(|(k, text)| json!({"id": k.to_string(), "text": text}))
        .collect();
    mount_screen_document(
        &mut ui,
        "mixed",
        MIXED_DOC,
        &[
            ("lines", Value::Array(lines)),
            ("wrapped", json!(MIXED_LINES[4])),
            ("name", json!("Name 冒険")),
        ],
    );
    step_screen(&mut ui, InputFrame::default(), vp);
    let e = ui.element("Name").expect("name").clone();
    let p = Vec2::new(e.x + e.width - 4.0, e.y + e.height / 2.0);
    step_screen(&mut ui, at(p, true, true, false), vp);
    step_screen(&mut ui, at(p, false, false, true), vp);
    step_screen(&mut ui, InputFrame::default(), vp);
    ui
}

/// The selectable text of `user-select-<width>` (tests/user_select.rs).
pub const USER_SELECT_DOC: &str = r#"<meta name="component" content="sel">
<column id="Doc">
  <p id="Wrap">The quick brown fox jumps over the lazy dog</p>
  <p id="Second">Second paragraph</p>
  <button id="Go">Go</button>
  <p id="Off">Not this one</p>
  <p id="Code">ID-1234</p>
</column>
<style>
#Doc { user-select: text; padding: 16px; gap: 12px; width: 600px; }
#Wrap { width: 80px; }
#Off { user-select: none; }
#Code { user-select: all; }
</style>"#;

/// `text-path-<width>`: an arched title, a badge ringed with text (one
/// ring outside, one inside on the right side) and a wave, each in
/// `-og-text-path` (tests/text_path.rs).
pub const TEXT_PATH_DOC: &str = r#"<meta name="component" content="tp">
<column id="Root">
  <p id="Arch">Legends of the Arch</p>
  <row id="Badges">
    <stack id="Badge"><column id="Seal"/><p id="Outer">GUILD OF MAKERS - EST 2026 -</p><p id="Inner">well met, traveller</p></stack>
    <p id="Wave">riding the wave of words</p>
  </row>
</column>
<style>
#Root { padding: 24px; gap: 16px; }
#Badges { gap: 24px; flex-wrap: wrap; }
#Arch { -og-text-path: path("M0 90 Q170 -20 340 90"); -og-text-path-offset: 50%; text-align: center; font: var(--text-xl); letter-spacing: var(--type-xl-tracking); }
#Badge { width: 180px; height: 180px; }
#Seal { width: 120px; height: 120px; border-radius: 60px; background: #7a2e22; translate: 30px 30px; }
#Outer { -og-text-path: circle(82px); translate: 8px 8px; }
#Inner { -og-text-path: circle(52px); -og-text-path-side: right; -og-text-path-offset: 50%; text-align: center; translate: 38px 38px; }
#Wave { -og-text-path: path("M0 40 C40 0 80 0 120 40 S200 80 240 40"); -og-text-path-offset: 8px; }
</style>"#;
