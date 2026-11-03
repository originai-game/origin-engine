//! Authored motion reactions: a humanoid clip's entry, hold and exit, with
//! an optional hand contact in the head's local frame.

use og_core::ogt::Document;

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

#[cfg(test)]
mod tests {
    use super::MotionAction;

    #[test]
    fn action_requires_a_finite_forward_hold_interval() {
        let text = "type motion-action\nversion 1\n---\nsource \"brace.glb\"\nclip \"brace\"\nholdStart 0.5\nholdEnd 1.5\nhand right\nhandTarget 0.1 0.02 -0.03\n";
        let action = MotionAction::parse(text).unwrap();
        assert_eq!(action.hand_target, [0.1, 0.02, -0.03]);
        for (from, to) in [
            ("holdEnd 1.5", "holdEnd 0.5"),
            ("holdStart 0.5", "holdStart -1"),
            ("hand right", "hand foot"),
            ("source \"brace.glb\"", "source \"\""),
        ] {
            assert!(MotionAction::parse(&text.replace(from, to)).is_err());
        }
    }
}
