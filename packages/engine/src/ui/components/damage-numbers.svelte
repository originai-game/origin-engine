<meta name="component" content="DamageNumbers">
<meta name="parts" content="DamageNumbers.figure">
<meta name="prop:hits" content="list" data-doc="The hits: each { id, entity, at, amount, color } kept for the rise's length, id unique per hit, entity the unit by its bits (0: at, a world point x y z), color #rrggbb (empty is white). Each new row spawns one number on the GPU that rises and fades.">
<meta name="fixture:default" content='{"hits":[{"id":"1","entity":0,"at":"0 2 0","amount":128,"color":""},{"id":"2","entity":0,"at":"0.5 2 0","amount":1240,"color":"#ffc93c"}]}'>

<div id="DamageNumbers">
  {#each hits as hit (hit.id)}
    <anchor part="DamageNumbers.figure" class="hit" class:tinted={!!hit.color} entity={hit.entity || null} at={hit.at} mass="damage" value={hit.amount} style:--tint={hit.color}></anchor>
  {/each}
</div>

<style>
:host { text-shadow: var(--tag-halo); }
.hit { color: var(--scene-ink); font: var(--text-xl); font-family: var(--figure-font); font-variant-numeric: tabular-nums; animation-name: rise; animation-duration: var(--motion-duration-turn); animation-timing-function: var(--motion-easing-out); }
.hit.tinted { color: var(--tint); }
@keyframes rise { 0% { translate: 0 0; scale: 1.4; opacity: 1; } 20% { scale: 1; } 100% { translate: 0 calc(var(--space-3xl) * -1); opacity: 0; } }
</style>
