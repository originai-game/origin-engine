//! `og.ui`'s documents, layer stack and actions (docs/ui.md §8, §9, §10,
//! §16): `og.load.document` reads a cooked `og.UiDocument` into the world's
//! UI; `og.ui.screen`, `component` and `openSlot` mount documents in the
//! frame; `push`, `pop`, `replace` and `clear` manage the layer stack from
//! anywhere; the UI actions, glyphs and the action bar; the probe's `ui:`
//! commands (set, state, play, seek, push, pop, save, clear, action); and
//! the hot reload of an edited document or style in place.
//!
//! A viewmodel's Capitalised root (`Wallet.gold`) reads that resource as
//! of the end of the last tick ([`snapshot`]): the UI is built inside the
//! tick, where the world is the scheduler's.

use super::super::engine::{Engine, engine};
use super::super::{ok, ok_void, parse, refuse};
use super::{BUILDING, LAST, ui_of, with_ui};
use og_ui::{ActionDesc, ActionTrigger, Device, SharedUi, Ui, UiDocument};
use serde_json::{Map, Value, json};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

/// A document an engine loaded: its asset id and its cooked output's hash.
#[derive(Clone, Debug)]
struct Loaded {
    id: String,
    hash: u64,
}

#[derive(Default)]
struct Docs {
    loaded: Vec<Loaded>,
    /// The resources the documents read (their Capitalised roots).
    roots: BTreeSet<String>,
    /// Those resources as of the last tick's end.
    resources: Map<String, Value>,
    /// Styles' cooked hashes, by load index (a hot reload re-applies an edited one).
    styles: BTreeMap<usize, u64>,
}

thread_local! {
    static DOCS: RefCell<BTreeMap<u32, Docs>> = const { RefCell::new(BTreeMap::new()) };
}

fn with<T>(engine: u32, f: impl FnOnce(&mut Docs) -> T) -> T {
    DOCS.with(|d| f(d.borrow_mut().entry(engine).or_default()))
}

/// The host side of a binding: the resources, the language, string
/// tables and textures of one engine.
pub(crate) struct RtEnv {
    engine: u32,
    resources: Map<String, Value>,
    language: String,
}

impl og_ui::bind::Env for RtEnv {
    fn lookup(&self, root: &str) -> Option<Value> {
        self.resources.get(root).cloned()
    }
    fn language(&self) -> &str {
        &self.language
    }
    fn translate(&self, key: &str) -> Option<String> {
        super::super::i18n::message(self.engine, key)
    }
    fn texture(&self, handle: f64) -> Option<og_render::TextureHandle> {
        let eng = engine(self.engine).ok()?;
        let rs = eng.render.try_borrow().ok()?;
        super::super::assets::index_of(handle).and_then(|i| rs.textures.get(i).copied())
    }
}

pub(crate) fn env(engine: u32) -> RtEnv {
    RtEnv {
        engine,
        resources: with(engine, |d| d.resources.clone()),
        language: super::super::i18n::language(engine),
    }
}

fn building() -> Option<u32> {
    BUILDING.with(Cell::get).or_else(|| LAST.with(Cell::get))
}

/// Reads the resources the documents name, after a tick (the world is
/// free here).
pub(crate) fn snapshot(engine: u32, world: &og_ecs::World) {
    let roots = with(engine, |d| d.roots.clone());
    if roots.is_empty() {
        return;
    }
    let mut out = Map::new();
    for r in roots {
        if let Some(v) = og_probe::resource_json(world, &r) {
            out.insert(r, v);
        }
    }
    with(engine, |d| d.resources = out);
}

/// The layers code did not build this tick, built with the engine's
/// bindings (before the frame ends); then the game's time scale follows
/// the pausing layers.
pub(crate) fn end_of_frame(engine: u32, ui: &mut Ui, ctx: &og_ecs::SystemContext<'_>) {
    ui.build_layers(&env(engine));
    if let Some(scale) = ui.pause_time_scale(ctx.time_scale().scale) {
        let _ = ctx.set_time_scale(scale);
    }
}

fn hash_of(eng: &Engine, id: &str) -> u64 {
    let a = eng.assets.borrow();
    a.bundle_for(id)
        .ok()
        .and_then(|(b, _)| b.manifest().outputs().find(|o| o.id == id).map(|o| o.hash))
        .unwrap_or(0)
}

