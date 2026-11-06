<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-tabBar"></a>

### `og.ui.tabBar`

Tabs with a sliding indicator and bumpers over the panel of the selected one.

```ts
og.ui.tabBar(id: string, desc: TabBarDesc, build: UiItemBuild): TabBarResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `TabBarDesc` | no | Tabs and the selected one. |
| `build` | `UiItemBuild` | no | Builds the selected tab's panel by its index. |

**Returns** `TabBarResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn tabBar {
  doc "Tabs with a sliding indicator and bumpers over the panel of the selected one."
  host false
  param id String "Its id."
  param desc TabBarDesc "Tabs and the selected one."
  param build UiItemBuild "Builds the selected tab's panel by its index."
  returns TabBarResult
}
```

<sub>`api.ogt` line 18781</sub>
