//! og-ui's new core behind the runtime's UI entry points, for a game whose
//! project.ogt says `uiCore next` (the ui-rewrite branch; this module goes
//! when the new core replaces og-ui). The game's store operations
//! (`og.ui.scope`) still reach og-ui, which keeps the values, answers the
//! reads and checkpoints them; each is also mirrored into an og-ui-next
//! signal. Its mounts go to the new core alone, so og-ui draws none of
//! them. At the end of the UI's tick the tree commits what the stores
//! changed, lays out and paints; its items are emitted to the overlay
//! beside og-ui's.

use og_ui_next::look::{Layer, Look};
use og_ui_next::queue;
use og_ui_next::tree::Tree;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::rc::Rc;

/// A mirrored keyed collection: its key field and rows in order.
struct Rows {
    key: String,
    rows: Vec<Map<String, Value>>,
}

impl Rows {
    fn at(&self, key: &str) -> Option<usize> {
        self.rows
            .iter()
            .position(|r| r.get(&self.key).map(queue::text_of).as_deref() == Some(key))
    }

    fn value(&self) -> Value {
        Value::Array(self.rows.iter().cloned().map(Value::Object).collect())
    }
}

/// What the probe reads of the last frames (`og.ui.next.*` spans).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Spans {
    /// Loading the look (tokens and surfaces) in the scheme, once per
    /// scheme: the cook's compiled blob replaces it.
    pub look: f64,
    /// The first frame after a mount: compile, instantiate, lay out, paint.
    pub build: f64,
    pub build_allocations: usize,
    /// The last frame: all of it, its commit and its layout and paint.
    pub frame: f64,
    pub commit: f64,
    pub layout: f64,
    pub allocations: usize,
    pub ops: usize,
    pub refolded: usize,
    pub elements: usize,
}

/// A mounted document's name and its inputs, as (prop, signal) pairs.
type Mount = (String, Vec<(String, u32)>);

pub(crate) struct Next {
    scheme: og_ui::Scheme,
    /// The style pack the look lays over the defaults (its id), if any.
    pack: Option<String>,
    look: Option<Rc<Look>>,
    /// The cooked section of the look's documents ([`og_ui_next::look::section`]).
    section: String,
    tree: Option<Tree>,
    signals: BTreeMap<(String, String), u32>,
    rows: BTreeMap<(String, String), Rows>,
    /// Mounted documents: mount id to (document, inputs).
    mounts: BTreeMap<(String, String), Mount>,
    built: bool,
    /// The items the overlay draws, as og-ui's frame carries them.
    pub frame: og_ui::UiFrame,
    pub spans: Spans,
    /// Whether the body font draws (has a handle in the scene).
    font: bool,
    /// The locale the tree's strings are in.
    language: String,
    /// The overlay texts and sprites its last emit added.
    pub emitted: std::cell::Cell<(usize, usize, usize)>,
}

/// Whether engine `world`'s project runs its documents on the new core.
pub(crate) fn wanted(world: u32) -> bool {
    crate::engine::engine(world)
        .ok()
        .is_some_and(|e| e.state.borrow().project.as_ref().is_some_and(|p| p.ui_next))
}

fn source(v: &Value) -> Option<(String, String)> {
    Some((
        v.get("scope")?.as_str()?.to_string(),
        v.get("id")?.as_str()?.to_string(),
    ))
}

/// Document `id`'s section `name` (its style the cook compiled for the
/// new core against one look), when its blob has it.
fn cooked_style(engine: u32, id: &str, name: &str) -> Option<Rc<[u8]>> {
    let eng = crate::engine::engine(engine).ok()?;
    let assets = eng.assets.borrow();
    let (bytes, bundle) = assets.blob(id).ok()?;
    bundle
        .document_section(id, bytes.bytes(), name)
        .map(Rc::from)
}

/// The style pack `og.ui.configure` set (its id and CSS files as the cook
/// stored them), none for the default style.
fn active_pack(engine: u32) -> Option<(String, Vec<(String, String)>)> {
    let eng = crate::engine::engine(engine).ok()?;
    let handle = eng.state.borrow().ui_desc.get("style")?.as_f64()?;
    let i = crate::world::assets::index_of(handle)?;
    let assets = eng.assets.borrow();
    let id = assets.styles.get(i)?.id.clone();
    let (bytes, bundle) = assets.blob(&id).ok()?;
    let text = bundle.style_text(&id, bytes.bytes()).ok()?;
    Some((id, serde_json::from_str(&text).ok()?))
}

