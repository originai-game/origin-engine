//! Looks (`.ogt` of `type look`): a frame's art direction as one
//! git-diffable file. A look names the engine resources it sets (the IDL's
//! `RenderSettings`, `PostEffects`, `Stylise`, `MaterialStyle`, `Fog`, `Sky`
//! and `Environment` for the frame, and the art contract's `ArtPalette`,
//! `ArtPixels`, `ArtTexels`, `ArtShape`, `ArtLine`, `ArtMotion` and
//! `ArtCamera` for the assets) with their fields by reflected name, the UI
//! style, and the antialiasing it wants, and may build on another look:
//!
//! ```text
//! type look
//! version 1
//! ---
//! name "Dusk ink"
//! extends "anime-cel-3d"        # a built-in preset (templates/looks/<slug>.ogt),
//!                               # the game's own assets/looks/<slug>.ogt first,
//!                               # or another look's asset id ("looks/base.ogt")
//! mood "warm"                   # bright, dark, warm, cold, soft or high-contrast:
//!                               # in a preset, the mood it is drawn in; in any
//!                               # other look, the mood asked of the preset its
//!                               # chain reaches (`MOODS`, the cook lays the
//!                               # mood's layer over that preset)
//! variant "8bit"                # one of the variants of the preset its chain
//!                               # reaches (templates/looks/variants/<slug>/):
//!                               # the variant's layer is laid over that preset
//!                               # before the mood's
//! ui "paper"                   # the UI style applied with the look: a built-in
//!                               # (templates/styles/<slug>), the game's own
//!                               # styles/<slug>/style.ogt first, or an asset id
//! antialias off                 # tier, off, msaa4 or taa
//! RenderSettings {
//!   toneMapping agx
//!   exposure 1.2
//! }
//! PostEffects {
//!   outline true
//!   outlineColor "#1b1420"      # a colour as three linear numbers or an sRGB hex
//!   lut "luts/dusk.cube"        # a handle field takes the asset's id
//! }
//! Stylise {
//!   posterize 6
//!   dither bayer4
//!   palette "palettes/dusk.hex"
//! }
//! MaterialStyle {
//!   toon true
//! }
//! ArtPalette {
//!   colors "@palettes/dusk16"   # the palette the assets' texels sit on
//!   rampSteps 4
//!   accents 3 7                 # a list on its one line
//! }
//! ArtLine {
//!   width 1
//!   color "#1b1420"
//! }
//! ```
//!
//! This module reads and writes the text ([`LookFile`]) and merges a look
//! over the one it extends; the cook (`cook::look`) resolves `extends`,
//! checks every field against the resources' reflected schema (the IDL's
//! records), turns hex colours into linear numbers, and writes the flattened
//! result, so the cooked blob ([`crate::format::Look`]) is the whole look
//! and the runtime never resolves anything but the assets it names.

use og_core::ogt::{Block, Document, Entry, Value};

/// The `.ogt` type of a look.
pub const LOOK_TYPE: &str = "look";

/// The resources a look may set, by IDL name (each registers as `og.<name>`):
/// the frame's render resources, then the art contract (the IDL's `art`
/// module, docs/art.md §2.1), what every asset of the style holds.
pub const LOOK_RESOURCES: [&str; 14] = [
    "RenderSettings",
    "PostEffects",
    "Stylise",
    "MaterialStyle",
    "Fog",
    "Sky",
    "Environment",
    "ArtPalette",
    "ArtPixels",
    "ArtTexels",
    "ArtShape",
    "ArtLine",
    "ArtMotion",
    "ArtCamera",
];

/// The art contract's resources among [`LOOK_RESOURCES`]: what the cook
/// reads from the project's look (project.ogt's `look`).
pub const ART_RESOURCES: [&str; 7] = [
    "ArtPalette",
    "ArtPixels",
    "ArtTexels",
    "ArtShape",
    "ArtLine",
    "ArtMotion",
    "ArtCamera",
];

