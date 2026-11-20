<meta name="component" content="RpgVitals">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgVitals.frame RpgVitals.picture RpgVitals.badge RpgVitals.track RpgVitals.fill">
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
<div id="RpgVitals" part="RpgVitals.frame" class="bare" data-layout="row" data-plate={plate} data-down={down} role="group" aria-label={name}>
  <div class="ring" part="RpgVitals.picture">{#if portrait}<img class="portrait" src={portrait} alt="">{/if}</div>
  {#if level}<div class="badge" part="RpgVitals.badge" data-layout="cover"><span class="badge-level">{formatNumber(level)}</span></div>{/if}
  <div class="bars" data-layout="stack" data-gap={plate == 'large' ? 'tight' : 'hair'}>
    <div class="bar health" part="RpgVitals.track" data-layout="row" role="meter" aria-label={t("rpg.party.health")} aria-valuetext={t("rpg.party.figure", { now: health, max: healthMax })} style:--fill={healthMax > 0 ? health / healthMax : 0}><div class="bar-fill" part="RpgVitals.fill"></div>{#if plate == 'large'}<div class="bar-figure" part="RpgVitals.badge"><span>{t("rpg.party.figure", { now: health, max: healthMax })}</span></div>{/if}</div>
    {#if manaMax}<div class="bar mana" part="RpgVitals.track" data-layout="row" role="meter" aria-label={t("rpg.party.mana")} aria-valuetext={t("rpg.party.figure", { now: mana, max: manaMax })} style:--fill={mana / manaMax}><div class="bar-fill" part="RpgVitals.fill"></div>{#if plate == 'large'}<div class="bar-figure" part="RpgVitals.badge"><span>{t("rpg.party.figure", { now: mana, max: manaMax })}</span></div>{/if}</div>{/if}
  </div>
</div>

<style>
/* Drawn over the scene: the style's tag halo holds its text there. */
:host { text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
:host { position: relative; inline-size: fit-content; }
.ring { width: var(--size-xl); height: var(--size-xl); flex: none; overflow: hidden; --picture-radius: var(--radius-capsule); }
[part="RpgVitals.frame"][data-plate=large] .ring { width: calc(var(--size-xxl) * 1.6); height: calc(var(--size-xxl) * 1.6); }
.portrait { width: stretch; height: stretch; object-fit: cover; object-position: center top; }
.badge { position: absolute; inset-inline-start: 0; top: calc(var(--size-xl) - var(--space-lg)); width: var(--space-xl); height: var(--space-xl); }
[part="RpgVitals.frame"][data-plate=large] .badge { top: calc(var(--size-xxl) * 1.6 - var(--size-md)); width: var(--size-md); height: var(--size-md); }
.bars { width: calc(var(--size-xxl) * 1.4); }
[part="RpgVitals.frame"][data-plate=large] .bars { width: calc(var(--size-xxl) * 2.6); }
.bar { position: relative; height: var(--space-sm); justify-content: center; overflow: hidden; --fill-color: var(--health); }
[part="RpgVitals.frame"][data-plate=large] .bar { height: calc(var(--space-md) * 1.6); min-block-size: var(--size-sm); }
.bar-fill { position: absolute; inset-inline-start: 0; top: 0; bottom: 0; inline-size: calc(var(--fill) * 100%); }
.mana { --fill-color: var(--mana); }
.bar-figure { position: relative; white-space: nowrap; padding-inline: var(--space-xs); }
@media (width < 600px), (height < 500px) {
  [part="RpgVitals.frame"][data-plate=compact] .bars { width: var(--size-xxl); }
  [part="RpgVitals.frame"][data-plate=large] .ring { width: calc(var(--size-xxl) * 1.05); height: calc(var(--size-xxl) * 1.05); }
  [part="RpgVitals.frame"][data-plate=large] .badge { top: calc(var(--size-xxl) * 1.05 - var(--size-md)); }
  [part="RpgVitals.frame"][data-plate=large] .bars { width: calc(var(--size-xxl) * 1.7); }
  [part="RpgVitals.frame"][data-plate=large] .bar { height: calc(var(--space-md) * 1.2); }
}
</style>
