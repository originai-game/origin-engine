//! Theme recipes (CONVENTIONS.md §Theme recipes): a `theme.json` picks one
//! value per dimension (`templates/themes/dimensions/<dimension>/<value>.json`)
//! and may extend another theme, mix in partial recipes and lay a pack over
//! the result. [`resolve`] reads a recipe and everything it inherits into
//! one recipe; [`compile`] turns that into a pack layer and the art spec the
//! cook draws (og-cook's `cook/ui_art.rs`).
//!
//! The taste is in the dimension files: this module only merges and maps.
//! The order is the recipe plan's (§3): the parent (`extends`, resolved
//! first), each `with` mix-in in order, then the recipe's own keys; objects
//! merge key by key, anything else (arrays included) is replaced. `pack`, a
//! pack.json written as a pack writes it, lies over the compiled layer and
//! the generated art, so a hand-drawn image replaces its generated one by
//! name and every other piece still comes from the generator.

use crate::pack::{Layer, Scheme};
use serde_json::{Map, Value, json};

/// The keys a recipe may set.
pub const RECIPE_KEYS: [&str; 14] = [
    "about", "extends", "with", "color", "material", "shape", "type", "motion", "ornament",
    "effects", "sound", "icons", "density", "pack",
];

/// The keys of a recipe's `color`.
pub const COLOR_KEYS: [&str; 6] = ["source", "accent", "neutral", "danger", "variant", "scheme"];

/// The pieces of art an ornament may be drawn on.
pub const ORNAMENT_SLOTS: [&str; 5] = ["button", "panel", "plate", "slot", "chip"];

/// Where a recipe, a mix-in or a dimension value is read from, relative to
/// `templates/` (the engine's themes) or a game's styles.
pub enum Source<'a> {
    /// A theme by name (`@themes/tactical` or `tactical`): its `theme.json`.
    Theme(&'a str),
    /// A mix-in by name (`templates/themes/mixins/<name>.json`).
    Mixin(&'a str),
    /// One dimension's value (`templates/themes/dimensions/<dim>/<value>.json`).
    Dimension(&'a str, &'a str),
}

impl Source<'_> {
    /// The file it names, relative to `templates/`.
    pub fn path(&self) -> String {
        match self {
            Source::Theme(n) => format!("themes/{}/theme.json", theme_name(n)),
            Source::Mixin(n) => format!("themes/mixins/{n}.json"),
            Source::Dimension(d, v) => format!("themes/dimensions/{d}/{v}.json"),
        }
    }
}

/// `@themes/tactical` and `tactical` both name `tactical`.
pub fn theme_name(name: &str) -> &str {
    name.strip_prefix("@themes/").unwrap_or(name)
}

/// Reads a source's JSON text, or says why it cannot.
pub type Read<'a> = dyn FnMut(&Source) -> Result<String, String> + 'a;

fn load(read: &mut Read, src: &Source) -> Result<Map<String, Value>, String> {
    let at = src.path();
    let text = read(src)?;
    match serde_json::from_str(&text).map_err(|e| format!("{at}: {e}"))? {
        Value::Object(m) => Ok(m),
        _ => Err(format!("{at}: not an object")),
    }
}

/// `over` merged into `have`: objects key by key, anything else replaced.
fn deep(have: &mut Value, over: &Value) {
    match (have, over) {
        (Value::Object(h), Value::Object(o)) => {
            for (k, v) in o {
                match h.get_mut(k) {
                    Some(slot) => deep(slot, v),
                    None => {
                        h.insert(k.clone(), v.clone());
                    }
                }
            }
        }
        (h, o) => *h = o.clone(),
    }
}

/// Checks a recipe's (or a mix-in's) keys and forms, and writes an
/// ornament named once (`"ornament": "studs"`) as one per piece, so a later
/// layer can change one piece's.
fn checked(mut r: Map<String, Value>, at: &str) -> Result<Map<String, Value>, String> {
    for (k, v) in &r {
        if !RECIPE_KEYS.contains(&k.as_str()) {
            return Err(format!(
                "{at}: `{k}` is not a recipe key; one of {}",
                RECIPE_KEYS.join(", ")
            ));
        }
        let ok = match k.as_str() {
            "about" | "extends" | "material" | "shape" | "type" | "motion" | "effects"
            | "sound" | "icons" | "density" => v.is_string(),
            "with" => v.as_array().is_some_and(|a| a.iter().all(Value::is_string)),
            "color" | "pack" => v.is_object(),
            "ornament" => v.is_string() || v.is_object(),
            _ => true,
        };
        if !ok {
            let want = match k.as_str() {
                "with" => "a list of mix-in names",
                "color" | "pack" => "an object",
                "ornament" => "an ornament's name, or one per piece",
                _ => "a name",
            };
            return Err(format!("{at}: `{k}` is {want}"));
        }
    }
    if let Some(Value::Object(c)) = r.get("color") {
        for (k, v) in c {
            if !COLOR_KEYS.contains(&k.as_str()) {
                return Err(format!(
                    "{at}: `color.{k}` is not a colour key; one of {}",
                    COLOR_KEYS.join(", ")
                ));
            }
            if !v.is_string() {
                return Err(format!("{at}: `color.{k}` is a string"));
            }
        }
        if let Some(s) = c.get("scheme").and_then(Value::as_str)
            && Scheme::ALL.iter().all(|x| x.name() != s)
        {
            return Err(format!("{at}: `color.scheme` is \"light\" or \"dark\""));
        }
    }
    match r.get("ornament") {
        Some(Value::String(o)) => {
            let all: Map<String, Value> = ORNAMENT_SLOTS
                .iter()
                .map(|s| (s.to_string(), json!(o)))
                .collect();
            r.insert("ornament".into(), Value::Object(all));
        }
        Some(Value::Object(o)) => {
            for (k, v) in o {
                if !ORNAMENT_SLOTS.contains(&k.as_str()) || !v.is_string() {
                    return Err(format!(
                        "{at}: `ornament.{k}`: an ornament's name for one of {}",
                        ORNAMENT_SLOTS.join(", ")
                    ));
                }
            }
        }
        _ => {}
    }
    Ok(r)
}

