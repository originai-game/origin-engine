//! The stage states `Ui` is made of (#41): each owns what one stage of
//! the frame writes.

use super::*;

/// Configuration: the theme, fonts, viewport and locale every stage reads.
#[derive(Clone, Debug)]
pub struct Env {
    /// The theme as configured ([`Ui::set_theme`]); `theme` is it with the
    /// viewport's preferences applied.
    pub(crate) configured: Theme,
    pub(crate) theme: Theme,
    /// Font inputs before role backing and linking add derived faces and caches.
    pub(crate) configured_fonts: Fonts,
    pub(crate) fonts: Fonts,
    pub(crate) ticks_per_second: u32,
    pub(crate) viewport: Viewport,
    /// The og-ui unit of the frame being built, CSS px (`begin`).
    pub(crate) unit: f32,
    /// The writing direction: right to left mirrors the layout and turns
    /// text alignment round (og.i18n's locale, or og.ui.configure).
    pub(crate) direction: og_i18n::Direction,
    /// Active BCP-47 locale, for display text casing.
    pub(crate) locale: String,
    pub(crate) icons: IconAtlas,
    /// The style's icon names, which its colour tokens' icon cues index
    /// (`Ui::set_cue_icons`); empty: the atlas's own names.
    pub(crate) cue_icons: Vec<String>,
    /// Pictures rich text draws by name (`[img=…]`).
    pub(crate) images: std::collections::BTreeMap<String, InlineImage>,
    /// The configured style's name (empty: the default style).
    pub(crate) style_name: String,
    /// The player's haptic strength (0: off, 1: as the style sets it).
    pub(crate) haptics: f32,
    /// The widest the HUD band is, over its height ([`Ui::hud_area`]).
    pub(crate) hud_max_aspect: f32,
    /// The host opens a file picker for `<input type="file">`
    /// ([`Ui::set_file_picker`]).
    pub(crate) file_picker: bool,
}

/// The fields of configuration the documents (og-ui) reach.
impl Env {
    pub fn theme(&self) -> &Theme {
        &self.theme
    }
    pub fn fonts(&self) -> &Fonts {
        &self.fonts
    }
    pub fn ticks_per_second(&self) -> u32 {
        self.ticks_per_second
    }
    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }
    pub fn unit(&self) -> f32 {
        self.unit
    }
    pub fn direction(&self) -> og_i18n::Direction {
        self.direction
    }
    pub fn locale(&self) -> &String {
        &self.locale
    }
    pub fn icons(&self) -> &IconAtlas {
        &self.icons
    }
    pub fn style_name(&self) -> &String {
        &self.style_name
    }
    pub fn hud_max_aspect(&self) -> f32 {
        self.hud_max_aspect
    }
    pub fn file_picker(&self) -> bool {
        self.file_picker
    }
}

/// The frame's output: what the host takes after it (the frame, warnings, events, feedback, clipboard) and its counts.
#[derive(Clone, Debug)]
pub struct Out {
    pub(crate) frame: UiFrame,
    pub(crate) warnings: Vec<String>,
    /// The (face, character) pairs [`UiFrame::missing_glyphs`] named.
    pub(crate) glyphs_missed: std::collections::BTreeSet<(og_draw::FontHandle, char)>,
    /// This frame's motion events (into `UiFrame::events`).
    pub(crate) events: Vec<UiEvent>,
    /// UI events the style gives a sound or a haptic, not yet taken by the
    /// host ([`Ui::take_feedback`]), each with where it is in a rising
    /// cascade (0–1; NaN: a random pitch in the event's range).
    pub(crate) feedback: Vec<(crate::tokens::SoundEvent, f32, crate::tokens::Feedback)>,
    /// The haptic patterns played since the host last took them
    /// ([`Ui::take_haptics`]).
    pub(crate) haptic_out: Vec<crate::tokens::Haptic>,
    /// Text a copy or cut took, for the host's clipboard ([`Ui::take_clipboard`]).
    pub(crate) clipboard: Option<String>,
    /// This frame's counts (into `UiFrame::stats`).
    pub(crate) stats: LayoutStats,
}

/// The fields of the frame's output the documents (og-ui) reach.
impl Out {
    pub fn frame(&self) -> &UiFrame {
        &self.frame
    }
    pub fn warnings(&self) -> &Vec<String> {
        &self.warnings
    }
    pub fn warnings_mut(&mut self) -> &mut Vec<String> {
        &mut self.warnings
    }
}

