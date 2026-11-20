<meta name="component" content="OverheadBars">
<meta name="prop:bars" content="list" data-doc="The units' bars, keyed rows of { id, entity, at, value, max, side, resource, resourceMax }: entity the unit by its bits (0: at, a world point x y z); value its health out of max; side whose it is, own (green), ally (blue), enemy (red) or neutral (yellow); resource out of resourceMax a thin line under the health (mana, energy), resourceMax 0 draws none. Thousands are fine: each bar is a GPU mass element of a fixed size (never under 30 × 3 px) that keeps the ghost of a hit.">
<meta name="prop:height" content="number" data-doc="World units over each unit's point (its head).">
<meta name="fixture:default" content='{"height":2,"bars":[{"id":"a","entity":0,"at":"-1 0 0","value":90,"max":100,"side":"own","resource":70,"resourceMax":100},{"id":"b","entity":0,"at":"1 0 0","value":24,"max":100,"side":"ally","resource":35,"resourceMax":100}]}'>
<meta name="fixture:sides" content='{"height":2,"bars":[{"id":"o","entity":0,"at":"-3 0 0","value":100,"max":100,"side":"own","resource":0,"resourceMax":0},{"id":"a","entity":0,"at":"-1 0 0","value":70,"max":100,"side":"ally","resource":0,"resourceMax":0},{"id":"e","entity":0,"at":"1 0 0","value":45,"max":100,"side":"enemy","resource":0,"resourceMax":0},{"id":"n","entity":0,"at":"3 0 0","value":88,"max":100,"side":"neutral","resource":0,"resourceMax":0}]}'>
<meta name="fixture:low" content='{"height":2,"bars":[{"id":"l","entity":0,"at":"0 0 0","value":12,"max":100,"side":"enemy","resource":0,"resourceMax":0}]}'>
<meta name="fixture:empty" content='{"height":2,"bars":[]}'>

<!--
  A health bar over every unit in the side's colour, a thin resource line
  under it: one GPU mass element per bar, so a battle's thousands cost
  nothing to place. Read only.
-->
<div id="OverheadBars" part="frame" variant="bare" role="group" aria-label={t("world.bars.name")}>
  {#each bars as bar (bar.id)}
    <anchor class="bar" part="bar" entity={bar.entity || null} at={bar.at} offset={"0 " + height + " 0"} mass="overhead" value={bar.value} max={bar.max} class:side-ally={(bar.side) == 'ally'} class:side-enemy={(bar.side) == 'enemy'} class:side-neutral={(bar.side) == 'neutral'}></anchor>
    {#if bar.resourceMax > 0}<anchor class="resource" part="resource" entity={bar.entity || null} at={bar.at} offset={"0 " + (height - 0.12) + " 0"} mass="overheadResource" value={bar.resource} max={bar.resourceMax}></anchor>{/if}
  {/each}
</div>

<style>
/* A bar's size is fixed in pixels, its colour the side's. */
.bar { width: var(--world-overhead-width); height: var(--world-overhead-height); color: var(--world-friendly-ink); }
.bar.side-ally { color: var(--world-ally-ink); }
.bar.side-enemy { color: var(--hud-foe); }
.bar.side-neutral { color: var(--world-neutral-ink); }
.resource { width: var(--world-resource-width); height: var(--world-resource-height); color: var(--hud-mana-ink); }
</style>
