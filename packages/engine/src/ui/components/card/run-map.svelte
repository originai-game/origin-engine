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
<meta name="prop:legend" content="list" data-doc="The legend, keyed rows of { id, icon, label }, one per room kind; empty hides it.">
<meta name="fixture:default" content='{"title":"The Sunken Road","floor":"Floor 3 of 8","view":"0 0 360 240","zoom":1,"panX":0,"panY":0,"selected":"e2","links":[{"id":"l1","d":"M30 120 L 115 60","active":true},{"id":"l2","d":"M30 120 L 115 180","active":false},{"id":"l3","d":"M115 60 L 200 35","active":false},{"id":"l4","d":"M115 60 L 200 125","active":false},{"id":"l5","d":"M115 180 L 200 125","active":false},{"id":"l6","d":"M200 35 L 275 80","active":false},{"id":"l7","d":"M200 125 L 275 80","active":false},{"id":"l8","d":"M275 80 L 335 150","active":false}],"nodes":[{"id":"start","x":30,"y":120,"label":"Fight","icon":"sword","state":"done","shape":"circle","team":""},{"id":"here","x":115,"y":60,"label":"Event","icon":"flag","state":"current","shape":"hex","team":""},{"id":"alt","x":115,"y":180,"label":"Shop","icon":"coin","state":"locked","shape":"square","team":""},{"id":"e1","x":200,"y":35,"label":"Elite","icon":"skull","state":"available","shape":"diamond","team":""},{"id":"e2","x":200,"y":125,"label":"Rest","icon":"fire","state":"available","shape":"square","team":""},{"id":"t","x":275,"y":80,"label":"Treasure","icon":"gem","state":"locked","shape":"hex","team":""},{"id":"boss","x":335,"y":150,"label":"Boss","icon":"crown","state":"locked","shape":"diamond","team":""}],"legend":[{"id":"fight","icon":"sword","label":"Fight"},{"id":"elite","icon":"skull","label":"Elite"},{"id":"rest","icon":"fire","label":"Rest"},{"id":"shop","icon":"coin","label":"Shop"},{"id":"event","icon":"flag","label":"Event"},{"id":"boss","icon":"crown","label":"Boss"}]}'>
<meta name="fixture:empty" content='{"title":"The Sunken Road","floor":"","view":"0 0 360 240","zoom":1,"panX":0,"panY":0,"selected":"","links":[],"nodes":[],"legend":[]}'>

<!--
  The run's map: a heading with the floor, the rooms and paths drawn by
  the core NodeGraph, and a legend of room kinds. The game lays out the
  rooms, says which are reachable and applies the choice; the map reports
  NodeGraph's pick (a room's key) and its zoom, pinch and pan.
-->
<section id="CardRunMap" part="CardRunMap.frame" aria-label={title || t("card.map.name")}>
  <header part="CardRunMap.band"><h2 part="CardRunMap.title">{title}</h2><span part="CardRunMap.floor" hidden={!floor}>{floor}</span></header>
  <div part="CardRunMap.board" class="board">
    <NodeGraph nodes={nodes} links={links} view={view} zoom={zoom} panX={panX} panY={panY} selected={selected} name={title || t("card.map.name")} />
  </div>
  <div part="CardRunMap.legend" role="list" hidden={!legend.length} aria-label={t("card.map.legend")}>
    {#each legend as l (l.id)}
      <div part="CardRunMap.row" role="listitem"><og-icon class="legend-icon" icon={l.icon}></og-icon><span class="legend-label">{l.label}</span></div>
    {/each}
  </div>
</section>

<style>
[part="CardRunMap.frame"][hidden], [part="CardRunMap.frame"] [hidden] { display: none; }
[part="CardRunMap.frame"] { display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-sm); }
[part="CardRunMap.band"] { display: flex; flex-wrap: wrap; width: stretch; align-items: baseline; justify-content: center; gap: var(--space-xs); flex: none; }
[part="CardRunMap.title"] { margin: 0; text-align: center; }
[part="CardRunMap.floor"] { white-space: nowrap; font-variant-numeric: tabular-nums; }
.board { display: flex; width: stretch; flex: 1; min-height: 0; align-items: center; justify-content: center; overflow: hidden; }
[part="CardRunMap.legend"] { display: flex; flex-wrap: wrap; width: stretch; flex: none; justify-content: center; gap: var(--space-md); }
[part="CardRunMap.row"] { display: flex; align-items: center; gap: var(--space-xs); flex: none; }
.legend-icon { width: var(--size-xs); height: var(--size-xs); }
</style>
