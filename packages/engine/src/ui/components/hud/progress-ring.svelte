<meta name="component" content="ProgressRing">
<meta name="strings" content="./progress-ring.en.ogt">
<meta name="parts" content="ProgressRing.ring ProgressRing.figure ProgressRing.track ProgressRing.fill">
<meta name="prop:value" content="number" data-doc="Done, 0 to 1.">
<meta name="prop:spinning" content="bool" data-doc="Working with no known value: the arc turns (indeterminate).">
<meta name="prop:label" content="string" data-doc="Text in the middle; empty is the percent (none while spinning).">
<meta name="prop:name" content="string" data-doc="What a screen reader calls it; empty is Progress, or Loading while spinning.">
<meta name="prop:tone" content="string" data-doc="The fill's tone: primary (empty), success, warn, danger or info.">
<meta name="fixture:default" content='{"value":0.62,"spinning":false,"label":"","name":"","tone":""}'>

<div id="ProgressRing" data-layout="grid" data-gap="none" role="progressbar" aria-label={name || (spinning ? t("component.progress-ring.loading") : t("component.progress-ring.name"))} aria-valuemin="0" aria-valuemax="100" aria-valuenow={value * 100} aria-busy={spinning} class:spinning={spinning} class:success={tone == 'success'} class:warn={tone == 'warn'} class:danger={tone == 'danger'} class:info={tone == 'info'}>
  <svg id="Ring" part="ProgressRing.ring" data-motion={spinning ? 'spin' : null} viewBox="0 0 48 48">
    <circle id="Track" part="ProgressRing.track" cx="24" cy="24" r="21" fill="none" stroke-width="5"></circle>
    <circle id="Fill" part="ProgressRing.fill" data-tone={tone} cx="24" cy="24" r="21" fill="none" stroke-width="5" stroke-linecap="round" pathLength="1" stroke-dasharray={spinning ? "0.3 1" : value + " 1"} transform="rotate(-90 24 24)"></circle>
  </svg>
  {#if !spinning || label}<span id="Label" part="ProgressRing.figure">{label || formatPercent(value)}</span>{/if}
</div>

<style>
:host { text-shadow: var(--tag-halo); }
:host { grid-template-columns: 1fr; width: var(--size-lg); height: var(--size-lg); align-items: center; justify-items: center; }
[part="ProgressRing.ring"] { grid-area: 1 / 1; width: var(--size-lg); height: var(--size-lg); }
[part="ProgressRing.figure"] { grid-area: 1 / 1; --figure-size: var(--font-step-sm); white-space: nowrap; }
</style>
