<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-breadcrumbs"></a>

### `og.ui.breadcrumbs`

The path to where the player is, collapsing its middle into a menu when long.

```ts
og.ui.breadcrumbs(id: string, desc?: BreadcrumbsDesc): BreadcrumbsResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `BreadcrumbsDesc` | yes | The crumbs. |

**Returns** `BreadcrumbsResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn breadcrumbs {
  doc "The path to where the player is, collapsing its middle into a menu when long."
  host false
  param id String "Its id."
  param desc BreadcrumbsDesc "The crumbs."
  returns BreadcrumbsResult
}
```

<sub>Module `ui` line 2934</sub>
