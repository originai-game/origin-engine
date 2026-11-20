<meta name="component" content="ProgressRing">
<meta name="strings" content="./progress-ring.en.ogt">
<meta name="prop:value" content="number" data-doc="Done, 0 to 1.">
<meta name="prop:spinning" content="bool" data-doc="Working with no known value: the arc turns (indeterminate).">
<meta name="prop:label" content="string" data-doc="Text in the middle; empty is the percent (none while spinning).">
<meta name="prop:name" content="string" data-doc="What a screen reader calls it; empty is Progress, or Loading while spinning.">
<meta name="prop:tone" content="string" data-doc="The fill's tone: primary (empty), success, warn, danger or info.">
<meta name="fixture:default" content='{"value":0.62,"spinning":false,"label":"","name":"","tone":""}'>

<div class="grid" id="ProgressRing" gap="none" role="progressbar" aria-label={name || (spinning ? t("component.progress-ring.loading") : t("component.progress-ring.name"))} aria-valuemin="0" aria-valuemax="100" aria-valuenow={value * 100} aria-busy={spinning} class:spinning={spinning} class:success={tone == 'success'} class:warn={tone == 'warn'} class:danger={tone == 'danger'} class:info={tone == 'info'}>
  <svg class="ring" id="Ring" part="ring" play={spinning ? 'spin' : null} viewBox="0 0 48 48">
    <circle id="Track" part="track" cx="24" cy="24" r="21" fill="none" stroke-width="5"></circle>
    <circle id="Fill" part="fill" tone={tone} cx="24" cy="24" r="21" fill="none" stroke-width="5" stroke-linecap="round" pathLength="1" stroke-dasharray={spinning ? "0.3 1" : value + " 1"} transform="rotate(-90 24 24)"></circle>
  </svg>
  {#if !spinning || label}<span class="figure" id="Label" part="figure">{label || formatPercent(value)}</span>{/if}
</div>

<style>
#ProgressRing { text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
#ProgressRing { display: grid; grid-template-columns: 1fr; width: var(--size-lg); height: var(--size-lg); align-items: center; justify-items: center; }
.ring { grid-area: 1 / 1; width: var(--size-lg); height: var(--size-lg); }
.figure { grid-area: 1 / 1; --figure-size: var(--font-step-sm); white-space: nowrap; }
</style>
