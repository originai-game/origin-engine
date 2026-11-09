use super::*;

/// What a version 2 file's blocks give, before they are checked complete.
#[derive(Default)]
struct V2 {
    colors: Option<tokens::ColorTokens>,
    typography: Option<tokens::TypeScale>,
    space: Option<[f32; SpaceStep::COUNT]>,
    size: Option<[f32; SizeStep::COUNT]>,
    radius: Option<[f32; RadiusStep::COUNT]>,
    stroke: Option<[f32; StrokeWidth::COUNT]>,
    elevation: Option<[tokens::Elevation; Level::COUNT]>,
    shape: Option<[ShapeToken; ElementClass::COUNT]>,
    motion: Option<tokens::MotionParse>,
    springs: Vec<(&'static str, tokens::SpringToken)>,
    feedback: Option<[tokens::Feedback; SoundEvent::COUNT]>,
}

fn once<T>(p: &P, slot: &mut Option<T>, v: T) -> Result<(), String> {
    if slot.is_some() {
        return Err(p.err("is set twice"));
    }
    *slot = Some(v);
    Ok(())
}

/// Parses a UI style. Version 2 (docs/ui.md §3):
///
/// ```text
/// type style
/// version 2
/// ---
/// name "fantasy"
/// scheme dark                                # dark (the default) or light: what the game reads as UiTheme.scheme
/// atlas "atlas.png" 384 256                  # its asset id and size in texels
/// scale 1                                    # CSS px per atlas texel for slice corners
/// palette { backdrop "#1a120b" … }           # every ThemeColor role (all required)
/// colors {                                   # every token required
///   success icon "check"                     # a status: the palette's colour, its cue (icon "<name>", pattern <word>, pips <n>)
///   epic "#a335ee" pips 4                    # common … mythic: a colour and its cue, a different cue per tier
///   team1 "#3d7eff"                          # team1 … team8, then physical fire ice lightning poison holy shadow arcane: a colour [and a cue]
///   series1 "#4e79a7"                        # series1 … series8
/// }
/// roles { corruption "#9b5cff" }             # optional: the style's own colour roles (lowerCamelCase, up to 16), read by name like a palette role
/// typography {                               # display title heading body label caption numeric (all required)
///   display 42 48 0.02 700 display {         # size, line height, tracking em, weight, font role (body, display, numeric, rounded)
///     phonePortrait 34 40                    # per breakpoint: size, line height
///   }
/// }
/// space { xxs 2 xs 4 sm 8 md 12 lg 16 xl 24 xxl 32 xl3 48 }  # one per line, every name
/// size { xs 16 sm 24 md 32 lg 48 xl 64 xxl 96 touch 44 }
/// radius { xs 2 sm 4 md 6 lg 9 xl 12 pill 9999 }
/// stroke { hairline 1 thin 1.5 regular 2 thick 3 focus 2 }
/// elevation { raised { shadow 0 4 8 0 "#00000066" } floating … overlay … top … }   # or `<level> none`
/// shape { panel round 4 button squircle md slot notch 6 … }  # every element class: corner and radius (px or a radius name)
/// ramp { displayLg 44 48 }                   # the v1 TypeStepName ramp the widgets draw with
/// fonts { … }  icons "icons.png" 8 4 { names … }
/// soft 252 104 64 64 24  corner …  divider …  flourish …  tile 204 172 64 64 64 64 0.5 6
/// textShadow display 0 2 0 "#000000a0"  textGlow …  textWeight …
/// motion {
///   enter scale  toast slideDown  progress 420  shake 5  pop 0.1  hoverSpring 350 0
///   spring ui 180 0                          # ui panel emphasis playful: duration ms, bounce -0.75 … 0.95
///   ease enter 0.23 1 0.32 1                 # standard enter exit emphasis: control points, linear or steps <n>
///   duration base 300                        # instant fast base slow spring bounce
///   stagger 35  staggerCap 300  overshoot 0.15
/// }
/// sounds {                                   # every event of §7.8
///   press "audio/press.wav" volume 0.8 1 pitch 0.95 1.05 haptic tick
///   hover none haptic none                   # several ids: a set played without repeats
/// }
/// dress "Button.base[pressed]" {             # a part dressing (crate::dress)
///   slice 276 4 48 48 10 tint
///   tone 0.69 1
///   shadow 0 1 2 0 "#00000080"
///   fill linear 90 "#ffd34a" "#e08a1e"        # paints, strokes, effects… kept for the UI pass
/// }
/// ```
///
/// Version 1 (read until `og style upgrade` has run everywhere) has
/// `radius { small control bar panel }`, `durations`, and the per-part lines
/// `slice <part> …`, `tone <part> …`, `shadow <part> …`, `glow <part> …`,
/// `glass <part> …`, `corners <parts>`, `tile … <parts>`, `press`, `hover`,
/// `sound <event> "<id>"`, with `hoverSpring` and `pressSpring` as damping
/// ratio and response ms; its tokens are [`StyleTokens::derive`]d.
pub fn parse_style(text: &str) -> Result<StyleFile, String> {
    let mut doc = super::css::read_style_doc(text)?;
    super::css::resolve_tokens(&mut doc)?;
    if doc.ty == "skin" {
        return Err(
            "`type skin` is now `type style`: change the first line, replace `base` with `scheme`, and give the palette every role and the radius block small, control, bar and panel".into(),
        );
    }
    if doc.ty != STYLE_TYPE {
        return Err(format!(
            "a UI style is `type {STYLE_TYPE}`, not `type {}`",
            doc.ty
        ));
    }
    let v2 = match doc.version {
        1 => false,
        2 | 3 => true,
        v => return Err(format!("style version {v} (this engine reads 1 to 3)")),
    };
    let body = &doc.body;
    if let Some(e) = body.get("base") {
        return Err(P { e }.err(
            "`base` is gone: a style is complete; write `scheme dark` or `scheme light` and set every palette role",
        ));
    }
    let scheme = match body.get("scheme").map(|e| P { e }.word(0)).transpose()? {
        None | Some("dark") => Scheme::Dark,
        Some("light") => Scheme::Light,
        Some(w) => return Err(format!("scheme is dark or light, not '{w}'")),
    };
    let mut theme = Theme::new(scheme);
    // A version 1 style (or `screens 1`) was made over the prototype ramp,
    // which set no tracking: a step it leaves untracked draws untracked, as
    // its game shipped, not with the design language's tightening.
    let legacy = !v2
        || body
            .get("screens")
            .is_some_and(|e| P { e }.num(0).is_ok_and(|n| n == 1.0));
    if legacy {
        for s in theme.type_ramp.steps_mut() {
            s.tracking_em = 0.0;
            s.cjk_tracking_em = 0.0;
        }
    }
    let mut file = StyleFile {
        name: String::new(),
        version: doc.version,
        theme,
        atlas: None,
        atlas_dense: Default::default(),
        fonts: Default::default(),
        font_weights: [0; 6],
        font_faces: Vec::new(),
        icons: None,
        sounds: Default::default(),
        feedback_sounds: Default::default(),
        dressings: Vec::new(),
        recolour: Vec::new(),
    };
    let mut size = (0.0f32, 0.0f32);
    if let Some(e) = body.get("atlas") {
        let p = P { e };
        file.atlas = Some(p.string(0)?.to_string());
        size = (p.num(1)?, p.num(2)?);
        if size.0 < 1.0 || size.1 < 1.0 {
            return Err(p.err("takes the atlas width and height in texels"));
        }
        // Then the denser copies it has: 2, 3 or both.
        for i in 3..e.values.len() {
            let n = p.num(i)?;
            let k = match n {
                2.0 => 0,
                3.0 => 1,
                _ => return Err(p.err("names its denser copies by density: 2, 3 or both")),
            };
            let base = p.string(0)?;
            file.atlas_dense[k] = Some(dense_name(base, n as u32));
        }
    }
    let mut scale = 1.0f32;
    let (mut has_palette, mut has_radius) = (false, false);
    if let Some(e) = body.get("scale") {
        scale = P { e }.num(0)?;
    }
    // The icons first: a colour's cue names one.
    if let Some(e) = body.get("icons") {
        let p = P { e };
        let names = match &e.block {
            Some(b) => b
                .get_all("names")
                .flat_map(|n| n.values.iter())
                .map(|v| {
                    v.as_str()
                        .map(str::to_string)
                        .ok_or_else(|| p.err("names takes quoted strings"))
                })
                .collect::<Result<Vec<_>, _>>()?,
            None => Vec::new(),
        };
        let (cols, rows) = (p.num(1)?, p.num(2)?);
        if cols < 1.0 || rows < 1.0 {
            return Err(p.err("takes the grid's columns and rows (at least 1)"));
        }
        file.icons = Some((p.string(0)?.to_string(), cols as u32, rows as u32, names));
    }
    let icon_names: Vec<String> = file.icons.as_ref().map(|i| i.3.clone()).unwrap_or_default();
    let mut v = V2::default();
    // The radius scale first: a shape names its steps.
    if v2 && let Some(e) = body.get("radius") {
        v.radius = Some(tokens::parse_lengths(
            &P { e },
            &names_of(RadiusStep::ALL, RadiusStep::name),
            "radius",
        )?);
    }
    // The style's own colour roles first: a dressing's paint names them.
    if let Some(e) = body.get("roles") {
        let p = P { e };
        if !v2 {
            return Err(p.err(
                "is a version 2 block: write `version 2` and the rest of the version 2 tokens (og style upgrade converts a version 1 style)",
            ));
        }
        theme.roles = parse_roles(&p)?;
    }
    let mut dressings: Vec<Dressing> = Vec::new();
    for e in &body.entries {
        let p = P { e };
        let needs_atlas = || {
            if file.atlas.is_none() {
                Err(p.err("needs an `atlas` line first"))
            } else {
                Ok(())
            }
        };
        let key = e.key.as_str();
        const V1_ONLY: [&str; 10] = [
            "slice",
            "tone",
            "shadow",
            "glow",
            "glass",
            "corners",
            "press",
            "hover",
            "sound",
            "durations",
        ];
        const V2_ONLY: [&str; 10] = [
            "colors",
            "roles",
            "typography",
            "space",
            "size",
            "stroke",
            "elevation",
            "shape",
            "sounds",
            "dress",
        ];
        if v2 && V1_ONLY.contains(&key) {
            return Err(p.err(match key {
                "durations" => "is a version 1 block: in version 2 the durations are `duration <name> <ms>` lines of the motion block",
                "sound" => "is a version 1 line: in version 2 every event is a line of the `sounds { }` block",
                _ => "is a version 1 part line: in version 2 a part is dressed in a `dress \"<Component.part>\" { … }` block (og style upgrade converts a version 1 style)",
            }));
        }
        if !v2 && V2_ONLY.contains(&key) {
            return Err(p.err(
                "is a version 2 block: write `version 2` and the rest of the version 2 tokens (og style upgrade converts a version 1 style)",
            ));
        }
        match key {
            "scheme" | "atlas" | "scale" | "icons" | "roles" => {}
            "name" => file.name = p.string(0)?.to_string(),
            "recolour" => {
                needs_atlas()?;
                file.recolour.push(recolour_at(&p, &theme.palette)?);
            }
            // The templates' screens the style was made for: 1 keeps a
            // shipped game's (a version 1 style is 1 whatever it says).
            "screens" => match p.num(0)? {
                1.0 => theme.art.legacy = true,
                2.0 => {}
                _ => return Err(p.err("is 1 (the screens a version 1 style drew) or 2")),
            },
            "palette" => {
                let block = p.block()?;
                for c in &block.entries {
                    let q = P { e: c };
                    let color = q.color(0)?;
                    *palette_color(&mut theme.palette, &c.key)
                        .ok_or_else(|| q.err("is not a ThemeColor name"))? = color;
                }
                // Text over a scene reads light on its dark separation in
                // every scheme: a light scheme's (dark ink) scene ink is its
                // inverse ink, and its plate the scrim, not its pale hud.
                let light = theme.palette.ink.luminance() < theme.palette.ink_inverse.luminance();
                let added = [
                    ("selection", "controlStrong"),
                    ("onSelection", "ink"),
                    ("accent", "highlight"),
                    ("sceneInk", if light { "inkInverse" } else { "ink" }),
                    ("scenePlate", if light { "scrim" } else { "hud" }),
                    ("sceneShadow", "shadow"),
                ];
                for (name, fallback) in added {
                    if block.get(name).is_none() {
                        let color = theme.palette.get(fallback).expect("palette fallback");
                        *theme.palette.get_mut(name).expect("palette role") = color;
                    }
                }
                let missing: Vec<&str> = Palette::NAMES
                    .iter()
                    .copied()
                    .filter(|n| block.get(n).is_none() && !added.iter().any(|(name, _)| n == name))
                    .collect();
                if !missing.is_empty() {
                    return Err(p.err(&format!(
                        "leaves out {}: a style sets every palette role",
                        missing.join(", ")
                    )));
                }
                has_palette = true;
            }
            "radius" if v2 => {}
            "radius" => {
                let block = p.block()?;
                for c in &block.entries {
                    let q = P { e: c };
                    *radius_slot(&mut theme.radius, &c.key).ok_or_else(|| {
                        q.err("is not a radius name (small, control, bar, panel)")
                    })? = q.num(0)?;
                }
                let missing: Vec<&str> = RADII
                    .iter()
                    .copied()
                    .filter(|n| block.get(n).is_none())
                    .collect();
                if !missing.is_empty() {
                    return Err(p.err(&format!(
                        "leaves out {}: a style sets every radius",
                        missing.join(", ")
                    )));
                }
                has_radius = true;
            }
            "ramp" => {
                for c in &p.block()?.entries {
                    let q = P { e: c };
                    let s = ramp_step(&mut theme.type_ramp, &c.key)
                        .ok_or_else(|| q.err("is not a TypeStepName"))?;
                    // Numbers first, then the step's OpenType features as
                    // words (`meta 12 16 20 0.04 0 tnum`, or `meta 12 16 tnum`).
                    let words = c.values.iter().position(|v| v.as_word().is_some());
                    let n = words.unwrap_or(c.values.len());
                    let or = |i: usize, d: f32| if i < n { q.num(i) } else { Ok(d) };
                    s.size = q.num(0)?;
                    s.line_height = q.num(1)?;
                    s.cjk_line_height = or(2, s.line_height.max(s.cjk_line_height))?;
                    s.tracking_em = or(3, s.tracking_em)?;
                    s.cjk_tracking_em = or(4, s.cjk_tracking_em)?;
                    if let Some(first) = words {
                        let tags = (first..c.values.len())
                            .map(|i| q.word(i))
                            .collect::<Result<Vec<_>, _>>()?;
                        s.features = crate::shape::Features::parse(tags).map_err(|m| q.err(&m))?;
                    }
                }
            }
            "durations" => {
                for c in &p.block()?.entries {
                    let q = P { e: c };
                    *duration_slot(&mut theme.durations_ms, &c.key)
                        .ok_or_else(|| q.err("is not a duration name"))? = q.num(0)?.max(0.0);
                }
            }
            "fonts" => {
                // `<role> [<locale>] [<weight>] [italic] "<file>"`: the
                // first plain face of a role is its own (cjk's in
                // Simplified Chinese forms); the rest are its other faces.
                for c in &p.block()?.entries {
                    let q = P { e: c };
                    if c.key == "displayMin" {
                        let n = q.num(0)?;
                        if c.values.len() != 1 || !(0.0..=96.0).contains(&n) {
                            return Err(q.err("takes one size of 0 to 96 CSS px"));
                        }
                        theme.art.display_min = n;
                        continue;
                    }
                    let (i, role, cjk) = match c.key.as_str() {
                        "body" => (0, crate::FontRole::Body, false),
                        "display" => (1, crate::FontRole::Display, false),
                        "numeric" | "mono" => (2, crate::FontRole::Mono, false),
                        "fallback" => (3, crate::FontRole::Body, false),
                        "cjk" => (4, crate::FontRole::Body, true),
                        "cjkDisplay" => (5, crate::FontRole::Display, true),
                        _ => {
                            return Err(q.err(
                                "is a font role (body, display, numeric, fallback, cjk or cjkDisplay) or displayMin",
                            ));
                        }
                    };
                    let last = c.values.len().saturating_sub(1);
                    let path = q.string(last)?.to_string();
                    let (mut locale, mut weight, mut italic) = (None, 0u16, false);
                    for k in 0..last {
                        if let Ok(n) = q.num(k) {
                            if !(100.0..=900.0).contains(&n) || n % 100.0 != 0.0 {
                                return Err(q.err("takes a weight of 100 to 900 in hundreds"));
                            }
                            weight = n as u16;
                            continue;
                        }
                        match q.word(k)? {
                            "italic" => italic = true,
                            w if cjk => {
                                locale = Some(
                                    crate::CjkLocale::ALL
                                        .into_iter()
                                        .find(|l| l.tag() == w)
                                        .ok_or_else(|| {
                                            q.err(&format!(
                                                "names '{w}', not a CJK locale: zh-Hans, zh-Hant, ja or ko"
                                            ))
                                        })?,
                                );
                            }
                            w => {
                                return Err(q.err(&format!(
                                    "takes a weight, `italic` and the file, not '{w}'"
                                )));
                            }
                        }
                    }
                    if i == 3 && (weight != 0 || italic) {
                        return Err(q.err("is one face: the fallback takes no weight or italic"));
                    }
                    let locale = locale.unwrap_or_default();
                    if file.fonts[i].is_none() && !italic && locale == crate::CjkLocale::Hans {
                        file.fonts[i] = Some(path);
                        file.font_weights[i] = weight;
                    } else {
                        file.font_faces.push(crate::style_file::StyleFace {
                            role,
                            cjk,
                            locale,
                            weight: if weight == 0 { 400 } else { weight },
                            italic,
                            path,
                        });
                    }
                }
            }
            "slice" => {
                needs_atlas()?;
                let part = part_at(&p, 0)?;
                theme.art.slices[part.index()] = Some(slice_at(&p, 1, size, scale)?);
            }
            "soft" => {
                needs_atlas()?;
                theme.art.soft = Some(slice_at(&p, 0, size, scale)?);
            }
            "tone" => {
                let part = part_at(&p, 0)?;
                let slice = theme.art.slices[part.index()].as_mut().ok_or_else(|| {
                    p.err(&format!(
                        "names '{}', which has no slice line before it",
                        part.name()
                    ))
                })?;
                slice.tone = tone_at(&p, 1)?;
            }
            "corner" | "divider" | "flourish" | "crest" | "edge" => {
                needs_atlas()?;
                let (uv, _) = region(&p, 0, size)?;
                let ink = match e.values.get(6) {
                    None => Ink::Authored,
                    Some(_) => ink_at(&p, 6)?,
                };
                let o = Ornament {
                    uv,
                    w: p.num(4)?,
                    h: p.num(5)?,
                    ink,
                    offset: p.num_or(7, 0.0)?,
                };
                match key {
                    "corner" => theme.art.corner = Some(o),
                    "divider" => theme.art.divider = Some(o),
                    "crest" => theme.art.crest = Some(o),
                    "edge" => theme.art.edge = Some(o),
                    _ => theme.art.flourish = Some(o),
                }
            }
            "corners" => theme.art.corner_parts = parts_mask(&p, 0)?,
            "tile" => {
                needs_atlas()?;
                let (uv, _) = region(&p, 0, size)?;
                let numbers = e.values[4.min(e.values.len())..]
                    .iter()
                    .take_while(|v| v.as_number().is_some())
                    .count();
                if v2 && e.values.len() > 4 + numbers {
                    return Err(p.err(
                        "names its parts, as version 1 did: in version 2 each part's dressing says `tile`",
                    ));
                }
                theme.art.tile = Some(Tile {
                    uv,
                    w: p.num(4)?,
                    h: p.num(5)?,
                    alpha: p.num(6)?.clamp(0.0, 1.0),
                    inset: if numbers >= 4 { p.num(7)? } else { 0.0 },
                    parts: parts_mask(&p, 4 + numbers)?,
                });
            }
            "shadow" => {
                let part = part_at(&p, 0)?;
                theme.art.shadows[part.class().index()] = Some(tokens::shadow_at(&p, 1)?);
            }
            "glow" => {
                let part = part_at(&p, 0)?;
                theme.art.glows[part.class().index()] = Some(glow_at(&p, 1)?);
            }
            "glass" => {
                let part = part_at(&p, 0)?;
                theme.art.glass[part.class().index()] = Some(glass_at(&p, 1)?);
            }
            "textShadow" => {
                let i = role_index(&p, p.word(0)?)?;
                theme.art.text_shadows[i] = Some(Shadow {
                    dx: p.num(1)?,
                    dy: p.num(2)?,
                    blur: p.num(3)?.max(0.0),
                    spread: 0.0,
                    color: p.color(4)?,
                });
            }
            "textGlow" => {
                let i = role_index(&p, p.word(0)?)?;
                theme.art.text_glows[i] = Some(glow_at(&p, 1)?);
            }
            "textWeight" => {
                let i = role_index(&p, p.word(0)?)?;
                let w = p.num(1)?;
                if !(0.0..=2.0).contains(&w) {
                    return Err(p.err("takes the thickening in device pixels, 0 to 2"));
                }
                theme.art.text_weights[i] = w;
            }
            "press" => theme.art.press = Some(press_at(&p)?),
            "hover" => theme.art.hover = Some(hover_at(&p)?),
            "motion" => {
                let mut tokens_motion = v2.then(tokens::MotionParse::new);
                if v2 && v.motion.is_some() {
                    return Err(p.err("is set twice"));
                }
                let m = &mut theme.art.motion;
                for c in &p.block()?.entries {
                    let q = P { e: c };
                    match c.key.as_str() {
                        "enter" | "toast" => {
                            let w = q.word(0)?;
                            let k = kind_of(w).ok_or_else(|| q.err(&format!("'{w}' is fade, slideUp, slideDown, slideLeft, slideRight, scale, pop or glitch")))?;
                            // `glitch <strength>`: how far it tears (0.05–2, 1 when left out).
                            if k == TransitionKind::Glitch {
                                m.glitch = q.num_or(1, 1.0)?.clamp(0.05, 2.0);
                            } else if c.values.len() > 1 {
                                return Err(q.err(&format!(
                                    "{w} takes no number (only glitch takes a strength)"
                                )));
                            }
                            if c.key == "enter" {
                                m.enter = Some(k);
                            } else {
                                m.toast = Some(k);
                            }
                        }
                        "stagger" if !v2 => m.stagger_ms = q.num(0)?.max(0.0),
                        "progress" => m.progress_ms = q.num(0)?.max(0.0),
                        "toastHold" => {
                            m.toast_hold_ms = q.num(0)?.max(1.0);
                            m.toast_danger_ms = q.num_or(1, m.toast_hold_ms)?.max(1.0);
                        }
                        "screen" => m.screen_ms = q.num(0)?.max(1.0),
                        "magnet" => m.magnet = Some(q.num(0)?.clamp(0.0, 1.0)),
                        "shake" => m.shake = q.num(0)?.max(0.0),
                        "pop" => m.pop = q.num(0)?.clamp(0.0, 1.0),
                        "hoverSpring" | "pressSpring" => {
                            let spring = if v2 {
                                // Version 2: duration and bounce, as every spring.
                                let s = tokens::spring_at(&q, 0)?;
                                v.springs.push((
                                    if c.key == "hoverSpring" {
                                        "hoverSpring"
                                    } else {
                                        "pressSpring"
                                    },
                                    s,
                                ));
                                s.to_spring()
                            } else {
                                crate::theme::Spring {
                                    damping: q.num(0)?.clamp(0.05, 4.0),
                                    response_ms: q.num(1)?.clamp(16.0, 4000.0),
                                }
                            };
                            if c.key == "hoverSpring" {
                                m.hover_spring = Some(spring);
                            } else {
                                m.press_spring = Some(spring);
                            }
                        }
                        _ => {
                            if let Some(t) = tokens_motion.as_mut()
                                && t.line(&q)?
                            {
                                continue;
                            }
                            return Err(q.err(if v2 {
                                "is a motion setting: enter, toast, toastHold, screen, magnet, progress, shake, pop, hoverSpring, pressSpring, spring, ease, duration, stagger, staggerCap or overshoot"
                            } else {
                                "is a motion setting: enter, stagger, toast, toastHold, screen, magnet, progress, shake, pop, hoverSpring or pressSpring"
                            }));
                        }
                    }
                }
                v.motion = tokens_motion;
            }
            "sound" => {
                let w = p.word(0)?;
                let s = UiSound::from_name(w).ok_or_else(|| {
                    p.err(&format!(
                        "'{w}' is hover, press, open, close, success or error"
                    ))
                })?;
                file.sounds[s as usize] = Some(p.string(1)?.to_string());
            }
            "colors" => once(&p, &mut v.colors, tokens::parse_colors(&p, &icon_names)?)?,
            "typography" => once(&p, &mut v.typography, tokens::parse_typography(&p)?)?,
            "space" => once(
                &p,
                &mut v.space,
                tokens::parse_lengths(&p, &names_of(SpaceStep::ALL, SpaceStep::name), "space")?,
            )?,
            "size" => once(
                &p,
                &mut v.size,
                tokens::parse_lengths(&p, &names_of(SizeStep::ALL, SizeStep::name), "size")?,
            )?,
            "stroke" => once(
                &p,
                &mut v.stroke,
                tokens::parse_lengths(
                    &p,
                    &names_of(StrokeWidth::ALL, StrokeWidth::name),
                    "stroke width",
                )?,
            )?,
            "elevation" => once(&p, &mut v.elevation, tokens::parse_elevation(&p)?)?,
            "shape" => {
                let radius = v.radius.as_ref().ok_or_else(|| {
                    p.err("needs the `radius { xs sm md lg xl pill }` scale its corners name")
                })?;
                let s = tokens::parse_shape(&p, radius)?;
                once(&p, &mut v.shape, s)?
            }
            "sounds" => {
                let (f, ids) = tokens::parse_sounds(&p)?;
                once(&p, &mut v.feedback, f)?;
                file.feedback_sounds = ids;
            }
            "dress" => {
                let sel = Selector::parse(p.string(0)?).map_err(|m| p.err(&m))?;
                if e.values.len() != 1 {
                    return Err(p.err("takes one quoted selector, then a { } block"));
                }
                let block = p.block()?;
                let i = match dressings.iter().position(|d| d.selector == sel) {
                    Some(i) => i,
                    None => {
                        dressings.push(Dressing {
                            selector: sel,
                            frame: DressFrame::default(),
                            draw: Default::default(),
                            recolour: None,
                            source: String::new(),
                            lines: Vec::new(),
                        });
                        dressings.len() - 1
                    }
                };
                let d = &mut dressings[i];
                d.lines.push(e.line);
                let atlas = crate::paint_entry::AtlasContext {
                    size: file.atlas.as_ref().map(|_| size),
                    scale,
                    roles: theme.roles,
                };
                let mut seen = crate::dress::DrawSeen::default();
                for c in &block.entries {
                    let q = P { e: c };
                    let bare = || {
                        if c.values.is_empty() {
                            Ok(true)
                        } else {
                            Err(q.err("takes no values"))
                        }
                    };
                    match c.key.as_str() {
                        "slice" if c.values.len() == 1 && q.word(0).is_ok_and(|w| w == "none") => {
                            d.frame.slice = None;
                            d.frame.unsliced = true;
                        }
                        "slice" => {
                            needs_atlas()?;
                            d.frame.slice = Some(slice_at(&q, 0, size, scale)?);
                        }
                        "tone" => {
                            let t = tone_at(&q, 0)?;
                            d.frame
                                .slice
                                .as_mut()
                                .ok_or_else(|| {
                                    q.err("sets a tone, but the dressing has no slice before it")
                                })?
                                .tone = t;
                        }
                        "shadow" => d.frame.shadow = Some(tokens::shadow_at(&q, 0)?),
                        "glow" => d.frame.glow = Some(glow_at(&q, 0)?),
                        "glass" => d.frame.glass = Some(glass_at(&q, 0)?),
                        "corners" => d.frame.corners = bare()?,
                        "crest" => d.frame.crest = bare()?,
                        "edges" => d.frame.edges = bare()?,
                        "tile" => d.frame.tile = bare()?,
                        "shape" => {
                            let radius = v.radius.as_ref().ok_or_else(|| {
                                q.err("needs the `radius { … }` scale its corner names")
                            })?;
                            d.frame.shape = Some(tokens::shape_at(&q, 0, radius)?);
                        }
                        "press" => d.frame.press = Some(press_at(&q)?),
                        "hover" => d.frame.hover = Some(hover_at(&q)?),
                        "recolour"
                            if c.values.len() == 1 && q.word(0).is_ok_and(|w| w == "none") =>
                        {
                            d.recolour = Some(Vec::new());
                        }
                        "recolour" => {
                            needs_atlas()?;
                            d.recolour
                                .get_or_insert_with(Vec::new)
                                .push(recolour_at(&q, &theme.palette)?);
                        }
                        k if DRAW_KEYS.contains(&k) => {
                            if matches!(k, "ink" | "step")
                                && crate::dress::BOX_PARTS.contains(&d.selector.part.as_str())
                            {
                                return Err(q.err(&format!(
                                    "`{k}` dresses a text part ({}.text, {}.placeholder, or a kit's text element naming its part), not the box part .{}",
                                    d.selector.component, d.selector.component, d.selector.part
                                )));
                            }
                            d.draw.apply(c, &atlas, &mut seen)?
                        }
                        _ => {
                            return Err(q.err(&format!(
                                "is not a dressing setting: slice, tone, recolour, shadow, glow, glass, corners, crest, edges, tile, shape, press, hover, or (for the UI pass) {}",
                                DRAW_KEYS.join(", ")
                            )));
                        }
                    }
                }
            }
            k => return Err(format!("line {}: unknown style setting '{k}'", e.line)),
        }
    }
    if v2 {
        // The dressings of the twelve version 1 parts are what the widgets draw.
        let tile_parts =
            dressings
                .iter()
                .fold(0u32, |m, d| match StylePart::from_selector(&d.selector) {
                    Some(p) if d.frame.tile && p.class() == p => m | p.bit(),
                    _ => m,
                });
        if tile_parts != 0 && theme.art.tile.is_none() {
            return Err("a dressing says `tile`, but the style has no `tile` line".into());
        }
        if dressings.iter().any(|d| d.frame.crest) && theme.art.crest.is_none() {
            return Err("a dressing says `crest`, but the style has no `crest` line".into());
        }
        if dressings.iter().any(|d| d.frame.edges) && theme.art.edge.is_none() {
            return Err("a dressing says `edges`, but the style has no `edge` line".into());
        }
        if let Some(t) = theme.art.tile.as_mut() {
            t.parts = tile_parts;
        }
        for d in &dressings {
            let Some(part) = StylePart::from_selector(&d.selector) else {
                continue;
            };
            let a = &mut theme.art;
            let f = &d.frame;
            if f.slice.is_some() {
                a.slices[part.index()] = f.slice;
            }
            if part.class() == part {
                let i = part.index();
                a.shadows[i] = f.shadow.or(a.shadows[i]);
                a.glows[i] = f.glow.or(a.glows[i]);
                a.glass[i] = f.glass.or(a.glass[i]);
                if f.corners {
                    a.corner_parts |= part.bit();
                }
            }
            if part == StylePart::Button {
                a.press = f.press.or(a.press);
                a.hover = f.hover.or(a.hover);
            }
        }
        let need = |what: &str| {
            format!(
                "a version 2 style needs a `{what}` block (og style upgrade writes one from a version 1 style)"
            )
        };
        let radius = v
            .radius
            .ok_or_else(|| need("radius { xs sm md lg xl pill }"))?;
        let shape = v
            .shape
            .ok_or_else(|| need("shape { <element class> <corner> <radius> }"))?;
        let space = v
            .space
            .ok_or_else(|| need("space { xxs xs sm md lg xl xxl xl3 }"))?;
        let motion = v
            .motion
            .ok_or_else(|| need("motion { spring … ease … duration … }"))?;
        let mblock = P {
            e: body.get("motion").expect("a motion block"),
        };
        let motion = motion.finish(&mblock, &v.springs)?;
        let feedback = v
            .feedback
            .ok_or_else(|| need("sounds { <event> … haptic … }"))?;
        theme.tokens = StyleTokens {
            colors: v.colors.ok_or_else(|| need("colors { … }"))?,
            typography: v.typography.ok_or_else(|| need("typography { … }"))?,
            space,
            size: v
                .size
                .ok_or_else(|| need("size { xs sm md lg xl xxl touch }"))?,
            radius,
            stroke: v
                .stroke
                .ok_or_else(|| need("stroke { hairline thin regular thick focus }"))?,
            elevation: v
                .elevation
                .ok_or_else(|| need("elevation { raised floating overlay top }"))?,
            shape,
            motion,
            feedback,
        };
        // What the widgets read today, from the tokens.
        let t = &theme.tokens;
        theme.radius = Radius {
            small: t.shape(ElementClass::Slot).radius,
            control: t.shape(ElementClass::Button).radius,
            bar: t.shape(ElementClass::Bar).radius,
            panel: t.shape(ElementClass::Panel).radius,
        };
        let s = &t.space;
        theme.space = crate::Space {
            xxs: s[0],
            xs: s[1],
            sm: s[2],
            md: s[3],
            lg: s[4],
            xl: s[5],
            xxl: s[6],
            xl3: s[7],
        };
        theme.durations_ms = t.motion.durations;
        theme.art.motion.stagger_ms = t.motion.stagger_ms;
        for u in UiSound::ALL {
            file.sounds[u as usize] = file.feedback_sounds[u.event().index()].first().cloned();
        }
        has_radius = true;
    } else {
        for u in UiSound::ALL {
            file.feedback_sounds[u.event().index()] =
                file.sounds[u as usize].iter().cloned().collect();
        }
    }
    let needs_soft = theme.art.shadows.iter().flatten().any(|s| s.blur > 0.0)
        || theme.art.glows.iter().any(Option::is_some);
    if needs_soft && theme.art.soft.is_none() {
        return Err(
            "a soft shadow or a glow needs a `soft` line (a blurred white rectangle in the atlas)"
                .into(),
        );
    }
    if !has_palette {
        return Err("a style needs a `palette { … }` block with every ThemeColor role".into());
    }
    if !has_radius {
        return Err("a style needs a `radius { small control bar panel }` block".into());
    }
    if v2 {
        file.dressings = dressings;
    } else {
        theme.tokens = StyleTokens::derive(&theme, &icon_names);
        theme.art.legacy = true;
        theme.art.v1_controls = true;
        // A version 1 style's sounds are the one-sound sets of its events.
        for u in UiSound::ALL {
            if file.sounds[u as usize].is_some() {
                let f = &mut theme.tokens.feedback[u.event().index()];
                f.count = 1;
            }
        }
        file.dressings = v1_dressings(&theme.art);
    }
    // The motion code reads the style's motion tokens (docs/ui.md §3.1).
    theme.motion = crate::motion::MotionTokens::from_style(&theme.tokens);
    theme.type_ramp = theme
        .tokens
        .typography
        .ramp(crate::Breakpoint::Desktop, 1.0);
    for dressing in &mut file.dressings {
        for (_, value) in &mut dressing.draw.appearance {
            if let crate::doc::Lit::Lengths(ty, values) = value {
                let resolved = crate::doc::resolve_lengths(values, &theme)?;
                *value = crate::doc::parse_lit(*ty, &resolved)?;
            }
            if let crate::doc::Lit::Paint(crate::doc::Paint::Token(name)) = value
                && theme.color(name).is_none()
            {
                return Err(format!(
                    "unresolved colour token --{}",
                    crate::doc::css::kebab(name)
                ));
            }
        }
    }
    file.theme = theme;
    Ok(file)
}

/// `roles { corruption "#7a2fd0" … }`: the colour roles a style declares
/// of its own (docs/ui.md §3.1), beside the palette's. Each is a name in
/// lowerCamelCase that no palette role or colour token has, and one colour.
fn parse_roles(p: &P) -> Result<crate::style::Roles, String> {
    let block = p.block()?;
    let mut roles = crate::style::Roles::NONE;
    let token_names: Vec<String> = crate::tokens::colour_token_names();
    for e in &block.entries {
        let q = P { e };
        let name = e.key.as_str();
        if Palette::NAMES.contains(&name) {
            return Err(q.err(
                "is a palette role: set it in `palette { }`; `roles { }` declares the style's own",
            ));
        }
        if token_names.iter().any(|t| t == name) {
            return Err(q.err("is a colour token: set it in `colors { }`"));
        }
        let ok = name.chars().next().is_some_and(|c| c.is_ascii_lowercase())
            && name.chars().all(|c| c.is_ascii_alphanumeric())
            && name.len() <= crate::style::Roles::NAME;
        if !ok {
            return Err(q.err(&format!(
                "is not a role name: lowerCamelCase letters and digits, up to {} bytes",
                crate::style::Roles::NAME
            )));
        }
        if roles.get(name).is_some() {
            return Err(q.err("is set twice"));
        }
        if e.values.len() != 1 {
            return Err(q.err("takes one colour, \"#rrggbb\" or \"#rrggbbaa\""));
        }
        roles.set(name, q.color(0)?).map_err(|m| q.err(&m))?;
    }
    if roles.is_empty() {
        return Err(p.err("declares no role: leave the block out"));
    }
    Ok(roles)
}

/// `recolour "<from>" <role>`: a painted accent and the palette role it
/// follows (docs/art.md §2.7).
fn recolour_at(p: &P, palette: &Palette) -> Result<Recolour, String> {
    if p.e.values.len() != 2 {
        return Err(p.err(
            "takes the colour the art was painted in and the palette role it follows: recolour \"#f5e60c\" primary",
        ));
    }
    let from = p.color(0)?;
    let role = p.word(1)?;
    if palette.get(role).is_none() {
        return Err(p.err(&format!("'{role}' is not a palette role")));
    }
    Ok(Recolour {
        from,
        role: role.to_string(),
    })
}

/// The file name of an atlas's copy at `density`: `atlas.png` at 2 is
/// `atlas@2x.png`.
pub fn dense_name(base: &str, density: u32) -> String {
    match base.rfind('.').filter(|&i| !base[i..].contains('/')) {
        Some(i) => format!("{}@{density}x{}", &base[..i], &base[i..]),
        None => format!("{base}@{density}x"),
    }
}
