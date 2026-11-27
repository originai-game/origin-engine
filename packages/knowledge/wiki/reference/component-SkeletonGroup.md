<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-SkeletonGroup"></a>
### `SkeletonGroup`

One band of light crosses the whole group: every bone's sheen is the same width (a share of the group's) and starts from the group's edge, so the bones light together as it passes.

`import { SkeletonGroup } from "@origingame/engine/ui/components"` (kit `base`, `packages/engine/src/ui/components/base/skeleton-group.tsx`).

| Prop | Type | What |
|---|---|---|
| `shape` | `string` | What it stands in for: lines (empty), avatar, block, row or card. |
| `items` | `ReadonlyArray<{ id: string }>` | One entry per repeat (rows of a list), each with a stable id. |
| `name` | `string` | What a screen reader says while it loads; empty is Loading. |

Fixtures: `default`.
