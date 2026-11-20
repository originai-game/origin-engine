<meta name="component" content="HudSlot">
<meta name="strings" content="./slot.en.ogt">
<meta name="prop:name" content="string" data-doc="What a reader calls it (Ember slash).">
<meta name="prop:icon" content="number" data-doc="Its art, a texture (og.load.texture); 0 draws the glyph in its place.">
<meta name="prop:glyph" content="string" data-doc="An icon of the atlas, drawn when there is no art; empty is none.">
<meta name="prop:key" content="string" data-doc="The input as the game names it (1, Q), on a key cap under the slot; empty is none.">
<meta name="prop:kind" content="string" data-default="skill" data-doc="skill, item (a consumable, its count on a tag) or ultimate (a hexagon inside a ring, gold by default (--hud-slot-ring), that fills with its charge).">
<meta name="prop:cooldown" content="number" data-doc="The share of its cooldown still to run, 0 to 1: a dark sweep with a lit leading edge; 0 is none.">
<meta name="prop:time" content="string" data-doc="The cooldown left as the game words it (12), over the sweep; empty is none.">
<meta name="prop:count" content="number" data-doc="How many are left, on a tag at the slot's corner; 0 shows none.">
<meta name="prop:ready" content="bool" data-doc="Ready and worth pressing now: a lit edge and a glow (an ultimate at full charge has them without it).">
<meta name="prop:unaffordable" content="bool" data-doc="It cannot be paid for: the art greyed under a veil.">
<meta name="prop:reason" content="string" data-doc="Why it cannot be used (No mana), on the slot while it is unaffordable and not cooling; empty is none.">
<meta name="prop:charges" content="number" data-doc="Charges held, the lit pips over the slot.">
<meta name="prop:chargesMax" content="number" data-doc="Charges it holds; 0 draws no pips.">
<meta name="prop:charge" content="number" data-doc="An ultimate's charge, 0 to 1: its ring's lit arc and, under 1, the percent on it.">
<meta name="fixture:default" content='{"name":"Ember slash","icon":0,"glyph":"sword","key":"1","kind":"skill","cooldown":0,"time":"","count":0,"ready":true,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0}'>
<meta name="fixture:cooling" content='{"name":"Arc step","icon":0,"glyph":"star","key":"3","kind":"skill","cooldown":0.4,"time":"3","count":0,"ready":false,"unaffordable":false,"reason":"","charges":1,"chargesMax":3,"charge":0}'>
<meta name="fixture:item" content='{"name":"Healing tonic","icon":0,"glyph":"potion","key":"Z","kind":"item","cooldown":0,"time":"","count":5,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0}'>
<meta name="fixture:ultimate" content='{"name":"Cataclysm","icon":0,"glyph":"fire","key":"R","kind":"ultimate","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0.58}'>
<meta name="fixture:unaffordable" content='{"name":"Void rend","icon":0,"glyph":"skull","key":"4","kind":"skill","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":true,"reason":"No mana","charges":0,"chargesMax":0,"charge":0}'>
<meta name="fixture:empty" content='{"name":"","icon":0,"glyph":"","key":"","kind":"skill","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0}'>
<meta name="fixture:overflow" content='{"name":"Thunderous riposte of the Ninefold Gate","icon":0,"glyph":"bolt","key":"Right Trigger","kind":"skill","cooldown":0.5,"time":"1:02:44","count":12345,"ready":false,"unaffordable":false,"reason":"","charges":7,"chargesMax":9,"charge":0}'>

<!--
  One slot of a HUD's skill bar, a block a bar is built from: the
  art in an octagon with a lit edge, its cooldown a dark radial sweep with
  a lit leading edge and the time left, a slot that cannot be paid for
  greyed with its reason on it, charge pips over a slot that holds
  charges, a consumable's count on a tag, and the ultimate in a hexagon
  inside a gold ring that fills with its charge; a key cap under it. Its
  side is --hud-slot-size; a bar sets it on the place it gives the
  slot. It is as wide as its own content wherever it stands, so the art
  and the key cap are one column of the slot's width. The game owns every cooldown, cost and charge; read only.
