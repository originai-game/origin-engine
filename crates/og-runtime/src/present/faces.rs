//! The UI's font collection configured: the glue between what a game
//! loads and og-text's [`Collection`], which lays every TSX text out.
//!
//! Each font file the UI may draw from is registered once: the fonts
//! og.ui.configure names, the active style pack's `fonts`, the locale's
//! fonts and font packs, and the default face. The text roles are CSS
//! family lists ([`Faces::configure`]): the theme's `fontFamily`, with the
//! family of a font the game gave a role first. The same lists are the
//! theme fonts the class resolver hands NSH (`font-sans`, `font-display`,
//! `font-mono`, `font-rounded`) and fontique's generic families. A
//! character no family of a text's list draws comes from the fallback
//! list: the game's `fallbackFonts`, its role fonts, the pack's files, the
//! default face, then the locale's and the font packs a frame loaded (the
//! bundled Chinese one among them). A pack's file still being fetched
//! joins when it lands; its characters draw from the next face meanwhile.

use crate::engine::Engine;
use og_text::fonts::{Collection, FaceId, GenericFamily};
use og_ui_next::pack::Layer;
use serde_json::Value;
use std::collections::BTreeMap;

/// The text roles, as a theme's `fontFamily` names them and as UiDesc
/// gives each a font.
const ROLES: [(&str, &str); 4] = [
    ("sans", "font"),
    ("display", "displayFont"),
    ("mono", "monoFont"),
    ("rounded", "roundedFont"),
];

/// The faces TSX text is laid out from.
#[derive(Default)]
pub(crate) struct Faces {
    pub collection: Collection,
    /// Each loaded font (its index in the engine's assets) once registered.
    ids: BTreeMap<usize, FaceId>,
    /// What the generic families and the fallback list were last set from.
    set: Option<([String; 5], Vec<FaceId>, String)>,
}

impl Faces {
    /// Loaded font `index`'s file in the collection; `None` for a font
    /// that ships no file.
    pub(crate) fn face(&mut self, eng: &Engine, index: usize) -> Option<FaceId> {
        if let Some(id) = self.ids.get(&index) {
            return Some(*id);
        }
        let bytes = eng.assets.borrow().fonts.get(index)?.bytes.clone()?;
        let id = self.collection.add(bytes).ok()?;
        self.ids.insert(index, id);
        Some(id)
    }

    /// Whether any text can draw.
    pub(crate) fn any(&self) -> bool {
        !self.ids.is_empty()
    }

    /// Whether a registered face draws `c`.
    pub(crate) fn draws(&self, c: char) -> bool {
        self.ids.values().any(|&id| self.collection.covers(id, c))
    }
}

/// The font files of engine `eng`'s style pack (its `fonts`), first first.
pub(crate) fn style_files(eng: &Engine) -> Vec<String> {
    let style = eng
        .state
        .borrow()
        .ui_desc
        .get("style")
        .and_then(Value::as_f64)
        .and_then(crate::world::assets::index_of);
    style
        .and_then(|i| eng.assets.borrow().styles.get(i).map(|s| s.files.clone()))
        .unwrap_or_default()
}

