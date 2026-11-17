<meta name="component" content="RacingTelemetry">
<meta name="strings" content="./racing.en.ogt">
<meta name="parts" content="RacingTelemetry.frame RacingTelemetry.dial RacingTelemetry.speed RacingTelemetry.units RacingTelemetry.gear RacingTelemetry.badge RacingTelemetry.plate RacingTelemetry.figure RacingTelemetry.label RacingTelemetry.track RacingTelemetry.fill RacingTelemetry.caption">
<meta name="prop:speed" content="number" data-doc="The speed as the game measures it, already in its units (a whole number reads best).">
<meta name="prop:units" content="string" data-doc="The speed's unit as the game words it (km/h, mph); shown under the figure and read with it.">
<meta name="prop:gear" content="string" data-doc="The gear as the game names it: R, N, 1 to 8; empty hides the gear plate.">
<meta name="prop:rpm" content="number" data-doc="Engine speed now.">
<meta name="prop:rpmMax" content="number" data-default="1" data-doc="The top of the rev range; the dial or bar shows rpm over it.">
<meta name="prop:redline" content="number" data-doc="The engine speed where the red zone starts (marked on the dial); 0 is none.">
<meta name="prop:shift" content="bool" data-doc="The game's shift cue: the Shift badge shows and the frame takes data-shift.">
<meta name="prop:boost" content="number" data-doc="Boost, nitro or turbo pressure now.">
<meta name="prop:boostMax" content="number" data-doc="The most boost; 0 hides the boost meter.">
<meta name="prop:boosting" content="bool" data-doc="Boost is firing now (data-boosting on the frame).">
<meta name="prop:dial" content="string" data-doc="radial (a rev dial round the speed, boost as a ring) or bar (a compact strip: rev and boost bars beside the speed).">
<meta name="fixture:default" content='{"speed":187,"units":"km/h","gear":"4","rpm":6200,"rpmMax":9000,"redline":7800,"shift":false,"boost":62,"boostMax":100,"boosting":false,"dial":"radial"}'>
<meta name="fixture:redline" content='{"speed":243,"units":"km/h","gear":"5","rpm":8400,"rpmMax":9000,"redline":7800,"shift":true,"boost":100,"boostMax":100,"boosting":true,"dial":"radial"}'>
<meta name="fixture:bar" content='{"speed":96,"units":"mph","gear":"3","rpm":4100,"rpmMax":8000,"redline":7000,"shift":false,"boost":35,"boostMax":100,"boosting":false,"dial":"bar"}'>
<meta name="fixture:reverse" content='{"speed":12,"units":"km/h","gear":"R","rpm":2300,"rpmMax":9000,"redline":7800,"shift":false,"boost":0,"boostMax":0,"boosting":false,"dial":"radial"}'>
<meta name="fixture:idle" content='{"speed":0,"units":"km/h","gear":"N","rpm":900,"rpmMax":9000,"redline":0,"shift":false,"boost":0,"boostMax":100,"boosting":false,"dial":"bar"}'>
<meta name="fixture:overflow" content='{"speed":1408,"units":"Kilometer pro Stunde","gear":"10","rpm":18500,"rpmMax":18000,"redline":15000,"shift":true,"boost":100,"boostMax":100,"boosting":true,"dial":"radial"}'>

<!--
  The car's instruments: the speed in the game's units, the gear, and the
  engine speed as a rev dial (radial) or a strip (bar), with a boost meter.
  The game decides every figure and the shift cue; read only.
