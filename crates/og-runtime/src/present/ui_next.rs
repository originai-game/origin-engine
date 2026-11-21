//! og-ui's new core behind the runtime's UI entry points: every game's
//! documents run on it (this module goes when the new core replaces
//! og-ui). The game's store operations
//! (`og.ui.scope`) go to the store here, a reactive session that keeps
//! the values, answers the reads and goes into checkpoints; each is also
//! mirrored into an og-ui-next signal. Its mounts go to the new core
//! alone, so og-ui draws none of them. At the end of the UI's tick the tree commits what the stores
//! changed, lays out and paints into its arena, which the render scene's
//! paint pass draws at the UI's layer (`widgets::emit`).

use og_reactive::reactive::session::{Command, Session};
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

/// A mounted document's name, its inputs as (prop, signal) pairs, and
/// their sources as the mount named them.
type Mount = (String, Vec<(String, u32)>, Vec<Value>);

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
    /// The game's stores: their values, reads and checkpoints.
    pub(crate) store: Session,
    /// What the tree last painted (og-draw's arena), swapped into the
    /// render scene when it is newer (`widgets::emit`); its generation is
    /// counted here, across trees.
    pub paint: og_render::paint::Paint,
    pub spans: Spans,
    /// Whether the body font draws (has a handle in the scene).
    font: bool,
    /// The locale the tree's strings are in.
    language: String,
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
            store: Session::default(),
            paint: og_render::paint::Paint::default(),
            spans: Spans::default(),
            font: false,
            language: String::new(),
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
                let mut sources = Vec::new();
                if let Some(m) = c.get("inputs").and_then(Value::as_object) {
                    for (name, s) in m {
                        let Some(id) = source(s) else { continue };
                        if !self.signals.contains_key(&id) {
                            self.signal(id.clone(), Value::Null);
                        }
                        inputs.push((name.clone(), self.signals[&id]));
                        sources.push(s.clone());
                    }
                }
                self.mounts.insert(mount, (document, inputs, sources));
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

    /// The UI patch batch `bytes`: the mounts taken here, every other
    /// command applied to the store (and mirrored); the replies in order.
    pub(crate) fn patch(&mut self, bytes: &[u8]) -> Vec<u8> {
        let wire = og_reactive::reactive::wire::encode;
        let Ok(Value::Array(commands)) = og_reactive::reactive::wire::decode(bytes) else {
            return self.store.apply(bytes);
        };
        let mut replies: Vec<Option<Value>> = Vec::with_capacity(commands.len());
        let mut forward = Vec::new();
        for c in commands {
            let reply = self.mirror(&c);
            if reply.is_none() {
                forward.push(c);
            }
            replies.push(reply);
        }
        let mut answered = match forward.is_empty() {
            true => Vec::new(),
            false => match wire(&Value::Array(forward))
                .map(|b| og_reactive::reactive::wire::decode(&self.answer(&b)))
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

    /// The store's replies to a batch of commands the mounts did not take.
    fn answer(&mut self, bytes: &[u8]) -> Vec<u8> {
        Session::apply_with(bytes, |command| self.command(command))
    }

    /// One store command: the mounts' own are answered here, the rest by
    /// the store.
    fn command(&mut self, command: Command) -> Result<Value, String> {
        match command {
            Command::MountInputs => Ok(Value::Array(
                self.mounts
                    .iter()
                    .map(|((scope, id), (_, _, sources))| {
                        json!({ "mount": { "scope": scope, "id": id }, "inputs": sources })
                    })
                    .collect(),
            )),
            Command::Mount { mount, .. } | Command::Unmount { mount } => Err(format!(
                "og.ui: '{}:{}' is not a document mount of this UI",
                mount.scope, mount.id
            )),
            // The mounts are this module's; the store keeps the sources.
            Command::Registered { sources, .. } => self.store.execute(Command::Registered {
                sources,
                mounts: Vec::new(),
            }),
            Command::Dispose { scope } => {
                let before = self.mounts.len();
                self.mounts.retain(|(s, _), _| *s != scope);
                if self.mounts.len() != before {
                    self.tree = None;
                }
                self.store.execute(Command::Dispose { scope })
            }
            command => self.store.execute(command),
        }
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
            for (doc, inputs, _) in self.mounts.values() {
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
            let generation = self.paint.generation + 1;
            std::mem::swap(&mut self.paint, &mut tree.drawn);
            self.paint.generation = generation;
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
        let Some(tree) = &self.tree else {
            return Vec::new();
        };
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
        let p = &self.paint;
        let mut kinds = [0usize; 4];
        for q in &p.quads {
            kinds[(q.kind as usize).min(3)] += 1;
        }
        // The first glyphs' places, as the paint pass reads them.
        let glyphs: Vec<Value> = p
            .quads
            .iter()
            .filter(|q| q.kind == og_render::paint::PaintKind::Glyph as u32)
            .take(4)
            .map(|q| json!({
                "char": char::from_u32(q.a).map(String::from), "x": q.rect[0], "baseline": q.rect[1],
                "size": q.rect[2], "color": q.color, "clip": q.clip, "font": q.b,
            }))
            .collect();
        let kinds =
            json!({ "solid": kinds[0], "gradient": kinds[1], "glyph": kinds[2], "icon": kinds[3] });
        json!({
            "look_ms": s.look, "build_ms": s.build, "build_allocations": s.build_allocations,
            "frame_ms": s.frame, "commit_ms": s.commit, "layout_ms": s.layout,
            "allocations": s.allocations, "ops": s.ops, "refolded": s.refolded,
            "elements": s.elements, "quads": p.quads.len(), "gradients": p.gradients.len(),
            "mounts": self.mounts.len(), "kinds": kinds, "font": self.font, "glyphs": glyphs,
            "generation": p.generation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::present::presentation::UiCheckpoint;

    fn patch(next: &mut Next, commands: Value) -> Value {
        let bytes = og_reactive::reactive::wire::encode(&commands).unwrap();
        og_reactive::reactive::wire::decode(&next.patch(&bytes)).unwrap()
    }

    fn read(next: &mut Next) -> Value {
        let src = json!({ "scope": "hud", "id": "gold" });
        patch(next, json!([{ "op": "read", "source": src }]))[0]["value"]["value"].clone()
    }

    /// The new core's stores answer reads and go into a checkpoint, which a
    /// save's text carries and a restore brings back.
    #[test]
    fn checkpoints_carry_the_new_cores_stores() {
        let mut next = Next::new();
        let src = json!({ "scope": "hud", "id": "gold" });
        let set = |v: i64| json!([{ "op": "set", "source": src, "value": v, "force": false }]);
        patch(
            &mut next,
            json!([{ "op": "signal", "source": src, "initial": 3 }]),
        );
        patch(&mut next, set(7));
        assert_eq!(read(&mut next), json!(7));
        let ui = og_ui::SharedUi::new(og_ui::Ui::default());
        let saved = UiCheckpoint::save(&ui, Some(&next.store));
        let text = serde_json::to_vec(&saved).unwrap();
        let saved: UiCheckpoint = serde_json::from_slice(&text).unwrap();
        saved.validate().unwrap();
        patch(&mut next, set(12));
        assert_eq!(read(&mut next), json!(12));
        saved.restore(&ui, Some(&mut next.store)).unwrap();
        assert_eq!(read(&mut next), json!(7));
    }
}
