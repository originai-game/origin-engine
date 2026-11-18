<meta name="component" content="RacingTrackMap">
<meta name="strings" content="./racing.en.ogt">
<meta name="parts" content="RacingTrackMap.frame RacingTrackMap.map RacingTrackMap.road RacingTrackMap.line RacingTrackMap.driven RacingTrackMap.start RacingTrackMap.car RacingTrackMap.self RacingTrackMap.name RacingTrackMap.length">
<meta name="prop:path" content="string" data-doc="The track's racing line as an SVG path (d) in the map's 260 by 220 box; empty draws no road.">
<meta name="prop:driven" content="string" data-doc="The part of the lap the player has driven, as an SVG path in the same box, lit in the HUD's accent; empty draws none.">
<meta name="prop:startX" content="number" data-doc="The start line's place in the box, across.">
<meta name="prop:startY" content="number" data-doc="The start line's place in the box, down.">
<meta name="prop:startAngle" content="number" data-doc="The start line's turn in degrees.">
<meta name="prop:cars" content="list" data-doc="The cars on the map, keyed rows of { id, x, y, heading, color, player, name }: x and y in the map's box; color the car's colour as a CSS colour (empty: neutral); player marks the player's car, drawn as an arrow turned to heading (degrees clockwise from up) over the others' dots; name for a reader.">
<meta name="prop:name" content="string" data-doc="The track's name as the game words it, under the map; empty hides it.">
<meta name="prop:length" content="string" data-doc="The track's length as the game words it (4.8 km), on a tag beside the name; empty hides it.">
<meta name="fixture:default" content='{"path":"M40 170 C10 120 30 50 90 40 C140 30 150 80 190 70 C240 58 250 120 215 150 C185 176 150 140 120 165 C95 188 60 200 40 170 Z","driven":"M40 170 C10 120 30 50 90 40 C140 30 150 80 190 70","startX":40,"startY":170,"startAngle":-60,"name":"Costa Serpente","length":"4.8 km","cars":[{"id":"okafor","x":212,"y":88,"heading":0,"color":"#ffd24a","player":false,"name":"M. Okafor"},{"id":"lindqvist","x":200,"y":72,"heading":0,"color":"#4f7bff","player":false,"name":"S. Lindqvist"},{"id":"castellanos","x":150,"y":150,"heading":0,"color":"#55e39a","player":false,"name":"R. Castellanos"},{"id":"you","x":190,"y":70,"heading":100,"color":"","player":true,"name":"You"}]}'>
<meta name="fixture:empty" content='{"path":"","driven":"","startX":0,"startY":0,"startAngle":0,"name":"","length":"","cars":[]}'>
<meta name="fixture:start" content='{"path":"M40 170 C10 120 30 50 90 40 C140 30 150 80 190 70 C240 58 250 120 215 150 C185 176 150 140 120 165 C95 188 60 200 40 170 Z","driven":"","startX":40,"startY":170,"startAngle":-60,"name":"Costa Serpente","length":"","cars":[{"id":"you","x":42,"y":176,"heading":-20,"color":"","player":true,"name":"You"},{"id":"okafor","x":46,"y":184,"heading":0,"color":"#ffd24a","player":false,"name":"M. Okafor"}]}'>
<meta name="fixture:overflow" content='{"path":"M30 110 L230 110","driven":"M30 110 L130 110","startX":30,"startY":110,"startAngle":90,"name":"Autodromo Internazionale del Circuito delle Colline di Costa Serpente","length":"20.832 km Nordschleife","cars":[{"id":"you","x":130,"y":110,"heading":90,"color":"","player":true,"name":"You"}]}'>

<!--
  The track map at the top right: the circuit as a light line over a dark
  band, the lap driven so far lit in the HUD's accent, the start line, the
  other cars as dots in their colours and the player's car as an arrow
  over them; the track's name and length under it. The game gives the
  shape and every position in the map's box; read only.
-->
<section id="RacingTrackMap" part="RacingTrackMap.frame" class="hud bare" data-layout="stack" role="group" aria-label={name != '' ? name : t("racing.map.name")}>
  <svg part="RacingTrackMap.map" class="map" viewBox="0 0 260 220" role="image" aria-label={t("racing.map.name")}>
    {#if path != ''}<path part="RacingTrackMap.road" class="road" d={path} fill="none" stroke-width="16" stroke-linejoin="round"></path>
    <path part="RacingTrackMap.line" class="line" d={path} fill="none" stroke-width="5" stroke-linejoin="round"></path>{/if}
    {#if driven != ''}<path part="RacingTrackMap.driven" class="driven" d={driven} fill="none" stroke-width="5" stroke-linecap="round"></path>{/if}
    {#if path != ''}<rect part="RacingTrackMap.start" class="start" x="-8" y="-2.5" width="16" height="5" transform={"translate(" + startX + " " + startY + ") rotate(" + startAngle + ")"}></rect>{/if}
    {#each cars as c (c.id)}
      {#if c.player}<path part="RacingTrackMap.self" class="self" d="M0 -10 L8 8 L0 4 L-8 8 Z" stroke-width="2" stroke-linejoin="round" transform={"translate(" + c.x + " " + c.y + ") rotate(" + c.heading + ")"} aria-label={c.name}></path>{:else}<circle part="RacingTrackMap.car" class="car" cx={c.x} cy={c.y} r="7" stroke-width="2" style:--hud-car={c.color != '' ? c.color : null} aria-label={c.name}></circle>{/if}
    {/each}
  </svg>
  {#if name != '' || length != ''}<div id="Under" data-layout="row">
    {#if name != ''}<span part="RacingTrackMap.name" class="name">{name}</span>{/if}
    {#if length != ''}<span part="RacingTrackMap.length" class="length" aria-label={t("racing.map.length") + " " + length}>{length}</span>{/if}
  </div>{/if}
</section>

<style>
:host { --map: calc(220 * var(--font-step-px)); inline-size: var(--map); max-inline-size: stretch; row-gap: calc(4 * var(--font-step-px)); color: var(--hud-ink); }
/* The map's box is sized from its width in the viewBox's proportion (og
   reads an auto height as the viewBox's height, not its proportion). */
.map { inline-size: var(--map); max-inline-size: 100%; block-size: calc(var(--map) * 220 / 260); filter: var(--hud-lift); }
.road { stroke: var(--hud-track-well); }
.line { stroke: var(--hud-track-line); }
.driven { stroke: var(--hud-accent); }
.start { fill: var(--hud-ink); }
.car { fill: var(--hud-car); stroke: var(--hud-track-well); }
.self { fill: var(--hud-ink); stroke: var(--hud-track-well); }
#Under { align-items: center; justify-content: space-between; column-gap: var(--space-sm); min-inline-size: 0; }
.name { flex: 0 1 auto; min-inline-size: 0; font-size: var(--hud-size-name); font-weight: var(--hud-label-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
.length { flex: none; max-inline-size: 50%; margin-inline-start: auto; padding: 0 calc(6 * var(--font-step-px)); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
/* On a phone the map is smaller and its name line goes (it has no room
   under a map that small). */
@media (width < 600px) {
  :host { --map: var(--hud-map-compact); }
  #Under { display: none; }
}
</style>
