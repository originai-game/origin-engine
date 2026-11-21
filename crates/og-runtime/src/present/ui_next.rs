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
use og_ui_next::pack::{Pack, Scheme};
use og_ui_next::queue;
use og_ui_next::tree::{Resolve, Tree};
use serde_json::{Map, Value, json};
use std::cell::RefCell;
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
    /// What the tree's class resolver was made for: the pack, the scheme
    /// (dark) and the UI box's size class.
    resolved: Option<(Option<String>, bool, SizeClass)>,
    /// The widths the resolver's theme breaks at (CSS px).
    widths: Vec<f32>,
    /// The pack images' texture slots by `src`, and the style they were
    /// loaded for ([`pack_images`]).
    images: BTreeMap<String, u32>,
    images_for: Option<Option<String>>,
    signals: BTreeMap<(String, String), u32>,
    rows: BTreeMap<(String, String), Rows>,
    /// Mounted documents: mount id to (document, inputs).
    mounts: BTreeMap<(String, String), Mount>,
    /// Warnings of a build an op batch made, for the tick's end.
    held: Vec<String>,
    /// What the new core's documents did since the game last took them
    /// (`og.ui.docEvents`).
    pub(crate) events: Vec<og_ui::DocEvent>,
    /// The game's stores: their values, reads and checkpoints.
    pub(crate) store: Session,
    /// What the tree last painted (og-draw's arena), swapped into the
    /// render scene when it is newer (`widgets::emit`); its generation is
    /// counted here, across trees.
    pub paint: og_render::paint::Paint,
    /// The last paint's generation: each paint's is newer than any the
    /// scene holds.
    generation: u64,
    pub spans: Spans,
    /// Whether the body font draws (has a handle in the scene).
    font: bool,
    /// The locale the tree's strings are in.
    language: String,
    /// The accessibility tree the host's mirror was last sent.
    a11y: og_ui_next::a11y::Tree,
    /// A tree was made and has not painted yet.
    fresh: bool,
    /// The render state's textures by Texture handle index, as of this
    /// frame: what the tree's images resolve their sources against (the
    /// tree paints while the render state is borrowed).
    textures: Rc<RefCell<Vec<og_render::TextureHandle>>>,
    /// The frame's run-time glyphs render all at once, not a few a frame
    /// (`glyphs::generate`): a fresh tree painted, so its text arrives
    /// with its panels.
    pub(crate) settle_glyphs: bool,
    /// The on-screen controls' zones, og-ui's documents' and the tree's,
    /// as last handed to the action map, which takes the whole set.
    doc_zones: Vec<og_input::ControlZone>,
    zones: Vec<og_input::ControlZone>,
    /// The device whose bindings the `<Glyph>` nodes show.
    device: Option<og_ui::Device>,
    /// The left stick's held direction and when it moves the focus again.
    stick: Option<(og_ui::Nav, f64)>,
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
    let id = style_id(engine)?;
    let assets = eng.assets.borrow();
    let (bytes, bundle) = assets.blob(&id).ok()?;
    let text = bundle.style_text(&id, bytes.bytes()).ok()?;
    Some((id, serde_json::from_str(&text).ok()?))
}

/// The id of the style `og.ui.configure` set, none for the default.
fn style_id(engine: u32) -> Option<String> {
    let eng = crate::engine::engine(engine).ok()?;
    let handle = eng.state.borrow().ui_desc.get("style")?.as_f64()?;
    let i = crate::world::assets::index_of(handle)?;
    Some(eng.assets.borrow().styles.get(i)?.id.clone())
}

