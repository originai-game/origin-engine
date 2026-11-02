use super::*;

// ───────────────────────────────────────────────────────────── parsing

/// A cue's icon, by name until the icons are known.
pub(crate) enum CueRef {
    None,
    Icon(String),
    Pattern(Pattern),
    Pips(u8),
}

/// Reads a cue at `values[at..]` (`icon "<name>"`, `pattern <word>`,
/// `pips <n>`), or none when nothing follows.
fn cue_at(p: &P, at: usize) -> Result<CueRef, String> {
    if p.e.values.len() <= at {
        return Ok(CueRef::None);
    }
    let cue =
        match p.word(at)? {
            "icon" => CueRef::Icon(p.string(at + 1)?.to_string()),
            "pattern" => {
                let w = p.word(at + 1)?;
                CueRef::Pattern(Pattern::from_name(w).ok_or_else(|| {
                    p.err(&format!("names the pattern '{w}' ({})", Pattern::names()))
                })?)
            }
            "pips" => {
                let n = p.num(at + 1)?;
                if !(1.0..=9.0).contains(&n) || n.fract() != 0.0 {
                    return Err(p.err("takes pips 1 to 9"));
                }
                CueRef::Pips(n as u8)
            }
            w => {
                return Err(p.err(&format!(
                "pairs its colour with a cue: icon \"<name>\", pattern <{}> or pips <n>, not '{w}'",
                Pattern::names().replace(", ", "|")
            )));
            }
        };
    let used = if matches!(cue, CueRef::None) { 0 } else { 2 };
    if p.e.values.len() > at + used {
        return Err(p.err("takes a colour and one cue"));
    }
    Ok(cue)
}

fn resolve_cue(p: &P, cue: CueRef, icons: &[String]) -> Result<Cue, String> {
    Ok(match cue {
        CueRef::None => Cue::None,
        CueRef::Pattern(x) => Cue::Pattern(x),
        CueRef::Pips(n) => Cue::Pips(n),
        CueRef::Icon(name) => Cue::Icon(icons.iter().position(|n| *n == name).ok_or_else(|| {
            if icons.is_empty() {
                p.err(&format!(
                    "names the icon \"{name}\", but the style has no `icons` names"
                ))
            } else {
                p.err(&format!(
                    "names the icon \"{name}\", which is not one of the style's icons"
                ))
            }
        })? as u16),
    })
}

fn missing_err(p: &P, missing: &[String], what: &str) -> Result<(), String> {
    if missing.is_empty() {
        Ok(())
    } else {
        Err(p.err(&format!("leaves out {}: {what}", missing.join(", "))))
    }
}

/// Every colour token's name (`colors { … }`), in the block's order.
pub fn colour_token_names() -> Vec<String> {
    Status::ALL
        .iter()
        .map(|s| s.name().to_string())
        .chain(Rarity::ALL.iter().map(|r| r.name().to_string()))
        .chain((1..=TEAMS).map(|k| format!("team{k}")))
        .chain(DamageType::ALL.iter().map(|d| d.name().to_string()))
        .chain((1..=SERIES).map(|k| format!("series{k}")))
        .collect()
}

