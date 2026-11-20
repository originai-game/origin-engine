<meta name="component" content="Minimap">
<meta name="prop:texture" content="number" data-doc="The map texture (og.load.texture); 0 draws a grid over the stage colour.">
<meta name="prop:u0" content="number" data-doc="The region of the map shown: its left edge, 0 to 1.">
<meta name="prop:v0" content="number" data-doc="Its top edge, 0 to 1 (v down).">
<meta name="prop:u1" content="number" data-default="1" data-doc="Its right edge (1: the whole width).">
<meta name="prop:v1" content="number" data-default="1" data-doc="Its bottom edge (1: the whole height).">
<meta name="prop:size" content="number" data-default="160" data-doc="Its side in px (160).">
<meta name="prop:markers" content="list" data-doc="Markers over the map: each { id, x, y, tone, player }, x and y across and down the shown region (0 to 1), tone neutral, info, success, warn, danger or accent; the player is the primary, larger and ringed.">
<meta name="prop:label" content="string" data-doc="A zone name under it; empty is none.">
<meta name="fixture:default" content='{"texture":0,"u0":0,"v0":0,"u1":1,"v1":1,"size":160,"label":"Ashen Vale","markers":[{"id":"me","x":0.5,"y":0.5,"tone":"neutral","player":true},{"id":"a","x":0.25,"y":0.3,"tone":"danger","player":false},{"id":"b","x":0.72,"y":0.65,"tone":"success","player":false},{"id":"c","x":0.6,"y":0.2,"tone":"warn","player":false}]}'>

<div id="Minimap" layout="stack" gap="loose" role="image" aria-label={label} style:--size={size + "px"}>
  <div class="frame" id="Frame" part="frame" variant="hud" layout="overlay">
    {#if texture}<img id="Map" anchor="fill" src={texture} style:--u0={u0} style:--v0={v0} style:--u1={u1} style:--v1={v1}>
    {:else}<svg class="grid" part="grid" anchor="fill" viewBox="0 0 160 160"><path part="gridline" d="M0 20H160 M0 40H160 M0 60H160 M0 80H160 M0 100H160 M0 120H160 M0 140H160 M20 0V160 M40 0V160 M60 0V160 M80 0V160 M100 0V160 M120 0V160 M140 0V160" fill="none"></path></svg>{/if}
    {#each markers as m (m.id)}
      <div class="marker cell" part="dot" shape="round" tone={m.player ? 'primary' : m.tone} class:player={m.player} class:info={m.tone == 'info'} class:success={m.tone == 'success'} class:warn={m.tone == 'warn'} class:danger={m.tone == 'danger'} class:accent={m.tone == 'accent'} style:--x={m.x} style:--y={m.y}></div>
    {/each}
  </div>
  {#if label}<span id="Label" class="label" part="tag">{label}</span>{/if}
</div>

<style>
#Minimap { align-items: center; --pip: var(--space-sm); --layout-anchor-inset: 0; --dot-fill: currentColor; }
.frame { width: var(--size); height: var(--size); padding: 0; overflow: hidden; }
#Map { width: 100%; height: 100%; object-view-box: inset(calc(var(--v0) * 100%) calc((1 - var(--u1)) * 100%) calc((1 - var(--v1)) * 100%) calc(var(--u0) * 100%)); }
.grid { width: 100%; height: 100%; }
.marker { position: absolute; width: var(--pip); height: var(--pip); inset-inline-start: calc(var(--x) * var(--size) - var(--pip) / 2); top: calc(var(--y) * var(--size) - var(--pip) / 2); }
.marker.accent { --dot-fill: var(--accent); }
.marker.player { --pip: var(--space-md); --dot-rim-width: var(--stroke-regular); --dot-rim: var(--ink); }
.label { white-space: nowrap; }
</style>
