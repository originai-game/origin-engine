<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="type-StorySaliency"></a>

### `StorySaliency` (enum)

How Yarn node and line groups choose eligible content. All random draws use the checkpointed story RNG fork.

A string: one of the variants below. The zero value is `"first"`.

| Variant | Doc |
|---|---|
| `"first"` | First eligible content in source order. |
| `"best"` | Most specific eligible condition, with source order breaking ties. |
| `"bestLeastRecentlyViewed"` | Least viewed eligible content, then most specific, with source order breaking ties. |
| `"randomBestLeastRecentlyViewed"` | Least viewed, then most specific, with random ties. |
| `"conditionalFirstWeighted"` | Prefer conditional content over fallback content; draw ties by #weight (the engine default). |

```ogt
enum StorySaliency {
  doc "How Yarn node and line groups choose eligible content. All random draws use the checkpointed story RNG fork."
  variant first "First eligible content in source order."
  variant best "Most specific eligible condition, with source order breaking ties."
  variant bestLeastRecentlyViewed "Least viewed eligible content, then most specific, with source order breaking ties."
  variant randomBestLeastRecentlyViewed "Least viewed, then most specific, with random ties."
  variant conditionalFirstWeighted "Prefer conditional content over fallback content; draw ties by #weight (the engine default)."
}
```

<sub>Module `story` line 6</sub>
