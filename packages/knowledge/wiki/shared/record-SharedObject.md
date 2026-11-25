<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="record-SharedObject"></a>

### `og.SharedObject` (component)

A shared object near this visitor, written by the engine: an entity with an og.Transform and the game's object component (og.shared.shape), spawned when the object comes into sight, rewritten when anyone changes it (og.shared.put), despawned when it leaves sight or is removed. The server owns it: change it with og.shared.put and og.shared.remove, never by writing the entity. Its key: og.shared.object.

See `shared-worlds §5`.

Also known as: networked object, replicated object, door state online, persistent world object, server-owned entity

Token `og.SharedObject`, registered under the schema name `og.SharedObject`; its value type is `SharedObject`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `version` | `number` | `0` | — | — | read only | Its version: 1 when created, one more each change. |

```ogt
component SharedObject {
  doc "A shared object near this visitor, written by the engine: an entity with an og.Transform and the game's object component (og.shared.shape), spawned when the object comes into sight, rewritten when anyone changes it (og.shared.put), despawned when it leaves sight or is removed. The server owns it: change it with og.shared.put and og.shared.remove, never by writing the entity. Its key: og.shared.object."
  aliases "networked object, replicated object, door state online, persistent world object, server-owned entity"
  guide "shared-worlds §5"
  field version u32 "Its version: 1 when created, one more each change." {
    readonly true
  }
}
```

<sub>Module `shared` line 81</sub>
