<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`stream`](../stream.md) · [index](../index.md)

<a id="fn-status"></a>

### `og.stream.status`

One streaming group's state.

```ts
og.stream.status(world: World, group: string): Result<StreamStatus>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `group` | `string` | no | The group's name, as its scene declares it. |

**Returns** `Result<StreamStatus>`: Refused for a group no scene declares.

Binding symbol `og_stream_status` · publish builds: kept

```ogt
fn status {
  doc "One streaming group's state."
  param world World "The world."
  param group String "The group's name, as its scene declares it."
  returns Result.StreamStatus "Refused for a group no scene declares."
}
```

<sub>`api.ogt` line 25597</sub>
