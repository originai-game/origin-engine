<meta name="component" content="DamageNumbers">
<meta name="prop:hits" content="list" data-doc="The hits: each { id, entity, at, amount, color } kept for the rise's length, id unique per hit, entity the unit by its bits (0: at, a world point x y z), color #rrggbb (empty is white). Each new row spawns one number on the GPU that rises and fades.">
<meta name="fixture:default" content='{"hits":[{"id":"1","entity":0,"at":"0 2 0","amount":128,"color":""},{"id":"2","entity":0,"at":"0.5 2 0","amount":1240,"color":"#ffc93c"}]}'>

<div id="DamageNumbers">
  {#each hits as hit (hit.id)}
    <anchor part="figure" class="hit" play="float" class:tinted={!!hit.color} entity={hit.entity || null} at={hit.at} mass="damage" value={hit.amount} style:--tint={hit.color}></anchor>
  {/each}
</div>

<style>
#DamageNumbers { text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
.hit { --figure-size: var(--font-step-xl); --figure-shadow: var(--tag-halo); }
.hit.tinted { --figure-ink: var(--tint); }
</style>
