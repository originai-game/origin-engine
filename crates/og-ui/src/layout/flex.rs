use super::*;
use crate::element::Position;

mod height;
mod margins;
mod position;
mod width;

pub(super) use height::*;
pub(super) use margins::*;
pub(crate) use position::on_screen;
use position::out_size;
pub(super) use position::{cross_offset, inline_rows, position};
pub(super) use width::*;

/// Raises `items` (by index into `w`) towards a common level so their sum
/// grows by `extra`, each capped at its own max: the smallest grow first.
fn fill_up(w: &mut [f32], idx: &[usize], max: &[f32], extra: f32) {
    if extra <= 0.0 || idx.is_empty() {
        return;
    }
    let base: Vec<f32> = idx.iter().map(|&i| w[i]).collect();
    let cap: Vec<f32> = idx
        .iter()
        .zip(&base)
        .map(|(&i, &b)| max[i].max(b))
        .collect();
    let f = |level: f32| -> f32 {
        base.iter()
            .zip(&cap)
            .map(|(&b, &c)| level.clamp(b, c) - b)
            .sum()
    };
    let mut points: Vec<f32> = base
        .iter()
        .chain(cap.iter())
        .copied()
        .filter(|v| v.is_finite())
        .collect();
    og_core::sort::sort_by(&mut points, f32::total_cmp);
    points.dedup();
    let mut lo = base.iter().copied().fold(f32::INFINITY, f32::min);
    let mut level = f32::INFINITY;
    let mut solved = false;
    for &p in &points {
        if p <= lo {
            continue;
        }
        if f(p) >= extra {
            let active = base
                .iter()
                .zip(&cap)
                .filter(|&(&b, &c)| b <= lo && c > lo)
                .count();
            level = lo + (extra - f(lo)) / active.max(1) as f32;
            solved = true;
            break;
        }
        lo = p;
    }
    if !solved {
        let active = base
            .iter()
            .zip(&cap)
            .filter(|&(&b, &c)| b <= lo && c > lo)
            .count();
        if active > 0 {
            level = lo + (extra - f(lo)) / active as f32;
        }
    }
    for (k, &i) in idx.iter().enumerate() {
        w[i] = level.clamp(base[k], cap[k]);
    }
}

/// Lowers `items` towards a common level so their sum shrinks by `deficit`,
/// none below its min: the widest shrink first.
fn fill_down(w: &mut [f32], idx: &[usize], min: &[f32], deficit: f32) {
    let mut neg: Vec<f32> = w.iter().map(|v| -v).collect();
    let neg_cap: Vec<f32> = min.iter().map(|v| -v).collect();
    fill_up(&mut neg, idx, &neg_cap, deficit);
    for &i in idx {
        w[i] = -neg[i];
    }
}

/// Grow and shrink one line of children along the main axis.
fn flex(sizes: &mut [f32], grow_max: &[Option<f32>], shrink_min: &[Option<f32>], avail: f32) {
    let total: f32 = sizes.iter().sum();
    if total < avail {
        let idx: Vec<usize> = (0..sizes.len())
            .filter(|&i| grow_max[i].is_some())
            .collect();
        let max: Vec<f32> = grow_max.iter().map(|m| m.unwrap_or(0.0)).collect();
        fill_up(sizes, &idx, &max, avail - total);
    } else if total > avail {
        let idx: Vec<usize> = (0..sizes.len())
            .filter(|&i| shrink_min[i].is_some_and(|m| m < sizes[i]))
            .collect();
        let min: Vec<f32> = shrink_min.iter().map(|m| m.unwrap_or(0.0)).collect();
        fill_down(sizes, &idx, &min, total - avail);
    }
}