/// Reads, parses and registers a document; answers it.
fn read_document(eng: &Engine, id: &str) -> Result<UiDocument, String> {
    let text = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(id)?;
        bundle
            .document_text(id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    UiDocument::parse(id, &text).map_err(|e| e.join("\n"))
}

fn roots_of(doc: &UiDocument) -> Vec<String> {
    let mut paths = Vec::new();
    for n in &doc.nodes {
        doc.node_paths(n, &mut paths);
    }
    paths
        .into_iter()
        .filter_map(|p| {
            let r = p.split('.').next().unwrap_or("").to_string();
            r.starts_with(|c: char| c.is_ascii_uppercase()).then_some(r)
        })
        .collect()
}

pub(crate) fn load_document(eng: &Engine, path: &str) -> Result<String, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    let doc = read_document(eng, &id)?;
    let name = doc.name.clone();
    let hash = hash_of(eng, &id);
    with(eng.id, |d| {
        d.roots.extend(roots_of(&doc));
        match d.loaded.iter_mut().find(|l| l.id == id) {
            Some(l) => l.hash = hash,
            None => d.loaded.push(Loaded {
                id: id.clone(),
                hash,
            }),
        }
    });
    let ui = eng.state.borrow().ui.clone().ok_or("the world has no UI")?;
    ui.lock().define_document(doc);
    Ok(name)
}

/// `og.load.document`.
pub fn og_load_document(world: u32, path: String) -> String {
    match engine(world).and_then(|eng| load_document(&eng, &path)) {
        Ok(_) => ok_void(),
        Err(e) => refuse(e),
    }
}

/// After a hot reload (`og dev`): every document whose cooked output
/// changed is read again and replaces its definition in place, and every
/// style the UI is configured with that changed is applied again; the
/// UI's state (the layer stack, focus, motion) is kept.
pub(crate) fn refresh(eng: &Engine) -> Vec<String> {
    let mut failed = Vec::new();
    let loaded = with(eng.id, |d| d.loaded.clone());
    let ui = eng.state.borrow().ui.clone();
    let Some(ui) = ui else { return failed };
    for l in loaded {
        let now = hash_of(eng, &l.id);
        if now == l.hash {
            continue;
        }
        match read_document(eng, &l.id) {
            Ok(doc) => {
                with(eng.id, |d| {
                    d.roots.extend(roots_of(&doc));
                    if let Some(x) = d.loaded.iter_mut().find(|x| x.id == l.id) {
                        x.hash = now;
                    }
                });
                ui.lock().define_document(doc);
            }
            Err(e) => failed.push(format!("og dev: {e} (the old document stays)")),
        }
    }
    // Styles: an edited one is parsed again and the UI configured again.
    let styles: Vec<(usize, String)> = eng
        .assets
        .borrow()
        .styles
        .iter()
        .enumerate()
        .map(|(i, s)| (i, s.id.clone()))
        .collect();
    let mut changed = false;
    for (i, id) in styles {
        let now = hash_of(eng, &id);
        let before = with(eng.id, |d| d.styles.insert(i, now));
        if before.is_none_or(|b| b == now) {
            continue;
        }
        let text = {
            let a = eng.assets.borrow();
            a.blob(&id).and_then(|(bytes, bundle)| {
                bundle
                    .style_text(&id, bytes.bytes())
                    .map_err(|e| e.to_string())
            })
        };
        match text.and_then(|t| og_ui::parse_style(&t).map_err(|e| format!("{id}: {e}"))) {
            Ok(mut file) => {
                file.resolve(&id, |r| eng.assets.borrow().bundle_for(r).is_ok());
                eng.assets.borrow_mut().styles[i].file = file;
                changed = true;
            }
            Err(e) => failed.push(format!("og dev: {e} (the old style stays)")),
        }
    }
    if changed {
        let r: Value = serde_json::from_str(&super::og_ui_configure(eng.id, Some("{}".into())))
            .unwrap_or(Value::Null);
        if r["ok"] == json!(false) {
            failed.push(format!("og dev: the style did not apply: {}", r["error"]));
        }
    }
    failed
}