/// The recipe `top` (the text of a `theme.json`, read from `at`) with
/// everything it inherits merged in: its parent's resolved recipe, then
/// each mix-in in order, then its own keys. `extends` and `with` are gone
/// from the result. A theme that extends itself through any chain is an
/// error naming the chain.
pub fn resolve(top: &str, at: &str, read: &mut Read) -> Result<Map<String, Value>, String> {
    let own = match serde_json::from_str(top).map_err(|e| format!("{at}: {e}"))? {
        Value::Object(m) => m,
        _ => return Err(format!("{at}: not an object")),
    };
    resolve_with(own, at, read, &mut Vec::new())
}

fn resolve_with(
    own: Map<String, Value>,
    at: &str,
    read: &mut Read,
    chain: &mut Vec<String>,
) -> Result<Map<String, Value>, String> {
    let mut own = checked(own, at)?;
    let mut out = Value::Object(Map::new());
    if let Some(Value::String(parent)) = own.remove("extends") {
        let name = theme_name(&parent).to_owned();
        if chain.contains(&name) {
            chain.push(name);
            return Err(format!("{at}: `extends` loops: {}", chain.join(" → ")));
        }
        chain.push(name.clone());
        let src = Source::Theme(&name);
        let path = src.path();
        let p = load(read, &src)?;
        out = Value::Object(resolve_with(p, &path, read, chain)?);
        chain.pop();
    }
    if let Some(Value::Array(with)) = own.remove("with") {
        for m in with {
            let src = Source::Mixin(m.as_str().unwrap_or_default());
            let path = src.path();
            let mix = checked(load(read, &src)?, &path)?;
            if mix.contains_key("extends") || mix.contains_key("with") {
                return Err(format!("{path}: a mix-in neither extends nor mixes in"));
            }
            deep(&mut out, &Value::Object(mix));
        }
    }
    deep(&mut out, &Value::Object(own));
    let Value::Object(out) = out else {
        unreachable!()
    };
    Ok(out)
}

/// A compiled theme: the pack layer its dimensions make, the art spec the
/// cook draws, and the recipe's `pack`, which lies over both.
#[derive(Clone, Debug)]
pub struct Theme {
    /// The pack.json the dimensions compile to, without art.
    pub pack: Map<String, Value>,
    /// The art spec (`templates/tools/theme-art.py`'s input): the palette,
    /// material, shape and ornaments.
    pub art: Value,
    /// The recipe's own pack, laid last.
    pub over: Map<String, Value>,
    /// The `effects` value (its textures, text glow and shadow), for the
    /// cook to carry.
    pub effects: Option<Map<String, Value>>,
    /// The `density` value (`compact`, `comfortable` or `touch`).
    pub density: Option<String>,
}

impl Theme {
    /// The layer, with `images` (the drawn art's pack entries, by name)
    /// under the recipe's own pack: what the cook lays over the default.
    pub fn layer(&self, images: Map<String, Value>) -> Result<Layer, String> {
        let mut pack = Value::Object(self.pack.clone());
        if !images.is_empty() {
            deep(&mut pack, &json!({ "images": images }));
        }
        deep(&mut pack, &Value::Object(self.over.clone()));
        Layer::parse(&pack.to_string()).map_err(|e| format!("the compiled theme: {e}"))
    }
}

/// A colour in OKLCH: lightness, chroma, hue in degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lch {
    pub l: f32,
    pub c: f32,
    pub h: f32,
}

