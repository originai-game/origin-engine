//! og-ui's new core behind the runtime's UI entry points: the TSX
//! renderer's op batches build its tree (`og.ui` ops), and the game's
//! store operations (`og.ui.scope`) go to the store here, a reactive
//! session that keeps the values, answers the reads and goes into
//! checkpoints. At the end of the UI's tick the tree takes the frame's
//! input, lays out and paints into its arena, which the render scene's
//! paint pass draws at the UI's layer (`widgets::emit`).

use crate::host::wire;
use og_reactive::reactive::session::Session;
use og_ui_next::pack::{Pack, Scheme};
use og_ui_next::tree::{Resolve, Tree};
use serde_json::{Map, Value, json};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

/// What the probe reads of the last frames (`og.ui.next.*` spans).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Spans {
    /// Making the tree, at the first op batch.
    pub build: f64,
    pub build_allocations: usize,
    /// The last frame: all of it, its commit and its layout and paint.
    pub frame: f64,
    pub commit: f64,
    pub layout: f64,
    pub allocations: usize,
    pub elements: usize,
}

pub(crate) struct Next {
    /// The UI's scheme is dark (og.ui.configure's, dark by default).
    dark: bool,
    /// The style pack the resolver lays over the defaults (its id), if any.
    pack: Option<String>,
    tree: Option<Tree>,
    /// What the tree's class resolver was made for ([`Next::sized`]): the
    /// pack, the seeds ([`configured_seeds`]), the scheme (dark), high
    /// contrast, the UI box's size class and its exact size when a class
    /// reads it.
    resolved: Option<Resolved>,
    /// Whether a class the resolver resolved reads the window's exact size
    /// (`w-screen`, `h-[16vh]`): then every resize resolves again.
    sized: Rc<std::cell::Cell<bool>>,
    /// The widths the resolver's theme breaks at (CSS px).
    widths: Vec<f32>,
    /// The pack images' texture slots by `src`, and the style and
    /// configured sounds (UiDesc.sounds) they and the sound table were
    /// loaded for ([`pack_images`]).
    images: BTreeMap<String, u32>,
    images_for: Option<(Option<String>, Value)>,
    /// The pack's glyph art (its `icons`), texture slots by og-input's
    /// glyph name (`xbox.a`, `key.keye`), loaded with its images.
    icons: BTreeMap<String, u32>,
    /// A TSX op batch or a document mount reached the core: it draws, so
    /// [`pack_images`] loads the pack's images. A core that only keeps the
    /// game's stores draws nothing and loads none (a game without a UI
    /// cooks no pack images).
    pub(crate) draws: bool,
    /// The screen transition cover a timeline set this tick, taken at its
    /// end ([`cover`]).
    cover: Option<og_ui_next::cover::Cover>,
    /// The virtual cursor og.ui.setVirtualCursor declared this tick,
    /// taken at its end.
    vcursor: Option<og_ui_next::tree::VirtualCursorDesc>,
    /// The pack's sounds with their clips loaded, by scope and event,
    /// loaded with its images ([`pack_images`]).
    sounds: Rc<og_ui_next::tree::SoundTable>,
    /// Warnings of a build an op batch made, for the tick's end.
    held: Vec<String>,
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
    /// The on-screen controls' zones as last handed to the action map.
    zones: Vec<og_input::ControlZone>,
    /// The og-ui unit the tree lays out in (`UiFrame::unit`): its CSS px
    /// are that many of the surface's, as og-ui's are, so `unitScale`
    /// and a large surface enlarge it. What leaves the tree for the
    /// surface (zones, the probe's and the mirror's boxes) is scaled by it.
    unit: f32,
    /// The device whose bindings the `<Glyph>` nodes show, and the
    /// bindings' revision they show ([`og_input::ActionMap::bindings_rev`]).
    device: Option<(og_input::Device, u64)>,
    /// UI actions the probe fired between ticks (`ui:action`): the tree
    /// takes them at the next tick's end.
    pub(crate) fired: Vec<String>,
    /// The left stick's held direction and when it moves the focus again.
    stick: Option<(og_input::frame::Nav, f64)>,
    /// og.ui.feedback's plays and haptic patterns since they were taken,
    /// its picks' generator, and the player's haptic strength
    /// (og.ui.setHaptics, 0 to 1).
    plays: Vec<og_ui_next::tree::Play>,
    haptics: Vec<og_ui_model::Haptic>,
    feedback_rng: u64,
    pub(crate) haptic_strength: f32,
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
pub(crate) fn style_id(engine: u32) -> Option<String> {
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
    {
        let st = eng.state.borrow();
        let given = st.ui_desc.get("sounds").unwrap_or(&Value::Null);
        match &eng.render.borrow().ui_next {
            Some(next)
                if next.draws
                    && next
                        .images_for
                        .as_ref()
                        .is_none_or(|(p, s)| *p != pack || s != given) => {}
            _ => return,
        }
    }
    let given = eng
        .state
        .borrow()
        .ui_desc
        .get("sounds")
        .cloned()
        .unwrap_or(Value::Null);
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
    // Its glyph art: the input prompts' icons, by glyph name.
    let mut icons = BTreeMap::new();
    for (name, src) in &all.icons {
        let slot = *slots.entry(src.clone()).or_insert_with(|| {
            match crate::world::assets::load_texture(&eng, src) {
                Ok(i) => eng.render.borrow().textures.get(i).map_or(0, |t| t.slot()),
                Err(e) => {
                    warnings.push(format!("og.ui: the pack's icon {name} ({src}): {e}"));
                    0
                }
            }
        });
        if slot != 0 {
            icons.insert(name.clone(), slot);
        }
    }
    // Its sounds' clips, each loaded once, as sound handles.
    let mut clips: BTreeMap<String, u64> = BTreeMap::new();
    let mut sounds = og_ui_next::tree::SoundTable::new();
    for (scope, events) in &all.sounds {
        for (event, s) in events {
            let mut handles = Vec::new();
            for clip in &s.clips {
                let h = *clips.entry(clip.clone()).or_insert_with(|| {
                    match crate::world::assets::load_sound(&eng, clip) {
                        Ok(i) => eng
                            .assets
                            .borrow()
                            .sound_handles
                            .get(i)
                            .copied()
                            .unwrap_or(0),
                        Err(e) => {
                            warnings.push(format!("og.ui: the pack's sound {clip}: {e}"));
                            0
                        }
                    }
                });
                if h != 0 {
                    handles.push(h);
                }
            }
            let cue = og_ui_next::tree::Cue {
                clips: handles,
                volume: s.volume,
                pitch: s.pitch,
            };
            sounds
                .entry(scope.clone())
                .or_default()
                .insert(event.clone(), cue);
        }
    }
    // og.ui.configure's sounds win over the pack's own for their events.
    let configured: wire::UiSounds = wire::value(&given, "sounds").unwrap_or_default();
    for (event, handle) in [
        ("hover", configured.hover),
        ("press", configured.press),
        ("open", configured.open),
        ("close", configured.close),
        ("success", configured.success),
        ("error", configured.error),
    ] {
        if let Some(h) = handle.filter(|&h| h != 0) {
            let cue = og_ui_next::tree::Cue {
                clips: vec![h],
                volume: [1.0, 1.0],
                pitch: [1.0, 1.0],
            };
            sounds
                .entry(String::new())
                .or_default()
                .insert(event.into(), cue);
        }
    }
    let sounds = Rc::new(sounds);
    if let Some(next) = &mut eng.render.borrow_mut().ui_next {
        if let Some(tree) = &mut next.tree {
            tree.set_sounds(sounds.clone());
        }
        next.sounds = sounds;
        next.images = slots;
        next.icons = icons;
        // The glyphs draw the new pack's art.
        next.device = None;
        next.images_for = Some((pack, given));
        next.resolved = None;
        next.held.extend(warnings);
    }
}

/// Covers the frame this tick with a timeline's screen transition
/// (presentation only: the tree draws it at the tick's end, over
/// everything).
pub(crate) fn cover(engine: u32, cover: og_ui_next::cover::Cover) {
    let Ok(eng) = crate::engine::engine(engine) else {
        return;
    };
    // A game with no UI mounted has no new core yet: the cover makes it.
    eng.render
        .borrow_mut()
        .ui_next
        .get_or_insert_with(|| Box::new(Next::new()))
        .cover = Some(cover);
}

/// og.ui.setVirtualCursor this tick: the cursor the UI's tick end moves.
pub(crate) fn set_virtual_cursor(engine: u32, desc: Option<og_ui_next::tree::VirtualCursorDesc>) {
    let Ok(eng) = crate::engine::engine(engine) else {
        return;
    };
    eng.render
        .borrow_mut()
        .ui_next
        .get_or_insert_with(|| Box::new(Next::new()))
        .vcursor = desc;
}

/// The virtual cursor as the last tick left it: where (CSS px) and
/// whether it shows; `None` when none is declared.
pub(crate) fn virtual_cursor(engine: u32) -> Option<([f32; 2], bool)> {
    let eng = crate::engine::engine(engine).ok()?;
    let rs = eng.render.borrow();
    let next = rs.ui_next.as_ref()?;
    // The tree's units, in the surface's px.
    let (p, shown) = next.tree.as_ref()?.virtual_cursor_at()?;
    Some((p.map(|v| v * next.unit), shown))
}

/// The cursor the TSX element under the pointer names (`cursor-*`), as
/// the last tick left it; `None` without a tree.
pub(crate) fn cursor(engine: u32) -> Option<og_ui_next::Cursor> {
    let eng = crate::engine::engine(engine).ok()?;
    let rs = eng.render.borrow();
    Some(rs.ui_next.as_ref()?.tree.as_ref()?.cursor())
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

/// The style pack's theme (the default pack with the game's over it) in
/// the scheme and contrast given, as og.ui.theme reads it: the theme the
/// class resolver styles with.
pub(crate) fn pack_theme(
    engine: u32,
    dark: bool,
    high_contrast: bool,
) -> serde_json::Map<String, Value> {
    let pack = Pack::new(&layers(engine, style_id(engine).as_deref()));
    let scheme = if dark { Scheme::Dark } else { Scheme::Light };
    let text = if high_contrast {
        pack.theme_json_high_contrast(scheme)
    } else {
        pack.theme_json(scheme)
    };
    serde_json::from_str(&text).unwrap_or_default()
}

/// The design's motion durations (`--motion-duration-*`) by name, in
/// milliseconds: read once.
pub(crate) fn durations() -> Rc<BTreeMap<String, f32>> {
    thread_local! {
        static READ: Rc<BTreeMap<String, f32>> = Rc::new(og_ui_next::pack::design::durations());
    }
    READ.with(Rc::clone)
}

/// The pack layers the UI styles with: the default pack, style `id`'s
/// pack over it, and the seeds og.ui.configure was given (a look's, a
/// faction's) over both, so the palette derives from them.
fn layers(engine: u32, id: Option<&str>) -> Vec<og_ui_next::pack::Layer> {
    let mut layers = vec![og_ui_next::pack::Layer::default_pack()];
    layers.extend(id.and_then(|id| pack_layer(engine, id)));
    let seeds = configured_seeds(engine);
    if !seeds.is_empty()
        && let Ok(layer) = og_ui_next::pack::Layer::parse(&seeds)
    {
        layers.push(layer);
    }
    layers
}

/// og.ui.configure's seeds as a pack layer (`{ "seeds": { … } }`, each
/// seed `#rrggbb`); empty when it gave none.
pub(crate) fn configured_seeds(engine: u32) -> String {
    let Ok(eng) = crate::engine::engine(engine) else {
        return String::new();
    };
    let st = eng.state.borrow();
    let Some(Value::Object(given)) = st.ui_desc.get("seeds") else {
        return String::new();
    };
    let channel = |c: &Value, k: &str| {
        crate::bind::modules::srgb8(c.get(k).and_then(Value::as_f64).unwrap_or(0.0) as f32)
    };
    let seeds: Map<String, Value> = given
        .iter()
        .filter(|(name, c)| og_ui_next::pack::SEEDS.contains(&name.as_str()) && c.is_object())
        .map(|(name, c)| {
            let rgb = [channel(c, "r"), channel(c, "g"), channel(c, "b")];
            (
                name.clone(),
                json!(og_core::color::hex_string([rgb[0], rgb[1], rgb[2], 255])),
            )
        })
        .collect();
    if seeds.is_empty() {
        return String::new();
    }
    json!({ "seeds": seeds }).to_string()
}

/// Where a UI box falls among what classes read of it: how many of the
/// theme's widths it reaches, and whether it is under 500 px tall and
/// wider than tall. A box that stays in one resolves every class alike
/// (an arbitrary `min-[…]:` width is not counted).
type SizeClass = (u8, bool, bool);

/// What a class resolver was made for: the pack, the dark scheme, high
/// contrast, the size class, and the window when a class it resolved
/// reads the window's exact size.
type Resolved = (
    Option<String>,
    String,
    bool,
    bool,
    SizeClass,
    Option<[f32; 2]>,
);

fn size_class(widths: &[f32], [w, h]: [f32; 2]) -> SizeClass {
    (
        widths.iter().filter(|&&x| w >= x).count() as u8,
        h < 500.0,
        w > h,
    )
}

/// The op nodes' class resolver: the default pack with style `pack`'s
/// and the configured seeds over it ([`layers`]), its theme in the
/// scheme, for a UI box of `window` CSS px; and the widths that theme
/// breaks at.
fn resolver(
    engine: u32,
    pack: Option<&str>,
    dark: bool,
    high_contrast: bool,
    window: [f32; 2],
    slots: &BTreeMap<String, u32>,
    sized: Rc<std::cell::Cell<bool>>,
) -> (Resolve, Vec<f32>) {
    let scheme = if dark { Scheme::Dark } else { Scheme::Light };
    let pack = Pack::new(&layers(engine, pack));
    // Tailwind's breakpoints (1rem = 16 px), the size classes' 600 and a
    // pack's own.
    let mut widths = vec![600.0, 640.0, 768.0, 1024.0, 1280.0, 1536.0];
    if let Some(Value::Object(b)) = pack.theme(scheme).get("breakpoints") {
        widths.extend(b.values().filter_map(Value::as_f64).map(|w| w as f32));
    }
    // The cook checked the theme: what it left out, it reported then.
    let theme = if high_contrast {
        pack.theme_json_high_contrast(scheme)
    } else {
        pack.theme_json(scheme)
    };
    let mut classes = og_ui_next::classes::Classes::new(Some(&theme))
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
        let mut c = classes.borrow_mut();
        let record = c.record_data(
            class,
            style,
            &og_ui_next::classes::Env { states, ..env },
            data,
        );
        sized.set(c.sized());
        record
    });
    (resolve, widths)
}

