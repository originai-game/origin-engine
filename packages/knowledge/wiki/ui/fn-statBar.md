<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-statBar"></a>

### `og.ui.statBar`

A health or resource bar to the bar of action games: blocks (segment), a ghost chunk that holds after a hit and then drains (Dark Souls, fighting games), a heal preview, an absorb overlay (World of Warcraft's white-blue), grey recoverable damage, the value inside its end or after it, a row of shield cells in their tier's colour over it (Apex Legends), low and critical states (warn, then danger pulsing), a mirrored fill for the right player, flat, slanted or pill ends, or discrete pips with a gap (style pips: n of max lit, the value after them). A drop is instant, a rise moves on the ui spring.

Parts: StatBar.frame[<style>][<level>], StatBar.track, StatBar.fill[<kind>][<level>], StatBar.ghost, StatBar.heal, StatBar.absorb[over], StatBar.recoverable, StatBar.block[filled], StatBar.cell[filled][<tier>], StatBar.track[pips], StatBar.pip[filled], StatBar.label, StatBar.value, StatBar.icon. States: group level (normal, low, critical, empty). Sounds: warning on crossing into critical. Accessibility: meter named by name, valueNow the value, valueMax max. Probe ids: <id>, <id>/Track, <id>/Track/Fill, <id>/Track/Ghost, <id>/Track/Block#i, <id>/Shield/Cell#i, <id>/Main/Pips/Pip#i (style pips), <id>/Main/Icon, <id>/Value (inside: <id>/Main/Track/Value/Text, outlined over a soft dark band that fades at both ends, with no absorb seam under it).

```ts
og.ui.statBar(id: string, desc?: StatBarDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `StatBarDesc` | yes | The value and the look. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn statBar {
  doc "A health or resource bar to the bar of action games: blocks (segment), a ghost chunk that holds after a hit and then drains (Dark Souls, fighting games), a heal preview, an absorb overlay (World of Warcraft's white-blue), grey recoverable damage, the value inside its end or after it, a row of shield cells in their tier's colour over it (Apex Legends), low and critical states (warn, then danger pulsing), a mirrored fill for the right player, flat, slanted or pill ends, or discrete pips with a gap (style pips: n of max lit, the value after them). A drop is instant, a rise moves on the ui spring."
  doc "Parts: StatBar.frame[<style>][<level>], StatBar.track, StatBar.fill[<kind>][<level>], StatBar.ghost, StatBar.heal, StatBar.absorb[over], StatBar.recoverable, StatBar.block[filled], StatBar.cell[filled][<tier>], StatBar.track[pips], StatBar.pip[filled], StatBar.label, StatBar.value, StatBar.icon. States: group level (normal, low, critical, empty). Sounds: warning on crossing into critical. Accessibility: meter named by name, valueNow the value, valueMax max. Probe ids: <id>, <id>/Track, <id>/Track/Fill, <id>/Track/Ghost, <id>/Track/Block#i, <id>/Shield/Cell#i, <id>/Main/Pips/Pip#i (style pips), <id>/Main/Icon, <id>/Value (inside: <id>/Main/Track/Value/Text, outlined over a soft dark band that fades at both ends, with no absorb seam under it)."
  host false
  param id String "Its id."
  param desc StatBarDesc "The value and the look."
  returns Void
}
```

<sub>`api.ogt` line 21714</sub>