/// `text` (any CSS colour that depends on nothing) in OKLCH.
pub fn lch(text: &str) -> Result<Lch, String> {
    let c = og_css::color::parse_str(text)
        .ok()
        .and_then(|c| c.literal())
        .ok_or_else(|| format!("{text}: not a colour"))?;
    let lin = c.rgb.map(|v| {
        let v = v as f32 / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            og_core::math::pow((v + 0.055) / 1.055, 2.4)
        }
    });
    let [l, a, b] = rune_motion::linear_to_oklab(lin[0], lin[1], lin[2]);
    let h = og_core::math::atan2(b, a).to_degrees();
    Ok(Lch {
        l,
        c: (a * a + b * b).sqrt(),
        h: if h < 0.0 { h + 360.0 } else { h },
    })
}

/// `c` as `#rrggbb`, mapped into sRGB as CSS maps it.
pub fn hex(c: Lch) -> String {
    let text = format!("oklch({} {} {})", c.l, c.c, c.h);
    let rgb = og_css::color::parse_str(&text)
        .ok()
        .and_then(|c| c.literal())
        .map_or([0; 3], |s| s.rgb);
    format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2])
}

fn at(c: Lch, l: f64, chroma: f64) -> String {
    hex(Lch {
        l: l as f32,
        c: chroma as f32,
        h: c.h,
    })
}

fn num(v: &Value, at: &str) -> Result<f64, String> {
    v.as_f64().ok_or_else(|| format!("{at}: a number"))
}

fn pair(v: &Value, at: &str) -> Result<[f64; 2], String> {
    match v.as_array().map(Vec::as_slice) {
        Some([a, b]) => Ok([num(a, at)?, num(b, at)?]),
        _ => Err(format!("{at}: [top, bottom]")),
    }
}

/// The WCAG relative luminance of `#rrggbb`.
fn luminance(hex: &str) -> f32 {
    let h = hex.trim_start_matches('#');
    let c = |i: usize| {
        let v = u8::from_str_radix(h.get(i..i + 2).unwrap_or("00"), 16).unwrap_or(0) as f32 / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            og_core::math::pow((v + 0.055) / 1.055, 2.4)
        }
    };
    0.2126 * c(0) + 0.7152 * c(2) + 0.0722 * c(4)
}

/// The WCAG contrast ratio of two `#rrggbb` colours.
pub fn contrast(a: &str, b: &str) -> f32 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// An ink in `hue`'s hue that holds `ratio` on every face, and the least
/// ratio it reaches: a near-white moved lighter or a near-black moved
/// darker, in OKLCH lightness only (hue and chroma kept); the better
/// extreme when none holds.
pub fn solve(hue: Lch, faces: &[String], ratio: f32) -> (String, f32) {
    let c = hue.c.min(0.02);
    let worst = |ink: &str| {
        faces
            .iter()
            .map(|f| contrast(ink, f))
            .fold(f32::MAX, f32::min)
    };
    // From near-white up and from near-black down: the first that holds,
    // else the better of the two extremes.
    let mut best = (String::new(), 0.0);
    for (from, step) in [(0.9f32, 0.005f32), (0.3, -0.005)] {
        let mut l = from;
        loop {
            let ink = hex(Lch { l, c, h: hue.h });
            let w = worst(&ink);
            if w >= ratio {
                return (ink, w);
            }
            if !(0.0..=1.0).contains(&(l + step)) {
                if w > best.1 {
                    best = (ink, w);
                }
                break;
            }
            l += step;
        }
    }
    best
}

/// The ink roles the solver holds on every ground, with their ratio.
pub const INKS: [(&str, f32); 26] = [
    ("fg", 4.5),
    ("fg-muted", 4.5),
    ("fg-subtle", 3.0),
    ("accent", 4.5),
    ("accent-ink", 4.5),
    ("danger", 4.5),
    ("warn", 4.5),
    ("success", 4.5),
    ("info", 4.5),
    ("gold", 4.5),
    ("gold-hi", 3.0),
    ("xp", 4.5),
    ("common", 4.5),
    ("uncommon", 4.5),
    ("rare", 4.5),
    ("epic", 4.5),
    ("legendary", 4.5),
    ("mythic", 4.5),
    ("hud-accent-ink", 4.5),
    ("foe-ink", 4.5),
    ("ally-ink", 4.5),
    ("boon-ink", 4.5),
    ("fever-ink", 4.5),
    ("late-ink", 4.5),
    ("team1", 4.5),
    ("epic-hi", 3.0),
];

/// The `--seed-*` CSS a compiled pack's seeds give.
fn seed_css(pack: &Map<String, Value>) -> String {
    let seeds: String = pack
        .get("seeds")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .map(|(k, v)| format!("--seed-{k}: {};", v.as_str().unwrap_or_default()))
        .collect();
    format!(":root {{ {seeds} }}")
}