-->
<div id="RacingTelemetry" part="RacingTelemetry.frame" class="hud" role="group" aria-label={t("racing.telemetry.name")} data-dial={dial} data-shift={shift} data-boosting={boosting} data-redline={redline > 0 && rpm >= redline}>
  {#if dial == 'bar'}
    <div class="strip" data-layout="row" data-gap="loose">
      {#if gear != ''}<div part="RacingTelemetry.gear RacingTelemetry.plate RacingTelemetry.frame" class="gear hud" data-layout="cover" data-gap="none" role="group" aria-label={t("racing.telemetry.gearValue", { gear: gear })}><span class="gear-value" part="RacingTelemetry.figure">{gear}</span></div>{/if}
      <div class="readout" data-layout="stack" data-gap="hair" role="group" aria-label={t("racing.telemetry.speed", { speed: speed, units: units })}>
        <span part="RacingTelemetry.speed RacingTelemetry.figure" class="speed">{formatNumber(speed)}</span>
        <span part="RacingTelemetry.units RacingTelemetry.label" class="units">{units}</span>
      </div>
      <div class="bars" data-layout="stack" data-gap="tight" data-fill>
        <div part="RacingTelemetry.dial" class="rev"><StatBar value={rpm} max={rpmMax} kind="stamina" segment={rpmMax / 8} shield={0} shieldMax={0} heal={0} recoverable={0} label="" icon="" showValue={false} valueInside={false} valueText={t("racing.telemetry.rpmValue", { rpm: rpm })} low={0} critical={0} bare={true} name={t("racing.telemetry.rpm")} /></div>
        {#if boostMax > 0}<div class="boost"><StatBar value={boost} max={boostMax} kind="mana" segment={0} shield={0} shieldMax={0} heal={0} recoverable={0} label={t("racing.telemetry.boost")} icon="" showValue={false} valueInside={false} valueText="" low={0} critical={0} bare={true} name={t("racing.telemetry.boost")} /></div>{/if}
      </div>
      {#if shift}<span part="RacingTelemetry.badge" class="shift" data-tone="warn" role="status">{t("racing.telemetry.shift")}</span>{/if}
    </div>
  {:else}
    <div class="gauge" data-layout="overlay">
      <svg part="RacingTelemetry.dial" class="dial" data-anchor="fill" viewBox="0 0 120 120" role="meter" aria-label={t("racing.telemetry.rpm")} aria-valuemin="0" aria-valuemax={rpmMax} aria-valuenow={rpm} aria-valuetext={t("racing.telemetry.rpmValue", { rpm: rpm })}>
        <circle class="track" part="RacingTelemetry.track" cx="60" cy="60" r="50" fill="none" stroke-width="8" pathLength="1" stroke-dasharray="0.75 1" transform="rotate(135 60 60)"></circle>
        <circle class="red" part="RacingTelemetry.fill" data-tone="danger" cx="60" cy="60" r="50" fill="none" stroke-width="8" pathLength="1" opacity={redline > 0 ? 1 : 0} stroke-dasharray={"0 " + (rpmMax > 0 ? 0.75 * redline / rpmMax : 0.75) + " " + (rpmMax > 0 ? 0.75 - 0.75 * redline / rpmMax : 0) + " 1"} transform="rotate(135 60 60)"></circle>
        <circle class="rev" part="RacingTelemetry.fill" cx="60" cy="60" r="50" fill="none" stroke-width="8" pathLength="1" stroke-dasharray={(rpmMax > 0 ? (rpm >= rpmMax ? 0.75 : 0.75 * rpm / rpmMax) : 0) + " 1"} transform="rotate(135 60 60)"></circle>
      </svg>
      <div class="centre" data-layout="stack" data-gap="hair" data-anchor="center" role="group" aria-label={t("racing.telemetry.speed", { speed: speed, units: units })}>
        <span part="RacingTelemetry.speed RacingTelemetry.figure" class="speed">{formatNumber(speed)}</span>
        <span part="RacingTelemetry.units RacingTelemetry.label" class="units">{units}</span>
      </div>
      {#if gear != ''}<div class="gear-anchor" data-anchor="bottom"><div part="RacingTelemetry.gear RacingTelemetry.plate RacingTelemetry.frame" class="gear hud" data-layout="cover" data-gap="none" role="group" aria-label={t("racing.telemetry.gearValue", { gear: gear })}><span class="gear-value" part="RacingTelemetry.figure">{gear}</span></div></div>{/if}
      {#if boostMax > 0}<div class="boost-ring" data-layout="stack" data-gap="none" data-anchor="bottom-end"><svg class="boost-dial" viewBox="0 0 56 56" role="meter" aria-label={t("racing.telemetry.boost")} aria-valuemin="0" aria-valuemax={boostMax} aria-valuenow={boost}><circle class="track" part="RacingTelemetry.track" cx="28" cy="28" r="22" fill="none" stroke-width="7"></circle><circle class="boost" part="RacingTelemetry.fill" cx="28" cy="28" r="22" fill="none" stroke-width="7" pathLength="1" stroke-dasharray={(boostMax > 0 ? (boost >= boostMax ? 1 : boost / boostMax) : 0) + " 1"} transform="rotate(-90 28 28)"></circle></svg><span class="boost-label" part="RacingTelemetry.caption">{t("racing.telemetry.boost")}</span></div>{/if}
      {#if shift}<div class="shift-anchor" data-anchor="top"><span part="RacingTelemetry.badge" class="shift" data-tone="warn" role="status">{t("racing.telemetry.shift")}</span></div>{/if}
    </div>
  {/if}
</div>

<style>
:host { position: relative; width: fit-content; max-width: 100%; text-shadow: var(--tag-halo); }
/* Radial: the speed in the dial's centre, the gear in the gap at its foot,
   the boost ring off its lower corner and the shift badge over its top. */
.gauge { width: calc(var(--size-xxl) * 2.2); height: calc(var(--size-xxl) * 2.2); --layout-anchor-inset: 0; }
.dial { width: 100%; height: 100%; }
.centre { padding: 0 calc(var(--size-xxl) * 0.35); }
.speed { --figure-size: var(--font-step-2xl); white-space: nowrap; }
.units { white-space: nowrap; text-overflow: ellipsis; overflow: hidden; max-width: stretch; }
.boost { --fill-color: var(--mana); }
.gauge .gear { width: calc(var(--size-md) * 1.5); height: calc(var(--size-md) * 1.5); padding: 0; }
.gear-value { --figure-size: var(--font-step-xl); white-space: nowrap; }
/* The boost ring stands beside the dial, clear of the redline, inside the frame's box. */
[part="RacingTelemetry.frame"]:not([data-dial=bar]) { padding-inline-end: calc(var(--size-lg) + var(--space-sm)); }
.boost-ring { inset-inline-start: calc(100% + var(--space-sm)); inset-inline-end: auto; align-items: center; }
.boost-dial { width: var(--size-lg); height: var(--size-lg); }
.boost-label { white-space: nowrap; }
.shift-anchor { top: 22%; }
.shift { white-space: nowrap; padding: 0 var(--space-xs); }
/* Bar: one strip, gear and speed at its start, the bars after them. */
.strip { padding: var(--space-xs) var(--space-md); max-width: stretch; }
.strip .gear { width: var(--size-lg); height: var(--size-lg); flex: none; padding: 0; }
.readout { flex: none; align-items: flex-start; }
.strip .shift { flex: none; white-space: nowrap; padding: 0 var(--space-xs); }
@media (width < 600px) {
  .gauge { width: calc(var(--size-xxl) * 1.6); height: calc(var(--size-xxl) * 1.6); }
  .strip { --layout-gap-loose: var(--layout-gap); padding: var(--space-xxs) var(--space-sm); }
}
</style>
