<meta name="component" content="ProgressRing">
<meta name="strings" content="./progress-ring.en.ogt">
<meta name="parts" content="ProgressRing.ring ProgressRing.figure">
<meta name="prop:value" content="number" data-doc="Done, 0 to 1.">
<meta name="prop:spinning" content="bool" data-doc="Working with no known value: the arc turns (indeterminate).">
<meta name="prop:label" content="string" data-doc="Text in the middle; empty is the percent (none while spinning).">
<meta name="prop:name" content="string" data-doc="What a screen reader calls it; empty is Progress, or Loading while spinning.">
<meta name="prop:tone" content="string" data-doc="The fill's tone: primary (empty), success, warn, danger or info.">
<meta name="fixture:default" content='{"value":0.62,"spinning":false,"label":"","name":"","tone":""}'>

<div id="ProgressRing" role="progressbar" aria-label={name || (spinning ? t("component.progress-ring.loading") : t("component.progress-ring.name"))} aria-valuemin="0" aria-valuemax="100" aria-valuenow={value * 100} aria-busy={spinning} class:spinning={spinning} class:success={tone == 'success'} class:warn={tone == 'warn'} class:danger={tone == 'danger'} class:info={tone == 'info'}>
  <svg id="Ring" part="ProgressRing.ring" viewBox="0 0 48 48">
    <circle id="Track" cx="24" cy="24" r="21" fill="none" stroke-width="5"></circle>
    <circle id="Fill" cx="24" cy="24" r="21" fill="none" stroke-width="5" stroke-linecap="round" pathLength="1" stroke-dasharray={spinning ? "0.3 1" : value + " 1"} transform="rotate(-90 24 24)"></circle>
  </svg>
  {#if !spinning || label}<span id="Label" part="ProgressRing.figure">{label || formatPercent(value)}</span>{/if}
</div>

<style>
:host { text-shadow: var(--tag-halo); }
:host { display: grid; width: var(--size-lg); height: var(--size-lg); align-items: center; justify-items: center; --tone: var(--primary); }
.success { --tone: var(--success); }
.warn { --tone: var(--warn); }
.danger { --tone: var(--danger); }
.info { --tone: var(--info); }
[part="ProgressRing.ring"] { grid-area: 1 / 1; width: var(--size-lg); height: var(--size-lg); }
#Track { stroke: var(--control-strong); }
#Fill { stroke: var(--tone); }
.spinning [part="ProgressRing.ring"] { animation-name: turn; animation-duration: var(--motion-duration-turn); animation-timing-function: linear; animation-iteration-count: infinite; }
[part="ProgressRing.figure"] { grid-area: 1 / 1; font: var(--text-caption); font-family: var(--figure-font); white-space: nowrap; }
@keyframes turn { from { rotate: 0deg; } to { rotate: 360deg; } }
</style>
