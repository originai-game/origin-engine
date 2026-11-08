<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-abilitySlot"></a>

### `og.ui.abilitySlot`

An ability slot as shipped action bars draw it (Diablo IV, League, Destiny 2, Genshin): the art fills the frame edge to edge and stays the dominant element in every state; one centred numeral per state, the cooldown's seconds (tenths under ten, outlined) over a clockwise darkening of about 60 % (as seen) on the part to come, the part done clean, with a bright leading edge; a flash and a pop the tick it comes ready; charges in a small corner badge (none when the seconds show) with the next one's recharge along the bottom; the cost a small figure on a dark pill while ready; unaffordable, out-of-range, disabled (a red slash) and locked (a lock over dark art) treatments of the art; a soft glow and thin accent edge while active; an ultimate 1.15 times the others with a gold ring filling clockwise and a small percent; a frame of the style's corner with a top-lit rim, an inner shadow and a float shadow; rank pips right under it and the hotkey a quiet label in one neutral ink just below. A press answers on touch and with the pointer.

Parts: AbilitySlot.frame[ready|cooling|disabled|locked|active], AbilitySlot.icon, AbilitySlot.tint[locked|disabled|unaffordable|outOfRange], AbilitySlot.ultimate, AbilitySlot.sweep, AbilitySlot.flash, AbilitySlot.lock, AbilitySlot.slash, AbilitySlot.cost[unaffordable], AbilitySlot.recharge, AbilitySlot.charges, AbilitySlot.edge[active|ultimate], AbilitySlot.pip[filled], AbilitySlot.hotkey. Slots: image (art kind ability). States: group ready (ready, cooling, disabled, locked), group use (idle, active, unaffordable). Sounds: focus (the style's ready chime) the tick it comes ready. Accessibility: button named by name, value its seconds left, Unavailable, Locked or Ready. Probe ids: <id>, <id>/Slot/Time, <id>/Slot/Ultimate/Value, <id>/Slot/Cost/Value, <id>/Slot/ChargeBadge/Charges, <id>/Hotkey.

```ts
og.ui.abilitySlot(id: string, desc?: AbilitySlotDesc): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `AbilitySlotDesc` | yes | The ability's state. |

**Returns** `Response`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn abilitySlot {
  doc "An ability slot as shipped action bars draw it (Diablo IV, League, Destiny 2, Genshin): the art fills the frame edge to edge and stays the dominant element in every state; one centred numeral per state, the cooldown's seconds (tenths under ten, outlined) over a clockwise darkening of about 60 % (as seen) on the part to come, the part done clean, with a bright leading edge; a flash and a pop the tick it comes ready; charges in a small corner badge (none when the seconds show) with the next one's recharge along the bottom; the cost a small figure on a dark pill while ready; unaffordable, out-of-range, disabled (a red slash) and locked (a lock over dark art) treatments of the art; a soft glow and thin accent edge while active; an ultimate 1.15 times the others with a gold ring filling clockwise and a small percent; a frame of the style's corner with a top-lit rim, an inner shadow and a float shadow; rank pips right under it and the hotkey a quiet label in one neutral ink just below. A press answers on touch and with the pointer."
  doc "Parts: AbilitySlot.frame[ready|cooling|disabled|locked|active], AbilitySlot.icon, AbilitySlot.tint[locked|disabled|unaffordable|outOfRange], AbilitySlot.ultimate, AbilitySlot.sweep, AbilitySlot.flash, AbilitySlot.lock, AbilitySlot.slash, AbilitySlot.cost[unaffordable], AbilitySlot.recharge, AbilitySlot.charges, AbilitySlot.edge[active|ultimate], AbilitySlot.pip[filled], AbilitySlot.hotkey. Slots: image (art kind ability). States: group ready (ready, cooling, disabled, locked), group use (idle, active, unaffordable). Sounds: focus (the style's ready chime) the tick it comes ready. Accessibility: button named by name, value its seconds left, Unavailable, Locked or Ready. Probe ids: <id>, <id>/Slot/Time, <id>/Slot/Ultimate/Value, <id>/Slot/Cost/Value, <id>/Slot/ChargeBadge/Charges, <id>/Hotkey."
  host false
  param id String "Its id."
  param desc AbilitySlotDesc "The ability's state."
  returns Response
}
```

<sub>Module `ui` line 5958</sub>
