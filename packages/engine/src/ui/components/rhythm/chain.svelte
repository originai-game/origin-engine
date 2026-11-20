<meta name="component" content="RhythmChain">
<meta name="strings" content="./rhythm.en.ogt">
<meta name="prop:multiplier" content="string" data-doc="The score multiplier as the game words it (×4); empty hides the chain.">
<meta name="prop:step" content="number" data-doc="How many of the steps to the next multiplier are lit.">
<meta name="prop:steps" content="number" data-default="8" data-doc="The steps to the next multiplier; 0 hides the strip (at its top multiplier).">
<meta name="prop:hit" content="number" data-doc="Changes when the multiplier rises: the plate pops again.">
<meta name="fixture:default" content='{"multiplier":"×4","step":6,"steps":8,"hit":1}'>
<meta name="fixture:empty" content='{"multiplier":"×1","step":0,"steps":8,"hit":0}'>
<meta name="fixture:top" content='{"multiplier":"×8","step":0,"steps":0,"hit":3}'>
<meta name="fixture:none" content='{"multiplier":"","step":0,"steps":8,"hit":0}'>
<meta name="fixture:overflow" content='{"multiplier":"×128.75","step":19,"steps":20,"hit":4}'>

<!--
  The score chain: a lit plate with the multiplier, and beside it the
  steps to the next one as leaning pips. The game counts both; read only.
-->
<section id="RhythmChain" part="frame" variant="hud" layout="row" gap="tight" role="group" aria-label={t("rhythm.chain.name", { multiplier: multiplier })}>
  {#if multiplier != ''}
  {#key hit}
  <div id="Plate" layout="row" gap="tight" play="pop">
    <span part="legend" class="label">{t("rhythm.chain.label")}</span>
    <span part="multiplier" class="multiplier">{multiplier}</span>
  </div>
  {/key}
  {#if steps > 0}
  <div id="Steps" part="steps" layout="overlay" role="meter" aria-label={t("rhythm.chain.steps")} aria-valuemin="0" aria-valuemax={steps} aria-valuenow={step < 0 ? 0 : step > steps ? steps : step} style:--pips={steps} style:--lit={step <= 0 ? 0 : step >= steps ? 1 : step / steps}>
    <div id="Lit" part="step" anchor="top-start"></div>
  </div>
  {/if}
  {/if}
</section>

<style>
#RhythmChain { align-items: center; max-inline-size: stretch; --layout-anchor-inset: 0; }
/* The plate: smoked glass, two corners cut, a lit top edge in the accent. */
#Plate { flex: none; align-items: baseline; padding: var(--space-xxs) var(--space-md); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-lit-rim); border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.label { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; }
.multiplier { font-size: var(--hud-size-md); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-accent); white-space: nowrap; }
/* The pips: one leaning strip cut into cells; the lit part is the steps
   taken. */
#Steps { flex: 0 1 auto; min-inline-size: 0; inline-size: calc(var(--pips) * 15 * var(--font-step-px)); block-size: calc(5 * var(--font-step-px)); transform: skewX(var(--bar-skew)); background: var(--hud-pip-off); mask-image: repeating-linear-gradient(90deg, #000 0 calc(12 * var(--font-step-px)), transparent calc(12 * var(--font-step-px)) calc(15 * var(--font-step-px))); }
#Lit { inline-size: calc(var(--lit) * 100%); block-size: 100%; background: var(--hud-pip-on); box-shadow: var(--hud-pip-glow); }
</style>
