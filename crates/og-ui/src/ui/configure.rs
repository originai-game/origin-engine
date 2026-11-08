use super::*;

impl Ui {
    pub fn new(desc: UiDesc) -> Ui {
        Ui {
            configured: desc.theme,
            theme: desc.theme,
            fonts: {
                let mut f = desc.fonts;
                f.link();
                f
            },
            ticks_per_second: if desc.ticks_per_second == 0 {
                60
            } else {
                desc.ticks_per_second
            },
            viewport: Viewport::default(),
            unit: 1.0,
            input: InputFrame::default(),
            #[cfg(not(target_arch = "wasm32"))]
            native_input: Default::default(),
            nodes: Vec::new(),
            kinds: Vec::new(),
            prev_kinds: Default::default(),
            render_slots: Default::default(),
            free_slots: Vec::new(),
            emitted: Vec::new(),
            retiring: Vec::new(),
            slot_present: Vec::new(),
            persistent: Vec::new(),
            owning: Vec::new(),
            in_place: None,
            layout_mutated: Vec::new(),
            built: Default::default(),
            anonymous: Default::default(),
            stack: Vec::new(),
            floating: Vec::new(),
            last_closed: NONE,
            warnings: Vec::new(),
            glyphs_missed: Default::default(),
            kept_spans: Vec::new(),
            tree: Default::default(),
            orders: Default::default(),
            semantics: Default::default(),
            building: false,
            state: State::default(),
            frame: UiFrame::default(),
            inspection: Default::default(),
            prev_floating: Vec::new(),
            prev_draws: Default::default(),
            nav: NavFrame::default(),
            touched: Default::default(),
            nav_on: true,
            players: 1,
            vcursor: None,
            hud_editor_key: 0,
            request_focus: 0,
            button_ink: Vec::new(),
            scopes: Vec::new(),
            icons: IconAtlas::default(),
            cue_icons: Vec::new(),
            kept_texts: Default::default(),
            images: Default::default(),
            toast_anchor: Anchor::Bottom,
            hud_max_aspect: HUD_MAX_ASPECT,
            feedback: Vec::new(),
            haptics: 1.0,
            haptic_out: Vec::new(),
            dress: std::sync::Arc::new(crate::dress::DressSet::new(
                crate::style_file::theme_dressings(&desc.theme.art),
            )),
            dress_set: false,
            part_draws: Default::default(),
            decor_draws: Default::default(),
            clipboard: None,
            selected: Vec::new(),
            text_target: None,
            cover: None,
            style_name: String::new(),
            gesture: false,
            direction: og_i18n::Direction::Ltr,
            locale: String::new(),
            texts: Default::default(),
            timelines: Default::default(),
            pending_plays: Vec::new(),
            events: Vec::new(),
            drop_paths: IdMap::new(),
            drag_paths: IdMap::new(),
            drop_accepts: IdMap::new(),
            kept: Default::default(),
            stats: LayoutStats::default(),
            drawing: Affine::IDENTITY,
            draws: Default::default(),
            dressable: Default::default(),
            docs: Default::default(),
            reactive: Default::default(),
            actions: Default::default(),
            world: Default::default(),
        }
    }

    /// The icon atlas `icon` draws from.
    pub fn set_icons(&mut self, icons: IconAtlas) {
        self.icons = icons;
    }

    pub fn icons(&self) -> &IconAtlas {
        &self.icons
    }

    /// The style's `icons` names, in its order: a colour token's icon cue
    /// is an index into them (`crate::tokens::Cue::Icon`), whichever atlas
    /// the game draws icons from. Empty takes the atlas's own names.
    pub fn set_cue_icons(&mut self, names: Vec<String>) {
        self.cue_icons = names;
    }

    /// The icon colour token `token`'s cue names (`-og-cue`, docs/ui.md
    /// §3.2), or `None` (warned) when the token has no icon cue.
    pub(crate) fn cue_icon(&mut self, token: &str) -> Option<String> {
        use crate::tokens::Cue;
        let cue = self.theme().tokens.colors.cue(token);
        let names = if self.cue_icons.is_empty() {
            &self.icons.names
        } else {
            &self.cue_icons
        };
        match cue {
            Some(Cue::Icon(i)) => {
                if let Some(n) = names.get(usize::from(i)) {
                    return Some(n.clone());
                }
                self.warn(format!("-og-cue: {token}'s icon {i} is not in the style's icons"));
            }
            Some(other) => self.warn(format!(
                "-og-cue: {token}'s cue is {other:?}, not an icon: give it `icon \"<name>\"` in the style's colors"
            )),
            None => self.warn(format!("-og-cue: '{token}' is not a colour token")),
        }
        None
    }

