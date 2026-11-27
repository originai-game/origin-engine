//! Checked look values and native composition. Author inputs are typed TS;
//! the cook validates reflected fields before encoding the runtime payload.
use og_core::ogt::Value;

mod cooked;

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
/// reads from the project's look (og-project.json's `look`).
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

impl LookFile {
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
