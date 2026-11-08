<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-popover"></a>

### `og.ui.popover`

A popover beside an element with an arrow at it; Back or a press outside dismisses it.

```ts
og.ui.popover(id: string, open: boolean, desc: PopoverDesc, build: UiBuild): PopoverResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `open` | `boolean` | no | Whether it is open. |
| `desc` | `PopoverDesc` | no | The element, the side and the size. |
| `build` | `UiBuild` | no | Builds its content. |

**Returns** `PopoverResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn popover {
  doc "A popover beside an element with an arrow at it; Back or a press outside dismisses it."
  host false
  param id String "Its id."
  param open bool "Whether it is open."
  param desc PopoverDesc "The element, the side and the size."
  param build UiBuild "Builds its content."
  returns PopoverResult
}
```

<sub>Module `ui` line 1541</sub>