/// The style hashes as loaded (so the first refresh compares against them).
pub(crate) fn note_styles(eng: &Engine) {
    let ids: Vec<(usize, String)> = eng
        .assets
        .borrow()
        .styles
        .iter()
        .enumerate()
        .map(|(i, s)| (i, s.id.clone()))
        .collect();
    for (i, id) in ids {
        let h = hash_of(eng, &id);
        with(eng.id, |d| {
            d.styles.entry(i).or_insert(h);
        });
    }
}

// ── og.ui: documents in the frame ───────────────────────────────────────────

fn response(r: og_ui::DocResponse) -> String {
    json!({
        "path": r.path,
        "events": r.events.iter().map(og_ui::DocEvent::to_json).collect::<Vec<_>>(),
    })
    .to_string()
}

fn vm_of(v: Option<String>) -> Value {
    parse(v.as_deref()).unwrap_or(Value::Null)
}

pub fn og_ui_screen(name: String, vm: Option<String>) -> String {
    let Some(id) = building() else {
        return response(Default::default());
    };
    let env = env(id);
    with_ui("og.ui.screen", |ui| {
        response(ui.screen(&name, vm_of(vm), &env))
    })
    .unwrap_or_else(|| response(Default::default()))
}

pub fn og_ui_component(id: String, name: String, props: Option<String>) -> String {
    let Some(e) = building() else {
        return response(Default::default());
    };
    let env = env(e);
    let mut props = vm_of(props);
    if props.is_null() {
        props = Value::Object(Map::new());
    }
    with_ui("og.ui.component", |ui| {
        let r = ui.component(&id, &name, &mut props, &env);
        let mut v: Value = serde_json::from_str(&response(r)).unwrap_or(Value::Null);
        v["props"] = props;
        v.to_string()
    })
    .unwrap_or_else(|| response(Default::default()))
}

pub fn og_ui_open_slot(path: String) -> bool {
    with_ui("og.ui.openSlot", |ui| ui.open_slot(&path)).unwrap_or(false)
}

// ── og.ui: the layer stack, from anywhere ───────────────────────────────────

fn on_ui<T>(world: u32, f: impl FnOnce(&mut Ui) -> T) -> Result<T, String> {
    let ui: SharedUi = ui_of(world).ok_or("the world has no UI")?;
    let mut g = ui.lock();
    Ok(f(&mut g))
}

pub fn og_ui_push(world: u32, screen: String, vm: Option<String>) -> String {
    match on_ui(world, |ui| ui.push_screen(&screen, vm_of(vm))).and_then(|r| r) {
        Ok(serial) => ok(json!(serial)),
        Err(e) => refuse(e),
    }
}

pub fn og_ui_pop(world: u32) -> String {
    on_ui(world, |ui| ui.pop_screen())
        .ok()
        .flatten()
        .unwrap_or_default()
}

pub fn og_ui_replace(world: u32, screen: String, vm: Option<String>) -> String {
    match on_ui(world, |ui| ui.replace_screen(&screen, vm_of(vm))).and_then(|r| r) {
        Ok(serial) => ok(json!(serial)),
        Err(e) => refuse(e),
    }
}

pub fn og_ui_clear(world: u32) {
    let _ = on_ui(world, |ui| ui.clear_screens());
}

pub fn og_ui_layers(world: u32) -> String {
    on_ui(world, |ui| {
        Value::Array(ui.layer_stack().iter().map(|l| l.to_json()).collect()).to_string()
    })
    .unwrap_or_else(|_| "[]".into())
}

pub fn og_ui_screen_vm(world: u32, screen: String) -> String {
    on_ui(world, |ui| {
        ui.screen_vm(&screen)
            .cloned()
            .unwrap_or(Value::Null)
            .to_string()
    })
    .unwrap_or_else(|_| "null".into())
}

pub fn og_ui_doc_events(world: u32) -> String {
    on_ui(world, |ui| {
        Value::Array(
            ui.take_doc_events()
                .iter()
                .map(og_ui::DocEvent::to_json)
                .collect(),
        )
        .to_string()
    })
    .unwrap_or_else(|_| "[]".into())
}

pub fn og_ui_game_input(world: u32) -> bool {
    on_ui(world, |ui| ui.game_input()).unwrap_or(true)
}

pub fn og_ui_paused(world: u32) -> bool {
    on_ui(world, |ui| ui.paused()).unwrap_or(false)
}

// ── og.ui: actions and glyphs ───────────────────────────────────────────────

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|s| s.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

