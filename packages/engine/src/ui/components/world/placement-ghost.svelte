<meta name="component" content="PlacementGhost">
<meta name="parts" content="PlacementGhost.reach PlacementGhost.footprint PlacementGhost.badge PlacementGhost.zone PlacementGhost.gridline PlacementGhost.fill">
<meta name="prop:at" content="string" data-default="0 0 0" data-doc="Its footprint's centre on the ground (x y z).">
<meta name="prop:width" content="number" data-default="1" data-doc="Its width along x before rotation, world units.">
<meta name="prop:depth" content="number" data-default="1" data-doc="Its depth along z, world units.">
<meta name="prop:rotation" content="number" data-doc="Its turn about y in degrees, clockwise from above.">
<meta name="prop:cells" content="number" data-doc="Grid cells across its width, drawn on the footprint (0: none).">
<meta name="prop:valid" content="bool" data-doc="It can be placed here (success); false is danger with hatching.">
<meta name="prop:label" content="string" data-doc="Its cost or why not (Blocked); empty is none.">
<meta name="prop:range" content="number" data-doc="A range ring round it as a multiple of its width (a tower's reach); 0 is none.">
<meta name="fixture:default" content='{"at":"0 0 0","width":3,"depth":3,"rotation":0,"cells":3,"valid":false,"label":"Blocked","range":0}'>

<ground id="PlacementGhost" at={at} size={(width * (range > 1 ? range : 1)) + " " + (depth * (range > 1 ? range : 1))} rotation={"0 " + rotation + " 0"}>
  <div id="Room" data-layout="overlay" class:invalid={!valid}>
    {#if range > 1}<svg id="Reach" part="PlacementGhost.reach" data-anchor="fill" viewBox="0 0 200 200"><circle part="PlacementGhost.fill" data-tone={valid ? 'success' : 'danger'} cx="100" cy="100" r="98" fill="none" stroke-width="2" stroke-dasharray="8 6"></circle></svg>{/if}
    <div id="Footprint" part="PlacementGhost.footprint PlacementGhost.zone" data-tone={valid ? 'success' : 'danger'} data-layout="overlay" data-anchor="center" style:--cells={cells || 1} style:--k={range > 1 ? 1 / range : 1}>
      {#if cells > 0}<div id="Grid" part="PlacementGhost.gridline" data-anchor="fill"></div>{/if}
      {#if label}<span id="Label" class="label" part="PlacementGhost.badge" data-anchor="center">{label}</span>{/if}
    </div>
  </div>
</ground>

<style>
:host { width: var(--measure-xs); height: var(--measure-xs); text-shadow: var(--tag-halo); }
#Room { width: var(--measure-xs); height: var(--measure-xs); --layout-anchor-inset: 0; }
[part="PlacementGhost.reach"] { width: var(--measure-xs); height: var(--measure-xs); }
[part~="PlacementGhost.footprint"] { width: calc(var(--k) * var(--measure-xs)); height: calc(var(--k) * var(--measure-xs)); }
#Grid { --cell: calc(100% / var(--cells)); mask-image: repeating-linear-gradient(0deg, #000 0 var(--stroke-hairline), transparent 0 var(--cell)), repeating-linear-gradient(90deg, #000 0 var(--stroke-hairline), transparent 0 var(--cell)); }
/* In the world, over any ground: the badge takes the scene's plate. */
[part="PlacementGhost.badge"] { white-space: nowrap; --badge-fill: var(--scene-plate); --badge-ink: var(--zone-color); }
</style>
