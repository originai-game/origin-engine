<meta name="component" content="RhythmGauge">
<meta name="strings" content="./rhythm.en.ogt">
<meta name="parts" content="RhythmGauge.frame RhythmGauge.meter RhythmGauge.level RhythmGauge.legend RhythmGauge.key RhythmGauge.state">
<meta name="prop:kind" content="string" data-default="life" data-doc="What it measures: life (the run fails at empty) or fever (a charge the player spends when full); on data-kind.">
<meta name="prop:value" content="number" data-doc="How full, 0 to 1.">
<meta name="prop:side" content="string" data-default="start" data-doc="The screen edge it stands at, start or end: it leans toward the play field.">
<meta name="prop:danger" content="bool" data-doc="The game says life is low: the gauge turns to danger, its rim pulses and Danger shows under it.">
<meta name="prop:failed" content="bool" data-doc="The game says the run is failed (life gone): Failed shows under it.">
<meta name="prop:ready" content="bool" data-doc="The game says the charge is ready to spend: the gauge glows and Ready shows.">
<meta name="prop:key" content="string" data-doc="The input that spends it as the game names it (Space, RB); empty hides the key cap.">
<meta name="fixture:default" content='{"kind":"life","value":0.78,"side":"start","danger":false,"failed":false,"ready":false,"key":""}'>
<meta name="fixture:fever" content='{"kind":"fever","value":0.92,"side":"end","danger":false,"failed":false,"ready":false,"key":"Space"}'>
<meta name="fixture:ready" content='{"kind":"fever","value":1,"side":"end","danger":false,"failed":false,"ready":true,"key":"Space"}'>
<meta name="fixture:danger" content='{"kind":"life","value":0.14,"side":"start","danger":true,"failed":false,"ready":false,"key":""}'>
<meta name="fixture:failed" content='{"kind":"life","value":0,"side":"start","danger":true,"failed":true,"ready":false,"key":""}'>
<meta name="fixture:overflow" content='{"kind":"fever","value":1.4,"side":"end","danger":false,"failed":false,"ready":true,"key":"Left Shoulder + Right Shoulder"}'>

<!--
  A tall gauge at a screen edge, cut into ten cells and leaning toward the
  play field: the run's life (green; danger red with a pulsing rim under
  the game's mark, Danger and Failed in words) or its fever charge (pink;
  Ready and the key that spends it). Its height is the space the page
  gives it. The game owns the value and every verdict; read only.
-->
<section id="RhythmGauge" part="RhythmGauge.frame" class="hud bare" data-layout="stack" data-gap="tight" data-kind={kind} data-side={side} data-danger={danger} data-ready={ready}>
  <div id="Meter" part="RhythmGauge.meter" data-layout="overlay" role="meter" aria-label={kind == 'fever' ? t("rhythm.gauge.fever") : t("rhythm.gauge.life")} aria-valuemin="0" aria-valuemax="100" aria-valuenow={value <= 0 ? 0 : value >= 1 ? 100 : value * 100} style:--value={value <= 0 ? 0 : value >= 1 ? 1 : value} class:danger={danger} class:ready={ready}>
    <div id="Level" part="RhythmGauge.level" data-anchor="bottom-start" data-kind={kind}></div>
    <div id="Cuts" data-anchor="fill"></div>
  </div>
  <span part="RhythmGauge.legend" class="label">{kind == 'fever' ? t("rhythm.gauge.fever") : t("rhythm.gauge.life")}</span>
  {#if failed}<span part="RhythmGauge.state" class="state" data-state="failed" role="status">{t("rhythm.gauge.failed")}</span>
  {:else}{#if danger}<span part="RhythmGauge.state" class="state" data-state="danger" role="status">{t("rhythm.gauge.danger")}</span>{/if}{/if}
  {#if ready}<span part="RhythmGauge.state" class="state" data-state="ready" role="status">{t("rhythm.gauge.ready")}</span>{/if}
  {#if key != ''}<span part="RhythmGauge.key" class="key" data-ready={ready}>{key}</span>{/if}
</section>

<style>
:host { align-items: center; block-size: stretch; min-block-size: calc(120 * var(--font-step-px)); max-inline-size: calc(96 * var(--font-step-px)); color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; --layout-anchor-inset: 0; }
/* The well: tall, narrow, leaning toward the field (mirrored at the end
   edge); the cells are cut from the level by a mask, so the well shows
   between them. */
#Meter { flex: 1; min-block-size: 0; inline-size: calc(26 * var(--font-step-px)); overflow: hidden; transform: skewY(calc(var(--gauge-lean) * -1)); background: var(--gauge-well); box-shadow: var(--gauge-rim); }
[data-side=end] #Meter { transform: skewY(var(--gauge-lean)); }
#Level { inline-size: 100%; block-size: calc(var(--value) * 100%); background: var(--gauge-life-fill); box-shadow: var(--gauge-life-glow); }
/* Ten cells: cuts across the whole well, so a level stops between them. */
#Cuts { background: var(--gauge-cut); mask-image: repeating-linear-gradient(0deg, transparent 0 calc(10% - 3 * var(--font-step-px)), #000 calc(10% - 3 * var(--font-step-px)) 10%); }
#Level[data-kind=fever] { background: var(--gauge-fever-fill); box-shadow: var(--gauge-fever-glow); }
.danger #Level { background: var(--gauge-danger-fill); }
.danger { box-shadow: var(--bar-critical-rim); animation: var(--pulse-motion); }
.ready { box-shadow: var(--gauge-fever-glow); }
.label { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; }
[data-kind=fever] .label { color: var(--gauge-fever-ink); }
.state { font-size: var(--hud-label-size); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); white-space: nowrap; }
.state[data-state=danger], .state[data-state=failed] { color: var(--trend-down); }
.state[data-state=ready] { color: var(--gauge-fever-ink); }
/* The key cap that spends the charge. */
.key { max-inline-size: stretch; padding: 0 var(--space-xs); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); text-align: center; color: var(--hud-key-ink); background: var(--hud-key-fill); border-radius: var(--hud-key-radius); box-shadow: var(--hud-key-shadow); -webkit-text-stroke-width: 0; text-shadow: none; overflow-wrap: anywhere; }
.key[data-ready=true] { background: var(--hud-key-lit-fill); box-shadow: var(--hud-key-lit-shadow); }
@media (width < 600px), (height < 500px) {
  :host { min-block-size: 0; }
  #Meter { inline-size: calc(16 * var(--font-step-px)); }
}
@media (prefers-reduced-motion: reduce) { .danger { animation: none; } }
</style>