impl Next {
    /// Gives the tree the class resolver of the pack, the scheme and the
    /// box's size class, when it has another one: every op node restyles.
    fn resolve(&mut self, engine: u32) {
        let Some(tree) = &mut self.tree else { return };
        let v = frame_of(engine).viewport();
        let window = [v.width, v.height];
        let dark = self.dark;
        // The page's `prefers-contrast: more`, as the host reports it.
        let high = v.high_contrast;
        let at = size_class(&self.widths, window);
        let exact = self.sized.get().then_some(window);
        let seeds = configured_seeds(engine);
        if self.resolved.as_ref().is_some_and(|(p, s, d, h, c, w)| {
            *p == self.pack && *s == seeds && *d == dark && *h == high && *c == at && *w == exact
        }) {
            return;
        }
        let sized = Rc::new(std::cell::Cell::new(false));
        let (resolve, widths) = resolver(
            engine,
            self.pack.as_deref(),
            dark,
            high,
            window,
            &self.images,
            sized.clone(),
        );
        tree.set_resolver(resolve);
        self.sized = sized;
        self.resolved = Some((
            self.pack.clone(),
            seeds,
            dark,
            high,
            size_class(&widths, window),
            exact,
        ));
        self.widths = widths;
    }
}

/// Action `name`'s glyph on `device`: its first binding there; on touch,
/// where it has none, its label under the `touch.<name>` icon.
pub(crate) fn glyph_of(
    map: &og_input::ActionMap,
    name: &str,
    device: og_input::Device,
) -> Option<og_input::Glyph> {
    let parts = map.glyphs(name, Some(device)).unwrap_or_default();
    og_input::combined_glyph(&parts, device).or_else(|| {
        let label = map
            .infos()
            .into_iter()
            .find(|a| a.desc.name == name)?
            .desc
            .label;
        (device == og_input::Device::Touch).then(|| og_input::Glyph {
            icon: format!("touch.{name}"),
            text: label,
        })
    })
}

