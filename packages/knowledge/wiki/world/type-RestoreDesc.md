<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="type-RestoreDesc"></a>

### `RestoreDesc` (desc struct)

How world.restore takes a save.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `session` | `boolean` | `false` | — | — | — | Take the save's input too (og.Input, og.Pointers, og.Gamepad, og.PointerLock: keys, buttons and pointers held and this tick's edges) instead of keeping the host's, so the world is the one saved, hash and all. A selection keeps the host's input either way. |

```ogt
struct RestoreDesc {
  doc "How world.restore takes a save."
  desc true
  field session bool "Take the save's input too (og.Input, og.Pointers, og.Gamepad, og.PointerLock: keys, buttons and pointers held and this tick's edges) instead of keeping the host's, so the world is the one saved, hash and all. A selection keeps the host's input either way." {
  }
}
```

<sub>Module `world` line 34</sub>
