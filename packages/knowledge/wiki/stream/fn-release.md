<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`stream`](../stream.md) · [index](../index.md)

<a id="fn-release"></a>

### `og.stream.release`

Stop asking for a group; it unloads at the next tick boundary unless its trigger still wants it.

```ts
og.stream.release(world: World, group: string): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `group` | `string` | no | The group's name. |

**Returns** `Result<void>`: Refused for a group no scene declares.

Binding symbol `og_stream_release` · publish builds: kept

```ogt
fn release {
  doc "Stop asking for a group; it unloads at the next tick boundary unless its trigger still wants it."
  param world World "The world."
  param group String "The group's name."
  returns Result.Void "Refused for a group no scene declares."
}
```

<sub>`api.ogt` line 25625</sub>