/// Loads the textures of the active pack's nine-slice images (both
/// schemes', over the default pack's) when the pack changed, so the
/// class resolver draws them from the first frame they are named in.
/// Called before the render state is borrowed for the UI (`load_texture`
/// takes it): before an op batch, the tick's end and the warm build.
pub(crate) fn pack_images(engine: u32) {
    let Ok(eng) = crate::engine::engine(engine) else {
        return;
    };
    let pack = style_id(engine);
    match &eng.render.borrow().ui_next {
        Some(next) if next.images_for.as_ref() != Some(&pack) => {}
        _ => return,
    }
    let mut layers = vec![og_ui_next::pack::Layer::default_pack()];
    layers.extend(pack.as_deref().and_then(|id| pack_layer(engine, id)));
    let all = Pack::new(&layers);
    let mut slots = BTreeMap::new();
    let mut warnings = Vec::new();
    for scheme in [Scheme::Light, Scheme::Dark] {
        for image in all.images(scheme).values() {
            if slots.contains_key(&image.src) {
                continue;
            }
            let slot = match crate::world::assets::load_texture(&eng, &image.src) {
                Ok(i) => eng.render.borrow().textures.get(i).map_or(0, |t| t.slot()),
                Err(e) => {
                    warnings.push(format!("og.ui: the pack's image {}: {e}", image.src));
                    0
                }
            };
            slots.insert(image.src.clone(), slot);
        }
    }
    if let Some(next) = &mut eng.render.borrow_mut().ui_next {
        next.images = slots;
        next.images_for = Some(pack);
        next.resolved = None;
        next.held.extend(warnings);
    }
}

/// Style `id`'s pack (its `pack.json`, cooked as an `og.UiPack`), if it
/// has one.
fn pack_layer(engine: u32, id: &str) -> Option<og_ui_next::pack::Layer> {
    let eng = crate::engine::engine(engine).ok()?;
    let assets = eng.assets.borrow();
    let blob = format!("{}/pack.json", id.trim_end_matches('/'));
    let (bytes, bundle) = assets.blob(&blob).ok()?;
    let text = bundle.pack_text(&blob, bytes.bytes()).ok()?;
    og_ui_next::pack::Layer::parse(&text).ok()
}

/// Where a UI box falls among what classes read of it: how many of the
/// theme's widths it reaches, and whether it is under 500 px tall and
/// wider than tall. A box that stays in one resolves every class alike
/// (an arbitrary `min-[…]:` width is not counted).
type SizeClass = (u8, bool, bool);

fn size_class(widths: &[f32], [w, h]: [f32; 2]) -> SizeClass {
    (
        widths.iter().filter(|&&x| w >= x).count() as u8,
        h < 500.0,
        w > h,
    )
}

/// The op nodes' class resolver: the default pack with style `pack`'s
/// over it, its theme in the scheme, for a UI box of `window` CSS px; and
/// the widths that theme breaks at.
fn resolver(
    engine: u32,
    pack: Option<&str>,
    dark: bool,
    window: [f32; 2],
    slots: &BTreeMap<String, u32>,
) -> (Resolve, Vec<f32>) {
    let mut layers = vec![og_ui_next::pack::Layer::default_pack()];
    layers.extend(pack.and_then(|id| pack_layer(engine, id)));
    let scheme = if dark { Scheme::Dark } else { Scheme::Light };
    let pack = Pack::new(&layers);
    // Tailwind's breakpoints (1rem = 16 px), the size classes' 600 and a
    // pack's own.
    let mut widths = vec![600.0, 640.0, 768.0, 1024.0, 1280.0, 1536.0];
    if let Some(Value::Object(b)) = pack.theme(scheme).get("breakpoints") {
        widths.extend(b.values().filter_map(Value::as_f64).map(|w| w as f32));
    }
    // The cook checked the theme: what it left out, it reported then.
    let mut classes = og_ui_next::classes::Classes::new(Some(&pack.theme_json(scheme)))
        .or_else(|_| og_ui_next::classes::Classes::new(None))
        .expect("the default theme reads")
        .0;
    // `packClass` at runtime: `@Component.slot[.variant]` tokens.
    classes.set_variants(pack.variants().clone());
    // `nine-<name>`: the pack's images, their textures loaded ahead
    // ([`pack_images`]).
    classes.set_images(
        pack.images(scheme)
            .iter()
            .map(|(name, image)| {
                let slot = slots.get(&image.src).copied().unwrap_or(0);
                (name.clone(), og_ui_next::classes::nine_slice(image, slot))
            })
            .collect(),
    );
    let classes = RefCell::new(classes);
    let env = og_ui_next::classes::Env {
        window,
        dark,
        ..Default::default()
    };
    // The tree asks for an entry's state records (motion's `starting:`
    // and `exit:`; the input's); the inline style lies over the classes.
    let resolve: Resolve = Box::new(move |class, style, states, data| {
        classes.borrow_mut().record_data(
            class,
            style,
            &og_ui_next::classes::Env { states, ..env },
            data,
        )
    });
    (resolve, widths)
}

