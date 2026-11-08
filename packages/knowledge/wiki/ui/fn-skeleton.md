<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-skeleton"></a>

### `og.ui.skeleton`

A skeleton: the neutral fill standing in for content that is loading, with a sheen that passes every 1.6 s (still under reduced motion).

```ts
og.ui.skeleton(id: string, desc?: SkeletonDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `SkeletonDesc` | yes | Width, line height and lines. |

**Returns** `void`.

Binding symbol `og_ui_skeleton` · publish builds: kept

```ogt
fn skeleton {
  doc "A skeleton: the neutral fill standing in for content that is loading, with a sheen that passes every 1.6 s (still under reduced motion)."
  param id String "Its id."
  param desc SkeletonDesc "Width, line height and lines."
  returns Void
}
```

<sub>Module `ui` line 1177</sub>
