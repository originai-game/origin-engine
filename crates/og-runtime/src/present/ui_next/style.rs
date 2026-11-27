//! Style pack loading, theme/seed resolution, class resolver caching,
//! and input binding glyph lookups for the TSX UI core.

use super::*;

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
    // The cook bundles only the default pack's images the code names
    // (`nine-<name>`); the rest are left out on purpose.
    let defaults: std::collections::BTreeSet<&str> = layers[0]
        .images
        .iter()
        .flat_map(|m| m.values().map(|n| n.src.as_str()))
        .collect();
    let mut slots = BTreeMap::new();
    let mut warnings = Vec::new();
    for scheme in [Scheme::Light, Scheme::Dark] {
        for image in all.images(scheme).values() {
            if slots.contains_key(&image.src) {
                continue;
            }
            if defaults.contains(image.src.as_str()) && eng.assets.borrow().lacks(&image.src) {
                slots.insert(image.src.clone(), 0);
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
    // Its icon sets, each loaded once as a font: under the game's own
    // icons, the top layer's first.
    let mut under: Option<std::sync::Arc<og_ui_next::icons::IconAtlas>> = None;
    for src in all.icon_sets.iter().rev() {
        match crate::world::assets::load_font(&eng, src) {
            Ok(i) => {
                let assets = eng.assets.borrow();
                let font = assets.fonts.get(i);
                match font.and_then(|f| Some((f.face.clone()?, f.icons.clone()))) {
                    Some((face, names)) if !names.is_empty() => {
                        under = Some(std::sync::Arc::new(og_ui_next::icons::IconAtlas {
                            face: Some(face),
                            names,
                            under,
                            ..Default::default()
                        }));
                    }
                    _ => warnings.push(format!(
                        "og.ui: the pack's icon set {src} is not an icon set"
                    )),
                }
            }
            Err(e) => warnings.push(format!("og.ui: the pack's icon set {src}: {e}")),
        }
    }
    eng.render.borrow_mut().ui_icons.under = under;
    let sounds = Rc::new(sounds);
    let mut haptics_table = og_ui_next::tree::HapticTable::new();
    for (scope, events) in &all.haptics {
        let entry = haptics_table.entry(scope.clone()).or_default();
        for (event, preset) in events {
            if let Some(h) = og_ui_next::pack::parse_haptic(preset) {
                entry.insert(event.clone(), h);
            }
        }
    }
    let haptics_table = Rc::new(haptics_table);
    if let Some(next) = &mut eng.render.borrow_mut().ui_next {
        if let Some(tree) = &mut next.tree {
            tree.set_sounds(sounds.clone());
            tree.set_haptics(haptics_table.clone());
        }
        next.sounds = sounds;
        next.haptics_table = haptics_table;
        next.images = slots;
        next.icons = icons;
        // The glyphs draw the new pack's art.
        next.device = None;
        next.images_for = Some((pack, given));
        next.resolved = None;
        next.held.extend(warnings);
    }
}

/// The native pack at the exact canonical style ID.
pub(crate) fn pack_layer(engine: u32, id: &str) -> Option<og_ui_next::pack::Layer> {
    let eng = crate::engine::engine(engine).ok()?;
    let assets = eng.assets.borrow();
    let (bytes, bundle) = assets.blob(id).ok()?;
    let data = bundle.pack_data(id, bytes.bytes()).ok()?;
    og_ui_next::pack::Layer::from_bytes(&data).ok()
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
        && let Ok(input) = serde_json::from_str::<og_ui_next::pack::input::PackInput>(&seeds)
        && let Ok(layer) = og_ui_next::pack::Layer::from_input(input, Clone::clone)
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

/// A texture's size in texels (the render scene's), when it has one.
pub(crate) type TextureSize<'a> = dyn Fn(og_render::TextureHandle) -> Option<(u32, u32)> + 'a;

/// Where a UI box falls among what classes read of it: how many of the
/// theme's widths it reaches, and whether it is under 500 px tall and
/// wider than tall. A box that stays in one resolves every class alike
/// (an arbitrary `min-[…]:` width is not counted).
type SizeClass = (u8, bool, bool);

/// What a class resolver was made for: the pack, the dark scheme, high
/// contrast, the size class, the window when a class it resolved
/// reads the window's exact size, the locale's direction, touch device mode,
/// and reduced-motion preference.
pub(super) type Resolved = (
    Option<String>,
    [String; 4],
    String,
    bool,
    bool,
    SizeClass,
    Option<[f32; 2]>,
    bool,
    bool,
    bool,
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
#[allow(clippy::too_many_arguments)]
fn resolver(
    engine: u32,
    pack: Option<&str>,
    dark: bool,
    high_contrast: bool,
    touch: bool,
    reduce_motion: bool,
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
    let slots = slots.clone();
    // The classes of the pack's theme, variants and images in a scheme.
    let make = move |scheme: Scheme| {
        // The cook checked the theme: what it left out, it reported then.
        let theme = if high_contrast {
            pack.theme_json_high_contrast(scheme)
        } else {
            pack.theme_json(scheme)
        };
        // The text roles' families as the UI's fonts are configured.
        let theme = crate::present::faces::themed(engine, theme);
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
        classes
    };
    let other_scheme = if dark { Scheme::Light } else { Scheme::Dark };
    // The page's scheme's classes, and the other scheme's for a subtree
    // that names it (`scheme-dark` over dark art), made when first asked.
    let classes = RefCell::new([Some(make(scheme)), None]);
    // The active locale's direction, coarse pointer, and reduced motion.
    let env = og_ui_next::classes::Env {
        window,
        dark,
        touch,
        reduce_motion,
        rtl: crate::bind::i18n::rtl(engine),
        ..Default::default()
    };
    // The tree asks for an entry's state records (motion's `starting:`
    // and `exit:`; the input's); the inline style lies over the classes.
    let resolve: Resolve = Box::new(move |class, style, states, data| {
        let other = states & og_ui_next::classes::OTHER_SCHEME != 0;
        let mut all = classes.borrow_mut();
        let c = all[usize::from(other)].get_or_insert_with(|| make(other_scheme));
        let env = og_ui_next::classes::Env {
            states,
            dark: dark != other,
            ..env
        };
        let record = c.record_data(class, style, &env, data);
        sized.set(c.sized());
        record
    });
    (resolve, widths)
}

impl Next {
    /// Gives the tree the class resolver of the pack, the scheme and the
    /// box's size class, when it has another one: every op node restyles.
    pub(super) fn resolve(&mut self, engine: u32) {
        let Some(tree) = &mut self.tree else { return };
        let v = frame_of(engine).viewport();
        let window = [v.width, v.height];
        let dark = self.dark;
        // The page's `prefers-contrast: more`, as the host reports it.
        let high = v.high_contrast;
        let touch = tree.is_touch();
        let reduce_motion = settings_of(engine).reduced_motion;
        let at = size_class(&self.widths, window);
        let exact = self.sized.get().then_some(window);
        let seeds = configured_seeds(engine);
        let rtl = crate::bind::i18n::rtl(engine);
        let roles = crate::present::faces::roles(engine);
        if self
            .resolved
            .as_ref()
            .is_some_and(|(p, r, s, d, h, c, w, l, t, rm)| {
                *p == self.pack
                    && *r == roles
                    && *s == seeds
                    && *d == dark
                    && *h == high
                    && *c == at
                    && *w == exact
                    && *l == rtl
                    && *t == touch
                    && *rm == reduce_motion
            })
        {
            return;
        }
        let sized = Rc::new(std::cell::Cell::new(false));
        let (resolve, widths) = resolver(
            engine,
            self.pack.as_deref(),
            dark,
            high,
            touch,
            reduce_motion,
            window,
            &self.images,
            sized.clone(),
        );
        tree.set_resolver(resolve);
        self.sized = sized;
        self.resolved = Some((
            self.pack.clone(),
            roles,
            seeds,
            dark,
            high,
            size_class(&widths, window),
            exact,
            rtl,
            touch,
            reduce_motion,
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