    /// The pictures rich text draws inline by name (`[img=<name>]`).
    pub fn set_images(&mut self, images: std::collections::BTreeMap<String, InlineImage>) {
        self.images = images;
    }

    pub fn images(&self) -> &std::collections::BTreeMap<String, InlineImage> {
        &self.images
    }

    /// Where toasts stack (default: bottom centre of the safe area).
    pub fn set_toast_anchor(&mut self, anchor: Anchor) {
        self.toast_anchor = anchor;
    }

    /// The widest the HUD band is, over its height (UiDesc.hudMaxAspect;
    /// default [`HUD_MAX_ASPECT`], 21:9): on a wider surface the anchored
    /// groups and toasts lay out in a centred band this wide, so an
    /// ultrawide's HUD corners stay near the play. 0 is the whole safe area.
    pub fn set_hud_max_aspect(&mut self, max_aspect: f32) {
        self.hud_max_aspect = if max_aspect.is_finite() {
            max_aspect.max(0.0)
        } else {
            HUD_MAX_ASPECT
        };
    }

    /// The widest the HUD band is, over its height ([`Ui::set_hud_max_aspect`]).
    pub fn hud_max_aspect(&self) -> f32 {
        self.hud_max_aspect
    }

    /// The HUD band this frame lays out in: the safe area at most
    /// UiDesc.hudMaxAspect wide, centred (CSS px). The anchored groups (HUD
    /// corners) and the toasts sit in it; a component that places itself
    /// reads it (og.ui.viewport's `hud`). Full-screen layers (dialogs,
    /// scrims, world-space markers) keep the whole safe area.
    pub fn hud_area(&self) -> Rect {
        self.viewport.hud_rect(self.hud_max_aspect)
    }

    /// The size class of the current play surface.
    pub fn breakpoint(&self) -> Breakpoint {
        self.viewport.breakpoint()
    }

    /// The play surface of the frame being built.
    pub fn viewport(&self) -> Viewport {
        self.viewport
    }

    /// The og-ui unit a frame on the CSS `viewport` would lay out in (the
    /// game's pin, else the surface's: docs/ui.md §6.1).
    pub fn unit_for(&self, viewport: &Viewport) -> f32 {
        surface_unit(&self.configured, viewport)
    }

    /// Frames begun so far: the UI's clock.
    pub fn tick(&self) -> u64 {
        self.state.tick
    }

    /// The UI's clock in seconds (frames over the tick rate).
    pub fn seconds(&self) -> f64 {
        self.state.tick as f64 / f64::from(self.ticks_per_second)
    }

    pub fn ticks_per_second(&self) -> u32 {
        self.ticks_per_second
    }

    /// Ticks a duration in milliseconds takes (at least 1).
    pub fn ticks_for(&self, ms: f32) -> u32 {
        Theme::ticks(ms, self.ticks_per_second)
    }

    /// Whether keyboard and gamepad navigation moves focus on this screen.
    /// On by default; a gameplay HUD whose keys move the player turns it off
    /// (an open dialog or menu turns it on while it is open). Takes effect
    /// from the next tick.
    pub fn set_navigation(&mut self, on: bool) {
        self.nav_on = on;
    }

    /// Local multiplayer: how many players this screen takes. Pads 1 to
    /// `count - 1` each move a focus of their own (a ring in their player
    /// colour) among the same controls, and their A presses the control
    /// under it ([`Response::pad`] says which pad); pad 0 shares the one
    /// focus with the keyboard and the pointer. 1 by default; declared every
    /// tick, like [`Ui::set_navigation`], and read from the next.
    pub fn set_players(&mut self, count: u32) {
        self.players = count.clamp(1, 4);
    }

    /// The last frame's accessibility tree: each element with a role, its
    /// name, value, states and bounds, and the focus order (`crate::a11y`).
    /// Whether keys and pads move the focus on the last frame's screen: off
    /// after [`Ui::set_navigation`]`(false)`, unless a dialog is open (a
    /// dialog's controls are always walked).
    pub fn navigation(&self) -> bool {
        !self.state.nav_off || self.state.scope != 0
    }

