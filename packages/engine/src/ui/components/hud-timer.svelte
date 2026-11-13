<meta name="component" content="HudTimer">
<meta name="strings" content="./hud-timer.en.ogt">
<meta name="parts" content="HudTimer.frame HudTimer.label HudTimer.ring HudTimer.figure">
<meta name="prop:seconds" content="number" data-doc="Seconds on the clock.">
<meta name="prop:total" content="number" data-doc="A countdown's whole time: a ring of what is left beside it (0: none).">
<meta name="prop:countdown" content="bool" data-doc="Counts down (warns and turns red near zero); false is a stopwatch.">
<meta name="prop:warnAt" content="number" data-doc="Seconds left when it turns warn; a third of it is critical.">
<meta name="prop:paused" content="bool" data-doc="Paused: the time dims and breathes.">
<meta name="prop:label" content="string" data-doc="A caption over it (Round 2); empty is none.">
<meta name="fixture:default" content='{"seconds":8.4,"total":90,"countdown":true,"warnAt":10,"paused":false,"label":"Round 2"}'>

<div id="HudTimer" part="HudTimer.frame" class="hud" role="timer" aria-label={label || t("component.hud-timer.name")} aria-valuetext={formatCountdown(seconds)} class:warn={countdown && seconds <= warnAt && seconds > warnAt / 3} class:critical={countdown && seconds <= warnAt / 3} class:paused={paused}>
  <span id="Label" part="HudTimer.label" hidden={!label}>{label}</span>
  <div id="Body">
    <svg id="Ring" part="HudTimer.ring" viewBox="0 0 22 22" hidden={!(countdown && total > 0)}>
      <circle class="track" cx="11" cy="11" r="9.5" fill="none" stroke-width="3"></circle>
      <circle id="Left" cx="11" cy="11" r="9.5" fill="none" stroke-width="3" pathLength="1" stroke-dasharray={(seconds / total) + " 1"} transform="rotate(-90 11 11)"></circle>
    </svg>
    <span id="Time" part="HudTimer.figure">{formatCountdown(seconds)}</span>
  </div>
</div>

<style>
[part="HudTimer.frame"] { align-items: center; --clock: var(--ink); }
[part="HudTimer.frame"].warn { --clock: var(--warn); }
[part="HudTimer.frame"].critical { --clock: var(--danger); }
[part="HudTimer.label"] { font: var(--text-caption); white-space: nowrap; }
#Body { display: flex; gap: var(--space-sm); align-items: center; }
[part="HudTimer.ring"] { width: var(--size-sm); height: var(--size-sm); }
.track { stroke: var(--well); }
#Left { stroke: var(--clock); }
[part="HudTimer.figure"] { font: var(--text-xl2); color: var(--clock); white-space: nowrap; }
.critical [part="HudTimer.figure"] { animation: beat calc(var(--motion-duration-slow) * 2.5) var(--motion-easing-out) infinite; }
.paused [part="HudTimer.figure"] { animation: breathe calc(var(--motion-duration-slow) * 2.5) var(--motion-easing-in-out) infinite alternate; }
@keyframes beat { 0% { scale: var(--motion-scale-swell); } 25% { scale: 1; } }
@keyframes breathe { from { opacity: 1; } to { opacity: var(--motion-scale-breathe); } }
</style>
