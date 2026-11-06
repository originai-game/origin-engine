<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="record-PointerLock"></a>

### `og.PointerLock` (resource)

Pointer lock for mouse-look (a first- or third-person camera turned by the mouse), as browser games ask for it with requestPointerLock: the game sets `wanted` (a system that writes og.PointerLock), the host hides the cursor and locks the pointer to the play surface, and og.Pointers.movement carries the mouse's movement each tick. Off by default.

On the web the shell reads `wanted` from each frame and asks for the lock on the next press on the canvas (browsers grant it only to a gesture, so the click that starts play takes it), releases it when `wanted` goes false, and reports `locked` from pointerlockchange: the player's Escape releases it, and `locked` goes false while `wanted` stays true, so show a "click to look" prompt then. While locked, og.Input.pointer and og.Pointers' positions stand still (the UI's hover stays where it was) and the mouse's buttons still press. Native runs (og run, og dev) are headless: `locked` stays false unless a script injects `{ kind: "pointerLock", locked: true }`. `locked` is the host's (session state, as og.Input is: a restore keeps it, lockstep peers do not compare it).

Token `og.PointerLock`, registered under the schema name `og.PointerLock`; its value type is `PointerLock`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `wanted` | `boolean` | `false` | — | — | — | The game wants the pointer locked and the cursor hidden. |
| `locked` | `boolean` | `false` | — | — | read only | The pointer is locked now, as the host last reported it. |

```ogt
resource PointerLock {
  doc "Pointer lock for mouse-look (a first- or third-person camera turned by the mouse), as browser games ask for it with requestPointerLock: the game sets `wanted` (a system that writes og.PointerLock), the host hides the cursor and locks the pointer to the play surface, and og.Pointers.movement carries the mouse's movement each tick. Off by default."
  doc "On the web the shell reads `wanted` from each frame and asks for the lock on the next press on the canvas (browsers grant it only to a gesture, so the click that starts play takes it), releases it when `wanted` goes false, and reports `locked` from pointerlockchange: the player's Escape releases it, and `locked` goes false while `wanted` stays true, so show a \"click to look\" prompt then. While locked, og.Input.pointer and og.Pointers' positions stand still (the UI's hover stays where it was) and the mouse's buttons still press. Native runs (og run, og dev) are headless: `locked` stays false unless a script injects `{ kind: \"pointerLock\", locked: true }`. `locked` is the host's (session state, as og.Input is: a restore keeps it, lockstep peers do not compare it)."
  field wanted bool "The game wants the pointer locked and the cursor hidden."
  field locked bool "The pointer is locked now, as the host last reported it." {
    readonly true
  }
}
```

<sub>`api.ogt` line 2953</sub>
