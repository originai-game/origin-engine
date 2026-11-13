<meta name="component" content="RacingTelemetry">
<meta name="strings" content="./racing.en.ogt">
<meta name="parts" content="RacingTelemetry.frame RacingTelemetry.dial RacingTelemetry.speed RacingTelemetry.units RacingTelemetry.gear RacingTelemetry.badge">
<meta name="prop:speed" content="number" data-doc="The speed as the game measures it, already in its units (a whole number reads best).">
<meta name="prop:units" content="string" data-doc="The speed's unit as the game words it (km/h, mph); shown under the figure and read with it.">
<meta name="prop:gear" content="string" data-doc="The gear as the game names it: R, N, 1 to 8; empty hides the gear plate.">
<meta name="prop:rpm" content="number" data-doc="Engine speed now.">
<meta name="prop:rpmMax" content="number" data-doc="The top of the rev range; the dial or bar shows rpm over it.">
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
    <div class="strip">
      <div part="RacingTelemetry.gear" class="gear" hidden={gear == ''} role="group" aria-label={t("racing.telemetry.gearValue", { gear: gear })}><span class="gear-value">{gear}</span></div>
      <div class="readout" role="group" aria-label={t("racing.telemetry.speed", { speed: speed, units: units })}>
        <span part="RacingTelemetry.speed" class="speed">{formatNumber(speed)}</span>
        <span part="RacingTelemetry.units" class="units">{units}</span>
      </div>
      <div class="bars">
        <div part="RacingTelemetry.dial" class="rev"><StatBar value={rpm} max={rpmMax} kind="stamina" segment={rpmMax / 8} shield={0} shieldMax={0} heal={0} recoverable={0} label="" icon="" showValue={false} valueInside={false} valueText={t("racing.telemetry.rpmValue", { rpm: rpm })} low={0} critical={0} bare={true} name={t("racing.telemetry.rpm")} /></div>
        <div class="boost" hidden={!(boostMax > 0)}><StatBar value={boost} max={boostMax} kind="mana" segment={0} shield={0} shieldMax={0} heal={0} recoverable={0} label={t("racing.telemetry.boost")} icon="" showValue={false} valueInside={false} valueText="" low={0} critical={0} bare={true} name={t("racing.telemetry.boost")} /></div>
      </div>
      <span part="RacingTelemetry.badge" class="shift" hidden={!shift} role="status">{t("racing.telemetry.shift")}</span>
    </div>
  {:else}
    <div class="gauge">
      <svg part="RacingTelemetry.dial" class="dial" viewBox="0 0 120 120" role="meter" aria-label={t("racing.telemetry.rpm")} aria-valuemin="0" aria-valuemax={rpmMax} aria-valuenow={rpm} aria-valuetext={t("racing.telemetry.rpmValue", { rpm: rpm })}>
        <circle class="track" cx="60" cy="60" r="50" fill="none" stroke-width="8" pathLength="1" stroke-dasharray="0.75 1" transform="rotate(135 60 60)"></circle>
        <circle class="red" cx="60" cy="60" r="50" fill="none" stroke-width="8" pathLength="1" opacity={redline > 0 ? 1 : 0} stroke-dasharray={"0 " + (rpmMax > 0 ? 0.75 * redline / rpmMax : 0.75) + " " + (rpmMax > 0 ? 0.75 - 0.75 * redline / rpmMax : 0) + " 1"} transform="rotate(135 60 60)"></circle>
        <circle class="rev" cx="60" cy="60" r="50" fill="none" stroke-width="8" pathLength="1" stroke-dasharray={(rpmMax > 0 ? (rpm >= rpmMax ? 0.75 : 0.75 * rpm / rpmMax) : 0) + " 1"} transform="rotate(135 60 60)"></circle>
      </svg>
      <div class="centre" role="group" aria-label={t("racing.telemetry.speed", { speed: speed, units: units })}>
        <span part="RacingTelemetry.speed" class="speed">{formatNumber(speed)}</span>
        <span part="RacingTelemetry.units" class="units">{units}</span>
      </div>
      <div part="RacingTelemetry.gear" class="gear" hidden={gear == ''} role="group" aria-label={t("racing.telemetry.gearValue", { gear: gear })}><span class="gear-value">{gear}</span></div>
      <div class="boost-ring" hidden={!(boostMax > 0)}><svg class="boost-dial" viewBox="0 0 56 56" role="meter" aria-label={t("racing.telemetry.boost")} aria-valuemin="0" aria-valuemax={boostMax} aria-valuenow={boost}><circle class="track" cx="28" cy="28" r="22" fill="none" stroke-width="7"></circle><circle class="boost" cx="28" cy="28" r="22" fill="none" stroke-width="7" pathLength="1" stroke-dasharray={(boostMax > 0 ? (boost >= boostMax ? 1 : boost / boostMax) : 0) + " 1"} transform="rotate(-90 28 28)"></circle></svg><span class="boost-label">{t("racing.telemetry.boost")}</span></div>
      <span part="RacingTelemetry.badge" class="shift" hidden={!shift} role="status">{t("racing.telemetry.shift")}</span>
    </div>
  {/if}