/// Documents: those defined, the layer stack, the containers they query
/// and what building them keeps.
#[derive(Clone, Debug, Default)]
pub struct Docs {
    /// The documents defined and the assets they name (docs/ui.md §2.1).
    pub(crate) library: crate::doc::build::library::Library,
    /// The layer stack (docs/ui.md §2.1).
    pub(crate) layers: crate::layer_stack::LayerStack,
    /// The query containers documents read.
    pub(crate) containers: crate::doc::build::containers::Containers,
    /// The documents' own state: their builds, outputs and memory, and the
    /// reactive session.
    pub(crate) state: crate::doc::build::state::DocState,
}

/// The fields of the documents' state the documents (og-ui) reach.
impl Docs {
    pub fn state(&self) -> &crate::doc::build::state::DocState {
        &self.state
    }
    pub fn state_mut(&mut self) -> &mut crate::doc::build::state::DocState {
        &mut self.state
    }
    /// A mounted element owns layout key `key`.
    pub fn mount_owns(&self, key: u64) -> bool {
        self.state.memory.retained.owns(key)
    }
    /// A mounted element keeps the widget memory of `key` while it is not built.
    pub fn mount_keeps_memory(&self, key: u64) -> bool {
        self.state.memory.retained.keeps_memory(key)
    }
    /// The path of the deferred mounted element `key` focus waits for.
    pub fn deferred_focus_path(&self, key: u64) -> Option<&str> {
        self.state.memory.retained.deferred_focus_path(key)
    }
    /// The mounts and the documents they mount.
    pub fn mounted_documents(&self) -> Vec<(crate::reactive::SourceId, String)> {
        self.state
            .memory
            .retained
            .registrations
            .iter()
            .map(|(m, r)| (m.clone(), r.document.clone()))
            .collect()
    }
    /// How far over SDR white the frame may go: the computed
    /// `dynamic-range-limit` (CSS Color HDR 1) of the mounted documents'
    /// root elements, the most limiting where they differ (the lowest peak
    /// on a display with four stops of headroom, 16×); `no-limit`, the
    /// initial value, when none declares one.
    pub fn range_limit(&self) -> og_draw::RangeLimit {
        let r = &self.state.memory.retained;
        r.roots
            .values()
            .filter_map(|root| match root {
                crate::doc::build::instances::Child::Element(n) => r.elements.get(n),
                _ => None,
            })
            .filter_map(|e| {
                let values = &e.resolved.as_ref()?.values.cascade.values;
                let (_, v, _) = values
                    .iter()
                    .rev()
                    .find(|(n, ..)| n == "dynamic-range-limit")?;
                crate::css::text_css::range_limit(v)
            })
            .min_by(|a, b| a.peak(16.0).total_cmp(&b.peak(16.0)))
            .unwrap_or_default()
    }
    /// The bindings the mounts evaluated and the elements they reconciled this tick.
    pub fn mount_counts(&self) -> (usize, usize) {
        let r = &self.state.memory.retained;
        (r.evaluated, r.reconciled)
    }
    /// What restyled mounted element `key` and its bindings, for the debug view.
    pub fn mount_debug_styles(
        &self,
        key: u64,
    ) -> Option<(Vec<crate::UiDebugStyle>, crate::ui::DebugBindings)> {
        self.state.memory.retained.debug_styles(key)
    }
    /// Runs a command of the reactive session.
    pub fn execute(&mut self, command: crate::reactive::session::Command) -> Result<(), String> {
        self.state.memory.reactive.execute(command).map(|_| ())
    }
    /// The locale changed: what formats by it reads afresh.
    pub fn locale_changed(&mut self) {
        self.state.memory.reactive.graph.locale_changed();
    }
}

/// A document's part rules laid into the global ones, and the sheet they
/// merge into.
pub(crate) type MergedPartRules = (
    Vec<crate::doc::StyleRule>,
    crate::doc::at_rules::Sheet,
    crate::doc::build::PartIndex,
);

/// A document and its rules merged into the global ones ([`MergedPartRules`]).
pub(crate) type DocumentPartRules = (
    std::sync::Arc<crate::doc::UiDocument>,
    std::sync::Arc<MergedPartRules>,
);

