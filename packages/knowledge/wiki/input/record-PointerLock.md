<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="record-PointerLock"></a>

### `og.PointerLock` (resource)

Pointer lock for mouse-look (a first- or third-person camera turned by the mouse), as browser games ask for it with requestPointerLock: the game sets `wanted` (a system that writes og.PointerLock), the host hides the cursor and locks the pointer to the play surface, and og.Pointers.movement carries the mouse's movement each tick. Off by default.

While locked, og.Input.pointer and og.Pointers' positions stand still and the mouse's buttons still press. `locked` is the host's session state: a restore keeps it, lockstep peers do not compare it.

See `input §2`.

Token `og.PointerLock`, registered under the schema name `og.PointerLock`; its value type is `PointerLock`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `wanted` | `boolean` | `false` | — | — | — | The game wants the pointer locked and the cursor hidden. |
| `locked` | `boolean` | `false` | — | — | read only | The pointer is locked now, as the host last reported it (the player's Escape releases it while `wanted` stays true; headless runs stay unlocked unless a script injects a pointerLock report). |

```ogt
resource PointerLock {
  doc "Pointer lock for mouse-look (a first- or third-person camera turned by the mouse), as browser games ask for it with requestPointerLock: the game sets `wanted` (a system that writes og.PointerLock), the host hides the cursor and locks the pointer to the play surface, and og.Pointers.movement carries the mouse's movement each tick. Off by default."
  doc "While locked, og.Input.pointer and og.Pointers' positions stand still and the mouse's buttons still press. `locked` is the host's session state: a restore keeps it, lockstep peers do not compare it."
  guide "input §2"
  field wanted bool "The game wants the pointer locked and the cursor hidden."
  field locked bool "The pointer is locked now, as the host last reported it (the player's Escape releases it while `wanted` stays true; headless runs stay unlocked unless a script injects a pointerLock report)." {
    readonly true
  }
}
```

<sub>Module `input` line 1000</sub>
