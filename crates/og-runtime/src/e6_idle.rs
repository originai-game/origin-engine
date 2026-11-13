//! What og.nav and og.ai leave in a world that never uses them: their
//! systems (names, declarations, queries) and their state blocks as an idle
//! full build saves them. A build without og-runtime's `nav` or `ai`
//! feature mounts these instead of og-ai's (src/nav_absent.rs,
//! src/ai_absent.rs), so its world, saves and world hash are the full
//! build's for a game that never names the module; the full build's tests
//! hold the two to the same bytes (tests at the end).
//!
//! An idle block is its first form's counts, all zero (og.nav: paths,
//! grids, navmeshes, flow fields; og.ai: memories, seen and heard ticks,
//! forward vectors, noises), then the trailer's origin section once
//! og.WorldOrigin has moved the world's zero. A save that holds anything
//! else is refused with the module's reason, never restored into nothing.

use og_core::glam::Vec3;
use og_core::{Error, ErrorKind, Result};
use og_ecs::{QueryDesc, Shift, StateBlock, SystemDesc, World, trailer};
use std::any::Any;

/// og.nav's system and state block name (og-ai's `nav::SYSTEM_NAME`).
pub(crate) const NAV: &str = "og.nav";
/// og.ai's (og-ai's `SYSTEM_NAME`).
pub(crate) const AI: &str = "og.ai";

/// An idle module's state block.
struct Idle {
    /// The module's block name, for errors.
    name: &'static str,
    /// Zero `u32` counts in its first form.
    counts: usize,
    /// Its trailer's origin tag.
    origin_tag: u16,
    origin: Vec3,
    /// Why a save holding the module's state is refused.
    absent: &'static str,
}

impl StateBlock for Idle {
    fn save(&self, out: &mut Vec<u8>) {
        out.resize(out.len() + self.counts * 4, 0);
        let mut sections = Vec::new();
        if self.origin != Vec3::ZERO {
            let mut b = Vec::with_capacity(12);
            for c in self.origin.to_array() {
                b.extend_from_slice(&c.to_le_bytes());
            }
            sections.push((self.origin_tag, b));
        }
        trailer::write(out, &sections);
    }

    fn decode(&self, bytes: &[u8]) -> Result<Box<dyn Any + Send>> {
        let refuse = || Error::new(ErrorKind::Invalid, self.absent);
        let n = self.counts * 4;
        if bytes.len() < n || bytes[..n].iter().any(|&b| b != 0) {
            return Err(refuse());
        }
        let mut origin = Vec3::ZERO;
        for (tag, b) in trailer::read(self.name, &bytes[n..])? {
            if tag != self.origin_tag || b.len() != 12 {
                return Err(refuse());
            }
            let c = |i: usize| f32::from_le_bytes(b[i..i + 4].try_into().unwrap());
            origin = Vec3::new(c(0), c(4), c(8));
        }
        Ok(Box::new(origin))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        self.origin = decoded
            .and_then(|d| d.downcast::<Vec3>().ok())
            .map_or(Vec3::ZERO, |o| *o);
    }

    fn shift(&mut self, shift: &Shift) {
        self.origin = shift.moved(self.origin);
    }
}

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

fn query(read: &[&str], write: &[&str]) -> QueryDesc {
    QueryDesc {
        read: names(read),
        write: names(write),
        ..QueryDesc::default()
    }
}

/// og.nav as og-ai's `nav::install` schedules it, running nothing.
#[cfg_attr(feature = "nav", allow(dead_code))]
pub(crate) fn mount_nav(world: &mut World, absent: &'static str) -> Result<()> {
    let desc = SystemDesc {
        name: NAV.into(),
        writes: names(&["og.NavAgent", "og.Transform", "og.Velocity", "og.Steering"]),
        queries: vec![query(&[], &["og.NavAgent", "og.Transform"])],
        ..SystemDesc::default()
    };
    world.add_system(desc, |_| Ok(()))?;
    world.add_state_block(
        NAV,
        Box::new(Idle {
            name: NAV,
            counts: 4,
            origin_tag: 5,
            origin: Vec3::ZERO,
            absent,
        }),
    )
}

