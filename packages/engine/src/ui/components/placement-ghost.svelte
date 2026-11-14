<meta name="component" content="PlacementGhost">
<meta name="parts" content="PlacementGhost.reach PlacementGhost.footprint PlacementGhost.badge">
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
  <div id="Room" class:invalid={!valid}>
    <svg id="Reach" part="PlacementGhost.reach" viewBox="0 0 200 200" hidden={!(range > 1)}><circle cx="100" cy="100" r="98" fill="none" stroke-width="2" stroke-dasharray="8 6"></circle></svg>
    <div id="Footprint" part="PlacementGhost.footprint" style:--cells={cells || 1} style:--k={range > 1 ? 1 / range : 1}>
      <div id="Grid" hidden={!(cells > 0)}></div>
      <span id="Label" part="PlacementGhost.badge" hidden={!label}>{label}</span>
    </div>
  </div>
</ground>

<style>
#PlacementGhost { width: var(--measure-xs); height: var(--measure-xs); }
#Room { width: var(--measure-xs); height: var(--measure-xs); align-items: center; justify-content: center; --tone: var(--success); --wash: color-mix(in srgb, var(--success) 20%, transparent); }
#Room.invalid { --tone: var(--danger); --wash: color-mix(in srgb, var(--danger) 25%, transparent); }
[part="PlacementGhost.reach"] { position: absolute; top: 0; inset-inline-start: 0; width: var(--measure-xs); height: var(--measure-xs); stroke: var(--tone); }
[part="PlacementGhost.footprint"] { width: calc(var(--k) * var(--measure-xs)); height: calc(var(--k) * var(--measure-xs)); border: var(--stroke-thick) solid var(--tone); background: var(--wash); align-items: center; justify-content: center; }
.invalid [part="PlacementGhost.footprint"] { background: repeating-linear-gradient(45deg, var(--danger-45) 0, var(--danger-45) var(--space-sm), transparent var(--space-sm), transparent var(--space-lg)), var(--wash); }
#Grid { position: absolute; top: 0; inset-inline-start: 0; width: stretch; height: stretch; opacity: var(--opacity-dim); --cell: calc(100% / var(--cells)); background: repeating-linear-gradient(0deg, var(--scene-ink) 0, var(--scene-ink) var(--stroke-hairline), transparent var(--stroke-hairline), transparent var(--cell)), repeating-linear-gradient(90deg, var(--scene-ink) 0, var(--scene-ink) var(--stroke-hairline), transparent var(--stroke-hairline), transparent var(--cell)); }
/* In the world, over any ground: the badge takes the scene's plate. */
[part="PlacementGhost.badge"] { font: var(--text-lg); padding: var(--space-xxs) var(--space-sm); white-space: nowrap; --badge-fill: var(--scene-plate); --badge-ink: var(--tone); --badge-radius: var(--radius-control); }
</style>