    /// What element `i` tells the accessibility tree: role, name,
    /// description, value and its own states (the frame adds `focusable`
    /// and `focused`).
    pub(crate) fn a11y_info(
        &self,
        i: usize,
    ) -> (&'static str, String, String, String, Vec<String>) {
        crate::a11y::node_info(&self.nodes, &self.kinds, i)
    }

    /// The path of the control pad `pad`'s focus is on in the last frame
    /// (pad 0: the shared focus), or `""`.
    pub fn focus_path(&self, pad: u32) -> &str {
        self.frame
            .focus_paths
            .get(pad as usize)
            .map_or("", String::as_str)
    }

    /// Queues a navigation intent for the next tick (a gamepad the input
    /// layer does not read yet, a custom binding, a test). A move queued
    /// while the focus is on nothing puts it on the first control, whatever
    /// its direction, and moves no further: every such move queued before
    /// the focus lands (a game asking each tick until it sees the focus)
    /// lands on that same first control.
    pub fn navigate(&mut self, nav: Nav) {
        let entering = self.state.focus == 0 && !matches!(nav, Nav::Activate | Nav::Back);
        self.state.pending.push((nav, entering));
    }

    /// The key of the focused control (0: none).
    pub fn focused_key(&self) -> u64 {
        self.state.focus
    }

    /// Moves the shared focus (the pointer's, the keys' and pad 0's) to the
    /// element at `path` (a full path, as [`Ui::focus_path`] names it) when
    /// this frame ends, if a focusable control is there then: a radio
    /// group's arrow keys moving the focus with the choice, a list opening
    /// at its chosen option. Called again in the frame, the last one wins.
    pub fn set_focus(&mut self, path: &str) {
        self.request_focus = crate::id::key_of_path(path);
    }

    /// Escape or gamepad B this tick, not taken by a dialog or menu: true
    /// once, then taken. A pause screen closes on it. While a dialog or menu
    /// Back closes is open (the topmost of the last frame), it answers false
    /// and leaves Back to that dialog, even when called before the dialog
    /// is built this tick (a HUD drawn under pause): og-ui's rule, the
    /// topmost dialog or menu closes. A locked dialog lets it through.
    pub fn back(&mut self) -> bool {
        if self.state.scope != 0 && !self.state.scope_locked {
            return false;
        }
        std::mem::take(&mut self.nav.back)
    }

    /// The widget memory of `key`, marked as used this frame.
    pub(crate) fn anim(&mut self, key: u64) -> Anim {
        self.touched.insert(key);
        if let Some(owned) = self.owning.last_mut() {
            owned.memories.push(key);
        }
        self.state.anims.get(&key).copied().unwrap_or_default()
    }

    pub(crate) fn has_anim(&self, key: u64) -> bool {
        self.state.anims.contains_key(&key)
    }

    pub(crate) fn set_anim(&mut self, key: u64, a: Anim) {
        self.touched.insert(key);
        let old = self.state.anims.insert(key, a);
        if let Some(owned) = self.owning.last_mut() {
            owned.memories.push(key);
            owned.moved |= old != Some(a);
        }
    }

    /// The key the next element with this id would get under the open element.
    pub(crate) fn key_for(&self, id: &str) -> u64 {
        let parent = self.nodes.get(self.parent()).map_or(1, |n| n.key);
        child_key(parent, id)
    }

