//! A timeline as og.flow plays it, a segment: footage (at most one video
//! track without an entity, which plays on the deck's own og.VideoPlayer),
//! a live scene (a `scene` track), or both; the markers `flow.open`,
//! `flow.lock` and `flow.cut` that time its choice window, `flow.loopStart`
//! and `flow.loopEnd` for a seamless idle choice loop, and `flow.hold`,
//! where a playable segment stops for the game (og.flow.advance). og-cook
//! checks a dialogue's segments and choices with it and og-runtime plays
//! them by it, so the two never disagree.

use crate::{HotspotKey, Timeline, Track};

/// The marker that shows a flow node's choices.
pub const OPEN: &str = "flow.open";
/// The marker that ends choosing.
pub const LOCK: &str = "flow.lock";
/// The marker that cuts to the next segment (else the end).
pub const CUT: &str = "flow.cut";
/// The marker where a playable segment stops until the game advances it.
pub const HOLD: &str = "flow.hold";
/// Start of an idle choice loop (`flow.loopStart` .. `flow.loopEnd`).
pub const LOOP_START: &str = "flow.loopStart";
/// End of an idle choice loop (`flow.loopStart` .. `flow.loopEnd`).
pub const LOOP_END: &str = "flow.loopEnd";

/// What og.flow reads of a segment.
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    /// The first key of its own video track: the video's asset id and its
    /// `from`; `None` for a live segment (a `scene` track alone).
    pub video: Option<(String, f32)>,
    /// Whether this segment has a `scene` track (live 3D/2D scene).
    pub live: bool,
    /// When the window opens (`flow.open`, else 0), locks (`flow.lock`,
    /// else the cut) and the segment cuts (`flow.cut`, else its duration;
    /// never past it), in seconds. A held segment's lock and cut are its
    /// hold: it never locks or cuts by time.
    pub open: f32,
    pub lock: f32,
    pub cut: f32,
    /// `flow.hold`: where it stops until og.flow.advance (or a choice).
    pub hold: Option<f32>,
    /// `flow.loopStart` .. `flow.loopEnd`: when present, the segment loops
    /// smoothly inside `[start, end]` while waiting for a choice or advance.
    pub loop_region: Option<(f32, f32)>,
    /// Every hotspot track's keys, by time (`eval::hotspots_at`).
    pub hotspots: Vec<HotspotKey>,
    /// Its footage plays under the scene (`layer under`), as the camera's
    /// background texture, not over it.
    pub under: bool,
}

/// A quick-time event: a choice taken by input in the window
/// (`#qte:press:<action>`, `#qte:hold:<action>:<ms>`,
/// `#qte:mash:<action>:<presses>`, `#qte:timing:<action>:<start>-<end>`,
/// `#qte:sequence:<a1>+<a2>+...`, `#qte:swipe:<dir>`, `#qte:drag:<dir>`).
#[derive(Clone, Debug, PartialEq)]
pub struct Qte {
    pub kind: QteKind,
    /// Primary og.input action (first action for `sequence`; empty for `swipe`/`drag`).
    pub action: String,
    /// Sequence actions for `QteKind::Sequence`.
    pub sequence: Vec<String>,
    /// Direction (`up`, `down`, `left`, `right`) for `Swipe` / `Drag`.
    pub direction: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QteKind {
    /// One press.
    Press,
    /// Held down this long without letting go, ms.
    Hold(f32),
    /// Pressed this many times.
    Mash(u32),
    /// Press while the window progress lies inside `[zone_start, zone_end]` (0..1).
    Timing(f32, f32),
    /// Press a sequence of actions in order.
    Sequence,
    /// Touch/pointer swipe in a cardinal direction (`up`, `down`, `left`, `right`).
    Swipe,
    /// Touch/pointer drag in a cardinal direction until progress reaches 1.
    Drag,
}

impl QteKind {
    pub fn name(self) -> &'static str {
        match self {
            QteKind::Press => "press",
            QteKind::Hold(_) => "hold",
            QteKind::Mash(_) => "mash",
            QteKind::Timing(_, _) => "timing",
            QteKind::Sequence => "sequence",
            QteKind::Swipe => "swipe",
            QteKind::Drag => "drag",
        }
    }
}

