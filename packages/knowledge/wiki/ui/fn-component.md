<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-component"></a>

### `og.ui.component`

Mount a component document (`type component`) here, inside whatever is open: its props are its viewmodel, its two-way writes come back in the answer's props.

```ts
og.ui.component(id: string, name: string, props?: unknown): UiDocResponse
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | The instance's root id. |
| `name` | `string` | no | The component's name. |
| `props` | `unknown` | yes | Its props: an object. |

**Returns** `UiDocResponse`.

Binding symbol `og_ui_component` · publish builds: kept

```ogt
fn component {
  doc "Mount a component document (`type component`) here, inside whatever is open: its props are its viewmodel, its two-way writes come back in the answer's props."
  param id String "The instance's root id."
  param name String "The component's name."
  param props Json "Its props: an object." {
    optional true
  }
  returns UiDocResponse
}
```

<sub>`api.ogt` line 23750</sub>