    /// The theme in effect: the configured one ([`Ui::set_theme`]) with the
    /// viewport's reduced-motion and contrast preferences applied.
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.configured = theme;
        self.theme = effective_theme(theme, &self.viewport);
        if !self.dress_set {
            self.dress = std::sync::Arc::new(crate::dress::DressSet::new(
                crate::style_file::theme_dressings(&theme.art),
            ));
        }
    }

    /// The style's part dressings (its [`crate::StyleFile::dressings`],
    /// bound to its loaded assets): what every part of the UI is drawn
    /// with where a dressing's selector matches it (docs/ui.md §3.2).
    /// Until a game sets them the theme's version 1 art makes them, or for
    /// a theme without art the default style's ([`crate::default_dressings`]).
    pub fn set_dressings(&mut self, dressings: Vec<crate::dress::Dressing>) {
        self.dress = std::sync::Arc::new(crate::dress::DressSet::new(dressings));
        self.dress_set = true;
        self.kept.epoch += 1;
    }

    /// The dressings in effect.
    pub fn dressings(&self) -> &crate::dress::DressSet {
        &self.dress
    }

    /// Every dressing that matches `part`, merged.
    pub fn resolve_part(&self, part: &crate::dress::PartRef) -> crate::dress::Resolved {
        self.dress.resolve(part)
    }

    /// Gives the element open now (a grid's cell, a panel, a card) a
    /// selector attribute its style dresses by: `ui.part_attr("rarity",
    /// "epic")` in a bag's cell makes it `Slot.frame[rarity=epic]`. An
    /// element without a part takes its version 1 frame's (a plain one, an
    /// `Element.base`). Answers whether the style dresses the part by that
    /// attribute (for any of its values), so a caller draws its own mark
    /// only where the style tells the values apart with none.
    pub fn part_attr(&mut self, key: &str, value: &str) -> bool {
        let Some(&i) = self.stack.last() else {
            return false;
        };
        let el = &mut self.nodes[i].el;
        let part = el
            .part
            .or_else(|| el.frame.map(StylePart::part_ref))
            .unwrap_or_default();
        let tagged = part.attr(crate::dress::intern(key), crate::dress::intern(value));
        el.part = Some(tagged);
        el.frame = None;
        self.dress.dresses_key(&tagged, key)
    }

    /// The player's haptic strength: 0 turns haptics off, 1 plays the
    /// style's patterns as set (docs/ui.md §3.1).
    pub fn set_haptics(&mut self, intensity: f32) {
        self.haptics = if intensity.is_finite() {
            intensity.clamp(0.0, 1.0)
        } else {
            1.0
        };
    }

    pub fn haptics(&self) -> f32 {
        self.haptics
    }

    /// The configured style's name, for the theme the host reports (empty: the default style).
    pub fn style_name(&self) -> &str {
        &self.style_name
    }

    pub fn set_style_name(&mut self, name: &str) {
        self.style_name = name.to_owned();
    }

    pub fn fonts(&self) -> &Fonts {
        &self.fonts
    }

    pub fn set_fonts(&mut self, fonts: Fonts) {
        self.fonts = fonts;
        self.kept.epoch += 1;
        self.fonts.link();
        self.texts.clear();
    }

    /// How many text measurements the last frame's layout made: a text's
    /// widths and its lines each count one, and a text whose string, font,
    /// type size and width are as the frame before made none (they are
    /// kept). For profiling and tests; never world state.
    pub fn text_measurements(&self) -> u32 {
        self.texts.measured()
    }

    /// The writing direction (left to right by default).
    pub fn direction(&self) -> og_i18n::Direction {
        self.direction
    }

    /// Right to left mirrors every frame's layout from the next `end` on.
    pub fn set_direction(&mut self, direction: og_i18n::Direction) {
        self.direction = direction;
    }

    /// The active BCP-47 locale for style-driven display casing. It never
    /// rewrites source strings, input fields or their selections.
    pub fn set_locale(&mut self, locale: &str) {
        if self.locale != locale {
            self.locale = locale.into();
            self.reactive.graph.locale_changed();
        }
    }

    /// The control sizes in effect for the current viewport.
    pub fn sizes(&self) -> Density {
        self.theme.sizes(&self.viewport)
    }

    /// The last completed frame.
    pub fn frame(&self) -> &UiFrame {
        &self.frame
    }

    /// The input of the frame being built.
    pub fn input(&self) -> &InputFrame {
        &self.input
    }

    /// Starts a frame whose root is a column over the whole viewport, padded by the safe area.
    pub fn begin(&mut self, input: InputFrame, viewport: Viewport) {
        self.begin_with(input, viewport, Element::column());
    }

    /// Starts a frame with a root of your own (its size is the viewport; the
    /// safe area is added to its padding; its background covers the whole viewport).
    pub fn begin_with(&mut self, input: InputFrame, viewport: Viewport, root: Element) {
        // The og-ui unit (docs/ui.md §6.1): the frame lays out in units of
        // it on the surface's size over it and draws it times larger
        // (`end`), so every length grows with the surface; input comes in
        // in units too.
        let unit = surface_unit(&self.configured, &viewport);
        let viewport = viewport.zoomed(unit);
        let mut input = input.zoomed(unit);
        self.unit = unit;
        // A pad's virtual cursor is the pointer while it shows, and the
        // HUD edit mode takes the presses on what it edits.
        self.virtual_cursor_input(&mut input, viewport.width, viewport.height);
        self.viewport = viewport;
        self.hud_edit_input(&mut input);
        // Scrolling first: a press on a scrollbar grabs it before anything
        // under it, two touches scroll or zoom what is under them.
        let grabbed = self.scroll_input(&input);
        if self.gesture {
            input.pressed = false;
            input.released = false;
            input.down = false;
        }
        if self.building {
            self.warnings.push(
                "begin without end: the previous frame was dropped; call ui.end() once per tick"
                    .into(),
            );
        }
        self.viewport = viewport;
        self.theme = effective_theme(self.configured, &viewport).to_unit(unit);
        // The style's atlas at the density this surface draws at.
        self.theme.art.atlas = self.configured.art.atlas_for(viewport.dpr());
        self.retire_nodes();
        self.kept.prev.clear();
        self.prev_kinds.clear();
        std::mem::swap(&mut self.emitted, &mut self.retiring);
        self.emitted.clear();
        for &i in &self.retiring {
            self.slot_present[i] = false;
        }
        self.built.clear();
        self.anonymous.clear();
        self.stack.clear();
        self.owning.clear();
        self.in_place = None;
        // Retained slots keep their layers and draws until written again.
        let persistent = &self.persistent;
        self.prev_floating = self.floating.clone();
        self.floating.retain(|&f| persistent[f]);
        self.prev_draws = std::mem::take(&mut self.draws);
        self.draws = self
            .prev_draws
            .iter()
            .filter(|&(&i, _)| persistent[i])
            .map(|(&i, d)| (i, d.clone()))
            .collect();
        self.dressable.retain(|&i| persistent[i]);
        self.warnings.clear();
        self.last_closed = NONE;
        self.building = true;

        self.state.tick += 1;
        // A Rive player whose element was not built last tick is gone.
        let tick = self.state.tick;
        self.state.rive.retain(|_, s| s.built + 1 >= tick);
        self.nav = NavFrame::default();
        self.touched.clear();
        self.request_focus = 0;
        self.scopes.clear();
        self.nav_on = true;
        self.players = 1;
        self.vcursor = None;
        self.hud_editor_key = 0;
        self.cover = None;
        // A pointer on a world panel is moved onto the panel's layout.
        self.world_begin(&mut input);
        self.docs_begin();
        self.actions_begin(&input);
        self.publish_ui_store();

        // Hit test against last frame's rectangles, topmost first.
        let p = input.pointer;
        let hot = if input.pointer_inside {
            crate::ui::pick_hit(&self.state.hits, p.x, p.y, |h| {
                !h.has(HIT_OFFSCREEN) && h.has(HIT_POINTER | HIT_SCROLL | HIT_TEXT)
            })
        } else {
            0
        };
        let hot = grabbed.unwrap_or(hot);
        self.state.hover_ticks = if hot != 0 && hot == self.state.hot {
            self.state.hover_ticks.saturating_add(1)
        } else {
            0
        };
        if hot != 0 && hot != self.state.hot && self.hit(hot).is_some_and(|h| h.has(HIT_NAV)) {
            self.sound(crate::style_file::UiSound::Hover);
        }
        if input.pressed && grabbed.is_none() && self.hit(hot).is_some_and(|h| h.has(HIT_NAV)) {
            self.sound(crate::style_file::UiSound::Press);
        }
        self.state.hot = hot;
        self.state.focus_click = None;
        if input.pressed {
            self.state.active = hot;
            let text = self.hit(hot).is_some_and(|h| h.has(HIT_TEXT));
            if text {
                self.state.focus = hot;
                self.state.focus_click = Some(p.x);
            } else if self.hit(hot).is_some_and(|h| h.has(HIT_NAV)) {
                // A pressed control takes focus without the ring (as :focus-visible).
                self.state.focus = hot;
            } else {
                // A control that takes no focus gives it to the focusable one
                // around it (a grid's item, its grid), as a click does on the web.
                self.state.focus = self.hit(hot).map_or(0, |h| h.owner);
            }
            self.state.focus_visible = false;
        }
        if grabbed.is_none() {
            self.select_input(&input, hot);
        }
        self.recognize(&input, hot, grabbed.is_some());
        let intents = self.intents(&input);
        self.apply_nav(&intents);
        self.apply_list_keys(&input);
        self.apply_a11y(&input.a11y);
        #[cfg(not(target_arch = "wasm32"))]
        self.apply_native_accessibility();
        self.apply_pads(&input);
        self.state.pointer = p;
        // Once a world has a typed-text stream, key presses type nothing.
        if input.typed_stream {
            self.state.typed_stream = true;
        } else if self.state.typed_stream {
            input.text.clear();
        }
        self.input = input;
        self.text_target = None;

        let mut el = root;
        el.width = Size::Fixed(viewport.width);
        el.height = Size::Fixed(viewport.height);
        el.padding = el.padding.add(&viewport.safe_area);
        el.scroll = false;
        self.write_node(Node::new(1, String::new(), NONE, el), ("root", false));
        self.stack.push(0);
    }
}

