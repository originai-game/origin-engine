<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`assert`](../assert.md) · [index](../index.md)

<a id="fn-always"></a>

### `og.assert.always`

The condition holds on every call: a false one is a violation, reported with its tick, its name, its place and `details`.

Also known as: invariant, assertion, must always hold, Antithesis always

```ts
og.assert.always(condition: boolean, name: string, details?: unknown | null): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `condition` | `boolean` | no | What must hold. |
| `name` | `string` | no | What it means, in a few words: `hp never negative`. |
| `details` | `unknown \| null` | yes | Values worth seeing when it fails, as `{ hp, damage }`. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn always {
  doc "The condition holds on every call: a false one is a violation, reported with its tick, its name, its place and `details`."
  aliases "invariant, assertion, must always hold, Antithesis always"
  host false
  param condition bool "What must hold."
  param name String "What it means, in a few words: `hp never negative`."
  param details Option.Json "Values worth seeing when it fails, as `{ hp, damage }`." {
    optional true
  }
}
```

<sub>`api.ogt` line 25918</sub>
