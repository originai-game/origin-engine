//! `og.ui`'s documents, layer stack and actions (docs/ui.md §2.1, §2.1, §2.1,
//! §16): `og.load.document` reads a cooked `og.UiDocument` into the world's
//! UI; `push`, `pop`, `replace` and `clear` manage the layer stack from
//! anywhere; the UI actions, glyphs and the action bar; the probe's `ui:`
//! commands (set, state, play, seek, push, pop, save, clear, action); and
//! the hot reload of an edited document or style in place.
//!
//! A viewmodel's Capitalised root (`Wallet.gold`) reads that resource as
//! of the end of the last tick ([`snapshot`]): the UI is built inside the
//! tick, where the world is the scheduler's.

use super::super::engine::{Engine, engine};
use super::super::parse;
use super::{BUILDING, ui_of, with_ui};
use crate::wire::{self, ToJson};
use og_ui::prelude::*;
use og_ui::{ActionDesc, ActionTrigger, Device, SharedUi, Ui, UiDocument};
use serde_json::{Map, Value, json};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

/// A document an engine loaded: its asset id and its cooked output's hash,
/// and the Rive artboards it plays with theirs.
#[derive(Clone, Debug)]
pub(crate) struct Loaded {
    id: String,
    hash: u64,
    rive: Vec<(String, u64)>,
}

#[derive(Default)]
pub(crate) struct Docs {
    loaded: Vec<Loaded>,
    /// The resources the documents read (their Capitalised roots).
    roots: BTreeSet<String>,
    /// Those resources as of the last tick's end.
    resources: Map<String, Value>,
    /// Styles' cooked hashes, by load index (a hot reload re-applies an edited one).
    styles: BTreeMap<usize, u64>,
}

/// The documents on one engine: what it loaded, and the resources they
/// read as of the last tick's end.
pub(crate) type ModuleState = RefCell<Docs>;

fn with<T>(engine: u32, f: impl FnOnce(&mut Docs) -> T) -> T {
    match crate::engine::engine(engine) {
        Ok(eng) => f(&mut eng.modules.docs.borrow_mut()),
        // An engine that is gone, or not made yet, has none: a fresh one.
        Err(_) => f(&mut Docs::default()),
    }
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
    fn media(&self, player: f64) -> Option<og_ui::bind::MediaState> {
        let (mut state, texture) = crate::video::media_state(self.engine, player as u64)?;
        state.texture = (texture != 0.0).then(|| self.texture(texture)).flatten();
        Some(state)
    }
}

pub(crate) fn env(engine: u32) -> RtEnv {
    RtEnv {
        engine,
        resources: with(engine, |d| d.resources.clone()),
        language: super::super::i18n::language(engine),
    }
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
/// bindings (before the frame ends). Pausing sends recorded input for the
/// next tick; it does not change simulation time from presentation code.
pub(crate) fn end_of_frame(
    engine: u32,
    ui: &mut Ui,
    ctx: &og_ecs::SystemContext<'_>,
) -> og_core::Result<()> {
    ui.build_layers(&env(engine));
    if let Some(scale) = ui.pause_time_scale(ctx.time_scale().scale) {
        let eng = super::super::engine::engine(engine)
            .map_err(|e| og_core::Error::new(og_core::ErrorKind::Invalid, e))?;
        let bytes = og_core::reflect::to_bytes(&og_input::PauseIntent { scale })?;
        crate::api::send_intent(&eng, Some(ctx), "og.PauseIntent", &bytes)
            .map_err(|e| og_core::Error::new(og_core::ErrorKind::Invalid, e))?;
    }
    let ops = ui.take_media_ops();
    if !ops.is_empty() {
        let eng = super::super::engine::engine(engine)
            .map_err(|e| og_core::Error::new(og_core::ErrorKind::Invalid, e))?;
        crate::video::send_media(&eng, ctx, &ops)
            .map_err(|e| og_core::Error::new(og_core::ErrorKind::Invalid, e))?;
    }
    Ok(())
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
    let (text, imports) = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(id)?;
        (
            bundle
                .document_text(id, bytes.bytes())
                .map_err(|e| e.to_string())?,
            bundle
                .document_imports(id, bytes.bytes())
                .map_err(|e| e.to_string())?,
        )
    };
    let doc = UiDocument::parse_with_imports(id, &text, imports).map_err(|e| e.join("\n"))?;
    let doc = match configured_style(eng).and_then(|h| super::assets::index_of(h as f64)) {
        Some(index) => eng
            .assets
            .borrow()
            .styles
            .get(index)
            .ok_or("the configured style is not alive")?
            .pack
            .apply_document(&doc)
            .map_err(|e| e.join("\n"))?,
        None => doc,
    };
    let tables = doc
        .strings
        .iter()
        .map(|id| {
            let assets = eng.assets.borrow();
            let (bytes, bundle) = assets.blob(id)?;
            let text = bundle
                .strings_text(id, bytes.bytes())
                .map_err(|e| e.to_string())?;
            og_i18n::StringTable::parse(&text).map_err(|e| format!("{id}: {e}"))
        })
        .collect::<Result<Vec<_>, String>>()?;
    for table in tables {
        let strings = table
            .entries
            .into_iter()
            .map(|(key, (text, _))| (key, text))
            .collect();
        super::super::i18n::base_strings(eng, &table.locale, &strings)?;
    }
    Ok(doc)
}

