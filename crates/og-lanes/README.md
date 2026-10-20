# og-lanes

Data-parallel work inside one system or one frame step, with the same
results as one thread (docs/design.md §2.2). The world, its tables and the
TypeScript systems stay on the engine thread; a lane is a pure function of
one item and of state shared read only, and the helpers return results in
item order, so world hashes and replays are the same with lanes on and off.

## The helpers

- `map`, `for_each_mut`, `for_each_indexed`: `f` for every item of a
  slice. A call splits only from two `grain`s of items (each task takes at
  least a grain); below that it runs serially on the calling thread.
- `install(site, items, grain, f)`: for a library built on rayon that
  splits its own work across every pool thread (rapier's `parallel` build).
  `f` goes to the pool when `items` gives every pool thread a grain;
  otherwise it runs on the calling thread inside a one-thread pool
  (`use_current_thread`), so the library sees one thread, takes its serial
  paths and never reaches rayon's global pool. `f` is told which, so the
  helpers it calls can split along.

## Where the threads come from

- Natively (the addon, `og`): a rayon pool of `OG_LANES` threads (`off`,
  `0` or `1` for none; unset, the cores up to 8).
- The lanes wasm (`+atomics,+bulk-memory`, docs/web-export.md §Lanes):
  wasm-bindgen-rayon's Workers, which the engine Worker starts and announces
  with `og_lanes_ready`. That call starts a relay on the pool, one Worker
  that runs whatever the engine thread sends it (and takes part in the
  rayon work it starts); every parallel call goes through it, since
  `rayon::spawn` reaches the pool its caller belongs to and the engine
  thread belongs to its one-thread pool once a call ran serially. The engine
  thread waits on a latch of og-lanes' own, which a lane's panic also rings.
- The single-threaded wasm: nowhere; every helper runs serially.

## Sites

Each call names its site, and `report()` (the probe's `dump_lanes`, the web
host's `og_web_lanes`) counts the parallel calls and their items per site:

| site | caller | grain |
|---|---|---|
| `anim` | og-anim's graph evaluation and joint palettes | 64 skeletons |
| `extract` | og-runtime's row decoding | 512 rows |
| `cull` | og-render's CPU frustum cull | 1024 bounds |
| `physics` | og-physics's whole step through `install` (rapier's parallel broad phase, narrow phase and solver), and its contact readout through `map` | 200 bodies for every pool thread (`og_physics::BODY_GRAIN`); 512 contact pairs a task once the step is on the pool |

The physics site splits natively and in the lanes wasm alike: the reference
scene's crowd (48 crates) never wakes the pool, a pile of 1,000 boxes on
four lanes does on every step (`packages/engine/tests/lanes.test.ts`).
Measurements: `tests/evidence/P3/lanes/` (the sites, the pool hop) and
`tests/evidence/P4/physics-lanes/` (the physics grain).

## Tests

`cargo test -p og-lanes`: a lane's panic wakes the waiting thread with its
message, site and tick (the wasm's latch, run natively); a native lane's
panic reaches the caller; an injected fault fires once at its site;
`install` stays on the calling thread below its grain and reaches the pool
from it; the relay reaches its pool from a thread that belongs to another.