fn apply_size(size: Size, pref: f32, min: f32) -> (f32, f32) {
    match size {
        Size::Fixed(v) => (v, v),
        Size::Fit { .. } | Size::Grow { .. } => {
            let (lo, hi) = size.bounds();
            // Its max never cuts under its content's least width (a wrapping
            // text's longest word): a word is never split between characters
            // because a box's max was narrower than it.
            let p = pref.clamp(lo, hi).max(min.min(pref));
            (p, min.max(lo).min(p))
        }
        Size::Percent(_) | Size::Calc(_) => (pref, min.min(pref)),
        Size::MinContent => (min, min),
        Size::MaxContent => (pref, pref),
    }
}

fn gaps(gap: f32, n: usize) -> f32 {
    gap * n.saturating_sub(1) as f32
}

/// A flex item's inputs along the main axis (CSS Flexbox §9.2, §9.3 and
/// §4.5): its flex base size, its hypothetical main size (the base held to
/// its bounds), its min and max, and its factors. An item without CSS flex
/// factors takes its house kind's: a stretch size grows and shrinks, a fit
/// one shrinks, and a definite one neither.
#[derive(Clone, Copy, Debug)]
struct FlexInputs {
    base: f32,
    hypo: f32,
    min: f32,
    max: f32,
    grow: f32,
    shrink: f32,
}

fn flex_inputs(n: &Node, horizontal: bool, inner: f32) -> FlexInputs {
    let axis = usize::from(!horizontal);
    let (size, pref, min) = if horizontal {
        (n.el.width, n.layout.pref_w, n.layout.min_w)
    } else {
        (n.el.height, n.layout.pref_h, n.layout.min_h)
    };
    let f = n.el.flex.unwrap_or(match size {
        Size::Grow { .. } => FlexItem {
            grow: 1.0,
            ..FlexItem::default()
        },
        Size::Fit { .. } => FlexItem::default(),
        _ => FlexItem {
            shrink: 0.0,
            ..FlexItem::default()
        },
    });
    // CSS Flexbox §4.4: a collapsed item keeps its cross size (its line's
    // strut) and gives up its main size.
    if n.props.interact.visibility == crate::element::Visibility::Collapse {
        return FlexInputs {
            base: 0.0,
            hypo: 0.0,
            min: 0.0,
            max: 0.0,
            grow: 0.0,
            shrink: 0.0,
        };
    }
    let specified = size.definite(pref, min, Some(inner));
    let base = f
        .basis
        .and_then(|b| b.definite(pref, min, Some(inner)))
        .or(specified)
        .unwrap_or(pref);
    let explicit = n.el.bounds[axis][0]
        .and_then(|b| b.definite(pref, min, Some(inner)))
        .or(match size {
            Size::Fit { min: m, .. } | Size::Grow { min: m, .. } if m > 0.0 => Some(m),
            _ => None,
        });
    // Its automatic minimum: its content's least size, at most its
    // specified one; a scroll container's is 0.
    let scrolls = if horizontal {
        n.el.scroll_x
    } else {
        n.el.scroll
    };
    let content = n.layout.min_content[axis];
    let auto = if scrolls {
        0.0
    } else {
        specified.map_or(min, |s| content.min(s))
    };
    let lo = explicit.unwrap_or(auto);
    let hi = n.el.bounds[axis][1]
        .and_then(|b| b.definite(pref, min, Some(inner)))
        .or(match size {
            Size::Fit { max, .. } | Size::Grow { max, .. } => Some(max),
            _ => None,
        })
        .unwrap_or(f32::INFINITY)
        .max(lo);
    FlexInputs {
        base,
        hypo: base.clamp(lo, hi),
        min: lo,
        max: hi,
        grow: f.grow.max(0.0),
        shrink: f.shrink.max(0.0),
    }
}