/// Fields game code animates tick by tick: a look never sets them, and
/// applying one keeps their current values.
pub const ANIMATED_FIELDS: [(&str, &str); 2] =
    [("PostEffects", "flash"), ("PostEffects", "shakeTrauma")];

/// The values of `antialias` (the IDL's `render.Antialias`).
pub const ANTIALIAS: [&str; 4] = ["tier", "off", "msaa4", "taa"];

/// The moods a look takes (`mood "<word>"`; `MOODS` in
/// `packages/knowledge/scripts/brief.ts`). Each is a
/// layer of the frame's grading, light and, for a palette-bound style, its
/// palette (`templates/looks/moods/`), docs/art.md §2.10.
pub const MOODS: [&str; 6] = ["bright", "dark", "warm", "cold", "soft", "high-contrast"];

/// The longest `extends` chain the cook follows.
pub const MAX_EXTENDS: usize = 8;

/// The top-level keys of a look file.
pub const LINES: &[og_core::ogt::line::Line] = &[
    og_core::ogt::line::Line::new("name", "\"<name>\"", "", "Display name.", ""),
    og_core::ogt::line::Line::new(
        "extends",
        "\"<look>\" ...",
        "",
        "Base looks, nearest first; at most eight inheritance levels. Asset id or a look slug.",
        "",
    ),
    og_core::ogt::line::Line::new(
        "mood",
        "\"<mood>\"",
        "",
        "Mood from MOODS; applied as the existing preset's explicit layer.",
        "",
    ),
    og_core::ogt::line::Line::new(
        "variant",
        "\"<variant>\"",
        "",
        "Existing preset variant slug: lowercase alphanumeric words separated by hyphens.",
        "",
    ),
    og_core::ogt::line::Line::new("ui", "\"<style>\"", "", "UI style slug or asset id.", ""),
    og_core::ogt::line::Line::new(
        "antialias",
        "<mode>",
        "",
        "Antialias mode from ANTIALIAS.",
        "",
    ),
];

/// The seed colours a look gives the UI (`uiSeeds { primary "#rrggbb" }`):
/// og-ui's `Seeds` (IDL `UiSeeds`), each an sRGB hex colour.
pub const UI_SEEDS: [&str; 8] = [
    "primary", "accent", "neutral", "danger", "warn", "success", "info", "team",
];

/// One resource block: its IDL name and its fields in file order, each with
/// its values as written (numbers, `true`/`false`, a variant word, or a
/// string: an asset id or an sRGB hex colour).
#[derive(Clone, Debug, PartialEq)]
pub struct LookBlock {
    pub resource: String,
    pub fields: Vec<(String, Vec<Value>, usize)>,
}

impl LookBlock {
    pub fn get(&self, field: &str) -> Option<&[Value]> {
        self.fields
            .iter()
            .find(|(f, _, _)| f == field)
            .map(|(_, v, _)| v.as_slice())
    }
}

/// A look file as written (or as cooked: flattened, `extends` then lists
/// the whole chain, nearest first).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LookFile {
    pub name: Option<String>,
    pub extends: Vec<String>,
    /// The mood: a preset's own, or the one a game's look asks of its
    /// preset; the cooked look carries the mood it was laid in.
    pub mood: Option<String>,
    /// The variant of the preset the chain reaches (docs/art.md §2.11): one
    /// a game's look asks for; the cooked look carries the variant it was
    /// laid in.
    pub variant: Option<String>,
    pub ui: Option<String>,
    /// The UI's seed colours ([`UI_SEEDS`] names, sRGB hex), so the UI's
    /// palette matches the scene: applying the look gives them to the UI
    /// with its style.
    pub ui_seeds: Vec<(String, String)>,
    pub antialias: Option<String>,
    pub blocks: Vec<LookBlock>,
}

fn one_string(e: &Entry) -> Result<String, String> {
    match e.values.as_slice() {
        [Value::String(s)] => Ok(s.clone()),
        _ => Err(format!("line {}: '{}' takes one string", e.line, e.key)),
    }
}