/// The `colors { … }` block.
pub(crate) fn parse_colors(p: &P, icons: &[String]) -> Result<ColorTokens, String> {
    let block = p.block()?;
    let black = Swatch {
        color: Srgba::BLACK,
        cue: Cue::None,
    };
    let mut c = ColorTokens {
        status: [Cue::None; Status::COUNT],
        rarity: [black; Rarity::COUNT],
        team: [black; TEAMS],
        damage: [black; DamageType::COUNT],
        series: [Srgba::BLACK; SERIES],
    };
    let mut seen: Vec<&str> = Vec::new();
    for e in &block.entries {
        let q = P { e };
        if seen.contains(&e.key.as_str()) {
            return Err(q.err("is set twice"));
        }
        seen.push(&e.key);
        let key = e.key.as_str();
        let numbered = |prefix: &str, n: usize| {
            key.strip_prefix(prefix)
                .and_then(|k| k.parse::<usize>().ok())
                .filter(|k| (1..=n).contains(k))
                .map(|k| k - 1)
        };
        if let Some(s) = Status::from_name(key) {
            let cue = resolve_cue(&q, cue_at(&q, 0)?, icons)?;
            if cue == Cue::None {
                return Err(q.err(
                    "takes the status colour's cue (icon \"<name>\", pattern <word> or pips <n>): the colour is the palette's",
                ));
            }
            c.status[s.index()] = cue;
        } else if let Some(r) = Rarity::from_name(key) {
            let color = q.color(0)?;
            let cue = resolve_cue(&q, cue_at(&q, 1)?, icons)?;
            if cue == Cue::None {
                return Err(q.err(&format!(
                    "pairs the rarity colour with a cue, so no tier is told by colour alone: `{key} \"#rrggbb\" pips {}`",
                    r.index() + 1
                )));
            }
            c.rarity[r.index()] = Swatch { color, cue };
        } else if let Some(d) = DamageType::from_name(key) {
            c.damage[d.index()] = Swatch {
                color: q.color(0)?,
                cue: resolve_cue(&q, cue_at(&q, 1)?, icons)?,
            };
        } else if let Some(k) = numbered("team", TEAMS) {
            c.team[k] = Swatch {
                color: q.color(0)?,
                cue: resolve_cue(&q, cue_at(&q, 1)?, icons)?,
            };
        } else if let Some(k) = numbered("series", SERIES) {
            if e.values.len() != 1 {
                return Err(q.err("takes one colour"));
            }
            c.series[k] = q.color(0)?;
        } else {
            return Err(q.err(&format!(
                "is not a colour token: a status ({}), a rarity ({}), team1 to team{TEAMS}, a damage type ({}) or series1 to series{SERIES}",
                Status::names(),
                Rarity::names(),
                DamageType::names()
            )));
        }
    }
    let want: Vec<String> = colour_token_names()
        .into_iter()
        .filter(|n| !seen.contains(&n.as_str()))
        .collect();
    missing_err(p, &want, "a style sets every colour token")?;
    for (group, cues) in [
        ("status", c.status.to_vec()),
        ("rarity", c.rarity.iter().map(|s| s.cue).collect()),
    ] {
        for (i, a) in cues.iter().enumerate() {
            if let Some(j) = cues[..i].iter().position(|b| b == a) {
                let name = |k: usize| {
                    if group == "status" {
                        Status::ALL[k].name()
                    } else {
                        Rarity::ALL[k].name()
                    }
                };
                return Err(p.err(&format!(
                    "gives {} and {} the same cue: each {group} needs its own, so colour is never the only difference",
                    name(j),
                    name(i)
                )));
            }
        }
    }
    Ok(c)
}