/// The most choices of a node a quick-time event may take (og.Flow.qte
/// keeps their progress).
pub const QTE_CHOICES: usize = 8;

/// A node's `og-flow-dissolve` header: the seconds its segment dissolves
/// in over the one before, more than 0 and at most 5.
pub fn dissolve(header: &str) -> Result<f32, String> {
    match header.trim().parse::<f32>() {
        Ok(s) if s > 0.0 && s <= 5.0 => Ok(s),
        _ => Err(format!(
            "og-flow-dissolve: {header} is not a dissolve's seconds, more than 0 and at most 5 (`og-flow-dissolve: 0.5`)"
        )),
    }
}

/// Parses an `og-flow-music` header: `<sound>` (or `none` to stop) with
/// optional `volume=<v> fade=<s_sec> duck=<d>` key=value pairs.
#[derive(Clone, Debug, PartialEq)]
pub struct MusicHeader {
    pub sound: String,
    pub volume: f32,
    pub fade: f32,
    pub duck: f32,
}

pub fn music_header(header: &str) -> Result<MusicHeader, String> {
    let mut parts = header.split_whitespace();
    let Some(first) = parts.next() else {
        return Err("og-flow-music: name a sound asset id or `none`".into());
    };
    let sound = if first == "none" {
        String::new()
    } else {
        first.to_string()
    };
    let mut volume = 1.0f32;
    let mut fade = 0.5f32;
    let mut duck = 0.4f32;
    for tok in parts {
        let Some((k, v)) = tok.split_once('=') else {
            return Err(format!(
                "og-flow-music: unexpected token '{tok}' (use volume=1 fade=0.5 duck=0.4)"
            ));
        };
        let n: f32 = v
            .parse()
            .map_err(|_| format!("og-flow-music: {k}='{v}' is not a number"))?;
        if !n.is_finite() {
            return Err(format!("og-flow-music: {k} must be finite"));
        }
        match k {
            "volume" if (0.0..=2.0).contains(&n) => volume = n,
            "fade" if n >= 0.0 => fade = n,
            "duck" if (0.0..=1.0).contains(&n) => duck = n,
            _ => {
                return Err(format!(
                    "og-flow-music: invalid setting {k}={v} (volume 0..2, fade >=0, duck 0..1)"
                ));
            }
        }
    }
    Ok(MusicHeader {
        sound,
        volume,
        fade,
        duck,
    })
}

fn valid_dir(d: &str) -> bool {
    matches!(d, "up" | "down" | "left" | "right")
}

