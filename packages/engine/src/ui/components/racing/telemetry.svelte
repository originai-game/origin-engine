<meta name="component" content="RacingTelemetry">
<meta name="strings" content="./racing.en.ogt">
<meta name="parts" content="RacingTelemetry.frame RacingTelemetry.dial RacingTelemetry.track RacingTelemetry.fill RacingTelemetry.redline RacingTelemetry.tick RacingTelemetry.mark RacingTelemetry.face RacingTelemetry.lights RacingTelemetry.speed RacingTelemetry.units RacingTelemetry.gear RacingTelemetry.mode RacingTelemetry.badge">
<meta name="prop:speed" content="number" data-doc="The speed as the game measures it, already in its units (a whole number reads best).">
<meta name="prop:units" content="string" data-doc="The speed's unit as the game words it (km/h, mph); shown under the figure and read with it.">
<meta name="prop:gear" content="string" data-doc="The gear as the game names it: R, N, 1 to 8; on a tag in the HUD's accent; empty hides it.">
<meta name="prop:mode" content="string" data-doc="The gearbox's mode as the game words it (Auto, Manual), beside the gear; empty hides it.">
<meta name="prop:rpm" content="number" data-doc="Engine speed now: the rev arc's length.">
<meta name="prop:rpmMax" content="number" data-default="1" data-doc="The top of the rev range, the arc's end.">
<meta name="prop:redline" content="number" data-doc="The engine speed where the red zone starts, marked on the arc (lit, with data-redline on the frame, once the revs reach it); 0 is none.">
<meta name="prop:scale" content="list" data-doc="The dial's figures, keyed rows of { id, value, label }: value an engine speed on the arc, label its figure as the game words it (2 for 2,000 rpm); a figure at or past the redline is red. A tick stands by each. Empty draws none. A phone shows the ticks alone.">
<meta name="prop:lights" content="number" data-default="-1" data-doc="The share of the shift lights lit, 0 to 1, as the game decides it (seven cells, green to amber to red); negative hides the lights.">
<meta name="prop:shift" content="bool" data-doc="The game's shift cue: the Shift word stands beside the gear in place of the mode, and the frame takes data-shift.">
<meta name="fixture:default" content='{"speed":247,"units":"km/h","gear":"5","mode":"Auto","rpm":6900,"rpmMax":10500,"redline":8500,"lights":0.857,"shift":false,"scale":[{"id":"0","value":0,"label":"0"},{"id":"2","value":2000,"label":"2"},{"id":"4","value":4000,"label":"4"},{"id":"6","value":6000,"label":"6"},{"id":"8","value":8000,"label":"8"},{"id":"10","value":10000,"label":"10"}]}'>
<meta name="fixture:redline" content='{"speed":288,"units":"km/h","gear":"6","mode":"Manual","rpm":9800,"rpmMax":10500,"redline":8500,"lights":1,"shift":true,"scale":[{"id":"0","value":0,"label":"0"},{"id":"2","value":2000,"label":"2"},{"id":"4","value":4000,"label":"4"},{"id":"6","value":6000,"label":"6"},{"id":"8","value":8000,"label":"8"},{"id":"10","value":10000,"label":"10"}]}'>
<meta name="fixture:pit" content='{"speed":60,"units":"km/h","gear":"2","mode":"Limiter","rpm":3100,"rpmMax":10500,"redline":8500,"lights":0,"shift":false,"scale":[{"id":"0","value":0,"label":"0"},{"id":"2","value":2000,"label":"2"},{"id":"4","value":4000,"label":"4"},{"id":"6","value":6000,"label":"6"},{"id":"8","value":8000,"label":"8"},{"id":"10","value":10000,"label":"10"}]}'>
<meta name="fixture:reverse" content='{"speed":12,"units":"mph","gear":"R","mode":"","rpm":2300,"rpmMax":8000,"redline":7000,"lights":-1,"shift":false,"scale":[]}'>
<meta name="fixture:idle" content='{"speed":0,"units":"km/h","gear":"N","mode":"Auto","rpm":900,"rpmMax":9000,"redline":0,"lights":0,"shift":false,"scale":[{"id":"0","value":0,"label":"0"},{"id":"3","value":3000,"label":"3"},{"id":"6","value":6000,"label":"6"},{"id":"9","value":9000,"label":"9"}]}'>
<meta name="fixture:overflow" content='{"speed":1408,"units":"Kilometer pro Stunde","gear":"10","mode":"Sequenziell mit Wippen","rpm":18500,"rpmMax":18000,"redline":15000,"lights":1,"shift":true,"scale":[{"id":"0","value":0,"label":"0"},{"id":"6","value":6000,"label":"6000"},{"id":"12","value":12000,"label":"12000"},{"id":"18","value":18000,"label":"18000"}]}'>