/// `<role> <size> <lineHeight> <tracking em> <weight> <font> [<feature>…] [{ <breakpoint> <size> <lineHeight> … }]`,
/// the features OpenType tags (`smcp c2sc tnum lnum case`).
pub(crate) fn parse_typography(p: &P) -> Result<TypeScale, String> {
    let block = p.block()?;
    let zero = TypeToken {
        size: 0.0,
        line_height: 0.0,
        tracking_em: 0.0,
        weight: 400,
        font: FontRole::Body,
        features: crate::shape::Features::NONE,
    };
    let mut scale = TypeScale {
        steps: [[zero; 4]; TypeRole::COUNT],
    };
    let mut seen = Vec::new();
    for e in &block.entries {
        let q = P { e };
        let role = TypeRole::from_name(&e.key)
            .ok_or_else(|| q.err(&format!("is not a type step ({})", TypeRole::names())))?;
        if seen.contains(&role) {
            return Err(q.err("is set twice"));
        }
        seen.push(role);
        let size = q.num(0)?;
        let line_height = q.num(1)?;
        let tracking_em = q.num(2)?;
        let weight = q.num(3)?;
        if !(100.0..=900.0).contains(&weight) || weight % 100.0 != 0.0 {
            return Err(q.err("takes a weight of 100 to 900 in hundreds at position 4"));
        }
        let font = match q.word(4)? {
            "body" => FontRole::Body,
            "display" => FontRole::Display,
            "numeric" | "mono" => FontRole::Mono,
            "rounded" => FontRole::Rounded,
            w => {
                return Err(q.err(&format!(
                    "takes the font role body, display, numeric or rounded, not '{w}'"
                )));
            }
        };
        let tags = (5..e.values.len())
            .map(|i| q.word(i))
            .collect::<Result<Vec<_>, _>>()?;
        let features = crate::shape::Features::parse(tags).map_err(|m| q.err(&m))?;
        let base = TypeToken {
            size,
            line_height,
            tracking_em,
            weight: weight as u16,
            font,
            features,
        };
        check_type(&q, &base)?;
        let mut at = [base; 4];
        if let Some(b) = &e.block {
            for c in &b.entries {
                let r = P { e: c };
                let bp = Breakpoint::from_name(&c.key).ok_or_else(|| {
                    r.err("is not a breakpoint (phonePortrait, phoneLandscape, tablet, desktop)")
                })?;
                let t = TypeToken {
                    size: r.num(0)?,
                    line_height: r.num(1)?,
                    ..base
                };
                check_type(&r, &t)?;
                at[bp as usize] = t;
            }
        }
        scale.steps[role.index()] = at;
    }
    let missing: Vec<String> = TypeRole::ALL
        .iter()
        .filter(|r| !seen.contains(r))
        .map(|r| r.name().to_string())
        .collect();
    missing_err(p, &missing, "a style sets every type step")?;
    Ok(scale)
}

fn check_type(q: &P, t: &TypeToken) -> Result<(), String> {
    if !(6.0..=400.0).contains(&t.size) {
        return Err(q.err("takes a size of 6 to 400 CSS px"));
    }
    if t.line_height < t.size * 0.8 {
        return Err(q.err("has a line height under 0.8 of its size: lines would overlap"));
    }
    if !(-0.2..=0.5).contains(&t.tracking_em) {
        return Err(q.err("takes a tracking of -0.2 to 0.5 em"));
    }
    Ok(())
}