pub fn og_ui_define_action(world: u32, desc: String) -> String {
    let d = match parse(Some(&desc)) {
        Ok(d) => d,
        Err(e) => return refuse(e),
    };
    let name = d["name"].as_str().unwrap_or_default().to_string();
    if name.is_empty() {
        return refuse("an action has a name");
    }
    let trigger = match d["trigger"].as_str() {
        Some("release") => ActionTrigger::Release,
        Some("hold") => ActionTrigger::Hold(d["holdMs"].as_f64().unwrap_or(500.0) as f32),
        _ => ActionTrigger::Press,
    };
    let desc = ActionDesc {
        name,
        keys: strings(&d["keys"]),
        buttons: strings(&d["buttons"]),
        trigger,
        toggle: d["toggle"].as_bool().unwrap_or(false),
        label: d["label"].as_str().unwrap_or_default().to_string(),
        action: d["action"].as_str().unwrap_or_default().to_string(),
    };
    match on_ui(world, |ui| ui.define_action(desc)) {
        Ok(()) => ok_void(),
        Err(e) => refuse(e),
    }
}

pub fn og_ui_action(name: String) -> bool {
    with_ui("og.ui.action", |ui| ui.action(&name)).unwrap_or(false)
}

pub fn og_ui_action_held(name: String) -> bool {
    with_ui("og.ui.actionHeld", |ui| ui.action_held(&name)).unwrap_or(false)
}

pub fn og_ui_glyph(world: u32, action: String) -> String {
    on_ui(world, |ui| match ui.glyph(&action) {
        Some(g) => {
            json!({ "icon": g.icon, "text": g.text, "device": ui.device().name() }).to_string()
        }
        None => "null".into(),
    })
    .unwrap_or_else(|_| "null".into())
}

pub fn og_ui_frame_glyph(action: String) -> String {
    with_ui("og.ui.frameGlyph", |ui| match ui.glyph(&action) {
        Some(g) => {
            json!({ "icon": g.icon, "text": g.text, "device": ui.device().name() }).to_string()
        }
        None => "null".into(),
    })
    .unwrap_or_else(|| "null".into())
}

pub fn og_ui_device(world: u32) -> String {
    on_ui(world, |ui| ui.device().name().to_string()).unwrap_or_else(|_| "keyboardMouse".into())
}

pub fn og_ui_set_device(world: u32, device: String) {
    if let Some(d) = Device::from_name(&device) {
        let _ = on_ui(world, |ui| {
            if matches!(
                d,
                Device::Xbox | Device::PlayStation | Device::Switch | Device::Gamepad
            ) {
                ui.set_pad_family(d);
            }
            ui.set_device(d);
        });
    }
}

pub fn og_ui_action_bar(world: u32) -> String {
    on_ui(world, |ui| {
        Value::Array(
            ui.action_bar()
                .iter()
                .map(og_ui::ActionHint::to_json)
                .collect(),
        )
        .to_string()
    })
    .unwrap_or_else(|_| "[]".into())
}

pub fn og_ui_document_schema() -> String {
    og_ui::doc::schema::to_json().to_string()
}

// ── live edits (Tune and the probe) ─────────────────────────────────────────

pub fn og_ui_set_prop(world: u32, path: String, prop: String, value: String) -> String {
    match on_ui(world, |ui| ui.set_doc_prop(&path, &prop, &value)).and_then(|r| r) {
        Ok(()) => ok_void(),
        Err(e) => refuse(e),
    }
}

/// Writes the probe's edits into their documents' `.ogt` sources under
/// the project's assets (as `save_scene` writes scenes); answers the files.
pub(crate) fn save_edits(eng: &Engine) -> Result<Vec<String>, String> {
    let ui = eng.state.borrow().ui.clone().ok_or("the world has no UI")?;
    let saved = ui.lock().save_doc_edits()?;
    let root = eng.assets.borrow().root.clone();
    let mut files = Vec::new();
    for (_, source, text) in saved {
        let file = format!("assets/{source}");
        if root.is_empty() {
            return Err(format!(
                "{file}: this host has no project directory to write the document into (og run <project>, og dev)"
            ));
        }
        let path = std::path::Path::new(&root).join(&file);
        std::fs::write(&path, &text).map_err(|e| format!("{}: {e}", path.display()))?;
        files.push(file);
    }
    Ok(files)
}

