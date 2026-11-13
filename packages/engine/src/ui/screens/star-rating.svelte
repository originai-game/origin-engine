<meta name="component" content="star-rating">
<meta name="parts" content="star-rating.frame">
<meta name="prop:stars" content="list" data-doc="A keyed collection of { id, index (0 first), lit }: one star each.">
<meta name="prop:label" content="string" data-doc="What it says to a screen reader (2 of 3 stars).">
<meta name="fixture:default" content="{&quot;label&quot;:&quot;2 of 3 stars&quot;,&quot;stars&quot;:[{&quot;id&quot;:&quot;1&quot;,&quot;index&quot;:0,&quot;lit&quot;:true},{&quot;id&quot;:&quot;2&quot;,&quot;index&quot;:1,&quot;lit&quot;:true},{&quot;id&quot;:&quot;3&quot;,&quot;index&quot;:2,&quot;lit&quot;:false}]}">

<div id="Stars" part="star-rating.frame" class="bare" role="image" aria-label={$label}>
  {#each $stars as s (s.id)}
    <og-icon class="star" class:lit={s.lit} icon="star" style:--i={s.index}/>
  {/each}
</div>

<style>
#Stars { display: flex; gap: var(--space-xs); }
.star { width: var(--size-md); height: var(--size-md); color: var(--well); }
.star.lit { color: var(--accent); animation: land var(--motion-named-ui-duration) ease-out both; animation-delay: calc(var(--i) * var(--motion-duration-instant)); }
@keyframes land { from { scale: 0.4; opacity: 0; } 70% { scale: 1.2; opacity: 1; } to { scale: 1; } }
</style>
