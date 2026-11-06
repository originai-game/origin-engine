<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`net`](../net.md) · [index](../index.md)

<a id="fn-submitInput"></a>

### `og.net.submitInput`

Give this peer's player's input when og.net.shape named an input component: its fields as og.net.submit's (`player` and `tick` are the engine's). It holds until the next call. Callable from a system.

```ts
og.net.submitInput<C extends AnyComponent>(world: World, component: C, value: Partial<Value<C>>): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `component` | `C` | no | The input component og.net.shape named. |
| `value` | `Partial<Value<C>>` | no | Its fields; omitted ones take their defaults. |

**Returns** `Result<void>`: Refused for another component than the shape's, or a field that does not fit.

Binding symbol `og_net_submit_input` · publish builds: kept

```ogt
fn submitInput {
  doc "Give this peer's player's input when og.net.shape named an input component: its fields as og.net.submit's (`player` and `tick` are the engine's). It holds until the next call. Callable from a system."
  generic C AnyComponent
  param world World "The world."
  param component C "The input component og.net.shape named."
  param value Partial.Value.C "Its fields; omitted ones take their defaults."
  returns Result.Void "Refused for another component than the shape's, or a field that does not fit."
}
```

<sub>`api.ogt` line 26464</sub>
