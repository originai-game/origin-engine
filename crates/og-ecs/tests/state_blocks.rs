//! State blocks: module state outside archetypes and resources travels with
//! the snapshot, is covered by the world hash, and comes back on restore; a
//! refused restore changes neither the world nor any block; a snapshot
//! without a block resets it; a world without blocks is untouched.

use og_core::ErrorKind;
use og_core::math::Vec3;
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
    /// A count is no place.
    fn shift(&mut self, _: &og_ecs::Shift) {}
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
