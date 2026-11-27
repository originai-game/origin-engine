<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-FlowScreen"></a>
### `FlowScreen`

A flow as the player sees it, what the engine mounts for project.ts's `flow`: its picture with dissolve, subtitles lifted clear of active choices, aspect-accurate hotspots (choice & inspect), swipe/drag touch gestures, rich QTEs, and a subtle buffering badge while waiting on network chunks. It sits over a picture, so it resolves in the dark scheme in both schemes.

`import { FlowScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/flow-screen.tsx`).

| Prop | Type | What |
|---|---|---|
| `flow?` | `Entity \| undefined` | The flow it draws; left out, og.flow.current's (project.ts's `flow`). |
| `fit?` | `"cover" \| "contain" \| undefined` | How the footage fits the play surface (`"cover"` default, or `"contain"`). |
| `onSkip?` | `((e: UiEvent) => void) \| undefined` | Optional skip handler shown at the top corner. |

Pack slots: `Button.plate`, `Plate.frame`, `Screen.frame`.

Fixtures: `default`, `contain`, `skip`.
