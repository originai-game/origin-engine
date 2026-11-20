<meta name="component" content="RpgSkills">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgSkills.frame RpgSkills.skill">
<meta name="prop:slots" content="list" data-doc="The bar's slots in the game's order, keyed rows of { id, name, icon, glyph, key, kind, cooldown, time, count, ready, unaffordable, reason, charges, chargesMax, charge }: name for a reader; icon a texture (0: the glyph, an icon name, in its place); key the input as the game names it (1, Q), on a key cap under the slot; kind skill, item (a consumable: a smaller slot with its count on a tag, set apart before the skills; a phone leaves items out) or ultimate (a larger hexagon inside a gold ring that fills with charge, 0 to 1, its share in figures until full; on data-kind); cooldown the share still to run, 0 to 1 (a dark radial sweep over the art with a lit leading edge), with time its figure as the game words it (8); count the items held (×5; 0 hides it); ready lit (a lit edge and glow); unaffordable it cannot be paid for (the art greys under a veil and reason, as the game words it (No mana), stands on it); chargesMax 0 hides the charge pips, else charges of them are lit over the slot.">
<meta name="fixture:default" content='{"slots":[{"id":"tonic","name":"Healing tonic","icon":0,"glyph":"potion","key":"Z","kind":"item","cooldown":0,"time":"","count":5,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"ether","name":"Ether","icon":0,"glyph":"drop","key":"X","kind":"item","cooldown":0,"time":"","count":2,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"slash","name":"Ember slash","icon":0,"glyph":"sword","key":"1","kind":"skill","cooldown":0,"time":"","count":0,"ready":true,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"frost","name":"Frost lance","icon":0,"glyph":"bolt","key":"2","kind":"skill","cooldown":0.65,"time":"8","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"arc","name":"Arc step","icon":0,"glyph":"star","key":"3","kind":"skill","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":false,"reason":"","charges":2,"chargesMax":3,"charge":0},{"id":"void","name":"Void rend","icon":0,"glyph":"skull","key":"4","kind":"skill","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":true,"reason":"No mana","charges":0,"chargesMax":0,"charge":0},{"id":"gale","name":"Gale","icon":0,"glyph":"map","key":"Q","kind":"skill","cooldown":0.18,"time":"2","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"ult","name":"Cataclysm","icon":0,"glyph":"fire","key":"R","kind":"ultimate","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":1}]}'>
<meta name="fixture:cooling" content='{"slots":[{"id":"slash","name":"Ember slash","icon":0,"glyph":"sword","key":"1","kind":"skill","cooldown":1,"time":"12","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"arc","name":"Arc step","icon":0,"glyph":"star","key":"3","kind":"skill","cooldown":0.4,"time":"3","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":3,"charge":0},{"id":"ult","name":"Cataclysm","icon":0,"glyph":"fire","key":"R","kind":"ultimate","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0.35}]}'>
<meta name="fixture:unaffordable" content='{"slots":[{"id":"void","name":"Void rend","icon":0,"glyph":"skull","key":"4","kind":"skill","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":true,"reason":"No mana","charges":0,"chargesMax":0,"charge":0},{"id":"tonic","name":"Healing tonic","icon":0,"glyph":"potion","key":"Z","kind":"item","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":true,"reason":"","charges":0,"chargesMax":0,"charge":0}]}'>
<meta name="fixture:empty" content='{"slots":[]}'>
<meta name="fixture:overflow" content='{"slots":[{"id":"a","name":"A","icon":0,"glyph":"potion","key":"Left Shoulder","kind":"item","cooldown":0,"time":"","count":12345,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"b","name":"B","icon":0,"glyph":"bolt","key":"Right Trigger","kind":"skill","cooldown":0.5,"time":"1:02:44","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"c","name":"C","icon":0,"glyph":"skull","key":"Dpad Up","kind":"skill","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":true,"reason":"Not enough soul essence","charges":0,"chargesMax":0,"charge":0},{"id":"d","name":"D","icon":0,"glyph":"star","key":"Y","kind":"skill","cooldown":0,"time":"","count":0,"ready":true,"unaffordable":false,"reason":"","charges":7,"chargesMax":9,"charge":0},{"id":"e","name":"E","icon":0,"glyph":"map","key":"B","kind":"skill","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"f","name":"F","icon":0,"glyph":"gem","key":"X","kind":"skill","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"u","name":"U","icon":0,"glyph":"fire","key":"Left Stick + Right Stick","kind":"ultimate","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0.999}]}'>

<!--
  The ability bar at the foot of the screen, a row of HudSlot blocks:
  the consumables in smaller slots, set apart before the skills, then each
  skill, and the ultimate, larger, at the end. The slot draws itself (its
  cooldown, cost, charges and key cap are HudSlot's); the bar only
  sizes and spaces them. A game that wants another bar lays HudSlot
  out itself. The game owns every cooldown, cost and charge; read only.
-->
<ul id="RpgSkills" part="RpgSkills.frame" class="hud bare" data-layout="row" aria-label={t("rpg.skills.name")}>
  {#each slots as s (s.id)}
  <li part="RpgSkills.skill" class="skill" data-kind={s.kind}><HudSlot name={s.name} icon={s.icon} glyph={s.glyph} key={s.key} kind={s.kind} cooldown={s.cooldown} time={s.time} count={s.count} ready={s.ready} unaffordable={s.unaffordable} reason={s.reason} charges={s.charges} chargesMax={s.chargesMax} charge={s.charge} /></li>
  {/each}
</ul>

<style>
:host { margin: 0; padding: 0; list-style: none; align-items: flex-end; column-gap: calc(10 * var(--font-step-px)); max-inline-size: stretch; color: var(--hud-ink); --s: calc(var(--size-xl) * 0.8); --item: calc(var(--size-xl) * 0.7); --ult: calc(var(--size-xl) * 1.3); }
/* A slot's place: the skills' size, the consumables smaller, the ultimate
   larger. */
.skill { flex: 0 1 auto; min-inline-size: 0; --hud-slot-size: var(--s); }
.skill[data-kind=item] { --hud-slot-size: var(--item); }
.skill[data-kind=ultimate] { --hud-slot-size: var(--ult); }
/* The consumables are set apart from the skills; the ultimate a little too. */
.skill[data-kind=item] + .skill:not([data-kind=item]) { margin-inline-start: calc(10 * var(--font-step-px)); }
.skill[data-kind=ultimate] { margin-inline-start: calc(6 * var(--font-step-px)); }
/* On a phone the skills and the ultimate fit the width; the items drop. */
@media (width < 600px) {
  :host { column-gap: calc(6 * var(--font-step-px)); --s: calc(var(--size-xl) * 0.6); --ult: calc(var(--size-xl) * 0.85); }
  .skill[data-kind=item] { display: none; }
  .skill[data-kind=item] + .skill:not([data-kind=item]) { margin-inline-start: 0; }
  .skill[data-kind=ultimate] { margin-inline-start: calc(2 * var(--font-step-px)); }
}
</style>
