<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Spinner"></a>
### `Spinner`

A ring that is always there and an arc that runs round it. `size` is its width; a host that holds nothing else while it waits (a portrait, a slot) sets `--spinner-size` on the box round it instead, so the ring is in proportion to it.

`import { Spinner } from "@origingame/engine/ui/components"` (kit `base`, `packages/engine/src/ui/components/base/spinner.tsx`).

| Prop | Type | What |
|---|---|---|
| `size?` | `number` | Its width and height, CSS px. Default 28. |
| `name` | `string` | What a screen reader calls it; empty is Loading. |

Fixtures: `default`.
