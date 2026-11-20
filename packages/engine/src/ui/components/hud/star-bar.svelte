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

<!-- A score against its thresholds: the figure over a track that fills in
     the style's gold, each threshold a star set on the track where it
     falls, lit once the score reaches it. The stars are the style's rating
     stars (--star-*), the same as a level's or a results screen's. -->
<div id="StarBar" data-layout={compact ? 'row' : 'stack'} data-gap="tight" role="meter" aria-label={label || t("component.star-bar.name")} aria-valuemin="0" aria-valuemax={max} aria-valuenow={value} class:compact={compact}>
  <div id="Head" data-layout="stack" data-gap="none">
    {#if label && !compact}<span id="Label" class="label" part="StarBar.label">{label}</span>{/if}
    <span id="Value" class="value" part="StarBar.figure">{valueText || formatNumber(value)}</span>
  </div>
  <div id="Bar" class="bar">
    <div id="Track" part="StarBar.well StarBar.track" data-layout="overlay" style:--fill={value / max}><div id="Fill" part="StarBar.fill" data-anchor="top-start"></div></div>
    <div id="Stars">
      {#each stars as star (star.id)}
        <svg class="star" part="StarBar.star" data-motion={value >= star.at * max ? 'beat' : null} class:lit={value >= star.at * max} viewBox="0 0 24 24" style:--at={star.at}>
          <circle class="seat" cx="12" cy="12" r="11"></circle>
          <path class="mark" transform="translate(12 12.4) scale(0.74) translate(-12 -12)" d="M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z"></path>
        </svg>
      {/each}
    </div>
  </div>
</div>

<style>
/* Drawn over the scene: the style's tag halo holds its text there. */
:host { text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
:host { align-items: center; --bar: var(--measure-xs); --star: var(--size-md); --track: var(--space-sm); --fill-color: var(--gold); --layout-anchor-inset: 0; }
.compact { --bar: var(--size-4xl); --star: var(--size-sm); --track: var(--space-xs); }
#Head { align-items: center; }
[part="StarBar.figure"] { --figure-size: var(--font-step-3xl); white-space: nowrap; }
.compact [part="StarBar.figure"] { --figure-size: var(--font-step-xl); }
/* The track runs through the stars' middles, from the first star's place
   for a score of nothing to the last star's. */
.bar { position: relative; width: var(--bar); height: var(--star); }
#Track { position: absolute; inset-inline: calc(var(--star) / 2); top: calc((var(--star) - var(--track)) / 2); height: var(--track); overflow: hidden; }
[part="StarBar.fill"] { width: calc(var(--fill) * 100%); height: 100%; }
#Stars { position: absolute; inset-inline: 0; top: 0; height: var(--star); }
[part="StarBar.star"] { position: absolute; top: 0; inset-inline-start: calc(var(--at) * (100% - var(--star))); width: var(--star); height: var(--star); }
/* Each star sits in a seat on the track: opaque and dark in the stars' own
   hue in both schemes, so the track's fill never shows through it and a
   star reads over a bright sky as over a dark floor. */
.seat { fill: color-mix(in oklab, var(--star-fill-solid) 16%, var(--color-shadow)); stroke: var(--star-empty-edge); stroke-width: 1; }
.mark { fill: var(--star-empty-fill); stroke: var(--star-empty-edge); stroke-width: var(--star-edge-width); stroke-linejoin: round; }
.lit { filter: var(--star-glow); }
.lit .seat { stroke: var(--star-fill-solid); }
.lit .mark { fill: var(--star-fill-solid); stroke: var(--star-edge); }
</style>