<meta name="component" content="NodeGraph">
<meta name="strings" content="./node-graph.en.ogt">
<meta name="parts" content="NodeGraph.well NodeGraph.link NodeGraph.node NodeGraph.icon NodeGraph.label NodeGraph.cell">
<meta name="prop:nodes" content="list" data-doc="The nodes: each { id, x, y, label, icon, state, shape, team }, x and y in graph units (CSS px at zoom 1), state available, locked, active, done or current (a ring and a mark as well as a colour), shape circle, square, hex or diamond, team ally, enemy or empty (pair a side with a shape so sides are not told by colour alone).">
<meta name="prop:links" content="list" data-doc="The links under the nodes: each { id, d, active }, d its SVG path data in graph units (the game draws a curve or a line), active a path taken.">
<meta name="prop:view" content="string" data-default="0 0 600 400" data-doc="The graph's extent in graph units, as an SVG viewBox (0 0 600 400): it fits the well as xMidYMid meet, scaled to the most that holds it and centred.">
<meta name="prop:zoom" content="number" data-default="1" data-doc="Its zoom: the game keeps it, from the wheel and pinch events, within its limits.">
<meta name="prop:panX" content="number" data-doc="How far it is panned across, CSS px (from its pointer events).">
<meta name="prop:panY" content="number" data-doc="How far it is panned down.">
<meta name="prop:selected" content="string" data-doc="The selected node's id: ringed in the focus ink.">
<meta name="prop:name" content="string" data-doc="What a screen reader calls it; empty is Graph.">
<meta name="action:pick" content="Pick a node">
<meta name="action:zoom" content="Zoom (wheel)">
<meta name="action:pinch" content="Zoom (pinch)">
<meta name="action:pan" content="Pan">
<meta name="fixture:default" content='{"view":"0 0 360 240","zoom":1,"panX":0,"panY":0,"selected":"cleave","name":"","links":[{"id":"a","d":"M40 120 C 100 120, 120 60, 180 60","active":true},{"id":"b","d":"M40 120 C 100 120, 120 180, 180 180","active":false},{"id":"c","d":"M180 60 L 320 120","active":false}],"nodes":[{"id":"root","x":40,"y":120,"label":"Strength","icon":"sword","state":"done","shape":"circle","team":""},{"id":"cleave","x":180,"y":60,"label":"Cleave","icon":"bolt","state":"available","shape":"hex","team":""},{"id":"guard","x":180,"y":180,"label":"Guard","icon":"shield","state":"active","shape":"square","team":"ally"},{"id":"rage","x":320,"y":120,"label":"Rage","icon":"fire","state":"locked","shape":"diamond","team":"enemy"}]}'>

