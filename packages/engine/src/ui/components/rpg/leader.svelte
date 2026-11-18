<meta name="component" content="RpgLeader">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgLeader.frame RpgLeader.portrait RpgLeader.face RpgLeader.hero RpgLeader.level RpgLeader.value RpgLeader.max RpgLeader.health RpgLeader.ghost RpgLeader.mana RpgLeader.stamina">
<meta name="prop:name" content="string" data-doc="The leader's name; empty hides it.">
<meta name="prop:level" content="number" data-doc="The level beside the name; 0 hides it.">
<meta name="prop:portrait" content="number" data-doc="A head-and-shoulders texture (og.load.texture); 0 draws the person glyph.">
<meta name="prop:health" content="number" data-doc="Health now, shown as a figure over its most and as the fat bar.">
<meta name="prop:healthMax" content="number" data-doc="Most health.">
<meta name="prop:ghost" content="number" data-doc="Health before the latest loss: a warm ghost behind the fill that drains after it (at or under health: none).">
<meta name="prop:mana" content="number" data-doc="Mana now.">
<meta name="prop:manaMax" content="number" data-doc="Most mana; 0 hides the mana bar.">
<meta name="prop:stamina" content="number" data-doc="Stamina (or energy) now.">
<meta name="prop:staminaMax" content="number" data-doc="Most stamina; 0 hides the stamina bar.">
<meta name="prop:segment" content="number" data-default="0" data-doc="Stamina per cell of the stamina bar (one dodge's cost); 0 draws it whole.">
<meta name="prop:low" content="number" data-default="0.25" data-doc="The share of health under which it is low: the fill turns to danger and the bar's rim glows (0.25).">
<meta name="fixture:default" content='{"name":"Ysolde","level":42,"portrait":0,"health":2184,"healthMax":3420,"ghost":2668,"mana":72,"manaMax":100,"stamina":5,"staminaMax":8,"segment":1,"low":0.25}'>
<meta name="fixture:full" content='{"name":"Ysolde","level":42,"portrait":0,"health":3420,"healthMax":3420,"ghost":0,"mana":100,"manaMax":100,"stamina":8,"staminaMax":8,"segment":1,"low":0.25}'>
<meta name="fixture:low" content='{"name":"Ysolde","level":42,"portrait":0,"health":610,"healthMax":3420,"ghost":1500,"mana":12,"manaMax":100,"stamina":1,"staminaMax":8,"segment":1,"low":0.25}'>
<meta name="fixture:critical" content='{"name":"Ysolde","level":42,"portrait":0,"health":42,"healthMax":3420,"ghost":300,"mana":0,"manaMax":100,"stamina":0,"staminaMax":8,"segment":1,"low":0.25}'>
<meta name="fixture:empty" content='{"name":"","level":0,"portrait":0,"health":0,"healthMax":0,"ghost":0,"mana":0,"manaMax":0,"stamina":0,"staminaMax":0,"segment":0,"low":0.25}'>
<meta name="fixture:overflow" content='{"name":"Ysolde Varrenhallow of the Ninefold Gate and the Ashen Vale","level":1234,"portrait":0,"health":123456,"healthMax":999999,"ghost":200000,"mana":99999,"manaMax":99999,"stamina":37,"staminaMax":40,"segment":4,"low":0.25}'>

<!--
  The party leader's vitals at the screen's top corner: the portrait in a
  large octagon with a lit edge, the name and level, health as a figure
  over its most, a fat health bar that leaves a warm ghost when it drops
  and turns to danger when low, a mana bar and a stamina bar cut into
  cells. Read only.
