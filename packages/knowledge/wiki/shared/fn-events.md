<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="fn-events"></a>

### `og.shared.events`

The events delivered this tick, in the order they arrived, each with `from` set to its sender's og.RemoteAvatar when the component has that field. Callable from a system.

```ts
og.shared.events<C extends AnyComponent>(world: World, component: C): Result<readonly Value<C>[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `component` | `C` | no | The event component og.shared.shape named. |

**Returns** `Result<readonly Value<C>[]>`: Refused for another component than the shape's.

Binding symbol `og_shared_events` · publish builds: kept

```ogt
fn events {
  doc "The events delivered this tick, in the order they arrived, each with `from` set to its sender's og.RemoteAvatar when the component has that field. Callable from a system."
  generic C AnyComponent
  param world World "The world."
  param component C "The event component og.shared.shape named."
  returns Result.List.Value.C "Refused for another component than the shape's."
}
```

<sub>Module `shared` line 249</sub>