impl Next {
    /// Gives the tree the class resolver of the pack, the scheme and the
    /// box's size class, when it has another one: every op node restyles.
    fn resolve(&mut self, engine: u32, ui: &og_ui::Ui) {
        let Some(tree) = &mut self.tree else { return };
        let v = ui.frame().viewport;
        let window = [v.width, v.height];
        let dark = self.scheme == og_ui::Scheme::Dark;
        let at = size_class(&self.widths, window);
        if self
            .resolved
            .as_ref()
            .is_some_and(|(p, d, c)| *p == self.pack && *d == dark && *c == at)
        {
            return;
        }
        let (resolve, widths) = resolver(engine, self.pack.as_deref(), dark, window, &self.images);
        tree.set_resolver(resolve);
        self.resolved = Some((self.pack.clone(), dark, size_class(&widths, window)));
        self.widths = widths;
    }
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
            resolved: None,
            widths: Vec::new(),
            images: BTreeMap::new(),
            images_for: None,
            signals: BTreeMap::new(),
            rows: BTreeMap::new(),
            mounts: BTreeMap::new(),
            held: Vec::new(),
            store: Session::default(),
            events: Vec::new(),
            paint: og_render::paint::Paint::default(),
            spans: Spans::default(),
            font: false,
            device: None,
            stick: None,
            language: String::new(),
            a11y: og_ui_next::a11y::Tree::new(),
            fresh: false,
            textures: Rc::default(),
            generation: 0,
            settle_glyphs: false,
            doc_zones: Vec::new(),
            zones: Vec::new(),
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
    /// The look in the UI's scheme and the game's pack, loaded when it
    /// changed: whether it did.
    fn load_look(&mut self, engine: u32, ui: &og_ui::Ui, now: &dyn Fn() -> f64) -> bool {
        if self.look.is_some() {
            return false;
        }
        let t = now();
        let scheme_name = if self.scheme == og_ui::Scheme::Light {
            "light"
        } else {
            "dark"
        };
        let mut layers = vec![Layer {
            tokens: og_ui_next::look::default_tokens(),
            surfaces: og_ui_next::look::DEFAULT_SURFACES,
        }];
        let pack = active_pack(engine);
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
        true
    }

    /// Makes the tree when there is none: its documents' mounts
    /// instantiated, ready for op batches. Its time and allocations are
    /// the build's.
    fn build(&mut self, engine: u32, ui: &og_ui::Ui, now: &dyn Fn() -> f64) {
        if self.tree.is_some() {
            return;
        }
        self.load_look(engine, ui, now);
        let a0 = allocations();
        let t0 = now();
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
        // An image's source is a Texture handle (`og.load.texture`, a
        // video player's texture).
        let textures = self.textures.clone();
        tree.set_textures(Box::new(move |handle| {
            crate::world::assets::index_of(handle)
                .and_then(|i| textures.borrow().get(i).map(|t| t.slot()))
                .unwrap_or(0)
        }));
        for (doc, inputs, _) in self.mounts.values() {
            let inputs: Vec<(&str, u32)> = inputs.iter().map(|(n, s)| (n.as_str(), *s)).collect();
            if let Err(e) = tree.mount(doc, &inputs) {
                self.held.push(e);
            }
        }
        self.tree = Some(tree);
        // Before the first op batch, so a mount's records are the pack's
        // from its first frame.
        self.pack = active_pack(engine).map(|(id, _)| id);
        self.resolved = None;
        self.resolve(engine, ui);
        // A new arena: the mirror is sent its whole tree again.
        self.a11y.reset();
        self.fresh = true;
        self.spans.build = now() - t0;
        self.spans.build_allocations = allocations() - a0;
    }