impl LookFile {
    /// Parses a look's text. Checks the grammar only: which resources and
    /// keys exist; the fields' names and values are the cook's to check
    /// against the reflected schema.
    pub fn parse(text: &str) -> Result<LookFile, String> {
        let doc = Document::parse(text).map_err(|e| e.to_string())?;
        if doc.ty != LOOK_TYPE {
            return Err(format!("type {}: a look is `type look`", doc.ty));
        }
        if doc.version != 1 {
            return Err(format!("version {}: looks are version 1", doc.version));
        }
        let mut look = LookFile::default();
        let mut seen_top: Vec<&str> = Vec::new();
        for e in &doc.body.entries {
            if e.key == "uiSeeds" {
                let block = e
                    .block
                    .as_ref()
                    .filter(|_| e.values.is_empty())
                    .ok_or_else(|| {
                        format!(
                            "line {}: 'uiSeeds' takes a block: uiSeeds {{ primary \"#rrggbb\" }}",
                            e.line
                        )
                    })?;
                if !look.ui_seeds.is_empty() {
                    return Err(format!("line {}: a second 'uiSeeds' block", e.line));
                }
                for f in &block.entries {
                    if !UI_SEEDS.contains(&f.key.as_str()) {
                        return Err(format!(
                            "line {}: '{}' is not a UI seed: {}",
                            f.line,
                            f.key,
                            UI_SEEDS.join(", ")
                        ));
                    }
                    let hex = one_string(f)?;
                    let six = hex
                        .strip_prefix('#')
                        .is_some_and(|h| h.len() == 6 && h.chars().all(|c| c.is_ascii_hexdigit()));
                    if !six {
                        return Err(format!(
                            "line {}: uiSeeds.{} is an sRGB \"#rrggbb\"",
                            f.line, f.key
                        ));
                    }
                    if look.ui_seeds.iter().any(|(n, _)| n == &f.key) {
                        return Err(format!("line {}: uiSeeds.{} is set twice", f.line, f.key));
                    }
                    look.ui_seeds.push((f.key.clone(), hex));
                }
                continue;
            }
            if let Some(block) = &e.block {
                if !LOOK_RESOURCES.contains(&e.key.as_str()) {
                    return Err(format!(
                        "line {}: '{}' is not a resource a look sets: {}",
                        e.line,
                        e.key,
                        LOOK_RESOURCES.join(", ")
                    ));
                }
                if !e.values.is_empty() {
                    return Err(format!("line {}: '{} {{' takes no values", e.line, e.key));
                }
                if look.blocks.iter().any(|b| b.resource == e.key) {
                    return Err(format!("line {}: a second '{}' block", e.line, e.key));
                }
                let mut fields: Vec<(String, Vec<Value>, usize)> = Vec::new();
                for f in &block.entries {
                    if f.block.is_some() {
                        return Err(format!(
                            "line {}: {}.{} opens a block; a field takes values only",
                            f.line, e.key, f.key
                        ));
                    }
                    // A key alone is an empty list; the cook refuses it for
                    // any other field (`cook::look`, which reads the records).
                    if fields.iter().any(|(n, _, _)| n == &f.key) {
                        return Err(format!("line {}: {}.{} is set twice", f.line, e.key, f.key));
                    }
                    if ANIMATED_FIELDS.contains(&(e.key.as_str(), f.key.as_str())) {
                        return Err(format!(
                            "line {}: {}.{} is animated by game code tick by tick; a look does not set it",
                            f.line, e.key, f.key
                        ));
                    }
                    fields.push((f.key.clone(), f.values.clone(), f.line));
                }
                look.blocks.push(LookBlock {
                    resource: e.key.clone(),
                    fields,
                });
                continue;
            }
            if e.key == "skin" {
                return Err(format!(
                    "line {}: 'skin' is now 'ui': `ui \"<slug>\"` names a built-in style (pixel, cartoon, fantasy, scifi, paper, neon) or the game's own styles/<slug>/style.ogt",
                    e.line
                ));
            }
            if !LINES.iter().any(|f| f.key == e.key) {
                return Err(format!(
                    "line {}: '{}' is not a look entry: {}, or a resource block ({})",
                    e.line,
                    e.key,
                    LINES.iter().map(|f| f.key).collect::<Vec<_>>().join(", "),
                    LOOK_RESOURCES.join(", ")
                ));
            }
            if seen_top.contains(&e.key.as_str()) {
                return Err(format!("line {}: '{}' is given twice", e.line, e.key));
            }
            seen_top.push(
                LINES
                    .iter()
                    .find(|f| f.key == e.key)
                    .expect("a top key")
                    .key,
            );
            match e.key.as_str() {
                "name" => look.name = Some(one_string(e)?),
                "mood" => {
                    let m = one_string(e)?;
                    if !MOODS.contains(&m.as_str()) {
                        return Err(format!(
                            "line {}: 'mood' is one of {}",
                            e.line,
                            MOODS.join(", ")
                        ));
                    }
                    look.mood = Some(m);
                }
                "variant" => {
                    let v = one_string(e)?;
                    let word = !v.is_empty()
                        && v.split('-').all(|w| {
                            !w.is_empty()
                                && w.chars()
                                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
                        });
                    if !word {
                        return Err(format!(
                            "line {}: 'variant' names one of the preset's variants, a lowercase-hyphen word",
                            e.line
                        ));
                    }
                    look.variant = Some(v);
                }
                "ui" => look.ui = Some(one_string(e)?),
                "extends" => {
                    look.extends = e
                        .values
                        .iter()
                        .map(|v| match v {
                            Value::String(s) if !s.is_empty() => Ok(s.clone()),
                            _ => Err(format!(
                                "line {}: 'extends' takes a preset's slug or a look's asset id, as a string",
                                e.line
                            )),
                        })
                        .collect::<Result<_, _>>()?;
                    if look.extends.is_empty() {
                        return Err(format!("line {}: 'extends' names no look", e.line));
                    }
                }
                _ => match e.values.as_slice() {
                    [Value::Word(w)] if ANTIALIAS.contains(&w.as_str()) => {
                        look.antialias = Some(w.clone())
                    }
                    _ => {
                        return Err(format!(
                            "line {}: 'antialias' is one of {}",
                            e.line,
                            ANTIALIAS.join(", ")
                        ));
                    }
                },
            }
        }
        Ok(look)
    }

