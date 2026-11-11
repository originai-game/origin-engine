<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-MouseDesc"></a>

### `MouseDesc` (desc struct)

How the host reads the mouse while the pointer is locked (og.PointerLock): the player's setting, kept by the game with its settings (og.platform.saveSettings). Host only: none of it is world state.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `raw` | `boolean` | `true` | — | — | — | Raw input: ask the browser for the mouse's movement without the OS's pointer acceleration (requestPointerLock with unadjustedMovement); where it refuses, the lock is the plain one (MouseInfo.rawUnsupported). Takes effect at the next lock. See `input §2`. |

```ogt
struct MouseDesc {
  doc "How the host reads the mouse while the pointer is locked (og.PointerLock): the player's setting, kept by the game with its settings (og.platform.saveSettings). Host only: none of it is world state."
  desc true
  field raw bool "Raw input: ask the browser for the mouse's movement without the OS's pointer acceleration (requestPointerLock with unadjustedMovement); where it refuses, the lock is the plain one (MouseInfo.rawUnsupported). Takes effect at the next lock." {
    guide "input §2"
    default true
  }
}
```

<sub>Module `input` line 267</sub>
