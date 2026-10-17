# og-ecs

The world: an archetype ECS with structure-of-arrays columns, advanced one
fixed tick at a time, snapshotted and restored whole.

## API map

| Item | Entry points |
|---|---|
| `World`, `WorldDesc { tick_rate: 60, seed: 0 }` | `World::new(&desc)` — `Time` and `Rng` are registered and present as resources |
| types | `register::<T>()`, `register_schema(Schema)` (host layouts by name), `id::<T>()`, `id_by_name`, `field::<T>("name")` → `FieldRef`, `schemas_json()` |
| entities | `spawn(bundle)`, `spawn_bytes`, `despawn`, `insert`, `insert_bytes`, `remove`, `remove_id`, `get`, `get_bytes`, `has`, `is_alive`, `components_of`, `entities()` |
| queries | `query(&QueryDesc { read, write, without })` → `QueryIter` of `ArchetypeView`: `entities()`, `read::<T>(field)`, `write::<T>(field)`, `read_bytes`, `write_bytes` |
| resources | `insert_resource`, `insert_resource_bytes`, `resource::<T>()`, `resource_mut::<T>()`, `resource_bytes`, `remove_resource`, `time()` |
| events | `send`, `send_bytes`, `pending_events::<T>()` |
| schedule | `add_system(SystemDesc { name, reads, writes, queries }, fn(&mut SystemContext) -> Result<()>)`, `step()` |
| state | `snapshot() -> Vec<u8>`, `restore(&[u8])`, `world_hash() -> u64` |

`SystemContext`: `time()`, `query(i)`, `resource`, `resource_mut`, `events`,
`send`, `get`, `set`, `spawn`, `insert`, `remove`, `despawn`, `is_alive`.

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
  (`ErrorKind::DeadHandle`); a refused spawn reserves no id.
- **Types**: one namespace for components, resources and events; ids follow
  registration order. A Rust type adopts a host schema of the same name and
  layout; the same name with another layout is refused.
- **Access**: registration refuses a query that writes (or reads) a type the
  system did not declare, unknown names, duplicate system names and writes to
  `Time`. At run time every column, resource, event and command is checked
  against the declaration; `Time` is readable by every system.
- **Ticks**: `step()` delivers pending events, runs systems in registration
  order, applies each system's commands when it returns (a system never sees
  its own structural changes; the next system sees them all), then advances
  `Time`. Events sent in tick N are read in tick N+1. A failing system stops
  the tick with `"tick N, system 'name': reason"`; its commands are dropped,
  earlier systems' effects stay, `Time` does not advance.
- **Determinism**: iteration is archetype creation order, then row order;
  no hash map is iterated; system closures are `Fn` (state lives in
  resources, so snapshots capture it).
- **Snapshots** hold the type table (names, layout hashes), the allocator
  (generations and free list in reuse order), every archetype with its rows in
  order, every resource and both event queues. `restore` maps types by name,
  requires equal layout hashes, validates everything and changes nothing on
  refusal. `world_hash` is FNV-1a of the snapshot bytes. Systems are code:
  restore into a world with the same schedule.
- `World` is `Send` (it can move to the engine thread), not `Sync`.

## Tests

`cargo test -p og-ecs`: `tests/churn.rs` (10,000 entities, 40,000 random
structural operations against a shadow model), `tests/snapshot.rs`,
`tests/bouncing_boxes.rs` (3600 ticks twice from one seed, and a halfway
snapshot restored into a fresh world, give the same hash),
`tests/schedule.rs`, `tests/host_schema.rs`.
