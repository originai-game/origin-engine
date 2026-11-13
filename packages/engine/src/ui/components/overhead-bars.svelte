<meta name="component" content="OverheadBars">
<meta name="parts" content="OverheadBars.bar">
<meta name="prop:bars" content="list" data-doc="The units' bars: each { id, entity, at, value, max, color }, entity the unit by its bits (0: at, a world point x y z), value its health out of max, color #rrggbb (empty is the health colour). Thousands are fine: each is a GPU mass element.">
<meta name="prop:height" content="number" data-doc="World units over each unit's point (its head).">
<meta name="fixture:default" content='{"height":2,"bars":[{"id":"a","entity":0,"at":"-1 0 0","value":72,"max":100,"color":""},{"id":"b","entity":0,"at":"1 0 0","value":30,"max":100,"color":"#e5484d"}]}'>

<div id="OverheadBars">
  {#each bars as bar (bar.id)}
    <anchor class="bar" class:tinted={!!bar.color} part="OverheadBars.bar" entity={bar.entity || null} at={bar.at} offset={"0 " + height + " 0"} mass="overhead" value={bar.value} max={bar.max} style:--tint={bar.color}></anchor>
  {/each}
</div>

<style>
.bar { color: var(--success); width: var(--size-xl); height: var(--space-xs); }
.bar.tinted { color: var(--tint); }
</style>
