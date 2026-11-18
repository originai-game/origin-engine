<meta name="component" content="Telegraph">
<meta name="parts" content="Telegraph.area Telegraph.wash Telegraph.sweep Telegraph.rim">
<meta name="prop:entity" content="number" data-doc="Its caster, by its bits (0: use at).">
<meta name="prop:at" content="string" data-default="0 0 0" data-doc="Its origin on the ground without an entity (x y z).">
<meta name="prop:shape" content="string" data-doc="Its area: circle (empty), cone, line or ring.">
<meta name="prop:radius" content="number" data-default="1" data-doc="A circle's or ring's radius, a cone's or line's length, in world units.">
<meta name="prop:inner" content="number" data-default="0.45" data-doc="A ring's safe inner radius as a fraction of radius (0 to 1).">
<meta name="prop:angle" content="number" data-default="70" data-doc="A cone's width in degrees.">
<meta name="prop:width" content="number" data-default="0.2" data-doc="A line's width as a fraction of its length (0 to 1).">
<meta name="prop:direction" content="number" data-doc="Which way a cone or line faces, degrees clockwise from +z seen from above.">
<meta name="prop:progress" content="number" data-doc="The wind-up 0 to 1 (the game's, read every tick): the fill sweeps out from the caster with a pale leading edge and reaches the rim as the hit lands.">
<meta name="prop:friendly" content="bool" data-doc="An ally's (a heal, a buff zone): cyan, without the hatch; false is an enemy's, red and hatched.">
<meta name="prop:hit" content="bool" data-doc="The hit landed: the area flashes 120 ms, then fades over 200 ms. The game removes it after.">
<meta name="fixture:default" content='{"entity":0,"at":"0 0 0","shape":"","radius":4,"inner":0.45,"angle":70,"width":0.2,"direction":0,"progress":0.62,"friendly":false,"hit":false}'>
<meta name="fixture:cone" content='{"shape":"cone","radius":6,"progress":0.55}'>
<meta name="fixture:line" content='{"shape":"line","radius":7,"progress":0.8}'>
<meta name="fixture:ring" content='{"shape":"ring","radius":4,"progress":0.4}'>
<meta name="fixture:impact" content='{"progress":1,"hit":true}'>
<meta name="fixture:ally" content='{"progress":0.5,"friendly":true}'>

<!--
  An attack's area on the ground before it lands: a rim, a faint hatched
  wash and a fill sweeping out from the caster with a pale leading edge.
  Read only; the game owns the timing and the hit.
-->
<ground id="Telegraph" danger entity={entity || null} at={at} size={(radius * 2) + " " + (radius * 2)} rotation={"0 " + direction + " 0"}>
  <div id="Area" part="Telegraph.area" data-shape={shape == '' ? 'circle' : shape} data-friendly={friendly} data-hit={hit} style:--p={progress <= 0 ? 0 : (progress >= 1 ? 1 : progress)} style:--a={angle} style:--w={width} style:--inner={inner}>
    <div part="Telegraph.wash" class="wash"></div>
    <div part="Telegraph.sweep" class="sweep"></div>
    <div part="Telegraph.rim" class="rim"></div>
    {#if shape == 'cone'}<div class="side"></div><div class="side end"></div>{/if}
  </div>
</ground>

<style>
/* The area in a 200 px box the ground lays over radius × 2 world units:
   the caster at its centre, a cone or a line facing up the box. */
#Telegraph { inline-size: var(--measure-xs); block-size: var(--measure-xs); }
#Area { position: relative; inline-size: var(--measure-xs); block-size: var(--measure-xs); filter: var(--world-telegraph-glow); }
#Area[data-friendly=true] { filter: var(--world-telegraph-ally-glow); }
.wash, .sweep, .rim { position: absolute; box-sizing: border-box; border-radius: var(--world-round); }
/* The wash: faint, hatched for an enemy. */
.wash { inset: 0; background: var(--world-telegraph-wash-fill); }
[data-friendly=true] .wash { background: var(--world-telegraph-ally-wash-fill); }
/* The fill, from the caster out to the rim; its edge pale. */
.sweep { left: calc(50% - var(--p) * 50%); top: calc(50% - var(--p) * 50%); inline-size: calc(var(--p) * 100%); block-size: calc(var(--p) * 100%); border: var(--world-telegraph-lead-border); background: var(--world-telegraph-sweep-fill); }
[data-friendly=true] .sweep { border: var(--world-telegraph-ally-lead-border); background: var(--world-telegraph-ally-sweep-fill); }
.rim { inset: 0; border: var(--world-telegraph-rim-border); }
[data-friendly=true] .rim { border: var(--world-telegraph-ally-rim-border); }
/* A cone: the circle cut to its angle, the sides drawn. */
[data-shape=cone] .wash, [data-shape=cone] .sweep, [data-shape=cone] .rim { mask-image: conic-gradient(from calc(var(--a) * -0.5deg), #000 0 calc(var(--a) * 1deg), transparent 0); }
.side { position: absolute; left: calc(50% - var(--stroke-thick) / 2); top: 0; inline-size: var(--stroke-thick); block-size: 50%; background: var(--world-telegraph); transform-origin: 50% 100%; rotate: calc(var(--a) * -0.5deg); }
[data-friendly=true] .side { background: var(--world-telegraph-ally); }
.side.end { rotate: calc(var(--a) * 0.5deg); }
/* A line: a strip from the caster up the box, the fill growing along it. */
[data-shape=line] .wash, [data-shape=line] .rim { left: calc(50% - var(--w) * 50%); right: auto; top: 0; bottom: 50%; inline-size: calc(var(--w) * 100%); border-radius: 0; }
[data-shape=line] .sweep { left: calc(50% - var(--w) * 50%); top: calc(50% - var(--p) * 50%); inline-size: calc(var(--w) * 100%); block-size: calc(var(--p) * 50%); border-radius: 0; border-inline-width: 0; border-block-end-width: 0; }
/* A ring: the safe middle cut out, the fill growing from it outward. */
#Area { --reach: calc(var(--inner) + (1 - var(--inner)) * var(--p)); }
[data-shape=ring] .wash, [data-shape=ring] .rim { mask-image: radial-gradient(closest-side, transparent calc(var(--inner) * 100%), #000 calc(var(--inner) * 100%)); }
[data-shape=ring] .sweep { left: calc(50% - var(--reach) * 50%); top: calc(50% - var(--reach) * 50%); inline-size: calc(var(--reach) * 100%); block-size: calc(var(--reach) * 100%); mask-image: radial-gradient(closest-side, transparent calc(var(--inner) / var(--reach) * 100%), #000 calc(var(--inner) / var(--reach) * 100%)); }
/* The hit: a flash, then the fade. */
#Area[data-hit=true] { filter: var(--world-telegraph-hit-glow); animation: var(--world-impact-motion); }
[data-hit=true] .sweep { background: var(--world-telegraph-hit-fill); }
[data-hit=true] .rim { border: var(--world-telegraph-hit-border); }
[data-hit=true] .side { background: var(--hud-ink); }
@media (prefers-reduced-motion: reduce) { #Area[data-hit=true] { animation: none; } }
</style>
