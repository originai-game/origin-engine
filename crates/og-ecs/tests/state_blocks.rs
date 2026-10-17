//! State blocks: module state outside archetypes and resources travels with
//! the snapshot, is covered by the world hash, and comes back on restore; a
//! refused restore changes neither the world nor any block; a snapshot
//! without a block resets it; a world without blocks is untouched.

use og_core::math::Vec3;
use og_core::ErrorKind;
use og_ecs::{Reflect, StateBlock, World, WorldDesc};
use std::any::Any;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Pos {
    p: Vec3,
}

/// A block whose state is one u64; `None` after a reset. Its bytes are the
/// value, little-endian; anything but 8 bytes is refused.
#[derive(Clone, Default)]
struct Counter(Arc<Mutex<Option<u64>>>);

/// A saved value that writes 3 bytes, which `decode` refuses.
const UNREADABLE: u64 = u64::MAX;

impl Counter {
    fn get(&self) -> Option<u64> {
        *self.0.lock().unwrap()
    }
    fn set(&self, v: u64) {
        *self.0.lock().unwrap() = Some(v);
    }
}

impl StateBlock for Counter {
    fn save(&self, out: &mut Vec<u8>) {
        match self.get() {
            Some(UNREADABLE) => out.extend_from_slice(&[1, 2, 3]),
            Some(v) => out.extend_from_slice(&v.to_le_bytes()),
            None => {}
        }
    }
    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn Any + Send>> {
        match bytes.len() {
            0 => Ok(Box::new(None::<u64>)),
            8 => Ok(Box::new(Some(u64::from_le_bytes(
                bytes.try_into().unwrap(),
            )))),
            n => Err(og_core::Error::new(
                ErrorKind::Parse,
                format!("a counter is 8 bytes, got {n}"),
            )),
        }
    }
    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        *self.0.lock().unwrap() = decoded.and_then(|d| *d.downcast::<Option<u64>>().unwrap());
    }
}

fn world() -> World {
    let mut w = World::new(&WorldDesc {
        seed: 3,
        tick_rate: 60,
    })
    .unwrap();
    w.register::<Pos>().unwrap();
    for i in 0..5 {
        w.spawn((Pos {
            p: Vec3::new(i as f32, 0.0, 0.0),
        },))
            .unwrap();
    }
    w
}

fn with_blocks() -> (World, Counter, Counter) {
    let mut w = world();
    let (a, b) = (Counter::default(), Counter::default());
    w.add_state_block("test.a", Box::new(a.clone())).unwrap();
    w.add_state_block("test.b", Box::new(b.clone())).unwrap();
    (w, a, b)
}

#[test]
fn blocks_travel_with_the_snapshot_into_the_same_and_a_fresh_world() {
    let (mut w, a, b) = with_blocks();
    a.set(41);
    b.set(7);
    let snap = w.snapshot();
    let hash = w.world_hash();

    a.set(1000);
    b.set(1001);
    w.step().unwrap();
    w.restore(&snap).unwrap();
    assert_eq!((a.get(), b.get()), (Some(41), Some(7)));
    assert_eq!(w.world_hash(), hash);
    assert_eq!(w.snapshot(), snap);

    let (mut fresh, fa, fb) = with_blocks();
    fresh.restore(&snap).unwrap();
    assert_eq!((fa.get(), fb.get()), (Some(41), Some(7)));
    assert_eq!(fresh.world_hash(), hash);

    let (mut migrated, ma, mb) = with_blocks();
    let report = migrated.restore_migrating(&snap).unwrap();
    assert!(report.is_exact(), "{report:?}");
    assert_eq!((ma.get(), mb.get()), (Some(41), Some(7)));
}

#[test]
fn the_hash_covers_every_block() {
    let (w1, a1, b1) = with_blocks();
    let (w2, a2, b2) = with_blocks();
    a1.set(5);
    a2.set(5);
    b1.set(9);
    b2.set(9);
    assert_eq!(w1.world_hash(), w2.world_hash());
    b2.set(10);
    assert_ne!(
        w1.world_hash(),
        w2.world_hash(),
        "state in a block changes the hash"
    );
    b2.set(9);
    a2.set(6);
    assert_ne!(w1.world_hash(), w2.world_hash());
}

#[test]
fn a_world_without_blocks_snapshots_and_hashes_as_before() {
    let plain = world();
    let (blocked, _, _) = with_blocks();
    // Empty blocks save nothing, but a world with blocks still says so.
    assert_eq!(
        &plain.snapshot()[4..8],
        &2u32.to_le_bytes(),
        "format 2 without blocks"
    );
    assert_eq!(
        &blocked.snapshot()[4..8],
        &3u32.to_le_bytes(),
        "format 3 with blocks"
    );
    let mut again = world();
    again.restore(&plain.snapshot()).unwrap();
    assert_eq!(again.world_hash(), plain.world_hash());
}

#[test]
fn a_snapshot_without_a_block_resets_it() {
    let plain = world().snapshot();
    let (mut w, a, b) = with_blocks();
    a.set(3);
    b.set(4);
    w.restore(&plain).unwrap();
    assert_eq!((a.get(), b.get()), (None, None));

    a.set(3);
    let report = w.restore_migrating(&plain).unwrap();
    assert_eq!(a.get(), None);
    assert!(
        report.warnings.iter().any(|m| m.contains("test.a")),
        "{report:?}"
    );
}

#[test]
fn a_refused_restore_changes_neither_the_world_nor_any_block() {
    // A block that cannot decode its bytes: the world keeps its entities.
    let (mut w, a, b) = with_blocks();
    a.set(11);
    b.set(UNREADABLE);
    let bad = w.snapshot();
    a.set(12);
    b.set(13);
    w.spawn((Pos { p: Vec3::ONE },)).unwrap();
    let before = w.snapshot();
    let err = w.restore(&bad).unwrap_err();
    assert!(err.reason().contains("test.b"), "{}", err.reason());
    assert!(
        err.reason().contains("a counter is 8 bytes"),
        "{}",
        err.reason()
    );
    assert_eq!(w.snapshot(), before, "world and blocks unchanged");
    assert_eq!((a.get(), b.get()), (Some(12), Some(13)));
    assert!(w.restore_migrating(&bad).is_err());
    assert_eq!(w.snapshot(), before);

    // A world-level refusal: no block sees the snapshot.
    let mut truncated = w.snapshot();
    a.set(20);
    truncated.truncate(truncated.len() - 3);
    assert!(w.restore(&truncated).is_err());
    assert_eq!(a.get(), Some(20));
}

#[test]
fn a_block_this_world_lacks_is_refused_and_dropped_on_migration() {
    let (w, a, _) = with_blocks();
    a.set(8);
    let snap = w.snapshot();
    let mut plain = world();
    plain.step().unwrap();
    let before = plain.snapshot();
    let err = plain.restore(&snap).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::NotFound);
    assert!(err.reason().contains("test.a"), "{}", err.reason());
    assert_eq!(plain.snapshot(), before);

    let report = plain.restore_migrating(&snap).unwrap();
    assert!(
        report.dropped.iter().any(|d| d.contains("test.a")),
        "{report:?}"
    );
    assert_eq!(
        plain.world_hash(),
        world().world_hash(),
        "the world itself restored"
    );
}

#[test]
fn block_names_are_unique() {
    let (mut w, _, _) = with_blocks();
    let err = w
        .add_state_block("test.a", Box::new(Counter::default()))
        .unwrap_err();
    assert!(err.reason().contains("test.a"), "{}", err.reason());
    assert!(w.add_state_block("", Box::new(Counter::default())).is_err());
}