<!--
  The tachometer at the bottom right: the revs as a white arc round a dark
  glass face with the red zone at its end, the dial's figures inside the
  arc, and on the face the shift lights, the speed in the game's units and
  the gear on a lit tag with the gearbox's mode. The game decides every
  figure, the lights and the shift cue; read only.
-->
<div id="RacingTelemetry" part="RacingTelemetry.frame" class="hud bare" role="group" aria-label={t("racing.telemetry.name")} data-shift={shift} data-redline={redline > 0 && rpm >= redline}>
  <div id="Gauge" data-layout="overlay" role="meter" aria-label={t("racing.telemetry.rpm")} aria-valuemin="0" aria-valuemax={rpmMax} aria-valuenow={rpm} aria-valuetext={t("racing.telemetry.rpmValue", { rpm: rpm })}>
    <svg part="RacingTelemetry.dial" class="dial" data-anchor="fill" viewBox="0 0 120 120">
      <circle class="well" part="RacingTelemetry.track" cx="60" cy="60" r="55" fill="none" stroke-width="8" pathLength="1" stroke-dasharray="0.75 1" transform="rotate(135 60 60)"></circle>
      <circle class="rev" part="RacingTelemetry.fill" cx="60" cy="60" r="55" fill="none" stroke-width="8" pathLength="1" stroke-dasharray={(rpmMax > 0 ? (rpm >= rpmMax ? 0.75 : rpm <= 0 ? 0 : 0.75 * rpm / rpmMax) : 0) + " 1"} transform="rotate(135 60 60)"></circle>
      {#if redline > 0 && rpmMax > 0 && redline < rpmMax}<circle class="red" part="RacingTelemetry.redline" cx="60" cy="60" r="55" fill="none" stroke-width="8" pathLength="1" stroke-dasharray={"0 " + (0.75 * redline / rpmMax) + " " + (0.75 - 0.75 * redline / rpmMax) + " 1"} transform="rotate(135 60 60)"></circle>{/if}
    </svg>
    <div class="ring" data-anchor="fill">
      {#each scale as s (s.id)}
        <div part="RacingTelemetry.tick" class="tick" data-red={redline > 0 && s.value >= redline} style:--at={rpmMax > 0 ? (s.value <= 0 ? 0 : s.value >= rpmMax ? 1 : s.value / rpmMax) : 0}></div>
        <span part="RacingTelemetry.mark" class="mark" data-red={redline > 0 && s.value >= redline} style:--at={rpmMax > 0 ? (s.value <= 0 ? 0 : s.value >= rpmMax ? 1 : s.value / rpmMax) : 0}>{s.label}</span>
      {/each}
    </div>
    <div part="RacingTelemetry.face" id="Face" data-anchor="center" data-layout="stack" role="group" aria-label={t("racing.telemetry.speed", { speed: speed, units: units })}>
      {#if lights >= 0}<div part="RacingTelemetry.lights" id="Lights" data-layout="overlay" role="meter" aria-label={t("racing.telemetry.lights")} aria-valuemin="0" aria-valuemax="100" aria-valuenow={lights >= 1 ? 100 : lights * 100} style:--lit={lights >= 1 ? 1 : lights}><div class="off" data-anchor="fill"></div><div class="on" data-anchor="fill"></div></div>{/if}
      <span part="RacingTelemetry.speed" class="speed">{formatNumber(speed)}</span>
      <span part="RacingTelemetry.units" class="units">{units}</span>
      {#if gear != '' || mode != '' || shift}<div id="Box" data-layout="row">
        {#if gear != ''}<span part="RacingTelemetry.gear" class="gear" role="image" aria-label={t("racing.telemetry.gearValue", { gear: gear })}>{gear}</span>{/if}
        {#if shift}<span part="RacingTelemetry.badge" class="shift" role="status">{t("racing.telemetry.shift")}</span>{:else if mode != ''}<span part="RacingTelemetry.mode" class="mode">{mode}</span>{/if}
      </div>{/if}
    </div>
  </div>
</div>

<style>
:host { --side: calc(230 * var(--font-step-px)); inline-size: fit-content; max-inline-size: stretch; color: var(--hud-ink); }
#Gauge { inline-size: var(--side); block-size: var(--side); max-inline-size: stretch; --layout-anchor-inset: 0; }
.dial { inline-size: 100%; block-size: 100%; filter: var(--hud-rev-glow); }
.well { stroke: var(--hud-rev-well); }
.rev { stroke: var(--hud-rev-fill); }
.red { stroke: var(--hud-redline); }
[data-redline=true] .red { stroke: var(--hud-redline-lit); }
/* The figures inside the arc and a tick by each, from the arc's start at
   the lower left round to its end at the lower right. */
.tick, .mark { position: absolute; --a: calc((var(--at) * 270 - 135) * 1deg); }
.tick { --r: calc(var(--side) * 0.405); inline-size: calc(2 * var(--font-step-px)); block-size: calc(7 * var(--font-step-px)); left: calc(50% + sin(var(--a)) * var(--r) - 1 * var(--font-step-px)); top: calc(50% - cos(var(--a)) * var(--r) - 3.5 * var(--font-step-px)); transform: rotate(var(--a)); background: var(--hud-tick); }
.tick[data-red=true] { background: var(--hud-redline-lit); }
.mark { --r: calc(var(--side) * 0.35); left: calc(50% + sin(var(--a)) * var(--r)); top: calc(50% - cos(var(--a)) * var(--r)); transform: translate(-50%, -50%); font-size: var(--hud-label-size); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
.mark[data-red=true] { color: var(--hud-redline-lit); }
/* The face: dark glass with a lit top edge, the readout on it. The face is
   the speed's size container: the speed's type step is held to a share of
   its width, so the figure stays inside the face at any text scale. */
#Face { container-type: inline-size; inline-size: calc(var(--side) * 0.6); block-size: calc(var(--side) * 0.6); align-items: center; justify-content: center; row-gap: calc(3 * var(--font-step-px)); padding: calc(6 * var(--font-step-px)); border-radius: var(--hud-radar-radius); background: var(--hud-dial-face); box-shadow: var(--hud-dial-rim); }
/* Seven leaning cells, lit from the left in a green to red ramp. */
#Lights { --cells: 7; flex: none; inline-size: calc(var(--side) * 0.4); block-size: calc(6 * var(--font-step-px)); margin-block-end: calc(2 * var(--font-step-px)); transform: skewX(var(--bar-skew)); mask-image: repeating-linear-gradient(90deg, #000 0 calc(100% / var(--cells) - 3 * var(--font-step-px)), transparent calc(100% / var(--cells) - 3 * var(--font-step-px)) calc(100% / var(--cells))); --layout-anchor-inset: 0; }
.off { background: var(--hud-pip-off); }
.on { background: var(--hud-shift-ramp); clip-path: inset(0 calc((1 - var(--lit)) * 100%) 0 0); filter: var(--hud-shift-glow); }
.speed { font-size: var(--hud-size-speed); line-height: 0.82; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.units, .mode, .shift { max-inline-size: 100%; min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.mode { color: var(--hud-ink-dim); }
.shift { color: var(--hud-redline-lit); }
#Box { align-items: center; justify-content: center; column-gap: calc(8 * var(--font-step-px)); max-inline-size: 100%; min-inline-size: 0; margin-block-start: calc(4 * var(--font-step-px)); }
.gear { flex: none; padding: 0 calc(8 * var(--font-step-px)); font-size: var(--hud-size-sm); line-height: 1.2; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; color: var(--hud-accent-ink); background: var(--hud-accent); clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
/* On a phone the dial is smaller and its figures go (the ticks stay). */
@media (width < 600px) {
  :host { --side: var(--hud-dial-compact); }
  .mark { display: none; }
  .tick { block-size: calc(5 * var(--font-step-px)); }
  #Face { row-gap: calc(1 * var(--font-step-px)); padding: calc(3 * var(--font-step-px)); }
  #Lights { block-size: calc(4 * var(--font-step-px)); }
  .speed { font-size: var(--hud-size-speed-compact); line-height: 1; }
  .units, .mode, .shift { letter-spacing: normal; line-height: 1; }
  /* The gearbox's mode goes; the gear and the Shift word stay. */
  .mode { display: none; }
  .gear { font-size: var(--hud-size-xs); }
  /* The gear stands in the arc's opening under the face, so the face
     holds only the lights, the speed and its units. */
  #Box { position: absolute; top: 100%; left: 0; right: 0; margin-block-start: calc(2 * var(--font-step-px)); column-gap: calc(5 * var(--font-step-px)); }
}
</style>