/// Engine `engine`'s UI frame, as og.ui.begin recorded it last; the
/// default without the engine.
fn frame_of(engine: u32) -> crate::bind::modules::UiFrame {
    crate::engine::engine(engine).map_or_else(
        |_| Default::default(),
        |eng| eng.state.borrow().ui_frame.clone(),
    )
}

/// Engine `engine`'s UI settings under the frame's viewport; the default
/// desc's when the engine is gone.
fn settings_of(engine: u32) -> crate::bind::modules::UiSettings {
    match crate::engine::engine(engine) {
        Ok(eng) => {
            let f = eng.state.borrow().ui_frame.clone();
            crate::bind::modules::settings(&eng, &f.css, f.unit)
        }
        Err(_) => crate::bind::modules::UiSettings::of(&Value::Null, String::new(), (false, false)),
    }
}

fn allocations() -> usize {
    og_core::alloc::thread_allocations()
}

impl Next {
    pub(crate) fn new() -> Next {
        Next {
            dark: true,
            pack: None,
            tree: None,
            resolved: None,
            sized: Rc::default(),
            widths: Vec::new(),
            images: BTreeMap::new(),
            images_for: None,
            icons: BTreeMap::new(),
            draws: false,
            cover: None,
            vcursor: None,
            sounds: Rc::default(),
            held: Vec::new(),
            store: Session::default(),
            paint: og_render::paint::Paint::default(),
            spans: Spans::default(),
            font: false,
            device: None,
            stick: None,
            fired: Vec::new(),
            plays: Vec::new(),
            haptics: Vec::new(),
            feedback_rng: 0,
            haptic_strength: 1.0,
            a11y: og_ui_next::a11y::Tree::new(),
            fresh: false,
            textures: Rc::default(),
            generation: 0,
            settle_glyphs: false,
            zones: Vec::new(),
            unit: 1.0,
        }
    }