fn allocations() -> usize {
    og_core::alloc::thread_allocations()
}

impl Next {
    pub(crate) fn new() -> Next {
        Next {
            scheme: og_ui::Scheme::Dark,
            pack: None,
            look: None,
            section: String::new(),
            tree: None,
            signals: BTreeMap::new(),
            rows: BTreeMap::new(),
            mounts: BTreeMap::new(),
            built: false,
            frame: og_ui::UiFrame::default(),
            spans: Spans::default(),
            font: false,
            language: String::new(),
            emitted: Default::default(),
        }
    }

    fn signal(&mut self, id: (String, String), value: Value) {
        match self.signals.get(&id) {
            Some(&s) => queue::set(s, value),
            None => {
                self.signals.insert(id, queue::signal(value));
            }
        }
    }

    fn rows_changed(&mut self, id: &(String, String)) {
        if let Some(r) = self.rows.get(id) {
            let v = r.value();
            self.signal(id.clone(), v);
        }
    }

    /// Mirrors one store command; a mount or unmount it takes for itself
    /// answers `Some(reply)`, every other command is og-ui's too.
    fn mirror(&mut self, c: &Value) -> Option<Value> {
        let op = c.get("op").and_then(Value::as_str)?;
        let src = c.get("source").and_then(source);
        match (op, src) {
            ("signal", Some(id)) => {
                if !self.signals.contains_key(&id) {
                    let v = c.get("initial").cloned().unwrap_or(Value::Null);
                    self.signal(id, v);
                }
            }
            ("memo", Some(id)) => {
                if !self.signals.contains_key(&id) {
                    let v = c.get("initial").cloned().unwrap_or(Value::Null);
                    self.signal(id, v);
                }
            }
            ("set" | "finish", Some(id)) => {
                let v = c.get("value").cloned().unwrap_or(Value::Null);
                self.signal(id, v);
            }
            ("collection", Some(id)) => {
                if !self.rows.contains_key(&id) {
                    let key = c.get("key").and_then(Value::as_str).unwrap_or("id");
                    let rows = c
                        .get("items")
                        .and_then(Value::as_array)
                        .map(|a| a.iter().filter_map(|r| r.as_object().cloned()).collect())
                        .unwrap_or_default();
                    self.rows.insert(
                        id.clone(),
                        Rows {
                            key: if key.is_empty() {
                                "id".into()
                            } else {
                                key.into()
                            },
                            rows,
                        },
                    );
                    self.rows_changed(&id);
                }
            }
            ("insert" | "remove" | "move" | "patch", Some(id)) => {
                let r = self.rows.get_mut(&id)?;
                let key = c.get("key").map(queue::text_of).unwrap_or_default();
                let before = c.get("before").and_then(Value::as_str).map(str::to_string);
                match op {
                    "insert" => {
                        let row = c.get("row").and_then(Value::as_object).cloned()?;
                        let at = before.and_then(|b| r.at(&b)).unwrap_or(r.rows.len());
                        r.rows.insert(at, row);
                    }
                    "remove" => {
                        if let Some(at) = r.at(&key) {
                            r.rows.remove(at);
                        }
                    }
                    "move" => {
                        if let Some(at) = r.at(&key) {
                            let row = r.rows.remove(at);
                            let to = before.and_then(|b| r.at(&b)).unwrap_or(r.rows.len());
                            r.rows.insert(to, row);
                        }
                    }
                    _ => {
                        let at = r.at(&key)?;
                        let fields = c.get("fields").and_then(Value::as_object)?;
                        let row = &mut r.rows[at];
                        let mut changed = false;
                        for (k, v) in fields {
                            if row.get(k) != Some(v) {
                                row.insert(k.clone(), v.clone());
                                changed = true;
                            }
                        }
                        if !changed {
                            return None;
                        }
                    }
                }
                self.rows_changed(&id);
            }
            ("mount", _) => {
                let mount = c.get("mount").and_then(source)?;
                let document = c.get("document").and_then(Value::as_str)?.to_string();
                let mut inputs = Vec::new();
                if let Some(m) = c.get("inputs").and_then(Value::as_object) {
                    for (name, s) in m {
                        let Some(id) = source(s) else { continue };
                        if !self.signals.contains_key(&id) {
                            self.signal(id.clone(), Value::Null);
                        }
                        inputs.push((name.clone(), self.signals[&id]));
                    }
                }
                self.mounts.insert(mount, (document, inputs));
                self.tree = None;
                return Some(json!({ "ok": true, "value": null }));
            }
            ("unmount", _) => {
                let mount = c.get("mount").and_then(source)?;
                self.mounts.remove(&mount)?;
                self.tree = None;
                return Some(json!({ "ok": true, "value": null }));
            }
            _ => {}
        }
        None
    }

