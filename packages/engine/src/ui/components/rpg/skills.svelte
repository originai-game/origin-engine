<meta name="component" content="RpgSkills">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgSkills.frame RpgSkills.skill RpgSkills.slot RpgSkills.art RpgSkills.cooldown RpgSkills.edge RpgSkills.time RpgSkills.reason RpgSkills.count RpgSkills.charges RpgSkills.pip RpgSkills.ring RpgSkills.key">
<meta name="prop:slots" content="list" data-doc="The bar's slots in the game's order, keyed rows of { id, name, icon, glyph, key, kind, cooldown, time, count, ready, unaffordable, reason, charges, chargesMax, charge }: name for a reader; icon a texture (0: the glyph, an icon name, in its place); key the input as the game names it (1, Q), on a key cap under the slot; kind skill, item (a consumable: a smaller slot with its count on a tag, set apart before the skills; a phone leaves items out) or ultimate (a larger hexagon inside a gold ring that fills with charge, 0 to 1, its share in figures until full; on data-kind); cooldown the share still to run, 0 to 1 (a dark radial sweep over the art with a lit leading edge), with time its figure as the game words it (8); count the items held (×5; 0 hides it); ready lit (a lit edge and glow); unaffordable it cannot be paid for (the art greys under a veil and reason, as the game words it (No mana), stands on it); chargesMax 0 hides the charge pips, else charges of them are lit over the slot.">
<meta name="fixture:default" content='{"slots":[{"id":"tonic","name":"Healing tonic","icon":0,"glyph":"potion","key":"Z","kind":"item","cooldown":0,"time":"","count":5,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"ether","name":"Ether","icon":0,"glyph":"drop","key":"X","kind":"item","cooldown":0,"time":"","count":2,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"slash","name":"Ember slash","icon":0,"glyph":"sword","key":"1","kind":"skill","cooldown":0,"time":"","count":0,"ready":true,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"frost","name":"Frost lance","icon":0,"glyph":"bolt","key":"2","kind":"skill","cooldown":0.65,"time":"8","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"arc","name":"Arc step","icon":0,"glyph":"star","key":"3","kind":"skill","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":false,"reason":"","charges":2,"chargesMax":3,"charge":0},{"id":"void","name":"Void rend","icon":0,"glyph":"skull","key":"4","kind":"skill","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":true,"reason":"No mana","charges":0,"chargesMax":0,"charge":0},{"id":"gale","name":"Gale","icon":0,"glyph":"map","key":"Q","kind":"skill","cooldown":0.18,"time":"2","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"ult","name":"Cataclysm","icon":0,"glyph":"fire","key":"R","kind":"ultimate","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":1}]}'>
<meta name="fixture:cooling" content='{"slots":[{"id":"slash","name":"Ember slash","icon":0,"glyph":"sword","key":"1","kind":"skill","cooldown":1,"time":"12","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"arc","name":"Arc step","icon":0,"glyph":"star","key":"3","kind":"skill","cooldown":0.4,"time":"3","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":3,"charge":0},{"id":"ult","name":"Cataclysm","icon":0,"glyph":"fire","key":"R","kind":"ultimate","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0.35}]}'>
<meta name="fixture:unaffordable" content='{"slots":[{"id":"void","name":"Void rend","icon":0,"glyph":"skull","key":"4","kind":"skill","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":true,"reason":"No mana","charges":0,"chargesMax":0,"charge":0},{"id":"tonic","name":"Healing tonic","icon":0,"glyph":"potion","key":"Z","kind":"item","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":true,"reason":"","charges":0,"chargesMax":0,"charge":0}]}'>
<meta name="fixture:empty" content='{"slots":[]}'>
<meta name="fixture:overflow" content='{"slots":[{"id":"a","name":"A","icon":0,"glyph":"potion","key":"Left Shoulder","kind":"item","cooldown":0,"time":"","count":12345,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"b","name":"B","icon":0,"glyph":"bolt","key":"Right Trigger","kind":"skill","cooldown":0.5,"time":"1:02:44","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"c","name":"C","icon":0,"glyph":"skull","key":"Dpad Up","kind":"skill","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":true,"reason":"Not enough soul essence","charges":0,"chargesMax":0,"charge":0},{"id":"d","name":"D","icon":0,"glyph":"star","key":"Y","kind":"skill","cooldown":0,"time":"","count":0,"ready":true,"unaffordable":false,"reason":"","charges":7,"chargesMax":9,"charge":0},{"id":"e","name":"E","icon":0,"glyph":"map","key":"B","kind":"skill","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"f","name":"F","icon":0,"glyph":"gem","key":"X","kind":"skill","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"u","name":"U","icon":0,"glyph":"fire","key":"Left Stick + Right Stick","kind":"ultimate","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0.999}]}'>

<!--
  The ability bar at the foot of the screen: the consumables in smaller
  octagons with their counts, then each skill in an octagon with a lit
  edge, its cooldown a dark radial sweep with a lit leading edge and the
  seconds left, a skill that cannot be paid for greyed with its reason on
  it, charge pips over a slot that holds charges, and the ultimate in a
  larger hexagon inside a gold ring that fills with its charge; a key cap
  under every slot. The game owns every cooldown, cost and charge; read
  only.
