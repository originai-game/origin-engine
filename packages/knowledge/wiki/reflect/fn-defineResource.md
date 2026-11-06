<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`reflect`](../reflect.md) · [index](../index.md)

<a id="fn-defineResource"></a>

### `defineResource`

Declare a resource: a reflected singleton, such as a tuning table. Its fields start at their defaults.

```ts
defineResource<F extends Fields>(name: string, fields: F): ResourceType<F>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The resource's unique name, PascalCase. |
| `fields` | `F` | no | The layout, as for defineComponent. |

**Returns** `ResourceType<F>`: The token `world.resource` and system declarations take.

Binding symbol `og_reflect_define_resource` · publish builds: kept

```ogt
fn defineResource {
  doc "Declare a resource: a reflected singleton, such as a tuning table. Its fields start at their defaults."
  generic F Fields
  param name String "The resource's unique name, PascalCase."
  param fields F "The layout, as for defineComponent."
  returns ResourceType.F "The token `world.resource` and system declarations take."
}
```

<sub>`api.ogt` line 846</sub>
