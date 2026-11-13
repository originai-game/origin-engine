<meta name="component" content="Minimap">
<meta name="parts" content="Minimap.frame Minimap.marker Minimap.label">
<meta name="prop:texture" content="number" data-doc="The map texture (og.load.texture); 0 is the stage colour.">
<meta name="prop:u0" content="number" data-doc="The region of the map shown: its left edge, 0 to 1.">
<meta name="prop:v0" content="number" data-doc="Its top edge, 0 to 1 (v down).">
<meta name="prop:u1" content="number" data-doc="Its right edge (1: the whole width).">
<meta name="prop:v1" content="number" data-doc="Its bottom edge (1: the whole height).">
<meta name="prop:size" content="number" data-doc="Its side in px (160).">
<meta name="prop:markers" content="list" data-doc="Markers over the map: each { id, x, y, tone, player }, x and y across and down the shown region (0 to 1), tone neutral, info, success, warn, danger or accent; the player is the primary, larger and ringed.">
<meta name="prop:label" content="string" data-doc="A zone name under it; empty is none.">
<meta name="fixture:default" content='{"texture":0,"u0":0,"v0":0,"u1":1,"v1":1,"size":160,"label":"Ashen Vale","markers":[{"id":"me","x":0.5,"y":0.5,"tone":"neutral","player":true},{"id":"a","x":0.25,"y":0.3,"tone":"danger","player":false},{"id":"b","x":0.72,"y":0.65,"tone":"success","player":false},{"id":"c","x":0.6,"y":0.2,"tone":"warn","player":false}]}'>

<div id="Minimap" role="image" aria-label={label} style:--size={size + "px"}>
  <div id="Frame" part="Minimap.frame" class="hud">
    <img id="Map" src={texture} hidden={!texture} style:--u0={u0} style:--v0={v0} style:--u1={u1} style:--v1={v1}>
    {#each markers as m (m.id)}
      <div class="marker" part="Minimap.marker" class:player={m.player} class:info={m.tone == 'info'} class:success={m.tone == 'success'} class:warn={m.tone == 'warn'} class:danger={m.tone == 'danger'} class:accent={m.tone == 'accent'} style:--x={m.x} style:--y={m.y}></div>
    {/each}
  </div>
  <span id="Label" part="Minimap.label" hidden={!label}>{label}</span>
</div>

<style>
#Minimap { align-items: center; gap: var(--space-xs); --pip: var(--space-sm); }
[part="Minimap.frame"] { width: var(--size); height: var(--size); padding: 0; overflow: hidden; }
#Map { position: absolute; top: 0; inset-inline-start: 0; width: stretch; height: stretch; object-view-box: inset(calc(var(--v0) * 100%) calc((1 - var(--u1)) * 100%) calc((1 - var(--v1)) * 100%) calc(var(--u0) * 100%)); }
[part="Minimap.marker"] { position: absolute; width: var(--pip); height: var(--pip); border-radius: var(--radius-capsule); inset-inline-start: calc(var(--x) * var(--size) - var(--pip) / 2); top: calc(var(--y) * var(--size) - var(--pip) / 2); background: var(--ink); border: var(--stroke-hairline) solid var(--scene-shadow); }
.marker.info { background: var(--info); }
.marker.success { background: var(--success); }
.marker.warn { background: var(--warn); }
.marker.danger { background: var(--danger); }
.marker.accent { background: var(--accent); }
.marker.player { --pip: var(--space-md); background: var(--primary); border: var(--stroke-regular) solid var(--ink); }
[part="Minimap.label"] { font: var(--text-caption); color: var(--scene-ink); text-shadow: 0 var(--stroke-hairline) var(--stroke-regular) var(--scene-shadow); white-space: nowrap; }
</style>