    /// The UI patch batch `bytes`: the store commands mirrored here and
    /// applied by og-ui, the mounts taken here; the replies in order.
    pub(crate) fn patch(&mut self, ui: &mut og_ui::Ui, bytes: &[u8]) -> Vec<u8> {
        let wire = og_ui::reactive::wire::encode;
        let Ok(Value::Array(commands)) = og_ui::reactive::wire::decode(bytes) else {
            return ui.apply_reactive(bytes);
        };
        let mut replies: Vec<Option<Value>> = Vec::with_capacity(commands.len());
        let mut forward = Vec::new();
        for mut c in commands {
            match self.mirror(&c) {
                Some(reply) => replies.push(Some(reply)),
                None => {
                    // og-ui is told only of its own mounts.
                    if c.get("op").and_then(Value::as_str) == Some("registered")
                        && let Some(Value::Array(m)) = c.get_mut("mounts")
                    {
                        m.retain(|s| source(s).is_none_or(|id| !self.mounts.contains_key(&id)));
                    }
                    replies.push(None);
                    forward.push(c);
                }
            }
        }
        let mut answered = match forward.is_empty() {
            true => Vec::new(),
            false => match wire(&Value::Array(forward))
                .map(|b| og_ui::reactive::wire::decode(&ui.apply_reactive(&b)))
            {
                Ok(Ok(Value::Array(a))) => a,
                Ok(Ok(whole)) => return wire(&whole).unwrap_or_default(),
                _ => Vec::new(),
            },
        }
        .into_iter();
        let out: Vec<Value> = replies
            .into_iter()
            .map(|r| r.unwrap_or_else(|| answered.next().unwrap_or(Value::Null)))
            .collect();
        wire(&Value::Array(out)).unwrap_or_default()
    }

    /// The UI's tick is ending: the tree is made again if a mount changed
    /// or the scheme did, then commits, lays out and paints.
    pub(crate) fn end(
        &mut self,
        engine: u32,
        ui: &og_ui::Ui,
        now: &dyn Fn() -> f64,
    ) -> Vec<String> {
        let mut warnings = Vec::new();
        let scheme = ui.theme().scheme;
        // A new locale is a new tree: its bindings read the strings anew.
        let language = crate::bind::i18n::language(engine);
        if language != self.language {
            self.language = language;
            self.tree = None;
        }
        // The game's pack over the defaults; a switch is another look.
        let pack = active_pack(engine);
        let pack_id = pack.as_ref().map(|(id, _)| id.clone());
        if scheme != self.scheme || pack_id != self.pack {
            self.scheme = scheme;
            self.pack = pack_id;
            self.look = None;
            self.tree = None;
        }
        if self.mounts.is_empty() {
            return warnings;
        }
        let scheme_name = if scheme == og_ui::Scheme::Light {
            "light"
        } else {
            "dark"
        };
        if self.look.is_none() {
            let t = now();
            let mut layers = vec![Layer {
                tokens: og_ui_next::look::default_tokens(),
                surfaces: og_ui_next::look::DEFAULT_SURFACES,
            }];
            if let Some(layer) = pack
                .as_ref()
                .and_then(|(_, files)| og_ui_next::look::pack_layer(files))
            {
                layers.push(layer);
            }
            // The cook compiled every document's style against this look
            // and carries its root: no token or sheet is read here.
            let key = Look::key_of(&layers, scheme_name);
            self.section = og_ui_next::look::section(key);
            let root = ui.documents().into_iter().find_map(|n| {
                let doc = ui.shared_document(&n)?;
                let bytes = cooked_style(engine, &doc.source, &self.section)?;
                og_ui_next::cooked::root(&bytes, key)
            });
            self.look = Some(Rc::new(match root {
                Some(steps) => Look::cooked(&layers, scheme_name, steps),
                None => Look::load(&layers, scheme_name),
            }));
            self.spans.look = now() - t;
        }
        let a0 = allocations();
        let t0 = now();
        let building = self.tree.is_none();
        if building {
            let library = ui
                .documents()
                .into_iter()
                .filter_map(|n| ui.shared_document(&n));
            let look = self.look.clone().expect("loaded above");
            let library: Vec<_> = library.collect();
            let mut tree = Tree::new(look, library.iter().cloned());
            for doc in &library {
                if let Some(bytes) = cooked_style(engine, &doc.source, &self.section) {
                    tree.set_cooked(&doc.source, bytes);
                }
            }
            tree.set_strings(
                &self.language,
                Box::new(move |key| crate::bind::i18n::message(engine, key)),
            );
            for (doc, inputs) in self.mounts.values() {
                let inputs: Vec<(&str, u32)> =
                    inputs.iter().map(|(n, s)| (n.as_str(), *s)).collect();
                if let Err(e) = tree.mount(doc, &inputs) {
                    warnings.push(e);
                }
            }
            self.tree = Some(tree);
        }
        let Some(tree) = &mut self.tree else {
            return warnings;
        };
        let v = ui.frame().viewport;
        let unit = ui.frame().unit.max(1.0);
        tree.resize(v.width * unit, v.height * unit);
        let commit = tree.commit();
        let t1 = now();
        self.font = ui.fonts().body.handle.is_some();
        let (_, painted) = tree.present(ui.fonts(), ui.icons());
        let t2 = now();
        if painted {
            // The tree paints its items afresh each time: the overlay takes
            // them, and the tree reuses the old ones' capacity.
            std::mem::swap(&mut self.frame.items, &mut tree.items);
            self.frame.viewport = og_ui::Viewport::new(v.width * unit, v.height * unit);
            self.frame.unit = 1.0;
            self.frame.generation = tree.generation;
        }
        let allocated = allocations() - a0;
        self.spans.frame = t2 - t0;
        self.spans.commit = t1 - t0;
        self.spans.layout = t2 - t1;
        self.spans.allocations = allocated;
        self.spans.ops = commit.ops;
        self.spans.refolded = commit.refolded;
        self.spans.elements = tree.len();
        if building || !self.built {
            self.built = true;
            self.spans.build = t2 - t0;
            self.spans.build_allocations = allocated;
        }
        warnings
    }

