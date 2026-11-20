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

<div id="ProgressBar" layout="stack" gap="tight" role="progressbar" aria-label={label || t("component.progress-bar.name")} aria-valuemin="0" aria-valuemax="100" aria-valuenow={value * 100} aria-valuetext={valueText || formatPercent(value)} aria-busy={indeterminate} data-meter={indeterminate ? "plain" : null} class:done={value >= 1 && !indeterminate} class:indeterminate={indeterminate} class:success={tone == 'success'} class:warn={tone == 'warn'} class:danger={tone == 'danger'} class:info={tone == 'info'}>
  {#if label || showValue}
    <div id="Head" layout="row">
      <span id="Label" class="label" part="label" fill>{label}</span>
      {#if showValue && !indeterminate}<span id="Value" class="value figure" part="figure">{valueText || formatPercent(value)}</span>{/if}
    </div>
  {/if}
  <div id="Track" part="track" layout="overlay" style:--value={value} style:--buffer={buffer}>
    {#if !indeterminate}<div class="buffer" id="Buffer" part="fill" anchor="top-start"></div>{/if}
    <div id="Fill" part="fill" anchor="top-start" play={indeterminate ? 'shimmer' : null}></div>
    {#if !indeterminate && value < 1}<div id="Sheen" part="sheen" anchor="top-start" play="shimmer"></div>{/if}
  </div>
</div>

<style>
/* Drawn over the scene: the style's tag halo holds its text there, and its
   words take the scene's opaque ink (a translucent ink lets the halo show
   through the glyphs). */
#ProgressBar { text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); --ink-label: var(--scene-ink); --figure-ink: var(--scene-ink); }
/* A bar fills the width its container gives it. */
#ProgressBar { inline-size: stretch; min-inline-size: 0; --tone: var(--primary); --fill-color: var(--tone); --layout-anchor-inset: 0; }
.success { --tone: var(--success); }
.warn { --tone: var(--warn); }
.danger { --tone: var(--danger); }
.info { --tone: var(--info); }
#Head { align-items: end; }
.label { white-space: nowrap; text-overflow: ellipsis; }
.figure { --figure-size: var(--font-step-sm); --figure-weight: var(--bar-figure-weight); font-stretch: var(--bar-figure-stretch); white-space: nowrap; }
/* The HUD's bar: it leans in a dark well with one lit rim, thinner than a
   vital's, and grows with the text scale. */
#Track { block-size: calc(8 * var(--font-step-px)); overflow: hidden; transform: skewX(var(--bar-skew)); border-radius: 0; background: var(--bar-well); box-shadow: var(--bar-rim); }
#Buffer { inline-size: calc(var(--buffer) * 100%); block-size: 100%; opacity: var(--opacity-faint); --fill-gloss: none; }
#Fill { inline-size: calc(var(--value) * 100%); block-size: 100%; --fill-gloss: var(--bar-gloss); }
#Sheen { block-size: 100%; }
.indeterminate #Fill { inline-size: 36%; }
</style>
