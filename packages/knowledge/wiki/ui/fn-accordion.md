<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-accordion"></a>

### `og.ui.accordion`

Sections whose headers open and close their bodies; the sections under one move on the panel spring.

```ts
og.ui.accordion(id: string, desc: AccordionDesc, build: UiItemBuild): AccordionResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `AccordionDesc` | no | Sections and the open ones. |
| `build` | `UiItemBuild` | no | Builds an open section's body by its index. |

**Returns** `AccordionResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn accordion {
  doc "Sections whose headers open and close their bodies; the sections under one move on the panel spring."
  host false
  param id String "Its id."
  param desc AccordionDesc "Sections and the open ones."
  param build UiItemBuild "Builds an open section's body by its index."
  returns AccordionResult
}
```

<sub>Module `ui` line 2930</sub>
