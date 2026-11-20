<meta name="component" content="PlacementGhost">
<meta name="parts" content="PlacementGhost.reach PlacementGhost.footprint PlacementGhost.cell PlacementGhost.badge">
<meta name="prop:at" content="string" data-default="0 0 0" data-doc="Its footprint's centre on the ground (x y z).">
<meta name="prop:width" content="number" data-default="1" data-doc="Its width along x before rotation, world units.">
<meta name="prop:depth" content="number" data-default="1" data-doc="Its depth along z, world units.">
<meta name="prop:rotation" content="number" data-doc="Its turn about y in degrees, clockwise from above.">
<meta name="prop:cells" content="list" data-doc="Its grid's cells in rows, keyed rows of { id, blocked }; columns cells across. Empty draws no grid.">
<meta name="prop:columns" content="number" data-default="1" data-doc="Grid cells across its width.">
<meta name="prop:valid" content="bool" data-doc="It can be placed here: green and lit; false is red. Blocked cells hatch: all of them when every cell is blocked, only the bad ones when partly.">
<meta name="prop:label" content="string" data-doc="Its name and cost (Watchtower 120), or why not (Blocked · water), on a dark badge; empty is none.">
<meta name="prop:range" content="number" data-doc="A dashed reach ring round it as a multiple of its width (a tower's reach); 0 is none.">
<meta name="fixture:default" content='{"at":"0 0 0","width":3,"depth":3,"rotation":45,"columns":3,"cells":[{"id":"0","blocked":false},{"id":"1","blocked":false},{"id":"2","blocked":false},{"id":"3","blocked":false},{"id":"4","blocked":false},{"id":"5","blocked":false},{"id":"6","blocked":false},{"id":"7","blocked":false},{"id":"8","blocked":false}],"valid":true,"label":"Watchtower · 120","range":2.8}'>
<meta name="fixture:blocked" content='{"valid":false,"label":"Blocked · water","range":0,"cells":[{"id":"0","blocked":true},{"id":"1","blocked":true},{"id":"2","blocked":true},{"id":"3","blocked":true},{"id":"4","blocked":true},{"id":"5","blocked":true},{"id":"6","blocked":true},{"id":"7","blocked":true},{"id":"8","blocked":true}]}'>
<meta name="fixture:partial" content='{"valid":false,"label":"3 cells on rock","range":0,"cells":[{"id":"0","blocked":true},{"id":"1","blocked":true},{"id":"2","blocked":false},{"id":"3","blocked":true},{"id":"4","blocked":false},{"id":"5","blocked":false},{"id":"6","blocked":false},{"id":"7","blocked":false},{"id":"8","blocked":false}]}'>

<!--
  Where a building would stand: its footprint and grid, green when it can
  be placed with a dashed reach ring and its cost; red and hatched when it
  cannot, with why (only the bad cells hatch when it is partly blocked).
  Read only; the game decides what is valid.
-->
<div id="PlacementGhost" class="bare">
<ground class="plane" at={at} size={(width * (range > 1 ? range : 1)) + " " + (depth * (range > 1 ? range : 1))} rotation={"0 " + rotation + " 0"}>
  <div id="Room" data-valid={valid} style:--k={range > 1 ? 1 / range : 1} style:--n={columns > 0 ? columns : 1}>
    {#if range > 1}<div part="PlacementGhost.reach" class="reach"></div>{/if}
    <div part="PlacementGhost.footprint" class="footprint">
      {#each cells as c (c.id)}<div part="PlacementGhost.cell" class="cell" data-blocked={c.blocked}></div>{/each}
    </div>
  </div>
</ground>
{#if label != ''}
  <!-- The badge stands upright under the footprint: it hangs from a point past the footprint's near edge, so it grows away from it. -->
  <anchor id="Badge" at={at} offset={"0 0 " + (depth * 0.75)} declutter="none" priority="5">
    <span part="PlacementGhost.badge" class="badge" data-valid={valid}>{label}</span>
  </anchor>
{/if}
</div>

<style>
.plane { inline-size: var(--measure-xs); block-size: var(--measure-xs); }
#Room { position: relative; display: flex; align-items: center; justify-content: center; inline-size: var(--measure-xs); block-size: var(--measure-xs); }
.reach { position: absolute; inset: 0; box-sizing: border-box; border-radius: var(--world-round); border: var(--world-reach-border); background: var(--world-reach-fill); }
.footprint { display: grid; grid-template-columns: repeat(var(--n), 1fr); inline-size: calc(var(--k) * 100%); block-size: calc(var(--k) * 100%); box-sizing: border-box; border: var(--world-valid-border); background: var(--world-valid-wash); box-shadow: var(--world-valid-glow); }
[data-valid=false] .footprint { border: var(--world-blocked-border); background: var(--world-blocked-faint); box-shadow: var(--world-blocked-glow); }
.cell { border: var(--world-valid-cell-border); background: var(--world-valid-cell-fill); }
[data-valid=false] .cell { border: var(--world-blocked-cell-border); background: var(--world-blocked-cell-fill); }
[data-valid=false] .cell[data-blocked=true] { background: var(--world-blocked-cell); }
/* The badge under the footprint: a dark tag, upright, a text box,
   hanging below its point. */
.badge { display: block; translate: 0 100%; padding: calc(2 * var(--font-step-px)) calc(8 * var(--font-step-px)); font-size: var(--world-size-badge); line-height: 1.2; font-weight: var(--hud-figure-weight); font-stretch: var(--world-caps-stretch); letter-spacing: var(--world-caps-tracking); text-transform: var(--hud-label-transform); white-space: nowrap; color: var(--world-friendly-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.badge[data-valid=false] { color: var(--world-hostile-ink); }
</style>
