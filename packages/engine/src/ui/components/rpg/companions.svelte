<meta name="component" content="RpgCompanions">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgCompanions.frame RpgCompanions.member RpgCompanions.portrait RpgCompanions.callsign RpgCompanions.fallen RpgCompanions.health RpgCompanions.mana RpgCompanions.key">
<meta name="prop:members" content="list" data-doc="The leader's companions in the game's order, keyed rows of { id, name, portrait, health, healthMax, ghost, mana, manaMax, key, fallen }: portrait a texture (0: the person glyph); ghost the health before the latest loss; manaMax 0 hides the mana bar; key the input that switches to them as the game names it (2), on a key cap at the plate's end, empty for none; fallen out of the fight (on data-fallen: the plate dims, the portrait greys and a Fallen tag stands beside the name).">
<meta name="prop:low" content="number" data-default="0.25" data-doc="The share of health under which a companion is low: the fill turns to danger and the rim glows (0.25).">
<meta name="fixture:default" content='{"low":0.25,"members":[{"id":"brannoch","name":"Brannoch","portrait":0,"health":91,"healthMax":100,"ghost":0,"mana":40,"manaMax":100,"key":"2","fallen":false},{"id":"sylvenne","name":"Sylvenne","portrait":0,"health":18,"healthMax":100,"ghost":44,"mana":88,"manaMax":100,"key":"3","fallen":false},{"id":"caelum","name":"Caelum","portrait":0,"health":0,"healthMax":100,"ghost":0,"mana":0,"manaMax":100,"key":"4","fallen":true}]}'>
<meta name="fixture:fallen" content='{"low":0.25,"members":[{"id":"caelum","name":"Caelum","portrait":0,"health":0,"healthMax":100,"ghost":12,"mana":0,"manaMax":100,"key":"4","fallen":true}]}'>
<meta name="fixture:critical" content='{"low":0.25,"members":[{"id":"sylvenne","name":"Sylvenne","portrait":0,"health":2,"healthMax":100,"ghost":30,"mana":0,"manaMax":0,"key":"","fallen":false}]}'>
<meta name="fixture:empty" content='{"low":0.25,"members":[]}'>
<meta name="fixture:overflow" content='{"low":0.25,"members":[{"id":"long","name":"Brannoch Hollowmere, Sworn Shield of the Ninefold Gate","portrait":0,"health":123456,"healthMax":999999,"ghost":200000,"mana":99999,"manaMax":99999,"key":"Left Shoulder + Dpad Up","fallen":false},{"id":"down","name":"Caelum of the Long Ashen Road","portrait":0,"health":0,"healthMax":10,"ghost":0,"mana":0,"manaMax":10,"key":"4","fallen":true}]}'>

<!--
  The leader's companions under the leader's vitals, built from the RPG
  HUD's blocks: each a small HudPortrait, the name over a thin health
  HudBar with its ghost and a thinner mana HudBar, and the key that
  switches to them on a cap at the plate's end. A fallen companion's
  plate dims with a Fallen tag. The game owns every value and state;
  read only.
