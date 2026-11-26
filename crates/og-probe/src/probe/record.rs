//! What `og render` needs of the probe (docs/design.md §2.9): the frame the
//! host drew last, read every frame without the `capture` round trip.

use super::Probe;
use crate::hooks::RawFrame;

impl Probe {
    /// The frame the host's loop drew last ([`crate::Capture::frame`]);
    /// `None` without a renderer or before the first frame.
    pub fn frame_rgba(&mut self) -> Option<RawFrame> {
        self.host.frame_rgba()
    }
}