    /// The spans as the probe's timings name them.
    pub(crate) fn timings(&self) -> Vec<(String, f64)> {
        let s = &self.spans;
        vec![
            ("og.ui.next.look".into(), s.look),
            ("og.ui.next.build".into(), s.build),
            ("og.ui.next.frame".into(), s.frame),
            ("og.ui.next.commit".into(), s.commit),
            ("og.ui.next.layout".into(), s.layout),
        ]
    }

    /// The new core's elements with a document id, as the probe's UI dump
    /// lists them: id, box on screen and text, from the arena.
    pub(crate) fn elements(&self) -> Vec<og_probe::UiElement> {
        let Some(tree) = &self.tree else { return Vec::new() };
        tree.dump()
            .into_iter()
            .filter(|(_, _, shown, _)| *shown)
            .map(|(id, [x, y, width, height], _, text)| og_probe::UiElement {
                id: id.into(),
                path: id.into(),
                text: text.into(),
                x,
                y,
                width,
                height,
                ..Default::default()
            })
            .collect()
    }

    /// What `__runtime_probe__` reads beside the timings.
    pub(crate) fn probe(&self) -> Value {
        let s = &self.spans;
        let texts: Vec<Value> = self
            .frame
            .items
            .iter()
            .filter_map(|it| match &it.cmd {
                og_ui::DrawCmd::Text {
                    x,
                    baseline,
                    size,
                    text,
                    color,
                    font,
                    clip,
                    bounds,
                    ..
                } => Some(json!({
                    "text": text, "x": x, "baseline": baseline, "size": size, "color": color,
                    "font": format!("{font:?}"), "clip": [clip.x, clip.y, clip.width, clip.height],
                    "bounds": [bounds.x, bounds.y, bounds.width, bounds.height], "level": it.level,
                })),
                _ => None,
            })
            .take(4)
            .collect();
        let mut kinds = serde_json::Map::new();
        for it in &self.frame.items {
            let k = format!("{:?}", it.cmd);
            let k = k.split([' ', '{', '(']).next().unwrap_or("").to_string();
            let n = kinds.get(&k).and_then(Value::as_u64).unwrap_or(0);
            kinds.insert(k, (n + 1).into());
        }
        json!({
            "look_ms": s.look, "build_ms": s.build, "build_allocations": s.build_allocations,
            "frame_ms": s.frame, "commit_ms": s.commit, "layout_ms": s.layout,
            "allocations": s.allocations, "ops": s.ops, "refolded": s.refolded,
            "elements": s.elements, "items": self.frame.items.len(), "mounts": self.mounts.len(),
            "kinds": kinds, "font": self.font, "texts": texts, "emitted": self.emitted.get(),
        })
    }
}
