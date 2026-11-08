<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-idi"></a>

### `og.ui.idi`

The id of the index-th of several siblings of one name: `idi("Slot", 3)` is `Slot#3`.

```ts
og.ui.idi(name: string, index: number): string
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The name. |
| `index` | `number` | no | The index. |

**Returns** `string`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn idi {
  doc "The id of the index-th of several siblings of one name: `idi(\"Slot\", 3)` is `Slot#3`."
  host false
  param name String "The name."
  param index u32 "The index."
  returns String
}
```

<sub>Module `ui` line 621</sub>
