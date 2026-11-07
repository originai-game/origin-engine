<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ElementDesc"></a>

### `ElementDesc` (desc struct)

A container: its layout and its look.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `direction` | `Direction` | `"row"` | — | — | — | Which way it lays out its children. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits the content. |
| `height` | `Size \| null` | `null` | — | — | — | Its height; null fits the content. |
| `padding` | `Insets` | `{ left: 0, top: 0, right: 0, bottom: 0 }` | — | — | — | Space inside its edges. |
| `gap` | `number` | `0` | — | `cssPx` | — | Space between children. |
| `align` | `Align` | `"start"` | — | — | — | Where children sit along the main axis. |
| `cross` | `Cross` | `"start"` | — | — | — | Where children sit across it. |
| `wrap` | `boolean` | `false` | — | — | — | Break a row into lines at its inner width. |
| `scroll` | `boolean` | `false` | — | — | — | Scroll its content vertically and clip it. |
| `scrollX` | `boolean` | `false` | — | — | — | Scroll its content horizontally and clip it (with scroll: both ways). |
| `interactive` | `boolean` | `false` | — | — | — | Report a Response from close(), as a button does. |
| `focusable` | `boolean` | `false` | — | — | — | With interactive: keys and pads reach it (a card, a menu item, a component's tab), it takes the focus ring, and Enter, Space and the pad's A press it, as a button. Its role is button unless og.ui.semantics gives another. |
| `adjustable` | `boolean` | `false` | — | — | — | With focusable: Left and Right (a pad's left and right) are its own while it has the focus: they step it (Response.adjust) instead of moving the focus, as a slider's do (a menu row opening its submenu, a carousel, a divider). |
| `arrows` | `ArrowClaim` | `{}` | — | — | — | With focusable: the arrows (a pad's d-pad and stick) it claims one by one while it has the focus: each steps it (Response.adjust for Left and Right, Response.adjustY for Up and Down) instead of moving the focus. A grid's own cursor (an inventory, a level map) claims the ones that keep it inside, so the focus leaves it at its edges. |
| `ownRing` | `boolean` | `false` | — | — | — | With focusable: the component draws its own focus (a hand rings its focused card, a grid its cursor cell) while Response.ring says the ring is up; og-ui draws no ring around the element. |
| `background` | `Paint \| null` | `null` | — | — | — | Its fill; null draws none. |
| `radius` | `number` | `0` | — | `cssPx` | — | Corner radius. |
| `frame` | `StylePart \| null` | `null` | — | — | — | The style frame its background takes (a HUD element, a card); null is a plain fill. Without a style the fill is drawn either way. |
| `part` | `string \| null` | `null` | — | — | — | The part its style dresses it as, a selector with its variant, state and attributes (`ShopCard.frame[rarity=epic][selected]`): the style's most specific matching `dress` blocks draw it, and a part the style does not dress draws from the tokens (docs/ui.md §3.2). Overrides frame; null keeps frame's. |
| `partClass` | `string \| null` | `null` | — | — | — | The element class whose corner, outline and elevation an undressed part takes from the tokens: panel, dialog, card, tooltip, button, field, chip, badge, tab, slot, bar, hud, avatar. Null: a decoration (a scrim, a sweep, a node's art), which takes nothing from the tokens, only what a style dresses it with; a part that is a card or a panel names its class. |
| `aspect` | `number` | `0` | ≥ 0 | — | — | Width over height (0: none): with the height left to fit, it is as tall as its laid-out width over this; with a fixed height and the width left to fit, as wide as the height times it. |

```ogt
struct ElementDesc {
  doc "A container: its layout and its look."
  desc true
  field direction Direction "Which way it lays out its children."
  field width Option.Size "Its width; null fits the content."
  field height Option.Size "Its height; null fits the content."
  field padding Insets "Space inside its edges."
  field gap f32 "Space between children." {
    unit cssPx
  }
  field align Align "Where children sit along the main axis."
  field cross Cross "Where children sit across it."
  field wrap bool "Break a row into lines at its inner width."
  field scroll bool "Scroll its content vertically and clip it."
  field scrollX bool "Scroll its content horizontally and clip it (with scroll: both ways)."
  field interactive bool "Report a Response from close(), as a button does."
  field focusable bool "With interactive: keys and pads reach it (a card, a menu item, a component's tab), it takes the focus ring, and Enter, Space and the pad's A press it, as a button. Its role is button unless og.ui.semantics gives another."
  field adjustable bool "With focusable: Left and Right (a pad's left and right) are its own while it has the focus: they step it (Response.adjust) instead of moving the focus, as a slider's do (a menu row opening its submenu, a carousel, a divider)."
  field arrows ArrowClaim "With focusable: the arrows (a pad's d-pad and stick) it claims one by one while it has the focus: each steps it (Response.adjust for Left and Right, Response.adjustY for Up and Down) instead of moving the focus. A grid's own cursor (an inventory, a level map) claims the ones that keep it inside, so the focus leaves it at its edges."
  field ownRing bool "With focusable: the component draws its own focus (a hand rings its focused card, a grid its cursor cell) while Response.ring says the ring is up; og-ui draws no ring around the element."
  field background Option.Paint "Its fill; null draws none."
  field radius f32 "Corner radius." {
    unit cssPx
  }
  field frame Option.StylePart "The style frame its background takes (a HUD element, a card); null is a plain fill. Without a style the fill is drawn either way."
  field part Option.String "The part its style dresses it as, a selector with its variant, state and attributes (`ShopCard.frame[rarity=epic][selected]`): the style's most specific matching `dress` blocks draw it, and a part the style does not dress draws from the tokens (docs/ui.md §3.2). Overrides frame; null keeps frame's."
  field partClass Option.String "The element class whose corner, outline and elevation an undressed part takes from the tokens: panel, dialog, card, tooltip, button, field, chip, badge, tab, slot, bar, hud, avatar. Null: a decoration (a scrim, a sweep, a node's art), which takes nothing from the tokens, only what a style dresses it with; a part that is a card or a panel names its class."
  field aspect f32 "Width over height (0: none): with the height left to fit, it is as tall as its laid-out width over this; with a fixed height and the width left to fit, as wide as the height times it." {
    min 0
  }
}
```

<sub>Module `ui` line 216</sub>
