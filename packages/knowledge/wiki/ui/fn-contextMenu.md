<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-contextMenu"></a>

### `og.ui.contextMenu`

A menu at a point: where a right press or a long press happened.

```ts
og.ui.contextMenu(id: string, open: boolean, at: Vector2, desc?: MenuDesc): MenuChoice
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `open` | `boolean` | no | Whether it is open. |
| `at` | `Vector2` | no | The point, CSS px. |
| `desc` | `MenuDesc` | yes | Entries. |

**Returns** `MenuChoice`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn contextMenu {
  doc "A menu at a point: where a right press or a long press happened."
  host false
  param id String "Its id."
  param open bool "Whether it is open."
  param at Vector2 "The point, CSS px."
  param desc MenuDesc "Entries."
  returns MenuChoice
}
```

<sub>Module `ui` line 2896</sub>
