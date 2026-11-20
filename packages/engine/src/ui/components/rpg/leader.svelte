<meta name="component" content="RpgLeader">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgLeader.frame RpgLeader.portrait RpgLeader.hero RpgLeader.level RpgLeader.value RpgLeader.max RpgLeader.health RpgLeader.mana RpgLeader.stamina">
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
  The party leader's vitals at the screen's top corner, built from the RPG
  HUD's blocks: an HudPortrait in a large octagon, the name and level,
  health as a figure over its most, and three HudBar: a fat health bar
  that leaves a warm ghost when it drops and turns to danger when low, a
  mana bar and a stamina bar cut into cells. A game that wants another
  plate lays the same blocks out itself. Read only.
-->
<section id="RpgLeader" part="RpgLeader.frame" class="hud bare" data-layout="row" role="group" aria-label={name != '' ? name : t("rpg.party.health")} class:low={healthMax > 0 && health <= healthMax * low}>
  <div id="Portrait" part="RpgLeader.portrait"><HudPortrait portrait={portrait} dimmed={false} /></div>
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
    <div id="Health" part="RpgLeader.health"><HudBar value={health} max={healthMax} kind="health" ghost={ghost} segment={0} low={healthMax > 0 && health <= healthMax * low} name="" /></div>
    {#if manaMax > 0}<div id="Mana" part="RpgLeader.mana"><HudBar value={mana} max={manaMax} kind="mana" ghost={0} segment={0} low={false} showValue={false} name="" /></div>{/if}
    {#if staminaMax > 0}<div id="Stamina" part="RpgLeader.stamina"><HudBar value={stamina} max={staminaMax} kind="stamina" ghost={0} segment={segment} low={false} showValue={false} name="" /></div>{/if}
  </div>
</section>

<style>
:host { align-items: center; column-gap: calc(12 * var(--font-step-px)); inline-size: calc(330 * var(--font-step-px)); max-inline-size: stretch; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
/* The portrait's place: a large one, lifted off the scene. */
#Portrait { flex: none; filter: var(--hud-lift); --hud-portrait-size: calc(68 * var(--font-step-px)); }
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
/* The bars' places, each shorter and thinner than the one above it. */
#Health { inline-size: 100%; --hud-bar-size: calc(16 * var(--font-step-px)); }
#Mana { inline-size: 86%; --hud-bar-size: calc(8 * var(--font-step-px)); }
#Stamina { inline-size: 72%; --hud-bar-size: calc(6 * var(--font-step-px)); }
.low .value { color: var(--hud-harm); }
/* On a phone the plate is compact: a smaller portrait and thinner bars
   across the row's width. */
@media (width < 600px), (height < 500px) {
  :host { column-gap: var(--space-sm); inline-size: stretch; }
  #Portrait { --hud-portrait-size: calc(44 * var(--font-step-px)); }
  #Body { row-gap: calc(4 * var(--font-step-px)); }
  .name, .value { font-size: var(--hud-size-sm); }
  #Health { --hud-bar-size: calc(12 * var(--font-step-px)); }
  #Mana { --hud-bar-size: calc(6 * var(--font-step-px)); }
  #Stamina { --hud-bar-size: calc(5 * var(--font-step-px)); }
}
</style>