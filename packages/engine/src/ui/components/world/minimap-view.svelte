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

<!--
  A minimap: a lit bezel round (or square) dark glass, bearing ticks
  inside its rim, a red north badge pinned on the rim, the player's arrow
  in the middle with a heading cone. Each marker kind keeps its own shape
  at any size (an ally a blue disc, an enemy a red diamond, an objective a
  larger gold diamond, loot a violet square, a ping an orange ring), so a
  kind is never told by colour alone. The zone's name sits on a plate
  under it. Reports pick (a tap on the glass).
-->
<div id="MinimapView" data-layout="stack" data-gap="tight" role="image" aria-label={zone || t("component.minimap-view.name")} class:square={square} style:--cx={centerX} style:--cy={centerY} style:--zoom={zoom} style:--heading={heading}>
  <div id="Bezel" part="MinimapView.bezel">
    <button id="Glass" part="MinimapView.glass" data-shape={square ? 'square' : 'round'} onclick={pick} aria-label={t("component.minimap-view.name")}>
      <div id="World" class:turning={rotate}>
        {#if texture}<img id="Map" src={texture}>{:else}<div id="Grid" part="MinimapView.grid"></div>{/if}
        {#each markers as m (m.id)}
          <div class="marker" part="MinimapView.marker" data-kind={m.kind} style:--x={m.x} style:--y={m.y} aria-label={m.label}>{#if m.kind == 'ping'}<div class="echo" data-motion="ping"></div>{/if}</div>
        {/each}
      </div>
      {#if !square}<div class="ticks"></div>{/if}
      <div class="cone" class:turning={!rotate}></div>
      <svg id="Player" part="MinimapView.player" viewBox="0 0 16 16" class:turning={!rotate}><path d="M8 1l6 13-6-3.5L2 14z"></path></svg>
    </button>
    <span id="North" class="north" part="MinimapView.north">{t("component.minimap-view.north")}</span>
  </div>
  {#if zone}<span id="Zone" part="MinimapView.badge">{zone}</span>{/if}
</div>

<style>
:host { align-items: center; color: var(--hud-ink); --side: var(--size-5xl); --rim: calc(5 * var(--font-step-px)); --pip: var(--map-pip); }
#Bezel { position: relative; width: calc(var(--side) + var(--rim) * 2); height: calc(var(--side) + var(--rim) * 2); padding: var(--rim); border-radius: var(--map-round); background: var(--map-bezel); box-shadow: var(--map-bezel-shadow); }
.square #Bezel { border-radius: var(--map-square-bezel-round); }
#Glass { position: relative; width: var(--side); height: var(--side); padding: 0; border: 0; overflow: hidden; border-radius: var(--map-round); background: var(--map-glass-well); box-shadow: none; }
.square #Glass { border-radius: var(--map-square-glass-round); }
#Glass::after { content: ""; position: absolute; inset: 0; border-radius: inherit; box-shadow: var(--map-glass-rim); }
#World { position: absolute; top: 0; inset-inline-start: 0; width: var(--side); height: var(--side); transform-origin: 50% 50%; }
#World.turning { rotate: calc(var(--heading) * -1deg); }
#Map { position: absolute; width: calc(var(--side) / var(--zoom)); height: calc(var(--side) / var(--zoom)); inset-inline-start: calc(var(--side) / 2 - var(--cx) * var(--side) / var(--zoom)); top: calc(var(--side) / 2 - var(--cy) * var(--side) / var(--zoom)); }
#Grid { position: absolute; inset: 0; background: var(--map-grid); }
.ticks { position: absolute; inset: 0; border-radius: var(--map-round); background: var(--map-ticks); mask-image: radial-gradient(circle, transparent 46%, black 46.5%); }
.cone { position: absolute; left: 50%; top: 50%; width: calc(var(--side) * 0.48); height: calc(var(--side) * 0.48); translate: -50% -100%; transform-origin: 50% 100%; background: var(--map-cone); }
.cone.turning { rotate: calc(var(--heading) * 1deg); }
/* A marker: its kind is its shape, at a size that follows the text. */
.marker { position: absolute; width: var(--pip); height: var(--pip); inset-inline-start: calc(var(--side) / 2 + (var(--x) - var(--cx)) * var(--side) / var(--zoom) - var(--pip) / 2); top: calc(var(--side) / 2 + (var(--y) - var(--cy)) * var(--side) / var(--zoom) - var(--pip) / 2); }
.marker[data-kind=ally], .marker[data-kind=poi] { border-radius: var(--map-round); background: var(--map-ally); box-shadow: var(--map-pip-ring); }
.marker[data-kind=enemy] { clip-path: polygon(50% 0, 100% 50%, 50% 100%, 0 50%); background: var(--map-enemy); }
.marker[data-kind=objective] { --pip: var(--map-pip-objective); clip-path: polygon(50% 0, 100% 50%, 50% 100%, 0 50%); background: var(--map-objective); filter: var(--map-quest-glow); }
.marker[data-kind=loot] { --pip: calc(var(--map-pip) * 0.85); background: var(--map-loot); box-shadow: var(--map-pip-ring); }
.marker[data-kind=ping] { --pip: calc(var(--map-pip) * 1.5); border-radius: var(--map-round); box-shadow: var(--map-ping-ring); }
.echo { position: absolute; inset: 0; border-radius: var(--map-round); box-shadow: var(--map-ping-ring); }
#Player { position: absolute; top: calc(var(--side) / 2 - var(--map-pip-objective) * 0.75); inset-inline-start: calc(var(--side) / 2 - var(--map-pip-objective) * 0.75); width: calc(var(--map-pip-objective) * 1.5); height: calc(var(--map-pip-objective) * 1.5); fill: var(--map-player); filter: var(--map-player-glow); transform-origin: 50% 50%; }
#Player.turning { rotate: calc(var(--heading) * 1deg); }
/* North: a red badge at the top of the glass, inside the bezel. */
.north { position: absolute; top: calc(var(--rim) + var(--space-xxs)); left: 50%; translate: -50% 0; min-width: var(--map-north-size); height: var(--map-north-size); display: block; text-align: center; line-height: var(--map-north-size); border-radius: var(--map-round); font-size: var(--hud-size-xs); font-weight: var(--hud-figure-weight); line-height: 1; color: var(--hud-ink); background: var(--map-north); box-shadow: var(--map-north-ring); white-space: nowrap; }
#Zone { padding: calc(2 * var(--font-step-px)) var(--space-sm); font-size: var(--hud-size-xs); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); white-space: nowrap; clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); }
</style>
