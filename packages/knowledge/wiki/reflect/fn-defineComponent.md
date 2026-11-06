<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`reflect`](../reflect.md) · [index](../index.md)

<a id="fn-defineComponent"></a>

### `defineComponent`

Declare a component: plain data stored per entity in engine memory, one column per field.

Registration is by name, so a hot reload re-registers the same component and migrates its fields by name.

```ts
defineComponent<F extends Fields>(name: string, fields: F): ComponentType<F>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The component's unique name, PascalCase, as it appears in scenes and dumps. |
| `fields` | `F` | no | The layout: field names to column factories, as `{ hp: f32(100) }`. |

**Returns** `ComponentType<F>`: The token systems, queries and the world take.

Binding symbol `og_reflect_define_component` · publish builds: kept

```ogt
fn defineComponent {
  doc "Declare a component: plain data stored per entity in engine memory, one column per field."
  doc "Registration is by name, so a hot reload re-registers the same component and migrates its fields by name."
  generic F Fields
  param name String "The component's unique name, PascalCase, as it appears in scenes and dumps."
  param fields F "The layout: field names to column factories, as `{ hp: f32(100) }`."
  returns ComponentType.F "The token systems, queries and the world take."
}
```

<sub>`api.ogt` line 837</sub>