    /// An op batch of the TS renderer (CONVENTIONS.md §Ops), applied to
    /// the tree now: the first one makes the tree, so a mount pays for its
    /// build while the game loads and the first frame only commits and
    /// paints. The reply is the events since the last batch, or the
    /// refusal of an op.
    pub(crate) fn ops(&mut self, engine: u32, ui: &og_ui::Ui, bytes: &[u8]) -> Vec<u8> {
        self.scheme = ui.theme().scheme;
        self.build(engine, ui, &crate::author::probe::now_ms);
        let tree = self.tree.as_mut().expect("built above");
        let mut out = Vec::new();
        match tree.apply(bytes) {
            Ok(()) => tree.take_events(&mut out),
            Err(e) => {
                out.extend_from_slice(og_ui_next::tree::REFUSED);
                out.extend_from_slice(e.as_bytes());
            }
        }
        out
    }

    /// The UI's tick is ending: the tree is made again if a mount changed
    /// or the scheme did, then commits, lays out and paints.
    /// `time` is the UI tick's end in seconds, unscaled, which motion
    /// samples at; `now` times the spans only.
    /// Behind the play gate (`loading::warm_tick`): a mounted document's
    /// tree is built, laid out and painted now, its glyphs asked for, so
    /// the first played frame only commits. Only when there is no tree yet:
    /// a live tree and its motion clock are left alone. Its warnings wait
    /// for the tick's end.
    /// The string tables the batches since the last call named, for the
    /// caller to register once it no longer holds the render state.
    /// The characters the tree's texts drew that no face has, with the
    /// body's handle: og-ui's frame's `missing_glyphs` for the
    /// new core, so an installed font pack that covers them loads.
    pub(crate) fn take_missing(
        &mut self,
        ui: &og_ui::Ui,
    ) -> Option<(og_render::FontHandle, Vec<char>)> {
        let chars = self.tree.as_mut()?.take_missing();
        let body = ui.fonts().body.handle?;
        (!chars.is_empty()).then_some((body, chars))
    }

    /// The action map's control zones, og-ui's `docs` (its
    /// `take_control_zones`) and the tree's on-screen controls together:
    /// `None` when neither changed.
    pub(crate) fn control_zones(
        &mut self,
        docs: Option<Vec<og_input::ControlZone>>,
    ) -> Option<Vec<og_input::ControlZone>> {
        let own = match &mut self.tree {
            Some(tree) => tree.take_zones(),
            None => (!self.zones.is_empty()).then(Vec::new),
        };
        if docs.is_none() && own.is_none() {
            return None;
        }
        if let Some(docs) = docs {
            self.doc_zones = docs;
        }
        if let Some(own) = own {
            self.zones = own;
        }
        Some(self.doc_zones.iter().chain(&self.zones).cloned().collect())
    }

    pub(crate) fn take_strings(&mut self) -> Vec<String> {
        self.tree
            .as_mut()
            .map(Tree::take_strings)
            .unwrap_or_default()
    }

    pub(crate) fn warm(
        &mut self,
        engine: u32,
        ui: &og_ui::Ui,
        textures: &[og_render::TextureHandle],
    ) {
        if self.tree.is_some() || self.mounts.is_empty() {
            return;
        }
        let warnings = self.end(
            engine,
            ui,
            textures,
            0.0,
            &crate::author::probe::now_ms,
            None,
        );
        self.held.extend(warnings);
    }