pub fn og_ui_save_edits(world: u32) -> String {
    match engine(world).and_then(|eng| save_edits(&eng)) {
        Ok(files) => ok(json!(files)),
        Err(e) => refuse(e),
    }
}

// ── the probe's ui: commands ────────────────────────────────────────────────

const USAGE: &str = "ui:set:<path>.<prop>=<value> | ui:state:<path>.<group>=<state> | ui:play:<path>.<timeline> | ui:seek:<path>.<timeline>=<ms> | ui:push:<screen>[:<fixture>] | ui:pop | ui:save | ui:clear[:<path>] | ui:action:<name>";

pub(crate) fn register_commands(probe: &mut og_probe::Probe, engine_id: u32) {
    let _ = probe.register_command(
        "ui",
        og_probe::CommandDesc {
            usage: USAGE.into(),
            doc: "adjust the UI live (docs/ui.md §16.2): set any prop of a document element (presentation only, marked in dump_ui as an edit; ui:save writes the edits into the documents' .ogt), force a state group's state (an empty state lets it go), play or seek a document timeline on an element (seek holds it there), push a screen with a fixture of its document (its first by default), pop the top layer, drop edits, fire a UI action".into(),
        },
        move |_world, args| command(engine_id, args),
    );
}

/// `<path>.<name>`: the last `.` splits them (paths hold no dot).
fn split_target(t: &str) -> Result<(&str, &str), String> {
    t.rsplit_once('.')
        .filter(|(p, n)| !p.is_empty() && !n.is_empty())
        .ok_or_else(|| format!("'{t}': write <path>.<name>; {USAGE}"))
}

fn command(engine_id: u32, args: &[String]) -> Result<(), String> {
    let verb = args.first().map(String::as_str).unwrap_or("");
    let rest = args.get(1..).map(|a| a.join(":")).unwrap_or_default();
    let eng = engine(engine_id)?;
    let ui = eng.state.borrow().ui.clone().ok_or("the world has no UI")?;
    match verb {
        "set" => {
            let (target, value) = rest
                .split_once('=')
                .ok_or_else(|| format!("write ui:set:<path>.<prop>=<value>; {USAGE}"))?;
            let (path, prop) = split_target(target)?;
            ui.lock().set_doc_prop(path, prop, value)
        }
        "state" => {
            let (target, state) = rest
                .split_once('=')
                .ok_or_else(|| format!("write ui:state:<path>.<group>=<state>; {USAGE}"))?;
            let (path, group) = split_target(target)?;
            let state = (!state.is_empty()).then_some(state);
            ui.lock().force_state(path, group, state);
            Ok(())
        }
        "play" => {
            let (path, tl) = split_target(&rest)?;
            ui.lock()
                .play_doc_timeline(path, tl, 0.0, false)
                .map(|_| ())
        }
        "seek" => {
            let (target, ms) = rest
                .split_once('=')
                .ok_or_else(|| format!("write ui:seek:<path>.<timeline>=<ms>; {USAGE}"))?;
            let (path, tl) = split_target(target)?;
            let ms: f32 = ms
                .trim()
                .parse()
                .map_err(|_| format!("'{ms}' is not a time in ms"))?;
            ui.lock().seek_doc_timeline(path, tl, ms).map(|_| ())
        }
        "push" => {
            let (screen, fixture) = match rest.split_once(':') {
                Some((s, f)) => (s.to_string(), f.to_string()),
                None => (rest.clone(), "default".to_string()),
            };
            let mut u = ui.lock();
            let vm = u
                .document(&screen)
                .and_then(|d| d.fixture(&fixture).cloned())
                .unwrap_or(Value::Null);
            if vm.is_null() && fixture != "default" {
                return Err(format!(
                    "the document '{screen}' has no fixture '{fixture}'"
                ));
            }
            u.push_screen(&screen, vm).map(|_| ())
        }
        "pop" => ui
            .lock()
            .pop_screen()
            .map(|_| ())
            .ok_or_else(|| "the layer stack is empty".to_string()),
        "save" => {
            drop(ui);
            save_edits(&eng).map(|_| ())
        }
        "clear" => {
            ui.lock().clear_doc_edits(&rest);
            Ok(())
        }
        "action" => {
            ui.lock().fire_action(&rest);
            Ok(())
        }
        _ => Err(format!("'{verb}' is not a ui command; {USAGE}")),
    }
}
