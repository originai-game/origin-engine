<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-release"></a>

### `og.load.release`

Free an uncooked load's memory, cancelling it if pending; its handle then answers failed, naming the release. Releasing again does nothing. Close an unused runtime bundle after its groups unload and live references are gone. In a system the close is pending until the next tick boundary; between ticks it can finish immediately. Refusal leaves it open with status failed and error starting 'bundle release refused:'; remove the references and retry. Success answers failed, naming the release. Closing changes no world hash, events or completion journal. The boot bundle cannot close.

See `loading §10`.

```ts
og.load.release(world: World, asset: Asset): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `asset` | `Asset` | no | An uncooked asset handle or a runtime bundle handle. |

**Returns** `Result<void>`: Accepts a bundle close request; poll status and error for its outcome. Refused for an individual cooked asset, which stays with its bundle (a streamed one with its group).

Binding symbol `og_load_release` · publish builds: kept

```ogt
fn release {
  doc "Free an uncooked load's memory, cancelling it if pending; its handle then answers failed, naming the release. Releasing again does nothing. Close an unused runtime bundle after its groups unload and live references are gone. In a system the close is pending until the next tick boundary; between ticks it can finish immediately. Refusal leaves it open with status failed and error starting 'bundle release refused:'; remove the references and retry. Success answers failed, naming the release. Closing changes no world hash, events or completion journal. The boot bundle cannot close."
  guide "loading §10"
  param world World "The world that owns the asset."
  param asset Asset "An uncooked asset handle or a runtime bundle handle."
  returns Result.Void "Accepts a bundle close request; poll status and error for its outcome. Refused for an individual cooked asset, which stays with its bundle (a streamed one with its group)."
}
```

<sub>Module `load` line 640</sub>
