<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-memory"></a>

### `og.ui.memory`

A number the UI keeps for an id under the open element (a component's tab, a count-up's start tick), in snapshots like the rest of the UI state. Read it every tick it is used: an unused memory is dropped.

```ts
og.ui.memory(id: string, initial: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | The id, unique among its siblings' memories. |
| `initial` | `number` | no | Its value the first time. |

**Returns** `number`.

Binding symbol `og_ui_memory` · publish builds: kept

```ogt
fn memory {
  doc "A number the UI keeps for an id under the open element (a component's tab, a count-up's start tick), in snapshots like the rest of the UI state. Read it every tick it is used: an unused memory is dropped."
  param id String "The id, unique among its siblings' memories."
  param initial f64 "Its value the first time."
  returns f64
}
```

<sub>`api.ogt` line 17548</sub>
