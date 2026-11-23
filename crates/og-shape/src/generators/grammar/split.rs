//! How a split shares a face's length among its parts.

use super::Size;

/// The most pieces one split may make.
const MAX_PIECES: f64 = 100_000.0;

/// The pieces a split of `total` makes of `parts` (each its kind and its
/// length, above 0): each piece's part and its start and end, from 0.
///
/// - With a `fill` or `repeat` part and room left after the exact parts,
///   the pieces fill `total`: every `repeat` part comes as many times as
///   whole copies of the repeated lengths fit in what the exact and fill
///   parts leave (at least once), then the fill and repeat pieces stretch
///   by one factor to fill it. The last piece ends at `total` exactly.
/// - Otherwise the exact parts are laid in order from 0 and cut at
///   `total`; what they do not reach is not made (CGA's split).
pub(super) fn pieces(parts: &[(Size, f64)], total: f64) -> Result<Vec<(usize, f64, f64)>, String> {
    let sum = |kind: Size| -> f64 { parts.iter().filter(|p| p.0 == kind).map(|p| p.1).sum() };
    let exact = sum(Size::Abs);
    let flexible = parts.iter().any(|p| p.0 != Size::Abs);
    let room = total - exact;
    let mut out = Vec::new();
    if flexible && room > 0.0 {
        let (fill, repeated) = (sum(Size::Fill), sum(Size::Repeat));
        let copies = if repeated > 0.0 {
            ((room - fill).max(0.0) / repeated).floor().max(1.0)
        } else {
            0.0
        };
        if copies * parts.len() as f64 > MAX_PIECES {
            return Err(format!(
                "a split of {total} m would make {copies} copies of its repeated parts"
            ));
        }
        let stretch = room / (fill + copies * repeated);
        let mut lengths = Vec::new();
        for (i, &(kind, length)) in parts.iter().enumerate() {
            match kind {
                Size::Abs => lengths.push((i, length)),
                Size::Fill => lengths.push((i, length * stretch)),
                Size::Repeat => {
                    lengths.extend((0..copies as usize).map(|_| (i, length * stretch)));
                }
            }
        }
        let mut start = 0.0;
        let last = lengths.len().saturating_sub(1);
        for (k, (i, length)) in lengths.into_iter().enumerate() {
            let end = if k == last { total } else { start + length };
            out.push((i, start, end));
            start = end;
        }
    } else {
        let mut start = 0.0;
        for (i, &(kind, length)) in parts.iter().enumerate() {
            if kind != Size::Abs {
                continue;
            }
            let end = (start + length).min(total);
            if end > start {
                out.push((i, start, end));
            }
            start = end;
            if start >= total {
                break;
            }
        }
    }
    Ok(out)
}