    /// Whether the look's art blocks ([`ART_RESOURCES`]) set any field off
    /// its zero value: the art contract's switch (docs/art.md §2.3). Under
    /// a look whose art blocks are absent or all at zero the cook conforms
    /// no texture, painted or not, and a painted 2D frame paints on screen.
    pub fn constrains_art(&self) -> bool {
        self.blocks
            .iter()
            .filter(|b| ART_RESOURCES.contains(&b.resource.as_str()))
            .any(|b| {
                b.fields.iter().any(|(_, v, _)| {
                    v.iter().any(|x| match x {
                        Value::Number(n) => n.as_f64() != 0.0,
                        Value::Word(w) => !matches!(
                            w.as_str(),
                            "false" | "free" | "any" | "none" | "outside" | "diamond"
                        ),
                        Value::String(_) => true,
                    })
                })
            })
    }

    /// The canonical text: top-level keys, then the blocks in
    /// [`LOOK_RESOURCES`] order, fields as given.
    pub fn to_text(&self) -> String {
        let mut doc = Document::new(LOOK_TYPE, 1);
        if let Some(n) = &self.name {
            doc.body
                .push(Entry::new("name").with(Value::String(n.clone())));
        }
        if !self.extends.is_empty() {
            let mut e = Entry::new("extends");
            e.values = self.extends.iter().cloned().map(Value::String).collect();
            doc.body.push(e);
        }
        if let Some(m) = &self.mood {
            doc.body
                .push(Entry::new("mood").with(Value::String(m.clone())));
        }
        if let Some(v) = &self.variant {
            doc.body
                .push(Entry::new("variant").with(Value::String(v.clone())));
        }
        if let Some(s) = &self.ui {
            doc.body
                .push(Entry::new("ui").with(Value::String(s.clone())));
        }
        if !self.ui_seeds.is_empty() {
            let mut inner = Block::new();
            for (name, hex) in &self.ui_seeds {
                inner.push(Entry::new(name.clone()).with(Value::String(hex.clone())));
            }
            doc.body.push(Entry::new("uiSeeds").with_block(inner));
        }
        if let Some(a) = &self.antialias {
            doc.body
                .push(Entry::new("antialias").with(Value::Word(a.clone())));
        }
        for r in LOOK_RESOURCES {
            if let Some(b) = self.blocks.iter().find(|b| b.resource == r) {
                let mut inner = Block::new();
                for (f, v, _) in &b.fields {
                    let mut e = Entry::new(f.clone());
                    e.values = v.clone();
                    inner.push(e);
                }
                doc.body.push(Entry::new(r).with_block(inner));
            }
        }
        doc.to_text().expect("a parsed look writes back")
    }

