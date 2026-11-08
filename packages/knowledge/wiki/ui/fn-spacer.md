<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-spacer"></a>

### `og.ui.spacer`

Empty space.

```ts
og.ui.spacer(id: string, size: Size | null): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `size` | `Size \| null` | no | Its size along both axes; null grows into the free space. |

**Returns** `void`.

Binding symbol `og_ui_spacer` · publish builds: kept

```ogt
fn spacer {
  doc "Empty space."
  param id String "Its id."
  param size Option.Size "Its size along both axes; null grows into the free space."
  returns Void
}
```

<sub>Module `ui` line 1298</sub>
