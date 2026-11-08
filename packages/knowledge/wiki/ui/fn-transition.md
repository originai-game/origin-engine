<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-transition"></a>

### `og.ui.transition`

Content that enters and leaves: fade, slide or scale by the tick from the style's motion. `build` runs while it is visible.

```ts
og.ui.transition(id: string, show: boolean, desc: TransitionDesc, build: UiBuild): Presence
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `show` | `boolean` | no | Whether it should be shown. |
| `desc` | `TransitionDesc` | no | How it moves. |
| `build` | `UiBuild` | no | Makes its content. |

**Returns** `Presence`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn transition {
  doc "Content that enters and leaves: fade, slide or scale by the tick from the style's motion. `build` runs while it is visible."
  host false
  param id String "Its id."
  param show bool "Whether it should be shown."
  param desc TransitionDesc "How it moves."
  param build UiBuild "Makes its content."
  returns Presence
}
```

<sub>Module `ui` line 1250</sub>