    /// This look laid over `base` (the look it extends): the name is this
    /// look's own (a look is not called after its base), its ui style and
    /// antialias win where given, its UI seeds win seed by seed, a resource either names takes both
    /// blocks' fields with this look's winning field by field, the mood
    /// and variant this look's where it names one, and `extends`
    /// becomes `base_id` followed by `base`'s own chain.
    pub fn over(&self, base: &LookFile, base_id: &str) -> LookFile {
        let mut blocks: Vec<LookBlock> = base.blocks.clone();
        for b in &self.blocks {
            match blocks.iter_mut().find(|x| x.resource == b.resource) {
                Some(x) => {
                    for (f, v, line) in &b.fields {
                        match x.fields.iter_mut().find(|(n, _, _)| n == f) {
                            Some(slot) => *slot = (f.clone(), v.clone(), *line),
                            None => x.fields.push((f.clone(), v.clone(), *line)),
                        }
                    }
                }
                None => blocks.push(b.clone()),
            }
        }
        let mut ui_seeds = base.ui_seeds.clone();
        for (name, hex) in &self.ui_seeds {
            match ui_seeds.iter_mut().find(|(n, _)| n == name) {
                Some(slot) => slot.1 = hex.clone(),
                None => ui_seeds.push((name.clone(), hex.clone())),
            }
        }
        let mut extends = vec![base_id.to_string()];
        extends.extend(base.extends.iter().cloned());
        LookFile {
            name: self.name.clone(),
            extends,
            mood: self.mood.clone().or_else(|| base.mood.clone()),
            variant: self.variant.clone().or_else(|| base.variant.clone()),
            ui: self.ui.clone().or_else(|| base.ui.clone()),
            ui_seeds,
            antialias: self.antialias.clone().or_else(|| base.antialias.clone()),
            blocks,
        }
    }

    /// The resources the look sets, in [`LOOK_RESOURCES`] order.
    pub fn resources(&self) -> Vec<&'static str> {
        LOOK_RESOURCES
            .into_iter()
            .filter(|r| self.blocks.iter().any(|b| b.resource == *r))
            .collect()
    }
}

/// project.ogt's `look "looks/<id>.ogt"` (the project's `body`): the asset
/// id of the look whose art contract the cook reads (docs/art.md §2.1),
/// the look GameDesc.look applies when it names none; `None` without the
/// line. A value that is not one quoted `.ogt` path is refused as
/// `<line>: ...`.
pub fn project_look(body: &Block) -> Result<Option<String>, String> {
    let Some(e) = body.get("look") else {
        return Ok(None);
    };
    match e.values.as_slice() {
        [Value::String(s)] if s.ends_with(".ogt") && e.block.is_none() => {
            Ok(Some(crate::runtime::asset_id(s).to_string()))
        }
        _ => Err(format!(
            "{}: `look` names the game's look by its asset id, such as `look \"looks/main.ogt\"`",
            e.line
        )),
    }
}

