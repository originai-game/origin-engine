<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`assert`](../assert.md) · [index](../index.md)

<a id="fn-unreachable"></a>

### `og.assert.unreachable`

This line never runs: running it is a violation.

```ts
og.assert.unreachable(name: string, details?: unknown | null): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | What reaching it would mean: `fell through the floor`. |
| `details` | `unknown \| null` | yes | Values worth seeing when it does. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn unreachable {
  doc "This line never runs: running it is a violation."
  host false
  param name String "What reaching it would mean: `fell through the floor`."
  param details Option.Json "Values worth seeing when it does." {
    optional true
  }
}
```

<sub>`api.ogt` line 25949</sub>
