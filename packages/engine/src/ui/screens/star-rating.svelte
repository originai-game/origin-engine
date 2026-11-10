<meta name="component" content="star-rating">
<meta name="prop:stars" content="list" data-doc="A keyed collection of { id, index (0 first), lit }: one star each.">
<meta name="prop:label" content="string" data-doc="What it says to a screen reader (2 of 3 stars).">
<meta name="fixture:default" content="{&quot;label&quot;:&quot;2 of 3 stars&quot;,&quot;stars&quot;:[{&quot;id&quot;:&quot;1&quot;,&quot;index&quot;:0,&quot;lit&quot;:true},{&quot;id&quot;:&quot;2&quot;,&quot;index&quot;:1,&quot;lit&quot;:true},{&quot;id&quot;:&quot;3&quot;,&quot;index&quot;:2,&quot;lit&quot;:false}]}">

<div id="Stars" role="image" aria-label={$label}>
  {#each $stars as s (s.id)}
    <og-icon class="star" class:lit={s.lit} name="star" style:--i={s.index}/>
  {/each}
</div>

<style>
#Stars { display: flex; gap: 4px; }
.star { width: 32px; height: 32px; color: var(--well); }
.star.lit { color: var(--accent); animation: land 320ms ease-out both; animation-delay: calc(var(--i) * 120ms); }
@keyframes land { from { scale: 0.4; opacity: 0; } 70% { scale: 1.2; opacity: 1; } to { scale: 1; } }
</style>