    /// The UI patch batch `bytes`, applied to the store: the replies in
    /// order.
    pub(crate) fn patch(&mut self, bytes: &[u8]) -> Vec<u8> {
        self.store.apply(bytes)
    }

    /// Makes the tree when there is none: its documents' mounts
    /// instantiated, ready for op batches. Its time and allocations are
    /// the build's.
    fn build(&mut self, engine: u32, now: &dyn Fn() -> f64) {
        if self.tree.is_some() {
            return;
        }
        let a0 = allocations();
        let t0 = now();
        let mut tree = Tree::new();
        tree.set_sounds(self.sounds.clone());
        // An image's source is a Texture handle (`og.load.texture`, a
        // video player's texture).
        let textures = self.textures.clone();
        tree.set_textures(Box::new(move |handle| {
            crate::world::assets::index_of(handle)
                .and_then(|i| textures.borrow().get(i).map(|t| t.slot()))
                .unwrap_or(0)
        }));
        self.tree = Some(tree);
        // Before the first op batch, so its records are the pack's from
        // its first frame.
        self.pack = active_pack(engine).map(|(id, _)| id);
        self.resolved = None;
        self.resolve(engine);
        // A new arena: the mirror is sent its whole tree again.
        self.a11y.reset();
        self.fresh = true;
        self.spans.build = now() - t0;
        self.spans.build_allocations = allocations() - a0;
    }

