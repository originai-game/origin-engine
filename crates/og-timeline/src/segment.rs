//! A timeline as og.flow plays it, a segment: one video track without an
//! entity (it plays on the deck's own og.VideoPlayer), and the markers
//! `flow.open`, `flow.lock` and `flow.cut` that time its choice window.
//! og-cook checks a dialogue's segments with it and og-runtime plays them
//! by it, so the two never disagree; so are a choice's `#qte` tags.

use crate::{HotspotKey, Timeline, Track};

/// The marker that shows a flow node's choices.
pub const OPEN: &str = "flow.open";
/// The marker that ends choosing.
pub const LOCK: &str = "flow.lock";
/// The marker that cuts to the next segment (else the end).
pub const CUT: &str = "flow.cut";

/// What og.flow reads of a segment.
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    /// The first key of its video track: the video's asset id and its `from`.
    pub video: Option<(String, f32)>,
    /// When the window opens (`flow.open`, else 0), locks (`flow.lock`,
    /// else the cut) and the segment cuts (`flow.cut`, else its duration;
    /// never past it), in seconds.
    pub open: f32,
    pub lock: f32,
    pub cut: f32,
    /// Every hotspot track's keys, by time (`eval::hotspots_at`).
    pub hotspots: Vec<HotspotKey>,
}

/// A quick-time event: a choice taken by an action's input in the window
/// (`#qte:press:<action>`, `#qte:hold:<action>:<ms>`,
/// `#qte:mash:<action>:<presses>`).
#[derive(Clone, Debug, PartialEq)]
pub struct Qte {
    pub kind: QteKind,
    /// The og.input action it reads.
    pub action: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QteKind {
    /// One press.
    Press,
    /// Held down this long without letting go, ms.
    Hold(f32),
    /// Pressed this many times.
    Mash(u32),
}

impl QteKind {
    pub fn name(self) -> &'static str {
        match self {
            QteKind::Press => "press",
            QteKind::Hold(_) => "hold",
            QteKind::Mash(_) => "mash",
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

impl Qte {
    /// A `#qte` tag's value (what follows `qte:`).
    pub fn parse(tag: &str) -> Result<Qte, String> {
        const FORMS: &str =
            "#qte:press:<action>, #qte:hold:<action>:<ms> or #qte:mash:<action>:<presses>";
        let parts: Vec<&str> = tag.split(':').collect();
        let action = parts
            .get(1)
            .filter(|a| !a.is_empty())
            .map(|a| a.to_string());
        let amount = parts.get(2).and_then(|n| n.parse::<f32>().ok());
        let kind = match (parts.first().copied(), parts.len(), amount) {
            (Some("press"), 2, _) => QteKind::Press,
            (Some("hold"), 3, Some(ms)) if ms > 0.0 && ms.is_finite() => QteKind::Hold(ms),
            (Some("mash"), 3, Some(n)) if n >= 1.0 && n.fract() == 0.0 && n <= 1000.0 => {
                QteKind::Mash(n as u32)
            }
            _ => return Err(format!("#qte:{tag} is not one of {FORMS}")),
        };
        let action = action.ok_or_else(|| format!("#qte:{tag} names no action: {FORMS}"))?;
        Ok(Qte { kind, action })
    }
}

impl Timeline {
    /// This timeline as a segment; refused unless it has exactly one video
    /// track without an entity and `open ≤ lock ≤ cut`.
    pub fn segment(&self) -> Result<Segment, String> {
        let own: Vec<_> = self
            .tracks
            .iter()
            .filter_map(|t| match t {
                Track::Video(v) if v.entity.is_empty() => Some(v),
                _ => None,
            })
            .collect();
        if own.len() != 1 {
            return Err(format!(
                "it has {} video tracks without an entity; a segment has exactly one (`track video {{ key 0 {{ video \"clips/<clip>.ogt\" }} }}`), which plays on its deck",
                own.len()
            ));
        }
        let marker = |name: &str| {
            self.tracks.iter().find_map(|t| match t {
                Track::Marker(keys) => keys.iter().find(|k| k.name == name).map(|k| k.time),
                _ => None,
            })
        };
        let cut = marker(CUT).unwrap_or(self.duration).min(self.duration);
        let open = marker(OPEN).unwrap_or(0.0);
        let lock = marker(LOCK).unwrap_or(cut);
        if !(open <= lock && lock <= cut) {
            return Err(format!(
                "its markers are out of order: {OPEN} {open} s, {LOCK} {lock} s, cut {cut} s (open ≤ lock ≤ cut)"
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
            video: own[0].keys.first().map(|k| (k.video.clone(), k.from)),
            open,
            lock,
            cut,
            hotspots,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(body: &str) -> Timeline {
        let text = format!("type timeline\nversion 1\n---\nduration 10\n{body}");
        Timeline::parse(&text, &mut |f: &str| Err(f.to_string())).unwrap()
    }

    const VIDEO: &str = "track video {\n  key 0 {\n    video \"clips/a.ogt\"\n    from 4\n  }\n}\n";

    #[test]
    fn the_markers_time_the_window_and_their_absence_falls_back_to_the_ends() {
        let s = parse(&format!(
            "{VIDEO}track marker {{\n  key 3 {{\n    name \"flow.open\"\n  }}\n  key 7 {{\n    name \"flow.lock\"\n  }}\n  key 8 {{\n    name \"flow.cut\"\n  }}\n}}\n"
        ))
        .segment()
        .unwrap();
        assert_eq!((s.open, s.lock, s.cut), (3.0, 7.0, 8.0));
        assert_eq!(s.video, Some(("clips/a.ogt".into(), 4.0)));
        // No markers: open at 0, locked and cut at the end.
        let s = parse(VIDEO).segment().unwrap();
        assert_eq!((s.open, s.lock, s.cut), (0.0, 10.0, 10.0));
        // A lock before the open, a cut past the end clamped below a lock.
        let late = "track marker {\n  key 6 {\n    name \"flow.open\"\n  }\n  key 5 {\n    name \"flow.lock\"\n  }\n}\n";
        assert!(
            parse(&format!("{VIDEO}{late}"))
                .segment()
                .unwrap_err()
                .contains("out of order")
        );
        // A video track on a named entity is not the segment's own.
        let named = "track video \"tv\" {\n  key 0 {\n    video \"clips/a.ogt\"\n  }\n}\n";
        assert!(
            parse(named)
                .segment()
                .unwrap_err()
                .contains("0 video tracks")
        );
        assert!(
            parse(&format!("{VIDEO}{VIDEO}"))
                .segment()
                .unwrap_err()
                .contains("2 video tracks")
        );
    }

    #[test]
    fn qte_tags_read_their_kind_action_and_amount_and_refuse_the_rest() {
        let q = |t: &str| Qte::parse(t).map(|q| (q.kind, q.action));
        assert_eq!(q("press:jump"), Ok((QteKind::Press, "jump".into())));
        assert_eq!(
            q("hold:grip:800"),
            Ok((QteKind::Hold(800.0), "grip".into()))
        );
        assert_eq!(q("mash:hit:12"), Ok((QteKind::Mash(12), "hit".into())));
        for bad in [
            "",
            "press",
            "press:",
            "hold:grip",
            "hold:grip:0",
            "mash:hit:2.5",
            "mash:hit:0",
            "tap:x",
            "press:jump:3",
        ] {
            assert!(Qte::parse(bad).is_err(), "{bad}");
        }
    }
}
