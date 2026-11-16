<meta name="component" content="ProgressBar">
<meta name="strings" content="./progress-bar.en.ogt">
<meta name="parts" content="ProgressBar.label ProgressBar.figure ProgressBar.track ProgressBar.buffer ProgressBar.fill ProgressBar.well">
<meta name="prop:value" content="number" data-doc="Done, 0 to 1.">
<meta name="prop:buffer" content="number" data-doc="A paler fill ahead of value (downloaded ahead of installed), 0 to 1.">
<meta name="prop:label" content="string" data-doc="A caption over it at the start; empty is none.">
<meta name="prop:valueText" content="string" data-doc="The value in words at the end; empty is the percent.">
<meta name="prop:showValue" content="bool" data-doc="Show the value at the end.">
<meta name="prop:indeterminate" content="bool" data-doc="Working with no known value: a band sweeps along the track.">
<meta name="prop:tone" content="string" data-doc="The fill's tone: primary (empty), success, warn, danger or info.">
<meta name="fixture:default" content='{"value":0.42,"buffer":0.6,"label":"Downloading","valueText":"","showValue":true,"indeterminate":false,"tone":""}'>

<div id="ProgressBar" role="progressbar" aria-label={label || t("component.progress-bar.name")} aria-valuemin="0" aria-valuemax="100" aria-valuenow={value * 100} aria-valuetext={valueText || formatPercent(value)} aria-busy={indeterminate} class:done={value >= 1 && !indeterminate} class:indeterminate={indeterminate} class:success={tone == 'success'} class:warn={tone == 'warn'} class:danger={tone == 'danger'} class:info={tone == 'info'}>
  {#if label || showValue}
    <div id="Head">
      <span id="Label" part="ProgressBar.label">{label}</span>
      {#if showValue && !indeterminate}<span id="Value" part="ProgressBar.figure">{valueText || formatPercent(value)}</span>{/if}
    </div>
  {/if}
  <div id="Track" part="ProgressBar.track ProgressBar.well" style:--value={value} style:--buffer={buffer}>
    {#if !indeterminate}<div id="Buffer" part="ProgressBar.buffer"></div>{/if}
    <div id="Fill" part="ProgressBar.fill"></div>
    {#if !indeterminate && value < 1}<div id="Sheen"></div>{/if}
  </div>
</div>

<style>
/* Drawn over the scene: the style's tag halo holds its text there. */
:host { text-shadow: var(--tag-halo); }
/* A bar fills the width its container gives it. */
:host { display: flex; flex-direction: column; gap: var(--space-xs); inline-size: stretch; min-inline-size: 0; --tone: var(--primary); }
.success { --tone: var(--success); }
.warn { --tone: var(--warn); }
.danger { --tone: var(--danger); }
.info { --tone: var(--info); }
#Head { display: flex; gap: var(--space-sm); align-items: end; }
[part="ProgressBar.label"] { font: var(--text-sm); font-family: var(--label-font); flex: 1; white-space: nowrap; text-overflow: ellipsis; }
[part="ProgressBar.figure"] { font: var(--text-sm); font-family: var(--figure-font); white-space: nowrap; }
[part="ProgressBar.track ProgressBar.well"] { position: relative; block-size: var(--space-sm); border-radius: var(--meter-radius); background: var(--control-strong); overflow: hidden; box-shadow: var(--meter-shadow); }
@media (width < 600px) { [part="ProgressBar.track ProgressBar.well"] { height: calc(var(--space-xs) + var(--space-xxs)); } }
[part="ProgressBar.buffer"] { position: absolute; top: 0; bottom: 0; inset-inline-start: 0; inline-size: calc(var(--buffer) * 100%); border-radius: var(--meter-radius); background: var(--tone); opacity: var(--opacity-faint); }
[part="ProgressBar.fill"] { position: absolute; top: 0; bottom: 0; inset-inline-start: 0; inline-size: calc(var(--value) * 100%); border-radius: var(--meter-radius); background: linear-gradient(180deg, var(--sheen), transparent 55%, var(--shadow-30)), var(--tone); box-shadow: var(--meter-fill-shadow); }
#Sheen { position: absolute; top: 0; bottom: 0; inset-inline-start: 0; inline-size: 24%; background: linear-gradient(90deg, transparent, var(--sheen), transparent); mix-blend-mode: screen; animation-name: sweep; animation-duration: var(--motion-duration-sheen); animation-timing-function: linear; animation-iteration-count: infinite; }
.done [part="ProgressBar.fill"] { box-shadow: 0 0 var(--space-sm) var(--tone); }
.indeterminate [part="ProgressBar.fill"] { inline-size: 36%; background: linear-gradient(90deg, transparent, var(--tone), transparent); box-shadow: none; animation-name: sweep; animation-duration: var(--motion-duration-sweep); animation-timing-function: var(--motion-easing-in-out); animation-iteration-count: infinite; }
@keyframes sweep { from { translate: -100% 0; } to { translate: 420% 0; } }
</style>