    /// An op batch of the TS renderer (CONVENTIONS.md §Ops), applied to
    /// the tree now: the first one makes the tree, so a mount pays for its
    /// build while the game loads and the first frame only lays out and
    /// paints. The reply is the events since the last batch, or the
    /// refusal of an op.
    pub(crate) fn ops(&mut self, engine: u32, bytes: &[u8]) -> Vec<u8> {
        self.dark = settings_of(engine).dark;
        self.build(engine, &crate::author::probe::now_ms);
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

    /// The characters the tree's texts drew that no face has, with the
    /// body's handle, so an installed font pack that covers them loads.
    pub(crate) fn take_missing(
        &mut self,
        fonts: &og_ui_next::fonts::Fonts,
    ) -> Option<(og_render::FontHandle, Vec<char>)> {
        let chars = self.tree.as_mut()?.take_missing();
        let body = fonts.body.handle?;
        (!chars.is_empty()).then_some((body, chars))
    }

    /// The action map's control zones, the tree's on-screen controls:
    /// `None` when they did not change.
    pub(crate) fn control_zones(&mut self) -> Option<Vec<og_input::ControlZone>> {
        let own = match &mut self.tree {
            Some(tree) => tree.take_zones(),
            None => (!self.zones.is_empty()).then(Vec::new),
        }?;
        let u = self.unit;
        self.zones = own
            .into_iter()
            .map(|z| og_input::ControlZone {
                x: z.x * u,
                y: z.y * u,
                width: z.width * u,
                height: z.height * u,
                ..z
            })
            .collect();
        Some(self.zones.clone())
    }

    /// What og-input read from the zones last handed out (its
    /// `zone_states`): the tree's controls show their press and knob.
    pub(crate) fn zone_states<'a>(
        &mut self,
        states: impl ExactSizeIterator<Item = &'a og_input::ZoneState>,
    ) {
        if let Some(tree) = &mut self.tree {
            tree.zone_states(states, self.unit);
        }
    }

    /// The sounds the tree's controls and og.ui.feedback played this
    /// frame (the pack's).
    pub(crate) fn take_sounds(&mut self) -> Vec<og_ui_next::tree::Play> {
        let mut out = std::mem::take(&mut self.plays);
        if let Some(t) = self.tree.as_mut() {
            out.extend(t.take_sounds());
        }
        out
    }

    /// og.ui.feedback: UI event `event`'s pack sound (the pack-wide one;
    /// confirm and back fall back to press), its pitch at `rise` of its
    /// range when given, and its haptic pattern at the player's strength.
    pub(crate) fn feedback(&mut self, event: &str, rise: Option<f32>) {
        let table = self.sounds.get("");
        let fallback = matches!(event, "confirm" | "back").then_some("press");
        let cue = [Some(event), fallback]
            .into_iter()
            .flatten()
            .find_map(|ev| table?.get(ev));
        if let Some(play) = cue.and_then(|c| c.pick(&mut self.feedback_rng, rise)) {
            self.plays.push(play);
        }
        let mut h = og_ui_model::event_haptic(event);
        let k = self.haptic_strength;
        if h.count > 0 && k > 0.0 && self.haptics.len() < 16 {
            for p in &mut h.pulses {
                p.low *= k;
                p.high *= k;
            }
            self.haptics.push(h);
        }
    }

    /// The haptic patterns og.ui.feedback played since the last take.
    pub(crate) fn take_haptics(&mut self) -> Vec<og_ui_model::Haptic> {
        std::mem::take(&mut self.haptics)
    }

    /// The string tables the batches since the last call named, for the
    /// caller to register once it no longer holds the render state.
    pub(crate) fn take_strings(&mut self) -> Vec<String> {
        self.tree
            .as_mut()
            .map(Tree::take_strings)
            .unwrap_or_default()
    }

