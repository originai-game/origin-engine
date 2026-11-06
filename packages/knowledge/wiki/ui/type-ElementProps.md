<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ElementProps"></a>

### `ElementProps` (desc struct)

Everything an element carries beyond its box (docs/ui.md §6.1): set with og.ui.props right after the element. The zero value is an element in the flow, unmoved and opaque, with no motion.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `transform` | `UiTransform` | `{}` | — | — | — | Its transform after layout. |
| `opacity` | `number` | `1` | 0 … 1 | — | — | Its opacity, multiplied down the tree. |
| `z` | `number` | `0` | — | — | — | Its drawing and hit-testing order among its siblings: higher draws over lower without leaving the flow. |
| `position` | `Position` | `"flow"` | — | — | — | How it is placed. |
| `insets` | `Edges` | `{}` | — | — | — | Its insets, for an absolute or anchored element. |
| `anchorTo` | `string` | `""` | — | — | — | The path of the element an anchored element is placed against. |
| `anchorAt` | `Anchor` | `"topLeft"` | — | — | — | The target's point it sits on. |
| `anchorSelf` | `Anchor` | `"topLeft"` | — | — | — | Its own point that sits there. |
| `hitSlop` | `number` | `0` | — | `cssPx` | — | Extra pointer reach around its box. |
| `layout` | `LayoutMode` | `"flex"` | — | — | — | How it lays out its children. |
| `grid` | `GridSpec` | `{}` | — | — | — | Its tracks and areas, for a grid. |
| `place` | `GridPlace` | `{}` | — | — | — | Its place in a grid parent. |
| `container` | `boolean` | `false` | — | — | — | Its container queries read its own width. |
| `variants` | `readonly UiVariant[]` | `[]` | — | — | — | Prop overrides by breakpoint, orientation and width; later ones win. |
| `overlaps` | `readonly string[]` | `[]` | — | — | — | Paths of elements it overlaps on purpose (an ancestor path covers its subtree; `*` covers everything, for what follows the pointer across the screen: a targeting arc, a dragged ghost). Undeclared overlaps are reported (og.ui.overlaps). |
| `dense` | `boolean` | `false` | — | — | — | Its text and its subtree's are a dense HUD's readouts (strategy, MMO, simulation: docs/ui.md §15.2): og ui check holds them to 12 px at 1080p instead of 18, still growing with the player's text scale and passing the contrast rule. Never for menus, body text or controls. |
| `match` | `string` | `""` | — | — | — | A shared-element key: the element with the same key on the screen being left morphs into this one. |
| `drag` | `DragSpec \| null` | `null` | — | — | — | It moves with a drag. |
| `pinch` | `PinchSpec \| null` | `null` | — | — | — | Two touches scale and turn it. |
| `drop` | `DropSpec \| null` | `null` | — | — | — | It takes drops. |
| `nav` | `NavLinks` | `{}` | — | — | — | Where keys and pads move the focus from it, over the spatial pick. |
| `focusGroup` | `FocusGroup \| null` | `null` | — | — | — | Its focusable controls are one focus group: arrows move among them first; at its edge it wraps or is left; it can remember its last member. |
| `gestures` | `GestureSpec \| null` | `null` | — | — | — | Tap, double tap and long press on it. |
| `cursor` | `Cursor` | `"auto"` | — | — | — | The pointer's shape over it and its subtree; auto takes its parent's. |
| `customize` | `Customize \| null` | `null` | — | — | — | The player can move, scale and fade it in the HUD edit mode (og.ui.setHudEdit); its placement applies on every frame (og.ui.setHudLayout). |
| `motion` | `Transition \| null` | `null` | — | — | — | How its animatable values move when they change; null is at once. |
| `layoutMotion` | `Transition \| null` | `null` | — | — | — | Moves from its last laid-out rectangle to its new one (FLIP); null jumps. |
| `enter` | `PresenceSpec \| null` | `null` | — | — | — | How it arrives. |
| `exit` | `PresenceSpec \| null` | `null` | — | — | — | How it leaves: kept, inert and out of hit tests and the accessibility tree, while it plays after the element leaves the tree. |
| `states` | `readonly StateGroup[]` | `[]` | — | — | — | Its visual state groups. |
| `state` | `readonly StateValue[]` | `[]` | — | — | — | The current state of its groups that are not derived from input. |
| `reduced` | `ReducedMotion \| null` | `null` | — | — | — | Its reduced-motion fallback; null is the style's. |
| `inert` | `boolean` | `false` | — | — | — | It and its subtree take no input. |

```ogt
struct ElementProps {
  doc "Everything an element carries beyond its box (docs/ui.md §6.1): set with og.ui.props right after the element. The zero value is an element in the flow, unmoved and opaque, with no motion."
  desc true
  field transform UiTransform "Its transform after layout."
  field opacity f32 "Its opacity, multiplied down the tree." {
    default 1
    min 0
    max 1
  }
  field z i32 "Its drawing and hit-testing order among its siblings: higher draws over lower without leaving the flow."
  field position Position "How it is placed."
  field insets Edges "Its insets, for an absolute or anchored element."
  field anchorTo String "The path of the element an anchored element is placed against."
  field anchorAt Anchor "The target's point it sits on."
  field anchorSelf Anchor "Its own point that sits there."
  field hitSlop f32 "Extra pointer reach around its box." {
    unit cssPx
  }
  field layout LayoutMode "How it lays out its children."
  field grid GridSpec "Its tracks and areas, for a grid."
  field place GridPlace "Its place in a grid parent."
  field container bool "Its container queries read its own width."
  field variants List.UiVariant "Prop overrides by breakpoint, orientation and width; later ones win."
  field overlaps List.String "Paths of elements it overlaps on purpose (an ancestor path covers its subtree; `*` covers everything, for what follows the pointer across the screen: a targeting arc, a dragged ghost). Undeclared overlaps are reported (og.ui.overlaps)."
  field dense bool "Its text and its subtree's are a dense HUD's readouts (strategy, MMO, simulation: docs/ui.md §15.2): og ui check holds them to 12 px at 1080p instead of 18, still growing with the player's text scale and passing the contrast rule. Never for menus, body text or controls."
  field match String "A shared-element key: the element with the same key on the screen being left morphs into this one."
  field drag Option.DragSpec "It moves with a drag."
  field pinch Option.PinchSpec "Two touches scale and turn it."
  field drop Option.DropSpec "It takes drops."
  field nav NavLinks "Where keys and pads move the focus from it, over the spatial pick."
  field focusGroup Option.FocusGroup "Its focusable controls are one focus group: arrows move among them first; at its edge it wraps or is left; it can remember its last member."
  field gestures Option.GestureSpec "Tap, double tap and long press on it."
  field cursor Cursor "The pointer's shape over it and its subtree; auto takes its parent's."
  field customize Option.Customize "The player can move, scale and fade it in the HUD edit mode (og.ui.setHudEdit); its placement applies on every frame (og.ui.setHudLayout)."
  field motion Option.Transition "How its animatable values move when they change; null is at once."
  field layoutMotion Option.Transition "Moves from its last laid-out rectangle to its new one (FLIP); null jumps."
  field enter Option.PresenceSpec "How it arrives."
  field exit Option.PresenceSpec "How it leaves: kept, inert and out of hit tests and the accessibility tree, while it plays after the element leaves the tree."
  field states List.StateGroup "Its visual state groups."
  field state List.StateValue "The current state of its groups that are not derived from input."
  field reduced Option.ReducedMotion "Its reduced-motion fallback; null is the style's."
  field inert bool "It and its subtree take no input."
}
```

<sub>`api.ogt` line 20640</sub>
