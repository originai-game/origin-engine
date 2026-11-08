<meta name="component" content="MinimapView">
<meta name="strings" content="./minimap-view.en.ogt">
<meta name="prop:texture" content="number" data-doc="The map texture (og.load.texture); 0 draws the well with a grid.">
<meta name="prop:centerX" content="number" data-doc="The map point at its centre (the player's), 0 to 1 across.">
<meta name="prop:centerY" content="number" data-doc="The map point at its centre, 0 to 1 down.">
<meta name="prop:zoom" content="number" data-doc="How much of the map it shows across, 0 to 1 (0.3).">
<meta name="prop:heading" content="number" data-doc="The player's heading in degrees clockwise from north.">
<meta name="prop:rotate" content="bool" data-doc="Player-up: the map turns with the heading; false keeps north up and turns the player's arrow.">
<meta name="prop:square" content="bool" data-doc="A square frame; false is round.">
<meta name="prop:markers" content="list" data-doc="Its markers: each { id, kind, x, y, label }, x and y on the map (0 to 1), kind ally, enemy, objective, ping, loot or poi.">
<meta name="prop:zone" content="string" data-doc="The zone's name under it; empty is none.">
<meta name="action:pick" content="Pick">
<meta name="fixture:default" content='{"texture":0,"centerX":0.5,"centerY":0.5,"zoom":0.3,"heading":30,"rotate":false,"square":false,"zone":"Ashen Vale","markers":[{"id":"ally","kind":"ally","x":0.45,"y":0.42,"label":"Mira"},{"id":"foe","kind":"enemy","x":0.58,"y":0.55,"label":"Enemy"},{"id":"quest","kind":"objective","x":0.6,"y":0.38,"label":"Quest"},{"id":"ping","kind":"ping","x":0.42,"y":0.6,"label":"Ping"},{"id":"chest","kind":"loot","x":0.53,"y":0.47,"label":"Chest"}]}'>

<div id="MinimapView" role="image" aria-label={zone || t("component.minimap-view.name")} class:square={square} style:--cx={centerX} style:--cy={centerY} style:--zoom={zoom} style:--heading={heading}>
  <div id="Bezel">
    <button id="Glass" onclick={pick} aria-label={t("component.minimap-view.name")}>
      <div id="World" class:turning={rotate}>
        <img id="Map" src={texture} hidden={!texture}>
        <div id="Grid" hidden={!!texture}></div>
        {#each markers as m (m.id)}
          <div class="marker" class:ally={m.kind == 'ally'} class:enemy={m.kind == 'enemy'} class:objective={m.kind == 'objective'} class:ping={m.kind == 'ping'} class:loot={m.kind == 'loot'} style:--x={m.x} style:--y={m.y} aria-label={m.label}></div>
        {/each}
      </div>
      <svg id="Player" viewBox="0 0 16 16" class:turning={!rotate}><path d="M8 1l6 13-6-3.5L2 14z"></path></svg>
    </button>
    <span id="North">{t("component.minimap-view.north")}</span>
  </div>
  <span id="Zone" hidden={!zone}>{zone}</span>
</div>

<style>
#MinimapView { align-items: center; gap: 6px; }
#Bezel { width: 200px; height: 200px; padding: 6px; border-radius: 100px; background: linear-gradient(180deg, #2a2f3aff, #10131aff); box-shadow: 0 6px 16px #0000008c; }
.square #Bezel { border-radius: 14px; }
#Glass { width: 188px; height: 188px; padding: 0; border-radius: 94px; overflow: hidden; background: var(--well); border: 1px solid #ffffff33; }
.square #Glass { border-radius: 10px; }
#World { position: absolute; top: 0; inset-inline-start: 0; width: 188px; height: 188px; transform-origin: 50% 50%; }
#World.turning { rotate: calc(var(--heading) * -1deg); }
#Map { position: absolute; width: calc(188px / var(--zoom)); height: calc(188px / var(--zoom)); inset-inline-start: calc(94px - var(--cx) * 188px / var(--zoom)); top: calc(94px - var(--cy) * 188px / var(--zoom)); }
#Grid { position: absolute; top: 0; inset-inline-start: 0; width: 188px; height: 188px; background: repeating-linear-gradient(0deg, #ffffff14 0px, #ffffff14 1px, #00000000 1px, #00000000 24px), repeating-linear-gradient(90deg, #ffffff14 0px, #ffffff14 1px, #00000000 1px, #00000000 24px); }
.marker { position: absolute; width: 8px; height: 8px; border-radius: 4px; inset-inline-start: calc(94px + (var(--x) - var(--cx)) * 188px / var(--zoom) - 4px); top: calc(94px + (var(--y) - var(--cy)) * 188px / var(--zoom) - 4px); background: var(--ink); border: 1px solid #000000b3; }
.marker.ally { background: var(--info); }
.marker.enemy { background: var(--danger); border-radius: 1px; rotate: 45deg; }
.marker.objective { width: 12px; height: 12px; border-radius: 2px; rotate: 45deg; background: var(--gold); }
.marker.ping { background: #00000000; border: 2px solid var(--warn); animation: ping 1200ms ease-out infinite; }
.marker.loot { border-radius: 1px; background: var(--xp); }
#Player { position: absolute; top: 86px; inset-inline-start: 86px; width: 16px; height: 16px; fill: var(--primary); stroke: var(--ink); stroke-width: 1.2; transform-origin: 50% 50%; }
#Player.turning { rotate: calc(var(--heading) * 1deg); }
#North { position: absolute; top: -4px; inset-inline-start: 92px; font: var(--text-caption); color: var(--danger); text-shadow: 0 1px 2px #000000ff; }
#Zone { font: var(--text-caption); padding: 2px 10px; border-radius: 10px; background: #000000a6; color: var(--ink); white-space: nowrap; }
@keyframes ping { from { scale: 0.6; opacity: 1; } to { scale: 2; opacity: 0; } }
</style>
