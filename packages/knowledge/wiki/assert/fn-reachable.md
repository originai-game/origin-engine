<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`assert`](../assert.md) · [index](../index.md)

<a id="fn-reachable"></a>

### `og.assert.reachable`

This line runs at least once in the run; one that never ran is listed as uncovered.

Also known as: coverage, reached, milestone, progress check, Antithesis reachable

```ts
og.assert.reachable(name: string, details?: unknown | null): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | What reaching it means: `door opened`. |
| `details` | `unknown \| null` | yes | Values worth seeing the first time. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn reachable {
  doc "This line runs at least once in the run; one that never ran is listed as uncovered."
  aliases "coverage, reached, milestone, progress check, Antithesis reachable"
  host false
  param name String "What reaching it means: `door opened`."
  param details Option.Json "Values worth seeing the first time." {
    optional true
  }
}
```

<sub>Module `assert` line 188</sub>