-->
<section id="RpgLeader" part="RpgLeader.frame" class="hud bare" data-layout="row" role="group" aria-label={name != '' ? name : t("rpg.party.health")} class:low={healthMax > 0 && health <= healthMax * low}>
  <div id="Portrait" part="RpgLeader.portrait" data-layout="overlay">
    {#if portrait}<img id="Face" part="RpgLeader.face" data-anchor="fill" src={portrait} alt="">{:else}<div id="Blank" data-anchor="fill" data-layout="center"><og-icon id="Glyph" icon="person"></og-icon></div>{/if}
  </div>
  <div id="Body" data-layout="stack" data-fill>
    <div id="Head" data-layout="row" data-gap="tight">
      <div id="Who" data-layout="row" data-gap="tight">
        {#if name != ''}<span part="RpgLeader.hero" class="name">{name}</span>{/if}
        {#if level > 0}<span part="RpgLeader.level" class="level">{t("rpg.leader.level", { level: level })}</span>{/if}
      </div>
      {#if healthMax > 0}<div id="Reading" data-layout="row" data-gap="tight">
        <span part="RpgLeader.value" class="value">{formatNumber(health < 0 ? 0 : health)}</span>
        <span part="RpgLeader.max" class="max">{t("rpg.leader.max", { max: formatNumber(healthMax) })}</span>
      </div>{/if}
    </div>
    <div id="Health" part="RpgLeader.health" class="bar" data-layout="overlay" role="meter" aria-label={t("rpg.party.health")} aria-valuemin="0" aria-valuemax={healthMax > 0 ? healthMax : 1} aria-valuenow={health <= 0 ? 0 : health} style:--value={healthMax <= 0 || health <= 0 ? 0 : health >= healthMax ? 1 : health / healthMax} style:--ghost={healthMax <= 0 || ghost <= health ? 0 : ghost >= healthMax ? 1 : ghost / healthMax}>
      <div id="Ghost" part="RpgLeader.ghost" data-anchor="top-start"></div>
      <div id="HealthFill" class="fill" data-anchor="top-start"></div>
    </div>
    {#if manaMax > 0}<div id="Mana" part="RpgLeader.mana" class="bar" data-layout="overlay" role="meter" aria-label={t("rpg.party.mana")} aria-valuemin="0" aria-valuemax={manaMax} aria-valuenow={mana <= 0 ? 0 : mana} style:--value={mana <= 0 ? 0 : mana >= manaMax ? 1 : mana / manaMax}>
      <div id="ManaFill" class="fill" data-anchor="top-start"></div>
    </div>{/if}
    {#if staminaMax > 0}<div id="Stamina" part="RpgLeader.stamina" class="bar" data-layout="overlay" role="meter" aria-label={t("rpg.leader.stamina")} aria-valuemin="0" aria-valuemax={staminaMax} aria-valuenow={stamina <= 0 ? 0 : stamina} style:--value={stamina <= 0 ? 0 : stamina >= staminaMax ? 1 : stamina / staminaMax} style:--cells={segment > 0 ? staminaMax / segment : 1}>
      <div id="StaminaFill" class="fill" data-anchor="top-start"></div>
      {#if segment > 0}<div class="cuts" data-anchor="fill"></div>{/if}
    </div>{/if}
  </div>
</section>

<style>
:host { align-items: center; column-gap: calc(12 * var(--font-step-px)); inline-size: calc(330 * var(--font-step-px)); max-inline-size: stretch; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; --layout-anchor-inset: 0; }
/* The portrait: a large octagon with a lit edge, the art inset inside it. */
#Portrait { flex: none; --layout-anchor-inset: calc(2 * var(--font-step-px)); inline-size: calc(68 * var(--font-step-px)); block-size: calc(68 * var(--font-step-px)); clip-path: var(--hud-octagon); background: var(--hud-edge); filter: var(--hud-lift); }
#Face, #Blank { clip-path: var(--hud-octagon); background: var(--hud-plate-fill); }
#Face { object-fit: cover; }
#Glyph { inline-size: 50%; block-size: 50%; color: var(--hud-ink-dim); }
#Body { min-inline-size: 0; row-gap: calc(5 * var(--font-step-px)); }
/* The name and level take the row first; the figure stands at its end and
   goes under them when they need its room, so a short name never shrinks
   (only a long one scrolls). */
#Head { flex-wrap: wrap; align-items: baseline; row-gap: 0; min-inline-size: 0; }
#Who { flex: 0 1 auto; align-items: baseline; min-inline-size: 0; }
#Reading { flex: none; align-items: baseline; margin-inline-start: auto; }
.name { min-inline-size: 0; font-size: var(--hud-size-title); font-weight: var(--hud-label-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.level { flex: none; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-accent); white-space: nowrap; }
.value { flex: none; font-size: var(--hud-size-title); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.max { flex: none; font-size: var(--hud-size-xs); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink-dim); white-space: nowrap; }
/* The bars lean forward, each shorter than the one above it; the stamina
   bar's cells are cut across its whole well. */
.bar { overflow: hidden; transform: skewX(var(--bar-skew)); background: var(--bar-well); box-shadow: var(--bar-rim); }
.fill { inline-size: calc(var(--value) * 100%); block-size: 100%; }
#Health { inline-size: 100%; block-size: calc(16 * var(--font-step-px)); }
#Mana { inline-size: 86%; block-size: calc(8 * var(--font-step-px)); }
#Stamina { inline-size: 72%; block-size: calc(6 * var(--font-step-px)); }
#Ghost { inline-size: calc(var(--ghost) * 100%); block-size: 100%; background: var(--bar-ghost); transition: var(--bar-ghost-motion); }
#HealthFill { background: var(--hud-life-fill); box-shadow: var(--hud-life-glow); }
#ManaFill { background: var(--hud-mana-fill); box-shadow: var(--hud-mana-glow); }
#StaminaFill { background: var(--hud-stamina-fill); box-shadow: var(--hud-stamina-glow); }
.cuts { background: var(--bar-notch); mask-image: repeating-linear-gradient(90deg, transparent 0 calc(100% / var(--cells) - 3 * var(--font-step-px)), #000 calc(100% / var(--cells) - 3 * var(--font-step-px)) calc(100% / var(--cells))); }
.low #HealthFill { background: var(--hud-low-fill); box-shadow: var(--hud-low-glow); }
.low #Health { box-shadow: var(--bar-critical-rim); animation: var(--pulse-motion); }
.low .value { color: var(--hud-harm); }
/* On a phone the plate is compact: a smaller portrait and thinner bars
   across the row's width. */
@media (width < 600px) {
  :host { column-gap: var(--space-sm); inline-size: stretch; }
  #Portrait { inline-size: calc(44 * var(--font-step-px)); block-size: calc(44 * var(--font-step-px)); }
  #Body { row-gap: calc(4 * var(--font-step-px)); }
  .name, .value { font-size: var(--hud-size-sm); }
  #Health { block-size: calc(12 * var(--font-step-px)); }
  #Mana { block-size: calc(6 * var(--font-step-px)); }
  #Stamina { block-size: calc(5 * var(--font-step-px)); }
}
@media (prefers-reduced-motion: reduce) { .low #Health { animation: none; } }
</style>
