<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-scrollArea"></a>

### `og.ui.scrollArea`

A scroll container whose content fades out at an edge where more lies beyond.

```ts
og.ui.scrollArea(id: string, desc: ScrollAreaDesc, build: UiBuild): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ScrollAreaDesc` | no | Axes, size and fades. |
| `build` | `UiBuild` | no | Builds its content. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn scrollArea {
  doc "A scroll container whose content fades out at an edge where more lies beyond."
  host false
  param id String "Its id."
  param desc ScrollAreaDesc "Axes, size and fades."
  param build UiBuild "Builds its content."
  returns Void
}
```

<sub>Module `ui` line 2829</sub>
