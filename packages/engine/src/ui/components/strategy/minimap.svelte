<meta name="component" content="StrategyMinimap">
<meta name="strings" content="./strategy.en.ogt">
<meta name="parts" content="StrategyMinimap.frame StrategyMinimap.map StrategyMinimap.dot StrategyMinimap.view StrategyMinimap.ping">
<meta name="prop:map" content="number" data-doc="The whole map's picture (og.load.texture), drawn square; 0 draws the dark well alone.">
<meta name="prop:dots" content="list" data-doc="What the minimap shows, keyed rows of { id, kind, x, y, base, label }: kind own, ally, foe or neutral (a resource, a relic; on data-kind, each its colour, a neutral a diamond); x and y on the map, 0 to 1; base marks a building or a town, drawn larger (data-base); label names it for a reader.">
<meta name="prop:viewX" content="number" data-doc="The camera's view on the map: its left edge, 0 to 1.">
<meta name="prop:viewY" content="number" data-doc="The camera's view: its top edge, 0 to 1.">
<meta name="prop:viewWidth" content="number" data-doc="The camera's view: its width, 0 to 1; 0 hides the frame.">
<meta name="prop:viewHeight" content="number" data-doc="The camera's view: its height, 0 to 1.">
<meta name="prop:pings" content="list" data-doc="Alerts on the map, keyed rows of { id, x, y, label }: a red ring round the place (x and y, 0 to 1); label names it for a reader.">
<meta name="fixture:default" content='{"map":0,"viewX":0.34,"viewY":0.38,"viewWidth":0.34,"viewHeight":0.24,"dots":[{"id":"keep","kind":"own","x":0.22,"y":0.5,"base":true,"label":"Your keep"},{"id":"u1","kind":"own","x":0.44,"y":0.52,"base":false,"label":"Men-at-Arms"},{"id":"u2","kind":"own","x":0.47,"y":0.51,"base":false,"label":"Archers"},{"id":"u3","kind":"own","x":0.5,"y":0.5,"base":false,"label":"Archers"},{"id":"u4","kind":"own","x":0.3,"y":0.36,"base":false,"label":"Villagers"},{"id":"camp","kind":"foe","x":0.82,"y":0.5,"base":true,"label":"Enemy camp"},{"id":"f1","kind":"foe","x":0.7,"y":0.46,"base":false,"label":"Raiders"},{"id":"f2","kind":"foe","x":0.73,"y":0.55,"base":false,"label":"Raiders"},{"id":"gold","kind":"neutral","x":0.2,"y":0.26,"base":false,"label":"Gold mine"}],"pings":[{"id":"p1","x":0.64,"y":0.34,"label":"The east tower is under attack"}]}'>
<meta name="fixture:allies" content='{"map":0,"viewX":0.05,"viewY":0.6,"viewWidth":0.3,"viewHeight":0.22,"dots":[{"id":"keep","kind":"own","x":0.18,"y":0.7,"base":true,"label":"Your keep"},{"id":"ally","kind":"ally","x":0.75,"y":0.25,"base":true,"label":"Ally keep"},{"id":"a1","kind":"ally","x":0.6,"y":0.35,"base":false,"label":"Ally army"},{"id":"relic","kind":"neutral","x":0.5,"y":0.5,"base":false,"label":"Relic"}],"pings":[]}'>
<meta name="fixture:empty" content='{"map":0,"viewX":0,"viewY":0,"viewWidth":0,"viewHeight":0,"dots":[],"pings":[]}'>
<meta name="fixture:overflow" content='{"map":0,"viewX":0.8,"viewY":0.85,"viewWidth":0.4,"viewHeight":0.3,"dots":[{"id":"d1","kind":"own","x":0,"y":0,"base":true,"label":"Corner"},{"id":"d2","kind":"foe","x":1,"y":1,"base":true,"label":"Corner"},{"id":"d3","kind":"ally","x":1,"y":0,"base":false,"label":"Corner"},{"id":"d4","kind":"neutral","x":0,"y":1,"base":false,"label":"Corner"},{"id":"d5","kind":"foe","x":0.5,"y":0.5,"base":false,"label":"Centre"},{"id":"d6","kind":"foe","x":0.51,"y":0.5,"base":false,"label":"Centre"},{"id":"d7","kind":"foe","x":0.52,"y":0.51,"base":false,"label":"Centre"}],"pings":[{"id":"p1","x":0.02,"y":0.98,"label":"A corner raid"},{"id":"p2","x":0.5,"y":0.5,"label":"The centre"},{"id":"p3","x":0.9,"y":0.1,"label":"The far north east"}]}'>