impl Ui {
    /// Queues a version 1 event (its version 2 event, [`UiSound::event`]).
    ///
    /// [`UiSound::event`]: crate::style_file::UiSound::event
    pub(crate) fn sound(&mut self, event: crate::style_file::UiSound) {
        self.feedback(event.event());
    }

    /// Plays a UI event's feedback (docs/ui.md §3.1): the sound of the
    /// style's set for it and its haptic, in the tick of the input that
    /// caused it. Widgets queue their own events (press, focus, toggles,
    /// tabs, slider steps, dialogs, toasts); game code queues the rest
    /// (rewards, rarity reveals, notifications). An event the style gives
    /// neither a sound nor a haptic queues nothing.
    pub fn feedback(&mut self, event: crate::tokens::SoundEvent) {
        self.queue_feedback(event, f32::NAN);
    }

    /// A step of a rising cascade (a counter's ticks): `rise` 0 at its
    /// start to 1 at its end walks the event's pitch range upwards.
    pub fn feedback_rising(&mut self, event: crate::tokens::SoundEvent, rise: f32) {
        self.queue_feedback(event, rise.clamp(0.0, 1.0));
    }

    fn queue_feedback(&mut self, event: crate::tokens::SoundEvent, rise: f32) {
        let f = self.theme.tokens.feedback(event);
        let heard = f.sounds[..usize::from(f.count)].iter().any(|s| *s != 0);
        if heard || f.haptic.count > 0 {
            self.feedback.push((event, rise));
        }
    }

