//! OKLCH colour conversion, WCAG contrast calculation, and foreground ink
//! solving for compiled themes.

use super::*;

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

pub(super) fn at(c: Lch, l: f64, chroma: f64) -> String {
    hex(Lch {
        l: l as f32,
        c: chroma as f32,
        h: c.h,
    })
}

pub(super) fn num(v: &Value, at: &str) -> Result<f64, String> {
    v.as_f64().ok_or_else(|| format!("{at}: a number"))
}

pub(super) fn pair(v: &Value, at: &str) -> Result<[f64; 2], String> {
    match v.as_array().map(Vec::as_slice) {
        Some([a, b]) => Ok([num(a, at)?, num(b, at)?]),
        _ => Err(format!("{at}: [top, bottom]")),
    }
}

/// The WCAG relative luminance of `#rrggbb`.
pub(super) fn luminance(hex: &str) -> f32 {
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
pub(super) fn seed_css(pack: &Map<String, Value>) -> String {
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
pub(super) fn solve_inks(
    seeds: &str,
    art: &[Vec<String>; 2],
    colors: &mut Map<String, Value>,
    more: bool,
) {
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
        // High contrast (`contrast: more`) holds body inks at 7:1 and the
        // large-figure ones at 4.5:1.
        let ratio = match (more, ratio >= 4.5) {
            (false, _) => ratio,
            (true, true) => 7.0,
            (true, false) => 4.5,
        };
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

/// The high-contrast inks (`solve_inks` with `more`) as the pack's
/// `contrast` layer, over any it had.
pub(super) fn set_more(pack: &mut Map<String, Value>, more: Map<String, Value>) {
    if more.is_empty() {
        return;
    }
    let c = pack
        .entry("contrast")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .and_then(|c| {
            c.entry("colors")
                .or_insert_with(|| json!({}))
                .as_object_mut()
        });
    if let Some(c) = c {
        c.extend(more);
    }
}

/// Each `<role>-fg` the seeds give, as the ink on its own fill
/// (`text-accent-fg` on `bg-accent`): held at 4.5:1 there, moved in OKLCH
/// lightness away from the fill (the other way when that end cannot reach
/// it). A pair that already holds in both schemes is not written.
pub(super) fn solve_pairs(seeds: &str, colors: &mut Map<String, Value>) {
    let design = crate::pack::design::colors(seeds);
    let pairs: Vec<String> = design[1]
        .keys()
        .filter(|k| {
            k.strip_suffix("-fg")
                .is_some_and(|g| design[1].contains_key(g))
        })
        .cloned()
        .collect();
    for role in pairs {
        let ground = &role[..role.len() - 3];
        let mut pair = Map::new();
        let mut moved = false;
        for (i, scheme) in ["light", "dark"].into_iter().enumerate() {
            // A role the ink solve already moved is read as it left it.
            let get = |k: &str| match colors.get(k) {
                Some(Value::Object(p)) => p.get(scheme).and_then(Value::as_str),
                Some(v) => v.as_str(),
                None => design[i].get(k).and_then(Value::as_str),
            };
            let (Some(have), Some(fill)) = (get(&role), get(ground)) else {
                continue;
            };
            let (have, fill) = (have.to_owned(), fill.to_owned());
            let (have, fill) = (&have[..7.min(have.len())], &fill[..7.min(fill.len())]);
            let Ok(start) = lch(have) else { continue };
            let mut best = (have.to_owned(), contrast(have, fill));
            if best.1 < 4.5 {
                let away = luminance(fill) < 0.18;
                for up in [away, !away] {
                    let mut c = start;
                    while if up { c.l < 1.0 } else { c.l > 0.0 } {
                        c.l = (c.l + if up { 0.005 } else { -0.005 }).clamp(0.0, 1.0);
                        let w = contrast(&hex(c), fill);
                        if w > best.1 {
                            best = (hex(c), w);
                        }
                        if w >= 4.5 {
                            break;
                        }
                    }
                    if best.1 >= 4.5 {
                        break;
                    }
                }
                moved = true;
            }
            pair.insert(scheme.into(), json!(best.0));
        }
        if moved && pair.len() == 2 {
            colors.insert(role, Value::Object(pair));
        }
    }
}
