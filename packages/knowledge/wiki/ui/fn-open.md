<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-open"></a>

### `og.ui.open`

Open a container; the calls until its close() build its children.

```ts
og.ui.open(id: string, desc?: ElementDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id, unique among its siblings. |
| `desc` | `ElementDesc` | yes | Its layout and look. |

**Returns** `void`.

Binding symbol `og_ui_open` · publish builds: kept

```ogt
fn open {
  doc "Open a container; the calls until its close() build its children."
  param id String "Its id, unique among its siblings."
  param desc ElementDesc "Its layout and look."
  returns Void
}
```

<sub>`api.ogt` line 16256</sub>
