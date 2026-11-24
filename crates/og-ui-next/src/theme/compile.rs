//! Compiling a resolved theme recipe into its layered pack, seed CSS,
//! procedural art, and typography.

use super::color::*;
use super::*;

/// The art's nine-slice pieces and the slots that draw them: each slot's
/// art is its edge (CONVENTIONS.md §Grounds and inks), so its fill is
/// cleared and the art's face is the ground.
pub(super) const ART_SLOTS: [(&str, &str); 9] = [
    ("Button.plate", "button"),
    ("Button.plate.primary", "button-primary"),
    ("Button.plate.danger", "button-danger"),
    ("Panel.frame", "panel"),
    ("Plate.frame", "plate"),
    ("Chip.frame", "chip"),
    ("Slot.frame", "slot"),
    ("Field.frame", "field"),
    ("Field.area", "field"),
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
    // A motion's own steps (how far a press, a hover or an arrival
    // scales), under what the recipe's pack sets.
    if let Some(Value::Object(t)) = motion.as_ref().and_then(|m| m.get("theme")) {
        for (k, v) in t {
            deep(theme.entry(k.clone()).or_insert(json!({})), v);
        }
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
                "button-danger" => "danger",
                other => other,
            };
            let (fg, s) = ink(format!("art-{image}-fg"), role, 4.5);
            let states = if slot.starts_with("Button") {
                format!(
                    " hover:nine-{image}-hover active:nine-{image}-pressed disabled:nine-button-disabled data-pressed:text-{fg} disabled:text-{disabled}"
                )
            } else if image == "field" {
                // A text field's states, each its own art: read-only lies
                // flat, an invalid value rings in danger, the focus in the
                // accent (over invalid: the field being typed in), disabled
                // over all, its ink the disabled faces'.
                // Its box keeps the design's size and padding, which the
                // art's edge sits in.
                format!(
                    " min-h-11 px-3 py-2 text-base data-readonly:nine-field-readonly aria-invalid:nine-field-invalid focus:nine-field-focus disabled:nine-field-disabled disabled:text-{disabled}"
                )
            } else {
                String::new()
            };
            variants.insert(
                slot.into(),
                json!(format!("scheme-{s} nine-{image}{states} bg-none bg-transparent hover:bg-transparent active:bg-transparent text-{fg}")),
            );
        }
        // The combat HUD's ability slot in the theme's own slot art (combat
        // HUD design §6): a round shape rings an ultimate with a disc, any
        // other with its slot's rim, both filled in the accent.
        let round = matches!(
            recipe.get("shape").and_then(Value::as_str),
            Some("round" | "soft" | "brush")
        );
        let ring = if round { "nine-disc" } else { "nine-slot-rim" };
        for (slot, dress) in [
            (
                "Ability.frame",
                if round {
                    "nine-slot bg-transparent rounded-card transition duration-control"
                } else {
                    "nine-slot bg-transparent rounded-none transition duration-control"
                }
                .to_owned(),
            ),
            (
                "Ability.frame.ultimate",
                if round {
                    "nine-none bg-transparent rounded-full"
                } else {
                    "nine-none bg-transparent"
                }
                .to_owned(),
            ),
            (
                "Ability.frame.ready",
                "shadow-glow animate-breathe".to_owned(),
            ),
            (
                "Ability.face",
                "inset-1 rounded-field overflow-hidden bg-transparent".to_owned(),
            ),
            (
                "Ability.face.ultimate",
                if round {
                    "inset-1/8 rounded-full nine-slot".to_owned()
                } else {
                    "inset-1.5 rounded-none nine-slot".to_owned()
                },
            ),
            (
                "Ability.ring",
                format!(
                    "{ring} bg-track {}",
                    if round {
                        "rounded-full"
                    } else {
                        "rounded-none"
                    }
                ),
            ),
            ("Ability.ring.fill", format!("{ring} bg-hud-accent")),
        ] {
            variants.insert(slot.into(), json!(dress));
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
            let role = match image {
                "button-primary" => "primary",
                "button-danger" => "danger",
                other => other,
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
        solve_inks(&seed_css(&pack), &grounds, &mut colors, false);
        let mut more = Map::new();
        solve_inks(&seed_css(&pack), &grounds, &mut more, true);
        set_more(&mut pack, more);
        art = json!({
            "scale": 4,
            "palette": palette,
            "material": m.get("art").cloned().unwrap_or(json!({})),
            "shape": shape.as_ref().and_then(|s| s.get("art")).cloned().unwrap_or(json!({})),
            "ornament": ornaments,
        });
        // A material's panel dressing (a coloured header band) goes to the
        // art as the generator reads it.
        if let Some(p) = m.get("panel") {
            art["panel"] = p.clone();
        }
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
    if let Some(ef) = &effects {
        let accent_hex = col("accent")
            .or_else(|| col("source"))
            .and_then(|c| lch(c).ok())
            .map(hex);
        let glow = ef.get("textGlow").and_then(Value::as_f64).unwrap_or(0.0);
        if glow > 0.0 {
            let blur = (glow * 24.0).round().clamp(4.0, 18.0) as u32;
            let alpha = (glow.clamp(0.0, 1.0) * 255.0).round() as u8;
            let tint = accent_hex.as_deref().unwrap_or("#ffffff");
            let mut ts = Map::new();
            ts.insert(
                "glow".into(),
                json!(format!("0 0 {blur}px {tint}{alpha:02x}")),
            );
            ts.insert(
                "tag".into(),
                json!(format!(
                    "1.5px 0 0 contrast-color(currentColor), -1.5px 0 0 contrast-color(currentColor), 0 1.5px 0 contrast-color(currentColor), 0 -1.5px 0 contrast-color(currentColor), 0 0 {blur}px {tint}{alpha:02x}"
                )),
            );
            deep(
                theme.entry("textShadow").or_insert(json!({})),
                &Value::Object(ts),
            );
        }
        if let Some(kind) = ef.get("shadow").and_then(Value::as_str) {
            let pixel = ef.get("pixel").and_then(Value::as_bool).unwrap_or(false);
            let mut sh = Map::new();
            match kind {
                "none" if pixel => {
                    sh.insert("panel".into(), json!("0 3px 0 0 #00000099"));
                    sh.insert("plate".into(), json!("0 2px 0 0 #00000080"));
                    sh.insert("float".into(), json!("0 2px 0 0 #00000099"));
                    sh.insert("overlay".into(), json!("0 4px 0 0 #000000b3"));
                    sh.insert("raise".into(), json!("0 2px 0 0 #00000073"));
                    sh.insert("raise-hover".into(), json!("0 3px 0 0 #00000099"));
                }
                "none" => {
                    for k in ["panel", "plate", "float", "overlay", "raise", "raise-hover"] {
                        sh.insert(k.into(), json!("none"));
                    }
                }
                "glow" => {
                    let tint = accent_hex.as_deref().unwrap_or("#38e8ff");
                    sh.insert("glow-sm".into(), json!(format!("0 0 8px 0 {tint}cc")));
                    sh.insert("glow".into(), json!(format!("0 0 14px 0 {tint}cc")));
                    sh.insert("glow-lg".into(), json!(format!("0 0 22px 0 {tint}e6")));
                    sh.insert(
                        "plate".into(),
                        json!(format!(
                            "inset 0 1px 0 0 #ffffff1f, 0 0 14px -2px {tint}59, 0 6px 18px -6px #00000099"
                        )),
                    );
                    sh.insert(
                        "panel".into(),
                        json!(format!(
                            "inset 0 1px 0 0 #ffffff24, 0 0 20px -4px {tint}4d, 0 20px 44px -16px #000000cc"
                        )),
                    );
                }
                "soft" => {
                    sh.insert(
                        "panel".into(),
                        json!({
                            "light": "inset 0 1px 0 0 #ffffffcc, 0 14px 36px -12px #0a10202e",
                            "dark": "inset 0 1px 0 0 #ffffff14, 0 18px 42px -12px #00000099"
                        }),
                    );
                    sh.insert(
                        "plate".into(),
                        json!({
                            "light": "inset 0 1px 0 0 #ffffffb3, 0 6px 18px -4px #0a102026",
                            "dark": "inset 0 1px 0 0 #ffffff12, 0 8px 22px -4px #00000073"
                        }),
                    );
                    sh.insert(
                        "float".into(),
                        json!({
                            "light": "0 8px 24px -6px #0a102026",
                            "dark": "0 10px 28px -6px #00000080"
                        }),
                    );
                }
                "drop" => {
                    if let Some(tint) = accent_hex.as_deref() {
                        sh.insert("glow-sm".into(), json!(format!("0 0 8px 0 {tint}")));
                        sh.insert("glow".into(), json!(format!("0 0 12px 0 {tint}")));
                        sh.insert("glow-lg".into(), json!(format!("0 0 16px 0 {tint}")));
                    }
                }
                _ => {}
            }
            if !sh.is_empty() {
                deep(
                    theme.entry("shadow").or_insert(json!({})),
                    &Value::Object(sh),
                );
            }
        }
    }
    if let Some(d) = density
        .as_ref()
        .and_then(|d| d.get("density").and_then(Value::as_str))
    {
        let extra: &[(&str, &str)] = match d {
            "comfortable" => &[
                ("Button.plate", "min-h-10 px-4"),
                ("Button.plate.primary", "min-h-10 px-4"),
                ("Button.plate.danger", "min-h-10 px-4"),
                ("Field.frame", "min-h-11 px-3.5 py-2.5"),
                ("Row.frame", "min-h-11 px-3.5 py-2"),
                ("Tab.item", "min-h-11 px-4"),
            ],
            "touch" => &[
                ("Button.plate", "min-h-11 px-4"),
                ("Button.plate.primary", "min-h-11 px-4"),
                ("Button.plate.danger", "min-h-11 px-4"),
                ("Field.frame", "min-h-12 px-4 py-2.5"),
                ("Row.frame", "min-h-12 px-4 py-2.5"),
                ("Tab.item", "min-h-12 px-4"),
            ],
            _ => &[],
        };
        for &(slot, add) in extra {
            let base = variants
                .get(slot)
                .and_then(Value::as_str)
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    crate::pack::design::SLOTS
                        .iter()
                        .find(|(k, _, _)| *k == slot)
                        .map(|(_, _, v)| (*v).to_owned())
                        .unwrap_or_default()
                });
            let merged = if base.is_empty() {
                add.to_owned()
            } else {
                format!("{base} {add}")
            };
            variants.insert(slot.into(), json!(merged));
        }
    }
    if pack.contains_key("seeds") {
        solve_pairs(&seed_css(&pack), &mut colors);
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
