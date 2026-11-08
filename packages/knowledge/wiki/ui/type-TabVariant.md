<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TabVariant"></a>

### `TabVariant` (enum)

How the selected tab is marked.

A string: one of the variants below. The zero value is `"underline"`.

| Variant | Doc |
|---|---|
| `"underline"` | An underline that slides between tabs (the default). |
| `"pill"` | A pill behind the tab, on a track. |

```ogt
enum TabVariant {
  doc "How the selected tab is marked."
  variant underline "An underline that slides between tabs (the default)."
  variant pill "A pill behind the tab, on a track."
}
```

<sub>Module `ui` line 1546</sub>
