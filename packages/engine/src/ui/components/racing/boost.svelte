<meta name="component" content="RacingBoost">
<meta name="strings" content="./racing.en.ogt">
<meta name="parts" content="RacingBoost.frame RacingBoost.icon RacingBoost.label RacingBoost.key RacingBoost.bar RacingBoost.fill">
<meta name="prop:label" content="string" data-doc="The meter's name as the game words it (Nitro, ERS, Boost); empty takes the kit's (Boost).">
<meta name="prop:icon" content="number" data-doc="The meter's picture (og.load.texture) in a cut tile before it; 0 hides it.">
<meta name="prop:key" content="string" data-doc="The input that fires it as the game names it (Shift), on a key cap after the label; empty for none; hidden under a touch screen.">
<meta name="prop:value" content="number" data-doc="What is left now.">
<meta name="prop:max" content="number" data-default="1" data-doc="The most there can be.">
<meta name="prop:cells" content="number" data-default="5" data-doc="The cells the bar is cut into (5); 1 draws one bar.">
<meta name="prop:firing" content="bool" data-doc="It is firing now: the fill glows (data-firing).">
<meta name="fixture:default" content='{"label":"Nitro","icon":0,"key":"Shift","value":60,"max":100,"cells":5,"firing":false}'>
<meta name="fixture:firing" content='{"label":"Nitro","icon":0,"key":"Shift","value":92,"max":100,"cells":5,"firing":true}'>
<meta name="fixture:empty" content='{"label":"ERS","icon":0,"key":"","value":0,"max":100,"cells":1,"firing":false}'>
<meta name="fixture:overflow" content='{"label":"Energy Recovery System Deployment","icon":0,"key":"Left Shoulder + Right Trigger","value":250,"max":100,"cells":40,"firing":true}'>

<!--
  A boost meter (nitro, ERS) beside the tachometer: its picture in a cut
  tile, the name with the key that fires it, and a leaning bar cut into
  cells, glowing while it fires. The game owns the store and when it
  fires; read only.
-->
<section id="RacingBoost" part="RacingBoost.frame" class="hud bare" data-layout="row" role="meter" aria-label={label != '' ? label : t("racing.boost.name")} aria-valuemin="0" aria-valuemax={max > 0 ? max : 1} aria-valuenow={value <= 0 ? 0 : value} data-firing={firing}>
  {#if icon}<img part="RacingBoost.icon" class="icon" src={icon} alt="">{/if}
  <div class="col" data-layout="stack">
    <div class="head" data-layout="row">
      <span part="RacingBoost.label" class="label">{label != '' ? label : t("racing.boost.name")}</span>
      {#if key != ''}<span part="RacingBoost.key" class="key">{key}</span>{/if}
    </div>
    <div part="RacingBoost.bar" class="bar" data-layout="overlay" style:--value={max <= 0 || value <= 0 ? 0 : value >= max ? 1 : value / max} style:--cells={cells > 0 ? cells : 1}>
      <div part="RacingBoost.fill" class="fill" data-anchor="top-start"></div>
      <div class="cuts" data-anchor="fill"></div>
    </div>
  </div>
</section>

<style>
:host { align-items: center; column-gap: calc(10 * var(--font-step-px)); inline-size: calc(240 * var(--font-step-px)); max-inline-size: stretch; color: var(--hud-ink); }
.icon { flex: none; inline-size: calc(34 * var(--font-step-px)); block-size: calc(34 * var(--font-step-px)); object-fit: cover; border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; filter: var(--hud-lift); }
.col { flex: 1 1 auto; min-inline-size: 0; row-gap: calc(5 * var(--font-step-px)); }
.head { align-items: center; column-gap: calc(8 * var(--font-step-px)); min-inline-size: 0; }
.label { flex: 0 1 auto; min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ally); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
.key { flex: 0 1 auto; min-inline-size: calc(22 * var(--font-step-px)); max-inline-size: 60%; padding: 0 var(--space-xs); font-size: var(--hud-tag-size); line-height: 1.5; font-weight: var(--hud-figure-weight); text-align: center; color: var(--hud-key-ink); background: var(--hud-key-fill); border-radius: var(--hud-key-radius); box-shadow: var(--hud-key-shadow); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
@media (pointer: coarse) { .key { display: none; } }
/* The bar: a leaning well, the fill in the shield's blue, cut into cells. */
.bar { inline-size: stretch; block-size: calc(12 * var(--font-step-px)); overflow: hidden; transform: skewX(var(--bar-skew)); background: var(--bar-well); box-shadow: var(--bar-rim); --layout-anchor-inset: 0; }
.fill { inline-size: calc(var(--value) * 100%); block-size: 100%; background: var(--hud-shield-fill); }
[data-firing=true] .fill { box-shadow: var(--hud-shield-glow); }
.cuts { background: var(--bar-notch); mask-image: repeating-linear-gradient(90deg, transparent 0 calc(100% / var(--cells) - 3 * var(--font-step-px)), #000 calc(100% / var(--cells) - 3 * var(--font-step-px)) calc(100% / var(--cells))); }
@media (width < 600px), (height < 500px) {
  :host { inline-size: stretch; column-gap: calc(6 * var(--font-step-px)); }
  .icon { inline-size: calc(24 * var(--font-step-px)); block-size: calc(24 * var(--font-step-px)); }
  .label { letter-spacing: normal; }
  .bar { block-size: calc(8 * var(--font-step-px)); }
}
</style>
