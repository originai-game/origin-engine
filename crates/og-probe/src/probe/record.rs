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
        self.capture.as_mut()?.frame()
    }

    /// The world as a render snapshot: its bytes and the probe's pointer.
    pub fn render_snapshot(&self, world: &World) -> Vec<u8> {
        let bytes = world.snapshot();
        let mut out = Vec::with_capacity(HEADER + bytes.len());
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&VERSION.to_le_bytes());
        out.extend_from_slice(&world.time().tick.to_le_bytes());
        out.extend_from_slice(&self.pointer.0.to_le_bytes());
        out.extend_from_slice(&self.pointer.1.to_le_bytes());
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
        self.pointer = (x, y);
        self.queue.retain(|&(at, _), _| at >= tick);
        self.shot = None;
        if let Some(c) = self.capture.as_mut() {
            c.restore_state(0);
        }
        Ok(tick)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use og_ecs::WorldDesc;

    fn world() -> World {
        World::new(&WorldDesc {
            tick_rate: 60,
            seed: 7,
        })
        .unwrap()
    }

    #[test]
    fn a_render_snapshot_moves_the_world_and_keeps_only_the_steps_after_it() {
        let mut a = world();
        let mut pa = Probe::new(&mut a).unwrap();
        pa.schedule(&a, "0.05:touch_move:10,20;0.5:touch_move:30,40;1:quit")
            .unwrap();
        pa.advance(&mut a, 40).unwrap();
        let blob = pa.render_snapshot(&a);

        // Another process: the same script from tick 0, then the leader's world.
        let mut b = world();
        let mut pb = Probe::new(&mut b).unwrap();
        pb.schedule(&b, "0.05:touch_move:10,20;0.5:touch_move:30,40;1:quit")
            .unwrap();
        assert_eq!(pb.pending(), 3);
        assert_eq!(pb.render_restore(&mut b, &blob).unwrap(), 40);
        assert_eq!(b.time().tick, 40);
        assert_eq!(b.world_hash(), a.world_hash());
        assert_eq!(
            pb.pointer,
            (30.0, 40.0),
            "the pointer the leader's steps left"
        );
        // Both moves ran in the leader (ticks 3 and 30): only quit (60) is left.
        assert_eq!(pb.pending(), 1);
        assert!(pb.render_restore(&mut b, b"nope").is_err());
    }
}