/// CSS Flexbox §9.7, resolving flexible lengths: one line's main sizes in
/// `avail` (its room less gaps and margins), each item grown by its
/// share of the positive free space or shrunk by its scaled share of the
/// negative, the items its bounds clamp frozen and the rest resolved again.
fn css_flex(items: &[FlexInputs], avail: f32) -> Vec<f32> {
    let n = items.len();
    let growing = items.iter().map(|it| it.hypo).sum::<f32>() < avail;
    let factor = |it: &FlexInputs| if growing { it.grow } else { it.shrink };
    let mut target: Vec<f32> = items.iter().map(|it| it.hypo).collect();
    let mut frozen: Vec<bool> = items
        .iter()
        .map(|it| {
            factor(it) == 0.0 || (growing && it.base > it.hypo) || (!growing && it.base < it.hypo)
        })
        .collect();
    let room = |target: &[f32], frozen: &[bool]| {
        avail
            - (0..n)
                .map(|k| if frozen[k] { target[k] } else { items[k].base })
                .sum::<f32>()
    };
    let initial = room(&target, &frozen);
    for _ in 0..=n {
        if frozen.iter().all(|&f| f) {
            break;
        }
        let open: Vec<usize> = (0..n).filter(|&k| !frozen[k]).collect();
        let mut free = room(&target, &frozen);
        let factors: f32 = open.iter().map(|&k| factor(&items[k])).sum();
        if factors < 1.0 && (initial * factors).abs() < free.abs() {
            free = initial * factors;
        }
        if growing {
            for &k in &open {
                let share = if factors > 0.0 {
                    items[k].grow / factors
                } else {
                    0.0
                };
                target[k] = items[k].base + free * share;
            }
        } else {
            let scaled: f32 = open.iter().map(|&k| items[k].shrink * items[k].base).sum();
            for &k in &open {
                let share = if scaled > 0.0 {
                    items[k].shrink * items[k].base / scaled
                } else {
                    0.0
                };
                target[k] = items[k].base + free * share;
            }
        }
        let mut total = 0.0;
        let mut violation = vec![0.0f32; n];
        for &k in &open {
            let c = target[k].clamp(items[k].min, items[k].max);
            violation[k] = c - target[k];
            total += violation[k];
            target[k] = c;
        }
        for &k in &open {
            if total == 0.0
                || (total > 0.0 && violation[k] > 0.0)
                || (total < 0.0 && violation[k] < 0.0)
            {
                frozen[k] = true;
            }
        }
    }
    target
}

/// Whether a line of `kids` lays out by CSS Flexbox's algorithm: one of
/// them declares flex factors.
pub(super) fn css_line(nodes: &[Node], kids: &[usize]) -> bool {
    kids.iter().any(|&c| nodes[c].el.flex.is_some())
}

/// Main-axis sizes of a line of children: base, then grow or shrink.
fn main_sizes(
    nodes: &[Node],
    kids: &[usize],
    horizontal: bool,
    inner: f32,
    avail: f32,
    allow_shrink: bool,
) -> Vec<f32> {
    if css_line(nodes, kids) {
        let items: Vec<FlexInputs> = kids
            .iter()
            .map(|&c| flex_inputs(&nodes[c], horizontal, inner))
            .collect();
        return css_flex(&items, avail);
    }
    let mut sizes = Vec::with_capacity(kids.len());
    let mut grow = Vec::with_capacity(kids.len());
    let mut shrink = Vec::with_capacity(kids.len());
    for &c in kids {
        let n = &nodes[c];
        let (size, pref, min) = if horizontal {
            (n.el.width, n.layout.pref_w, n.layout.min_w)
        } else {
            (n.el.height, n.layout.pref_h, n.layout.min_h)
        };
        let base = size.definite(pref, min, Some(inner)).unwrap_or(pref);
        sizes.push(clamp_to(n, usize::from(!horizontal), Some(inner), base));
        grow.push(match size {
            Size::Grow { .. } => Some(size.bounds().1),
            _ => None,
        });
        shrink.push(match size {
            Size::Fit { .. } | Size::Grow { .. } if allow_shrink => Some(min),
            _ => None,
        });
    }
    flex(&mut sizes, &grow, &shrink, avail);
    for (s, &c) in sizes.iter_mut().zip(kids) {
        *s = clamp_to(&nodes[c], usize::from(!horizontal), Some(inner), *s);
    }
    sizes
}