/// Each ink role ([`INKS`]) as a light/dark pair that holds its ratio on
/// every ground of its scheme: `art` (the art faces registered per scheme,
/// `Scheme::ALL` order) and the design's own surfaces. Hue and chroma are
/// the design's; only OKLCH lightness moves, away from the grounds. A role
/// the solve leaves where the design put it in both schemes is not written.
fn solve_inks(seeds: &str, art: &[Vec<String>; 2], colors: &mut Map<String, Value>) {
    let design = crate::pack::design::colors(seeds);
    const SURFACES: [&str; 8] = [
        "bg",
        "bg-subtle",
        "surface",
        "surface-raised",
        "layer-1",
        "layer-2",
        "layer-3",
        "layer-4",
    ];
    for (role, ratio) in INKS {
        let mut pair = Map::new();
        let mut moved = false;
        for (i, scheme) in ["light", "dark"].into_iter().enumerate() {
            let Some(have) = design[i].get(role).and_then(Value::as_str) else {
                continue;
            };
            let mut grounds = art[i].clone();
            grounds.extend(
                SURFACES
                    .iter()
                    .filter_map(|s| design[i].get(*s)?.as_str().map(str::to_owned)),
            );
            let worst = |ink: &str| {
                grounds
                    .iter()
                    .map(|g| contrast(ink, g))
                    .fold(f32::MAX, f32::min)
            };
            let mut ink = have[..7.min(have.len())].to_owned();
            let Ok(mut c) = lch(&ink) else { continue };
            let up = i == 1;
            while worst(&ink) < ratio && (if up { c.l < 1.0 } else { c.l > 0.0 }) {
                c.l = (c.l + if up { 0.005 } else { -0.005 }).clamp(0.0, 1.0);
                ink = hex(c);
                moved = true;
            }
            pair.insert(scheme.into(), json!(ink));
        }
        if moved && pair.len() == 2 {
            colors.entry(role.to_owned()).or_insert(Value::Object(pair));
        }
    }
}

/// The art's nine-slice pieces and the slots that draw them: each slot's
/// art is its edge (CONVENTIONS.md §Grounds and inks), so its fill is
/// cleared and the art's face is the ground.
const ART_SLOTS: [(&str, &str); 7] = [
    ("Button.plate", "button"),
    ("Button.plate.primary", "button-primary"),
    ("Panel.frame", "panel"),
    ("Plate.frame", "plate"),
    ("Chip.frame", "chip"),
    ("Slot.frame", "slot"),
    ("Field.frame", "field"),
];

