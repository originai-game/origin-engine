<meta name="component" content="StarBar">
<meta name="strings" content="./star-bar.en.ogt">
<meta name="parts" content="StarBar.label StarBar.figure StarBar.track StarBar.fill StarBar.star">
<meta name="prop:value" content="number" data-doc="The score.">
<meta name="prop:max" content="number" data-doc="The bar's full score (its last threshold).">
<meta name="prop:stars" content="list" data-doc="The thresholds: each { id, at } with at a fraction of max (0 to 1) and lit once the score reaches it.">
<meta name="prop:label" content="string" data-doc="A caption over the score; empty is none.">
<meta name="prop:valueText" content="string" data-doc="The score in words instead of the grouped number.">
<meta name="prop:compact" content="bool" data-doc="One row: the score beside a shorter bar.">
<meta name="fixture:default" content='{"value":2140,"max":3000,"stars":[{"id":"1","at":0.3333},{"id":"2","at":0.6667},{"id":"3","at":1}],"label":"Score","valueText":"","compact":false}'>

<div id="StarBar" role="meter" aria-label={label || t("component.star-bar.name")} aria-valuemin="0" aria-valuemax={max} aria-valuenow={value} class:compact={compact}>
  <span id="Label" part="StarBar.label" hidden={!label || compact}>{label}</span>
  <span id="Value" part="StarBar.figure">{valueText || formatNumber(value)}</span>
  <stack id="Bar">
    <div id="Track" part="StarBar.track" style:--fill={value / max}><div id="Fill" part="StarBar.fill"></div></div>
    <div id="Stars">
      {#each stars as star (star.id)}
        <svg class="star" part="StarBar.star" class:lit={value >= star.at * max} viewBox="0 0 24 24" style:--at={star.at}>
          <circle class="disc" cx="12" cy="12" r="12"></circle>
          <path class="mark" d="M12 2.5l2.9 6.1 6.6.8-4.9 4.6 1.3 6.6L12 17.3l-5.9 3.3 1.3-6.6-4.9-4.6 6.6-.8z"></path>
        </svg>
      {/each}
    </div>
  </stack>
</div>

<style>
#StarBar { align-items: center; gap: var(--space-xs); --bar: var(--measure-xs); --star: var(--size-md); }
#StarBar.compact { display: flex; gap: var(--space-sm); --bar: var(--size-4xl); --star: var(--size-sm); }
[part="StarBar.label"] { font: var(--text-caption); font-family: var(--label-font); }
[part="StarBar.figure"] { font: var(--text-xl3); font-family: var(--figure-font); white-space: nowrap; }
.compact [part="StarBar.figure"] { font: var(--text-lg); font-family: var(--figure-font); }
#Bar { width: var(--bar); height: calc(var(--star) + var(--space-xs)); }
.compact #Bar { height: var(--star); }
[part="StarBar.track"] { position: absolute; inset-inline-start: 0; bottom: 0; width: stretch; height: var(--space-md); border-radius: var(--meter-radius); background: var(--well); box-shadow: var(--meter-shadow); overflow: hidden; }
[part="StarBar.fill"] { width: calc(var(--fill) * 100%); height: stretch; border-radius: var(--meter-radius); background: linear-gradient(180deg, var(--sheen), transparent 50%, var(--shadow-30)), var(--gold); box-shadow: 0 0 var(--space-xs) var(--gold); }
#Stars { position: absolute; inset-inline-start: 0; top: 0; width: stretch; height: var(--star); }
[part="StarBar.star"] { position: absolute; top: 0; inset-inline-start: calc(var(--at) * (var(--bar) - var(--star))); width: var(--star); height: var(--star); }
.disc { fill: var(--scrim); }
.mark { fill: none; stroke: var(--ink-faint); stroke-width: 1.6; }
.lit .disc { fill: transparent; }
.lit .mark { fill: var(--gold); stroke: var(--shadow-45); }
[part="StarBar.star"].lit { animation-name: land; animation-duration: var(--motion-duration-slow); animation-timing-function: var(--motion-easing-out); }
@keyframes land { 0% { scale: 1; } 50% { scale: 1.35; } 100% { scale: 1; } }
</style>
