//! Generational handles: a dead handle is an error value, never an alias.

use og_core::handle::{HandleAllocator, SlotState};
use og_core::{ErrorKind, Handle, Pool, RawHandle};

#[test]
fn a_freed_slot_reused_does_not_revive_old_handles() {
    let mut pool: Pool<&str> = Pool::new();
    let a = pool.insert("a");
    let b = pool.insert("b");
    assert_eq!(*pool.get(a).unwrap(), "a");
    assert_eq!(pool.remove(a).unwrap(), "a");
    let c = pool.insert("c"); // reuses a's slot
    assert_eq!(c.raw().index(), a.raw().index());
    assert_ne!(c, a);
    let err = pool.get(a).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::DeadHandle);
    assert!(err.reason().contains("dead"), "{}", err.reason());
    assert_eq!(pool.remove(a).unwrap_err().kind(), ErrorKind::DeadHandle);
    assert_eq!(*pool.get(c).unwrap(), "c");
    assert_eq!(pool.len(), 2);
    let live: Vec<_> = pool.iter().map(|(h, v)| (h, *v)).collect();
    assert_eq!(live, vec![(c, "c"), (b, "b")]);
    let never = Handle::<&str>::from_raw(RawHandle::new(99, 1));
    assert_eq!(pool.get(never).unwrap_err().kind(), ErrorKind::DeadHandle);
}

#[test]
fn bits_round_trip_and_zero_is_never_live() {
    let mut alloc = HandleAllocator::new();
    let h = alloc.alloc();
    assert_eq!(h.generation(), 1);
    assert_ne!(h.to_bits(), 0);
    assert_eq!(RawHandle::from_bits(h.to_bits()), h);
    assert!(!alloc.is_alive(RawHandle::from_bits(0)));
}

#[test]
fn allocator_parts_rebuild_the_same_allocator_and_reject_inconsistency() {
    let mut alloc = HandleAllocator::new();
    let hs: Vec<_> = (0..5).map(|_| alloc.alloc()).collect();
    alloc.free(hs[1]).unwrap();
    alloc.free(hs[3]).unwrap();
    let slots: Vec<SlotState> = alloc.slots().collect();
    let rebuilt = HandleAllocator::from_parts(&slots, alloc.free_list()).unwrap();
    assert_eq!(rebuilt, alloc);
    let (mut a, mut b) = (alloc.clone(), rebuilt);
    assert_eq!(a.alloc(), b.alloc());

    let mut live_in_free = slots.clone();
    live_in_free[1].alive = true;
    assert!(HandleAllocator::from_parts(&live_in_free, &[1, 3]).is_err());
    assert!(HandleAllocator::from_parts(&slots, &[3]).is_err()); // slot 1 lost
    assert!(HandleAllocator::from_parts(&slots, &[1, 1, 3]).is_err());
    assert!(HandleAllocator::from_parts(&slots, &[1, 3, 9]).is_err());
}