    /// The UI's tick is ending: the tree takes the frame's input, lays
    /// out and paints. `time` is the UI tick's end in seconds, unscaled,
    /// which motion samples at; `now` times the spans only.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn end(
        &mut self,
        engine: u32,
        (fonts, icons): (&og_ui_next::fonts::Fonts, &og_ui_next::icons::IconAtlas),
        view: Option<og_ui_next::view::WorldView>,
        textures: &[og_render::TextureHandle],
        time: f64,
        tps: u32,
        now: &dyn Fn() -> f64,
        world: Option<&dyn Fn(u64) -> Option<og_core::glam::Vec3>>,
        occluded: &mut dyn FnMut(og_core::glam::Vec3, og_core::glam::Vec3, Option<u64>) -> bool,
    ) -> Vec<String> {
        let mut warnings = std::mem::take(&mut self.held);
        let settings = settings_of(engine);
        // The game's pack over the defaults and the scheme: a switch
        // resolves every node's classes again ([`Next::resolve`]).
        self.dark = settings.dark;
        self.pack = active_pack(engine).map(|(id, _)| id);
        // A timeline's transition covers the frame with no UI mounted too:
        // an empty tree draws it.
        let cover = self.cover.take();
        if cover.is_none() && self.tree.is_none() {
            return warnings;
        }
        self.build(engine, now);
        self.resolve(engine);
        warnings.append(&mut self.held);
        let a0 = allocations();
        let t0 = now();
        // The viewport and the input are in og-ui units already (the UI
        // frame's): the tree lays out in them.
        let frame = frame_of(engine);
        self.unit = frame.unit.max(1.0);
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
        let v = frame.viewport();
        tree.resize(v.width, v.height);
        let hud = v.hud_insets(og_ui_model::HUD_MAX_ASPECT);
        tree.set_insets([hud.top, hud.right, hud.bottom, hud.left]);
        tree.set_time(time, settings.reduced_motion);
        tree.set_cover(cover, settings.reduced_motion);
        // Anchored nodes where the tick's camera sees their entity or point
        // (CONVENTIONS.md §World), and whether drawn geometry stands in
        // front of the ones that ask; none without a camera or a world.
        let (w, h) = (v.width, v.height);
        tree.place_anchors(|target, lift, ask| {
            let (view, world) = (view.as_ref()?, world?);
            let (p, skip) = match target {
                og_ui_next::tree::AnchorTarget::Entity(bits) => (world(bits)?, Some(bits)),
                og_ui_next::tree::AnchorTarget::Point(p) => (p.into(), None),
            };
            let p = p + og_core::glam::Vec3::from(lift);
            let mut seen = sighting(view, p, w, h);
            if ask && !seen.behind {
                // From the eye, or along the view axis for an orthographic one.
                let from = if view.orthographic {
                    p - view.forward * seen.depth
                } else {
                    view.eye
                };
                seen.occluded = occluded(from, p, skip);
            }
            Some(seen)
        });
        // The pointer, in the units og-ui's input is in, on the tree's
        // surface: a press and a release over one element click it.
        let input = &frame.input;
        // og-input's actions this tick: the UI's built-in ones (`ui`
        // context) and the game's, the device used last and the bindings'
        // revision.
        let map = crate::engine::engine(engine)
            .ok()
            .and_then(|eng| eng.state.borrow().actions.clone());
        let map = map.as_ref().map(og_input::actions::lock);
        let device = map
            .as_ref()
            .map_or(og_input::Device::KeyboardMouse, |m| m.device());
        let rev = map.as_ref().map_or(0, |m| m.bindings_rev());
        let state = |name: &str| map.as_ref().and_then(|m| m.state(name)).unwrap_or_default();
        let mut fired: Vec<String> = std::mem::take(&mut self.fired);
        for a in og_input::ui_actions() {
            if state(&a.name).pressed && !fired.contains(&a.name) {
                fired.push(a.name);
            }
        }
        // A finger pans what scrolls and long-presses.
        tree.set_device(device == og_input::Device::Touch, device.is_pad());
        // A pad's virtual cursor turns pad 0 into this pointer while it
        // shows (its A the button, its right stick the wheel).
        let g = &input.gamepad;
        let a = og_input::frame::GamepadFrame::A;
        let mut pad = og_ui_next::tree::CursorPad {
            stick: [g.stick.x, g.stick.y],
            stick_r: [g.stick_r.x, g.stick_r.y],
            a_pressed: g.pressed & a != 0,
            a_held: g.held & a != 0,
        };
        let mut ptr = og_ui_next::tree::CursorPointer {
            x: input.pointer.x,
            y: input.pointer.y,
            inside: input.pointer_inside,
            pressed: input.pressed,
            released: input.released,
            wheel: [input.wheel.x, input.wheel.y],
        };
        let a_was = pad.a_pressed || pad.a_held;
        tree.virtual_cursor(self.vcursor.take(), &mut pad, &mut ptr, tps);
        // A press the cursor took confirms nothing at the focus.
        let a_taken = a_was && !(pad.a_pressed || pad.a_held);
        tree.pointer(ptr.x, ptr.y, ptr.pressed, ptr.released);
        // Every finger: pinches, turns and swipes on the elements they
        // went down on.
        if tree.has_ops() {
            tree.touches(&input.pointers);
        }
        // Op nodes take the wheel, the keys, the UI's input actions and an
        // assistive technology's requests; the `<Glyph>` nodes follow the
        // device.
        if tree.has_ops() {
            tree.wheel(ptr.x, ptr.y, ptr.wheel[0], ptr.wheel[1]);
            // Typed text goes into the focused `Input` before its keys
            // edit it, as they arrived.
            if !input.text.is_empty() {
                tree.type_text(&input.text);
            }
            for code in &input.keys_pressed {
                tree.key(code, input.shift, input.ctrl, input.alt, input.meta);
            }
            for name in &fired {
                if !(a_taken && name == "confirm") {
                    tree.action(name);
                }
            }
            for (act, path) in &input.a11y {
                let Some(node) = og_ui_next::a11y::target(path) else {
                    continue;
                };
                match act {
                    og_input::A11yAct::Focus => tree.a11y_focus(node),
                    // A reader's activation lands where its cursor is: the
                    // focus moves there, as a press on it moves it.
                    og_input::A11yAct::Activate => {
                        tree.a11y_focus(node);
                        _ = tree.a11y_act(node, None);
                    }
                    og_input::A11yAct::Increment => _ = tree.a11y_act(node, Some(true)),
                    og_input::A11yAct::Decrement => _ = tree.a11y_act(node, Some(false)),
                }
            }
            // The d-pad moves the focus on each press; the left stick past
            // half-way on its dominant axis once, then after og-ui's slow
            // duration every fast one while held, as og-ui's navigation.
            let g = &input.gamepad;
            use og_input::frame::{GamepadFrame as Pad, Nav};
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
            let [sx, sy] = pad.stick;
            let dir = if sx.abs().max(sy.abs()) < 0.5 {
                None
            } else if sx.abs() > sy.abs() {
                Some(if sx > 0.0 { Nav::Right } else { Nav::Left })
            } else {
                Some(if sy > 0.0 { Nav::Down } else { Nav::Up })
            };
            let d = durations();
            let ms = |name: &str, or: f32| d.get(name).copied().unwrap_or(or);
            let (fast, slow) = (ms("fast", 180.0), ms("slow", 400.0));
            self.stick = match (dir, self.stick) {
                (None, _) => None,
                (Some(dir), Some((held, next))) if held == dir => {
                    if time >= next {
                        tree.nav(dir);
                        Some((dir, next + f64::from(fast) / 1000.0))
                    } else {
                        Some((dir, next))
                    }
                }
                (Some(dir), _) => {
                    tree.nav(dir);
                    Some((dir, time + f64::from(slow) / 1000.0))
                }
            };
            tree.input_done(state("confirm").down && !a_taken);
            // A rebind shows on the next frame, as a device change does:
            // og-input's bindings revision moves with every rebind.
            let shown = (device, rev);
            if tree.glyphs_wanted() || self.device != Some(shown) {
                self.device = Some(shown);
                let icons = &self.icons;
                tree.set_glyphs(&|action| {
                    let g = map.as_ref().and_then(|m| glyph_of(m, action, device))?;
                    Some((g.text, icons.get(&g.icon).copied().unwrap_or(0)))
                });
            }
        }
        let t1 = now();
        self.font = fonts.body.handle.is_some();
        let (_, painted) = tree.present(fonts, icons);
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
        self.spans.elements = tree.len();
        warnings
    }

    /// On a frame drawn between ticks: the anchored nodes the tick showed
    /// move to where `project` (the drawn camera, the targets' drawn
    /// places) puts them, and the tree repaints when one moved
    /// (CONVENTIONS.md §World). The next tick's end places them again.
    pub(crate) fn follow_anchors(
        &mut self,
        (fonts, icons): (&og_ui_next::fonts::Fonts, &og_ui_next::icons::IconAtlas),
        project: impl FnMut(
            og_ui_next::tree::AnchorTarget,
            [f32; 3],
            bool,
        ) -> Option<og_ui_next::tree::Sighting>,
    ) {
        let Some(tree) = self.tree.as_mut() else {
            return;
        };
        if tree.anchor_targets().next().is_none() {
            return;
        }
        tree.follow_anchors(project);
        let (_, painted) = tree.present(fonts, icons);
        if painted {
            self.generation += 1;
            std::mem::swap(&mut self.paint, &mut tree.drawn);
            self.paint.generation = self.generation;
        }
    }

    /// The UI events the pack gives a pack-wide sound (`hover`, `press`…).
    pub(crate) fn pack_sound_events(&self) -> Vec<String> {
        self.sounds
            .get("")
            .map(|events| events.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// The surface the tree lays out on, CSS px: what its anchors project
    /// onto.
    pub(crate) fn surface(&self) -> Option<(f32, f32)> {
        self.tree.as_ref().map(|t| t.size())
    }

    /// The accessibility tree's update for the web shell's mirror
    /// ([`og_ui_next::a11y::web`]), when what it shows changed since the
    /// last one; in CSS px, as the tree lays out.
    pub(crate) fn a11y(&mut self) -> Option<Value> {
        let tree = self.tree.as_ref()?;
        let update = tree.a11y(&mut self.a11y, tree.focused(), self.unit)?;
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
            ("og.ui.next.build".into(), s.build),
            ("og.ui.next.frame".into(), s.frame),
            ("og.ui.next.commit".into(), s.commit),
            ("og.ui.next.layout".into(), s.layout),
        ]
    }

    /// The TSX renderer's nodes, as the probe's UI dump lists them.
    pub(crate) fn elements(&self) -> Vec<og_probe::UiElement> {
        self.tree
            .as_ref()
            .map(|tree| Self::op_elements(tree, self.unit).0)
            .unwrap_or_default()
    }

    /// The TSX renderer's nodes that show, and the focus order as their
    /// paths, with the accessibility tree built whole now (only when the
    /// probe asks). Each is what a game-making agent reads: its op id, its
    /// `path` of op ids (what `touch_move_to` and the captures name), its
    /// box and text; a control's `role`; `disabled`, `checked` and
    /// `focused` among its states; its name only where its text is not it.
    fn op_elements(tree: &Tree, unit: f32) -> (Vec<og_probe::UiElement>, Vec<String>) {
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
            // What a scroller or a cut box hides entirely is not on screen.
            .filter(|n| n.shown && n.visible[2] > n.visible[0] && n.visible[3] > n.visible[1])
            .map(|n| {
                let f = facts.get(&n.node);
                let control = f
                    .is_some_and(|f| CONTROLS.contains(&f.role) || f.states.contains(&"focusable"));
                // In the surface's CSS px, as og-ui's elements are.
                // The part that shows; clipped when that is not all of it.
                let [x0, y0, x1, y1] = n.visible;
                let clipped = [x0, y0, x1 - x0, y1 - y0] != n.rect;
                let [x, y, width, height] = [x0, y0, x1 - x0, y1 - y0].map(|v| v * unit);
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
                    clipped,
                    x,
                    y,
                    width,
                    height,
                    detail: {
                        let mut d = serde_json::Map::new();
                        // A word its lines cut where UAX #14 has no break.
                        if !n.broken.is_empty() {
                            d.insert("brokenWords".into(), json!(n.broken));
                        }
                        // Its text as drawn, for reading its ground in a
                        // frame: the ink as sRGB 0-255 with alpha 0-1, the
                        // font size and each line's box, in CSS px, before
                        // its transform.
                        if !n.lines.is_empty() {
                            let [r, g, b, a] = n.ink;
                            let srgb = og_core::color::linear_to_srgb8;
                            d.insert("ink".into(), json!([srgb(r), srgb(g), srgb(b), a]));
                            d.insert("fontSize".into(), json!(n.size * unit));
                            let lines: Vec<[f32; 4]> =
                                n.lines.iter().map(|l| l.map(|v| v * unit)).collect();
                            d.insert("textLines".into(), json!(lines));
                            // A transformed text's lines are boxes through
                            // this matrix (the quad it draws in).
                            if let Some([a, b, c, d2, tx, ty]) = n.transform {
                                d.insert(
                                    "textTransform".into(),
                                    json!([a, b, c, d2, tx * unit, ty * unit]),
                                );
                            }
                        }
                        d.into_iter().collect()
                    },
                    ..Default::default()
                }
            })
            .collect();
        (elements, order)
    }

    /// A TSX node's box on the UI's surface, in CSS px, as last laid out
    /// (the inspector's panel): the tree's og-ui units times the unit.
    #[cfg(not(feature = "publish"))]
    pub(crate) fn op_box(&self, id: u32) -> Option<[f32; 4]> {
        let b = self.tree.as_ref()?.op_box(id)?;
        Some(b.map(|v| v * self.unit))
    }

    /// The new core's op nodes as the probe lists them, and the focus
    /// order as their paths: for `og ui check`.
    pub(crate) fn op_dump(&self) -> (Vec<og_probe::UiElement>, Vec<String>) {
        self.tree
            .as_ref()
            .map(|tree| Self::op_elements(tree, self.unit))
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
            "build_ms": s.build, "build_allocations": s.build_allocations,
            "frame_ms": s.frame, "commit_ms": s.commit, "layout_ms": s.layout,
            "allocations": s.allocations,
            "elements": s.elements, "quads": p.quads.len(), "gradients": p.gradients.len(),
            "kinds": kinds, "font": self.font, "glyphs": glyphs,
            "generation": p.generation,
        })
    }
}

