//! One stable sort for the engine's own slices, compiled once.
//!
//! `[T]::sort_by` and its siblings monomorphise the standard library's
//! driftsort for every element type and every comparator closure: two to
//! eighteen kilobytes of wasm each, about 930 kB of the 3D publish module
//! across the engine's and its dependencies' sorts
//! (`tests/evidence/r4/size-2/NOTES.md`). These functions sort the same
//! way, stably, with the same result for every comparator that is a total
//! order, through one merge sort of `u32` positions that is not generic:
//! each call site adds only its comparator and the permutation's
//! application. A comparison is an indirect call, so a sort costs a little
//! more time than driftsort's; none of the engine's sorts is on a path where
//! that shows (they order draw lists, events and handles, not millions of
//! items).
//!
//! The order of equal elements is their original order, as `sort_by`'s is,
//! so swapping one for the other never moves a frame or a world hash.
//! `sort_unstable*` callers keep the standard library's unstable sort: what
//! it does with ties is its own, and a world that sorts with it depends on
//! that.

use std::cmp::Ordering;

/// Sorts `v` stably by `compare`, as `v.sort_by(compare)` does.
pub fn sort_by<T, F: FnMut(&T, &T) -> Ordering>(v: &mut [T], mut compare: F) {
    if v.len() < 2 {
        return;
    }
    let order = positions(v.len(), &mut |a, b| {
        compare(&v[a as usize], &v[b as usize]) == Ordering::Less
    });
    permute(v, order);
}

/// Sorts `v` stably by the key `key` reads, as `v.sort_by_key(key)` does
/// (the key is read on every comparison).
pub fn sort_by_key<T, K: Ord, F: FnMut(&T) -> K>(v: &mut [T], mut key: F) {
    sort_by(v, |a, b| key(a).cmp(&key(b)));
}

/// Sorts `v` stably by its own order, as `v.sort()` does.
pub fn sort<T: Ord>(v: &mut [T]) {
    sort_by(v, T::cmp);
}

/// The positions `0..len` sorted stably by `less`: a bottom-up merge sort
/// over runs of [`RUN`] made by insertion. Not generic, so it is compiled
/// once for every caller.
#[inline(never)]
fn positions(len: usize, less: &mut dyn FnMut(u32, u32) -> bool) -> Vec<u32> {
    const RUN: usize = 16;
    assert!(
        len <= u32::MAX as usize,
        "sort: more than u32::MAX elements"
    );
    let mut a: Vec<u32> = (0..len as u32).collect();
    for run in a.chunks_mut(RUN) {
        for i in 1..run.len() {
            let x = run[i];
            let mut j = i;
            while j > 0 && less(x, run[j - 1]) {
                run[j] = run[j - 1];
                j -= 1;
            }
            run[j] = x;
        }
    }
    if len <= RUN {
        return a;
    }
    let mut b = vec![0u32; len];
    let mut width = RUN;
    while width < len {
        let mut lo = 0;
        while lo < len {
            let mid = (lo + width).min(len);
            let hi = (lo + 2 * width).min(len);
            // Two runs already in order (a list sorted last frame is):
            // copied as they are, one comparison.
            if mid == hi || !less(a[mid], a[mid - 1]) {
                b[lo..hi].copy_from_slice(&a[lo..hi]);
                lo = hi;
                continue;
            }
            let (mut i, mut j, mut k) = (lo, mid, lo);
            // A right element goes first only when it is strictly less:
            // equal elements keep their order (stability).
            while i < mid && j < hi {
                if less(a[j], a[i]) {
                    b[k] = a[j];
                    j += 1;
                } else {
                    b[k] = a[i];
                    i += 1;
                }
                k += 1;
            }
            b[k..k + mid - i].copy_from_slice(&a[i..mid]);
            k += mid - i;
            b[k..k + hi - j].copy_from_slice(&a[j..hi]);
            lo = hi;
        }
        std::mem::swap(&mut a, &mut b);
        width *= 2;
    }
    a
}

/// Moves `v[order[i]]` to `v[i]` for every `i`, one cycle of the
/// permutation at a time, by swaps.
fn permute<T>(v: &mut [T], mut order: Vec<u32>) {
    for start in 0..v.len() {
        let mut at = start;
        while order[at] as usize != at {
            let from = order[at] as usize;
            order[at] = at as u32;
            if from == start {
                break;
            }
            v.swap(at, from);
            at = from;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Rng;

    /// Every length across the insertion runs and the merge widths, with
    /// many ties, in random, ascending and descending order: the result is
    /// the standard library's stable sort's, element for element (each
    /// carries its original position, so a tie in the wrong order shows).
    #[test]
    fn sorts_as_the_standard_stable_sort_does() {
        let mut rng = Rng::new(7);
        for len in (0..80).chain([127, 128, 129, 255, 256, 257, 1000, 4099]) {
            for keys in [2u32, 7, 1_000_000, 0, 1] {
                let v: Vec<(u32, usize)> = (0..len)
                    .map(|i| match keys {
                        0 => (i as u32 / 3, i),
                        1 => ((len - i) as u32 / 3, i),
                        _ => (rng.next_u32() % keys, i),
                    })
                    .collect();
                let mut ours = v.clone();
                let mut std = v.clone();
                sort_by_key(&mut ours, |e| e.0);
                std.sort_by_key(|e| e.0);
                assert_eq!(ours, std, "len {len}, {keys} keys");
                let mut ours = v.clone();
                let mut std = v;
                sort_by(&mut ours, |a, b| b.0.cmp(&a.0));
                std.sort_by_key(|e| std::cmp::Reverse(e.0));
                assert_eq!(ours, std, "len {len}, {keys} keys, descending");
            }
        }
    }

    #[test]
    fn sorts_values_that_own_memory() {
        let mut v: Vec<String> = ["pear", "fig", "apple", "fig", "kiwi"]
            .map(String::from)
            .to_vec();
        sort(&mut v);
        assert_eq!(v, ["apple", "fig", "fig", "kiwi", "pear"]);
    }
}
