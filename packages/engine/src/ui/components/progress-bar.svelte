<meta name="component" content="ProgressBar">
<meta name="strings" content="./progress-bar.en.ogt">
<meta name="prop:value" content="number" data-doc="Done, 0 to 1.">
<meta name="prop:buffer" content="number" data-doc="A paler fill ahead of value (downloaded ahead of installed), 0 to 1.">
<meta name="prop:label" content="string" data-doc="A caption over it at the start; empty is none.">
<meta name="prop:valueText" content="string" data-doc="The value in words at the end; empty is the percent.">
<meta name="prop:showValue" content="bool" data-doc="Show the value at the end.">
<meta name="prop:indeterminate" content="bool" data-doc="Working with no known value: a band sweeps along the track.">
<meta name="prop:tone" content="string" data-doc="The fill's tone: primary (empty), success, warn, danger or info.">
<meta name="fixture:default" content='{"value":0.42,"buffer":0.6,"label":"Downloading","valueText":"","showValue":true,"indeterminate":false,"tone":""}'>

<div id="ProgressBar" role="progressbar" aria-label={label || t("component.progress-bar.name")} aria-valuemin="0" aria-valuemax="100" aria-valuenow={value * 100} aria-valuetext={valueText || formatPercent(value)} aria-busy={indeterminate} class:done={value >= 1 && !indeterminate} class:indeterminate={indeterminate} class:success={tone == 'success'} class:warn={tone == 'warn'} class:danger={tone == 'danger'} class:info={tone == 'info'}>
  <div id="Head" hidden={!(label || showValue)}>
    <span id="Label">{label}</span>
    <span id="Value" hidden={!showValue || indeterminate}>{valueText || formatPercent(value)}</span>
  </div>
  <div id="Track" style:--value={value} style:--buffer={buffer}>
    <div id="Buffer" hidden={indeterminate}></div>
    <div id="Fill"></div>
    <div id="Sheen" hidden={indeterminate || value >= 1}></div>
  </div>
</div>

<style>
#ProgressBar { width: stretch; gap: 4px; --tone: var(--primary); }
#ProgressBar.success { --tone: var(--success); }
#ProgressBar.warn { --tone: var(--warn); }
#ProgressBar.danger { --tone: var(--danger); }
#ProgressBar.info { --tone: var(--info); }
#Head { display: flex; width: stretch; gap: 8px; align-items: end; }
#Label { font: var(--text-sm); color: var(--ink); width: stretch; white-space: nowrap; text-overflow: ellipsis; }
#Value { font: var(--text-sm); font-family: var(--font-mono); white-space: nowrap; }
#Track { width: stretch; height: 8px; border-radius: 4px; background: var(--control-strong); overflow: hidden; box-shadow: inset 0 1px 2px #00000066; }
@media (width < 600px) { #Track { height: 6px; border-radius: 3px; } }
#Buffer { position: absolute; inset-inline-start: 0; top: 0; width: calc(var(--buffer) * 100%); height: stretch; border-radius: 4px; background: var(--tone); opacity: 0.3; }
#Fill { position: absolute; inset-inline-start: 0; top: 0; width: calc(var(--value) * 100%); height: stretch; border-radius: 4px; background: linear-gradient(180deg, #ffffff2e, #00000000 55%, #00000038), var(--tone); border: 1px solid #ffffff33; }
#Sheen { position: absolute; top: 0; inset-inline-start: 0; width: 24%; height: stretch; background: linear-gradient(90deg, #ffffff00, #ffffff55, #ffffff00); mix-blend-mode: screen; animation: sweep 2200ms linear infinite; }
.done #Fill { box-shadow: 0 0 10px var(--tone); }
.indeterminate #Fill { width: 36%; background: linear-gradient(90deg, #00000000, var(--tone), #00000000); animation: sweep 1400ms ease-in-out infinite; }
@keyframes sweep { from { translate: -240px 0; } to { translate: 960px 0; } }
</style>