    pub(crate) fn end(
        &mut self,
        engine: u32,
        ui: &og_ui::Ui,
        textures: &[og_render::TextureHandle],
        time: f64,
        now: &dyn Fn() -> f64,
        world: Option<&dyn Fn(u64) -> Option<og_core::glam::Vec3>>,
    ) -> Vec<String> {
        let mut warnings = std::mem::take(&mut self.held);
        let scheme = ui.theme().scheme;
        // A tree of op nodes stays: its texts are the renderer's, and a new
        // look restyles it in place.
        let ops = self.tree.as_ref().is_some_and(|t| t.has_ops());
        // A new locale is a new tree: its bindings read the strings anew.
        let language = crate::bind::i18n::language(engine);
        if language != self.language {
            self.language = language;
            if !ops {
                self.tree = None;
            }
        }
        // The game's pack over the defaults; a switch is another look.
        let pack_id = active_pack(engine).map(|(id, _)| id);
        if scheme != self.scheme || pack_id != self.pack {
            self.scheme = scheme;
            self.pack = pack_id;
            self.look = None;
            if !ops {
                self.tree = None;
            }
        }
        if self.mounts.is_empty() && !ops {
            return warnings;
        }
        if self.load_look(engine, ui, now)
            && let (Some(tree), Some(look)) = (&mut self.tree, &self.look)
        {
            tree.set_look(look.clone());
        }
        self.build(engine, ui, now);
        self.resolve(engine, ui);
        warnings.append(&mut self.held);
        let a0 = allocations();
        let t0 = now();
        let Some(tree) = &mut self.tree else {
            return warnings;
        };
        // A texture loaded or replaced: images resolve again.
        if *self.textures.borrow() != textures {
            let mut t = self.textures.borrow_mut();
            t.clear();
            t.extend_from_slice(textures);
            tree.repaint_images();
        }
        let v = ui.frame().viewport;
        let unit = ui.frame().unit.max(1.0);
        tree.resize(v.width * unit, v.height * unit);
        tree.set_time(time, ui.theme().reduced_motion);
        let commit = tree.commit();
        // Anchored nodes where the tick's camera sees their entity or point
        // (CONVENTIONS.md §World); none without a camera or a world.
        let view = ui.world_view();
        let (w, h) = (v.width * unit, v.height * unit);
        tree.place_anchors(|target, lift| {
            let (view, world) = (view.as_ref()?, world?);
            let p = match target {
                og_ui_next::tree::AnchorTarget::Entity(bits) => world(bits)?,
                og_ui_next::tree::AnchorTarget::Point(p) => p.into(),
            };
            let (x, y, _) = view.project(p + og_core::glam::Vec3::from(lift), w, h)?;
            Some([x, y])
        });
        // The pointer, in the units og-ui's input is in, on the tree's
        // surface: a press and a release over one element click it.
        let input = ui.input();
        let mut done = Vec::new();
        // A finger pans what scrolls and long-presses.
        let device = ui.device();
        tree.set_device(device == og_ui::Device::Touch, device.is_pad());
        tree.pointer(
            input.pointer.x * unit,
            input.pointer.y * unit,
            input.pressed,
            input.released,
            &mut done,
        );
        // Op nodes take the wheel, the keys, the UI's input actions and an
        // assistive technology's requests; the `<Glyph>` nodes follow the
        // device.
        if tree.has_ops() {
            let (x, y) = (input.pointer.x * unit, input.pointer.y * unit);
            tree.wheel(x, y, input.wheel.x * unit, input.wheel.y * unit);
            for code in &input.keys_pressed {
                tree.key(code, input.shift, input.ctrl, input.alt, input.meta);
            }
            for name in ui.fired_actions() {
                tree.action(name);
            }
            for (act, path) in &input.a11y {
                let Some(node) = og_ui_next::a11y::target(path) else {
                    continue;
                };
                match act {
                    og_input::A11yAct::Focus => tree.a11y_focus(node),
                    og_input::A11yAct::Activate => _ = tree.a11y_act(node, None),
                    og_input::A11yAct::Increment => _ = tree.a11y_act(node, Some(true)),
                    og_input::A11yAct::Decrement => _ = tree.a11y_act(node, Some(false)),
                }
            }
            // The d-pad moves the focus on each press; the left stick past
            // half-way on its dominant axis once, then after og-ui's slow
            // duration every fast one while held, as og-ui's navigation.
            let g = &input.gamepad;
            use og_ui::{GamepadFrame as Pad, Nav};
            for (bit, nav) in [
                (Pad::UP, Nav::Up),
                (Pad::DOWN, Nav::Down),
                (Pad::LEFT, Nav::Left),
                (Pad::RIGHT, Nav::Right),
            ] {
                if g.pressed & bit != 0 {
                    tree.nav(nav);
                }
            }
            let s = g.stick;
            let dir = if s.x.abs().max(s.y.abs()) < 0.5 {
                None
            } else if s.x.abs() > s.y.abs() {
                Some(if s.x > 0.0 { Nav::Right } else { Nav::Left })
            } else {
                Some(if s.y > 0.0 { Nav::Down } else { Nav::Up })
            };
            let d = ui.theme().durations_ms;
            self.stick = match (dir, self.stick) {
                (None, _) => None,
                (Some(dir), Some((held, next))) if held == dir => {
                    if time >= next {
                        tree.nav(dir);
                        Some((dir, next + f64::from(d.fast) / 1000.0))
                    } else {
                        Some((dir, next))
                    }
                }
                (Some(dir), _) => {
                    tree.nav(dir);
                    Some((dir, time + f64::from(d.slow) / 1000.0))
                }
            };
            tree.input_done(ui.action_down("confirm"));
            if tree.glyphs_wanted() || self.device != Some(device) {
                self.device = Some(device);
                tree.set_glyphs(&|action| ui.glyph(action).map(|g| g.text));
            }
        }
        // Looked up per event, so a frame without one allocates nothing.
        let mounts = &self.mounts;
        self.events.extend(done.into_iter().map(|e| {
            og_ui::DocEvent {
                mount: mounts
                    .keys()
                    .nth(e.mount as usize)
                    .map_or_else(String::new, |m| m.1.clone()),
                path: e.path.into(),
                row_key: e.row_key.to_string(),
                kind: e.kind.into(),
                name: e.name.into(),
                value: Value::Null,
            }
        }));
        let t1 = now();
        self.font = ui.fonts().body.handle.is_some();
        let (_, painted) = tree.present(ui.fonts(), ui.icons());
        let t2 = now();
        if painted {
            // The tree paints its items afresh each time: the overlay takes
            // them, and the tree reuses the old ones' capacity.
            // Numbered here: after `widgets::emit` swaps, `self.paint` is
            // the scene's older arena, whose generation is behind.
            self.generation += 1;
            std::mem::swap(&mut self.paint, &mut tree.drawn);
            self.paint.generation = self.generation;
            self.settle_glyphs |= std::mem::take(&mut self.fresh);
        }
        let allocated = allocations() - a0;
        self.spans.frame = t2 - t0;
        self.spans.commit = t1 - t0;
        self.spans.layout = t2 - t1;
        self.spans.allocations = allocated;
        self.spans.ops = commit.ops;
        self.spans.refolded = commit.refolded;
        self.spans.elements = tree.len();
        warnings
    }

