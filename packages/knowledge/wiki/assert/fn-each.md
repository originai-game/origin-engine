<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`assert`](../assert.md) · [index](../index.md)

<a id="fn-each"></a>

### `og.assert.each`

One bucket per distinct key (a level, a weapon, an enemy kind): each bucket the run saw must hold its condition once (Antithesis's SOMETIMES_EACH). Buckets seen that never held are listed as uncovered, and every bucket is in the probe's report, the hints a search of the game's states starts from.

```ts
og.assert.each(key: unknown, condition: boolean, name: string): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `key` | `unknown` | no | The bucket: a string, a number, or a small object of either (`{ level, weapon }`). |
| `condition` | `boolean` | no | What must hold once in the bucket. |
| `name` | `string` | no | What it means: `each level cleared`. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn each {
  doc "One bucket per distinct key (a level, a weapon, an enemy kind): each bucket the run saw must hold its condition once (Antithesis's SOMETIMES_EACH). Buckets seen that never held are listed as uncovered, and every bucket is in the probe's report, the hints a search of the game's states starts from."
  host false
  param key Json "The bucket: a string, a number, or a small object of either (`{ level, weapon }`)."
  param condition bool "What must hold once in the bucket."
  param name String "What it means: `each level cleared`."
}
```

<sub>`api.ogt` line 26009</sub>
