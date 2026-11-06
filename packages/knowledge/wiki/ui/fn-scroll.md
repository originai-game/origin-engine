<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-scroll"></a>

### `og.ui.scroll`

A vertical scroll container built by `build`: openScroll(id, desc), build(), close().

Also known as: scroll view, ScrollRect, Godot ScrollContainer, scrollable list

```ts
og.ui.scroll(id: string, desc: ScrollDesc, build: UiBuild): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ScrollDesc` | no | Its size and spacing. |
| `build` | `UiBuild` | no | Makes its children. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn scroll {
  doc "A vertical scroll container built by `build`: openScroll(id, desc), build(), close()."
  aliases "scroll view, ScrollRect, Godot ScrollContainer, scrollable list"
  host false
  param id String "Its id."
  param desc ScrollDesc "Its size and spacing."
  param build UiBuild "Makes its children."
  returns Void
}
```

<sub>`api.ogt` line 16295</sub>