-->
<section id="RpgCompanions" part="RpgCompanions.frame" class="hud bare" data-layout="stack" role="group" aria-label={t("rpg.party.name")}>
  {#each members as m (m.id)}
  <div part="RpgCompanions.member" class="member" data-fallen={m.fallen} class:low={!m.fallen && m.healthMax > 0 && m.health <= m.healthMax * low} role="group" aria-label={m.fallen ? t("rpg.companions.member", { name: m.name, state: t("rpg.companions.fallen") }) : m.name}>
    <div class="portrait" part="RpgCompanions.portrait"><HudPortrait portrait={m.portrait} dimmed={m.fallen} /></div>
    <div class="who" data-layout="row" data-gap="tight">
      <span part="RpgCompanions.callsign" class="name">{m.name}</span>
      {#if m.fallen}<span part="RpgCompanions.fallen" class="fallen">{t("rpg.companions.fallen")}</span>{/if}
    </div>
    <div part="RpgCompanions.health" class="health"><HudBar value={m.health} max={m.healthMax} kind="health" ghost={m.ghost} segment={0} low={!m.fallen && m.healthMax > 0 && m.health <= m.healthMax * low} name="" /></div>
    {#if m.manaMax > 0 && !m.fallen}<div part="RpgCompanions.mana" class="mana"><HudBar value={m.mana} max={m.manaMax} kind="mana" ghost={0} segment={0} low={false} showValue={false} name="" /></div>{/if}
    {#if m.key}<span part="RpgCompanions.key" class="key">{m.key}</span>{/if}
  </div>
  {/each}
</section>

<style>
:host { row-gap: calc(9 * var(--font-step-px)); inline-size: calc(224 * var(--font-step-px)); max-inline-size: stretch; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
/* A plate: the portrait beside the name over the bars, the key at the end. */
.member { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; grid-template-rows: auto auto auto; grid-template-areas: "portrait who key" "portrait health key" "portrait mana key"; column-gap: calc(10 * var(--font-step-px)); row-gap: calc(4 * var(--font-step-px)); align-items: center; min-inline-size: 0; --layout-anchor-inset: 0; }
.member[data-fallen=true] { opacity: 0.6; }
.portrait { grid-area: portrait; }
.who { grid-area: who; align-items: baseline; min-inline-size: 0; }
.name { flex: 0 1 auto; min-inline-size: 0; font-size: var(--hud-size-name); font-weight: var(--hud-label-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.fallen { flex: none; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-harm); white-space: nowrap; }
/* Thin leaning bars: health and its ghost over a shorter mana bar. A
   companion's low bar glows and does not pulse; the leader's does. */
.health { grid-area: health; inline-size: 100%; --hud-bar-size: calc(6 * var(--font-step-px)); --pulse-motion: none; }
.mana { grid-area: mana; inline-size: 80%; --hud-bar-size: calc(4 * var(--font-step-px)); }
/* The key cap that switches to the companion. */
.key { grid-area: key; align-self: center; min-inline-size: calc(22 * var(--font-step-px)); max-inline-size: calc(72 * var(--font-step-px)); padding: 0 var(--space-xs); font-size: var(--hud-tag-size); line-height: 1.5; font-weight: var(--hud-figure-weight); text-align: center; color: var(--hud-key-ink); background: var(--hud-key-fill); border-radius: var(--hud-key-radius); box-shadow: var(--hud-key-shadow); -webkit-text-stroke-width: 0; text-shadow: none; white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
/* Under a touch screen a plate is the control: no key caps. */
@media (pointer: coarse) { .key { display: none; } }
/* On a phone the plates stand side by side, each compact: the name over
   the bars, no portrait, so a short name fits whole at a large text scale
   (they wrap only when it cannot). */
@media (width < 600px), (height < 500px) {
  :host { flex-direction: row; flex-wrap: wrap; inline-size: stretch; column-gap: var(--space-md); row-gap: var(--space-sm); }
  /* The rows start at the plate's top and keep their own heights (the row
     stretches every plate to the tallest one's height; stretched rows
     would move a fallen plate's bar, whose tag row is taller than a mana
     bar), so every bar sits directly under its name. */
  .member { flex: 1 1 calc(70 * var(--font-step-px)); grid-template-columns: minmax(0, 1fr); grid-template-areas: "who" "health" "mana"; row-gap: calc(3 * var(--font-step-px)); align-content: start; }
  .portrait { display: none; }
  .name { font-size: var(--hud-size-xs); }
  /* A fallen companion has no mana bar: its tag stands in that row. */
  .who { display: contents; }
  .name { grid-area: who; }
  .fallen { grid-area: mana; justify-self: start; letter-spacing: normal; }
  .health { --hud-bar-size: calc(5 * var(--font-step-px)); }
  .mana { --hud-bar-size: calc(3 * var(--font-step-px)); }
  .key { display: none; }
}
</style>