/// Style: the part rules and tokens, the motion pass's memory, timelines and the plays asked for.
#[derive(Clone, Debug)]
pub struct Style {
    /// Global CSS part rules in cascade order and their at-rule descriptors.
    pub(crate) part_rules: std::sync::Arc<Vec<crate::doc::StyleRule>>,
    pub(crate) part_sheet: std::sync::Arc<crate::css::at_rules::Sheet>,
    pub(crate) root_tokens: Vec<(String, String)>,
    pub(crate) root_type_tokens: [Vec<(String, String)>; 4],
    /// Root cascades for backdrop off/on. The off answer bootstraps the
    /// style's setting before its on-state fills are cascaded; keeping both
    /// avoids recascading twice on every frame of a style opt-in.
    pub(crate) root_cascade: std::cell::RefCell<
        [Option<(
            crate::doc::build::RootContext,
            crate::doc::build::RootIdentity,
        )>; 2],
    >,
    /// The palettes of the `accent-color`s drawn, each the root tokens
    /// read with the accent as `--seed-primary` ([`crate::Ui::accent_palette`]),
    /// for one media context. Derived from the root inputs: dropped with
    /// the root cascade.
    pub(crate) accent_palettes:
        std::cell::RefCell<Vec<(crate::css::media::Context, Srgba, crate::Palette)>>,
    /// The theme the root tokens were last read into
    /// ([`crate::Ui::read_theme_tokens`]) and what it was read from. The
    /// read also depends on the root inputs above and on the fonts: a
    /// write of either drops it.
    pub(crate) theme_read: Option<Box<(ThemeSource, Theme)>>,
    /// Part samples read one before-change state throughout a frame. Commit
    /// their next state together after rendering the owning nodes.
    pub(crate) part_commits: std::cell::RefCell<
        std::collections::BTreeMap<
            (u64, &'static str),
            std::sync::Arc<crate::doc::build::StyleCommit>,
        >,
    >,
    /// [`crate::doc::build::PartIndex`] of `part_rules`, made on first use.
    /// Derived from it alone: a write of it clears this.
    pub(crate) part_index: std::cell::RefCell<Option<std::sync::Arc<crate::doc::build::PartIndex>>>,
    /// Each document's part rules laid into `part_rules` and `part_sheet`
    /// (`Ui::document_part_rules`), by its name and the document they were
    /// merged from. Derived from those two alone: a write of either clears it.
    pub(crate) document_part_rules:
        std::cell::RefCell<std::collections::BTreeMap<String, DocumentPartRules>>,
    /// Each part's style from one tick to the next
    /// ([`crate::doc::build::PartStyles`]). Derived from the part rules
    /// and the root inputs: a write of either clears it.
    pub(crate) part_styles: std::cell::RefCell<crate::doc::build::PartStyles>,
    pub(crate) css_sounds: std::collections::BTreeMap<String, u64>,
    /// Timelines by name hash (`define_timeline`): configuration, like the theme.
    pub(crate) timelines: std::collections::BTreeMap<u64, (String, Timeline)>,
    /// Plays asked for this tick, flattened at `end`.
    pub(crate) pending_plays: Vec<crate::motion::run::PendingPlay>,
    /// The motion pass's memory between ticks.
    pub(crate) motion: crate::layout::MotionCache,
}

/// The fields of style the documents (og-ui) reach.
impl Style {
    /// Takes a style's part rules and sheet as they were read: shared,
    /// not copied.
    pub(crate) fn set_part_sheet(&mut self, (rules, sheet): crate::doc::stylesheet::PartSheet) {
        self.part_rules = rules;
        self.part_sheet = sheet;
        self.part_rules_changed();
    }
    /// Drops what is derived from `part_rules` and `part_sheet`.
    pub(crate) fn part_rules_changed(&mut self) {
        self.part_index.take();
        self.document_part_rules.take();
        self.part_styles.get_mut().clear();
    }
    pub fn css_sounds_mut(&mut self) -> &mut std::collections::BTreeMap<String, u64> {
        &mut self.css_sounds
    }
    pub fn motion_mut(&mut self) -> &mut crate::layout::MotionCache {
        &mut self.motion
    }
}

/// What a reading of the root tokens into a theme starts from, beside
/// the root inputs and the fonts ([`Style::theme_read`]).
#[derive(Clone, Debug, PartialEq)]
pub struct ThemeSource {
    /// The theme the tokens are read into.
    pub(crate) theme: Theme,
    pub(crate) configured: Theme,
    /// The shared root answer, reused while its media answers hold.
    pub(crate) root: crate::doc::build::RootIdentity,
}

impl Style {
    /// The part rules, their sheet or the root tokens were written.
    pub fn root_inputs_changed(&mut self) {
        *self.root_cascade.get_mut() = Default::default();
        self.accent_palettes.get_mut().clear();
        self.theme_read = None;
        self.part_styles.get_mut().clear();
    }
}

impl Paint {
    /// The map of the element whose commands are being made.
    pub(crate) fn map(&self) -> Affine {
        self.ctx.as_ref().map_or(Affine::IDENTITY, |c| c.map)
    }