/// Load CSS textures by their authored asset paths, including transition covers.
fn read_textures(
    eng: &Engine,
    doc: &UiDocument,
) -> Result<Vec<(String, og_render::TextureHandle)>, String> {
    let mut paths = doc.sheet.assets.clone();
    paths.extend(doc.cover_image().map(str::to_owned));
    let mut textures = Vec::new();
    for path in paths {
        let ty = {
            let assets = eng.assets.borrow();
            // Missing URLs remain unresolved so cursor candidates can fall
            // through and image paint can report its missing asset.
            let Ok((_, bundle)) = assets.blob(&path) else {
                continue;
            };
            bundle.output(&path).map_err(|e| e.to_string())?.ty.clone()
        };
        if ty != "og.Texture" {
            continue;
        }
        let i = crate::assets::load_texture(eng, &path)
            .map_err(|e| format!("{}: CSS texture {path}: {e}", doc.source))?;
        textures.push((path, eng.render.borrow().textures[i]));
    }
    Ok(textures)
}

/// The alpha channels of the textures its floats' `shape-outside: url()`
/// names, read on the CPU as a cursor image is: an RGBA texture, or a
/// cooked one decoded again from the bundle (any natively; on the web a
/// lossless one, `kind pixel`). One that cannot be read is noted and its
/// float keeps its box.
fn read_shape_alphas(
    eng: &Engine,
    doc: &UiDocument,
    textures: &[(String, og_render::TextureHandle)],
) -> Vec<(String, og_ui::ImageAlpha)> {
    let mut out = Vec::new();
    for path in doc.shape_images() {
        let Some((_, texture)) = textures.iter().find(|(p, _)| *p == path) else {
            continue;
        };
        match crate::ui_cursor::read(eng, *texture, [0.0, 0.0, 1.0, 1.0]) {
            Ok((width, height, rgba)) => out.push((
                path,
                og_ui::ImageAlpha {
                    width,
                    height,
                    alpha: rgba.as_chunks::<4>().0.iter().map(|p| p[3]).collect(),
                },
            )),
            Err(why) => eng.render.borrow_mut().notes.push(format!(
                "{}: shape-outside: url({path}) {}; the float's box is its shape",
                doc.source,
                why.unwrap_or_else(|| "is not readable on the CPU".into())
            )),
        }
    }
    out
}

/// The UI materials a document's `-og-material` names, loaded by path.
#[allow(clippy::type_complexity)]
fn read_materials(
    eng: &Engine,
    doc: &UiDocument,
) -> Result<
    Vec<(
        String,
        (
            og_render::UiMaterialHandle,
            [[f32; 4]; og_render::UI_MATERIAL_PARAMS],
        ),
    )>,
    String,
> {
    doc.material_assets()
        .into_iter()
        .map(|path| {
            let m = crate::ui_draw::ui_material(eng, &path)
                .map_err(|e| format!("{}: -og-material: \"{path}\": {e}", doc.source))?;
            Ok((path, m))
        })
        .collect()
}

