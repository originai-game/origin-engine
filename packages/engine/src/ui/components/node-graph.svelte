<meta name="component" content="NodeGraph">
<meta name="strings" content="./node-graph.en.ogt">
<meta name="prop:nodes" content="list" data-doc="The nodes: each { id, x, y, label, icon, state, shape, team }, x and y in graph units (CSS px at zoom 1), state available, locked, active, done or current (a ring and a mark as well as a colour), shape circle, square, hex or diamond, team ally, enemy or empty (pair a side with a shape so sides are not told by colour alone).">
<meta name="prop:links" content="list" data-doc="The links under the nodes: each { id, d, active }, d its SVG path data in graph units (the game draws a curve or a line), active a path taken.">
<meta name="prop:view" content="string" data-doc="The graph's extent in graph units, as an SVG viewBox (0 0 600 400).">
<meta name="prop:zoom" content="number" data-doc="Its zoom: the game keeps it, from the wheel and pinch events, within its limits.">
<meta name="prop:panX" content="number" data-doc="How far it is panned across, CSS px (from its pointer events).">
<meta name="prop:panY" content="number" data-doc="How far it is panned down.">
<meta name="prop:selected" content="string" data-doc="The selected node's id: ringed in the focus ink.">
<meta name="prop:name" content="string" data-doc="What a screen reader calls it; empty is Graph.">
<meta name="action:pick" content="Pick a node">
<meta name="action:zoom" content="Zoom (wheel)">
<meta name="action:pinch" content="Zoom (pinch)">
<meta name="action:pan" content="Pan">
<meta name="fixture:default" content='{"view":"0 0 360 240","zoom":1,"panX":0,"panY":0,"selected":"cleave","name":"","links":[{"id":"a","d":"M40 120 C 100 120, 120 60, 180 60","active":true},{"id":"b","d":"M40 120 C 100 120, 120 180, 180 180","active":false},{"id":"c","d":"M180 60 L 320 120","active":false}],"nodes":[{"id":"root","x":40,"y":120,"label":"Strength","icon":"sword","state":"done","shape":"circle","team":""},{"id":"cleave","x":180,"y":60,"label":"Cleave","icon":"bolt","state":"available","shape":"hex","team":""},{"id":"guard","x":180,"y":180,"label":"Guard","icon":"shield","state":"active","shape":"square","team":"ally"},{"id":"rage","x":320,"y":120,"label":"Rage","icon":"fire","state":"locked","shape":"diamond","team":"enemy"}]}'>

<div id="NodeGraph" role="group" aria-label={name || t("component.node-graph.name")} aria-valuetext={t("component.node-graph.zoom", { zoom: zoom })} onwheel={zoom} onpinch={pinch} onpointermove={pan}>
  <div id="Canvas" style:--zoom={zoom} style:--pan-x={panX} style:--pan-y={panY}>
    <svg id="Links" viewBox={view}>
      {#each links as link (link.id)}
        <path class="link" class:active={link.active} d={link.d}></path>
      {/each}
    </svg>
    {#each nodes as node (node.id)}
      <button class="node" class:locked={node.state == 'locked'} class:active={node.state == 'active'} class:done={node.state == 'done'} class:current={node.state == 'current'} class:square={node.shape == 'square'} class:hex={node.shape == 'hex'} class:diamond={node.shape == 'diamond'} class:ally={node.team == 'ally'} class:enemy={node.team == 'enemy'} class:selected={node.id == selected} aria-label={node.label} aria-valuetext={node.state} aria-disabled={node.state == 'locked'} style:--x={node.x} style:--y={node.y} onclick={pick}>
        <og-icon class="icon" icon={node.icon}></og-icon>
        <span class="caption">{node.label}</span>
      </button>
    {/each}
  </div>
</div>

<style>
#NodeGraph { width: 420px; height: 300px; overflow: hidden; border-radius: 12px; background: var(--well); }
#Canvas { position: absolute; top: 0; inset-inline-start: 0; width: 360px; height: 240px; translate: calc(var(--pan-x) * 1px + 30px) calc(var(--pan-y) * 1px + 30px); scale: var(--zoom); transform-origin: 0% 0%; }
#Links { position: absolute; top: 0; inset-inline-start: 0; width: 360px; height: 240px; }
.link { fill: none; stroke: var(--line-strong); stroke-width: 3; stroke-dasharray: 6 6; }
.link.active { stroke: var(--primary); stroke-dasharray: none; }
.node { position: absolute; top: calc(var(--y) * 1px - 24px); inset-inline-start: calc(var(--x) * 1px - 24px); width: 48px; height: 48px; padding: 0; border-radius: 24px; border: 3px solid var(--line-strong); background: var(--panel-raised); }
.node.square { border-radius: 10px; }
.node.hex { clip-path: polygon(25% 0%, 75% 0%, 100% 50%, 75% 100%, 25% 100%, 0% 50%); border-radius: 0; }
.node.diamond { rotate: 45deg; border-radius: 6px; }
.node.diamond .icon, .node.diamond .caption { rotate: -45deg; }
.node.done { border-color: var(--success); }
.node.active { border-color: var(--primary); box-shadow: 0 0 12px var(--primary); }
.node.current { border-color: var(--highlight); }
.node.locked { opacity: 0.5; }
.node.ally { background: var(--info); }
.node.enemy { background: var(--danger); }
.node.selected { outline: 3px solid var(--focus); outline-offset: 2px; }
.icon { width: 24px; height: 24px; }
.caption { position: absolute; top: 52px; font: var(--text-caption); white-space: nowrap; text-shadow: 0 1px 2px #000000cc; }
</style>