<!--
  The minimap at the bottom left: the whole map square in a lit-edged
  tile with two corners cut, your units and buildings, your allies' and
  your foes' as dots in their colours (a base larger, a resource a gold
  diamond), the camera's view as a white frame and each alert a red ring.
  The game gives every position on the map, 0 to 1; read only.
-->
<section id="StrategyMinimap" part="StrategyMinimap.frame" class="hud bare" role="group" aria-label={t("strategy.minimap.name")}>
  <div id="Scope" role="image" aria-label={t("strategy.minimap.name")}>
    {#if map}<img id="Map" part="StrategyMinimap.map" src={map} alt="">{/if}
    {#each dots as d (d.id)}<div part="StrategyMinimap.dot" class="dot" role="image" data-kind={d.kind} data-base={d.base} style:--x={d.x} style:--y={d.y} aria-label={d.label}></div>{/each}
    {#if viewWidth > 0}<div part="StrategyMinimap.view" id="View" style:--vx={viewX} style:--vy={viewY} style:--vw={viewWidth} style:--vh={viewHeight}></div>{/if}
    {#each pings as p (p.id)}<div part="StrategyMinimap.ping" class="ping" role="image" style:--x={p.x} style:--y={p.y} aria-label={p.label}></div>{/each}
  </div>
</section>

<style>
/* The tile is art: it keeps its size at any text scale. Its fill is its
   lit edge; the map is inset inside it. */
:host { --side: calc(var(--size-5xl) * 1.04); flex: none; inline-size: var(--side); block-size: var(--side); max-inline-size: stretch; padding: calc(2 * var(--font-step-px)); background: var(--hud-edge); clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); filter: var(--hud-lift); }
#Scope { position: relative; inline-size: 100%; block-size: 100%; overflow: hidden; background: var(--hud-radar-well); clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); }
#Map { position: absolute; left: 0; top: 0; inline-size: 100%; block-size: 100%; object-fit: cover; }
.dot { position: absolute; --pip: calc(5 * var(--font-step-px)); inline-size: var(--pip); block-size: var(--pip); left: calc(var(--x) * 100% - var(--pip) / 2); top: calc(var(--y) * 100% - var(--pip) / 2); background: var(--hud-ink); box-shadow: var(--hud-marker-rim); }
.dot[data-base=true] { --pip: calc(9 * var(--font-step-px)); }
.dot[data-kind=own] { background: var(--hud-accent); }
.dot[data-kind=ally] { background: var(--hud-ally); }
.dot[data-kind=foe] { background: var(--hud-foe); }
.dot[data-kind=neutral] { transform: rotate(45deg); background: var(--hud-gold); }
/* The camera's view: a white frame over a dark keyline. */
#View { position: absolute; left: calc(var(--vx) * 100%); top: calc(var(--vy) * 100%); inline-size: calc(var(--vw) * 100%); block-size: calc(var(--vh) * 100%); box-shadow: var(--hud-frustum-rim); }
/* An alert: a red ring with a glow round the place. */
.ping { position: absolute; --ring: calc(18 * var(--font-step-px)); inline-size: var(--ring); block-size: var(--ring); left: calc(var(--x) * 100% - var(--ring) / 2); top: calc(var(--y) * 100% - var(--ring) / 2); border-radius: var(--hud-radar-radius); box-shadow: var(--hud-ping-ring); }
/* On a phone the tile is smaller, in its corner. */
@media (width < 600px) {
  :host { --side: var(--size-xxl); }
  .dot { --pip: calc(4 * var(--font-step-px)); }
  .dot[data-base=true] { --pip: calc(7 * var(--font-step-px)); }
  .ping { --ring: calc(13 * var(--font-step-px)); }
}
</style>