    /// The theme the element being painted draws with.
    pub(crate) fn theme<'a>(&'a self, env: &'a Env) -> &'a Theme {
        match &self.ctx {
            Some(c) if c.accented => &self.accented,
            _ => &env.theme,
        }
    }

    /// Readies the environment's theme with `palette`, an `accent-color`'s
    /// ([`crate::Ui::accent_palette`]), for the element about to be
    /// painted; whether it has one ([`PaintCtx::accented`]). Only an
    /// accented element copies the theme.
    pub(crate) fn accent(&mut self, env: &Env, palette: Option<crate::Palette>) -> bool {
        let Some(palette) = palette else { return false };
        *self.accented = env.theme;
        self.accented.palette = palette;
        true
    }
}

impl crate::ui::Ui {
    /// The theme paint draws with: the environment's, its primary ink the
    /// painted element's `accent-color`.
    pub(crate) fn paint_theme(&self) -> &Theme {
        self.paint.theme(&self.env)
    }

    /// Node `n`'s clip as paint draws it: the painted element's where its
    /// commands draw.
    pub(crate) fn paint_clip(&self, n: &Node) -> Rect {
        match &self.paint.ctx {
            Some(c) if std::ptr::eq(n, &self.tree.nodes[c.node]) => c.clip,
            _ => n.placed.clip,
        }
    }
}

/// What paint draws one element with, set around its commands: its clip
/// where they draw (a 3D plane's in its layer, mapped back to its layout
/// box under a map), the map (a nine-slice is cut at its size on the
/// screen, its corners kept crisp) and whether its CSS `accent-color` is
/// the primary ink ([`Paint::accent`]). Paint reads it in place of the
/// node's clip and the environment's theme, which it no longer writes.
#[derive(Clone, Copy, Debug)]
pub struct PaintCtx {
    pub(crate) node: usize,
    pub(crate) clip: Rect,
    pub(crate) map: Affine,
    pub(crate) accented: bool,
}

/// Paint: its reuse cache, the part and decoration draws, the element
/// being drawn and the last frame's draws and inspection.
#[derive(Clone, Debug)]
pub struct Paint {
    /// This frame's command ranges kept from last frame's paint, in order.
    pub(crate) kept_spans: Vec<std::ops::Range<usize>>,
    /// Elements open in this frame's command pass whose backgrounds are
    /// cut to text (`background-clip: text`), innermost last: each node,
    /// where its background ends (where the group cut to its subtree's
    /// text goes when the subtree closes), and the keys of the nodes
    /// whose next-frame ranges start from its own (moved with the group).
    pub(crate) text_clips: Vec<(usize, usize, Vec<u64>)>,
    /// This frame's command pass drew a CSS `element()` image's group,
    /// filled when the pass ends (`Ui::fill_element_images`).
    pub(crate) element_images: bool,
    /// What each part of this frame draws through the UI pass, by node
    /// (made at `end` from its dressing, or the tokens).
    pub(crate) part_draws: std::collections::HashMap<usize, crate::paint::ElementDraw>,
    /// What each part draw was made from ([`PartInput`]): kept while that
    /// is the same.
    pub(crate) part_inputs: std::collections::HashMap<usize, PartInput>,
    /// Parts whose computed paint changed since the preceding frame.
    pub(crate) part_changed: std::collections::HashSet<usize>,
    /// This frame's draws of elements with CSS box decorations: their
    /// draw or dressing with the decorations over it.
    pub(crate) decor_draws: std::collections::HashMap<usize, crate::paint::ElementDraw>,
    /// The element whose commands are being made (`None` between
    /// elements): its clip, map and theme as paint draws it.
    pub(crate) ctx: Option<PaintCtx>,
    /// The environment's theme with the painted element's accent
    /// ([`Paint::accent`]).
    pub(crate) accented: Box<Theme>,
    pub(crate) prev_draws:
        std::collections::HashMap<usize, std::sync::Arc<crate::paint::ElementDraw>>,
    /// The last frame's inspection, made when asked for (`inspect`).
    pub(crate) inspection: inspect::Inspection,
    /// Last frame's paint, for reuse.
    pub(crate) cache: crate::layout::PaintCache,
}

/// Layout: its text measurements, its reuse cache and the boxes placement replaced for one frame.
#[derive(Clone, Debug)]
pub struct Layout {
    /// Text measurements kept from the frames before (layout's).
    pub(crate) texts: crate::layout::TextCache,
    /// Last frame's input hash of each element and its subtree, with its
    /// layout, for reuse (docs/ui.md §2).
    pub(crate) cache: crate::layout::LayoutCache,
    /// Container queries: the measurements watched and the roots a
    /// settlement lays out again.
    pub(crate) queries: settlement::ContainerQueries,
    /// Nodes whose used box placement fixed (a layer's size, a modal's
    /// safe padding): let go before the next layout measures them.
    pub(crate) placed: Vec<usize>,
}

/// The fields of layout the documents (og-ui) reach.
impl Layout {
    pub fn cache_mut(&mut self) -> &mut crate::layout::LayoutCache {
        &mut self.cache
    }
}

/// Interaction: the frame's input, the persisted interaction state and what navigation, gestures and selection decided.
#[derive(Clone, Debug)]
pub struct Interaction {
    pub(crate) input: InputFrame,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) native_input: crate::native_a11y::Input,
    pub(crate) state: State,
    pub(crate) nav: NavFrame,
    /// Navigation declared for this frame (`set_navigation`).
    pub(crate) nav_on: bool,
    /// Players declared for this frame (`set_players`).
    pub(crate) players: u32,
    /// The virtual cursor declared for this frame (`set_virtual_cursor`).
    pub(crate) vcursor: Option<crate::vcursor::VirtualCursorDesc>,
    /// The control a widget asked to focus this frame.
    pub(crate) request_focus: u64,
    /// `state.hits`' order for sequential focus where a container's
    /// `reading-flow` changes it, as indices into them (`None`: theirs;
    /// [`crate::layout::reading`]). Pointer hits keep the tree's order.
    pub(crate) reading: Option<Vec<u32>>,
    /// Two touches move a scroller this tick: the primary pointer
    /// presses nothing.
    pub(crate) gesture: bool,
    /// The element a resize drag moved this tick (written again).
    pub(crate) resize_moved: Option<u64>,
    /// The focused text control (into [`UiFrame::text_input`]), as it was
    /// last built: kept while it holds the focus and is not built again.
    pub(crate) text_target: Option<TextTarget>,
    /// The key of the text control `text_target` is.
    pub(crate) text_target_key: u64,
    /// The keys of the text elements last painted with a selection.
    pub(crate) selected: Vec<u64>,
    /// The custom highlights registered, in order
    /// ([`crate::ui::highlight`]), and the text elements last marked with
    /// them.
    pub(crate) highlights: Vec<crate::ui::highlight::Registered>,
    pub(crate) highlight_marked: Vec<u64>,
    /// Last frame's drop targets and dragged elements: their paths, and the
    /// tags each target takes.
    pub(crate) drop_paths: IdMap<u64, String>,
    pub(crate) drag_paths: IdMap<u64, String>,
    pub(crate) drop_accepts: IdMap<u64, Vec<String>>,
    /// Named UI actions and the last device (docs/ui.md §2.1).
    pub(crate) actions: crate::actions::Actions,
}

