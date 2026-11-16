<meta name="component" content="Spinner">
<meta name="strings" content="./spinner.en.ogt">
<meta name="parts" content="Spinner.ring">
<meta name="prop:size" content="number" data-default="28" data-doc="Its width and height, CSS px.">
<meta name="prop:name" content="string" data-doc="What a screen reader calls it; empty is Loading.">
<meta name="fixture:default" content='{"size":28,"name":""}'>

<div id="Spinner" role="progressbar" aria-busy="true" aria-label={name || t("component.spinner.name")} style:--size={size + "px"}>
  <svg id="Ring" part="Spinner.ring" data-motion="spin" viewBox="0 0 40 40">
    <circle id="Track" cx="20" cy="20" r="16" fill="none" stroke-width="4"></circle>
    <circle id="Arc" cx="20" cy="20" r="16" fill="none" stroke-width="4" stroke-linecap="round" pathLength="1" stroke-dasharray="0.3 1" transform="rotate(-90 20 20)"></circle>
  </svg>
</div>

<style>
:host { inline-size: var(--size); block-size: var(--size); flex: none; }
[part="Spinner.ring"] { inline-size: 100%; block-size: 100%; }
#Track { stroke: var(--control-strong); }
#Arc { stroke: var(--ink); }
</style>
