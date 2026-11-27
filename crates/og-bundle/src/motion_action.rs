//! Checked humanoid reactions: a clip's entry, hold and exit, with an optional
//! hand contact in the head's local frame. No author text parser lives here.

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
    /// Domain invariants shared by cook and every runtime host. The clip's
    /// actual duration is checked when its animation is loaded for playback.
    pub fn validate(&self) -> Result<(), String> {
        if self.source.is_empty() || self.clip.is_empty() {
            return Err("a motion-action needs source and clip".into());
        }
        if !(self.hold_start.is_finite()
            && self.hold_end.is_finite()
            && self.hold_start >= 0.0
            && self.hold_end > self.hold_start)
        {
            return Err("motion-action needs 0 <= holdStart < holdEnd".into());
        }
        if !matches!(self.hand.as_str(), "" | "left" | "right")
            || !self.hand_target.iter().all(|v| v.is_finite())
        {
            return Err("motion-action hand must be left or right with a finite handTarget".into());
        }
        Ok(())
    }
}
