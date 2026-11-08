<meta name="component" content="DamageNumbers">
<meta name="prop:hits" content="list" data-doc="The hits: each { id, entity, at, amount, color } kept for the rise's length, id unique per hit, entity the unit by its bits (0: at, a world point x y z), color #rrggbb (empty is white). Each new row spawns one number on the GPU that rises and fades.">
<meta name="fixture:default" content='{"hits":[{"id":"1","entity":0,"at":"0 2 0","amount":128,"color":""},{"id":"2","entity":0,"at":"0.5 2 0","amount":1240,"color":"#ffc93c"}]}'>

<div id="DamageNumbers">
  {#each hits as hit (hit.id)}
    <anchor class="hit" entity={hit.entity || null} at={hit.at} mass="damage" value={hit.amount} style:--tint={hit.color || '#ffffffff'}></anchor>
  {/each}
</div>

<style>
.hit { color: var(--tint); font: var(--text-xl); animation: rise 900ms ease-out; }
@keyframes rise { 0% { translate: 0 0; scale: 1.4; opacity: 1; } 20% { scale: 1; } 100% { translate: 0 -48px; opacity: 0; } }
</style>
