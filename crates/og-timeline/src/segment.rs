//! A timeline as og.film plays it, a segment: one video track without an
//! entity (it plays on the deck's own og.VideoPlayer), and the markers
//! `film.open`, `film.lock` and `film.cut` that time its choice window.
//! og-cook checks a dialogue's segments with it and og-runtime plays them
//! by it, so the two never disagree.

use crate::{Timeline, Track};

/// The marker that shows a film node's choices.
pub const OPEN: &str = "film.open";
/// The marker that ends choosing.
pub const LOCK: &str = "film.lock";
/// The marker that cuts to the next segment (else the end).
pub const CUT: &str = "film.cut";

/// What og.film reads of a segment.
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    /// The first key of its video track: the video's asset id and its `from`.
    pub video: Option<(String, f32)>,
    /// When the window opens (`film.open`, else 0), locks (`film.lock`,
    /// else the cut) and the segment cuts (`film.cut`, else its duration;
    /// never past it), in seconds.
    pub open: f32,
    pub lock: f32,
    pub cut: f32,
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
        Ok(Segment {
            video: own[0].keys.first().map(|k| (k.video.clone(), k.from)),
            open,
            lock,
            cut,
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
            "{VIDEO}track marker {{\n  key 3 {{\n    name \"film.open\"\n  }}\n  key 7 {{\n    name \"film.lock\"\n  }}\n  key 8 {{\n    name \"film.cut\"\n  }}\n}}\n"
        ))
        .segment()
        .unwrap();
        assert_eq!((s.open, s.lock, s.cut), (3.0, 7.0, 8.0));
        assert_eq!(s.video, Some(("clips/a.ogt".into(), 4.0)));
        // No markers: open at 0, locked and cut at the end.
        let s = parse(VIDEO).segment().unwrap();
        assert_eq!((s.open, s.lock, s.cut), (0.0, 10.0, 10.0));
        // A lock before the open, a cut past the end clamped below a lock.
        let late = "track marker {\n  key 6 {\n    name \"film.open\"\n  }\n  key 5 {\n    name \"film.lock\"\n  }\n}\n";
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
}
