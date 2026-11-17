<meta name="component" content="MinimapView">
<meta name="strings" content="./minimap-view.en.ogt">
<meta name="parts" content="MinimapView.bezel MinimapView.glass MinimapView.grid MinimapView.marker MinimapView.player MinimapView.north MinimapView.badge MinimapView.plate MinimapView.cell MinimapView.frame MinimapView.dot MinimapView.gridline MinimapView.tag">
<meta name="prop:texture" content="number" data-doc="The map texture (og.load.texture); 0 draws the well with a grid.">
<meta name="prop:centerX" content="number" data-doc="The map point at its centre (the player's), 0 to 1 across.">
<meta name="prop:centerY" content="number" data-doc="The map point at its centre, 0 to 1 down.">
<meta name="prop:zoom" content="number" data-default="0.3" data-doc="How much of the map it shows across, 0 to 1 (0.3).">
<meta name="prop:heading" content="number" data-doc="The player's heading in degrees clockwise from north.">
<meta name="prop:rotate" content="bool" data-doc="Player-up: the map turns with the heading; false keeps north up and turns the player's arrow.">
<meta name="prop:square" content="bool" data-doc="A square frame; false is round.">
<meta name="prop:markers" content="list" data-doc="Its markers: each { id, kind, x, y, label }, x and y on the map (0 to 1), kind ally, enemy, objective, ping, loot or poi.">
<meta name="prop:zone" content="string" data-doc="The zone's name under it; empty is none.">
<meta name="action:pick" content="Pick">
<meta name="fixture:default" content='{"texture":0,"centerX":0.5,"centerY":0.5,"zoom":0.3,"heading":30,"rotate":false,"square":false,"zone":"Ashen Vale","markers":[{"id":"ally","kind":"ally","x":0.45,"y":0.42,"label":"Mira"},{"id":"foe","kind":"enemy","x":0.58,"y":0.55,"label":"Enemy"},{"id":"quest","kind":"objective","x":0.6,"y":0.38,"label":"Quest"},{"id":"ping","kind":"ping","x":0.42,"y":0.6,"label":"Ping"},{"id":"chest","kind":"loot","x":0.53,"y":0.47,"label":"Chest"}]}'>

<div id="MinimapView" data-layout="stack" data-gap="loose" role="image" aria-label={zone || t("component.minimap-view.name")} class:square={square} style:--cx={centerX} style:--cy={centerY} style:--zoom={zoom} style:--heading={heading}>
  <div id="Bezel" part="MinimapView.bezel MinimapView.frame" class="hud" data-layout="overlay">
    <button id="Glass" part="MinimapView.glass MinimapView.cell" data-shape={square ? 'square' : 'round'} onclick={pick} aria-label={t("component.minimap-view.name")}>
      <div id="World" data-layout="overlay" class:turning={rotate}>
        {#if texture}<img id="Map" src={texture}>
        {:else}<svg id="Grid" part="MinimapView.grid MinimapView.plate" data-anchor="fill" viewBox="0 0 200 200"><path part="MinimapView.gridline" d="M0 20H200 M0 40H200 M0 60H200 M0 80H200 M0 100H200 M0 120H200 M0 140H200 M0 160H200 M0 180H200 M20 0V200 M40 0V200 M60 0V200 M80 0V200 M100 0V200 M120 0V200 M140 0V200 M160 0V200 M180 0V200" fill="none"></path></svg>{/if}
        {#each markers as m (m.id)}
          <div class="marker" part="MinimapView.marker MinimapView.cell MinimapView.dot" data-shape={m.kind == 'enemy' || m.kind == 'objective' ? 'diamond' : m.kind == 'loot' ? 'square' : 'round'} data-tone={m.kind == 'ally' ? 'info' : m.kind == 'enemy' ? 'danger' : m.kind == 'objective' ? 'primary' : m.kind == 'ping' ? 'warn' : null} data-motion={m.kind == 'ping' ? 'ping' : null} class:ally={m.kind == 'ally'} class:enemy={m.kind == 'enemy'} class:objective={m.kind == 'objective'} class:ping={m.kind == 'ping'} class:loot={m.kind == 'loot'} style:--x={m.x} style:--y={m.y} aria-label={m.label}></div>
        {/each}
      </div>
      <svg id="Player" part="MinimapView.player" viewBox="0 0 16 16" class:turning={!rotate}><path d="M8 1l6 13-6-3.5L2 14z"></path></svg>
      <span id="North" class="north" part="MinimapView.north MinimapView.tag" data-anchor="top" data-tone="danger">{t("component.minimap-view.north")}</span>
    </button>
  </div>
  {#if zone}<span id="Zone" part="MinimapView.badge">{zone}</span>{/if}
</div>

<style>
:host { align-items: center; --side: var(--size-5xl); --pip: var(--space-sm); --layout-anchor-inset: var(--space-xs); --dot-fill: currentColor; }
#Bezel { width: calc(var(--side) + var(--space-xs) * 2); height: calc(var(--side) + var(--space-xs) * 2); padding: var(--space-xs); --plate-radius: var(--radius-capsule); }
.square #Bezel { --plate-radius: var(--radius-panel); }
#Glass { position: relative; width: var(--side); height: var(--side); padding: 0; overflow: hidden; --cell-fill: var(--well); --cell-radius: var(--radius-card); }
#World { position: absolute; top: 0; inset-inline-start: 0; width: var(--side); height: var(--side); transform-origin: 50% 50%; }
#World.turning { rotate: calc(var(--heading) * -1deg); }
#Map { position: absolute; width: calc(var(--side) / var(--zoom)); height: calc(var(--side) / var(--zoom)); inset-inline-start: calc(var(--side) / 2 - var(--cx) * var(--side) / var(--zoom)); top: calc(var(--side) / 2 - var(--cy) * var(--side) / var(--zoom)); }
#Grid { width: var(--side); height: var(--side); }
.marker { position: absolute; width: var(--pip); height: var(--pip); inset-inline-start: calc(var(--side) / 2 + (var(--x) - var(--cx)) * var(--side) / var(--zoom) - var(--pip) / 2); top: calc(var(--side) / 2 + (var(--y) - var(--cy)) * var(--side) / var(--zoom) - var(--pip) / 2); }
.marker.objective { --pip: var(--space-md); }
.marker.ping { --dot-fill: transparent; --dot-rim: var(--tone-warn); --dot-rim-width: var(--stroke-regular); }
.marker.loot { --dot-fill: var(--xp); }
[part="MinimapView.player"] { position: absolute; top: calc(var(--side) / 2 - var(--size-xs) / 2); inset-inline-start: calc(var(--side) / 2 - var(--size-xs) / 2); width: var(--size-xs); height: var(--size-xs); fill: var(--dot-fill); stroke: var(--dot-rim); stroke-width: var(--dot-rim-width); --dot-fill: var(--primary); --dot-rim: var(--ink); transform-origin: 50% 50%; }
[part="MinimapView.player"].turning { rotate: calc(var(--heading) * 1deg); }
/* Inside the glass, under the bezel's top: on the map, clear of the bezel. */
[part~="MinimapView.north"] { white-space: nowrap; }
[part="MinimapView.badge"] { white-space: nowrap; }
</style>
