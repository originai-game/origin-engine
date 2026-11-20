<meta name="component" content="RpgBar">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgBar.groove RpgBar.ghost RpgBar.lit RpgBar.cuts">
<meta name="prop:value" content="number" data-doc="The value now.">
<meta name="prop:max" content="number" data-default="1" data-doc="Its most; 0 draws the bar empty.">
<meta name="prop:kind" content="string" data-default="health" data-doc="What it measures, which is its fill: health, mana, stamina or xp.">
<meta name="prop:ghost" content="number" data-doc="The value before the latest loss: a warm ghost behind the fill that drains after it (at or under value: none).">
<meta name="prop:segment" content="number" data-doc="Value per cell (one dodge's cost): the bar is cut into cells; 0 draws it whole.">
<meta name="prop:low" content="bool" data-doc="Low: the fill turns to danger and the rim glows.">
<meta name="prop:name" content="string" data-doc="What a screen reader calls it; empty is its kind (Health, Mana, Stamina, Experience).">
<meta name="fixture:default" content='{"value":2184,"max":3420,"kind":"health","ghost":2668,"segment":0,"low":false,"name":""}'>
<meta name="fixture:mana" content='{"value":72,"max":100,"kind":"mana","ghost":0,"segment":0,"low":false,"name":""}'>
<meta name="fixture:cells" content='{"value":5,"max":8,"kind":"stamina","ghost":0,"segment":1,"low":false,"name":""}'>
<meta name="fixture:low" content='{"value":42,"max":3420,"kind":"health","ghost":300,"segment":0,"low":true,"name":""}'>
<meta name="fixture:empty" content='{"value":0,"max":0,"kind":"health","ghost":0,"segment":0,"low":false,"name":""}'>
<meta name="fixture:overflow" content='{"value":999999,"max":10,"kind":"xp","ghost":2000000,"segment":3,"low":false,"name":"Experience towards the next level of the Ninefold Gate"}'>

<!--
  One bar of the RPG HUD, a block a plate is built from: it leans in a
  dark well with a lit rim and fills to value over max in its kind's
  colour; a loss leaves a warm ghost that drains after the fill, a low bar
  turns to danger, and segment cuts it into cells. It is as wide as its
  place and --rpg-bar-size thick; a plate sets both on the place it gives
  the bar. Read only.
-->
<div id="RpgBar" part="RpgBar.groove" data-layout="overlay" role="meter" aria-label={name != '' ? name : kind == 'mana' ? t("rpg.party.mana") : kind == 'stamina' ? t("rpg.leader.stamina") : kind == 'xp' ? t("rpg.xp.name") : t("rpg.party.health")} aria-valuemin="0" aria-valuemax={max > 0 ? max : 1} aria-valuenow={value <= 0 ? 0 : max > 0 && value >= max ? max : value} class:mana={kind == 'mana'} class:stamina={kind == 'stamina'} class:xp={kind == 'xp'} class:low={low} style:--value={max <= 0 || value <= 0 ? 0 : value >= max ? 1 : value / max} style:--ghost={max <= 0 || ghost <= value ? 0 : ghost >= max ? 1 : ghost / max} style:--cells={segment > 0 && max > 0 ? max / segment : 1}>
  <div id="Ghost" part="RpgBar.ghost" data-anchor="top-start"></div>
  <div id="Lit" part="RpgBar.lit" data-anchor="top-start"></div>
  {#if segment > 0 && max > 0}<div id="Cuts" part="RpgBar.cuts" data-anchor="fill"></div>{/if}
</div>

<style>
:host { inline-size: 100%; block-size: var(--rpg-bar-size); overflow: hidden; transform: skewX(var(--bar-skew)); background: var(--bar-well); box-shadow: var(--bar-rim); --layout-anchor-inset: 0; }
#Ghost { inline-size: calc(var(--ghost) * 100%); block-size: 100%; background: var(--bar-ghost); transition: var(--bar-ghost-motion); }
#Lit { inline-size: calc(var(--value) * 100%); block-size: 100%; background: var(--hud-life-fill); box-shadow: var(--hud-life-glow); }
.mana #Lit { background: var(--hud-mana-fill); box-shadow: var(--hud-mana-glow); }
.stamina #Lit { background: var(--hud-stamina-fill); box-shadow: var(--hud-stamina-glow); }
.xp #Lit { background: var(--hud-xp-fill); box-shadow: var(--hud-xp-glow); }
/* The cells are cut across the whole well, lit or not. */
#Cuts { background: var(--bar-notch); mask-image: repeating-linear-gradient(90deg, transparent 0 calc(100% / var(--cells) - 3 * var(--font-step-px)), #000 calc(100% / var(--cells) - 3 * var(--font-step-px)) calc(100% / var(--cells))); }
.low #Lit { background: var(--hud-low-fill); box-shadow: var(--hud-low-glow); }
#RpgBar.low { box-shadow: var(--bar-critical-rim); animation: var(--pulse-motion); }
@media (prefers-reduced-motion: reduce) { #RpgBar.low { animation: none; } }
</style>
