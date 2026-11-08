<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-space"></a>

### `og.ui.space`

Empty space of a step of the style's spacing scale (so a style's density moves every gap at once), or all the room left. Probe: `<id>`, kind spacer.

```ts
og.ui.space(id: string, desc?: SpaceDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `SpaceDesc` | yes | How much. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn space {
  doc "Empty space of a step of the style's spacing scale (so a style's density moves every gap at once), or all the room left. Probe: `<id>`, kind spacer."
  host false
  param id String "Its id."
  param desc SpaceDesc "How much."
  returns Void
}
```

<sub>Module `ui` line 3270</sub>
