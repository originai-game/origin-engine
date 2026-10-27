//! What a view shows, in words, for the prompt: which labelled parts are in
//! the area to paint and roughly where, and which pixels are roofs or floor.
//! Image models invent less when told what the grey shapes are.

use super::GBuffer;

/// Per pixel (rows from the top): the part label (-1 sky), roof, floor.
pub struct Classes {
    pub label: Vec<i32>,
    pub roof: Vec<u8>,
    pub floor: Vec<u8>,
}

/// The parts in an area with rough positions, biggest first, e.g.
/// "house walls (left), stone stairs (bottom)"; empty without part names.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Description {
    pub parts: String,
    pub roof_share: f32,
    pub floor_share: f32,
}

/// A roof surface faces up at least this much: gable ends labelled roof are walls.
pub const ROOF_MIN_NY: f32 = 0.25;

/// Labels that name roofs; `None` when none does (roofs are then found by
/// slope and height).
pub fn roof_labels(labels: &[String]) -> Option<Vec<bool>> {
    let r: Vec<bool> = labels
        .iter()
        .map(|l| {
            l.to_lowercase()
                .split(|c: char| !c.is_alphanumeric())
                .any(|w| w == "roof" || w == "roofs")
        })
        .collect();
    r.iter().any(|&b| b).then_some(r)
}

/// Roof test for one surface point.
pub fn is_roof(roofs: Option<&[bool]>, label: usize, ny: f32, y: f32, ground_y: f32) -> bool {
    match roofs {
        Some(r) => r.get(label).copied().unwrap_or(false) && ny > ROOF_MIN_NY,
        None => ny > 0.35 && y > ground_y + 3.5,
    }
}

pub fn classify(g: &GBuffer, labels: &[String], ground_y: f32) -> Classes {
    let roofs = roof_labels(labels);
    let n = g.hits.len();
    let mut c = Classes {
        label: vec![-1; n],
        roof: vec![0; n],
        floor: vec![0; n],
    };
    for (i, h) in g.hits.iter().enumerate() {
        let Some(h) = h else { continue };
        c.label[i] = h.label as i32;
        if is_roof(
            roofs.as_deref(),
            h.label as usize,
            h.normal.y,
            h.position.y,
            ground_y,
        ) {
            c.roof[i] = 1;
        } else if h.normal.y > 0.7 {
            c.floor[i] = 1;
        }
    }
    c
}

fn place(cx: f32, cy: f32) -> String {
    let h = if cx < 0.33 {
        "left"
    } else if cx > 0.67 {
        "right"
    } else {
        "center"
    };
    let v = if cy < 0.33 {
        "top"
    } else if cy > 0.67 {
        "bottom"
    } else {
        ""
    };
    match (v, h) {
        ("", h) => h.to_string(),
        (v, "center") => v.to_string(),
        (v, h) => format!("{v} {h}"),
    }
}

/// The parts inside `area` (every geometry pixel when `None`): each part
/// over 0.5 % of it, at most twelve, so a small object the model would
/// otherwise guess at (a fountain beside a market stall) is named.
pub fn describe(c: &Classes, labels: &[String], size: u32, area: Option<&[u8]>) -> Description {
    let k = labels.len();
    let (mut count, mut sx, mut sy) = (vec![0f64; k], vec![0f64; k], vec![0f64; k]);
    let (mut total, mut roofs, mut floors) = (0f64, 0f64, 0f64);
    for i in 0..c.label.len() {
        let l = c.label[i];
        if l < 0 || area.is_some_and(|a| a[i] == 0) {
            continue;
        }
        total += 1.0;
        roofs += c.roof[i] as f64;
        floors += c.floor[i] as f64;
        let l = l as usize;
        if l < k {
            count[l] += 1.0;
            sx[l] += (i as u32 % size) as f64 / size as f64;
            sy[l] += (i as u32 / size) as f64 / size as f64;
        }
    }
    let mut parts: Vec<(usize, f64)> = (0..k)
        .map(|l| (l, if total > 0.0 { count[l] / total } else { 0.0 }))
        .filter(|&(_, s)| s >= 0.005)
        .collect();
    parts.sort_by(|a, b| b.1.total_cmp(&a.1));
    let text = if k > 1 {
        parts
            .iter()
            .take(12)
            .map(|&(l, share)| {
                let at = if share > 0.4 {
                    "most of the frame".to_string()
                } else {
                    place((sx[l] / count[l]) as f32, (sy[l] / count[l]) as f32)
                };
                format!(
                    "{} ({}{})",
                    labels[l],
                    at,
                    if share < 0.04 { ", small" } else { "" }
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        String::new()
    };
    let share = |v: f64| if total > 0.0 { (v / total) as f32 } else { 0.0 };
    Description {
        parts: text,
        roof_share: share(roofs),
        floor_share: share(floors),
    }
}