    /// Adds a warning to the frame being built (a host's refused call),
    /// reported with the frame's own.
    pub fn warn(&mut self, message: impl Into<String>) {
        self.warnings.push(message.into());
    }

    /// Text a copy (Ctrl or Cmd with C) or cut (with X) took from a text
    /// control's selection since the last call: the host puts it on the
    /// clipboard (the browser shell does on the page's copy event).
    pub fn take_clipboard(&mut self) -> Option<String> {
        self.clipboard.take()
    }

    /// The feedback of the events queued since the last call, oldest first;
    /// taking it empties the queue (the host plays it). Each play is one
    /// sound of the event's set, chosen at random and never the one it
    /// played last, at a volume and pitch from the set's ranges (a rising
    /// cascade's pitch climbs its range instead), and the set's haptic at
    /// the player's strength ([`Ui::set_haptics`]; none when off). The
    /// choices come from the UI state's own random stream, so a replay of
    /// the same input plays the same sounds.
    pub fn take_feedback(&mut self) -> Vec<FeedbackPlay> {
        let queued = std::mem::take(&mut self.feedback);
        let mut out = Vec::with_capacity(queued.len());
        for (event, rise) in queued {
            let f = *self.theme.tokens.feedback(event);
            let n = usize::from(f.count);
            let mut next = || {
                // xorshift64*, seeded on first use.
                let mut x = self.state.feedback_rng;
                if x == 0 {
                    x = 0x9e37_79b9_7f4a_7c15;
                }
                x ^= x >> 12;
                x ^= x << 25;
                x ^= x >> 27;
                self.state.feedback_rng = x;
                (x.wrapping_mul(0x2545_f491_4f6c_dd1d) >> 40) as f32 / (1u64 << 24) as f32
            };
            let mut sound = 0;
            if n > 0 {
                let last = usize::from(self.state.feedback_last[event.index()]);
                let mut k = ((next() * n as f32) as usize).min(n - 1);
                if n > 1 && k == last {
                    k = (k + 1 + ((next() * (n - 1) as f32) as usize).min(n - 2)) % n;
                }
                self.state.feedback_last[event.index()] = k as u8;
                sound = f.sounds[k];
            }
            let range = |r: [f32; 2], t: f32| r[0] + (r[1] - r[0]) * t;
            let volume = range(f.volume, next());
            let pitch = if rise.is_nan() {
                range(f.pitch, next())
            } else {
                range(f.pitch, rise)
            };
            let mut haptic = f.haptic;
            if self.haptics <= 0.0 {
                haptic = crate::tokens::Haptic::NONE;
            }
            for p in haptic.pulses.iter_mut() {
                p.low *= self.haptics;
                p.high *= self.haptics;
            }
            if haptic.count > 0 && self.haptic_out.len() < 16 {
                self.haptic_out.push(haptic);
            }
            out.push(FeedbackPlay {
                event,
                sound,
                volume,
                pitch,
                haptic,
            });
        }
        out
    }

