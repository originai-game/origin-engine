<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-openAnchor"></a>

### `og.ui.openAnchor`

Open an anchored group, closed by close(): og.ui.anchor without the callback.

```ts
og.ui.openAnchor(id: string, desc?: AnchorDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id; the probe path is the id alone. |
| `desc` | `AnchorDesc` | yes | Where it sits and how it lays out. |

**Returns** `void`.

Binding symbol `og_ui_open_anchor` · publish builds: kept

```ogt
fn openAnchor {
  doc "Open an anchored group, closed by close(): og.ui.anchor without the callback."
  param id String "Its id; the probe path is the id alone."
  param desc AnchorDesc "Where it sits and how it lays out."
  returns Void
}
```

<sub>Module `ui` line 1257</sub>
