<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="fn-send"></a>

### `og.shared.send`

Send an event to the visitors near this one (those whose sight holds this avatar): reliable, at most 1 KiB, rate-limited by the server. They read it with og.shared.events at the start of the tick after it arrives.

```ts
og.shared.send<C extends AnyComponent>(world: World, component: C, value: Partial<Value<C>>): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `component` | `C` | no | The event component og.shared.shape named. |
| `value` | `Partial<Value<C>>` | no | Its fields; omitted ones take their defaults (`from` is the engine's). |

**Returns** `Result<void>`: Refused without a joined session, without an event shape, or for another component.

Binding symbol `og_shared_send` · publish builds: kept

```ogt
fn send {
  doc "Send an event to the visitors near this one (those whose sight holds this avatar): reliable, at most 1 KiB, rate-limited by the server. They read it with og.shared.events at the start of the tick after it arrives."
  generic C AnyComponent
  param world World "The world."
  param component C "The event component og.shared.shape named."
  param value Partial.Value.C "Its fields; omitted ones take their defaults (`from` is the engine's)."
  returns Result.Void "Refused without a joined session, without an event shape, or for another component."
}
```

<sub>Module `shared` line 193</sub>
