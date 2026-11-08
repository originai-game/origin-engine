<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="type-RestoreDesc"></a>

### `RestoreDesc` (desc struct)

How world.restore takes a save.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `migrate` | `boolean` | `false` | — | — | — | Map a type whose layout changed since the save field by field, by name, as a hot reload does: a field added since takes its default, a removed one is dropped, an enum value is mapped by its variant's name, a field of another type starts at its default, and a type this build no longer registers is dropped. Off, such a save is refused naming the type and its fields, except for the host's input the restore keeps (og.Input, og.Pointers, og.Gamepad, og.PointerLock, og.Audio), which is mapped either way: a save from before og.Gamepad's eight pads restores without it. |
| `session` | `boolean` | `false` | — | — | — | Take the save's input too (og.Input, og.Pointers, og.Gamepad, og.PointerLock: the keys, buttons and pointers held and this tick's edges) instead of keeping the host's. Off, a loaded game reads the keys the player holds now; on, the world is the one saved, hash and all, which a tool returning to a state needs (og explore, a replay from a snapshot). A selection keeps the host's input either way. |

```ogt
struct RestoreDesc {
  doc "How world.restore takes a save."
  desc true
  field migrate bool "Map a type whose layout changed since the save field by field, by name, as a hot reload does: a field added since takes its default, a removed one is dropped, an enum value is mapped by its variant's name, a field of another type starts at its default, and a type this build no longer registers is dropped. Off, such a save is refused naming the type and its fields, except for the host's input the restore keeps (og.Input, og.Pointers, og.Gamepad, og.PointerLock, og.Audio), which is mapped either way: a save from before og.Gamepad's eight pads restores without it."
  field session bool "Take the save's input too (og.Input, og.Pointers, og.Gamepad, og.PointerLock: the keys, buttons and pointers held and this tick's edges) instead of keeping the host's. Off, a loaded game reads the keys the player holds now; on, the world is the one saved, hash and all, which a tool returning to a state needs (og explore, a replay from a snapshot). A selection keeps the host's input either way."
}
```

<sub>Module `world` line 32</sub>
