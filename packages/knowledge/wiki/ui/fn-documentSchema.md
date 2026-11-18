<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-documentSchema"></a>

### `og.ui.documentSchema`

Reflected document kinds, attributes and writeback (docs/ui.md §2.1). Anchors expose distance scaling, clamp/containment, LOD, text-floor, reserved HUD/player boxes and opt-in danger avoidance, with placement feedback on their root. stack-group/stack-max/stack-gap form bounded keyed loot columns; a stack-more row's bind:hidden writes ranked overflow keys into a string-list signal. Mass bars accept bar-ghost ink. See guide components for attribute types and details.

```ts
og.ui.documentSchema(): unknown
```

**Returns** `unknown`.

Binding symbol `og_ui_document_schema` · publish builds: kept

```ogt
fn documentSchema {
  doc "Reflected document kinds, attributes and writeback (docs/ui.md §2.1). Anchors expose distance scaling, clamp/containment, LOD, text-floor, reserved HUD/player boxes and opt-in danger avoidance, with placement feedback on their root. stack-group/stack-max/stack-gap form bounded keyed loot columns; a stack-more row's bind:hidden writes ranked overflow keys into a string-list signal. Mass bars accept bar-ghost ink. See guide components for attribute types and details."
  returns Json
}
```

<sub>Module `ui` line 1909</sub>
