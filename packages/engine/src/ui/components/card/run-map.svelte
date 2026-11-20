<meta name="component" content="CardRunMap">
<meta name="strings" content="./card.en.ogt">
<meta name="parts" content="CardRunMap.frame CardRunMap.band CardRunMap.title CardRunMap.floor CardRunMap.board CardRunMap.legend CardRunMap.row">
<meta name="prop:title" content="string" data-doc="The map's heading (the act's name).">
<meta name="prop:floor" content="string" data-doc="Where the run stands as the game words it (Floor 6 of 15); empty hides it.">
<meta name="prop:nodes" content="list" data-doc="The rooms, NodeGraph's nodes: keyed rows of { id, x, y, label, icon, state, shape, team }. A room's kind is its icon and shape (fight sword circle, elite skull diamond, rest fire square, shop coin square, event flag hex, treasure gem hex, boss crown diamond, or the game's own) and its label; state is done (visited), current (where the player stands), available (reachable next, the game's answer) or locked (out of reach).">
<meta name="prop:links" content="list" data-doc="The paths, NodeGraph's links: keyed rows of { id, d, active }, d SVG path data in graph units, active a path taken.">
<meta name="prop:view" content="string" data-default="0 0 360 240" data-doc="The map's extent in graph units, an SVG viewBox (0 0 360 240).">
<meta name="prop:zoom" content="number" data-default="1" data-doc="NodeGraph's zoom, kept by the game from the zoom and pinch intents.">
<meta name="prop:panX" content="number" data-doc="NodeGraph's pan across, from the pan intent.">
<meta name="prop:panY" content="number" data-doc="NodeGraph's pan down.">
<meta name="prop:selected" content="string" data-doc="The room ringed as chosen or focused; empty is none.">
<meta name="prop:legend" content="list" data-doc="The legend, keyed rows of { id, icon, label, shape }, one per room kind, shape the shape its rooms take (circle, or diamond for an elite or the boss) so the legend draws the same mark; empty hides it.">
<meta name="fixture:default" content='{"title":"The Sunken Road","floor":"Floor 3 of 8","view":"0 0 360 240","zoom":1,"panX":0,"panY":0,"selected":"e2","links":[{"id":"l1","d":"M30 120 L 115 60","active":true},{"id":"l2","d":"M30 120 L 115 180","active":false},{"id":"l3","d":"M115 60 L 200 35","active":false},{"id":"l4","d":"M115 60 L 200 125","active":false},{"id":"l5","d":"M115 180 L 200 125","active":false},{"id":"l6","d":"M200 35 L 275 80","active":false},{"id":"l7","d":"M200 125 L 275 80","active":false},{"id":"l8","d":"M275 80 L 335 150","active":false}],"nodes":[{"id":"start","x":30,"y":120,"label":"Fight","icon":"sword","state":"done","shape":"circle","team":""},{"id":"here","x":115,"y":60,"label":"Event","icon":"flag","state":"current","shape":"circle","team":""},{"id":"alt","x":115,"y":180,"label":"Shop","icon":"coin","state":"locked","shape":"circle","team":""},{"id":"e1","x":200,"y":35,"label":"Elite","icon":"skull","state":"available","shape":"diamond","team":""},{"id":"e2","x":200,"y":125,"label":"Rest","icon":"fire","state":"available","shape":"circle","team":""},{"id":"t","x":275,"y":80,"label":"Treasure","icon":"gem","state":"locked","shape":"circle","team":""},{"id":"boss","x":335,"y":150,"label":"Boss","icon":"crown","state":"locked","shape":"diamond","team":""}],"legend":[{"id":"fight","icon":"sword","label":"Fight","shape":"circle"},{"id":"elite","icon":"skull","label":"Elite","shape":"diamond"},{"id":"rest","icon":"fire","label":"Rest","shape":"circle"},{"id":"shop","icon":"coin","label":"Shop","shape":"circle"},{"id":"event","icon":"flag","label":"Event","shape":"circle"},{"id":"treasure","icon":"gem","label":"Treasure","shape":"circle"},{"id":"boss","icon":"crown","label":"Boss","shape":"diamond"}]}'>
<meta name="fixture:empty" content='{"title":"The Sunken Road","floor":"","view":"0 0 360 240","zoom":1,"panX":0,"panY":0,"selected":"","links":[],"nodes":[],"legend":[]}'>