fn read_rive(
    eng: &Engine,
    doc: &UiDocument,
) -> Result<Vec<(String, u64, std::sync::Arc<og_ui::rive::Artboard>)>, String> {
    doc.rive_ids()
        .into_iter()
        .map(|id| {
            let raw = {
                let assets = eng.assets.borrow();
                let (bytes, bundle) = assets
                    .blob(&id)
                    .map_err(|e| format!("{}: og-rive src=\"{id}\": {e}", doc.source))?;
                bundle
                    .rive_artboard(&id, bytes.bytes())
                    .map_err(|e| e.to_string())?
            };
            let mut artboard = og_ui::rive::decode(&raw).map_err(|e| format!("{id}: {e}"))?;
            // Its images are textures the cook made beside it.
            let mut missing = None;
            artboard.bind_images(&mut |image| match crate::assets::load_texture(eng, image) {
                Ok(i) => eng.render.borrow().textures.get(i).copied(),
                Err(e) => {
                    missing.get_or_insert(format!("{id}: {image}: {e}"));
                    None
                }
            });
            // Its text's fonts too, each as the UI draws with it.
            artboard.bind_fonts(&mut |font| {
                let loaded = crate::assets::load_font(eng, font)
                    .and_then(|i| super::ui_config::ui_font(eng, crate::assets::handle(i) as f64));
                match loaded {
                    Ok(f) => Some(f),
                    Err(e) => {
                        missing.get_or_insert(format!("{id}: {font}: {e}"));
                        None
                    }
                }
            });
            if let Some(e) = missing {
                return Err(e);
            }
            Ok((id.clone(), hash_of(eng, &id), std::sync::Arc::new(artboard)))
        })
        .collect()
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
    let rive = read_rive(eng, &doc)?;
    let materials = read_materials(eng, &doc)?;
    let textures = read_textures(eng, &doc)?;
    let alphas = read_shape_alphas(eng, &doc, &textures);
    let name = doc.name.clone();
    let hash = hash_of(eng, &id);
    let hashes: Vec<(String, u64)> = rive.iter().map(|(r, h, _)| (r.clone(), *h)).collect();
    with(eng.id, |d| {
        d.roots.extend(roots_of(&doc));
        match d.loaded.iter_mut().find(|l| l.id == id) {
            Some(l) => (l.hash, l.rive) = (hash, hashes),
            None => d.loaded.push(Loaded {
                id: id.clone(),
                hash,
                rive: hashes,
            }),
        }
    });
    // A <video> or <audio> asks its player through og.MediaIntent.
    if doc
        .nodes
        .iter()
        .any(|n| matches!(n.kind, og_ui::doc::NodeKind::Element("video" | "audio")))
    {
        crate::lazy::ensure(eng, eng.world()?, crate::video::MEDIA_INTENT)?;
    }
    let faces = family_faces(eng, &doc)?;
    let ui = eng.state.borrow().ui.clone().ok_or("the world has no UI")?;
    let mut ui = ui.lock();
    for (r, _, artboard) in rive {
        ui.define_rive(&r, artboard);
    }
    for (path, (handle, defaults)) in materials {
        ui.define_material(&path, handle, defaults);
    }
    for (path, texture) in textures {
        ui.define_texture(&path, texture);
    }
    for (path, alpha) in alphas {
        ui.define_texture_alpha(&path, alpha);
    }
    define(&mut ui, doc, faces);
    Ok(name)
}

/// The faces a document's `@font-face` rules declare, each from its first
/// `url()` source the bundle has (the cook bundled it as a font; a
/// `local()` source names no font the engine has), limited to its
/// `unicode-range`.
fn family_faces(eng: &Engine, doc: &UiDocument) -> Result<Vec<og_ui::FamilyFace>, String> {
    // Pack faces have a pack owner so replacing a pack also removes its fonts.
    let authored =
        UiDocument::parse_with_imports(&doc.source, &doc.text, doc.sheet.imports.clone())
            .map_err(|e| e.join("\n"))?;
    sheet_faces(eng, &authored.sheet, &doc.source, &doc.name)
}