-->
<ul id="RpgSkills" part="RpgSkills.frame" class="hud bare" data-layout="row" aria-label={t("rpg.skills.name")}>
  {#each slots as s (s.id)}
  <li part="RpgSkills.skill" class="skill" data-layout="stack" data-gap="none" data-kind={s.kind} data-ready={s.ready || (s.kind == 'ultimate' && s.charge >= 1)} data-unaffordable={s.unaffordable} aria-label={s.unaffordable && s.reason != '' ? t("rpg.skills.slot", { name: s.name, state: s.reason }) : s.name}>
    {#if s.chargesMax > 0}<div part="RpgSkills.charges" class="charges" data-layout="overlay" role="meter" aria-label={t("rpg.skills.charges")} aria-valuemin="0" aria-valuemax={s.chargesMax} aria-valuenow={s.charges <= 0 ? 0 : s.charges} style:--n={s.chargesMax} style:--on={s.charges <= 0 ? 0 : s.charges >= s.chargesMax ? 1 : s.charges / s.chargesMax}><div part="RpgSkills.pip" class="pip" data-anchor="top-start"></div></div>{/if}
    <div class="holder" data-layout="overlay">
      {#if s.kind == 'ultimate'}<div part="RpgSkills.ring" class="ring" data-anchor="fill" role="meter" aria-label={t("rpg.skills.charge")} aria-valuemin="0" aria-valuemax="100" aria-valuenow={s.charge <= 0 ? 0 : s.charge >= 1 ? 100 : s.charge * 100}><svg class="dial" viewBox="0 0 100 100"><path class="well" d="M 50 4 A 46 46 0 1 1 50 96 A 46 46 0 1 1 50 4" fill="none"></path><path class="lit" d="M 50 4 A 46 46 0 1 1 50 96 A 46 46 0 1 1 50 4" fill="none" pathLength="1" stroke-dasharray={(s.charge <= 0 ? 0 : s.charge >= 1 ? 1 : s.charge) + " 1"}></path></svg></div>{/if}
      <div part="RpgSkills.slot" class="slot" data-anchor="center" data-layout="overlay">
        {#if s.icon}<img part="RpgSkills.art" class="art" data-anchor="fill" src={s.icon} alt="">{:else}<div class="blank" data-anchor="fill" data-layout="center">{#if s.glyph != ''}<og-icon class="glyph" icon={s.glyph}></og-icon>{/if}</div>{/if}
        {#if s.cooldown > 0}<div part="RpgSkills.cooldown" class="cooldown" data-anchor="fill" style:--left={s.cooldown >= 1 ? 1 : s.cooldown}></div><div part="RpgSkills.edge" class="edge" data-anchor="fill" style:--left={s.cooldown >= 1 ? 1 : s.cooldown}></div>{/if}
        {#if s.unaffordable}<div class="veil" data-anchor="fill"></div>{/if}
        {#if s.cooldown > 0 && s.time != ''}<span part="RpgSkills.time" class="time" data-anchor="center">{s.time}</span>{/if}
        {#if s.unaffordable && s.reason != '' && s.cooldown <= 0}<span part="RpgSkills.reason" class="reason" data-anchor="center">{s.reason}</span>{/if}
        {#if s.kind == 'ultimate' && s.charge < 1}<span part="RpgSkills.time" class="time" data-anchor="center">{formatPercent(s.charge <= 0 ? 0 : s.charge)}</span>{/if}
      </div>
      {#if s.count > 0}<span part="RpgSkills.count" class="count" data-anchor="top-end">{t("rpg.skills.count", { count: formatNumber(s.count) })}</span>{/if}
    </div>
    {#if s.key != ''}<span part="RpgSkills.key" class="key">{s.key}</span>{/if}
  </li>
  {/each}
</ul>

<style>
:host { margin: 0; padding: 0; list-style: none; align-items: flex-end; column-gap: calc(10 * var(--font-step-px)); max-inline-size: stretch; color: var(--hud-ink); --s: calc(var(--size-xl) * 0.8); --item: calc(var(--size-xl) * 0.7); --ult: calc(var(--size-xl) * 1.3); }
/* The slots are art: they keep their size at any text scale, and their
   figures and key caps follow the text. */
.skill { flex: 0 1 auto; min-inline-size: 0; align-items: center; row-gap: calc(5 * var(--font-step-px)); filter: var(--hud-lift); }
/* The consumables are set apart from the skills; the ultimate a little too. */
.skill[data-kind=item] + .skill:not([data-kind=item]) { margin-inline-start: calc(10 * var(--font-step-px)); }
.skill[data-kind=ultimate] { margin-inline-start: calc(6 * var(--font-step-px)); }
.skill { --side: var(--s); }
.skill[data-kind=item] { --side: var(--item); }
.skill[data-kind=ultimate] { --side: var(--ult); }
.holder { inline-size: var(--side); block-size: var(--side); --layout-anchor-inset: 0; }
/* The slot: an octagon whose fill is its lit edge, the art inset inside. */
.slot { container-type: size; inline-size: var(--side); block-size: var(--side); clip-path: var(--hud-octagon); background: var(--hud-edge); --layout-anchor-inset: calc(2 * var(--font-step-px)); }
[data-kind=ultimate] .slot { inline-size: calc(var(--side) * 0.79); block-size: calc(var(--side) * 0.79); clip-path: var(--hud-hexagon); }
[data-ready=true] .slot { background: var(--hud-ready-edge); }
[data-ready=true] .holder { filter: var(--hud-ready-glow); }
[data-kind=ultimate][data-ready=true] .holder { filter: var(--hud-gold-glow); }
.art, .blank, .cooldown, .edge, .veil { clip-path: var(--hud-octagon); }
[data-kind=ultimate] .art, [data-kind=ultimate] .blank, [data-kind=ultimate] .cooldown, [data-kind=ultimate] .edge, [data-kind=ultimate] .veil { clip-path: var(--hud-hexagon); }
.art { object-fit: cover; }
.blank { background: var(--hud-plate-fill); }
.glyph { inline-size: 50%; block-size: 50%; color: var(--hud-ink); }
[data-unaffordable=true] .art, [data-unaffordable=true] .glyph { filter: var(--hud-unready); }
.veil { background: var(--hud-veil); }
/* The cooldown still to run is dark; it turns clockwise from the top, its
   leading edge lit. */
.cooldown { background: var(--hud-sweep); mask-image: conic-gradient(transparent calc((1 - var(--left)) * 360deg), #000 0); }
.edge { background: var(--hud-sweep-edge); mask-image: conic-gradient(transparent calc((1 - var(--left)) * 360deg - 4deg), #000 calc((1 - var(--left)) * 360deg - 4deg) calc((1 - var(--left)) * 360deg), transparent calc((1 - var(--left)) * 360deg)); }
.time { font-size: var(--hud-size-md); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
[data-kind=ultimate] .time { font-size: var(--hud-size-sm); }
/* The reason stands on the slot in short capitals; a word per line when
   the slot is narrower than the phrase (No / mana). */
.reason { max-inline-size: 100%; max-block-size: 84%; font-size: var(--hud-size-reason); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: normal; text-transform: var(--hud-label-transform); text-align: center; color: var(--hud-mana-ink); overflow: hidden; text-shadow: var(--hud-shadow); }
/* The ultimate's ring: a well and the charge as a lit gold arc. */
.dial { inline-size: 100%; block-size: 100%; }
.well, .lit { stroke-width: 6; }
.well { stroke: var(--hud-ring-well); }
.lit { stroke: var(--hud-gold); }
/* Charge pips over the slot: one leaning step per charge, the held ones lit. */
.charges { --pip: calc(12 * var(--font-step-px)); inline-size: calc(var(--n) * var(--pip)); max-inline-size: var(--side); block-size: calc(5 * var(--font-step-px)); margin-block-end: calc(5 * var(--font-step-px)); transform: skewX(var(--bar-skew)); background: var(--hud-pip-off); mask-image: repeating-linear-gradient(90deg, #000 0 calc(100% / var(--n) - 3 * var(--font-step-px)), transparent calc(100% / var(--n) - 3 * var(--font-step-px)) calc(100% / var(--n))); --layout-anchor-inset: 0; }
.pip { inline-size: calc(var(--on) * 100%); block-size: 100%; background: var(--hud-pip-on); box-shadow: var(--hud-pip-glow); }
.count { max-inline-size: calc(var(--side) * 1.2); margin-block-start: calc(-6 * var(--font-step-px)); margin-inline-end: calc(-4 * var(--font-step-px)); padding: 0 calc(5 * var(--font-step-px)); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
.key { min-inline-size: calc(24 * var(--font-step-px)); max-inline-size: calc(var(--s) * 1.3); padding: 0 var(--space-xs); font-size: var(--hud-tag-size); line-height: 1.6; font-weight: var(--hud-figure-weight); text-align: center; color: var(--hud-key-ink); background: var(--hud-key-fill); border-radius: var(--hud-key-radius); box-shadow: var(--hud-key-shadow); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
[data-kind=ultimate] .key { background: var(--hud-key-gold-fill); }
/* Under a touch screen the slots are the controls: no key caps. */
@media (pointer: coarse) { .key { display: none; } }
/* On a phone the skills and the ultimate fit the width; the items drop. */
@media (width < 600px) {
  :host { column-gap: calc(6 * var(--font-step-px)); --s: calc(var(--size-xl) * 0.6); --ult: calc(var(--size-xl) * 0.85); }
  .skill[data-kind=item] { display: none; }
  .skill[data-kind=item] + .skill:not([data-kind=item]) { margin-inline-start: 0; }
  .skill[data-kind=ultimate] { margin-inline-start: calc(2 * var(--font-step-px)); }
  .time { font-size: var(--hud-size-sm); }
  [data-kind=ultimate] .time { font-size: var(--hud-size-xs); }
  .charges { --pip: calc(9 * var(--font-step-px)); block-size: calc(3 * var(--font-step-px)); }
}
</style>
