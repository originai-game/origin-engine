<meta name="component" content="PathLine">
<meta name="parts" content="PathLine.line PathLine.badge">
<meta name="prop:at" content="string" data-doc="The ground box's centre (x y z), the path laid in it.">
<meta name="prop:size" content="number" data-doc="The box's side in world units; the path's points are in its 0 0 200 200 view box.">
<meta name="prop:path" content="string" data-doc="The path from the mover as SVG path data in the box (M x y L x y …), the game's world points projected onto it.">
<meta name="prop:reach" content="string" data-doc="The part past the mover's reach this turn, as path data, drawn in danger; empty is none.">
<meta name="prop:endX" content="number" data-doc="The path's end in the box, across.">
<meta name="prop:endY" content="number" data-doc="The path's end in the box, down.">
<meta name="prop:label" content="string" data-doc="A label at its end (12 m · 2 AP); empty is none.">
<meta name="prop:invalid" content="bool" data-doc="It cannot be taken: danger, with a cross at its end.">
<meta name="fixture:default" content='{"at":"0 0 0","size":10,"path":"M40 160 L80 120 L120 120 L150 70","reach":"M150 70 L170 40","endX":170,"endY":40,"label":"12 m · 2 AP","invalid":false}'>

<ground id="PathLine" at={at} size={size + " " + size}>
  <svg id="Line" part="PathLine.line" viewBox="0 0 200 200" class:invalid={invalid}>
    <path id="Walk" d={path} fill="none" stroke-width="4" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="10 6"></path>
    <path id="Past" d={reach} fill="none" stroke-width="4" stroke-linecap="round" stroke-dasharray="10 6" hidden={!reach}></path>
    <circle id="End" cx={endX} cy={endY} r="7" hidden={invalid}></circle>
    <path id="Cross" d={"M" + (endX - 7) + " " + (endY - 7) + " l14 14 m0 -14 l-14 14"} stroke-width="4" stroke-linecap="round" hidden={!invalid}></path>
  </svg>
  <span id="Label" part="PathLine.badge" hidden={!label} style:--x={endX} style:--y={endY}>{label}</span>
</ground>

<style>
#PathLine { width: var(--measure-xs); height: var(--measure-xs); }
[part="PathLine.line"] { width: var(--measure-xs); height: var(--measure-xs); --tone: var(--highlight); }
[part="PathLine.line"].invalid { --tone: var(--danger); }
#Walk { stroke: var(--tone); }
#Past { stroke: var(--danger); }
#End { fill: var(--tone); stroke: var(--scene-shadow); stroke-width: 2; }
#Cross { stroke: var(--danger); }
/* In the world, over any ground: the badge takes the scene's plate and ink. */
[part="PathLine.badge"] { position: absolute; inset-inline-start: calc(var(--x) / 200 * var(--measure-xs) + var(--space-sm)); top: calc(var(--y) / 200 * var(--measure-xs) - var(--space-sm)); font: var(--text-caption); padding: 0 var(--space-xs); white-space: nowrap; --badge-fill: var(--scene-plate); --badge-ink: var(--scene-ink); --badge-radius: var(--radius-control); }
</style>
