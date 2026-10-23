# og-timeline

Paths and the sequencer as data (`docs/design.md` §2.7, "Timeline and
paths"; `docs/depth.md` §2). No ECS: og-runtime applies it to a world
(`og.path`, `og.timeline`), og-cook cooks `type timeline` files into blobs.

| Item | What it is |
|---|---|
| `Path::new(PathDesc { points, closed, kind, tension })` | `catmullRom` (three.js's `CatmullRomCurve3`, `tension` 0.5), `bezier` (`p0 c0 c1 p1 ...`: 3n + 1 points open, 3n closed) or `linear`; the arc length tabulated at `SAMPLES` chords a segment |
| `Path::sample(t)`, `sample_distance(d)`, `length()`, `nearest(p)` | position, unit tangent and up (world +Y made perpendicular) at the arc-length parameter 0..1 (closed paths wrap, open ones clamp); the nearest point with its parameter and distance |
| `Timeline::parse(text, read)`, `to_text()` | the asset: `name`, `duration`, `path "<name>" { ... }`, `track <kind> ["<entity>"] { key <s> { ... } }`; `read` loads the caption files a caption track names; `to_text` is the canonical text the cook stores (captions imported, rotations as quaternions) |
| `eval::{camera_at, transform_at, clip_at, caption_at, cover_at, music_gain, crossed}` | every value a track sets as a function of the time alone |
| `captions::{parse_srt, group_words, parse_word_json}` | SubRip cues; word timestamps grouped into lines (`import` feature: the JSON reader) |

## Invariants

- **Seeking is exact**: a key's value at `t` depends on `t` and the keys,
  never on the ticks before, so a seek to `t` and playing to `t` evaluate
  the same. Between two keys that set a channel the later key's `ease`
  (og-tween's curves, linear by default) mixes them; before the first and
  after the last the value is held.
- **Deterministic arithmetic**: `+ − × ÷`, `sqrt` and og-core's `sin`,
  `cos` and `atan2`; quaternions slerp through them.
- **Canonical text**: `to_text` then `parse` gives the same `Timeline`, and
  `to_text` of that the same bytes (`model::tests`).

## Tests

`cargo test -p og-timeline --features import`: arc length against known
lengths, Catmull-Rom through its points at even speed, Bezier anchors and
closed wrap, nearest; channels held and eased by the later key, slerp,
covers through, in, out and cut, music fades; the round trip, caption
import, and refusals that name the line.