    /// The accessibility tree's update for the web shell's mirror
    /// ([`og_ui_next::a11y::web`]), when what it shows changed since the
    /// last one; in CSS px, as the tree lays out.
    pub(crate) fn a11y(&mut self) -> Option<Value> {
        let tree = self.tree.as_ref()?;
        let update = tree.a11y(&mut self.a11y, tree.focused(), 1.0)?;
        Some(og_ui_next::a11y::web(&update))
    }

    /// The accessibility update for a native window's AccessKit adapter
    /// (`client`: what that session was sent), at `width` physical pixels
    /// across: the tree's CSS px scaled to them by its root.
    pub(crate) fn a11y_native(
        &self,
        client: &mut og_ui_next::a11y::Tree,
        width: u32,
    ) -> Option<og_ui_next::a11y::accesskit::TreeUpdate> {
        let tree = self.tree.as_ref()?;
        let css = tree.size().0.max(1.0);
        tree.a11y(client, tree.focused(), width as f32 / css)
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
            .chain(Self::op_elements(tree).0)
            .collect()
    }

    /// The TSX renderer's nodes that show, and the focus order as their
    /// paths, with the accessibility tree built whole now (only when the
    /// probe asks). Each is what a game-making agent reads: its op id, its
    /// `path` of op ids (what `touch_move_to` and the captures name), its
    /// box and text; a control's `role`; `disabled`, `checked` and
    /// `focused` among its states; its name only where its text is not it.
    fn op_elements(tree: &Tree) -> (Vec<og_probe::UiElement>, Vec<String>) {
        /// The roles a player operates: what the model may press or move.
        const CONTROLS: &[&str] = &[
            "button",
            "link",
            "checkbox",
            "radio",
            "switch",
            "slider",
            "textbox",
            "combobox",
            "option",
            "tab",
            "spinbutton",
            "menuitem",
            "listbox",
            "grid",
            "tree",
            "radiogroup",
        ];
        const STATES: &[&str] = &["disabled", "checked", "focused"];
        let (facts, order) = tree.a11y_snapshot();
        let ops = tree.dump_ops();
        let parent: std::collections::HashMap<u32, u32> =
            ops.iter().map(|n| (n.id, n.parent)).collect();
        let path = |mut id: u32| {
            let mut ids = vec![id];
            while let Some(&p) = parent.get(&id).filter(|&&p| p != 0) {
                ids.push(p);
                id = p;
            }
            ids.iter()
                .rev()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join("/")
        };
        let by_node: std::collections::HashMap<_, _> = ops.iter().map(|n| (n.node, n.id)).collect();
        let order = order
            .iter()
            .filter_map(|node| by_node.get(node).map(|&id| path(id)))
            .collect();
        let elements = ops
            .iter()
            .filter(|n| n.shown)
            .map(|n| {
                let f = facts.get(&n.node);
                let control = f
                    .is_some_and(|f| CONTROLS.contains(&f.role) || f.states.contains(&"focusable"));
                let [x, y, width, height] = n.rect;
                og_probe::UiElement {
                    id: n.id.to_string(),
                    parent: if n.parent == 0 {
                        String::new()
                    } else {
                        n.parent.to_string()
                    },
                    path: path(n.id),
                    text: n.text.into(),
                    role: f.filter(|_| control).map_or("", |f| f.role).into(),
                    name: f
                        .filter(|f| control && f.name != n.text)
                        .map_or(String::new(), |f| f.name.clone()),
                    states: f.map_or(Vec::new(), |f| {
                        let mut s: Vec<String> = f
                            .states
                            .iter()
                            .filter(|s| STATES.contains(s))
                            .map(|s| s.to_string())
                            .collect();
                        s.dedup();
                        s
                    }),
                    disabled: f.is_some_and(|f| f.states.contains(&"disabled")),
                    x,
                    y,
                    width,
                    height,
                    ..Default::default()
                }
            })
            .collect();
        (elements, order)
    }