impl Qte {
    /// A `#qte` tag's value (what follows `qte:`).
    pub fn parse(tag: &str) -> Result<Qte, String> {
        const FORMS: &str = "#qte:press:<action>, #qte:hold:<action>:<ms>, #qte:mash:<action>:<presses>, #qte:timing:<action>:<start>-<end>, #qte:sequence:<a1>+<a2>+..., #qte:swipe:<up|down|left|right>, or #qte:drag:<up|down|left|right>";
        let parts: Vec<&str> = tag.split(':').collect();
        match parts.first().copied() {
            Some("press") if parts.len() == 2 && !parts[1].is_empty() => Ok(Qte {
                kind: QteKind::Press,
                action: parts[1].to_string(),
                sequence: Vec::new(),
                direction: String::new(),
            }),
            Some("hold") if parts.len() == 3 && !parts[1].is_empty() => {
                let ms = parts[2]
                    .parse::<f32>()
                    .ok()
                    .filter(|m| *m > 0.0 && m.is_finite())
                    .ok_or_else(|| format!("#qte:{tag} is not one of {FORMS}"))?;
                Ok(Qte {
                    kind: QteKind::Hold(ms),
                    action: parts[1].to_string(),
                    sequence: Vec::new(),
                    direction: String::new(),
                })
            }
            Some("mash") if parts.len() == 3 && !parts[1].is_empty() => {
                let n = parts[2]
                    .parse::<f32>()
                    .ok()
                    .filter(|n| *n >= 1.0 && n.fract() == 0.0 && *n <= 1000.0)
                    .ok_or_else(|| format!("#qte:{tag} is not one of {FORMS}"))?;
                Ok(Qte {
                    kind: QteKind::Mash(n as u32),
                    action: parts[1].to_string(),
                    sequence: Vec::new(),
                    direction: String::new(),
                })
            }
            Some("timing") if parts.len() == 3 && !parts[1].is_empty() => {
                let (a, b) = parts[2]
                    .split_once('-')
                    .ok_or_else(|| format!("#qte:{tag}: timing needs <start>-<end> (0..1)"))?;
                let (start, end) = match (a.parse::<f32>(), b.parse::<f32>()) {
                    (Ok(s), Ok(e))
                        if s.is_finite() && e.is_finite() && s >= 0.0 && e <= 1.0 && s < e =>
                    {
                        (s, e)
                    }
                    _ => {
                        return Err(format!(
                            "#qte:{tag}: timing zone must satisfy 0 <= start < end <= 1"
                        ));
                    }
                };
                Ok(Qte {
                    kind: QteKind::Timing(start, end),
                    action: parts[1].to_string(),
                    sequence: Vec::new(),
                    direction: String::new(),
                })
            }
            Some("sequence") if parts.len() == 2 && !parts[1].is_empty() => {
                let seq: Vec<String> = parts[1]
                    .split('+')
                    .filter(|a| !a.is_empty())
                    .map(str::to_string)
                    .collect();
                if seq.len() < 2 || seq.len() > 16 {
                    return Err(format!(
                        "#qte:{tag}: sequence needs 2 to 16 `+`-separated actions"
                    ));
                }
                let first = seq[0].clone();
                Ok(Qte {
                    kind: QteKind::Sequence,
                    action: first,
                    sequence: seq,
                    direction: String::new(),
                })
            }
            Some("swipe") if parts.len() == 2 && valid_dir(parts[1]) => Ok(Qte {
                kind: QteKind::Swipe,
                action: String::new(),
                sequence: Vec::new(),
                direction: parts[1].to_string(),
            }),
            Some("drag") if parts.len() == 2 && valid_dir(parts[1]) => Ok(Qte {
                kind: QteKind::Drag,
                action: String::new(),
                sequence: Vec::new(),
                direction: parts[1].to_string(),
            }),
            _ => Err(format!("#qte:{tag} is not one of {FORMS}")),
        }
    }
}

/// A node's `#outcome:<name>` choices, by choice index (`None` for a
/// choice the player sees), from each choice's index and tags: hidden from
/// the player and taken by og.flow.advance. Refused when its segment does
/// not hold (`held`: it has `flow.hold`), for a repeated or empty name, and
/// for an outcome that is also `#default`, `#qte`, `#hotspot` or `#immediate`.
pub fn outcomes<'a>(
    held: bool,
    choices: impl IntoIterator<Item = (u32, &'a std::collections::BTreeMap<String, String>)>,
) -> Result<Vec<Option<String>>, String> {
    let mut out: Vec<Option<String>> = Vec::new();
    for (index, tags) in choices {
        let Some(name) = tags.get("outcome") else {
            out.push(None);
            continue;
        };
        if name.is_empty() || name.contains(char::is_whitespace) {
            return Err(format!(
                "choice {index}: #outcome names its result in one word (`#outcome:won`)"
            ));
        }
        if !held {
            return Err(format!(
                "choice {index} is #outcome:{name}, but its segment has no `{HOLD}` marker: an outcome ends a playable segment, which holds there until the game calls og.flow.advance"
            ));
        }
        if let Some(t) = ["default", "qte", "hotspot", "immediate"]
            .into_iter()
            .find(|t| tags.contains_key(*t))
        {
            return Err(format!(
                "choice {index} is #outcome:{name} and #{t}: the player never sees an outcome, the game takes it"
            ));
        }
        if out.iter().flatten().any(|o| o == name) {
            return Err(format!(
                "two choices are #outcome:{name}; each outcome names one choice"
            ));
        }
        out.push(Some(name.clone()));
    }
    Ok(out)
}

