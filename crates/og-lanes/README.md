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
- `install_from(site, items, from, f)`: the same from `from` items,
  whatever the pool's threads, for a caller that measured where its
  library pays for each thread count (og-physics's `pool_bodies`, read
  with `threads()`: the threads lanes would use now, 1 when off).
- `serial(site, f)`: work outside the helpers that may reach rayon (a
  physics query: parry's own `rayon::join`), on the calling thread inside
  its one-thread pool, so rayon from `f` runs inline: never rayon's global
  pool, never a wait on another thread. A fault injected at the site fires
  there, on the calling thread, and a panic in `f` is the calling thread's
  (on the web, the engine Worker traps with the structured crash).

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
  Once the relay runs, `og_lanes_ready` puts the engine thread in its own
  one-thread pool for good, so rayon reached from it outside every helper
  runs inline: the relay and its latch are the only way from the engine
  thread onto the pool, and no rayon wait without a panic latch is left
  on it (`serial` covers the same for work before the pool starts, and in
  the single-threaded hosts).
- The single-threaded wasm: nowhere; every helper runs serially.

## Sites

Each call names its site, and `report()` (the probe's `dump_lanes`, the web
host's `og_web_lanes`) counts the parallel calls and their items per site:

| site | caller | grain |
|---|---|---|
| `anim` | og-anim's graph evaluation and joint palettes | 64 skeletons |
| `extract` | og-runtime's row decoding | 512 rows |
| `cull` | og-render's CPU frustum cull | 1024 bounds |
| `physics` | og-physics's whole step through `install_from` (rapier's parallel broad phase, narrow phase and solver), its contact readout through `map`, ropes, cloths and fluid sets stepped side by side and the fluids' per-particle passes through `for_each_mut` and `map`, its queries through `serial` | the step from `og_physics::pool_bodies` (500 bodies on two to four threads, 1,000 on five or six, 1,500 from seven); 512 contact pairs a task once the step is on the pool; the soft bodies' grains in `og_physics::Grains` |

The physics site splits natively and in the lanes wasm alike: the reference
scene's crowd (48 crates) never wakes the pool, a pile of 1,000 boxes on
four lanes does on every step (`packages/engine/tests/lanes.test.ts`).
Measurements: `tests/evidence/P3/lanes/` (the sites, the pool hop),
`tests/evidence/P4/physics-lanes/` (the physics grain) and
`tests/evidence/P4/physics-lanes-2/` (the threshold per thread count, the
soft bodies' grains).

## Tests

`cargo test -p og-lanes`: a lane's panic wakes the waiting thread with its
message, site and tick (the wasm's latch, run natively); a native lane's
panic reaches the caller; an injected fault fires once at its site;
`install` stays on the calling thread below its grain and reaches the pool
from it, `install_from` from its number of items exactly; `serial` runs
rayon inline on a fresh thread and leaves it in its own pool, its fault on
the caller; `threads` follows the switch; the relay reaches its pool from a
thread that belongs to another.
