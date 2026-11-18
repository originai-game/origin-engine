<meta name="component" content="RpgWorldMap">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgWorldMap.frame RpgWorldMap.marker RpgWorldMap.row RpgWorldMap.plate RpgWorldMap.caption">
<meta name="prop:map" content="number" data-doc="The map's texture (og.load.texture); markers sit on it in fractions of its size.">
<meta name="prop:markers" content="list" data-doc="Keyed rows of { id, kind, x, y, label }: kind is quest, waypoint, danger or player; x and y 0 to 1 across and down the map.">
<meta name="prop:selected" content="string" data-doc="The marker selected; empty is none.">
<meta name="prop:zoom" content="number" data-default="1" data-doc="How far the map is zoomed in: 1 (and 0, and when left out) fits it to the box, 2 shows it twice the size. The game keeps it, from the zoom and pinch events, within its own limits.">
<meta name="prop:panX" content="number" data-default="0" data-doc="How far the zoomed map is moved across, CSS px (0, and when left out, pins its start corner). The game keeps it, from grab, pan and release.">
<meta name="prop:panY" content="number" data-default="0" data-doc="How far the zoomed map is moved down, CSS px; 0 when left out.">
<meta name="prop:legend" content="list" data-doc="The legend, keyed rows of { id, kind, label }, one line along the map's foot; empty hides it.">
<meta name="prop:here" content="string" data-doc="A caption pinned to the map's top corner (Current location); empty hides it.">
<meta name="fixture:default" content='{"map":0,"markers":[{"id":"crown","kind":"quest","x":0.48,"y":0.3,"label":"Dawnhold"},{"id":"me","kind":"player","x":0.3,"y":0.55,"label":"You"}],"selected":"crown","zoom":1,"panX":0,"panY":0,"legend":[{"id":"quest","kind":"quest","label":"Quest"},{"id":"danger","kind":"danger","label":"Danger"}],"here":""}'>
<meta name="fixture:places" content='{"map":0,"markers":[{"id":"abbey","kind":"quest","x":0.42,"y":0.22,"label":"Saltmere Abbey"},{"id":"w1","kind":"waypoint","x":0.15,"y":0.4,"label":"Reedford"},{"id":"d1","kind":"danger","x":0.72,"y":0.32,"label":"Drowned Barrows"},{"id":"w2","kind":"waypoint","x":0.66,"y":0.62,"label":"Gullstone"},{"id":"me","kind":"player","x":0.34,"y":0.55,"label":"You"}],"selected":"abbey","zoom":1,"panX":0,"panY":0,"legend":[{"id":"quest","kind":"quest","label":"Quest"},{"id":"waypoint","kind":"waypoint","label":"Waypoint"},{"id":"danger","kind":"danger","label":"Danger"},{"id":"player","kind":"player","label":"You"}],"here":"Brannoch Ford"}'>

