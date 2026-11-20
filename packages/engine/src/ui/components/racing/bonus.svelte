<meta name="component" content="RacingBonus">
<meta name="strings" content="./racing.en.ogt">
<meta name="prop:meter" content="string" data-doc="A meter's name as the game words it (Slipstream, Drafting), over a bar in the HUD's accent; empty hides the meter.">
<meta name="prop:value" content="number" data-doc="The meter's share, 0 to 1.">
<meta name="prop:points" content="string" data-doc="The points being earned now as the game words them (+2,450), large and slanted; empty hides them.">
<meta name="prop:detail" content="string" data-doc="What earns them as the game words it (Drift · Near miss ×3), in gold under the points; empty hides it.">
<meta name="prop:at" content="number" data-doc="The chain's id: a new value replays the points' entrance.">
<meta name="fixture:default" content='{"meter":"Slipstream","value":0.72,"points":"+2,450","detail":"Drift · Near miss ×3","at":1}'>
<meta name="fixture:meter" content='{"meter":"Slipstream","value":0.3,"points":"","detail":"","at":0}'>
<meta name="fixture:points" content='{"meter":"","value":0,"points":"+800","detail":"Clean overtake","at":2}'>
<meta name="fixture:empty" content='{"meter":"","value":0,"points":"","detail":"","at":0}'>
<meta name="fixture:overflow" content='{"meter":"Slipstream from the car ahead at full draft","value":4,"points":"+1,234,567,890","detail":"Drift · Near miss ×3 · Airtime · Burnout · Clean section · Perfect landing","at":3}'>

<!--
  Feedback above the road at the top centre: a meter (the slipstream) as
  a word over a bar in the HUD's accent, and the points a drift or a
  chain earns, large and slanted, with what earns them in gold under it.
  The game decides the meter and every figure; read only.
-->
<section id="RacingBonus" part="frame" variant="hud" layout="stack" role="group" aria-label={t("racing.bonus.name")}>
  {#if meter != ''}<div part="meter" class="meter" layout="stack" role="meter" aria-label={meter} aria-valuemin="0" aria-valuemax="100" aria-valuenow={value <= 0 ? 0 : value >= 1 ? 100 : value * 100}>
    <span part="label" class="label">{meter}</span>
    <div part="bar" class="bar" layout="overlay" style:--value={value <= 0 ? 0 : value >= 1 ? 1 : value}><div part="fill" class="fill" anchor="top-start"></div></div>
  </div>{/if}
  {#if points != ''}{#key at}<div class="chain" layout="stack" play="stamp" role="status">
    <span part="points" class="points">{points}</span>
    {#if detail != ''}<span part="detail" class="detail">{detail}</span>{/if}
  </div>{/key}{/if}
</section>

<style>
#RacingBonus { align-items: center; row-gap: calc(22 * var(--font-step-px)); inline-size: fit-content; max-inline-size: stretch; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
.meter { align-items: center; row-gap: calc(6 * var(--font-step-px)); max-inline-size: 100%; }
.label { max-inline-size: 100%; min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-banner-tracking); text-transform: var(--hud-label-transform); color: var(--hud-accent); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.bar { inline-size: calc(180 * var(--font-step-px)); max-inline-size: 100%; block-size: calc(7 * var(--font-step-px)); overflow: hidden; transform: skewX(var(--bar-skew)); background: var(--bar-well); box-shadow: var(--bar-rim); --layout-anchor-inset: 0; }
.fill { inline-size: calc(var(--value) * 100%); block-size: 100%; background: var(--hud-accent-fill); box-shadow: var(--hud-accent-glow); }
.chain { align-items: center; row-gap: calc(2 * var(--font-step-px)); max-inline-size: 100%; min-inline-size: 0; }
/* The points are a figure: never cut, the slant's overhang inside the padding. */
.points { padding-inline: calc(4 * var(--font-step-px)) calc(8 * var(--font-step-px)); font-size: var(--hud-size-lg); line-height: 1; font-style: var(--hud-figure-slant); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.detail { max-inline-size: 100%; min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-gold); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
/* On a phone one compact row: the meter, then the points over what earns
   them. */
@media (width < 600px) {
  #RacingBonus { flex-direction: row; justify-content: center; column-gap: var(--space-md); }
  .meter { flex: 0 1 auto; min-inline-size: 0; }
  .chain { flex: 0 1 auto; align-items: flex-start; }
  .label, .detail { letter-spacing: normal; }
  .bar { inline-size: calc(100 * var(--font-step-px)); block-size: calc(5 * var(--font-step-px)); }
  .points { padding-inline: 0 calc(6 * var(--font-step-px)); font-size: var(--hud-size-sm); }
}
</style>
