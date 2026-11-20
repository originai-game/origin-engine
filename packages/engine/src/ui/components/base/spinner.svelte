<meta name="component" content="Spinner">
<meta name="strings" content="./spinner.en.ogt">
<meta name="prop:size" content="number" data-default="28" data-doc="Its width and height, CSS px.">
<meta name="prop:name" content="string" data-doc="What a screen reader calls it; empty is Loading.">
<meta name="fixture:default" content='{"size":28,"name":""}'>

<!-- A ring that is always there and an arc that runs round it. `size` is
     its width; a host that holds nothing else while it waits (a portrait,
     a slot) sets `--spinner-size` on the box round it instead, so the ring
     is in proportion to it. -->
<div id="Spinner" role="progressbar" aria-busy="true" aria-label={name || t("component.spinner.name")} style:--size={size + "px"}>
  <svg class="ring" id="Ring" part="ring" play="spin" viewBox="0 0 40 40">
    <circle id="Track" cx="20" cy="20" r="17" fill="none" stroke-width="4.5"></circle>
    <circle id="Arc" cx="20" cy="20" r="17" fill="none" stroke-width="4.5" stroke-linecap="round" pathLength="1" stroke-dasharray="0.28 1" transform="rotate(-90 20 20)"></circle>
  </svg>
</div>

<style>
#Spinner { inline-size: var(--spinner-size, var(--size)); block-size: var(--spinner-size, var(--size)); flex: none; }
.ring { inline-size: 100%; block-size: 100%; }
#Track { stroke: var(--spinner-track); }
#Arc { stroke: var(--spinner-ink); }
</style>