<!--
  A map with its markers, a one-line legend and an optional caption. The
  map's art is the screen: no panel lies over it, only each marker's name
  on a small cut plate, the legend and the caption. A marker's kind is its
  shape as well as its colour (a quest a gold diamond, a waypoint a cyan
  ring, danger a red triangle, the player a cyan disc with an arrow); its
  name is white at any kind. The chosen marker is ringed and larger.
  The map is drawn to cover the box it is given, times zoom and moved by
  panX and panY; markers sit at fractions of it, keep their size at any
  zoom and report choose (a marker's key). The player's marker is not a
  button target. The game owns the zoom and the pan: the map reports zoom
  (the wheel, with deltaY), pinch (two touches, with scale), and grab,
  pan and release (a press, its moves and its end, each with x and y in
  the map's box), and draws what the game sets.
-->
<figure id="RpgWorldMap" part="RpgWorldMap.frame" class="bare" data-layout="shell" aria-label={t("rpg.map.name")}>
  <div class="sheet" data-fill onwheel={zoom} onpinch={pinch} onpointerdown={grab} onpointermove={pan} onpointerup={release}>
    <div class="land" style:--zoom={zoom || 1} style:--pan-x={panX + "px"} style:--pan-y={panY + "px"}>
      <img class="terrain" src={map} alt="">
      <div class="markers" role="group" aria-label={t("rpg.map.places")}>
        {#each markers as m (m.id)}
          <button part="RpgWorldMap.marker" class="marker" data-kind={m.kind} disabled={m.kind == 'player'} aria-selected={m.id == selected} aria-label={m.label} style:--x={m.x} style:--y={m.y} onclick={choose}><div class="face" data-layout="cover">
            <div class="ring"></div>
            <div class="pin" data-kind={m.kind}><div class="shape"></div>{#if m.kind == 'player'}<svg class="arrow" viewBox="0 0 16 16"><path d="M8 1l6 13-6-3.5L2 14z"></path></svg>{:else}{#if m.kind == 'quest' || m.kind == 'danger'}<div class="bang"><i class="bar"></i><i class="spot"></i></div>{/if}{/if}</div>
            <span class="name" part="RpgWorldMap.caption">{m.label}</span>
          </div></button>
        {/each}
      </div>
    </div>
    {#if here}<div class="here" part="RpgWorldMap.plate" data-layout="row" data-gap="tight"><div class="here-mark"></div><span class="here-text">{here}</span></div>{/if}
  </div>
  {#if legend.length}<div class="legend" part="RpgWorldMap.plate" data-layout="cluster" data-gap="loose" aria-label={t("rpg.map.legend")}>
    {#each legend as l (l.id)}
      <div part="RpgWorldMap.row" class="key" data-layout="row" data-gap="tight" data-kind={l.kind}><div class="mark" data-kind={l.kind}></div><span class="legend-label">{l.label}</span></div>
    {/each}
  </div>{/if}
</figure>

<style>
:host { position: relative; margin: 0; container-type: inline-size; color: var(--hud-ink); --pin: var(--map-marker); --cut: var(--hud-plate-cut); }
.sheet { position: relative; width: stretch; overflow: hidden; }
.land { position: absolute; left: 0; top: 0; width: calc(var(--zoom) * 100%); height: calc(var(--zoom) * 100%); translate: var(--pan-x) var(--pan-y); }
.terrain { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; }
.markers { position: absolute; inset: 0; }
/* A marker: a touch-sized button centred on its place, its pin inside,
   its name on a plate under it. */
.marker { position: absolute; left: calc(var(--x) * 100%); top: calc(var(--y) * 100%); width: var(--size-touch); height: var(--size-touch); padding: 0; border: 0; background: none; box-shadow: none; translate: -50% -50%; overflow: visible; }
.marker:disabled { opacity: 1; }
.marker .face { position: relative; width: stretch; height: stretch; overflow: visible; }
.ring { display: none; position: absolute; left: 50%; top: 50%; width: calc(var(--map-marker-on) * 1.6); height: calc(var(--map-marker-on) * 1.6); translate: -50% -50%; border-radius: var(--map-round); box-shadow: var(--map-chosen-ring); }
.marker[aria-selected=true] .ring { display: block; }
.marker:focus-visible .ring { display: block; box-shadow: var(--map-focus-ring); }
.marker[aria-selected=true] { --pin: var(--map-marker-on); z-index: 1; }
.pin { position: relative; width: var(--pin); height: var(--pin); display: flex; align-items: center; justify-content: center; }
.shape { position: absolute; inset: 0; }
.pin[data-kind=quest] { filter: var(--map-quest-glow); }
.pin[data-kind=quest] .shape { clip-path: polygon(50% 0, 100% 50%, 50% 100%, 0 50%); background: var(--map-quest); }
.pin[data-kind=quest] .bang i { background: var(--map-quest-ink); }
.pin[data-kind=danger] .shape { clip-path: polygon(50% 4%, 100% 96%, 0 96%); background: var(--map-danger); }
.pin[data-kind=danger] .bang i { background: var(--map-danger-ink); }
.pin[data-kind=danger] .bang { padding-top: calc(var(--pin) * 0.22); }
.pin[data-kind=waypoint] { width: calc(var(--pin) * 0.84); height: calc(var(--pin) * 0.84); border-radius: var(--map-round); background: var(--map-way); box-shadow: var(--map-way-glow); }
.pin[data-kind=waypoint] .shape { inset: 18%; border-radius: var(--map-round); background: var(--map-way-well); }
.pin[data-kind=player] { width: calc(var(--pin) * 0.84); height: calc(var(--pin) * 0.84); border-radius: var(--map-round); background: var(--map-player); box-shadow: var(--map-player-halo); }
.arrow { position: relative; width: 60%; height: 60%; rotate: 35deg; fill: var(--map-player-ink); }
/* The pin's mark of attention: a bar over a spot, drawn, not a glyph. */
.bang { position: relative; display: flex; flex-direction: column; align-items: center; gap: calc(var(--pin) * 0.06); }
.bar { width: calc(var(--pin) * 0.11); height: calc(var(--pin) * 0.3); }
.spot { width: calc(var(--pin) * 0.11); height: calc(var(--pin) * 0.11); }
/* The name: white on a small cut plate under the pin, at any kind. */
.name { position: absolute; left: 50%; top: calc(50% + var(--pin) / 2 + var(--space-xs)); translate: -50% 0; padding: calc(2 * var(--font-step-px)) var(--space-xs); white-space: nowrap; font-size: var(--map-name-size); font-weight: var(--hud-label-weight); color: var(--hud-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); clip-path: polygon(var(--cut) 0, 100% 0, 100% calc(100% - var(--cut)), calc(100% - var(--cut)) 100%, 0 100%, 0 var(--cut)); }
.marker[aria-selected=true] .name { top: calc(50% + var(--map-marker-on) * 0.8 + var(--space-xs)); font-size: var(--map-name-on-size); }
/* The caption and the legend: cut plates over the art. */
.here { position: absolute; left: var(--space-sm); top: var(--space-sm); align-items: center; padding: var(--space-xxs) var(--space-sm); margin: 0; font-size: var(--hud-size-xs); color: var(--hud-ink); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); clip-path: polygon(var(--cut) 0, 100% 0, 100% calc(100% - var(--cut)), calc(100% - var(--cut)) 100%, 0 100%, 0 var(--cut)); }
.here:dir(rtl) { left: auto; right: var(--space-sm); }
.here-mark { width: var(--map-legend-mark); height: var(--map-legend-mark); flex: none; border-radius: var(--map-round); background: var(--map-player); box-shadow: var(--map-way-glow); }
.legend { position: absolute; left: var(--space-sm); bottom: var(--space-sm); max-width: calc(100% - var(--space-sm) * 2); align-items: center; padding: var(--space-xxs) var(--space-md); font-size: var(--hud-size-xs); color: var(--hud-ink); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); clip-path: polygon(var(--cut) 0, 100% 0, 100% calc(100% - var(--cut)), calc(100% - var(--cut)) 100%, 0 100%, 0 var(--cut)); }
.legend:dir(rtl) { left: auto; right: var(--space-sm); }
.key { flex: none; align-items: center; white-space: nowrap; }
/* Each legend mark draws its marker's shape. */
.mark { width: var(--map-legend-mark); height: var(--map-legend-mark); flex: none; }
.mark[data-kind=quest] { clip-path: polygon(50% 0, 100% 50%, 50% 100%, 0 50%); background: var(--map-quest); }
.mark[data-kind=danger] { clip-path: polygon(50% 4%, 100% 96%, 0 96%); background: var(--map-danger); }
.mark[data-kind=waypoint] { border-radius: var(--map-round); box-shadow: var(--map-way-ring); }
.mark[data-kind=player] { border-radius: var(--map-round); background: var(--map-player); }
/* A narrow map (a phone's) keeps its places apart: pins follow the text
   size up to a share of the map's width, only the chosen place keeps its
   name on the art (every marker still says its name to assistive tech),
   and the legend runs in one row along the foot. */
@container (width < 560px) {
  :host { --pin: var(--map-marker-compact); }
  .marker[aria-selected=true] { --pin: var(--map-marker-on-compact); }
  .ring { width: calc(var(--map-marker-on-compact) * 1.6); height: calc(var(--map-marker-on-compact) * 1.6); }
  .marker:not([aria-selected=true]) .name { display: none; }
  .marker[aria-selected=true] .name { top: calc(50% + var(--map-marker-on-compact) * 0.8 + var(--space-xs)); font-size: var(--map-name-compact-size); }
  .legend { left: var(--space-xs); right: var(--space-xs); bottom: var(--space-xs); flex-wrap: nowrap; justify-content: center; column-gap: var(--space-sm); padding: var(--space-xxs) var(--space-sm); font-size: var(--map-name-compact-size); }
  .legend:dir(rtl) { left: var(--space-xs); right: var(--space-xs); }
  .here { left: var(--space-xs); top: var(--space-xs); font-size: var(--map-name-compact-size); }
}
</style>
