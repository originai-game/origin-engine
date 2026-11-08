<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-component"></a>

### `og.ui.component`

Mount a component document (`type component`) here, inside whatever is open: its props are its viewmodel, its two-way writes come back in the answer's props.

```ts
og.ui.component<C extends ComponentName>(id: string, name: C, props?: ComponentProps<C> | null): ComponentResponse<C>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | The instance's root id. |
| `name` | `C` | no | The component's name; a name `og ui types` declared types its props and events. |
| `props` | `ComponentProps<C> \| null` | yes | Its props: an object. |

**Returns** `ComponentResponse<C>`.

Binding symbol `og_ui_component` · publish builds: kept

```ogt
fn component {
  doc "Mount a component document (`type component`) here, inside whatever is open: its props are its viewmodel, its two-way writes come back in the answer's props."
  generic C ComponentName
  param id String "The instance's root id."
  param name C "The component's name; a name `og ui types` declared types its props and events."
  param props Option.ComponentProps.C "Its props: an object." {
    optional true
  }
  returns ComponentResponse.C
}
```

<sub>Module `ui` line 8101</sub>
