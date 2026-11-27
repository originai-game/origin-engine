<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-KeyValues"></a>
### `KeyValues`

A table of key and value rows, each with an optional change (a stat comparison). Read only.

`import { KeyValues } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/key-values.tsx`).

| Prop | Type | What |
|---|---|---|
| `rows` | `ReadonlyArray<{ id: string; key: string; value: string; delta: string }>` | The rows, a keyed collection of { id, key, value, delta } (keyValueRows; statCompare when delta is set: + better, - worse). |

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`.
