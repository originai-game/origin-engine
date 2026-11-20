<meta name="component" content="RpgCombo">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgCombo.frame RpgCombo.count RpgCombo.word RpgCombo.decay RpgCombo.lit RpgCombo.bonus">
<meta name="prop:count" content="number" data-doc="The hits in the chain; 0 hides the counter.">
<meta name="prop:word" content="string" data-doc="The word after the figure as the game words it; empty takes the component's Hits.">
<meta name="prop:remaining" content="number" data-default="-1" data-doc="The share of the chain's window still to run, 0 to 1 (the decay bar); negative hides the bar.">
<meta name="prop:multiplier" content="number" data-default="0" data-doc="The damage multiplier the chain gives (Damage ×1.6); 0 or 1 hides it.">
<meta name="fixture:default" content='{"count":27,"word":"","remaining":0.6,"multiplier":1.6}'>
<meta name="fixture:fading" content='{"count":4,"word":"","remaining":0.08,"multiplier":1}'>
<meta name="fixture:empty" content='{"count":0,"word":"","remaining":-1,"multiplier":0}'>
<meta name="fixture:overflow" content='{"count":1234567,"word":"Uninterrupted strikes","remaining":1,"multiplier":12.5}'>

<!--
  The combo counter beside the fight: the hits as a large slanted figure
  with its word, the window still to run as a decay bar under it and the
  damage multiplier the chain gives. The game owns the chain; read only.
-->
<section id="RpgCombo" part="RpgCombo.frame" class="hud bare" data-layout="stack" data-gap="none" role="status" aria-label={t("rpg.combo.hits", { count: count })}>
  {#if count > 0}
  <div id="Line" data-layout="row" data-gap="tight">
    <span part="RpgCombo.count" class="count" data-motion="pop">{formatNumber(count)}</span>
    <span part="RpgCombo.word" class="word">{word != '' ? word : t("rpg.combo.word")}</span>
  </div>
  {#if remaining >= 0}<div id="Decay" part="RpgCombo.decay" data-layout="overlay" style:--value={remaining >= 1 ? 1 : remaining}><div id="Lit" part="RpgCombo.lit" data-anchor="top-start"></div></div>{/if}
  {#if multiplier > 1}<span part="RpgCombo.bonus" class="bonus">{t("rpg.combo.bonus", { value: multiplier })}</span>{/if}
  {/if}
</section>

<style>
:host { display: grid; grid-template-areas: "line" "decay" "bonus"; justify-items: end; max-inline-size: stretch; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; --layout-anchor-inset: 0; }
#Line { grid-area: line; align-items: baseline; justify-content: flex-end; max-inline-size: 100%; min-inline-size: 0; }
.count { flex: none; font-size: var(--hud-size-combo); line-height: 0.9; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-style: var(--hud-figure-slant); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.word { min-inline-size: 0; font-size: var(--hud-size-md); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-style: var(--hud-figure-slant); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
/* The window drains toward its start. */
#Decay { grid-area: decay; inline-size: calc(160 * var(--font-step-px)); max-inline-size: 100%; block-size: calc(6 * var(--font-step-px)); margin-block-start: calc(6 * var(--font-step-px)); overflow: hidden; transform: skewX(var(--bar-skew)); background: var(--bar-well); box-shadow: var(--bar-rim); }
#Lit { inline-size: calc(var(--value) * 100%); block-size: 100%; background: var(--hud-accent-fill); box-shadow: var(--hud-accent-glow); }
.bonus { grid-area: bonus; margin-block-start: calc(6 * var(--font-step-px)); font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-accent); white-space: nowrap; }
@media (width < 600px), (height < 500px) {
  .count { font-size: var(--hud-size-combo-compact); }
  .word { font-size: var(--hud-size-xs); }
  #Decay { inline-size: calc(110 * var(--font-step-px)); block-size: calc(5 * var(--font-step-px)); margin-block-start: calc(4 * var(--font-step-px)); }
  .bonus { margin-block-start: calc(3 * var(--font-step-px)); letter-spacing: normal; }
}
</style>
