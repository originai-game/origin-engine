<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`assert`](../assert.md) · [index](../index.md)

<a id="fn-sometimes"></a>

### `og.assert.sometimes`

The condition holds on at least one call in the run; one that never held is listed as uncovered when the run ends.

```ts
og.assert.sometimes(condition: boolean, name: string, details?: unknown | null): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `condition` | `boolean` | no | What must hold at least once. |
| `name` | `string` | no | What it means: `boss reaches phase 2`. |
| `details` | `unknown \| null` | yes | Values worth seeing at the first time it holds. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn sometimes {
  doc "The condition holds on at least one call in the run; one that never held is listed as uncovered when the run ends."
  host false
  param condition bool "What must hold at least once."
  param name String "What it means: `boss reaches phase 2`."
  param details Option.Json "Values worth seeing at the first time it holds." {
    optional true
  }
}
```

<sub>`api.ogt` line 25972</sub>
