<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`net`](../net.md) · [index](../index.md)

<a id="fn-command"></a>

### `og.net.command`

Give a command for this peer's player: in lockstep, for the tick inputDelay ticks on (with this tick's input), sent once and read on every peer by og.net.commands at that tick; alone, or with lockstep off, at the next tick. Given while a rollback re-simulates a tick it is ignored (its first run gave it already). Callable from a system.

```ts
og.net.command<C extends AnyComponent>(world: World, component: C, value: Partial<Value<C>>): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `component` | `C` | no | The command component og.net.shape named. |
| `value` | `Partial<Value<C>>` | no | Its fields (`player` is the engine's). |

**Returns** `Result<void>`: Refused with no command shape, for another component, or past maxCommands this tick.

Binding symbol `og_net_command` · publish builds: kept

```ogt
fn command {
  doc "Give a command for this peer's player: in lockstep, for the tick inputDelay ticks on (with this tick's input), sent once and read on every peer by og.net.commands at that tick; alone, or with lockstep off, at the next tick. Given while a rollback re-simulates a tick it is ignored (its first run gave it already). Callable from a system."
  generic C AnyComponent
  param world World "The world."
  param component C "The command component og.net.shape named."
  param value Partial.Value.C "Its fields (`player` is the engine's)."
  returns Result.Void "Refused with no command shape, for another component, or past maxCommands this tick."
}
```

<sub>`api.ogt` line 26430</sub>