/// The fields of interaction the documents (og-ui) reach.
impl Interaction {
    pub fn input(&self) -> &InputFrame {
        &self.input
    }
    pub fn state(&self) -> &State {
        &self.state
    }
    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }
    pub fn nav(&self) -> &NavFrame {
        &self.nav
    }
    pub fn nav_mut(&mut self) -> &mut NavFrame {
        &mut self.nav
    }
    pub fn nav_on(&self) -> bool {
        self.nav_on
    }
    pub fn players(&self) -> u32 {
        self.players
    }
    pub fn request_focus_mut(&mut self) -> &mut u64 {
        &mut self.request_focus
    }
    pub fn actions(&self) -> &crate::actions::Actions {
        &self.actions
    }
    pub fn actions_mut(&mut self) -> &mut crate::actions::Actions {
        &mut self.actions
    }
}

/// Tree: the node arena and its slots, the build cursor, the trees' index and orders, and what the frame may keep.
#[derive(Clone, Debug)]
pub struct Tree {
    pub(crate) nodes: Vec<Node>,
    pub(crate) kinds: Vec<(&'static str, bool)>,
    /// The nodes this frame's build replaced or writes in place, as the
    /// last frame left them, by slot: what a build reads of the last frame
    /// (`Tree::previous_node`) and what the frame compares each written
    /// node with, once for layout (`Layout::reuse_check`, kept as
    /// `Node::changed`) and once for paint (`Ui::paint_reuse`). Empty from
    /// then until the next build writes.
    pub(crate) replaced: std::collections::HashMap<usize, Node>,
    /// Last frame's `kinds`, beside `replaced`.
    pub(crate) prev_kinds: std::collections::HashMap<usize, (&'static str, bool)>,
    pub(crate) render_slots: std::collections::HashMap<(u64, usize), usize>,
    pub(crate) free_slots: Vec<usize>,
    pub(crate) emitted: Vec<usize>,
    pub(crate) retiring: Vec<usize>,
    pub(crate) slot_present: Vec<bool>,
    /// Per slot: owned by a retained element, so it lives across frames
    /// until its owner retires it (`retire_owned`), not for one frame.
    pub(crate) persistent: Vec<bool>,
    /// The slots each retained element being emitted writes: its own node
    /// and its widget's internal nodes, innermost element last.
    pub(crate) owning: Vec<build::Owned>,
    /// A retained element re-emitted into the slot it already holds in its
    /// parent's children: `(key, parent)` of the push that must not append.
    pub(crate) in_place: Option<(u64, usize)>,
    /// The document host of the control about to be built, by its key:
    /// the node written with that key holds it from the start, so the
    /// control's parts resolve against their own host as it builds them.
    pub(crate) pending_host: Option<(u64, std::sync::Arc<crate::doc::build::PartHost>)>,
    /// The content scope the next node is written in: the name of the
    /// snippet a mounted document renders it from (`body`; `a/b` when
    /// nested). Its id and key carry the scope between its parent's and
    /// its own, as its path does.
    pub(crate) scope: Option<String>,
    /// This frame's element keys under a parent, which a repeated id
    /// would take again (`Ui::push` renames it), and the anonymous
    /// elements of each kind under each parent: kept as built, so a
    /// long list pays no scan of its siblings for each element.
    pub(crate) built: crate::id::KeySet,
    pub(crate) anonymous: crate::id::KeyMap<u32>,
    pub(crate) stack: Vec<usize>,
    pub(crate) floating: Vec<usize>,
    pub(crate) last_closed: usize,
    /// The trees' membership, keys and features (`tree`).
    pub(crate) index: tree::TreeIndex,
    /// The frame's orders and each feature's nodes (`visit`).
    pub(crate) orders: visit::Orders,
    /// What may have changed for screen readers, by frame (`tree`).
    pub(crate) semantics: tree::SemanticLog,
    pub(crate) building: bool,
    /// Open focus scopes this frame, in build order.
    pub(crate) scopes: Vec<usize>,
    /// What elements of this frame draw with ([`Ui::set_draw`]), by node.
    /// Shared, so a retained slot keeps its draw without copying it and
    /// paint knows it unchanged by its pointer (`DrawSig`).
    pub(crate) draws: std::collections::HashMap<usize, std::sync::Arc<crate::paint::ElementDraw>>,
    /// The draws of [`Ui::set_draw_dressable`]: a component's own look for
    /// a part, which a style that dresses the part restyles.
    pub(crate) dressable: std::collections::HashSet<usize>,
    /// Last frame's layers and draws while this frame is built, for
    /// inspecting the last frame mid-build.
    pub(crate) prev_floating: Vec<usize>,
    /// What this frame may keep from the last.
    pub(crate) invalidation: crate::layout::Invalidation,
}

/// The fields of the tree the documents (og-ui) reach.
impl Tree {
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }
    /// Node `i`'s accessibility semantics, to write.
    pub fn semantics_mut(&mut self, i: usize) -> &mut crate::a11y::A11y {
        &mut self.nodes[i].a11y
    }
    /// Hides node `i` this frame.
    pub fn hide(&mut self, i: usize) {
        self.nodes[i].hidden = true;
    }
    /// Places node `i` (a popover) at `point` with its `arrow` (size, colour).
    pub fn anchor_at(
        &mut self,
        i: usize,
        point: Option<[f32; 2]>,
        arrow: Option<(f32, crate::Srgba)>,
    ) {
        let n = &mut self.nodes[i];
        n.anchor_point = point;
        n.arrow = arrow;
    }
    /// Whether node `i` (a layer) catches the pointer behind it and scopes focus.
    pub fn capture(&mut self, i: usize, backdrop: bool, scope: bool) {
        let n = &mut self.nodes[i];
        n.backdrop = backdrop;
        n.scope = scope;
    }
    pub fn kinds(&self) -> &Vec<(&'static str, bool)> {
        &self.kinds
    }
    pub fn owning_mut(&mut self) -> &mut Vec<build::Owned> {
        &mut self.owning
    }
    pub fn in_place_mut(&mut self) -> &mut Option<(u64, usize)> {
        &mut self.in_place
    }
    pub fn scope_mut(&mut self) -> &mut Option<String> {
        &mut self.scope
    }
    pub fn stack_mut(&mut self) -> &mut Vec<usize> {
        &mut self.stack
    }
    pub fn floating_mut(&mut self) -> &mut Vec<usize> {
        &mut self.floating
    }
    pub fn last_closed(&self) -> usize {
        self.last_closed
    }
    pub fn index(&self) -> &tree::TreeIndex {
        &self.index
    }
    pub fn invalidation(&self) -> &crate::layout::Invalidation {
        &self.invalidation
    }
    pub fn invalidation_mut(&mut self) -> &mut crate::layout::Invalidation {
        &mut self.invalidation
    }
}

/// The immediate front end's memory: widget memories used, kept texts, button inks, the control accent, the HUD editor and the screen cover.
#[derive(Clone, Debug)]
pub struct Front {
    /// Widget memories used this frame (others are dropped at `end`).
    pub(crate) touched: std::collections::BTreeSet<u64>,
    /// Text a control keeps between ticks while it is open (a dropdown's
    /// filter), by key: names of the frame's UI, dropped when it closes.
    pub(crate) kept_texts: crate::id::KeyMap<String>,
    /// The content buttons open now and their ink (the innermost last).
    pub(crate) button_ink: Vec<(usize, Srgba)>,
    /// CSS `accent-color` of the document control being built: its
    /// primary ink (`widgets::controls`' roles).
    pub(crate) accent: Option<crate::Srgba>,
    /// This frame's screen-transition cover.
    pub(crate) cover: Option<crate::juice::Cover>,
}

/// The fields of the immediate front end's memory the documents (og-ui) reach.
impl Front {
    pub fn accent_mut(&mut self) -> &mut Option<crate::Srgba> {
        &mut self.accent
    }
    pub fn cover(&self) -> &Option<crate::juice::Cover> {
        &self.cover
    }
}

/// What a part's draw is made from (`Ui::build_part_draws`): its node, its
/// values, its size and ground, the game draw it restyles and the
/// viewport. The values and draw are held, so their pointers name them.
#[derive(Clone)]
pub(crate) struct PartInput {
    pub(crate) key: u64,
    pub(crate) values: std::sync::Arc<crate::doc::build::Resolved>,
    pub(crate) size: [f32; 2],
    pub(crate) ground: Option<og_css::Srgba>,
    pub(crate) base: Option<std::sync::Arc<crate::paint::ElementDraw>>,
    pub(crate) viewport: Viewport,
}

impl PartInput {
    pub(crate) fn same(&self, other: &PartInput) -> bool {
        self.key == other.key
            && std::sync::Arc::ptr_eq(&self.values, &other.values)
            && self.size == other.size
            && self.ground == other.ground
            && match (&self.base, &other.base) {
                (None, None) => true,
                (Some(a), Some(b)) => std::sync::Arc::ptr_eq(a, b),
                _ => false,
            }
            && self.viewport == other.viewport
    }
}

impl std::fmt::Debug for PartInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PartInput")
            .field("key", &self.key)
            .field("size", &self.size)
            .finish_non_exhaustive()
    }
}