pub(super) fn sheet_faces(
    eng: &Engine,
    sheet: &og_ui::doc::at_rules::Sheet,
    source: &str,
    owner: &str,
) -> Result<Vec<og_ui::FamilyFace>, String> {
    let mut out = Vec::new();
    for face in &sheet.font_faces {
        let Some(src) = face
            .src
            .iter()
            .find(|s| !s.starts_with("local:") && eng.assets.borrow().bundle_for(s).is_ok())
        else {
            continue;
        };
        let i = crate::assets::load_font(eng, src)
            .map_err(|e| format!("{source}: @font-face {}: {src}: {e}", face.family))?;
        let mut font = super::ui_config::ui_font(eng, crate::assets::handle(i) as f64)?;
        font.metrics = font.metrics.with_range(&face.unicode_range);
        let weight = |w: f32| w.round().clamp(1.0, 1000.0) as u16;
        out.push(og_ui::FamilyFace {
            family: face.family.clone(),
            document: owner.to_owned(),
            weight: (weight(face.weight.0), weight(face.weight.1)),
            style: match face.style.as_str() {
                "italic" => og_ui::FaceStyle::Italic,
                "oblique" => og_ui::FaceStyle::Oblique,
                _ => og_ui::FaceStyle::Normal,
            },
            font,
            ..og_ui::FamilyFace::default()
        });
    }
    Ok(out)
}

/// Defines `doc` in the UI with its named families' faces in place of the
/// ones it declared before.
fn define(ui: &mut og_ui::Ui, doc: UiDocument, faces: Vec<og_ui::FamilyFace>) {
    let had = ui.fonts().families.iter().any(|f| f.document == doc.name);
    if had || !faces.is_empty() {
        let mut fonts = ui.fonts().clone();
        fonts.families.retain(|f| f.document != doc.name);
        fonts.families.extend(faces);
        ui.set_fonts(fonts);
    }
    ui.define_document(doc);
}

