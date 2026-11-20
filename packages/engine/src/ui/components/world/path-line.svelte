<meta name="component" content="PathLine">
<meta name="prop:at" content="string" data-default="0 0 0" data-doc="The ground box's centre (x y z), the path laid in it.">
<meta name="prop:size" content="number" data-default="1" data-doc="The box's side in world units; the path's points are in its 0 0 200 200 view box.">
<meta name="prop:path" content="string" data-doc="The path from the mover as SVG path data in the box (M x y L x y …), the game's world points projected onto it.">
<meta name="prop:reach" content="string" data-doc="The part past the mover's reach this turn, as path data, drawn red; empty is none.">
<meta name="prop:stops" content="list" data-doc="The numbered waypoints on the way, keyed rows of { id, x, y } in the box.">
<meta name="prop:endX" content="number" data-doc="The path's end in the box, across.">
<meta name="prop:endY" content="number" data-doc="The path's end in the box, down.">
<meta name="prop:label" content="string" data-doc="A badge at its end (64 m · 3 orders); empty is none.">
<meta name="prop:invalid" content="bool" data-doc="It cannot be taken: red, with a cross at its end.">
<meta name="fixture:default" content='{"at":"0 0 0","size":10,"path":"M30 150 L80 120 L120 110 L150 70","reach":"M150 70 L172 40","stops":[{"id":"1","x":80,"y":120},{"id":"2","x":120,"y":110}],"endX":172,"endY":40,"label":"64 m · 3 orders","invalid":false}'>
<meta name="fixture:invalid" content='{"reach":"","stops":[],"endX":150,"endY":70,"label":"No path","invalid":true}'>

<!--
  The way a unit will go, laid on the ground: white dashes over a dark
  under-stroke, numbered stops, the part past this turn's reach red and a
  badge at its end; an impossible path red with a cross. Read only.
-->
<div id="PathLine" class="bare">
<ground class="plane" at={at} size={size + " " + size}>
  <div id="Room" style:--x={endX} style:--y={endY} class:invalid={invalid}>
    <svg part="line" class="line" viewBox="0 0 200 200">
      <path part="under" class="under" d={path}></path>
      <path part="walk" class="walk" d={path}></path>
      {#if reach != '' && !invalid}<path part="under" class="under" d={reach}></path><path part="past" class="past" d={reach}></path>{/if}
      {#each stops as s (s.id)}<circle part="stop" class="stop" cx={s.x} cy={s.y} r="7"></circle>{/each}
      {#if invalid}<path part="end" class="cross" d={"M" + (endX - 7) + " " + (endY - 7) + " l14 14 m0 -14 l-14 14"}></path>
      {:else}<circle part="end" class="end" cx={endX} cy={endY} r="8"></circle>{/if}
    </svg>
    {#each stops as s, i (s.id)}<span class="number" style:--sx={s.x} style:--sy={s.y}>{i + 1}</span>{/each}
  </div>
</ground>
{#if label != ''}
  <!-- The badge stands upright beside the end, where the box's point lies on the ground. -->
  <anchor id="Badge" at={at} offset={((endX / 200 - 0.5) * size) + " 0 " + ((endY / 200 - 0.5) * size)} declutter="none" priority="5">
    <span part="badge" class="badge" class:invalid={invalid}>{label}</span>
  </anchor>
{/if}
</div>

<style>
.plane { inline-size: var(--measure-xs); block-size: var(--measure-xs); }
#Room { position: relative; inline-size: var(--measure-xs); block-size: var(--measure-xs); }
.line { position: absolute; inset: 0; inline-size: 100%; block-size: 100%; filter: var(--world-path-glow); }
.under { fill: none; stroke: var(--world-path-under); stroke-width: 8; stroke-linecap: round; stroke-linejoin: round; }
.walk { fill: none; stroke: var(--world-path-ink); stroke-width: 3.5; stroke-linecap: round; stroke-linejoin: round; stroke-dasharray: 11 7; }
.invalid .walk { stroke: var(--world-hostile-ink); }
.past { fill: none; stroke: var(--world-hostile-ink); stroke-width: 3.5; stroke-linecap: round; stroke-dasharray: 11 7; }
.stop { fill: var(--world-player-ink); stroke: var(--world-path-under); stroke-width: 2.5; }
.end { fill: none; stroke: var(--world-path-ink); stroke-width: 3; }
.cross { stroke: var(--world-hostile-ink); stroke-width: 4; stroke-linecap: round; }
.number { position: absolute; left: calc(var(--sx) / 200 * 100%); top: calc(var(--sy) / 200 * 100%); translate: -50% -50%; font-size: var(--world-size-title); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); color: var(--world-path-stop-ink); }
/* The badge beside the end: a dark tag, upright, a text box. */
.badge { display: block; margin-inline-start: calc(2 * var(--world-badge-lift)); translate: 50% 50%; padding: calc(2 * var(--font-step-px)) calc(8 * var(--font-step-px)); font-size: var(--world-size-badge); line-height: 1.2; font-weight: var(--hud-figure-weight); font-stretch: var(--world-caps-stretch); letter-spacing: var(--world-caps-tracking); text-transform: var(--hud-label-transform); white-space: nowrap; color: var(--hud-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.badge.invalid { color: var(--world-hostile-ink); }
</style>
