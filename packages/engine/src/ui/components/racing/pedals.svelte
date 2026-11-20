<meta name="component" content="RacingPedals">
<meta name="strings" content="./racing.en.ogt">
<meta name="prop:throttle" content="number" data-doc="The throttle's travel now, 0 to 1.">
<meta name="prop:brake" content="number" data-doc="The brake's travel now, 0 to 1.">
<meta name="fixture:default" content='{"throttle":0.86,"brake":0.12}'>
<meta name="fixture:braking" content='{"throttle":0,"brake":1}'>
<meta name="fixture:empty" content='{"throttle":0,"brake":0}'>
<meta name="fixture:overflow" content='{"throttle":4,"brake":-2}'>

<!--
  The pedals at the bottom left: throttle and brake as two leaning upright
  wells filled from the foot, green and red, each named under it. Read
  only.
-->
<section id="RacingPedals" part="frame" variant="hud" layout="row" role="group" aria-label={t("racing.pedals.name")}>
  <div part="pedal" class="pedal" layout="stack" class:pedal-throttle={true}>
    <div part="well" class="well" layout="overlay" role="meter" aria-label={t("racing.pedals.throttleName")} aria-valuemin="0" aria-valuemax="100" aria-valuenow={throttle <= 0 ? 0 : throttle >= 1 ? 100 : throttle * 100} style:--value={throttle <= 0 ? 0 : throttle >= 1 ? 1 : throttle}><div part="fill" class="fill" anchor="bottom-start"></div></div>
    <span part="label" class="label">{t("racing.pedals.throttle")}</span>
  </div>
  <div part="pedal" class="pedal" layout="stack" class:pedal-brake={true}>
    <div part="well" class="well" layout="overlay" role="meter" aria-label={t("racing.pedals.brakeName")} aria-valuemin="0" aria-valuemax="100" aria-valuenow={brake <= 0 ? 0 : brake >= 1 ? 100 : brake * 100} style:--value={brake <= 0 ? 0 : brake >= 1 ? 1 : brake}><div part="fill" class="fill" anchor="bottom-start"></div></div>
    <span part="label" class="label">{t("racing.pedals.brake")}</span>
  </div>
</section>

<style>
#RacingPedals { align-items: flex-end; column-gap: calc(12 * var(--font-step-px)); color: var(--hud-ink); }
.pedal { align-items: center; row-gap: calc(5 * var(--font-step-px)); }
.well { inline-size: calc(12 * var(--font-step-px)); block-size: calc(58 * var(--font-step-px)); overflow: hidden; transform: skewX(-8deg); background: var(--bar-well); box-shadow: var(--hud-pedal-rim); --layout-anchor-inset: 0; }
.fill { inline-size: 100%; block-size: calc(var(--value) * 100%); }
.pedal-throttle .fill { background: var(--hud-shift-low); }
.pedal-brake .fill { background: var(--hud-harm); }
.label { font-size: var(--hud-tag-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
#RacingPedals:size(compact) { column-gap: calc(8 * var(--font-step-px)); }
.well:size(compact) { inline-size: calc(9 * var(--font-step-px)); block-size: calc(40 * var(--font-step-px)); }
.label:size(compact) { letter-spacing: normal; }
</style>