/// The profile a project draws and simulates in: its creation form's
/// (`project_profile`). The cook records it in its manifest, which the
/// build, publish and the play page read; a new profile is a value here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Profile {
    /// Form `2.5d`, `3d` or none: rapier3d, models.
    #[default]
    ThreeD,
    /// Form `2d`: rapier2d, sprites.
    TwoD,
}

impl Profile {
    pub const ALL: [Profile; 2] = [Profile::ThreeD, Profile::TwoD];

    /// Its name in the cooked manifest and the build: `3d` or `2d`.
    pub fn name(self) -> &'static str {
        match self {
            Profile::ThreeD => "3d",
            Profile::TwoD => "2d",
        }
    }

    pub fn from_name(name: &str) -> Option<Profile> {
        Profile::ALL.into_iter().find(|p| p.name() == name)
    }
}

/// The project's profile (project.ogt's `body`): its creation form,
/// `create { form "..." }`, the one field the profile follows. A top-level
/// `profile` line is refused with its fix.
pub fn project_profile(body: &Block) -> Result<Profile, String> {
    if let Some(e) = body.get("profile") {
        return Err(format!(
            "{}: `profile` was removed; the creation form sets it: `create {{ form \"2d\" }}` for a 2D game, none (or form \"3d\") for a 3D one",
            e.line
        ));
    }
    let form = body
        .get("create")
        .and_then(|c| c.block.as_ref())
        .and_then(|b| b.get("form"));
    Ok(
        match form.and_then(|f| f.string(0).ok().map(|v| (f.line, v))) {
            Some((_, "2d")) => Profile::TwoD,
            Some((line, "film")) => {
                return Err(format!(
                    "{line}: form \"film\" was removed: an interactive film is `genre \"interactive-film\"` in the form its live scenes draw in (form \"3d\"), its flow started by project.ogt's `flow \"story/<name>.yarn\"`; og upgrade rewrites the form"
                ));
            }
            _ => Profile::ThreeD,
        },
    )
}

/// project.ogt's `flow "story/<name>.yarn"`: the dialogue the engine runs
/// as the game's flow (og.flow) from the first tick, and who draws it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectFlow {
    /// The dialogue's asset id (`story/game.yarn`).
    pub dialogue: String,
    /// `screen game`: the game draws the flow itself, so the engine mounts
    /// no FlowScreen; the default, `screen engine`, mounts it.
    pub game_screen: bool,
}

/// project.ogt's `flow` line (the project's `body`), `None` without one. A
/// value that is not one quoted `.yarn` path, or a block with anything but
/// `screen engine|game`, is refused as `<line>: ...`.
pub fn project_flow(body: &Block) -> Result<Option<ProjectFlow>, String> {
    let Some(e) = body.get("flow") else {
        return Ok(None);
    };
    let dialogue = match e.values.as_slice() {
        [Value::String(s)] if s.ends_with(".yarn") => crate::runtime::asset_id(s).to_string(),
        _ => {
            return Err(format!(
                "{}: `flow` names the game's flow by its Yarn dialogue, such as `flow \"story/game.yarn\"`",
                e.line
            ));
        }
    };
    let mut game_screen = false;
    for s in e.block.iter().flat_map(|b| &b.entries) {
        match (s.key.as_str(), s.word(0).ok()) {
            ("screen", Some("engine")) => game_screen = false,
            ("screen", Some("game")) => game_screen = true,
            _ => {
                return Err(format!(
                    "{}: a `flow` block takes `screen engine` (the built-in FlowScreen draws the flow, the default) or `screen game` (the game draws it)",
                    s.line
                ));
            }
        }
    }
    Ok(Some(ProjectFlow {
        dialogue,
        game_screen,
    }))
}

