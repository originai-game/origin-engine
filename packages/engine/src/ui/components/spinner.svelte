<meta name="component" content="Spinner">
<meta name="strings" content="./spinner.en.ogt">
<meta name="prop:size" content="number" data-doc="Its width and height, CSS px; 0 is 28.">
<meta name="prop:name" content="string" data-doc="What a screen reader calls it; empty is Loading.">
<meta name="fixture:default" content='{"size":28,"name":""}'>

<div id="Spinner" role="progressbar" aria-busy="true" aria-label={name || t("component.spinner.name")} style:--size={size || 28}>
  <svg id="Ring" viewBox="0 0 40 40">
    <circle id="Track" cx="20" cy="20" r="16" fill="none" stroke-width="4"></circle>
    <circle id="Arc" cx="20" cy="20" r="16" fill="none" stroke-width="4" stroke-linecap="round" pathLength="1" stroke-dasharray="0.3 1" transform="rotate(-90 20 20)"></circle>
  </svg>
</div>

<style>
#Spinner { width: calc(var(--size) * 1px); height: calc(var(--size) * 1px); }
#Ring { width: stretch; height: stretch; animation: turn 900ms linear infinite; }
#Track { stroke: var(--control-strong); }
#Arc { stroke: var(--ink); }
@keyframes turn { from { rotate: 0deg; } to { rotate: 360deg; } }
</style>