    /// The new core's op nodes as the probe lists them, and the focus
    /// order as their paths: for `og ui check`.
    pub(crate) fn op_dump(&self) -> (Vec<og_probe::UiElement>, Vec<String>) {
        self.tree
            .as_ref()
            .map(Self::op_elements)
            .unwrap_or_default()
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

    /// The resolver reads the pack's theme in the UI's scheme and gives the
    /// tree's state records: `bg-surface` is the scheme's surface, and
    /// `starting:` holds only under its bit.
    #[test]
    fn the_resolver_reads_the_scheme_and_the_state_bits() {
        let pack = Pack::new(&[og_ui_next::pack::Layer::default_pack()]);
        for (dark, scheme) in [(false, Scheme::Light), (true, Scheme::Dark)] {
            let (resolve, _) = resolver(u32::MAX, None, dark, [1280.0, 720.0], &BTreeMap::new());
            let hex = pack.theme(scheme)["colors"]["surface"].as_str().unwrap();
            let want = resolve(&format!("bg-[{hex}]"), "", 0, "").skin.fill;
            assert_eq!(
                resolve("bg-surface", "", 0, "").skin.fill,
                want,
                "{scheme:?}"
            );
        }
        let light = pack.theme(Scheme::Light)["colors"]["surface"].clone();
        assert_ne!(light, pack.theme(Scheme::Dark)["colors"]["surface"]);
        let (resolve, _) = resolver(u32::MAX, None, true, [1280.0, 720.0], &BTreeMap::new());
        let starting = og_ui_next::classes::state::STARTING;
        assert_eq!(
            resolve("opacity-50 starting:opacity-0", "", 0, "")
                .skin
                .opacity,
            0.5
        );
        assert_eq!(
            resolve("opacity-50 starting:opacity-0", "", starting, "")
                .skin
                .opacity,
            0.0
        );
        // The op's inline style is over its classes.
        assert_eq!(
            resolve("opacity-50", "opacity: 0.25", 0, "").skin.opacity,
            0.25
        );
    }
}
