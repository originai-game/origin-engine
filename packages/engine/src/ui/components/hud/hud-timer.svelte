<meta name="component" content="HudTimer">
<meta name="strings" content="./hud-timer.en.ogt">
<meta name="parts" content="HudTimer.frame HudTimer.label HudTimer.ring HudTimer.figure HudTimer.caption HudTimer.track HudTimer.fill">
<meta name="prop:seconds" content="number" data-doc="Seconds on the clock.">
<meta name="prop:total" content="number" data-doc="A countdown's whole time: a ring of what is left beside it (0: none).">
<meta name="prop:countdown" content="bool" data-doc="Counts down (warns and turns red near zero); false is a stopwatch.">
<meta name="prop:warnAt" content="number" data-doc="Seconds left when it turns warn; a third of it is critical.">
<meta name="prop:paused" content="bool" data-doc="Paused: the time dims and breathes.">
<meta name="prop:label" content="string" data-doc="A caption over it (Round 2); empty is none.">
<meta name="fixture:default" content='{"seconds":8.4,"total":90,"countdown":true,"warnAt":10,"paused":false,"label":"Round 2"}'>

<div id="HudTimer" part="HudTimer.frame" class="hud" data-layout="stack" data-gap="hair" role="timer" aria-label={label || t("component.hud-timer.name")} aria-valuetext={formatCountdown(seconds)} class:warn={countdown && seconds <= warnAt && seconds > warnAt / 3} class:critical={countdown && seconds <= warnAt / 3} class:paused={paused}>
  {#if label}<span id="Label" class="label" part="HudTimer.label HudTimer.caption">{label}</span>{/if}
  <div id="Body" data-layout="row">
    {#if countdown && total > 0}<svg id="Ring" part="HudTimer.ring" viewBox="0 0 22 22">
      <circle class="track" part="HudTimer.track" cx="11" cy="11" r="9.5" fill="none" stroke-width="3"></circle>
      <circle id="Left" part="HudTimer.fill" data-tone={countdown && seconds <= warnAt ? seconds <= warnAt / 3 ? 'danger' : 'warn' : null} cx="11" cy="11" r="9.5" fill="none" stroke-width="3" pathLength="1" stroke-dasharray={(seconds / total) + " 1"} transform="rotate(-90 11 11)"></circle>
    </svg>{/if}
    <span id="Time" part="HudTimer.figure" data-tone={countdown && seconds <= warnAt ? seconds <= warnAt / 3 ? 'danger' : 'warn' : null} data-motion={paused ? 'pulse' : countdown && seconds <= warnAt / 3 ? 'beat' : null}>{formatCountdown(seconds)}</span>
  </div>
</div>

<style>
:host { text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
:host { align-items: center; padding: var(--space-sm) var(--space-lg); --fill-color: var(--figure-ink); }
[part~="HudTimer.label"] { white-space: nowrap; }
[part="HudTimer.ring"] { width: var(--size-sm); height: var(--size-sm); }
[part="HudTimer.figure"] { --figure-size: var(--font-step-2xl); white-space: nowrap; }
</style>