/// An sRGB `#rrggbb` (or `rrggbb`) colour as linear RGB, as three.js
/// `Color.setHex` gives under its colour management.
pub fn hex_to_linear(hex: &str) -> Option<[f64; 3]> {
    let h = hex.strip_prefix('#').unwrap_or(hex);
    if h.len() != 6 {
        return None;
    }
    let [r, g, b, _] = og_core::color::parse_hex(h)?;
    Some([r, g, b].map(|c| og_core::color::srgb_to_linear_f64(f64::from(c) / 255.0)))
}

/// The colours of a palette file: `.hex` (one `rrggbb` per line, `#`
/// optional, blank lines and `;` comments skipped) or GIMP `.gpl` (a
/// `GIMP Palette` header, `Name:`/`Columns:` lines and `#` comments, then
/// `r g b [name]` per colour). sRGB bytes in file order, at most 256.
pub fn parse_palette(name: &str, text: &str) -> Result<Vec<[u8; 3]>, String> {
    let gpl = name.to_ascii_lowercase().ends_with(".gpl");
    let mut out = Vec::new();
    let mut lines = text.lines().enumerate().peekable();
    if gpl {
        match lines.next() {
            Some((_, l)) if l.trim() == "GIMP Palette" => {}
            _ => return Err("a .gpl palette starts with the line `GIMP Palette`".into()),
        }
    }
    for (i, raw) in lines {
        let line = raw.trim();
        let n = i + 1;
        if line.is_empty() || line.starts_with(';') || (gpl && line.starts_with('#')) {
            continue;
        }
        if gpl {
            if line.starts_with("Name:") || line.starts_with("Columns:") {
                continue;
            }
            let rgb: Vec<u8> = line
                .split_whitespace()
                .take(3)
                .map(|t| t.parse::<u8>())
                .collect::<Result<_, _>>()
                .map_err(|_| format!("line {n}: a colour is `r g b [name]` with 0-255 channels"))?;
            if rgb.len() != 3 {
                return Err(format!(
                    "line {n}: a colour is `r g b [name]` with 0-255 channels"
                ));
            }
            out.push([rgb[0], rgb[1], rgb[2]]);
        } else {
            let h = line.strip_prefix('#').unwrap_or(line);
            let h = h.split_whitespace().next().unwrap_or("");
            match og_core::color::parse_hex(h) {
                Some([r, g, b, _]) if h.len() == 6 => out.push([r, g, b]),
                _ => return Err(format!("line {n}: '{line}' is not an rrggbb colour")),
            }
        }
        if out.len() > 256 {
            return Err("a palette holds at most 256 colours".into());
        }
    }
    if out.is_empty() {
        return Err("the palette holds no colours".into());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_what_is_not_a_look() {
        let bad = |body: &str| {
            LookFile::parse(&format!("type look\nversion 1\n---\n{body}")).unwrap_err()
        };
        assert!(bad("Camera {\n  fov 30\n}\n").contains("not a resource a look sets"));
        assert!(bad("PostEffects {\n  flash 1\n}\n").contains("animated by game code"));
        assert!(bad("antialias fxaa\n").contains("one of tier, off, msaa4, taa"));
        assert!(bad("palette \"p.hex\"\n").contains("not a look entry"));
        assert!(bad("Stylise {\n  posterize 4\n  posterize 5\n}\n").contains("set twice"));
        assert!(bad("variant \"8 Bit\"\n").contains("lowercase-hyphen"));
        assert!(bad("uiSeeds {\n  brand \"#ffffff\"\n}\n").contains("not a UI seed"));
        assert!(bad("uiSeeds {\n  primary \"red\"\n}\n").contains("sRGB"));
        assert!(
            bad("mood \"gloomy\"\n")
                .contains("one of bright, dark, warm, cold, soft, high-contrast")
        );
    }
}
