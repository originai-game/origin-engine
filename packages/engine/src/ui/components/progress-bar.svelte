<meta name="component" content="ProgressBar">
<meta name="strings" content="./progress-bar.en.ogt">
<meta name="parts" content="ProgressBar.label ProgressBar.figure ProgressBar.track ProgressBar.buffer ProgressBar.fill">
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
    <span id="Label" part="ProgressBar.label">{label}</span>
    <span id="Value" part="ProgressBar.figure" hidden={!showValue || indeterminate}>{valueText || formatPercent(value)}</span>
  </div>
  <div id="Track" part="ProgressBar.track" style:--value={value} style:--buffer={buffer}>
    <div id="Buffer" part="ProgressBar.buffer" hidden={indeterminate}></div>
    <div id="Fill" part="ProgressBar.fill"></div>
    <div id="Sheen" hidden={indeterminate || value >= 1}></div>
  </div>
</div>

<style>
#ProgressBar { width: stretch; gap: var(--space-xs); --tone: var(--primary); }
#ProgressBar.success { --tone: var(--success); }
#ProgressBar.warn { --tone: var(--warn); }
#ProgressBar.danger { --tone: var(--danger); }
#ProgressBar.info { --tone: var(--info); }
#Head { display: flex; width: stretch; gap: var(--space-sm); align-items: end; }
[part="ProgressBar.label"] { font: var(--text-sm); font-family: var(--label-font); width: stretch; white-space: nowrap; text-overflow: ellipsis; }
[part="ProgressBar.figure"] { font: var(--text-sm); font-family: var(--figure-font); white-space: nowrap; }
[part="ProgressBar.track"] { width: stretch; height: var(--space-sm); border-radius: var(--meter-radius); background: var(--control-strong); overflow: hidden; box-shadow: var(--meter-shadow); }
@media (width < 600px) { [part="ProgressBar.track"] { height: calc(var(--space-xs) + var(--space-xxs)); } }
[part="ProgressBar.buffer"] { position: absolute; inset-inline-start: 0; top: 0; width: calc(var(--buffer) * 100%); height: stretch; border-radius: var(--meter-radius); background: var(--tone); opacity: var(--opacity-faint); }
[part="ProgressBar.fill"] { position: absolute; inset-inline-start: 0; top: 0; width: calc(var(--value) * 100%); height: stretch; border-radius: var(--meter-radius); background: linear-gradient(180deg, var(--sheen), transparent 55%, var(--shadow-30)), var(--tone); box-shadow: var(--meter-fill-shadow); }
#Sheen { position: absolute; top: 0; inset-inline-start: 0; width: 24%; height: stretch; background: linear-gradient(90deg, transparent, var(--sheen), transparent); mix-blend-mode: screen; animation-name: sweep; animation-duration: calc(var(--motion-duration-slow) * 5); animation-timing-function: linear; animation-iteration-count: infinite; }
.done [part="ProgressBar.fill"] { box-shadow: 0 0 var(--space-sm) var(--tone); }
.indeterminate [part="ProgressBar.fill"] { width: 36%; background: linear-gradient(90deg, transparent, var(--tone), transparent); box-shadow: none; animation-name: sweep; animation-duration: calc(var(--motion-duration-slow) * 3.5); animation-timing-function: var(--motion-easing-in-out); animation-iteration-count: infinite; }
@keyframes sweep { from { translate: -100% 0; } to { translate: 420% 0; } }
</style>
