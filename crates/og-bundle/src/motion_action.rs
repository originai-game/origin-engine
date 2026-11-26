//! Authored motion reactions: a humanoid clip's entry, hold and exit, with
//! an optional hand contact in the head's local frame.

use og_core::ogt::Document;

/// Authored action fields, shared by the reader and file directory.
pub const LINES: &[og_core::ogt::line::Line] = &[
    og_core::ogt::line::Line::new(
        "source",
        "\"<model asset>\"",
        "required",
        "Source animation model, checked by cook.",
        "",
    ),
    og_core::ogt::line::Line::new(
        "clip",
        "\"<clip name>\"",
        "required",
        "Authored clip name, not a runtime handle.",
        "",
    ),
    og_core::ogt::line::Line::new(
        "rig",
        "\"<rig asset>\"",
        "detected",
        "Optional humanoid map, checked by cook.",
        "",
    ),
    og_core::ogt::line::Line::new(
        "holdStart",
        "<seconds >= 0>",
        "0",
        "Beginning of the authored hold interval.",
        "",
    ),
    og_core::ogt::line::Line::new(
        "holdEnd",
        "<seconds > holdStart>",
        "required",
        "End of the authored hold interval.",
        "",
    ),
    og_core::ogt::line::Line::new("hand", "left | right", "", "Optional hand contact.", ""),
    og_core::ogt::line::Line::new(
        "handTarget",
        "<x> <y> <z>",
        "0 0 0",
        "Finite target in the head's local frame.",
        "",
    ),
];

#[derive(Clone, Debug, PartialEq)]
pub struct MotionAction {
    pub source: String,
    pub clip: String,
    pub rig: String,
    pub hold_start: f32,
    pub hold_end: f32,
    /// `left` or `right`; empty means no hand contact.
    pub hand: String,
    pub hand_target: [f32; 3],
}

impl MotionAction {
    pub fn parse(text: &str) -> Result<Self, String> {
        let doc = Document::parse(text).map_err(|e| e.to_string())?;
        if doc.ty != "motion-action" || doc.version != 1 {
            return Err("expected type motion-action, version 1".into());
        }
        let mut action = Self {
            source: String::new(),
            clip: String::new(),
            rig: String::new(),
            hold_start: 0.0,
            hold_end: 0.0,
            hand: String::new(),
            hand_target: [0.0; 3],
        };
        for e in &doc.body.entries {
            let err = |x: og_core::Error| format!("line {}: {x}", e.line);
            if !LINES.iter().any(|f| f.key == e.key) {
                return Err(format!(
                    "line {}: unknown motion-action field '{}'",
                    e.line, e.key
                ));
            }
            match e.key.as_str() {
                "source" => action.source = e.string(0).map_err(err)?.into(),
                "clip" => action.clip = e.string(0).map_err(err)?.into(),
                "rig" => action.rig = e.string(0).map_err(err)?.into(),
                "holdStart" => action.hold_start = e.f32(0).map_err(err)?,
                "holdEnd" => action.hold_end = e.f32(0).map_err(err)?,
                "hand" => action.hand = e.word(0).map_err(err)?.into(),
                "handTarget" => {
                    action.hand_target = [
                        e.f32(0).map_err(err)?,
                        e.f32(1).map_err(err)?,
                        e.f32(2).map_err(err)?,
                    ]
                }
                _ => {
                    return Err(format!(
                        "line {}: unknown motion-action field '{}'",
                        e.line, e.key
                    ));
                }
            }
        }
        if action.source.is_empty() || action.clip.is_empty() {
            return Err("a motion-action needs source and clip".into());
        }
        if !(action.hold_start.is_finite()
            && action.hold_end.is_finite()
            && action.hold_start >= 0.0
            && action.hold_end > action.hold_start)
        {
            return Err("motion-action needs 0 <= holdStart < holdEnd".into());
        }
        if !matches!(action.hand.as_str(), "" | "left" | "right")
            || !action.hand_target.iter().all(|v| v.is_finite())
        {
            return Err("motion-action hand must be left or right with a finite handTarget".into());
        }
        Ok(action)
    }
}
