# r4/raycast-still: the still raycast record back under its pre-merge cost

Base: `origin/main` @ 96e89601 (the merge of 758bdeda, which split
`Hierarchy::refresh` into capture and apply). On the Apple M4 Pro,
`perf_raycast_record_20k` (nothing moving) went from 0.348 ms (73cf1228)
to 0.384 ms (fails its 0.368 ms limit) and the moving case from
1.214 to 1.248 ms.

**Measured in a CPU-only orb** (16 cores, gVisor). Times are relative
only. This is not acceptance evidence; the Mac lane is. No perf baseline
was re-recorded or edited.

## What the still read did after 758bdeda

`perf_raycast_record_20k` raycasts outside a system, so every read is
`drawn` (capture, then all 20,000 og.Mesh rows through `mesh_rows`, as
before the merge) and `compose` (20,000 `world_of`, whose first call
applies). The stages, timed with temporary `Instant`s in this orb (min of
51 reads, default malloc):

| stage                      | 73cf1228 | 96e89601        | this branch     |
|----------------------------|----------|-----------------|-----------------|
| hierarchy read             | 0.117 (refresh) | 0.068 capture + 0.048 apply | 0.068 capture + 0.000 apply |
| compose (20,000 `world_of`)| 0.367    | 0.400 – 0.498   | 0.369 – 0.370   |
| rest of `drawn` (`mesh_rows`) | ~0.88 | ~0.79           | ~0.72 – 0.90    |

What 758bdeda added to a still read:

- **The apply still walked everything.** When every archetype compared
  equal, apply still copied `local_set` of every slot into `was`, cleared
  it, walked the 20,000 entities of `columns` to set `changed = false`,
  and walked the slots again. That is the old refresh's per-entity work,
  now run after `mesh_rows`, which likely evicted the entities and slots
  from the caches; the old refresh walked them right after the compare.
  The eviction is a hypothesis, not measured.
- **`world_of` called `apply()` on each of the 20,000 targets.**
  Each call did `Option::take` on `pending`, a read and a write, and the
  call sat at the head of the function the compose loop calls.

The ECS keeps no change ticks or column versions (`og-ecs` has only
`structural_changes`), so the column compare in capture stays. It is the
only per-row work a still read needs, and it is the compare the old
refresh did too.

## The change

- `Hierarchy::apply` returns before walking anything when nothing
  changed since the slots were last brought to a read: no reparenting,
  the same number of archetypes as at the last apply, and each archetype
  still (its entities and three columns unchanged at every capture since
  that apply, as `Pending` already accumulated them). Every slot, its
  `local_set` and its kept world matrix are then exactly what the walk
  would leave. `changed` is only read inside apply, which sets it for
  every slot before reading it. `loose` is still cleared, as before.
  The archetype count matters because the query skips empty archetypes:
  if the last archetype in query order empties, the remaining archetypes
  all compare still while its entities lost og.Transform.
- `world_of` tests `pending.is_some()` in place and calls `apply`
  (`#[inline(never)]`) only when a read waits.
- `Hierarchy::walked` counts the entities and slots the last apply
  walked, the observable the new test asserts.

Moving reads and partly moving reads take the same path as before. The
record's bytes, frames and world hashes are unchanged: the skipped walk
would have rewritten every slot with the values it already held.

## Tests

- `geometry::tests::the_raycast_record_composes_only_what_moved`: unchanged,
  passes.
- `geometry::tests::a_still_read_walks_no_entity` (new). A tick's first
  raycast and a raycast between ticks walk 0 entities when nothing moved
  since the last read. A move at that read, or at a tick no raycast read,
  walks them. So does emptying the last archetype by removing og.Transform
  from every parented mesh. Each step also checks the record bit for bit
  against a fresh read (`checked`).
  - On 96e89601 (fast path disabled) it fails: `nothing moved`, left 120,
    right 0.
  - Without the archetype-count guard it fails `checked` with a stale
    world matrix. Without the reparenting guard, the existing test fails
    with a stale world matrix.
- `cargo test -p og-runtime` (all), `cargo fmt --check`,
  `cargo clippy -p og-runtime --all-targets -- -D warnings`: pass.

## Perf, this orb

`cargo test -p og-runtime --release --test raycast_record_cost -- --ignored`.
The three test binaries were built once each and run alternately, six
rounds (best-of-50 still reads, best-of-7 moving reads per run). Median
of the six runs, ms:

| malloc            | case   | 73cf1228 | 96e89601 | this branch |
|-------------------|--------|----------|----------|-------------|
| heap resident ¹   | still  | 1.030    | 1.066    | **0.982**   |
| heap resident ¹   | moving | 2.353    | 2.373    | 2.343       |
| glibc default     | still  | 1.856    | 1.159    | 1.673       |
| glibc default     | moving | 3.276    | 2.464    | 2.334       |

¹ `GLIBC_TUNABLES=glibc.malloc.trim_threshold=4000000000:glibc.malloc.mmap_threshold=33554432:glibc.malloc.top_pad=67108864`:
large blocks come from a heap that is never trimmed, so no read pays for
page faults.

Under default glibc malloc in gVisor, a read's multi-megabyte vectors
(`mesh_rows`, the `Drawn` list, the targets) are mmapped or trimmed
depending on the allocation history. The swing is ±0.5 ms on a read
whose hierarchy work is about 0.1 ms. It lands in `mesh_rows`: the stage
timings above show the hierarchy and compose stages ordered the same way
in both modes. With the heap resident, the orb shows what the Mac
showed: 96e89601's still read +3.5 % over 73cf1228. On this branch it is
4.7 % under 73cf1228 and 7.8 % under 96e89601, with moving unchanged.
The Mac's 96e89601 regression matches the resident mode's direction,
not the default mode's. That suggests, but does not prove, that the Mac
behaves like the resident mode. The Mac lane decides.

The WebGL2 spheres gain of 758bdeda (the record's tick start only
captures) is untouched: capture is unchanged, and a moving apply takes
the same path.
