<meta name="component" content="Minimap">
<meta name="prop:texture" content="number" data-doc="The map texture (og.load.texture); 0 is the stage colour.">
<meta name="prop:u0" content="number" data-doc="The region of the map shown: its left edge, 0 to 1.">
<meta name="prop:v0" content="number" data-doc="Its top edge, 0 to 1 (v down).">
<meta name="prop:u1" content="number" data-doc="Its right edge (1: the whole width).">
<meta name="prop:v1" content="number" data-doc="Its bottom edge (1: the whole height).">
<meta name="prop:size" content="number" data-doc="Its side in px (160).">
<meta name="prop:markers" content="list" data-doc="Markers over the map: each { id, x, y, tone, player }, x and y across and down the shown region (0 to 1), tone neutral, info, success, warn, danger or accent; the player is the primary, larger and ringed.">
<meta name="prop:label" content="string" data-doc="A zone name under it; empty is none.">
<meta name="fixture:default" content='{"texture":0,"u0":0,"v0":0,"u1":1,"v1":1,"size":160,"label":"Ashen Vale","markers":[{"id":"me","x":0.5,"y":0.5,"tone":"neutral","player":true},{"id":"a","x":0.25,"y":0.3,"tone":"danger","player":false},{"id":"b","x":0.72,"y":0.65,"tone":"success","player":false},{"id":"c","x":0.6,"y":0.2,"tone":"warn","player":false}]}'>

<div id="Minimap" role="image" aria-label={label} style:--size={size}>
  <div id="Frame">
    <img id="Map" src={texture} hidden={!texture} style:--u0={u0} style:--v0={v0} style:--u1={u1} style:--v1={v1}>
    {#each markers as m (m.id)}
      <div class="marker" class:player={m.player} class:info={m.tone == 'info'} class:success={m.tone == 'success'} class:warn={m.tone == 'warn'} class:danger={m.tone == 'danger'} class:accent={m.tone == 'accent'} style:--x={m.x} style:--y={m.y}></div>
    {/each}
  </div>
  <span id="Label" hidden={!label}>{label}</span>
</div>

<style>
#Minimap { align-items: center; gap: 6px; }
#Frame { width: calc(var(--size) * 1px); height: calc(var(--size) * 1px); border-radius: 12px; overflow: hidden; background: var(--well); border: 2px solid var(--line-strong); box-shadow: 0 4px 12px #00000080; }
#Map { position: absolute; top: 0; inset-inline-start: 0; width: stretch; height: stretch; -og-uv: var(--u0) var(--v0) var(--u1) var(--v1); }
.marker { position: absolute; width: 8px; height: 8px; border-radius: 4px; inset-inline-start: calc(var(--x) * var(--size) * 1px - 4px); top: calc(var(--y) * var(--size) * 1px - 4px); background: var(--ink); border: 1px solid #000000b3; }
.marker.info { background: var(--info); }
.marker.success { background: var(--success); }
.marker.warn { background: var(--warn); }
.marker.danger { background: var(--danger); }
.marker.accent { background: var(--accent); }
.marker.player { width: 12px; height: 12px; border-radius: 6px; inset-inline-start: calc(var(--x) * var(--size) * 1px - 6px); top: calc(var(--y) * var(--size) * 1px - 6px); background: var(--primary); border: 2px solid var(--ink); }
#Label { font: var(--text-caption); color: var(--ink); text-shadow: 0 1px 2px #000000ff; white-space: nowrap; }
</style>
