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
    /// The pack's haptics by scope and event, loaded with its images.
    haptics_table: Rc<og_ui_next::tree::HapticTable>,
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
    /// The render state's textures by Texture handle index, as of this
    /// frame: what the tree's images resolve their sources against (the
    /// tree paints while the render state is borrowed).
    textures: Rc<RefCell<Vec<og_render::TextureHandle>>>,
    materials: Rc<RefCell<super::ui_draw::MaterialAssets>>,
    /// Unscaled UI clock, also advanced when the retained paint is unchanged.
    pub(crate) time: f32,
    /// Each of `textures`' size in texels (0 unknown): `object-*` fits an
    /// image's source by it.
    texture_sizes: Rc<RefCell<Vec<[f32; 2]>>>,
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

mod checkpoint;
mod probe;
mod style;
pub(crate) use checkpoint::{cover, cursor, set_virtual_cursor, sighting, virtual_cursor};
use style::{Resolved, TextureSize};
pub(crate) use style::{
    configured_seeds, durations, glyph_of, pack_images, pack_layer, pack_theme, style_id,
};

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
            haptics_table: Rc::default(),
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
            textures: Rc::default(),
            materials: Rc::default(),
            time: 0.0,
            texture_sizes: Rc::default(),
            generation: 0,
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
        tree.set_haptics(self.haptics_table.clone());
        // An image's source is a Texture handle (`og.load.texture`, a
        // video player's texture).
        let (textures, sizes) = (self.textures.clone(), self.texture_sizes.clone());
        tree.set_textures(Box::new(move |handle| {
            crate::world::assets::index_of(handle)
                .and_then(|i| {
                    let slot = textures.borrow().get(i)?.slot();
                    Some((slot, sizes.borrow().get(i).copied().unwrap_or_default()))
                })
                .unwrap_or_default()
        }));
        let materials = self.materials.clone();
        tree.set_materials(Box::new(move |text| {
            super::ui_draw::resolve_material(&materials.borrow(), text)
        }));
        self.tree = Some(tree);
        // Before the first op batch, so its records are the pack's from
        // its first frame.
        self.pack = style_id(engine);
        self.resolved = None;
        self.resolve(engine);
        // A new arena: the mirror is sent its whole tree again.
        self.a11y.reset();
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

    /// The characters the tree's texts drew that no face has, so an
    /// installed font pack that covers them loads.
    pub(crate) fn take_missing(&mut self) -> Vec<char> {
        self.tree
            .as_mut()
            .map(Tree::take_missing)
            .unwrap_or_default()
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

    /// og.ui.feedback: UI event `event`'s pack sound (falling back along
    /// [`og_ui_next::pack::sound_fallback`]), its pitch at `rise` of its
    /// range when given, and its haptic pattern at the player's strength.
    pub(crate) fn feedback(&mut self, event: &str, rise: Option<f32>) {
        let table = self.sounds.get("");
        let cue = std::iter::successors(Some(event), |ev| og_ui_next::pack::sound_fallback(ev))
            .find_map(|ev| table?.get(ev));
        if let Some(play) = cue.and_then(|c| c.pick(&mut self.feedback_rng, rise)) {
            self.plays.push(play);
        }
        let h_table = self.haptics_table.get("");
        let mut h = std::iter::successors(Some(event), |ev| {
            (*ev != "tick")
                .then(|| og_ui_next::pack::sound_fallback(ev))
                .flatten()
        })
        .find_map(|ev| h_table?.get(ev).copied())
        .unwrap_or_else(|| {
            if matches!(event, "hover" | "tick") {
                og_ui_model::Haptic::NONE
            } else {
                std::iter::successors(Some(event), |ev| og_ui_next::pack::sound_fallback(ev))
                    .map(og_ui_model::event_haptic)
                    .find(|h| h.count > 0)
                    .unwrap_or(og_ui_model::Haptic::NONE)
            }
        });
        let k = self.haptic_strength;
        if h.count > 0 && k > 0.0 && self.haptics.len() < 16 {
            for p in &mut h.pulses {
                p.low *= k;
                p.high *= k;
            }
            self.haptics.push(h);
        }
    }

    /// The haptic patterns og.ui.feedback and the tree's controls played
    /// since the last take, scaled by the player's haptic strength.
    pub(crate) fn take_haptics(&mut self) -> Vec<og_ui_model::Haptic> {
        let mut out = std::mem::take(&mut self.haptics);
        if let Some(t) = self.tree.as_mut() {
            let tree_haptics = t.take_haptics();
            let k = self.haptic_strength;
            if k > 0.0 {
                for mut h in tree_haptics {
                    if out.len() >= 16 {
                        break;
                    }
                    if h.count > 0 {
                        for p in &mut h.pulses {
                            p.low *= k;
                            p.high *= k;
                        }
                        out.push(h);
                    }
                }
            }
        }
        out
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
    /// which motion samples at, and `tick` its number, which `<Particles>`
    /// step to; `emitters` answers a loaded emitter by the handle the game
    /// holds; `now` times the spans only.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn end<'e>(
        &mut self,
        engine: u32,
        (faces, atlas, icons): (
            &mut super::faces::Faces,
            &mut og_render::glyphs::Atlas,
            &og_ui_next::icons::IconAtlas,
        ),
        view: Option<og_ui_next::view::WorldView>,
        (textures, size_of): (&[og_render::TextureHandle], &TextureSize<'_>),
        (time, tick): (f64, u64),
        tps: u32,
        emitters: &dyn Fn(f64) -> Option<&'e og_render::ParticleEmitter>,
        now: &dyn Fn() -> f64,
        world: Option<&dyn Fn(u64) -> Option<og_core::glam::Vec3>>,
        occluded: &mut dyn FnMut(og_core::glam::Vec3, og_core::glam::Vec3, Option<u64>) -> bool,
    ) -> Vec<String> {
        self.time = time as f32;
        let mut warnings = std::mem::take(&mut self.held);
        let settings = settings_of(engine);
        // The game's pack over the defaults and the scheme: a switch
        // resolves every node's classes again ([`Next::resolve`]).
        self.dark = settings.dark;
        self.pack = style_id(engine);
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
        // A texture loaded, replaced or resized in place (a streamed video's
        // starts parked at 1x1 and takes its frame size when it plays):
        // images resolve and fit again.
        let sizes_now: Vec<[f32; 2]> = textures
            .iter()
            .map(|&h| size_of(h).map_or([0.0; 2], |(w, h)| [w as f32, h as f32]))
            .collect();
        if *self.textures.borrow() != textures || *self.texture_sizes.borrow() != sizes_now {
            let mut t = self.textures.borrow_mut();
            t.clear();
            t.extend_from_slice(textures);
            *self.texture_sizes.borrow_mut() = sizes_now;
            tree.repaint_images();
        }
        let v = frame.viewport();
        tree.resize(v.width, v.height);
        let hud = v.hud_insets(og_ui_model::HUD_MAX_ASPECT);
        tree.set_insets([hud.top, hud.right, hud.bottom, hud.left]);
        tree.set_time(time, settings.reduced_motion);
        // The `<Particles>` nodes step to the tick's number, and one that
        // waits for its emitter reads it from the loaded ones
        // (CONVENTIONS.md §Particles).
        tree.set_tick(tick, tps);
        if tree.wants_emitters() {
            for line in tree.resolve_emitters(emitters) {
                warnings.push(format!(
                    "og.ui: a <Particles> emitter's `{line}` does not draw in the UI: only its shape, emission, motion and look over life do (CONVENTIONS.md §Particles)"
                ));
            }
        }
        tree.set_cover(cover, settings.reduced_motion);
        // Glyphs raster at the size they are drawn: the surface's device
        // pixels to one of the tree's units. Texts take the player's text
        // size, and the locale's direction and forms.
        let dpr = if frame.css.dpr > 0.0 {
            frame.css.dpr
        } else {
            1.0
        };
        tree.set_scale(dpr * self.unit);
        tree.set_text_scale(settings.text_scale);
        tree.set_writing(
            crate::bind::i18n::direction(engine),
            &crate::bind::i18n::language(engine),
        );
        // Anchored nodes where the tick's camera sees their entity or point
        // (CONVENTIONS.md §World), and whether drawn geometry stands in
        // front of the ones that ask; none without a camera or a world.
        let (w, h) = (v.width, v.height);
        tree.place_anchors(|target, lift, ask| {
            let (view, world) = (view.as_ref()?, world?);
            let (p, skip) = match target {
                og_ui_next::tree::AnchorTarget::Entity(bits) => (world(bits)?, Some(bits)),
                og_ui_next::tree::AnchorTarget::Point(p) => (p.into(), None),
                // The tree places a node anchored to a UI node itself.
                og_ui_next::tree::AnchorTarget::Node(_) => return None,
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
        // A pointer not over the surface (never come, or gone out) hovers
        // nothing, as a page's after pointerleave: not what lies at its
        // last place, or at the origin before it first moves.
        let (px, py) = if ptr.inside || ptr.pressed || ptr.released {
            (ptr.x, ptr.y)
        } else {
            (f32::MIN, f32::MIN)
        };
        tree.pointer(px, py, ptr.pressed, ptr.released);
        // A right click asks the node under it for its context menu.
        if input.secondary_pressed {
            tree.context_menu(ptr.x, ptr.y);
        }
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
            // Typed text and IME preedit go into the focused `Input` before
            // its keys edit it.
            tree.text_frame(&input.text, input.composition.as_deref());
            // A control a live gameplay map binds ahead of the UI's (arrows
            // that steer, Space that fires, the stick or d-pad that flies) is
            // that map's: it neither moves the UI's focus nor presses the
            // focused control, unless an `Input` is taking text.
            let claimed = |c: &str| map.as_ref().is_some_and(|m| m.claims(c));
            for code in &input.keys_pressed {
                if tree.text_target().is_none() && claimed(code) {
                    continue;
                }
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
            for (bit, nav, control) in [
                (Pad::UP, Nav::Up, "pad.up"),
                (Pad::DOWN, Nav::Down, "pad.down"),
                (Pad::LEFT, Nav::Left, "pad.left"),
                (Pad::RIGHT, Nav::Right, "pad.right"),
            ] {
                if g.pressed & bit != 0 && !claimed(control) {
                    tree.nav(nav);
                }
            }
            let [sx, sy] = pad.stick;
            let dir = if sx.abs().max(sy.abs()) < 0.5 || claimed("pad.leftXY") {
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
        self.font = faces.any();
        let (_, painted) = tree.present(&mut faces.collection, atlas, icons);
        warnings.extend(
            tree.take_warnings()
                .into_iter()
                .map(|w| format!("og.ui: {w}")),
        );
        // The atlas's frame: a plot no text read for long enough empties.
        atlas.advance();
        let t2 = now();
        if painted {
            // The tree paints its items afresh each time: the overlay takes
            // them, and the tree reuses the old ones' capacity.
            // Numbered here: after `widgets::emit` swaps, `self.paint` is
            // the scene's older arena, whose generation is behind.
            self.generation += 1;
            std::mem::swap(&mut self.paint, &mut tree.drawn);
            self.paint.generation = self.generation;
        }
        let allocated = allocations() - a0;
        self.spans.frame = t2 - t0;
        self.spans.commit = t1 - t0;
        self.spans.layout = t2 - t1;
        self.spans.allocations = allocated;
        self.spans.elements = tree.len();
        warnings
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
}

/// The binary, host-neutral retained UI protocol.
pub fn og_engine_ui_patch(world: u32, bytes: &[u8]) -> Vec<u8> {
    match crate::engine::engine(world) {
        // A world: the TS renderer's ops and the game's stores.
        Ok(eng) => {
            let ops = bytes.strip_prefix(og_ui_next::tree::MAGIC);
            if ops.is_some() {
                eng.render
                    .borrow_mut()
                    .ui_next
                    .get_or_insert_with(|| Box::new(Next::new()))
                    .draws = true;
                pack_images(world);
            }
            let (reply, tables) = {
                let mut rs = eng.render.borrow_mut();
                let super::extract::RenderState {
                    ui_next,
                    ui_draw,
                    scene,
                    ..
                } = &mut *rs;
                let next = ui_next.get_or_insert_with(|| Box::new(Next::new()));
                next.sync_materials(ui_draw, scene);
                // The TS renderer's op batches, beside the store's commands.
                match ops {
                    Some(ops) => {
                        let reply = next.ops(world, ops);
                        (reply, next.take_strings())
                    }
                    None => (next.patch(bytes), Vec::new()),
                }
            };
            // Its components' string tables, registered once the render
            // state is free (a table's locale may load its fonts).
            crate::bind::i18n::component_tables(&eng, tables);
            reply
        }
        Err(reason) => og_reactive::reactive::wire::encode(&serde_json::json!({
            "ok": false, "reason": reason
        }))
        .expect("bounded error reply"),
    }
}

impl Next {
    pub(crate) fn sync_materials(
        &mut self,
        draw: &super::ui_draw::UiDrawState,
        scene: &og_render::RenderScene,
    ) {
        let changed = draw.sync_materials(scene, &mut self.materials.borrow_mut());
        if changed
            && let Some(tree) = &mut self.tree
            && let Err(error) = tree.refresh_materials()
        {
            self.held.push(error);
        }
    }
}
