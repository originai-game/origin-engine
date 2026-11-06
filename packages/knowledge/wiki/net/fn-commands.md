<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`net`](../net.md) · [index](../index.md)

<a id="fn-commands"></a>

### `og.net.commands`

The commands every seat gave for the tick now running, the same list on every peer: by seat, then in the order given, each with `player` set to its seat. Empty without a command shape. Callable from a system.

```ts
og.net.commands<C extends AnyComponent>(world: World, component: C): Result<readonly Value<C>[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `component` | `C` | no | The command component og.net.shape named. |

**Returns** `Result<readonly Value<C>[]>`: Refused for another component than the shape's.

Binding symbol `og_net_commands` · publish builds: kept

```ogt
fn commands {
  doc "The commands every seat gave for the tick now running, the same list on every peer: by seat, then in the order given, each with `player` set to its seat. Empty without a command shape. Callable from a system."
  generic C AnyComponent
  param world World "The world."
  param component C "The command component og.net.shape named."
  returns Result.List.Value.C "Refused for another component than the shape's."
}
```

<sub>`api.ogt` line 26490</sub>
