<meta name="component" content="RpgVitals">
<meta name="strings" content="./rpg.en.ogt">
<meta name="prop:name" content="string" data-doc="The character's name; read out, and shown on the large plate.">
<meta name="prop:portrait" content="number" data-doc="A head-and-shoulders texture (og.load.texture); 0 is none.">
<meta name="prop:level" content="number" data-doc="The level on the badge; 0 hides the badge.">
<meta name="prop:health" content="number" data-doc="Health now.">
<meta name="prop:healthMax" content="number" data-doc="Most health; the bar shows health over it.">
<meta name="prop:mana" content="number" data-doc="Mana now.">
<meta name="prop:manaMax" content="number" data-doc="Most mana; 0 hides the mana bar.">
<meta name="prop:plate" content="string" data-doc="large (the player's plate, figures on the bars) or compact (a party row).">
<meta name="prop:down" content="bool" data-doc="Out of the fight: the plate greys.">
<meta name="fixture:default" content='{"name":"Brenna","portrait":0,"level":12,"health":842,"healthMax":842,"mana":237,"manaMax":237,"plate":"large","down":false}'>
<meta name="fixture:compact" content='{"name":"Idris","portrait":0,"level":11,"health":40,"healthMax":82,"mana":30,"manaMax":54,"plate":"compact","down":false}'>

<!--
  A character's vitals: a round portrait with a level badge and health and
  mana bars beside it. large is the player's plate, with the figures on the
  bars; compact is a party row. Read only.
-->
<div id="RpgVitals" class="rpg-vitals" data-plate={plate} data-down={down} role="group" aria-label={name}>
  <div class="ring"><img class="portrait" src={portrait} alt="" hidden={!portrait}></div>
  <div class="badge" hidden={!level}><span class="badge-level">{formatNumber(level)}</span></div>
  <div class="bars">
    <div class="bar health" role="meter" aria-label={t("rpg.party.health")} aria-valuetext={t("rpg.party.figure", { now: health, max: healthMax })} style:--fill={healthMax > 0 ? health / healthMax : 0}><div class="bar-fill"></div><span class="bar-figure" hidden={plate != 'large'}>{t("rpg.party.figure", { now: health, max: healthMax })}</span></div>
    <div class="bar mana" role="meter" hidden={!manaMax} aria-label={t("rpg.party.mana")} aria-valuetext={t("rpg.party.figure", { now: mana, max: manaMax })} style:--fill={manaMax > 0 ? mana / manaMax : 0}><div class="bar-fill"></div><span class="bar-figure" hidden={plate != 'large'}>{t("rpg.party.figure", { now: mana, max: manaMax })}</span></div>
  </div>
</div>

<style>
.rpg-vitals[hidden], .rpg-vitals [hidden] { display: none; }
.rpg-vitals { position: relative; display: flex; align-items: center; gap: var(--space-xs); width: fit-content; }
.ring { width: var(--size-xl); height: var(--size-xl); flex: none; overflow: hidden; border-radius: 50%; }
.rpg-vitals[data-plate=large] .ring { width: calc(var(--size-xxl) * 1.6); height: calc(var(--size-xxl) * 1.6); }
.portrait { width: stretch; height: stretch; object-fit: cover; object-position: center top; }
.badge { position: absolute; left: 0; top: calc(var(--size-xl) - var(--space-lg)); width: var(--space-xl); height: var(--space-xl); align-items: center; justify-content: center; }
.rpg-vitals[data-plate=large] .badge { top: calc(var(--size-xxl) * 1.6 - var(--size-md)); width: var(--size-md); height: var(--size-md); }
.bars { display: flex; flex-direction: column; gap: var(--space-xxs); width: calc(var(--size-xxl) * 1.4); }
.rpg-vitals[data-plate=large] .bars { width: calc(var(--size-xxl) * 2.6); gap: var(--space-xs); }
.bar { position: relative; display: flex; width: stretch; height: var(--space-sm); align-items: center; justify-content: center; }
.rpg-vitals[data-plate=large] .bar { height: calc(var(--space-md) * 1.6); }
.bar-fill { position: absolute; left: 0; top: 0; width: calc(var(--fill) * 100%); height: stretch; }
.bar-fill:dir(rtl) { left: auto; right: 0; }
.bar-figure { position: relative; font-variant-numeric: tabular-nums; white-space: nowrap; }
@media (width < 600px) {
  .rpg-vitals[data-plate=compact] .bars { width: var(--size-xxl); }
  .rpg-vitals[data-plate=large] .ring { width: calc(var(--size-xxl) * 1.05); height: calc(var(--size-xxl) * 1.05); }
  .rpg-vitals[data-plate=large] .badge { top: calc(var(--size-xxl) * 1.05 - var(--size-md)); }
  .rpg-vitals[data-plate=large] .bars { width: calc(var(--size-xxl) * 1.7); }
  .rpg-vitals[data-plate=large] .bar { height: calc(var(--space-md) * 1.2); }
}
</style>
