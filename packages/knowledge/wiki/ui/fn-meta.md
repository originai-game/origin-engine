<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-meta"></a>

### `og.ui.meta`

A meta layer this tick: a damage vignette, a low-health pulse, a material over the whole frame.

```ts
og.ui.meta(id: string, desc?: MetaDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `MetaDesc` | yes | What it draws. |

**Returns** `void`.

Binding symbol `og_ui_meta` · publish builds: kept

```ogt
fn meta {
  doc "A meta layer this tick: a damage vignette, a low-health pulse, a material over the whole frame."
  param id String "Its id."
  param desc MetaDesc "What it draws."
  returns Void
}
```

<sub>Module `ui` line 7974</sub>