/// og.ai as og-ai's `install` schedules it (reading every record of the
/// world in `records`), running nothing.
#[cfg_attr(feature = "ai", allow(dead_code))]
pub(crate) fn mount_ai(world: &mut World, records: &[String], absent: &'static str) -> Result<()> {
    let mut desc = SystemDesc {
        name: AI.into(),
        reads: names(&[
            "og.Perceivable",
            "og.Parent",
            "og.Transform",
            "og.Velocity",
            "og.SoundStarted",
            "og.AudioSource",
        ]),
        writes: names(&["og.AiAgent", "og.Perception", "og.NavAgent", "og.AiChanged"]),
        queries: vec![
            query(&["og.Transform"], &["og.AiAgent"]),
            query(&["og.Perceivable", "og.Transform"], &[]),
        ],
        ..SystemDesc::default()
    };
    for name in records {
        let Ok(id) = world.id_by_name(name) else {
            continue;
        };
        if world.schema(id).is_err() {
            continue;
        }
        if !desc.reads.contains(name) && !desc.writes.contains(name) {
            desc.reads.push(name.clone());
        }
    }
    world.add_system(desc, |_| Ok(()))?;
    world.add_state_block(
        AI,
        Box::new(Idle {
            name: AI,
            counts: 5,
            origin_tag: 2,
            origin: Vec3::ZERO,
            absent,
        }),
    )
}

#[cfg(all(test, feature = "nav", feature = "ai"))]
mod tests {
    use og_ecs::{World, WorldDesc};

    /// The records og-ai's two `install`s register, by their types.
    fn records(w: &mut World) {
        w.register::<og_ai::nav::system::Transform>().unwrap();
        w.register::<og_ai::nav::system::Velocity>().unwrap();
        w.register::<og_ai::nav::NavAgent>().unwrap();
        w.register::<og_ai::nav::Steering>().unwrap();
        w.register::<og_ai::AiAgent>().unwrap();
        w.register::<og_ai::Perception>().unwrap();
        w.register::<og_ai::Perceivable>().unwrap();
        w.register::<og_ai::Parent>().unwrap();
        w.register::<og_ai::AiChanged>().unwrap();
        w.register::<og_audio::schema::SoundStarted>().unwrap();
        w.register::<og_audio::AudioSource>().unwrap();
    }

    fn full() -> World {
        let mut w = World::new(&WorldDesc::default()).unwrap();
        records(&mut w);
        let plane = og_ai::nav::Plane::Xz;
        og_ai::install(
            &mut w,
            &["og.NavAgent".to_string()],
            og_ai::AiDesc {
                plane,
                ray: None,
                sink: None,
                behaviours: None,
            },
        )
        .unwrap();
        og_ai::nav::install(
            &mut w,
            og_ai::nav::SharedNavs::default(),
            og_ai::nav::NavDesc {
                plane,
                raycast: None,
                plans_per_tick: og_ai::nav::PLANS_PER_TICK,
            },
        )
        .unwrap();
        w
    }

    fn idle() -> World {
        let mut w = World::new(&WorldDesc::default()).unwrap();
        records(&mut w);
        super::mount_ai(&mut w, &["og.NavAgent".to_string()], "no ai").unwrap();
        super::mount_nav(&mut w, "no nav").unwrap();
        w
    }

    /// A build without `nav` and `ai` saves and hashes an idle world as the
    /// full build does, and each restores the other's save.
    #[test]
    fn the_idle_modules_save_what_an_idle_full_build_saves() {
        let (mut a, mut b) = (full(), idle());
        assert_eq!(a.snapshot(), b.snapshot());
        assert_eq!(a.world_hash(), b.world_hash());
        let (sa, sb) = (a.snapshot(), b.snapshot());
        b.restore(&sa).unwrap();
        a.restore(&sb).unwrap();
        assert_eq!(a.world_hash(), b.world_hash());
    }
}
