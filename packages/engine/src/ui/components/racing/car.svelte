<meta name="component" content="RacingCar">
<meta name="strings" content="./racing.en.ogt">
<meta name="prop:tyres" content="list" data-doc="The tyres in the game's order (front left, front right, rear left, rear right), keyed rows of { id, label, text, state }: label and text as the game words them (FL, 92); state good, worn or bad, a green, gold or red ink on the tag. Two to a line. Empty hides the tyres.">
<meta name="prop:fuel" content="number" data-default="-1" data-doc="The fuel left, 0 to 1; negative hides the fuel.">
<meta name="prop:fuelText" content="string" data-doc="The fuel as the game words it (18 L · 4 laps), after the bar; empty for none.">
<meta name="prop:fuelLow" content="bool" data-doc="The fuel is low: the bar turns to danger with a Fuel low word (data-low).">
<meta name="prop:pit" content="bool" data-doc="The team calls the car in: a Box this lap tag stands under the readout (data-pit).">
<meta name="fixture:default" content='{"tyres":[{"id":"fl","label":"FL","text":"92","state":"good"},{"id":"fr","label":"FR","text":"90","state":"good"},{"id":"rl","label":"RL","text":"71","state":"worn"},{"id":"rr","label":"RR","text":"48","state":"bad"}],"fuel":-1,"fuelText":"","fuelLow":false,"pit":false}'>
<meta name="fixture:fuel" content='{"tyres":[{"id":"fl","label":"FL","text":"88","state":"good"},{"id":"fr","label":"FR","text":"86","state":"good"},{"id":"rl","label":"RL","text":"80","state":"good"},{"id":"rr","label":"RR","text":"79","state":"good"}],"fuel":0.62,"fuelText":"38 L · 7 laps","fuelLow":false,"pit":false}'>
<meta name="fixture:fuelLow" content='{"tyres":[{"id":"fl","label":"FL","text":"64","state":"worn"},{"id":"fr","label":"FR","text":"61","state":"worn"},{"id":"rl","label":"RL","text":"33","state":"bad"},{"id":"rr","label":"RR","text":"29","state":"bad"}],"fuel":0.08,"fuelText":"4 L · 1 lap","fuelLow":true,"pit":true}'>
<meta name="fixture:empty" content='{"tyres":[],"fuel":-1,"fuelText":"","fuelLow":false,"pit":false}'>
<meta name="fixture:overflow" content='{"tyres":[{"id":"fl","label":"Front left","text":"100 % · 104 °C","state":"good"},{"id":"fr","label":"Front right","text":"100 % · 104 °C","state":"good"},{"id":"rl","label":"Rear left","text":"3 % · 141 °C","state":"bad"},{"id":"rr","label":"Rear right","text":"0 % · puncture","state":"bad"}],"fuel":3,"fuelText":"112.5 litres · 23 laps at this pace","fuelLow":true,"pit":true}'>

<!--
  The car's condition at the bottom left: the tyres as tags two to a line,
  each worded and inked by its wear, the fuel as a thin bar with the
  game's words after it (danger and Fuel low when it runs low), and Box
  this lap when the team calls the car in. Read only.
-->
<section id="RacingCar" part="frame" variant="hud" layout="stack" role="group" aria-label={t("racing.car.name")} data-pit={pit} class:low={fuelLow}>
  {#if tyres.length}<span part="title" class="title heading">{t("racing.car.tyres")}</span>
  <div class="tyres" role="list" aria-label={t("racing.car.tyres")}>
    {#each tyres as y (y.id)}<span part="tyre" class="tyre" role="listitem" class:state-bad={(y.state) == 'bad'} class:state-good={(y.state) == 'good'} class:state-worn={(y.state) == 'worn'}>{y.label} {y.text}</span>{/each}
  </div>{/if}
  {#if fuel >= 0}<div part="fuel" class="fuel" layout="row" role="meter" aria-label={t("racing.car.fuel")} aria-valuemin="0" aria-valuemax="100" aria-valuenow={fuel >= 1 ? 100 : fuel * 100} aria-valuetext={fuelText}>
    <span part="title" class="title">{fuelLow ? t("racing.car.fuelLow") : t("racing.car.fuel")}</span>
    <div part="bar" class="bar" layout="overlay" style:--value={fuel >= 1 ? 1 : fuel}><div part="fill" class="fill" anchor="top-start"></div></div>
    {#if fuelText != ''}<span part="reading" class="reading">{fuelText}</span>{/if}
  </div>{/if}
  {#if pit}<span part="badge" class="pit" role="status">{t("racing.car.pit")}</span>{/if}
</section>

<style>
#RacingCar { row-gap: calc(6 * var(--font-step-px)); inline-size: fit-content; max-inline-size: stretch; align-items: flex-start; color: var(--hud-ink); }
.title { max-inline-size: 100%; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
.tyres { display: grid; grid-template-columns: repeat(2, minmax(0, max-content)); gap: calc(5 * var(--font-step-px)); max-inline-size: 100%; }
.tyre { min-inline-size: 0; padding: 0 calc(6 * var(--font-step-px)); font-size: var(--hud-tag-size); line-height: 1.5; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; color: var(--hud-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.tyre.state-good { color: var(--hud-boon); }
.tyre.state-worn { color: var(--hud-gold); }
.tyre.state-bad { color: var(--hud-harm); }
.fuel { align-items: center; column-gap: calc(8 * var(--font-step-px)); max-inline-size: 100%; min-inline-size: 0; }
.fuel .title { flex: none; }
.low .fuel .title { color: var(--hud-harm); }
.bar { flex: none; inline-size: calc(70 * var(--font-step-px)); block-size: calc(6 * var(--font-step-px)); overflow: hidden; transform: skewX(var(--bar-skew)); background: var(--bar-well); box-shadow: var(--bar-rim); --layout-anchor-inset: 0; }
.fill { inline-size: calc(var(--value) * 100%); block-size: 100%; background: var(--hud-health-fill); }
.low .bar { box-shadow: var(--bar-critical-rim); }
.low .fill { background: var(--hud-low-fill); box-shadow: var(--hud-low-glow); }
.reading { flex: 0 1 auto; min-inline-size: 0; font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
.pit { max-inline-size: 100%; min-inline-size: 0; padding: 0 calc(8 * var(--font-step-px)); font-size: var(--hud-label-size); line-height: 1.6; font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; color: var(--hud-key-ink); background: var(--hud-gold); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
#RacingCar:size(compact) { row-gap: calc(3 * var(--font-step-px)); }
.title:size(compact), .pit:size(compact) { letter-spacing: normal; }
.heading:size(compact) { display: none; }
.tyres:size(compact) { gap: calc(3 * var(--font-step-px)); }
.bar:size(compact) { inline-size: calc(44 * var(--font-step-px)); }
</style>
