# og-ecs

The world: an archetype ECS with structure-of-arrays columns, advanced one
fixed tick at a time, snapshotted and restored whole.

## API map

| Item | Entry points |
|---|---|
| `World`, `WorldDesc { tick_rate: 60, seed: 0 }` | `World::new(&desc)` — `Time`, `Rng` and `TimeScale` are registered (as `og.Time`, `og.Rng`, `og.TimeScale`) and present as resources |
| time scale | `time_scale()`, `scaled_dt()`, `set_time_scale(scale)` (0..=10), `hit_stop(ticks)` (never shortens one in progress) |
| types | `register::<T>()`, `register_schema(Schema)` (host layouts by name), `id::<T>()`, `id_by_name`, `SchemaId::{index, from_index}`, `field::<T>("name")` → `FieldRef`, `schemas_json()` |
| entities | `spawn(bundle)`, `spawn_bytes`, `despawn`, `insert`, `insert_bytes`, `remove`, `remove_id`, `get`, `get_bytes`, `has`, `is_alive`, `components_of`, `entities()` |
| names | `set_name(e, name)`, `clear_name`, `name(e)`, `find(name)`, `names()`, `MAX_NAME` |
| structure | `structural_changes()`: bumped by every operation that may move a column |
| queries | `query(&QueryDesc { read, write, without })` → `QueryIter` of `ArchetypeView`: `entities()`, `read::<T>(field)`, `write::<T>(field)`, `read_bytes`, `write_bytes` |
| resources | `insert_resource`, `insert_resource_bytes`, `resource::<T>()`, `resource_mut::<T>()`, `resource_bytes`, `remove_resource`, `remove_resource_id`, `has_resource`, `has_resource_id`, `time()` |
| events | `send`, `send_bytes`, `pending_events::<T>()`, `pending_events_bytes` |
| schedule | `add_system(SystemDesc { name, reads, writes, queries }, fn(&mut SystemContext) -> Result<()>)`, `step()`, `systems()`, `system_info()` → `SystemInfo { name, reads, writes, queries, matched, query_matched, last_tick, ms, total_ms, runs }`, `set_clock(Some(fn() -> f64))` times each system's run (measurement only; the hosts give `performance.now`-like clocks) |
| state | `add_state_block(name, Box<dyn StateBlock>)`, `snapshot() -> Vec<u8>`, `restore(&[u8])`, `restore_migrating(&[u8]) -> MigrationReport`, `world_hash() -> u64` |

`SystemContext`: `time()`, `time_scale()`, `scaled_dt()`, `set_time_scale`, `hit_stop`, `query(i)`, `resource`, `resource_mut`, `events`,
`send`, `get`, `set`, `spawn`, `insert`, `remove`, `despawn`, `is_alive`,
`name_of`, `find`, and the packed-byte forms by `SchemaId` with the same
declared-access checks: `spawn_bytes`, `insert_bytes`, `remove_id`,
`get_bytes`, `set_bytes`, `resource_bytes`, `set_resource_bytes`,
`has_resource_id`, `events_bytes`, `send_bytes`.

```rust
let pos = world.field::<Position>("p")?;
let vel = world.field::<Velocity>("v")?;
world.add_system(
    SystemDesc::new("move").read::<Velocity>().write::<Position>()
        .query(QueryDesc::new().read::<Velocity>().write::<Position>()),
    move |ctx| {
        let dt = ctx.time().dt;
        for a in ctx.query(0)? {                       // one call per archetype
            let v = a.read::<Vec3>(vel)?;
            for (p, v) in a.write::<Vec3>(pos)?.iter_mut().zip(v.iter()) { *p += *v * dt; }
        }
        Ok(())
    },
)?;
```

## Invariants

- **Storage**: one archetype per component set; one 16-byte-aligned column
  per (component, field), so a field is a dense `&[f32]`/`&[Vec3]`/... (a
  `Float32Array` on the web). Rows follow `ArchetypeView::entities()`.
- **Entities**: `u64` = `generation << 32 | index`; a despawned id stays dead
  (`ErrorKind::DeadHandle`); a refused spawn reserves no id. A slot is retired
  when its generation reaches 2^21 (`og_core::handle::RETIRED_GENERATION`), so
  every id is at most 2^53 - 1 and crosses to JavaScript as an exact
  `number`.
- **Types**: one namespace for components, resources and events; ids follow
  registration order. Engine-owned names are dotted (`og.Time`, `og.Rng`),
  matching the TypeScript tokens. A Rust type adopts a host schema of the same name and
  layout; the same name with another layout is refused. A string field
  (`strN`) and a list field (`u64[8]`) are plain data in components,
  resources and events alike.
- **Values**: every byte entry point (and every typed one) checks the packed
  value with `Schema::check_packed`: a bool byte is 0 or 1, an enum index
  names a variant, a string is zero-padded UTF-8, a list's count is a whole
  number up to its capacity with zeros past it; the reason names the field.
  Column views are raw memory: a view may store any byte in a bool column
  (read as true when non-zero), and snapshots keep it as it is.
- **Access**: registration refuses a query that writes (or reads) a type the
  system did not declare, unknown names, duplicate system names and writes to
  `Time`. At run time every column, resource, event and command is checked
  against the declaration; `Time` is readable by every system.
