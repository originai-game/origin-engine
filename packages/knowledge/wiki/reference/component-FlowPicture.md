<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-FlowPicture"></a>
### `FlowPicture`

The footage filling the surface (`object-cover` or `object-contain`), never blocking pointer input to live 3D scenes underneath: the previous picture sits under it while a dissolve runs.

`import { FlowPicture } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/flow-screen.tsx`).

| Prop | Type | What |
|---|---|---|
| `picture` | `Texture \| 0` | The footage's picture: FlowView.texture, or a cutscene's og.VideoPlayer.texture (0 before its first frame). |
| `previous?` | `Texture \| 0 \| undefined` | The picture dissolving out under it (FlowView.previous; 0: none). |
| `mix?` | `number \| undefined` | How far the dissolve is: the picture's opacity over the previous one (FlowView.mix). |
| `fit?` | `"cover" \| "contain" \| undefined` | How the footage fits the surface (`"cover"` crops to fill; `"contain"` letterboxes/pillarboxes cleanly). Default `"cover"`. |
| `onSkip?` | `((e: UiEvent) => void) \| undefined` | Given, a Skip button at the top corner calls it: a cutscene the player has seen (og.flow.skip, or the game's own). |

Pack slots: `Button.plate`, `Plate.frame`, `Screen.frame`.

Fixtures: `default`, `contain`, `skip`.

Functions beside it, imported by name from `@origingame/engine/ui/components`:

- `computeVideoBox( viewportWidth: number, viewportHeight: number, videoWidth: number, videoHeight: number, fit: "cover" | "contain" = "cover", ): VideoBox`: Computes the rendered image box in normalized surface coordinates (`0..1`) matching CSS `object-cover` or `object-contain`, so hotspots land on exact video features on landscape desktop, ultrawide, and 9:16 mobile portrait.
