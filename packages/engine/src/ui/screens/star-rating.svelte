<meta name="component" content="star-rating">
<meta name="parts" content="star-rating.frame star-rating.star star-rating.glyph">
<meta name="prop:stars" content="list" data-doc="A keyed collection of { id, index (0 first), lit }: one star each; a lit star is earned and drawn filled.">
<meta name="prop:label" content="string" data-doc="What it says to a screen reader (2 of 3 stars).">
<meta name="fixture:default" content="{&quot;label&quot;:&quot;2 of 3 stars&quot;,&quot;stars&quot;:[{&quot;id&quot;:&quot;1&quot;,&quot;index&quot;:0,&quot;lit&quot;:true},{&quot;id&quot;:&quot;2&quot;,&quot;index&quot;:1,&quot;lit&quot;:true},{&quot;id&quot;:&quot;3&quot;,&quot;index&quot;:2,&quot;lit&quot;:false}]}">
<meta name="fixture:none" content="{&quot;label&quot;:&quot;0 of 3 stars&quot;,&quot;stars&quot;:[{&quot;id&quot;:&quot;1&quot;,&quot;index&quot;:0,&quot;lit&quot;:false},{&quot;id&quot;:&quot;2&quot;,&quot;index&quot;:1,&quot;lit&quot;:false},{&quot;id&quot;:&quot;3&quot;,&quot;index&quot;:2,&quot;lit&quot;:false}]}">
<meta name="fixture:all" content="{&quot;label&quot;:&quot;3 of 3 stars&quot;,&quot;stars&quot;:[{&quot;id&quot;:&quot;1&quot;,&quot;index&quot;:0,&quot;lit&quot;:true},{&quot;id&quot;:&quot;2&quot;,&quot;index&quot;:1,&quot;lit&quot;:true},{&quot;id&quot;:&quot;3&quot;,&quot;index&quot;:2,&quot;lit&quot;:true}]}">

<div id="Stars" part="star-rating.frame" class="bare" role="image" aria-label={label}>
  {#each stars as s (s.id)}
    <div class="star" part="star-rating.star" class:lit={s.lit} style:--i={s.index}>
      <og-icon class="well" icon="star"/>
      <svg class="glyph" part="star-rating.glyph" viewBox="0 0 24 24"><path d="M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z"></path></svg>
    </div>
  {/each}
</div>

<style>
#Stars { display: flex; gap: var(--space-xs); }
/* Each star is the star icon as a dim well under a glyph of the same
   shape drawn as an outline. An earned star fills the glyph (the style's
   star gold) under a lit edge, with a glow; an unearned one is a faint
   outline over the dim well. A pack recolours both through the --star-*
   roles or the star and glyph parts. */
.star { position: relative; display: block; width: var(--size-md); height: var(--size-md); }
.glyph, .well { position: absolute; inset: 0; width: 100%; height: 100%; }
.well { color: var(--star-empty-fill); }
.glyph { fill: none; stroke: var(--star-empty-edge); stroke-width: var(--star-edge-width); stroke-linejoin: round; }
.star.lit { filter: var(--star-glow); animation: land var(--motion-named-ui-duration) ease-out both; animation-delay: calc(var(--i) * var(--motion-duration-instant)); }
.star.lit .well { color: var(--star-fill-solid); }
.star.lit .glyph { fill: var(--star-fill-solid); stroke: var(--star-edge); }
@keyframes land { from { scale: 0.4; opacity: 0; } 70% { scale: 1.2; opacity: 1; } to { scale: 1; } }
</style>