<!--
  The run's map over the act's art, with no board panel: a heading with
  the floor, the rooms and paths drawn by the core NodeGraph (restyled
  only through its tokens), and a legend of room kinds that draws each
  kind's mark as its rooms do. A room is a disc, or a diamond for an elite
  or the boss, its kind in its icon. A visited room is ringed gold, the
  room the player stands in is lit gold, a reachable room is ringed
  white, and the path taken is solid gold over the faint paths ahead. The
  game lays out the rooms, says which are reachable and applies the
  choice; the map reports NodeGraph's pick (a room's key) and its zoom,
  pinch and pan.
-->
<section id="CardRunMap" part="CardRunMap.frame" class="bare" data-layout="shell" aria-label={title || t("card.map.name")}>
  <header part="CardRunMap.band" class="band" data-layout="stack" data-gap="tight"><h2 part="CardRunMap.title" class="title">{title}</h2>{#if floor}<span part="CardRunMap.floor" class="floor">{floor}</span>{/if}</header>
  <div part="CardRunMap.board" class="board" data-layout="row" data-fill>
    <NodeGraph nodes={nodes} links={links} view={view} zoom={zoom} panX={panX} panY={panY} selected={selected} name={title || t("card.map.name")} />
  </div>
  {#if legend.length}<div part="CardRunMap.legend" class="legend" data-layout="cluster" data-gap="loose" role="list" aria-label={t("card.map.legend")}>
    {#each legend as l (l.id)}
      <div part="CardRunMap.row" class="key" data-layout="row" data-gap="tight" role="listitem"><div class="mark" data-shape={l.shape == 'diamond' ? 'diamond' : 'circle'} data-layout="center"><og-icon class="legend-icon" icon={l.icon}></og-icon></div><span class="legend-label">{l.label}</span></div>
    {/each}
  </div>{/if}
</section>

<style>
/* NodeGraph is restyled only through what it reads: the base tokens its
   own defaults come from, set on the board it inherits from (no ground, rooms of the
   map's fill and rim, the path taken and the room the player stands in
   gold, a visited room ringed gold), and the roles it leaves to its
   container (the rooms' shadow and rim width, the paths' width and dash,
   a chosen or hovered room ringed white). */
:host { inline-size: stretch; block-size: stretch; min-block-size: 0; container-type: inline-size; color: var(--hud-ink);
  --cell-shadow: var(--map-room-shadow); --cell-rim-width: var(--map-room-rim-width);
  --cell-selected-rim: var(--map-room-open); --cell-hover-shadow: var(--map-room-open-glow); --highlight: var(--map-room-open);
  --link-width: var(--map-path-width); --link-dash: var(--map-path-dash); }
.band { width: stretch; flex: none; align-items: center; padding-block: var(--space-sm) var(--space-xs); }
.title { margin: 0; max-width: stretch; text-align: center; font-size: var(--map-run-title-size); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); text-shadow: var(--hud-shadow); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.floor { padding: calc(3 * var(--font-step-px)) var(--space-sm); font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); white-space: nowrap; border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.board { width: stretch; justify-content: center; overflow: hidden;
  --data-graph-fill: none; --hud-plate-rim: none; --data-well: var(--map-room-fill); --data-guide: var(--map-room-rim); --success: var(--map-check); --hud-accent: var(--map-path-taken); --data-grip-glow: var(--map-room-current-glow);
  --graph-node: var(--map-run-room); --graph-icon: var(--map-run-room-icon); --graph-caption: var(--map-run-caption-size); }
.legend { flex: none; align-self: center; justify-content: center; margin-block-end: var(--space-sm); padding: var(--space-xs) var(--space-md); font-size: var(--hud-size-xs); color: var(--hud-ink); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.key { flex: none; align-items: center; white-space: nowrap; }
/* Each legend mark is a small room: the same shape, the same icon. */
.mark { width: var(--map-legend-room); height: var(--map-legend-room); flex: none; justify-content: center; border-radius: var(--map-round); background: var(--map-room-fill); box-shadow: var(--map-legend-room-rim); }
.mark[data-shape=diamond] { border-radius: var(--map-diamond-round); rotate: 45deg; }
.mark[data-shape=diamond] .legend-icon { rotate: -45deg; }
.legend-icon { width: 60%; height: 60%; color: var(--map-room-ink); }
/* A room, its icon and its caption follow the text size up to a share of
   the graph's width and height (NodeGraph is their container), so a phone's map at a large text size keeps its rooms
   apart. */
/* A narrow map (a phone's) keeps its height for the rooms: the legend's
   keys pack closer, on as few rows as fit. */
@container (width < 560px) {
  .band { padding-block: var(--space-xs) 0; }
  .legend { column-gap: var(--space-sm); row-gap: var(--space-xs); padding: var(--space-xs) var(--space-sm); margin-block-end: var(--space-xs); }
  .key { gap: var(--space-xs); }
}
</style>