-->
<div id="HudSlot" part="skill" layout="stack" gap="none" role="group" class:ultimate={kind == 'ultimate'} class:ready={ready || (kind == 'ultimate' && charge >= 1)} class:unaffordable={unaffordable} class:cooling={cooldown > 0 || (kind == 'ultimate' && charge < 1)} aria-label={unaffordable && reason ? t("hud.slot.state", { name: name, state: reason }) : name}>
  {#if chargesMax > 0}<div part="charges" class="charges" layout="overlay" role="meter" aria-label={t("hud.slot.charges")} aria-valuemin="0" aria-valuemax={chargesMax} aria-valuenow={charges <= 0 ? 0 : charges} style:--n={chargesMax} style:--on={charges <= 0 ? 0 : charges >= chargesMax ? 1 : charges / chargesMax}><div part="pip" class="pip" anchor="top-start"></div></div>{/if}
  <div class="holder" layout="overlay">
    {#if kind == 'ultimate'}<div part="ring" class="ring" anchor="fill" role="meter" aria-label={t("hud.slot.charge")} aria-valuemin="0" aria-valuemax="100" aria-valuenow={charge <= 0 ? 0 : charge >= 1 ? 100 : charge * 100}><svg class="dial" viewBox="0 0 100 100"><path class="well" d="M 50 4 A 46 46 0 1 1 50 96 A 46 46 0 1 1 50 4" fill="none"></path><path class="lit" d="M 50 4 A 46 46 0 1 1 50 96 A 46 46 0 1 1 50 4" fill="none" pathLength="1" stroke-dasharray={(charge <= 0 ? 0 : charge >= 1 ? 1 : charge) + " 1"}></path></svg></div>{/if}
    <div part="slot" class="slot" anchor="fill" layout="overlay">
      {#if icon}<img part="art" class="art" anchor="fill" src={icon} alt="">{:else}<div class="blank" anchor="fill" layout="center">{#if glyph}<og-icon class="glyph" icon={glyph}></og-icon>{/if}</div>{/if}
      {#if cooldown > 0}<div part="cooldown" class="cooldown" anchor="fill" style:--left={cooldown >= 1 ? 1 : cooldown}></div><div part="edge" class="edge" anchor="fill" style:--left={cooldown >= 1 ? 1 : cooldown}></div>{/if}
      {#if unaffordable}<div class="veil" anchor="fill"></div>{/if}
      {#if cooldown > 0 && time}<span layout="center" part="time" class="time" anchor="fill">{time}</span>{/if}
      {#if unaffordable && reason && cooldown <= 0}<span layout="center" part="reason" class="reason" anchor="fill">{reason}</span>{/if}
      {#if kind == 'ultimate' && charge < 1}<span layout="center" part="time" class="time" anchor="fill">{formatPercent(charge <= 0 ? 0 : charge)}</span>{/if}
    </div>
    {#if count > 0}<span part="count" class="count" anchor="top-end">{t("hud.slot.count", { count: formatNumber(count) })}</span>{/if}
  </div>
  {#if key}<span part="key" class="key">{key}</span>{/if}
</div>

<style>
/* The slot is art: it keeps its size at any text scale, and its figures
   and key cap follow the text. */
#HudSlot { inline-size: fit-content; max-inline-size: 100%; align-items: center; row-gap: calc(5 * var(--font-step-px)); color: var(--hud-ink); filter: var(--hud-lift); --side: var(--hud-slot-size); }
.holder { inline-size: var(--side); block-size: var(--side); --layout-anchor-inset: 0; }
/* The slot: an octagon whose fill is its lit edge, the art inset inside. */
.slot { container-type: size; inline-size: var(--side); block-size: var(--side); clip-path: var(--hud-octagon); background: var(--hud-edge); --layout-anchor-inset: calc(2 * var(--font-step-px)); }
.ultimate .slot { inline-size: calc(var(--side) * 0.79); block-size: calc(var(--side) * 0.79); clip-path: var(--hud-hexagon); }
.ready .slot { background: var(--hud-ready-edge); }
.ready .holder { filter: var(--hud-ready-glow); }
.ultimate.ready .holder { filter: var(--hud-gold-glow); }
.art, .blank, .cooldown, .edge, .veil { clip-path: var(--hud-octagon); }
.ultimate .art, .ultimate .blank, .ultimate .cooldown, .ultimate .edge, .ultimate .veil { clip-path: var(--hud-hexagon); }
.art { object-fit: cover; }
.blank { background: var(--hud-plate-fill); }
.glyph { inline-size: 50%; block-size: 50%; color: var(--hud-ink); }
.unaffordable .art, .unaffordable .glyph { filter: var(--hud-unready); }
/* While a figure or a reason stands on the slot, the art dims under it so it reads. */
.cooling .art, .cooling .glyph, .unaffordable .art, .unaffordable .glyph { opacity: var(--opacity-dim); }
.veil { background: var(--hud-veil); }
/* The cooldown still to run is dark; it turns clockwise from the top, its
   leading edge lit. */
.cooldown { background: var(--hud-sweep); mask-image: conic-gradient(transparent calc((1 - var(--left)) * 360deg), #000 0); }
.edge { background: var(--hud-sweep-edge); mask-image: conic-gradient(transparent calc((1 - var(--left)) * 360deg - 4deg), #000 calc((1 - var(--left)) * 360deg - 4deg) calc((1 - var(--left)) * 360deg), transparent calc((1 - var(--left)) * 360deg)); }
.time { font-size: var(--hud-size-md); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
.ultimate .time { font-size: var(--hud-size-sm); }
/* The reason stands on the slot in short capitals; a word per line when
   the slot is narrower than the phrase (No / mana). */
.reason { max-inline-size: 100%; max-block-size: 84%; font-size: var(--hud-size-reason); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: normal; text-transform: var(--hud-label-transform); text-align: center; color: var(--hud-mana-ink); overflow: hidden; text-shadow: var(--hud-shadow); }
/* The ultimate's ring: a well and the charge as a lit arc, gold unless
   the place sets --hud-slot-ring. */
.dial { inline-size: 100%; block-size: 100%; }
.well, .lit { stroke-width: 6; }
.well { stroke: var(--hud-ring-well); }
.lit { stroke: var(--hud-slot-ring); }
/* Charge pips over the slot: one leaning step per charge, the held ones lit. */
.charges { --pip: calc(12 * var(--font-step-px)); inline-size: calc(var(--n) * var(--pip)); max-inline-size: var(--side); block-size: calc(5 * var(--font-step-px)); margin-block-end: calc(5 * var(--font-step-px)); transform: skewX(var(--bar-skew)); background: var(--hud-pip-off); mask-image: repeating-linear-gradient(90deg, #000 0 calc(100% / var(--n) - 3 * var(--font-step-px)), transparent calc(100% / var(--n) - 3 * var(--font-step-px)) calc(100% / var(--n))); --layout-anchor-inset: 0; }
.pip { inline-size: calc(var(--on) * 100%); block-size: 100%; background: var(--hud-pip-on); box-shadow: var(--hud-pip-glow); }
.count { max-inline-size: calc(var(--side) * 1.2); margin-block-start: calc(-6 * var(--font-step-px)); margin-inline-end: calc(-4 * var(--font-step-px)); padding: 0 calc(5 * var(--font-step-px)); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.key { min-inline-size: calc(24 * var(--font-step-px)); max-inline-size: calc(var(--side) * 1.3); padding: 0 var(--space-xs); font-size: var(--hud-tag-size); line-height: 1.6; font-weight: var(--hud-figure-weight); text-align: center; color: var(--hud-key-ink); background: var(--hud-key-fill); border-radius: var(--hud-key-radius); box-shadow: var(--hud-key-shadow); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.ultimate .key { max-inline-size: calc(var(--side) * 0.8); background: var(--hud-key-gold-fill); }
/* Under a touch screen the slots are the controls: no key caps. */
@media (pointer: coarse) { .key { display: none; } }
/* On a phone its figures and pips are smaller. */
@media (width < 600px) {
  .time { font-size: var(--hud-size-sm); }
  .ultimate .time { font-size: var(--hud-size-xs); }
  .charges { --pip: calc(9 * var(--font-step-px)); block-size: calc(3 * var(--font-step-px)); }
}
</style>
