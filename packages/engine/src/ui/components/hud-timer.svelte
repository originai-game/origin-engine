<meta name="component" content="HudTimer">
<meta name="strings" content="./hud-timer.en.ogt">
<meta name="prop:seconds" content="number" data-doc="Seconds on the clock.">
<meta name="prop:total" content="number" data-doc="A countdown's whole time: a ring of what is left beside it (0: none).">
<meta name="prop:countdown" content="bool" data-doc="Counts down (warns and turns red near zero); false is a stopwatch.">
<meta name="prop:warnAt" content="number" data-doc="Seconds left when it turns warn; a third of it is critical.">
<meta name="prop:paused" content="bool" data-doc="Paused: the time dims and breathes.">
<meta name="prop:label" content="string" data-doc="A caption over it (Round 2); empty is none.">
<meta name="fixture:default" content='{"seconds":8.4,"total":90,"countdown":true,"warnAt":10,"paused":false,"label":"Round 2"}'>

<div id="HudTimer" role="timer" aria-label={label || t("component.hud-timer.name")} aria-valuetext={formatCountdown(seconds)} class:warn={countdown && seconds <= warnAt && seconds > warnAt / 3} class:critical={countdown && seconds <= warnAt / 3} class:paused={paused}>
  <span id="Label" hidden={!label}>{label}</span>
  <div id="Body">
    <svg id="Ring" viewBox="0 0 22 22" hidden={!(countdown && total > 0)}>
      <circle cx="11" cy="11" r="9.5" fill="none" stroke="#00000099" stroke-width="3"></circle>
      <circle id="Left" cx="11" cy="11" r="9.5" fill="none" stroke-width="3" pathLength="1" stroke-dasharray={(seconds / total) + " 1"} transform="rotate(-90 11 11)"></circle>
    </svg>
    <span id="Time">{formatCountdown(seconds)}</span>
  </div>
</div>

<style>
#HudTimer { align-items: center; padding: 4px 12px; border-radius: 8px; background: var(--hud); --clock: var(--ink); }
#HudTimer.warn { --clock: var(--warn); }
#HudTimer.critical { --clock: var(--danger); }
#Label { font: var(--text-caption); color: var(--ink-muted); white-space: nowrap; }
#Body { display: flex; gap: 8px; align-items: center; }
#Ring { width: 22px; height: 22px; }
#Left { stroke: var(--clock); }
#Time { font: var(--text-xl2); color: var(--clock); white-space: nowrap; text-shadow: 0 1px 3px #000000cc; }
.critical #Time { animation: beat 1000ms ease-out infinite; }
.paused #Time { animation: breathe 1000ms ease-in-out infinite alternate; }
@keyframes beat { 0% { scale: 1.14; } 25% { scale: 1; } }
@keyframes breathe { from { opacity: 1; } to { opacity: 0.45; } }
</style>