- **Time scale**: `og.TimeScale` (`scale`, `hitStop`, `elapsed`) is how
  much simulation time the modules that honour it advance per tick
  (`scaled_dt()`: 0 while `hitStop` > 0, else `dt × scale`); the tick itself
  never changes, so every system still runs every tick. Every system may
  read it and call `set_time_scale`/`hit_stop` without declaring it; a
  declared write is refused. After the last system of a tick the world adds
  the tick's scaled dt to `elapsed` and counts `hitStop` down, so a hit stop
  of n started in tick N freezes ticks N..N+n-1. A restored snapshot
  without it reads as the default (scale 1).
- **Ticks**: `step()` delivers pending events, runs systems in registration
  order, applies each system's commands when it returns (a system never sees
  its own structural changes; the next system sees them all), then advances
  `Time`. Events sent in tick N are read in tick N+1. A failing system stops
  the tick with `"tick N, system 'name': reason"`; its commands are dropped,
  earlier systems' effects stay, `Time` does not advance.
- **Commands** apply in the order they were queued. Despawning an entity twice
  in one batch is a no-op the second time. An `insert` or `remove` queued
  after a despawn of the same entity fails when applied (`DeadHandle`) and
  stops the tick like a failing system: the commands before it in the batch
  stay applied, it and the rest are dropped (reserved spawns are freed).
- **Determinism**: iteration is archetype creation order, then row order;
  no hash map is iterated; system closures are `Fn` (state lives in
  resources, so snapshots capture it).
- **Names**: one name per live entity, one live entity per name (another
  holder is refused; naming again renames). A name is non-empty, at most 256
  bytes, has no whitespace, `:`, `;` or `=` and does not start with `#`.
  Despawning takes the name with it. Names are in snapshots and the hash.
- **Structural counter**: `structural_changes()` moves on spawn, despawn, an
  insert that adds a component, remove, restore (commands count when they
  are applied), and never on overwrites, view writes, resources, events,
  names or `Time`. Views stamped with it are valid until it moves. It is
  not world state.
- **System info**: `matched` is the number of entities each query matched
  when the system started in the last tick it ran (after earlier systems'
  commands landed), `reads` excludes what `writes` lists, names in id order.
- **Snapshots** (format 2) hold the type table (names, layout hashes and the
  field table: version, field names, types, enum variants), the allocator
  (generations and free list in reuse order), every archetype with its rows
  in order, every resource, both event queues and the names; the layout is
  in `src/snapshot.rs`. Format 1 snapshots (M1–M5) are still read.
  `restore` maps types by name, requires equal layout hashes, validates
  everything and changes nothing on refusal. Systems are code: restore into
  a world with the same schedule.
- **State blocks** (`StateBlock`: `save`, `decode`, `apply`) are module
  state outside archetypes and resources (a physics engine's contacts and
  solver caches). A world with blocks writes format 3: format 2 plus each
  block's name and bytes in added order. A restore decodes every block
  before changing anything, refuses a block this world did not add, and
  resets (`apply(None)`) a block the snapshot lacks; a migrating restore
  drops the unknown block and warns about the missing one, both by name.
- **World hash**: FNV-1a of the state stream, which is format 1's bytes plus,
  only when an entity is named, the names, then, only when the world has
  state blocks, the blocks: a world without either hashes exactly as in M1 (the bouncing-boxes hash is unchanged), and equal state
  gives an equal hash.
- **Migrating restore** (`restore_migrating`, for hot reload): types by
  name, fields by name; an added field takes its schema default, a removed
  field or type is dropped, a field whose type changed restarts at its
  default, enum values follow their variant names, a string or a list of
  another capacity (the same element type) keeps a value that fits and
  starts one that does not empty, never truncated; entity ids, row order,
  names, `Time` and rng states are kept, archetypes only a dropped type kept
  apart merge in snapshot order, and a resource of a type the snapshot does
  not know keeps its current value. `MigrationReport { migrated, added,
  removed, dropped, warnings }` says what happened; empty (`is_exact()`)
  means the result equals `restore`. Format 1 snapshots migrate only when
  every type fits exactly. Nothing changes on refusal.
- `World` is `Send` (it can move to the engine thread), not `Sync`.

## Tests

`cargo test -p og-ecs`: `tests/churn.rs` (10,000 entities, 40,000 random
structural operations against a shadow model), `tests/snapshot.rs`,
`tests/bouncing_boxes.rs` (3600 ticks twice from one seed, and a halfway
snapshot restored into a fresh world, give the same hash; the world itself is
`tests/support/bouncing_boxes.rs`, shared with `tools/wasm-determinism/`),
`tests/schedule.rs`, `tests/host_schema.rs`, `tests/field_types.rs` (a bool
column through snapshot and hash, mat4 columns, string and list columns through snapshot, hash and a capacity change),
`tests/packed_access.rs` (the `SystemContext` byte calls and their access
checks), `tests/introspection.rs` (`system_info` matched counts, names in
snapshots and the hash, the structural counter), `tests/migrate.rs` (added,
removed, retyped fields, a dropped type, enum variants, events in flight).
Cross-target agreement of the bouncing-boxes hash:
`tools/wasm-determinism/run.sh`.