</div>

<style>
[part="RacingTelemetry.frame"][hidden], [part="RacingTelemetry.frame"] [hidden] { display: none; }
[part="RacingTelemetry.frame"] { position: relative; width: fit-content; max-width: stretch; }
/* Radial: the speed in the dial's centre, the gear in the gap at its foot,
   the boost ring off its lower corner and the shift badge over its top. */
.gauge { position: relative; width: calc(var(--size-xxl) * 2.2); height: calc(var(--size-xxl) * 2.2); }
.dial { position: absolute; left: 0; top: 0; width: stretch; height: stretch; }
.centre { position: absolute; left: 0; top: 0; width: stretch; height: stretch; align-items: center; justify-content: center; gap: 0; padding: 0 calc(var(--size-xxl) * 0.35); }
.speed { font-variant-numeric: tabular-nums; white-space: nowrap; line-height: 1; }
.units { white-space: nowrap; text-overflow: ellipsis; overflow: hidden; max-width: stretch; }
.gauge .gear { position: absolute; left: calc(50% - var(--size-md) * 0.75); bottom: 0; width: calc(var(--size-md) * 1.5); height: calc(var(--size-md) * 1.5); align-items: center; justify-content: center; }
.gear-value { white-space: nowrap; line-height: 1; }
/* The boost ring stands beside the dial, clear of the redline, inside the frame's box. */
[part="RacingTelemetry.frame"]:not([data-dial=bar]) { padding-inline-end: calc(var(--size-lg) + var(--space-sm)); }
.boost-ring { position: absolute; inset-inline-start: calc(100% + var(--space-sm)); bottom: 0; align-items: center; gap: 0; }
.boost-dial { width: var(--size-lg); height: var(--size-lg); }
.boost-label { white-space: nowrap; }
.gauge .shift { position: absolute; left: 50%; top: 22%; translate: -50% 0; white-space: nowrap; padding: 0 var(--space-xs); }
/* Bar: one strip, gear and speed at its start, the bars after them. */
.strip { display: flex; align-items: center; gap: var(--space-md); padding: var(--space-xs) var(--space-md); max-width: stretch; }
.strip .gear { width: var(--size-lg); height: var(--size-lg); flex: none; align-items: center; justify-content: center; }
.readout { flex: none; align-items: flex-start; gap: 0; }
.bars { gap: var(--space-xs); min-width: 0; }
.strip .shift { flex: none; white-space: nowrap; padding: 0 var(--space-xs); }
@media (width < 600px) {
  .gauge { width: calc(var(--size-xxl) * 1.6); height: calc(var(--size-xxl) * 1.6); }
  .strip { gap: var(--space-sm); padding: var(--space-xxs) var(--space-sm); }
}
</style>
