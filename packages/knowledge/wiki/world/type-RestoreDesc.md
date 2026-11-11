<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="type-RestoreDesc"></a>

### `RestoreDesc` (desc struct)

How world.restore takes a save.

See `saves §5`.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `migrate` | `boolean` | `false` | — | — | — | Map a type whose layout changed since the save by field name, as hot reload does (added fields take defaults, removed ones and unregistered types drop, enums map by variant name, retyped ones reset). Off, such a save is refused, naming the type; the host input the restore keeps is mapped either way. See `saves §5`. |
| `session` | `boolean` | `false` | — | — | — | Take the save's input too (og.Input, og.Pointers, og.Gamepad, og.PointerLock: keys, buttons and pointers held and this tick's edges) instead of keeping the host's, so the world is the one saved, hash and all. A selection keeps the host's input either way. See `saves §4`. |

```ogt
struct RestoreDesc {
  doc "How world.restore takes a save."
  desc true
  guide "saves §5"
  field migrate bool "Map a type whose layout changed since the save by field name, as hot reload does (added fields take defaults, removed ones and unregistered types drop, enums map by variant name, retyped ones reset). Off, such a save is refused, naming the type; the host input the restore keeps is mapped either way." {
    guide "saves §5"
  }
  field session bool "Take the save's input too (og.Input, og.Pointers, og.Gamepad, og.PointerLock: keys, buttons and pointers held and this tick's edges) instead of keeping the host's, so the world is the one saved, hash and all. A selection keeps the host's input either way." {
    guide "saves §4"
  }
}
```

<sub>Module `world` line 33</sub>
