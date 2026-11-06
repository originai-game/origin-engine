<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Elevation"></a>

### `Elevation` (enum)

The soft shadow a glass surface casts.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | No shadow (with a glass weight: raised). |
| `"raised"` | The hairline alone (an outer one in the light scheme). |
| `"float"` | One soft, short shadow: capsules and menus. |
| `"lift"` | Two deeper shadows: dialogs and sheets. |

```ogt
enum Elevation {
  doc "The soft shadow a glass surface casts."
  variant none "No shadow (with a glass weight: raised)."
  variant raised "The hairline alone (an outer one in the light scheme)."
  variant float "One soft, short shadow: capsules and menus."
  variant lift "Two deeper shadows: dialogs and sheets."
}
```

<sub>`api.ogt` line 22893</sub>
