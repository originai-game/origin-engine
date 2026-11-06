<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-rebind"></a>

### `og.input.rebind`

Rebind one slot of an action (or one part of a composite) for the player. With policy refuse (the default) a binding that shares a control with another action of its context changes nothing and answers the conflicts; swap, replace and allow resolve them. The player's bindings stay until reset and are what og.input.saveBindings writes.

```ts
og.input.rebind(world: World, desc?: RebindDesc): Result<RebindAnswer>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `RebindDesc` | yes | The action, slot, part, binding and policy. |

**Returns** `Result<RebindAnswer>`: Refused for an unknown or locked action, a slot past one after the last, a part the kind has not, or a binding that does not fit.

Binding symbol `og_input_rebind` · publish builds: kept

```ogt
fn rebind {
  doc "Rebind one slot of an action (or one part of a composite) for the player. With policy refuse (the default) a binding that shares a control with another action of its context changes nothing and answers the conflicts; swap, replace and allow resolve them. The player's bindings stay until reset and are what og.input.saveBindings writes."
  param world World "The world."
  param desc RebindDesc "The action, slot, part, binding and policy."
  returns Result.RebindAnswer "Refused for an unknown or locked action, a slot past one after the last, a part the kind has not, or a binding that does not fit."
}
```

<sub>`api.ogt` line 2365</sub>