/// `og.load.document`.
pub fn og_load_document(world: u32, path: String) -> Result<(), String> {
    match engine(world).and_then(|eng| load_document(&eng, &path)) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
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
        // An edited .riv reloads its document's artboards too.
        if now == l.hash && l.rive.iter().all(|(r, h)| hash_of(eng, r) == *h) {
            continue;
        }
        match read_document(eng, &l.id).and_then(|doc| {
            Ok((
                read_rive(eng, &doc)?,
                read_materials(eng, &doc)?,
                read_textures(eng, &doc)?,
                family_faces(eng, &doc)?,
                doc,
            ))
        }) {
            Ok((rive, materials, textures, faces, doc)) => {
                let alphas = read_shape_alphas(eng, &doc, &textures);
                let hashes: Vec<(String, u64)> =
                    rive.iter().map(|(r, h, _)| (r.clone(), *h)).collect();
                with(eng.id, |d| {
                    d.roots.extend(roots_of(&doc));
                    if let Some(x) = d.loaded.iter_mut().find(|x| x.id == l.id) {
                        (x.hash, x.rive) = (now, hashes);
                    }
                });
                let mut ui = ui.lock();
                for (r, _, artboard) in rive {
                    ui.define_rive(&r, artboard);
                }
                for (path, (handle, defaults)) in materials {
                    ui.define_material(&path, handle, defaults);
                }
                for (path, texture) in textures {
                    ui.define_texture(&path, texture);
                }
                for (path, alpha) in alphas {
                    ui.define_texture_alpha(&path, alpha);
                }
                define(&mut ui, doc, faces);
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
        let before = with(eng.id, |d| d.styles.get(&i).copied());
        if before.is_none_or(|b| b == now) {
            continue;
        }
        match super::ui_config::read_style(eng, &id) {
            Ok(style) => {
                eng.assets.borrow_mut().styles[i] = style;
                with(eng.id, |d| d.styles.insert(i, now));
                changed = true;
            }
            Err(e) => failed.push(format!("og dev: {e} (the old style stays)")),
        }
    }
    if (changed || configured_style(eng).is_some())
        && let Err(reason) = super::og_ui_configure(eng.id, Some("{}".into()))
    {
        failed.push(format!("og dev: the style did not apply: {reason}"));
    }
    failed
}

/// The style the last og.ui.configure named (its UiDesc, as configure
/// merged and read it).
fn configured_style(eng: &Engine) -> Option<u64> {
    let st = eng.state.borrow();
    wire::value::<wire::UiDesc>(&st.ui_desc, "desc").ok()?.style
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

// ── og.ui: the layer stack, from anywhere ───────────────────────────────────

fn vm_of(v: Option<String>) -> Value {
    parse(v.as_deref()).unwrap_or(Value::Null)
}

fn on_ui<T>(world: u32, f: impl FnOnce(&mut Ui) -> T) -> Result<T, String> {
    let ui: SharedUi = ui_of(world).ok_or("the world has no UI")?;
    let mut g = ui.lock();
    Ok(f(&mut g))
}

pub fn og_ui_push(
    world: u32,
    screen: String,
    vm: Option<String>,
) -> Result<serde_json::Value, String> {
    match on_ui(world, |ui| ui.push_screen(&screen, vm_of(vm))).and_then(|r| r) {
        Ok(serial) => Ok(json!(serial)),
        Err(e) => Err(e),
    }
}

pub fn og_ui_pop(world: u32) -> String {
    on_ui(world, |ui| ui.pop_screen())
        .ok()
        .flatten()
        .unwrap_or_default()
}

pub fn og_ui_replace(
    world: u32,
    screen: String,
    vm: Option<String>,
) -> Result<serde_json::Value, String> {
    match on_ui(world, |ui| ui.replace_screen(&screen, vm_of(vm))).and_then(|r| r) {
        Ok(serial) => Ok(json!(serial)),
        Err(e) => Err(e),
    }
}

pub fn og_ui_clear(world: u32) {
    let _ = on_ui(world, |ui| ui.clear_screens());
}

/// A layer as og.ui.layers answers it (IDL `ui.UiLayer`).
fn layer_wire(l: &og_ui::LayerInfo) -> crate::wire::UiLayer {
    let s = &l.spec;
    wire::UiLayer {
        serial: l.serial,
        screen: l.screen.clone(),
        mount: l
            .mount
            .as_ref()
            .map_or(String::new(), |m| format!("{}:{}", m.scope, m.id)),
        kind: s.kind.name().to_string(),
        input: s.input.name().to_string(),
        capture: s.capture,
        pause: s.pause,
        under: s.under.name().to_string(),
        back: match &s.back {
            og_ui::doc::Back::Pop => "pop".to_string(),
            og_ui::doc::Back::None => "none".into(),
            og_ui::doc::Back::Emit(n) => format!("emit {n}"),
        },
        focus: s.focus.clone(),
        remember: s.remember,
        top: l.top,
        active: l.active,
        hidden: l.hidden,
        inert: l.inert,
    }
}

pub fn og_ui_layers(world: u32) -> Vec<wire::UiLayer> {
    on_ui(world, |ui| {
        ui.layer_stack().iter().map(layer_wire).collect()
    })
    .unwrap_or_default()
}

pub fn og_ui_screen_vm(world: u32, screen: String) -> Option<Value> {
    on_ui(world, |ui| ui.screen_vm(&screen).cloned())
        .ok()
        .flatten()
}

pub fn og_ui_doc_events(world: u32) -> Vec<wire::UiDocEvent> {
    on_ui(world, |ui| {
        ui.take_doc_events()
            .into_iter()
            .map(|e| wire::UiDocEvent {
                mount: e.mount,
                path: e.path,
                row_key: e.row_key,
                kind: e.kind,
                name: e.name,
                value: e.value,
            })
            .collect()
    })
    .unwrap_or_default()
}

pub fn og_ui_game_input(world: u32) -> bool {
    on_ui(world, |ui| ui.game_input()).unwrap_or(true)
}

pub fn og_ui_paused(world: u32) -> bool {
    on_ui(world, |ui| ui.paused()).unwrap_or(false)
}

// ── og.ui: actions and glyphs ───────────────────────────────────────────────

pub fn og_ui_define_action(world: u32, desc: String) -> Result<(), String> {
    let d: crate::wire::UiActionDesc = crate::wire::arg(Some(&desc), "desc")?;
    if d.name.is_empty() {
        return Err("an action has a name".into());
    }
    let trigger = match d.trigger {
        crate::wire::UiActionTrigger::Release => ActionTrigger::Release,
        // A hold of 0 is 500 ms.
        crate::wire::UiActionTrigger::Hold => {
            ActionTrigger::Hold(if d.hold_ms == 0.0 { 500.0 } else { d.hold_ms })
        }
        crate::wire::UiActionTrigger::Press => ActionTrigger::Press,
    };
    let desc = ActionDesc {
        name: d.name,
        keys: d.keys,
        buttons: d.buttons,
        trigger,
        toggle: d.toggle,
        label: d.label,
        action: d.action.unwrap_or_default(),
    };
    match on_ui(world, |ui| ui.define_action(desc)) {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn og_ui_action(name: String) -> bool {
    with_ui("og.ui.action", |ui| ui.action(&name)).unwrap_or(false)
}

pub fn og_ui_action_held(name: String) -> bool {
    with_ui("og.ui.actionHeld", |ui| ui.action_held(&name)).unwrap_or(false)
}

/// A UI action's glyph on the UI's device (IDL `ui.UiGlyph`).
fn glyph_wire(ui: &Ui, action: &str) -> Option<crate::wire::UiGlyph> {
    let g = ui.glyph(action)?;
    Some(wire::UiGlyph {
        icon: g.icon,
        text: g.text,
        device: wire::UiDevice::parse(ui.device().name()).unwrap_or_default(),
    })
}

pub fn og_ui_glyph(world: u32, action: String) -> Option<crate::wire::UiGlyph> {
    on_ui(world, |ui| glyph_wire(ui, &action)).ok().flatten()
}

pub fn og_ui_frame_glyph(action: String) -> Value {
    let g = with_ui("og.ui.frameGlyph", |ui| glyph_wire(ui, &action)).flatten();
    // Not a UI action: a gameplay action of og.input, on the device used
    // last in the world whose frame is being built (an ability slot's key).
    match (g, BUILDING.with(Cell::get)) {
        (None, Some(world)) => super::super::og_input_glyph(world, action, None).to_json(),
        (g, _) => g.to_json(),
    }
}

pub fn og_ui_device(world: u32) -> String {
    on_ui(world, |ui| ui.device().name().to_string()).unwrap_or_else(|_| "keyboardMouse".into())
}

/// No host opens a file picker yet: false on the web and natively.
pub fn og_ui_file_picker(world: u32) -> bool {
    on_ui(world, |ui| ui.file_picker()).unwrap_or(false)
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

pub fn og_ui_action_bar(world: u32) -> Vec<wire::UiActionHint> {
    on_ui(world, |ui| {
        ui.action_bar()
            .into_iter()
            .map(|h| wire::UiActionHint {
                action: h.action,
                hint: h.hint,
                icon: h.glyph.icon,
                text: h.glyph.text,
                hold: h.hold,
            })
            .collect()
    })
    .unwrap_or_default()
}

pub fn og_ui_document_schema() -> String {
    og_ui::doc::schema::to_json().to_string()
}

// ── live edits (Tune and the probe) ─────────────────────────────────────────

pub fn og_ui_set_prop(world: u32, path: String, prop: String, value: String) -> Result<(), String> {
    match on_ui(world, |ui| ui.set_doc_prop(&path, &prop, &value)).and_then(|r| r) {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
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

pub fn og_ui_save_edits(world: u32) -> Result<serde_json::Value, String> {
    match engine(world).and_then(|eng| save_edits(&eng)) {
        Ok(files) => Ok(json!(files)),
        Err(e) => Err(e),
    }
}

// ── the probe's ui: commands ────────────────────────────────────────────────

const USAGE: &str = "ui:set:<path>.<prop>=<value> | ui:material:<asset>.<parameter>=<numbers> | ui:state:<path>.<group>=<state> | ui:play:<path>.<timeline> | ui:seek:<path>.<timeline>=<ms> | ui:push:<screen>[:<fixture>] | ui:pop | ui:save | ui:clear[:<path>] | ui:action:<name>";

pub(crate) fn register_commands(probe: &mut og_probe::Probe, engine_id: u32) {
    let _ = probe.register_command(
        "ui",
        og_probe::CommandDesc {
            usage: USAGE.into(),
            doc: "adjust the UI live (docs/ui.md §6): set any prop of a document element (presentation only, marked in dump_ui as an edit; ui:save writes the edits into the documents' .ogt), force a state group's state (an empty state lets it go), play or seek a document timeline on an element (seek holds it there), push a screen with a fixture of its document (its first by default), pop the top layer, drop edits, fire a UI action".into(),
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
        "material" => {
            let (target, value) = rest.split_once('=').ok_or_else(|| {
                format!("write ui:material:<asset>.<parameter>=<numbers>; {USAGE}")
            })?;
            let (path, parameter) = split_target(target)?;
            super::super::ui_draw::tune_material(&eng, path, parameter, value)
        }
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
