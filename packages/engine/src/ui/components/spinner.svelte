<meta name="component" content="Spinner">
<meta name="strings" content="./spinner.en.ogt">
<meta name="prop:size" content="number" data-doc="Its width and height, CSS px; 0 is 28.">
<meta name="prop:name" content="string" data-doc="What a screen reader calls it; empty is Loading.">
<meta name="fixture:default" content='{"size":28,"name":""}'>

<div id="Spinner" role="progressbar" aria-busy="true" aria-label={name || t("component.spinner.name")} style:--size={size || 28}>
  <svg id="Dots" viewBox="0 0 40 40">
    <circle cx="20" cy="4" r="4" opacity="1"></circle>
    <circle cx="31.31" cy="8.69" r="4" opacity="0.22"></circle>
    <circle cx="36" cy="20" r="4" opacity="0.33"></circle>
    <circle cx="31.31" cy="31.31" r="4" opacity="0.44"></circle>
    <circle cx="20" cy="36" r="4" opacity="0.56"></circle>
    <circle cx="8.69" cy="31.31" r="4" opacity="0.67"></circle>
    <circle cx="4" cy="20" r="4" opacity="0.78"></circle>
    <circle cx="8.69" cy="8.69" r="4" opacity="0.89"></circle>
  </svg>
</div>

<style>
#Spinner { width: calc(var(--size) * 1px); height: calc(var(--size) * 1px); }
#Dots { width: stretch; height: stretch; animation: turn 800ms steps(8) infinite; }
#Dots circle { fill: var(--ink); }
@keyframes turn { from { rotate: 0deg; } to { rotate: 360deg; } }
</style>