<div id="NodeGraph" part="NodeGraph.well" role="group" aria-label={name || t("component.node-graph.name")} aria-valuetext={t("component.node-graph.zoom", { zoom: zoom })} onwheel={zoom} onpinch={pinch} onpointermove={pan}>
  <div id="Canvas" style:--vx={numbers(view)[0]} style:--vy={numbers(view)[1]} style:--vw={numbers(view)[2]} style:--vh={numbers(view)[3]} style:--zoom={zoom} style:--pan-x={panX + "px"} style:--pan-y={panY + "px"}>
    <svg id="Links" viewBox={view}>
      {#each links as link (link.id)}
        <path class="link" part="NodeGraph.link" class:active={link.active} d={link.d}></path>
      {/each}
    </svg>
    {#each nodes as node (node.id)}
      <button class="node" part="NodeGraph.node NodeGraph.cell" class:locked={node.state == 'locked'} class:active={node.state == 'active'} class:done={node.state == 'done'} class:current={node.state == 'current'} class:square={node.shape == 'square'} class:hex={node.shape == 'hex'} class:diamond={node.shape == 'diamond'} class:ally={node.team == 'ally'} class:enemy={node.team == 'enemy'} class:selected={node.id == selected} aria-label={node.label} aria-valuetext={node.state} aria-disabled={node.state == 'locked'} style:--x={node.x} style:--y={node.y} onclick={pick}>
        <og-icon class="icon" part="NodeGraph.icon" icon={node.icon}></og-icon>
        <span class="caption" part="NodeGraph.label">{node.label}</span>
      </button>
    {/each}
  </div>
</div>

<style>
/* It sizes as a box does: across its container and down it, never under a
   graph's worth of height. The graph fits in it as an SVG viewBox does under
   xMidYMid meet: the view's units are scaled by --fit, the most that holds
   the whole view (and a node's half beyond each edge) in the well, and it
   stands in the middle. The nodes keep their size; only where they stand,
   and the links between them, scale. It zooms about that middle. */
[part="NodeGraph.well"] { width: stretch; height: stretch; min-height: var(--size-6xl); overflow: hidden; container-type: size; }
#Canvas { --fit: min((100cqw - var(--size-lg) * 2) / var(--vw), (100cqh - var(--size-lg) * 2) / var(--vh)); position: absolute; inset: 0; margin: auto; width: calc(var(--vw) * var(--fit)); height: calc(var(--vh) * var(--fit)); translate: var(--pan-x) var(--pan-y); scale: var(--zoom); transform-origin: 50% 50%; }
#Links { position: absolute; inset: 0; width: stretch; height: stretch; }
[part="NodeGraph.link"] { fill: none; stroke: var(--line-strong); stroke-width: 3; stroke-dasharray: 6 6; }
.link.active { stroke: var(--primary); stroke-dasharray: none; }
[part="NodeGraph.node NodeGraph.cell"] { position: absolute; top: calc((var(--y) - var(--vy)) / var(--vh) * 100% - var(--size-lg) / 2); left: calc((var(--x) - var(--vx)) / var(--vw) * 100% - var(--size-lg) / 2); width: var(--size-lg); height: var(--size-lg); padding: 0; border-radius: var(--radius-capsule); border: var(--stroke-thick) solid var(--line-strong); background: var(--panel-raised); }
.node.square { border-radius: var(--radius-card); }
/* A hexagon as its corners, not a clip: a clip-path would cut its caption too. */
.node.hex { corner-shape: bevel; border-radius: 25% / 50%; }
/* A diamond is the node turned, so a pack's plate turns with it. Its icon
   turns back in place, and its caption turns back about the node's middle,
   so it stands under the node as every caption does, not beside it over a
   neighbour's path. */
.node.diamond { rotate: 45deg; border-radius: var(--radius-field); }
.node.diamond .icon { rotate: -45deg; }
.node.diamond .caption { rotate: -45deg; transform-origin: 0 calc(var(--size-lg) / -2 - var(--space-xs)); }
.node.done { border-color: var(--success); }
.node.active { border-color: var(--primary); box-shadow: 0 0 var(--space-md) var(--primary); }
.node.current { border-color: var(--highlight); }
/* A locked node is disabled, so its plate takes the disabled dress; its
   icon dims and its caption takes the muted ink. Opacity on the whole node
   would also thin its caption's chip, and a path would show through it. */
.node.locked .icon { opacity: var(--opacity-dim); }
.node.locked .caption { color: var(--ink-muted); }
.node.ally { background: var(--info); }
.node.enemy { background: var(--danger); }
.node.selected { outline: var(--state-focus-ring-width) solid var(--focus); outline-offset: var(--state-focus-ring-offset); }
[part="NodeGraph.icon"] { width: var(--size-sm); height: var(--size-sm); }
/* A caption is centred under its node (a transform, so a diamond's turn
   back takes it with it) on a chip of the well, so a path runs under it. */
[part="NodeGraph.label"] { position: absolute; top: calc(var(--size-lg) + var(--space-xs)); left: 50%; transform: translateX(-50%); padding: 0 var(--space-xxs); border-radius: var(--radius-sm); background: var(--well-fill); font: var(--text-caption); white-space: nowrap; text-shadow: 0 var(--stroke-hairline) var(--stroke-regular) var(--scene-shadow); }
</style>
