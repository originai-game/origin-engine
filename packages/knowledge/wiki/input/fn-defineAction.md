<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-defineAction"></a>

### `og.input.defineAction`

Define (or redefine) a gameplay action. A redefinition keeps the player's bindings while they fit it, and its state while its kind stays. Callable anywhere (GameDesc.start, a system); the map is world state, so snapshots carry it.

Also known as: input action, InputAction, Godot InputMap action, key binding

```ts
og.input.defineAction(world: World, desc?: ActionDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `ActionDesc` | yes | The action. |

**Returns** `Result<void>`: Refused, naming the fix, for a binding whose control names none or that does not fit the kind (a stick on a button, a key alone on a two-axis action).

Binding symbol `og_input_define_action` · publish builds: kept

```ogt
fn defineAction {
  doc "Define (or redefine) a gameplay action. A redefinition keeps the player's bindings while they fit it, and its state while its kind stays. Callable anywhere (GameDesc.start, a system); the map is world state, so snapshots carry it."
  aliases "input action, InputAction, Godot InputMap action, key binding"
  param world World "The world."
  param desc ActionDesc "The action."
  returns Result.Void "Refused, naming the fix, for a binding whose control names none or that does not fit the kind (a stick on a button, a key alone on a two-axis action)."
}
```

<sub>Module `input` line 328</sub>
