<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`stream`](../stream.md) · [index](../index.md)

<a id="fn-request"></a>

### `og.stream.request`

Ask for a group (any trigger) until og.stream.release: it loads from the next tick boundary, within the budget by its priority.

```ts
og.stream.request(world: World, group: string): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `group` | `string` | no | The group's name. |

**Returns** `Result<void>`: Refused for a group no scene declares.

Binding symbol `og_stream_request` · publish builds: kept

```ogt
fn request {
  doc "Ask for a group (any trigger) until og.stream.release: it loads from the next tick boundary, within the budget by its priority."
  param world World "The world."
  param group String "The group's name."
  returns Result.Void "Refused for a group no scene declares."
}
```

<sub>Module `stream` line 126</sub>
