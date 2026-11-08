<meta name="component" content="StarBar">
<meta name="strings" content="./star-bar.en.ogt">
<meta name="prop:value" content="number" data-doc="The score.">
<meta name="prop:max" content="number" data-doc="The bar's full score (its last threshold).">
<meta name="prop:stars" content="list" data-doc="The thresholds: each { id, at } with at a fraction of max (0 to 1) and lit once the score reaches it.">
<meta name="prop:label" content="string" data-doc="A caption over the score; empty is none.">
<meta name="prop:valueText" content="string" data-doc="The score in words instead of the grouped number.">
<meta name="prop:compact" content="bool" data-doc="One row: the score beside a shorter bar.">
<meta name="fixture:default" content='{"value":2140,"max":3000,"stars":[{"id":"1","at":0.3333},{"id":"2","at":0.6667},{"id":"3","at":1}],"label":"Score","valueText":"","compact":false}'>

<div id="StarBar" role="meter" aria-label={label || t("component.star-bar.name")} aria-valuemin="0" aria-valuemax={max} aria-valuenow={value} class:compact={compact}>
  <span id="Label" hidden={!label || compact}>{label}</span>
  <span id="Value">{valueText || formatNumber(value)}</span>
  <stack id="Bar">
    <div id="Track" style:--fill={value / max}><div id="Fill"></div></div>
    <div id="Stars">
      {#each stars as star (star.id)}
        <svg class="star" class:lit={value >= star.at * max} viewBox="0 0 24 24" style:--at={star.at}>
          <circle class="disc" cx="12" cy="12" r="12"></circle>
          <path class="mark" d="M12 2.5l2.9 6.1 6.6.8-4.9 4.6 1.3 6.6L12 17.3l-5.9 3.3 1.3-6.6-4.9-4.6 6.6-.8z"></path>
        </svg>
      {/each}
    </div>
  </stack>
</div>

<style>
#StarBar { align-items: center; gap: 4px; --gold: #ffc93cff; }
#StarBar.compact { display: flex; gap: 10px; }
#Label { font: var(--text-caption); color: var(--ink-muted); text-transform: uppercase; }
#Value { font: var(--text-xl3); white-space: nowrap; }
.compact #Value { font: var(--text-lg); }
#Bar { width: 220px; height: 34px; }
.compact #Bar { width: 150px; height: 24px; }
#Track { position: absolute; inset-inline-start: 0; bottom: 0; width: stretch; height: 12px; border-radius: 6px; background: #000000c7; border: 1px solid #ffffff22; overflow: hidden; }
#Fill { width: calc(var(--fill) * 100%); height: stretch; border-radius: 6px; background: linear-gradient(180deg, #ffdf8aff, #ffc93cff 50%, #d79a1eff); box-shadow: 0 0 6px #ffc93c80; }
#Stars { position: absolute; inset-inline-start: 0; top: 0; width: stretch; height: 30px; }
.star { position: absolute; top: 0; inset-inline-start: calc(var(--at) * 190px); width: 30px; height: 30px; }
.compact .star { width: 22px; height: 22px; inset-inline-start: calc(var(--at) * 128px); }
.disc { fill: #0000008c; }
.mark { fill: none; stroke: var(--ink-faint); stroke-width: 1.6; }
.star.lit .disc { fill: #00000000; }
.star.lit .mark { fill: var(--gold); stroke: #b06a00ff; }
.star.lit { animation: land 330ms ease-out; }
@keyframes land { 0% { scale: 1; } 50% { scale: 1.35; } 100% { scale: 1; } }
</style>