impl Timeline {
    /// This timeline as a segment; refused unless it has at most one video
    /// track without an entity and that or a `scene` track, and
    /// `open ≤ lock ≤ cut` (a held one: `open ≤ hold`, without `flow.lock`
    /// or `flow.cut`; a looping one: `flow.loopStart < flow.loopEnd <= cut`).
    pub fn segment(&self) -> Result<Segment, String> {
        let own: Vec<_> = self
            .tracks
            .iter()
            .filter_map(|t| match t {
                Track::Video(v) if v.entity.is_empty() => Some(v),
                _ => None,
            })
            .collect();
        if own.len() > 1 {
            return Err(format!(
                "it has {} video tracks without an entity; a segment has at most one, which plays on its deck (give the others an entity: `track video \"<screen>\"`)",
                own.len()
            ));
        }
        let live = self.tracks.iter().any(|t| matches!(t, Track::Scene(_)));
        if own.is_empty() && !live {
            return Err("it has neither footage nor a scene: a segment has a video track without an entity (`track video { key 0 { video \"clips/<clip>.ogt\" } }`), a scene track (`track scene { key 0 { scene \"scenes/<scene>.ogt\" } }`), or both".into());
        }
        let marker = |name: &str| {
            self.tracks.iter().find_map(|t| match t {
                Track::Marker(keys) => keys.iter().find(|k| k.name == name).map(|k| k.time),
                _ => None,
            })
        };
        let open = marker(OPEN).unwrap_or(0.0);
        let hold = marker(HOLD);
        let loop_start = marker(LOOP_START);
        let loop_end = marker(LOOP_END);
        let loop_region = match (loop_start, loop_end) {
            (Some(a), Some(b)) => {
                if hold.is_some() {
                    return Err(format!(
                        "it has both {HOLD} and {LOOP_START}/{LOOP_END}: use one or the other"
                    ));
                }
                if !(a >= 0.0 && a < b && b <= self.duration) {
                    return Err(format!(
                        "its loop region {LOOP_START} ({a} s) .. {LOOP_END} ({b} s) must satisfy 0 ≤ loopStart < loopEnd ≤ duration ({} s)",
                        self.duration
                    ));
                }
                Some((a, b))
            }
            (Some(_), None) | (None, Some(_)) => {
                return Err(format!(
                    "a looping segment needs both `{LOOP_START}` and `{LOOP_END}` markers"
                ));
            }
            (None, None) => None,
        };
        if let Some(h) = hold {
            if let Some(m) = [LOCK, CUT].into_iter().find(|m| marker(m).is_some()) {
                return Err(format!(
                    "it has {HOLD} and {m}: a held segment's window stays open while it holds and it cuts when the game calls og.flow.advance, never by time"
                ));
            }
            if open > h {
                return Err(format!(
                    "its {OPEN} ({open} s) is past its {HOLD} ({h} s): the window opens before it holds"
                ));
            }
        }
        let cut = hold.unwrap_or_else(|| marker(CUT).unwrap_or(self.duration).min(self.duration));
        let lock = hold.unwrap_or_else(|| marker(LOCK).unwrap_or(cut));
        if !(open <= lock && lock <= cut) {
            return Err(format!(
                "its markers are out of order: {OPEN} {open} s, {LOCK} {lock} s, cut {cut} s (open ≤ lock ≤ cut)"
            ));
        }
        if let Some((a, b)) = loop_region
            && (open > b || b > cut)
        {
            return Err(format!(
                "its loop region ({a}..{b} s) must open before {LOOP_END} and end at or before cut ({cut} s)"
            ));
        }
        let mut hotspots: Vec<HotspotKey> = self
            .tracks
            .iter()
            .filter_map(|t| match t {
                Track::Hotspot(keys) => Some(keys.iter().cloned()),
                _ => None,
            })
            .flatten()
            .collect();
        og_core::sort::sort_by(&mut hotspots, |a, b| a.time.total_cmp(&b.time));
        Ok(Segment {
            video: own
                .first()
                .and_then(|v| v.keys.first())
                .map(|k| (k.video.clone(), k.from)),
            live,
            open,
            lock,
            cut,
            hold,
            loop_region,
            hotspots,
            under: own
                .first()
                .is_some_and(|v| v.layer == crate::VideoLayer::Under),
        })
    }
}