/// What `view` sees of world point `p` on a `w`×`h` CSS px surface, for an
/// anchored TSX node (og-ui-next CONVENTIONS.md §World): its surface point
/// and depth, or behind the eye a point well off the surface the way the
/// camera would turn to it, where an edge-clamped node pins.
pub(crate) fn sighting(
    view: &og_ui_next::view::WorldView,
    p: og_core::glam::Vec3,
    w: f32,
    h: f32,
) -> og_ui_next::tree::Sighting {
    match view.project(p, w, h) {
        Some((x, y, depth)) => og_ui_next::tree::Sighting {
            at: [x, y],
            depth,
            ..Default::default()
        },
        None => {
            let d = view.direction(p, w, h) * (w + h);
            og_ui_next::tree::Sighting {
                at: [w / 2.0 + d.x, h / 2.0 + d.y],
                behind: true,
                ..Default::default()
            }
        }
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
        let saved = UiCheckpoint::save(Some(&next.store));
        let text = serde_json::to_vec(&saved).unwrap();
        let saved: UiCheckpoint = serde_json::from_slice(&text).unwrap();
        saved.validate(None).unwrap();
        patch(&mut next, set(12));
        assert_eq!(read(&mut next), json!(12));
        saved.restore(Some(&mut next.store)).unwrap();
        assert_eq!(read(&mut next), json!(7));
    }

    /// The resolver reads the pack's theme in the UI's scheme and gives the
    /// tree's state records: `bg-surface` is the scheme's surface, and
    /// `starting:` holds only under its bit.
    #[test]
    fn the_resolver_reads_the_scheme_and_the_state_bits() {
        let pack = Pack::new(&[og_ui_next::pack::Layer::default_pack()]);
        for (dark, scheme) in [(false, Scheme::Light), (true, Scheme::Dark)] {
            let (resolve, _) = resolver(
                u32::MAX,
                None,
                dark,
                false,
                [1280.0, 720.0],
                &BTreeMap::new(),
                Rc::default(),
            );
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
        // High contrast: a tile ground takes the pack's high-contrast tile.
        let (high, _) = resolver(
            u32::MAX,
            None,
            true,
            true,
            [1280.0, 720.0],
            &BTreeMap::new(),
            Rc::default(),
        );
        let theme: serde_json::Value =
            serde_json::from_str(&pack.theme_json_high_contrast(Scheme::Dark)).unwrap();
        let tile = theme["colors"]["tile"].as_str().unwrap();
        assert_eq!(
            high("bg-tile", "", 0, "").skin.fill,
            high(&format!("bg-[{tile}]"), "", 0, "").skin.fill
        );
        let (resolve, _) = resolver(
            u32::MAX,
            None,
            true,
            false,
            [1280.0, 720.0],
            &BTreeMap::new(),
            Rc::default(),
        );
        assert_ne!(
            resolve("bg-tile", "", 0, "").skin.fill,
            high("bg-tile", "", 0, "").skin.fill
        );
        // A class that reads the window's exact size says so, for the
        // resolver to be made again at every resize; a breakpoint does not.
        let sized = Rc::new(std::cell::Cell::new(false));
        let (by_size, _) = resolver(
            u32::MAX,
            None,
            true,
            false,
            [1280.0, 720.0],
            &BTreeMap::new(),
            sized.clone(),
        );
        by_size("bg-tile md:p-4", "", 0, "");
        assert!(!sized.get());
        by_size("h-[16vh]", "", 0, "");
        assert!(sized.get());
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
