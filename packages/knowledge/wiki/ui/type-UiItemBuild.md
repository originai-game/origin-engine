<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiItemBuild"></a>

### `UiItemBuild` (callback)

Builds one row or cell of a virtualised list or grid: ui calls made inside nest under it.

Game code supplies this function.

```ts
type UiItemBuild = (index: number) => void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `index` | `number` | no | Its index. |

```ogt
callback UiItemBuild {
  doc "Builds one row or cell of a virtualised list or grid: ui calls made inside nest under it."
  param index u32 "Its index."
}
```

<sub>Module `ui` line 1125</sub>
