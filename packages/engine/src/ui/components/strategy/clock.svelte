<meta name="component" content="StrategyClock">
<meta name="strings" content="./strategy.en.ogt">
<meta name="parts" content="StrategyClock.frame StrategyClock.label StrategyClock.time StrategyClock.speed">
<meta name="prop:label" content="string" data-doc="What the clock counts as the game words it (Day 3, Match), before the time; empty hides it.">
<meta name="prop:time" content="string" data-doc="The time as the game words it (12:34).">
<meta name="prop:speed" content="string" data-doc="The game's speed as it words it (×1, ×2), on a tag after the time; empty hides it.">
<meta name="prop:paused" content="bool" data-doc="The game is paused: the Paused word stands on the tag in place of the speed (data-paused).">
<meta name="fixture:default" content='{"label":"Day 3","time":"12:34","speed":"×1","paused":false}'>
<meta name="fixture:paused" content='{"label":"Day 3","time":"12:34","speed":"×1","paused":true}'>
<meta name="fixture:empty" content='{"label":"","time":"0:00","speed":"","paused":false}'>
<meta name="fixture:overflow" content='{"label":"Third Day of the Long Winter Campaign","time":"1,284:59:59","speed":"×16","paused":false}'>

<!--
  The match clock at the top right: what it counts (Day 3), the time large
  and the game's speed on a tag, or Paused. The game owns the time and the
  speed; read only.
-->
<section id="StrategyClock" part="StrategyClock.frame" class="hud" role="timer" aria-label={label != '' ? label : t("strategy.clock.name")} aria-valuetext={time} data-paused={paused}>
  {#if label != ''}<span part="StrategyClock.label" class="label">{label}</span>{/if}
  <span part="StrategyClock.time" class="time">{time}</span>
  {#if paused}<span part="StrategyClock.speed" class="speed" role="status">{t("strategy.clock.paused")}</span>{:else if speed != ''}<span part="StrategyClock.speed" class="speed">{speed}</span>{/if}
</section>

<style>
:host { display: flex; align-items: center; column-gap: calc(10 * var(--font-step-px)); inline-size: fit-content; max-inline-size: stretch; min-inline-size: 0; padding: calc(6 * var(--font-step-px)) calc(14 * var(--font-step-px)); color: var(--hud-ink); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.label { flex: 0 1 auto; min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.time { flex: none; font-size: var(--hud-size-readout); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-shadow: var(--hud-shadow); }
.speed { flex: none; min-inline-size: calc(22 * var(--font-step-px)); padding: 0 calc(6 * var(--font-step-px)); font-size: var(--hud-tag-size); line-height: 1.5; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); text-align: center; white-space: nowrap; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
[data-paused=true] .speed { color: var(--hud-gold); }
@media (width < 600px) {
  :host { inline-size: stretch; column-gap: calc(6 * var(--font-step-px)); padding: calc(4 * var(--font-step-px)) calc(10 * var(--font-step-px)); }
  .label { font-size: var(--hud-size-label-compact); letter-spacing: normal; }
  .time { font-size: var(--hud-size-readout-compact); }
}
</style>
