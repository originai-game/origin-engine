<meta name="component" content="NodeGraph">
<meta name="strings" content="./node-graph.en.ogt">
<meta name="parts" content="NodeGraph.well NodeGraph.link NodeGraph.node NodeGraph.icon NodeGraph.label NodeGraph.cell NodeGraph.badge">
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
        <path class="link" part="NodeGraph.link" data-active={link.active} class:active={link.active} d={link.d}></path>
      {/each}
    </svg>
    {#each nodes as node (node.id)}
      <button class="node" part="NodeGraph.node NodeGraph.cell" data-shape={node.shape == 'circle' ? 'round' : node.shape} data-active={node.state == 'active'} data-done={node.state == 'done'} data-selected={node.id == selected} class:locked={node.state == 'locked'} class:active={node.state == 'active'} class:done={node.state == 'done'} class:current={node.state == 'current'} class:square={node.shape == 'square'} class:hex={node.shape == 'hex'} class:diamond={node.shape == 'diamond'} class:ally={node.team == 'ally'} class:enemy={node.team == 'enemy'} class:selected={node.id == selected} aria-label={node.label} aria-valuetext={node.state} aria-disabled={node.state == 'locked'} style:--x={node.x} style:--y={node.y} onclick={pick}>
        <og-icon class="icon" part="NodeGraph.icon" icon={node.icon}></og-icon>
        <span class="caption" part="NodeGraph.label NodeGraph.badge">{node.label}</span>
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
:host { position: relative; inline-size: stretch; block-size: stretch; min-block-size: var(--size-6xl); overflow: hidden; container-type: size; min-inline-size: 0; --well-fill: var(--data-well); --link-stroke: var(--data-guide); --link-done-stroke: var(--hud-accent); --link-active-stroke: var(--hud-accent); --cell-fill: var(--data-well); --cell-rim: var(--data-guide); --cell-done-rim: var(--success); --cell-active-rim: var(--hud-accent); --cell-active-shadow: var(--data-grip-glow); --badge-fill: var(--hud-tag-fill); }
/* The well: the HUD's smoked glass under a faint dot grid. */
#NodeGraph { background: var(--data-graph-fill); box-shadow: var(--hud-plate-rim); }
#Canvas { --fit: min((100cqw - var(--size-lg) * 2) / var(--vw), (100cqh - var(--size-lg) * 2) / var(--vh)); position: absolute; inset: 0; margin: auto; width: calc(var(--vw) * var(--fit)); height: calc(var(--vh) * var(--fit)); translate: var(--pan-x) var(--pan-y); scale: var(--zoom); transform-origin: 50% 50%; }
#Links { position: absolute; inset: 0; inline-size: 100%; block-size: 100%; }
[part="NodeGraph.node NodeGraph.cell"] { position: absolute; top: calc((var(--y) - var(--vy)) / var(--vh) * 100% - var(--size-lg) / 2); left: calc((var(--x) - var(--vx)) / var(--vw) * 100% - var(--size-lg) / 2); width: var(--size-lg); height: var(--size-lg); padding: 0; }
/* A diamond is the node turned, so a pack's plate turns with it. Its icon
   turns back in place, and its caption turns back about the node's middle,
   so it stands under the node as every caption does, not beside it over a
   neighbour's path. */
.node.diamond { --cell-radius: var(--radius-field); rotate: 45deg; }
.node.diamond .icon { rotate: -45deg; }
.node.diamond .caption { rotate: -45deg; transform-origin: 0 calc(var(--size-lg) / -2 - var(--space-xs)); }
.node.current { --cell-rim: var(--hud-accent); --cell-shadow: var(--data-grip-glow); }
/* A locked node is disabled, so its plate takes the disabled dress; its
   icon dims and its caption takes the muted ink. Opacity on the whole node
   would also thin its caption's chip, and a path would show through it. */
.node.locked { --badge-ink: var(--ink-muted); }
.node.ally { --cell-fill: var(--info); }
.node.enemy { --cell-fill: var(--danger); }
[part="NodeGraph.icon"] { width: var(--size-sm); height: var(--size-sm); }
/* A caption is centred under its node (a transform, so a diamond's turn
   back takes it with it) on a chip of the well, so a path runs under it. */
[part~="NodeGraph.label"] { position: absolute; top: calc(var(--size-lg) + var(--space-xs)); left: 50%; transform: translateX(-50%); white-space: nowrap; font-size: var(--hud-tag-size); font-weight: var(--hud-label-weight); font-stretch: var(--hud-figure-stretch); color: var(--hud-ink); }
.node.locked [part~="NodeGraph.label"] { color: var(--hud-ink-dim); }
</style>
