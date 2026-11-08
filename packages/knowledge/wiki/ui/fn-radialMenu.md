<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-radialMenu"></a>

### `og.ui.radialMenu`

A radial wheel chosen by where the pointer or a stick points. A slice's cost sits under its label, in the danger hue while it cannot be paid; with the labels hidden (a small wheel, a large text size), the centre shows the aimed slice's cost under its name.

```ts
og.ui.radialMenu(id: string, open: boolean, desc?: RadialDesc): RadialResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `open` | `boolean` | no | Whether it is open. |
| `desc` | `RadialDesc` | yes | Slices, aim and release. |

**Returns** `RadialResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn radialMenu {
  doc "A radial wheel chosen by where the pointer or a stick points. A slice's cost sits under its label, in the danger hue while it cannot be paid; with the labels hidden (a small wheel, a large text size), the centre shows the aimed slice's cost under its name."
  host false
  param id String "Its id."
  param open bool "Whether it is open."
  param desc RadialDesc "Slices, aim and release."
  returns RadialResult
}
```

<sub>Module `ui` line 2930</sub>