/// A block of named lengths (`space`, `size`, `radius`, `stroke`), every
/// name required, none negative.
pub(crate) fn parse_lengths<const N: usize>(
    p: &P,
    names: &[&'static str],
    what: &str,
) -> Result<[f32; N], String> {
    let block = p.block()?;
    let mut out = [f32::NAN; N];
    for e in &block.entries {
        let q = P { e };
        let i = names
            .iter()
            .position(|n| *n == e.key)
            .ok_or_else(|| q.err(&format!("is not a {what} name ({})", names.join(", "))))?;
        if !out[i].is_nan() {
            return Err(q.err("is set twice"));
        }
        let v = q.num(0)?;
        if v < 0.0 {
            return Err(q.err("takes a length of 0 or more"));
        }
        out[i] = v;
    }
    let missing: Vec<String> = names
        .iter()
        .zip(out)
        .filter(|(_, v)| v.is_nan())
        .map(|(n, _)| n.to_string())
        .collect();
    missing_err(p, &missing, &format!("a style sets every {what}"))?;
    Ok(out)
}

pub(crate) fn names_of<T: Copy>(all: &[T], name: fn(T) -> &'static str) -> Vec<&'static str> {
    all.iter().map(|v| name(*v)).collect()
}

/// `shadow <dx> <dy> <blur> <spread> "<colour>"` at `values[at..]`.
pub(crate) fn shadow_at(p: &P, at: usize) -> Result<Shadow, String> {
    Ok(Shadow {
        dx: p.num(at)?,
        dy: p.num(at + 1)?,
        blur: p.num(at + 2)?.max(0.0),
        spread: p.num(at + 3)?,
        color: p.color(at + 4)?,
    })
}

/// `elevation { <level> { shadow … } | <level> none }`.
pub(crate) fn parse_elevation(p: &P) -> Result<[Elevation; Level::COUNT], String> {
    let block = p.block()?;
    let mut out = [None; Level::COUNT];
    for e in &block.entries {
        let q = P { e };
        let l = Level::from_name(&e.key)
            .ok_or_else(|| q.err(&format!("is not an elevation level ({})", Level::names())))?;
        if out[l.index()].is_some() {
            return Err(q.err("is set twice"));
        }
        let mut el = Elevation::FLAT;
        match (&e.block, e.values.first().and_then(Value::as_word)) {
            (None, Some("none")) if e.values.len() == 1 => {}
            (Some(b), None) if e.values.is_empty() => {
                for s in &b.entries {
                    let r = P { e: s };
                    if s.key != "shadow" {
                        return Err(r.err("is not `shadow <dx> <dy> <blur> <spread> \"<colour>\"`"));
                    }
                    if usize::from(el.count) == LEVEL_SHADOWS {
                        return Err(r.err(&format!(
                            "is past the {LEVEL_SHADOWS} shadows a level holds"
                        )));
                    }
                    el.shadows[usize::from(el.count)] = shadow_at(&r, 0)?;
                    el.count += 1;
                }
                if el.count == 0 {
                    return Err(q.err("holds no shadow: write `<level> none` for a flat level"));
                }
            }
            _ => return Err(q.err("takes a { shadow … } block, or `none`")),
        }
        out[l.index()] = Some(el);
    }
    let missing: Vec<String> = Level::ALL
        .iter()
        .filter(|l| out[l.index()].is_none())
        .map(|l| l.name().to_string())
        .collect();
    missing_err(p, &missing, "a style sets every elevation level")?;
    Ok(out.map(|e| e.expect("checked")))
}

/// `shape { <class> <corner> <radius px or radius name> }`.
pub(crate) fn parse_shape(
    p: &P,
    radius: &[f32; RadiusStep::COUNT],
) -> Result<[ShapeToken; ElementClass::COUNT], String> {
    let block = p.block()?;
    let mut out: [Option<ShapeToken>; ElementClass::COUNT] = [None; ElementClass::COUNT];
    for e in &block.entries {
        let q = P { e };
        let c = ElementClass::from_name(&e.key).ok_or_else(|| {
            q.err(&format!(
                "is not an element class ({})",
                ElementClass::names()
            ))
        })?;
        if out[c.index()].is_some() {
            return Err(q.err("is set twice"));
        }
        out[c.index()] = Some(shape_at(&q, 0, radius)?);
    }
    let missing: Vec<String> = ElementClass::ALL
        .iter()
        .filter(|c| out[c.index()].is_none())
        .map(|c| c.name().to_string())
        .collect();
    missing_err(p, &missing, "a style gives every element class its corner")?;
    Ok(out.map(|s| s.expect("checked")))
}

/// `<corner> <radius>` at `values[at..]`: the radius in CSS px or a
/// radius scale name.
pub(crate) fn shape_at(
    q: &P,
    at: usize,
    radius: &[f32; RadiusStep::COUNT],
) -> Result<ShapeToken, String> {
    let w = q.word(at)?;
    let corner = CornerShape::from_name(w).ok_or_else(|| {
        q.err(&format!(
            "names the corner '{w}' ({})",
            CornerShape::names()
        ))
    })?;
    let r = match q.e.values.get(at + 1) {
        Some(Value::Word(n)) => {
            radius[RadiusStep::from_name(n)
                .ok_or_else(|| {
                    q.err(&format!(
                        "names the radius '{n}' ({}, or a number)",
                        RadiusStep::names()
                    ))
                })?
                .index()]
        }
        _ => q.num(at + 1)?,
    };
    if r < 0.0 {
        return Err(q.err("takes a radius of 0 or more"));
    }
    if corner == CornerShape::Square && r != 0.0 {
        return Err(q.err("is square with a radius: write `square 0` or another corner"));
    }
    if q.e.values.len() > at + 2 {
        return Err(q.err("takes a corner and a radius"));
    }
    Ok(ShapeToken { corner, radius: r })
}

/// `<duration ms> <bounce>` at `values[at..]`.
pub(crate) fn spring_at(q: &P, at: usize) -> Result<SpringToken, String> {
    let duration_ms = q.num(at)?;
    let bounce = q.num(at + 1)?;
    if !(16.0..=4000.0).contains(&duration_ms) {
        return Err(q.err("takes a duration of 16 to 4000 ms"));
    }
    if !(-0.75..=0.95).contains(&bounce) {
        return Err(q.err("takes a bounce of -0.75 to 0.95"));
    }
    Ok(SpringToken {
        duration_ms,
        bounce,
    })
}

/// What the v2 `motion` block sets besides the v1 widget motion.
pub(crate) struct MotionParse {
    pub springs: [Option<SpringToken>; SpringName::COUNT],
    pub eases: [Option<Ease>; EaseName::COUNT],
    pub durations: [Option<f32>; 6],
    pub stagger: Option<f32>,
    pub stagger_cap: Option<f32>,
    pub overshoot: Option<f32>,
}

/// The v1 duration names, in the order of [`MotionParse::durations`].
pub(crate) const DURATIONS: [&str; 6] = ["instant", "fast", "base", "slow", "spring", "bounce"];

impl MotionParse {
    pub fn new() -> MotionParse {
        MotionParse {
            springs: [None; SpringName::COUNT],
            eases: [None; EaseName::COUNT],
            durations: [None; 6],
            stagger: None,
            stagger_cap: None,
            overshoot: None,
        }
    }

    /// Reads one token line of the motion block; false for a key it does not own.
    pub fn line(&mut self, q: &P) -> Result<bool, String> {
        let e = q.e;
        match e.key.as_str() {
            "spring" => {
                let w = q.word(0)?;
                let s = SpringName::from_name(w).ok_or_else(|| {
                    q.err(&format!("names the spring '{w}' ({})", SpringName::names()))
                })?;
                self.springs[s.index()] = Some(spring_at(q, 1)?);
            }
            "ease" => {
                let w = q.word(0)?;
                let n = EaseName::from_name(w).ok_or_else(|| {
                    q.err(&format!("names the ease '{w}' ({})", EaseName::names()))
                })?;
                let ease = match e.values.get(1).and_then(Value::as_word) {
                    Some("linear") => Ease::Bezier([0.0, 0.0, 1.0, 1.0]),
                    Some("steps") => {
                        let k = q.num(2)?;
                        if !(1.0..=60.0).contains(&k) || k.fract() != 0.0 {
                            return Err(q.err("takes steps 1 to 60"));
                        }
                        Ease::Steps(k as u8)
                    }
                    Some(w) => {
                        return Err(q.err(&format!(
                            "takes four control points, linear or steps <n>, not '{w}'"
                        )));
                    }
                    None => {
                        let b = [q.num(1)?, q.num(2)?, q.num(3)?, q.num(4)?];
                        if !(0.0..=1.0).contains(&b[0]) || !(0.0..=1.0).contains(&b[2]) {
                            return Err(q.err("takes control points whose x is 0 to 1"));
                        }
                        Ease::Bezier(b)
                    }
                };
                self.eases[n.index()] = Some(ease);
            }
            "duration" => {
                let w = q.word(0)?;
                let i = DURATIONS.iter().position(|d| *d == w).ok_or_else(|| {
                    q.err(&format!(
                        "names the duration '{w}' ({})",
                        DURATIONS.join(", ")
                    ))
                })?;
                self.durations[i] = Some(q.num(1)?.max(0.0));
            }
            "stagger" => self.stagger = Some(q.num(0)?.max(0.0)),
            "staggerCap" => self.stagger_cap = Some(q.num(0)?.max(0.0)),
            "overshoot" => {
                let v = q.num(0)?;
                if !(0.0..=1.0).contains(&v) {
                    return Err(q.err("takes the overshoot allowed, 0 to 1"));
                }
                self.overshoot = Some(v);
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    /// The tokens, every one set, every spring within the overshoot.
    pub fn finish(self, p: &P, extra: &[(&str, SpringToken)]) -> Result<MotionTokens, String> {
        let mut missing: Vec<String> = Vec::new();
        for s in SpringName::ALL {
            if self.springs[s.index()].is_none() {
                missing.push(format!("spring {}", s.name()));
            }
        }
        for n in EaseName::ALL {
            if self.eases[n.index()].is_none() {
                missing.push(format!("ease {}", n.name()));
            }
        }
        for (i, d) in DURATIONS.iter().enumerate() {
            if self.durations[i].is_none() {
                missing.push(format!("duration {d}"));
            }
        }
        for (k, v) in [
            ("stagger", self.stagger),
            ("staggerCap", self.stagger_cap),
            ("overshoot", self.overshoot),
        ] {
            if v.is_none() {
                missing.push(k.into());
            }
        }
        missing_err(p, &missing, "a version 2 style sets every motion token")?;
        let overshoot = self.overshoot.expect("checked");
        let springs = self.springs.map(|s| s.expect("checked"));
        let named = SpringName::ALL
            .iter()
            .map(|s| (s.name(), springs[s.index()]))
            .chain(extra.iter().copied());
        for (name, s) in named {
            if s.overshoot() > overshoot + 1e-3 {
                return Err(p.err(&format!(
                    "has the spring {name} overshoot {:.2} past the style's overshoot {overshoot}: lower its bounce or raise overshoot",
                    s.overshoot()
                )));
            }
        }
        let d = self.durations.map(|d| d.expect("checked"));
        Ok(MotionTokens {
            springs,
            eases: self.eases.map(|e| e.expect("checked")),
            durations: Durations {
                instant: d[0],
                fast: d[1],
                base: d[2],
                slow: d[3],
                spring: d[4],
                bounce: d[5],
            },
            stagger_ms: self.stagger.expect("checked"),
            stagger_cap_ms: self.stagger_cap.expect("checked"),
            overshoot,
        })
    }
}

/// `sounds { <event> none|"<id>"… [volume lo hi] [pitch lo hi] haptic <preset>|none|<pulses> }`:
/// the tokens and each event's sound asset ids.
#[allow(clippy::type_complexity)]
pub(crate) fn parse_sounds(
    p: &P,
) -> Result<
    (
        [Feedback; SoundEvent::COUNT],
        [Vec<String>; SoundEvent::COUNT],
    ),
    String,
> {
    let block = p.block()?;
    let mut out = [Feedback::SILENT; SoundEvent::COUNT];
    let mut ids: [Vec<String>; SoundEvent::COUNT] = std::array::from_fn(|_| Vec::new());
    let mut seen = Vec::new();
    for e in &block.entries {
        let q = P { e };
        let ev = SoundEvent::from_name(&e.key)
            .ok_or_else(|| q.err(&format!("is not a UI event ({})", SoundEvent::names())))?;
        if seen.contains(&ev) {
            return Err(q.err("is set twice"));
        }
        seen.push(ev);
        let mut f = Feedback::SILENT;
        let mut i = 0;
        let v = &e.values;
        if v.first().and_then(Value::as_word) == Some("none") {
            i = 1;
        } else {
            while let Some(s) = v.get(i).and_then(Value::as_str) {
                ids[ev.index()].push(s.to_string());
                i += 1;
            }
            if ids[ev.index()].is_empty() {
                return Err(
                    q.err("starts with its sound (\"<asset id>\", several for a set) or none")
                );
            }
            if ids[ev.index()].len() > SET_SOUNDS {
                return Err(q.err(&format!(
                    "names more than the {SET_SOUNDS} sounds a set holds"
                )));
            }
            f.count = ids[ev.index()].len() as u8;
        }
        let mut haptic = None;
        while i < v.len() {
            match q.word(i)? {
                "volume" | "pitch" => {
                    let k = q.word(i)?;
                    let (lo, hi) = (q.num(i + 1)?, q.num(i + 2)?);
                    let ok = lo <= hi
                        && if k == "volume" {
                            (0.0..=1.0).contains(&lo) && (0.0..=1.0).contains(&hi)
                        } else {
                            (0.25..=4.0).contains(&lo) && (0.25..=4.0).contains(&hi)
                        };
                    if !ok {
                        return Err(q.err(&format!(
                            "takes {k} as a low and a high value ({})",
                            if k == "volume" { "0 to 1" } else { "0.25 to 4" }
                        )));
                    }
                    if k == "volume" {
                        f.volume = [lo, hi];
                    } else {
                        f.pitch = [lo, hi];
                    }
                    i += 3;
                }
                "haptic" => {
                    let (h, used) = haptic_at(&q, i + 1)?;
                    haptic = Some(h);
                    i += 1 + used;
                }
                w => {
                    return Err(q.err(&format!("has '{w}' where volume, pitch or haptic goes")));
                }
            }
        }
        f.haptic = haptic.ok_or_else(|| {
            q.err("leaves out its haptic: `haptic <preset>`, `haptic none` or pulses `haptic <start ms> <ms> <low> <high> …`")
        })?;
        out[ev.index()] = f;
    }
    let missing: Vec<String> = SoundEvent::ALL
        .iter()
        .filter(|e| !seen.contains(e))
        .map(|e| e.name().to_string())
        .collect();
    missing_err(
        p,
        &missing,
        "a style says what every UI event plays (`<event> none haptic none` for nothing)",
    )?;
    Ok((out, ids))
}

/// A haptic at `values[at..]` and how many values it took.
fn haptic_at(q: &P, at: usize) -> Result<(Haptic, usize), String> {
    match q.e.values.get(at) {
        Some(Value::Word(w)) if w == "none" => Ok((Haptic::NONE, 1)),
        Some(Value::Word(w)) => HapticPreset::from_name(w)
            .map(|h| (Haptic::preset(h), 1))
            .ok_or_else(|| {
                q.err(&format!(
                    "names the haptic '{w}' ({}, none, or pulses)",
                    HapticPreset::names()
                ))
            }),
        Some(Value::Number(_)) => {
            let n = q.e.values[at..]
                .iter()
                .take_while(|v| v.as_number().is_some())
                .count();
            if n % 4 != 0 || n / 4 > PULSES {
                return Err(q.err(&format!(
                    "takes up to {PULSES} pulses of four numbers: start ms, ms, low and high strength"
                )));
            }
            let mut h = Haptic::NONE;
            let mut last = 0.0;
            for k in 0..n / 4 {
                let b = at + k * 4;
                let pulse = Pulse {
                    start_ms: q.num(b)?,
                    ms: q.num(b + 1)?,
                    low: q.num(b + 2)?,
                    high: q.num(b + 3)?,
                };
                if pulse.start_ms < last
                    || !(1.0..=1000.0).contains(&pulse.ms)
                    || !(0.0..=1.0).contains(&pulse.low)
                    || !(0.0..=1.0).contains(&pulse.high)
                {
                    return Err(q.err(
                        "takes pulses in order of start, 1 to 1000 ms long, strengths 0 to 1",
                    ));
                }
                last = pulse.start_ms;
                h.pulses[k] = pulse;
            }
            h.count = (n / 4) as u8;
            Ok((h, n))
        }
        _ => Err(q.err("takes a haptic after `haptic`")),
    }
}