/// Compiles a resolved recipe ([`resolve`]): each dimension value read
/// through `read`, mapped into the pack (seeds, radius, fonts, tracking,
/// the motion and art slots, the icon set and sounds) and the art spec.
/// A dimension left unset adds nothing; a value with no file is an error.
pub fn compile(recipe: &Map<String, Value>, read: &mut Read) -> Result<Theme, String> {
    let mut dim = |name: &str| -> Result<Option<Map<String, Value>>, String> {
        let Some(v) = recipe.get(name).and_then(Value::as_str) else {
            return Ok(None);
        };
        let src = Source::Dimension(name, v);
        load(read, &src)
            .map(Some)
            .map_err(|e| format!("{name} \"{v}\": no such value ({e})"))
    };
    let material = dim("material")?;
    let shape = dim("shape")?;
    let ty = dim("type")?;
    let motion = dim("motion")?;
    let effects = dim("effects")?;
    let sound = dim("sound")?;
    let icons = dim("icons")?;
    let density = dim("density")?;
    let mut ornaments = Map::new();
    if let Some(Value::Object(o)) = recipe.get("ornament") {
        for (slot, v) in o {
            let v = v.as_str().unwrap_or_default();
            let src = Source::Dimension("ornament", v);
            let d =
                load(read, &src).map_err(|e| format!("ornament \"{v}\": no such value ({e})"))?;
            ornaments.insert(slot.clone(), d.get("art").cloned().unwrap_or(json!({})));
        }
    }

    let empty = Map::new();
    let color = recipe
        .get("color")
        .and_then(Value::as_object)
        .unwrap_or(&empty);
    let col = |k: &str| color.get(k).and_then(Value::as_str);
    let scheme = col("scheme")
        .or_else(|| material.as_ref()?.get("scheme")?.as_str())
        .map(str::to_owned);

    let mut pack = Map::new();
    pack.insert(
        "about".into(),
        json!(
            recipe
                .get("about")
                .cloned()
                .unwrap_or(json!("A compiled theme recipe."))
        ),
    );
    if let Some(s) = &scheme {
        pack.insert("scheme".into(), json!(s));
    }
    let mut seeds = Map::new();
    for (key, seed) in [
        ("source", "primary"),
        ("accent", "accent"),
        ("neutral", "neutral"),
        ("danger", "danger"),
    ] {
        if let Some(c) = col(key) {
            lch(c).map_err(|e| format!("color.{key}: {e}"))?;
            seeds.insert(seed.into(), json!(c));
        }
    }
    if !seeds.is_empty() {
        pack.insert("seeds".into(), Value::Object(seeds));
    }
    let mut theme = Map::new();
    let mut variants = Map::new();
    if let Some(r) = shape.as_ref().and_then(|s| s.get("radius")) {
        theme.insert("radius".into(), r.clone());
    }
    if let Some(t) = &ty {
        for k in ["fontFamily", "tracking"] {
            if let Some(Value::Object(m)) = t.get(k)
                && !m.is_empty()
            {
                theme.insert(k.into(), Value::Object(m.clone()));
            }
        }
        if let Some(f) = t.get("fonts") {
            pack.insert("fonts".into(), f.clone());
        }
    }
    if let Some(Value::Object(v)) = motion.as_ref().and_then(|m| m.get("variants")) {
        variants.extend(v.clone());
    }
    if let Some(s) = sound.as_ref().and_then(|s| s.get("sounds")) {
        pack.insert("sounds".into(), s.clone());
    }

    // The art: faces from the source, accent and neutral hues at the
    // material's lightness, the slots that draw it, and its spec.
    let mut art = Value::Null;
    let mut palette = Map::new();
    let mut colors = Map::new();
    if let Some(m) = &material {
        let source = lch(col("source").unwrap_or("#808080"))?;
        let accent = col("accent").map(lch).transpose()?.unwrap_or(source);
        let neutral = col("neutral").map(lch).transpose()?.unwrap_or(source);
        let dark = scheme.as_deref() != Some("light");
        let danger = col("danger").map(lch).transpose()?.unwrap_or(lch(if dark {
            "#ff7a80"
        } else {
            "#b20011"
        })?);
        let faces = m.get("faces").and_then(Value::as_object).unwrap_or(&empty);
        for (role, f) in faces {
            let fa = format!("material.faces.{role}");
            let hue = match f.get("hue").and_then(Value::as_str) {
                Some("a") => accent,
                _ => neutral,
            };
            let [t, b] = pair(&f["l"], &format!("{fa}.l"))?;
            let c = num(&f["c"], &format!("{fa}.c"))?;
            palette.insert(role.clone(), json!([at(hue, t, c), at(hue, b, c)]));
        }
        let band = m.get("primary").cloned().unwrap_or(json!({}));
        let [t, b] = pair(&band["l"], "material.primary.l")?;
        let floor = band.get("c").and_then(Value::as_f64).unwrap_or(0.0);
        for (role, h) in [("primary", source), ("danger", danger)] {
            let c = (h.c as f64).max(floor);
            palette.insert(role.into(), json!([at(h, t, c), at(h, b, c)]));
        }
        palette.insert(
            "disabled".into(),
            json!([at(neutral, 0.5, 0.01), at(neutral, 0.42, 0.01)]),
        );
        let outline = at(neutral, if dark { 0.1 } else { 0.3 }, 0.03);
        palette.insert(
            "shadow".into(),
            json!(if dark {
                "#000000".to_owned()
            } else {
                outline.clone()
            }),
        );
        palette.insert("outline".into(), json!(outline));
        palette.insert(
            "orn".into(),
            json!(at(accent, if dark { 0.78 } else { 0.45 }, accent.c as f64)),
        );
        palette.insert("accent".into(), json!(hex(accent)));
        palette.insert(
            "header".into(),
            json!(at(source, 0.45, (source.c as f64).max(0.1))),
        );
        // Each art slot's ink, solved on the faces it draws (4.5:1 for
        // text, 3:1 on the disabled button): the component's own ink was
        // chosen for its own ground, so an art slot names its own. When no
        // ink holds on a face pair, the faces move away from the ink in
        // lightness (the material's band narrows) until one does.
        // A slot's scheme is its own faces' (a material mixes light and dark
        // pieces), so the inks of the text inside it are that scheme's.
        let mut ink = |name: String, role: &str, ratio: f32| -> (String, &'static str) {
            let mut faces: Vec<String> = palette
                .get(role)
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|f| f.as_str().map(str::to_owned))
                .collect();
            let (mut fg, mut worst) = solve(neutral, &faces, ratio);
            let away = if luminance(&fg) > 0.5 { -0.01 } else { 0.01 };
            for _ in 0..60 {
                if worst >= ratio {
                    break;
                }
                faces = faces
                    .iter()
                    .map(|f| {
                        let c = lch(f).unwrap_or(neutral);
                        hex(Lch {
                            l: (c.l + away).clamp(0.0, 1.0),
                            ..c
                        })
                    })
                    .collect();
                (fg, worst) = solve(neutral, &faces, ratio);
            }
            let light = faces.iter().map(|f| luminance(f)).sum::<f32>()
                / (faces.len().max(1) as f32)
                >= 0.18;
            palette.insert(role.into(), json!(faces));
            colors.insert(name.clone(), json!(fg));
            (name, if light { "light" } else { "dark" })
        };
        let (disabled, _) = ink("art-button-disabled-fg".into(), "disabled", 3.0);
        for (slot, image) in ART_SLOTS {
            let role = match image {
                "button-primary" => "primary",
                other => other,
            };
            let (fg, s) = ink(format!("art-{image}-fg"), role, 4.5);
            let states = if slot.starts_with("Button") {
                format!(
                    " hover:nine-{image}-hover active:nine-{image}-pressed disabled:nine-button-disabled data-pressed:text-{fg} disabled:text-{disabled}"
                )
            } else {
                String::new()
            };
            variants.insert(
                slot.into(),
                json!(format!("scheme-{s} nine-{image}{states} bg-transparent hover:bg-transparent active:bg-transparent text-{fg}")),
            );
        }
        // A button variant without art of its own keeps its own plate over
        // the art the theme gives `Button.plate`: no art (a second edge on
        // it), the page's scheme and the variant's own fill and ink.
        for (slot, _, dress) in crate::pack::design::SLOTS {
            let Some(_) = slot.strip_prefix("Button.plate.") else {
                continue;
            };
            if ART_SLOTS.iter().any(|(s, _)| s == slot) {
                continue;
            }
            variants.insert(
                (*slot).into(),
                json!(format!(
                    "scheme-normal nine-none hover:nine-none active:nine-none disabled:nine-none {dress}"
                )),
            );
        }
        // Ground registration (plan §4): each art slot's faces are grounds
        // of the scheme its text reads in, with the design's own surfaces;
        // every ink role is moved in OKLCH lightness until it holds its
        // ratio on all of that scheme's grounds.
        let mut grounds: [Vec<String>; 2] = [Vec::new(), Vec::new()];
        for (_, image) in ART_SLOTS {
            let role = if image == "button-primary" {
                "primary"
            } else {
                image
            };
            let faces: Vec<String> = palette
                .get(role)
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|f| f.as_str().map(str::to_owned))
                .collect();
            let light = faces.iter().map(|f| luminance(f)).sum::<f32>()
                / (faces.len().max(1) as f32)
                >= 0.18;
            grounds[usize::from(!light)].extend(faces);
        }
        solve_inks(&seed_css(&pack), &grounds, &mut colors);
        art = json!({
            "scale": 4,
            "palette": palette,
            "material": m.get("art").cloned().unwrap_or(json!({})),
            "shape": shape.as_ref().and_then(|s| s.get("art")).cloned().unwrap_or(json!({})),
            "ornament": ornaments,
        });
    }
    if let Some(i) = &icons {
        if let Some(set) = i.get("set") {
            pack.insert("iconSet".into(), set.clone());
        }
        if let Some(Value::Object(cs)) = i.get("colors") {
            let seeds: String = pack
                .get("seeds")
                .and_then(Value::as_object)
                .into_iter()
                .flatten()
                .map(|(k, v)| format!("--seed-{k}: {};", v.as_str().unwrap_or_default()))
                .collect();
            let design = crate::pack::design::colors(&format!(":root {{ {seeds} }}"));
            for (k, v) in cs {
                let v = v.as_str().unwrap_or_default();
                // A bare name is an art palette role (`outline`), else a
                // design colour role (`fg`) as the recipe's seeds derive it.
                let v = match palette.get(v) {
                    Some(c @ Value::String(_)) => c.clone(),
                    _ if v.starts_with('#') => json!(v),
                    _ => {
                        let [l, d] = &design;
                        match (l.get(v), d.get(v)) {
                            (Some(l), Some(d)) => json!({ "light": l, "dark": d }),
                            _ => {
                                return Err(format!(
                                    "icons.colors.{k}: \"{v}\" is no colour, palette role or colour role"
                                ));
                            }
                        }
                    }
                };
                colors.insert(k.clone(), v);
            }
        }
    }
    if !colors.is_empty() {
        theme.insert("colors".into(), Value::Object(colors));
    }
    if !theme.is_empty() {
        pack.insert("theme".into(), Value::Object(theme));
    }
    if !variants.is_empty() {
        pack.insert("variants".into(), Value::Object(variants));
    }
    let over = recipe
        .get("pack")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    Ok(Theme {
        pack,
        art,
        over,
        effects: effects.map(|mut e| {
            e.remove("about");
            e
        }),
        density: density.and_then(|d| d.get("density")?.as_str().map(str::to_owned)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn files(list: &[(&str, &str)]) -> BTreeMap<String, String> {
        list.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn reader(f: &BTreeMap<String, String>) -> impl FnMut(&Source) -> Result<String, String> + '_ {
        move |s: &Source| {
            let p = s.path();
            f.get(&p).cloned().ok_or_else(|| format!("{p}: missing"))
        }
    }

    #[test]
    fn a_chain_resolves_parent_then_mixins_in_order_then_its_own_keys() {
        let f = files(&[
            (
                "themes/base/theme.json",
                r##"{ "color": { "source": "#111111", "neutral": "#222222" }, "material": "metal", "motion": "calm", "ornament": "studs" }"##,
            ),
            (
                "themes/mid/theme.json",
                r##"{ "extends": "@themes/base", "color": { "source": "#333333" }, "motion": "snappy" }"##,
            ),
            (
                "themes/mixins/cold.json",
                r##"{ "color": { "source": "#5cc8ff" }, "shape": "cut" }"##,
            ),
            (
                "themes/mixins/night.json",
                r##"{ "color": { "source": "#000044" } }"##,
            ),
        ]);
        let top = r##"{ "extends": "mid", "with": ["cold", "night"], "ornament": { "panel": "hairline" } }"##;
        let r = resolve(top, "game/theme.json", &mut reader(&f)).unwrap();
        // the parent's own keys beat its parent's; the later mix-in beats
        // the earlier; objects merge, so `neutral` survives from the root
        assert_eq!(
            r["color"],
            json!({ "source": "#000044", "neutral": "#222222" })
        );
        assert_eq!(r["motion"], "snappy");
        assert_eq!(r["shape"], "cut");
        assert_eq!(r["material"], "metal");
        // a named ornament spreads to every piece, and the recipe's own
        // replaces one piece's
        assert_eq!(r["ornament"]["panel"], "hairline");
        assert_eq!(r["ornament"]["plate"], "studs");
        assert!(r.get("extends").is_none() && r.get("with").is_none());

        // the recipe's own keys win over its mix-ins
        let top = r##"{ "extends": "mid", "with": ["cold"], "color": { "source": "#abcdef" } }"##;
        let r = resolve(top, "t", &mut reader(&f)).unwrap();
        assert_eq!(r["color"]["source"], "#abcdef");
    }

    #[test]
    fn arrays_are_replaced_not_joined() {
        let f = files(&[(
            "themes/a/theme.json",
            r#"{ "pack": { "fonts": ["a.ttf", "b.ttf"] } }"#,
        )]);
        let r = resolve(
            r#"{ "extends": "a", "pack": { "fonts": ["c.ttf"] } }"#,
            "t",
            &mut reader(&f),
        )
        .unwrap();
        assert_eq!(r["pack"]["fonts"], json!(["c.ttf"]));
    }

    #[test]
    fn a_loop_names_its_whole_chain() {
        let f = files(&[
            ("themes/a/theme.json", r#"{ "extends": "b" }"#),
            ("themes/b/theme.json", r#"{ "extends": "@themes/c" }"#),
            ("themes/c/theme.json", r#"{ "extends": "a" }"#),
        ]);
        let e = resolve(r#"{ "extends": "a" }"#, "t", &mut reader(&f)).unwrap_err();
        assert!(e.contains("a → b → c → a"), "{e}");
    }

    #[test]
    fn unknown_keys_and_forms_are_errors_naming_where() {
        let f = files(&[("themes/mixins/m.json", r#"{ "colour": {} }"#)]);
        for (top, want) in [
            (r#"{ "materal": "metal" }"#, "`materal` is not a recipe key"),
            (
                r##"{ "color": { "sauce": "#fff" } }"##,
                "`color.sauce` is not a colour key",
            ),
            (
                r#"{ "color": { "scheme": "dusk" } }"#,
                "\"light\" or \"dark\"",
            ),
            (r#"{ "with": "m" }"#, "a list of mix-in names"),
            (r#"{ "ornament": { "wall": "studs" } }"#, "`ornament.wall`"),
            (r#"{ "with": ["m"] }"#, "themes/mixins/m.json: `colour`"),
            (
                r#"{ "extends": "gone" }"#,
                "themes/gone/theme.json: missing",
            ),
        ] {
            let e = resolve(top, "game/theme.json", &mut reader(&f)).unwrap_err();
            assert!(e.contains(want), "{top}: {e}");
        }
    }

    fn templates() -> impl FnMut(&Source) -> Result<String, String> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates");
        move |s: &Source| {
            let p = root.join(s.path());
            std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", s.path()))
        }
    }

    /// Every dimension file compiles: a recipe of each value parses as a
    /// pack layer, so no dimension names a key, slot or form the pack
    /// refuses.
    #[test]
    fn every_dimension_value_compiles_to_a_layer() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../templates/themes/dimensions");
        let mut n = 0;
        for d in std::fs::read_dir(&root).unwrap() {
            let d = d.unwrap().path();
            if !d.is_dir() {
                continue;
            }
            let dim = d.file_name().unwrap().to_string_lossy().into_owned();
            for v in std::fs::read_dir(&d).unwrap() {
                let v = v.unwrap().path();
                let value = v.file_stem().unwrap().to_string_lossy().into_owned();
                let mut r = Map::new();
                r.insert(dim.clone(), json!(value));
                r.insert(
                    "color".into(),
                    json!({ "source": "#d9a531", "neutral": "#4c5643" }),
                );
                if dim != "material" {
                    r.insert("material".into(), json!("metal"));
                }
                let t =
                    compile(&r, &mut templates()).unwrap_or_else(|e| panic!("{dim}/{value}: {e}"));
                t.layer(Map::new())
                    .unwrap_or_else(|e| panic!("{dim}/{value}: {e}"));
                n += 1;
            }
        }
        assert!(n > 50, "{n}");
    }

    /// A tactical recipe as the plan writes it: the seeds are the recipe's,
    /// the faces take the neutral's hue at the material's lightness, the
    /// art slots draw the art, the motion and type come through, and the
    /// recipe's pack lies over all of it.
    #[test]
    fn a_recipe_compiles_into_seeds_art_slots_and_its_pack_over_them() {
        let top = r##"{
          "color": { "source": "#d9a531", "accent": "#e8c25a", "neutral": "#4c5643", "scheme": "dark" },
          "material": "metal", "shape": "cut", "type": "condensed-stencil", "motion": "snappy",
          "ornament": { "panel": "brackets", "plate": "studs" }, "effects": "spark",
          "sound": "mechanical", "icons": "sticker", "density": "touch",
          "pack": { "variants": { "Panel.frame": "nine-panel-hand" }, "images": { "panel": { "src": "art/panel-hand.png", "slice": 24 } } }
        }"##;
        let r = resolve(top, "t", &mut templates()).unwrap();
        let t = compile(&r, &mut templates()).unwrap();
        assert_eq!(t.pack["seeds"]["primary"], "#d9a531");
        assert_eq!(t.pack["scheme"], "dark");
        let n = lch("#4c5643").unwrap();
        let panel = lch(t.art["palette"]["panel"][0].as_str().unwrap()).unwrap();
        assert!((panel.l - 0.30).abs() < 0.01, "{panel:?}");
        assert!((panel.h - n.h).abs() < 3.0, "{panel:?} {n:?}");
        assert_eq!(t.art["ornament"]["panel"]["kind"], "brackets");
        assert_eq!(t.art["ornament"]["plate"]["kind"], "studs");
        assert_eq!(t.art["shape"]["family"], "cut");
        assert!(
            t.pack["variants"]["Button.plate.primary"]
                .as_str()
                .unwrap()
                .starts_with("scheme-light nine-button-primary hover:nine-button-primary-hover")
        );
        // A button variant without art of its own draws its own plate in
        // the page's scheme, not a second edge over the base's art.
        for v in ["danger", "outline", "ghost-muted"] {
            let class = t.pack["variants"][format!("Button.plate.{v}")]
                .as_str()
                .unwrap();
            assert!(
                class.starts_with("scheme-normal nine-none "),
                "{v}: {class}"
            );
        }
        assert!(
            t.pack["variants"]["Press.feedback"]
                .as_str()
                .unwrap()
                .contains("active:scale-96")
        );
        assert!(
            t.pack["theme"]["fontFamily"]["display"]
                .as_str()
                .unwrap()
                .contains("Big Shoulders Stencil")
        );
        assert_eq!(
            t.pack["theme"]["colors"]["icon-outline"],
            t.art["palette"]["outline"]
        );
        assert_eq!(t.density.as_deref(), Some("touch"));
        assert!(t.effects.as_ref().unwrap().contains_key("textures"));
        let generated = json!({ "panel": { "src": "art/panel.png", "slice": 24 }, "plate": { "src": "art/plate.png", "slice": 12 } });
        let layer = t.layer(generated.as_object().unwrap().clone()).unwrap();
        assert_eq!(layer.variants["Panel.frame"], "nine-panel-hand");
        assert_eq!(
            layer.images[Scheme::Dark as usize]["panel"].src,
            "art/panel-hand.png"
        );
        assert_eq!(
            layer.images[Scheme::Dark as usize]["plate"].src,
            "art/plate.png"
        );
    }

    /// Every material, light or dark: each art slot names its own ink, and
    /// that ink holds 4.5:1 on the faces the slot draws (a component's ink
    /// was chosen for its own ground, which the art replaces).
    #[test]
    fn each_art_slot_names_an_ink_that_reads_on_its_faces() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../templates/themes/dimensions/material");
        for m in std::fs::read_dir(&root).unwrap() {
            let m = m
                .unwrap()
                .path()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            for source in ["#d9a531", "#5cc8ff", "#ff5c9a"] {
                let mut r = Map::new();
                r.insert("material".into(), json!(m));
                r.insert(
                    "color".into(),
                    json!({ "source": source, "neutral": "#4c5643" }),
                );
                let t = compile(&r, &mut templates()).unwrap();
                for (slot, image) in ART_SLOTS {
                    let role = if image == "button-primary" {
                        "primary"
                    } else {
                        image
                    };
                    let ink = format!("art-{image}-fg");
                    assert!(
                        t.pack["variants"][slot]
                            .as_str()
                            .unwrap()
                            .contains(&format!("text-{ink}")),
                        "{m} {slot}"
                    );
                    let ink = t.pack["theme"]["colors"][&ink].as_str().unwrap();
                    for face in t.art["palette"][role].as_array().unwrap() {
                        let r = contrast(ink, face.as_str().unwrap());
                        assert!(r >= 4.5, "{m} {source} {slot}: {ink} on {face} {r:.2}");
                    }
                }
            }
        }
    }

    #[test]
    fn a_dimension_value_with_no_file_is_an_error() {
        let mut r = Map::new();
        r.insert("material".into(), json!("velvet"));
        let e = compile(&r, &mut templates()).unwrap_err();
        assert!(e.contains("material \"velvet\": no such value"), "{e}");
    }
}