    /// The haptic patterns the feedback played since the last call, at the
    /// player's strength, for the host to run (pad rumble; a phone's
    /// vibration): the frame answer's `haptics`.
    pub fn take_haptics(&mut self) -> Vec<crate::tokens::Haptic> {
        std::mem::take(&mut self.haptic_out)
    }

    /// The version 1 events of the last frames that have a sound, oldest
    /// first, with the sound each plays; taking them empties the feedback
    /// queue ([`Ui::take_feedback`] is the whole of it).
    pub fn take_sounds(&mut self) -> Vec<(crate::style_file::UiSound, u64)> {
        self.take_feedback()
            .into_iter()
            .filter(|f| f.sound != 0)
            .filter_map(|f| {
                crate::style_file::UiSound::ALL
                    .into_iter()
                    .find(|u| u.event() == f.event)
                    .map(|u| (u, f.sound))
            })
            .collect()
    }

    /// `key`'s value moved towards `target` on the style's `ui` spring (a
    /// widget's state amount: a hover, a press, a knob's place); it
    /// starts at its first target, and under reduced motion it is there.
    pub(crate) fn spring_to(&mut self, key: u64, target: f32) -> f32 {
        self.animate_key(
            key,
            target,
            crate::motion::Transition::Token(crate::motion::MotionToken::Ui),
            None,
        )
    }

    /// How far into hover and press node `i` is for its response `r`, on
    /// the style's `ui` spring: what its part's states mix by.
    pub(crate) fn state_amounts(&mut self, i: usize, r: &Response) -> (f32, f32) {
        let key = self.nodes[i].key;
        let h = self.spring_to(
            child_key(key, "state.hover"),
            if r.hovered { 1.0 } else { 0.0 },
        );
        let p = self.spring_to(
            child_key(key, "state.press"),
            if r.pressed { 1.0 } else { 0.0 },
        );
        (h.clamp(0.0, 1.5), p.clamp(0.0, 1.5))
    }

    /// The colour of a control in its state: `rest` mixed towards `hover`
    /// and `pressed` by how far into each it is.
    pub(crate) fn state_color(
        rest: Srgba,
        hover: Srgba,
        pressed: Srgba,
        (h, p): (f32, f32),
    ) -> Srgba {
        mix_srgba(mix_srgba(rest, hover, h), pressed, p)
    }

    /// The style's type step for `role` at this frame's breakpoint
    /// (docs/ui.md §3.1): its size, line height and tracking; Chinese,
    /// Japanese and Korean text a line height half again its size.
    pub fn type_step(&self, role: crate::tokens::TypeRole) -> TypeStep {
        let tok = self.theme.tokens.typography.get(role, self.breakpoint());
        // The tokens are the style's as authored; the player's text scale
        // grows them here (the ramp carries it already).
        tok.step(self.theme.ramp_scale)
    }

    /// The font role the style sets `role` in.
    pub fn type_font(&self, role: crate::tokens::TypeRole) -> FontRole {
        self.theme
            .tokens
            .typography
            .get(role, self.breakpoint())
            .font
    }

    /// One line of a widget's text in a type role (its step and font).
    pub(crate) fn role_text(
        &self,
        text: &str,
        role: crate::tokens::TypeRole,
        color: Srgba,
    ) -> TextContent {
        let mut c = self.text_content(text, self.type_step(role), color);
        c.font = self.type_font(role);
        c.wrap = false;
        c
    }
}