/// Sets engine `eng`'s roles and fallbacks from its UiDesc, its style
/// pack, `style_fonts` (the pack's files fetched so far) and `chain`: the
/// default face, then the locale's fonts in order (loaded fonts by
/// index). Called whenever one of them changes, or a file lands.
pub(crate) fn configure(eng: &Engine, style_fonts: &[usize], chain: &[usize]) {
    let desc = eng.state.borrow().ui_desc.clone();
    let font_of = |field: &str| {
        desc.get(field)
            .and_then(Value::as_f64)
            .and_then(crate::world::assets::index_of)
    };
    // The style's own families for each role.
    let style = font_of("style");
    let layer = style
        .and_then(|i| eng.assets.borrow().styles.get(i).map(|s| s.id.clone()))
        .and_then(|id| super::ui_next::pack_layer(eng.id, &id));
    let default = Layer::default_pack();
    let mut rs = eng.render.borrow_mut();
    let faces = &mut rs.ui_faces;
    let mut fallback: Vec<FaceId> = Vec::new();
    let push = |list: &mut Vec<FaceId>, id: Option<FaceId>| {
        if let Some(id) = id
            && !list.contains(&id)
        {
            list.push(id);
        }
    };
    for h in desc
        .get("fallbackFonts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let i = h.as_f64().and_then(crate::world::assets::index_of);
        let id = i.and_then(|i| faces.face(eng, i));
        push(&mut fallback, id);
    }
    // A role's list: the game's font for it, the style's families, then
    // the list of the role it falls back to (rounded to display, display
    // and mono to sans); the default pack's sans ends them all.
    let mut roles: [String; 4] = Default::default();
    for (k, (key, field)) in ROLES.into_iter().enumerate() {
        let mut list: Vec<String> = Vec::new();
        if let Some(id) = font_of(field).and_then(|i| faces.face(eng, i)) {
            push(&mut fallback, Some(id));
            if let Some(name) = faces.collection.family_name(id) {
                list.push(format!("\"{}\"", name.replace('"', "")));
            }
        }
        list.extend(layer.as_ref().and_then(|l| l.font_family(key)));
        match k {
            0 => list.extend(default.font_family(key)),
            3 => list.push(roles[1].clone()),
            _ => list.push(roles[0].clone()),
        }
        roles[k] = list.join(", ");
    }
    let serif = layer
        .as_ref()
        .and_then(|l| l.font_family("serif"))
        .map_or_else(|| roles[0].clone(), |s| format!("{s}, {}", roles[0]));
    for &i in style_fonts.iter().chain(chain) {
        let id = faces.face(eng, i);
        push(&mut fallback, id);
    }
    let locale = crate::bind::i18n::language(eng.id);
    let [sans, display, mono, rounded] = roles.clone();
    let key = ([sans, display, mono, rounded, serif], fallback, locale);
    // Setting either lays every text out again: only when it changed, or
    // a file the lists name was registered since.
    if faces.set.as_ref() != Some(&key) {
        let (generics, fallback, locale) = &key;
        let c = &mut faces.collection;
        c.set_generic(GenericFamily::SansSerif, &generics[0]);
        c.set_generic(GenericFamily::Monospace, &generics[2]);
        c.set_generic(GenericFamily::UiRounded, &generics[3]);
        c.set_generic(GenericFamily::Serif, &generics[4]);
        // Every locale's texts first, then the active locale's own list
        // where fontique keeps one (Collection::set_fallback).
        c.set_fallback(None, fallback);
        c.set_fallback(Some(locale), fallback);
        faces.set = Some(key);
    }
    drop(rs);
    eng.state.borrow_mut().ui_font_roles = roles;
}

/// `theme` (a pack's, as JSON) with its text roles' `fontFamily` lists
/// replaced by engine `engine`'s ([`configure`]): what `font-sans`,
/// `font-display`, `font-mono` and `font-rounded` give a text.
pub(crate) fn themed(engine: u32, theme: String) -> String {
    let Ok(eng) = crate::engine::engine(engine) else {
        return theme;
    };
    let roles = eng.state.borrow().ui_font_roles.clone();
    if roles.iter().all(String::is_empty) {
        return theme;
    }
    let Ok(Value::Object(mut t)) = serde_json::from_str::<Value>(&theme) else {
        return theme;
    };
    let families = t
        .entry("fontFamily")
        .or_insert_with(|| Value::Object(Default::default()));
    if let Value::Object(f) = families {
        for ((key, _), list) in ROLES.into_iter().zip(roles) {
            f.insert(key.into(), Value::String(list));
        }
    }
    Value::Object(t).to_string()
}

/// The role lists the resolver's theme was made with, for its cache key.
pub(crate) fn roles(engine: u32) -> [String; 4] {
    crate::engine::engine(engine)
        .map(|eng| eng.state.borrow().ui_font_roles.clone())
        .unwrap_or_default()
}
