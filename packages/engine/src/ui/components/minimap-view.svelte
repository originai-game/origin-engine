<meta name="component" content="MinimapView">
<meta name="strings" content="./minimap-view.en.ogt">
<meta name="parts" content="MinimapView.bezel MinimapView.glass MinimapView.grid MinimapView.marker MinimapView.player MinimapView.north MinimapView.badge">
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

<div id="MinimapView" role="image" aria-label={zone || t("component.minimap-view.name")} class:square={square} style:--cx={centerX} style:--cy={centerY} style:--zoom={zoom} style:--heading={heading}>
  <div id="Bezel" part="MinimapView.bezel">
    <button id="Glass" part="MinimapView.glass" onclick={pick} aria-label={t("component.minimap-view.name")}>
      <div id="World" class:turning={rotate}>
        <img id="Map" src={texture} hidden={!texture}>
        <div id="Grid" part="MinimapView.grid" hidden={!!texture}></div>
        {#each markers as m (m.id)}
          <div class="marker" part="MinimapView.marker" class:ally={m.kind == 'ally'} class:enemy={m.kind == 'enemy'} class:objective={m.kind == 'objective'} class:ping={m.kind == 'ping'} class:loot={m.kind == 'loot'} style:--x={m.x} style:--y={m.y} aria-label={m.label}></div>
        {/each}
      </div>
      <svg id="Player" part="MinimapView.player" viewBox="0 0 16 16" class:turning={!rotate}><path d="M8 1l6 13-6-3.5L2 14z"></path></svg>
    </button>
    <span id="North" part="MinimapView.north">{t("component.minimap-view.north")}</span>
  </div>
  <span id="Zone" part="MinimapView.badge" hidden={!zone}>{zone}</span>
</div>

<style>
#MinimapView { align-items: center; gap: var(--space-xs); --side: var(--size-5xl); --pip: var(--space-sm); }
[part="MinimapView.bezel"] { width: calc(var(--side) + var(--space-xs) * 2); height: calc(var(--side) + var(--space-xs) * 2); padding: var(--space-xs); border-radius: var(--radius-capsule); background: linear-gradient(180deg, var(--panel-raised), var(--well)); box-shadow: var(--shadow-minimap); }
.square [part="MinimapView.bezel"] { border-radius: var(--radius-panel); }
[part="MinimapView.glass"] { width: var(--side); height: var(--side); padding: 0; border-radius: var(--radius-capsule); overflow: hidden; background: var(--well); border: var(--stroke-hairline) solid var(--color-minimap-edge); }
.square [part="MinimapView.glass"] { border-radius: var(--radius-card); }
#World { position: absolute; top: 0; inset-inline-start: 0; width: var(--side); height: var(--side); transform-origin: 50% 50%; }
#World.turning { rotate: calc(var(--heading) * -1deg); }
#Map { position: absolute; width: calc(var(--side) / var(--zoom)); height: calc(var(--side) / var(--zoom)); inset-inline-start: calc(var(--side) / 2 - var(--cx) * var(--side) / var(--zoom)); top: calc(var(--side) / 2 - var(--cy) * var(--side) / var(--zoom)); }
[part="MinimapView.grid"] { position: absolute; top: 0; inset-inline-start: 0; width: var(--side); height: var(--side); background: repeating-linear-gradient(0deg, var(--color-minimap-edge) 0%, var(--color-minimap-edge) var(--stroke-hairline), transparent var(--stroke-hairline), transparent var(--size-sm)), repeating-linear-gradient(90deg, var(--color-minimap-edge) 0%, var(--color-minimap-edge) var(--stroke-hairline), transparent var(--stroke-hairline), transparent var(--size-sm)); }
[part="MinimapView.marker"] { position: absolute; width: var(--pip); height: var(--pip); border-radius: var(--radius-capsule); inset-inline-start: calc(var(--side) / 2 + (var(--x) - var(--cx)) * var(--side) / var(--zoom) - var(--pip) / 2); top: calc(var(--side) / 2 + (var(--y) - var(--cy)) * var(--side) / var(--zoom) - var(--pip) / 2); background: var(--ink); border: var(--stroke-hairline) solid var(--scene-shadow); }
.marker.ally { background: var(--info); }
.marker.enemy { background: var(--danger); border-radius: 0; rotate: 45deg; }
.marker.objective { --pip: var(--space-md); border-radius: var(--radius-control); rotate: 45deg; background: var(--gold); }
.marker.ping { background: transparent; border: var(--stroke-regular) solid var(--warn); animation: ping calc(var(--motion-duration-slow) * 3) var(--motion-easing-out) infinite; }
.marker.loot { border-radius: 0; background: var(--xp); }
[part="MinimapView.player"] { position: absolute; top: calc(var(--side) / 2 - var(--size-xs) / 2); inset-inline-start: calc(var(--side) / 2 - var(--size-xs) / 2); width: var(--size-xs); height: var(--size-xs); fill: var(--primary); stroke: var(--ink); stroke-width: 1.2; transform-origin: 50% 50%; }
[part="MinimapView.player"].turning { rotate: calc(var(--heading) * 1deg); }
[part="MinimapView.north"] { position: absolute; top: calc(var(--space-xs) * -1); inset-inline-start: 0; width: stretch; text-align: center; font: var(--text-caption); color: var(--danger); text-shadow: 0 var(--stroke-hairline) var(--stroke-regular) var(--scene-shadow); }
[part="MinimapView.badge"] { font: var(--text-caption); padding: var(--space-xxs) var(--space-sm); white-space: nowrap; }
@keyframes ping { from { scale: var(--motion-scale-shrink); opacity: 1; } to { scale: 2; opacity: 0; } }
</style>
