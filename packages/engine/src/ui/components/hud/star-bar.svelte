<meta name="component" content="StarBar">
<meta name="strings" content="./star-bar.en.ogt">
<meta name="parts" content="StarBar.label StarBar.figure StarBar.track StarBar.fill StarBar.star StarBar.well">
<meta name="prop:value" content="number" data-doc="The score.">
<meta name="prop:max" content="number" data-default="1" data-doc="The bar's full score (its last threshold).">
<meta name="prop:stars" content="list" data-doc="The thresholds: each { id, at } with at a fraction of max (0 to 1) and lit once the score reaches it.">
<meta name="prop:label" content="string" data-doc="A caption over the score; empty is none.">
<meta name="prop:valueText" content="string" data-doc="The score in words instead of the grouped number.">
<meta name="prop:compact" content="bool" data-doc="One row: the score beside a shorter bar.">
<meta name="fixture:default" content='{"value":2140,"max":3000,"stars":[{"id":"1","at":0.3333},{"id":"2","at":0.6667},{"id":"3","at":1}],"label":"Score","valueText":"","compact":false}'>

<div id="StarBar" data-layout={compact ? 'row' : 'stack'} data-gap="tight" role="meter" aria-label={label || t("component.star-bar.name")} aria-valuemin="0" aria-valuemax={max} aria-valuenow={value} class:compact={compact}>
  {#if label && !compact}<span id="Label" class="label" part="StarBar.label">{label}</span>{/if}
  <span id="Value" class="value" part="StarBar.figure">{valueText || formatNumber(value)}</span>
  <div id="Bar" class="bar">
    <div id="Track" part="StarBar.well StarBar.track" data-layout="overlay" style:--fill={value / max}><div id="Fill" part="StarBar.fill" data-anchor="top-start"></div></div>
    <div id="Stars">
      {#each stars as star (star.id)}
        <svg class="star" part="StarBar.star" data-motion={value >= star.at * max ? 'beat' : null} class:lit={value >= star.at * max} viewBox="0 0 24 24" style:--at={star.at}>
          <circle class="disc" cx="12" cy="12" r="12"></circle>
          <path class="mark" d="M12 2.5l2.9 6.1 6.6.8-4.9 4.6 1.3 6.6L12 17.3l-5.9 3.3 1.3-6.6-4.9-4.6 6.6-.8z"></path>
        </svg>
      {/each}
    </div>
  </div>
</div>

<style>
/* Drawn over the scene: the style's tag halo holds its text there. */
:host { text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
:host { align-items: center; --bar: var(--measure-xs); --star: var(--size-md); --fill-color: var(--gold); --layout-anchor-inset: 0; }
.compact { --bar: var(--size-4xl); --star: var(--size-sm); }
[part="StarBar.figure"] { --figure-size: var(--font-step-2xl); white-space: nowrap; }
.compact [part="StarBar.figure"] { --figure-size: var(--font-step-lg); }
.bar { position: relative; width: var(--bar); height: calc(var(--star) + var(--space-xs) + var(--space-md)); }
#Track { position: absolute; inset-inline: calc(var(--star) / 2); bottom: 0; height: var(--space-md); overflow: hidden; }
[part="StarBar.fill"] { width: calc(var(--fill) * 100%); height: 100%; }
#Stars { position: absolute; inset-inline: 0; top: 0; height: var(--star); }
[part="StarBar.star"] { position: absolute; top: 0; inset-inline-start: calc(var(--at) * (100% - var(--star))); width: var(--star); height: var(--star); }
.disc { fill: var(--track-fill); --track-fill: var(--scrim); }
.mark { fill: none; stroke: var(--dot-rim); stroke-width: var(--dot-rim-width); --dot-rim: var(--ink-faint); --dot-rim-width: var(--stroke-regular); }
.lit .disc { fill: transparent; }
.lit .mark { fill: var(--fill-color); stroke: var(--dot-rim); --dot-rim: transparent; }
</style>
