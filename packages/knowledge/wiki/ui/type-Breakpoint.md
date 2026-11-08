<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Breakpoint"></a>

### `Breakpoint` (enum)

The play surface's size class.

A string: one of the variants below. The zero value is `"desktop"`.

| Variant | Doc |
|---|---|
| `"desktop"` | 1024 CSS px wide or more, the shorter side 600 or more. |
| `"tablet"` | The shorter side 600 or more, under 1024 wide. |
| `"phoneLandscape"` | The shorter side under 600, wider than tall. |
| `"phonePortrait"` | The shorter side under 600, taller than wide. |

```ogt
enum Breakpoint {
  doc "The play surface's size class."
  variant desktop "1024 CSS px wide or more, the shorter side 600 or more."
  variant tablet "The shorter side 600 or more, under 1024 wide."
  variant phoneLandscape "The shorter side under 600, wider than tall."
  variant phonePortrait "The shorter side under 600, taller than wide."
}
```

<sub>Module `ui` line 756</sub>
