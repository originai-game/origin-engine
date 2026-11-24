//! What `og render` needs of the probe (docs/design.md §2.9): the frame the
//! host drew last, read every frame without the `capture` round trip, and a
//! whole-world snapshot that leaves this process, so a job of `og render
//! --jobs N` starts its range from the world the leader simulated.
//!
//! A render snapshot is not a save: it is the world exactly (the input
//! resources included, which a save keeps from the session) plus the
//! probe's pointer, and restoring it drops the script steps scheduled
//! before its tick (the leader ran them; their effects are in the world)
//! instead of moving them with the clock as a save's restore does.

use super::Probe;
use crate::hooks::RawFrame;
use og_core::{Error, ErrorKind};
use og_ecs::World;

const MAGIC: &[u8; 4] = b"OGRS";
const VERSION: u32 = 1;
/// Magic, version, tick, pointer x and y.
const HEADER: usize = 4 + 4 + 8 + 4 + 4;

impl Probe {
    /// The frame the host's loop drew last ([`crate::Capture::frame`]);
    /// `None` without a renderer or before the first frame.
    pub fn frame_rgba(&mut self) -> Option<RawFrame> {
        self.host.frame_rgba()
    }

    /// The world as a render snapshot: its bytes and the probe's pointer.
    pub fn render_snapshot(&self, world: &World) -> Vec<u8> {
        let bytes = world.snapshot();
        let mut out = Vec::with_capacity(HEADER + bytes.len());
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&VERSION.to_le_bytes());
        out.extend_from_slice(&world.time().tick.to_le_bytes());
        out.extend_from_slice(&self.inject.pointer.0.to_le_bytes());
        out.extend_from_slice(&self.inject.pointer.1.to_le_bytes());
        out.extend_from_slice(&bytes);
        out
    }

    /// Restores a [`Probe::render_snapshot`] taken by another process of the
    /// same game and script: the world as it was, the pointer, and of the
    /// steps scheduled here only those at or after its tick. The tick.
    pub fn render_restore(&mut self, world: &mut World, blob: &[u8]) -> og_core::Result<u64> {
        let bad = |why: &str| Error::new(ErrorKind::Parse, format!("render snapshot: {why}"));
        if blob.len() < HEADER || &blob[0..4] != MAGIC {
            return Err(bad("not one (no OGRS header)"));
        }
        let version = u32::from_le_bytes(blob[4..8].try_into().expect("4"));
        if version != VERSION {
            return Err(bad(&format!(
                "version {version}, this engine reads {VERSION}"
            )));
        }
        let tick = u64::from_le_bytes(blob[8..16].try_into().expect("8"));
        let x = f32::from_le_bytes(blob[16..20].try_into().expect("4"));
        let y = f32::from_le_bytes(blob[20..24].try_into().expect("4"));
        world.restore(&blob[HEADER..])?;
        if world.time().tick != tick {
            return Err(bad(&format!(
                "its world is at tick {}, its header says {tick}",
                world.time().tick
            )));
        }
        self.inject.pointer = (x, y);
        self.schedule.queue.retain(|&(at, _), _| at >= tick);
        self.schedule.shot = None;
        if let Some(c) = self.host.capture.as_mut() {
            c.restore_state(0);
        }
        Ok(tick)
    }
}
