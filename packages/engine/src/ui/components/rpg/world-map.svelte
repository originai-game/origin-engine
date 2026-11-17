<meta name="component" content="RpgWorldMap">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgWorldMap.frame RpgWorldMap.marker RpgWorldMap.row RpgWorldMap.cell RpgWorldMap.dot RpgWorldMap.plate RpgWorldMap.caption">
<meta name="prop:map" content="number" data-doc="The map's texture (og.load.texture); markers sit on it in fractions of its size.">
<meta name="prop:markers" content="list" data-doc="Keyed rows of { id, kind, x, y, label }: kind is quest, waypoint, danger or player; x and y 0 to 1 across and down the map.">
<meta name="prop:selected" content="string" data-doc="The marker selected; empty is none.">
<meta name="prop:zoom" content="number" data-default="1" data-doc="How far the map is zoomed in: 1 (and 0, and when left out) fits it to the box, 2 shows it twice the size. The game keeps it, from the zoom and pinch events, within its own limits.">
<meta name="prop:panX" content="number" data-default="0" data-doc="How far the zoomed map is moved across, CSS px (0, and when left out, pins its start corner). The game keeps it, from grab, pan and release.">
<meta name="prop:panY" content="number" data-default="0" data-doc="How far the zoomed map is moved down, CSS px; 0 when left out.">
<meta name="prop:legend" content="list" data-doc="The legend, keyed rows of { id, kind, label }, one line along the map's foot; empty hides it.">
<meta name="prop:here" content="string" data-doc="A caption pinned to the map's lower corner (Current location); empty hides it.">
<meta name="fixture:default" content='{"map":0,"markers":[{"id":"crown","kind":"quest","x":0.48,"y":0.3,"label":"Dawnhold"},{"id":"me","kind":"player","x":0.3,"y":0.55,"label":"You"}],"selected":"crown","zoom":1,"panX":0,"panY":0,"legend":[{"id":"quest","kind":"quest","label":"Quest"},{"id":"danger","kind":"danger","label":"Danger"}],"here":""}'>

<!--
  A map with its markers, a one-line legend and an optional caption. The
  map is drawn to cover the box it is given, times zoom and moved by panX
  and panY; markers sit at fractions of it, keep their size at any zoom
  and report choose (a marker's key). The player's marker is not a
  button. The game owns the zoom and the pan: the map reports zoom (the
  wheel, with deltaY), pinch (two touches, with scale), and grab, pan and
  release (a press, its moves and its end, each with x and y in the map's
  box), and draws what the game sets.
-->
<figure id="RpgWorldMap" part="RpgWorldMap.frame" class="bare" data-layout="shell" aria-label={t("rpg.map.name")}>
  <div class="sheet" data-fill onwheel={zoom} onpinch={pinch} onpointerdown={grab} onpointermove={pan} onpointerup={release}>
    <div class="land" style:--zoom={zoom || 1} style:--pan-x={panX + "px"} style:--pan-y={panY + "px"}>
      <img class="terrain" src={map} alt="">
      <div class="markers" role="group" aria-label={t("rpg.map.places")}>
        {#each markers as m (m.id)}
          <button part="RpgWorldMap.marker RpgWorldMap.cell" data-kind={m.kind} disabled={m.kind == 'player'} aria-selected={m.id == selected} aria-label={m.label} style:--x={m.x} style:--y={m.y} onclick={choose}><div class="face" data-layout="cover">
            <div class="marker-mark" part="RpgWorldMap.dot" data-shape={m.kind == 'quest' ? 'diamond' : null} data-tone={m.kind == 'danger' ? 'danger' : m.kind == 'player' ? 'success' : 'primary'}></div>
          </div></button>
        {/each}
      </div>
    </div>
    {#if here}<div class="here hud" part="RpgWorldMap.plate RpgWorldMap.frame" data-layout="row" data-gap="tight"><div class="here-mark" part="RpgWorldMap.dot" data-tone="success"></div><span class="here-text" part="RpgWorldMap.caption">{here}</span></div>{/if}
  </div>
  {#if legend.length}<div class="legend hud" part="RpgWorldMap.plate RpgWorldMap.frame" data-layout="row" data-gap="loose" aria-label={t("rpg.map.legend")}>
    {#each legend as l (l.id)}
      <div part="RpgWorldMap.row" data-layout="row" data-gap="tight" data-kind={l.kind}><div class="legend-mark" part="RpgWorldMap.dot" data-shape={l.kind == 'quest' ? 'diamond' : null} data-tone={l.kind == 'danger' ? 'danger' : l.kind == 'player' ? 'success' : 'primary'}></div><span class="legend-label" part="RpgWorldMap.caption">{l.label}</span></div>
    {/each}
  </div>{/if}
</figure>

<style>
:host { position: relative; margin: 0; --plate-fill: var(--scene-plate); --caption-ink: var(--scene-ink); --row-fill: none; --row-pad: 0px; }
.sheet { position: relative; width: stretch; overflow: hidden; }
.land { position: absolute; left: 0; top: 0; width: calc(var(--zoom) * 100%); height: calc(var(--zoom) * 100%); translate: var(--pan-x) var(--pan-y); }
.terrain { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; }
.markers { position: absolute; inset: 0; }
[part="RpgWorldMap.marker RpgWorldMap.cell"] { position: absolute; left: calc(var(--x) * 100%); top: calc(var(--y) * 100%); width: var(--size-md); height: var(--size-md); padding: 0; translate: -50% -50%; align-items: center; justify-content: center; }
[part="RpgWorldMap.marker RpgWorldMap.cell"] .face { width: stretch; height: stretch; }
.marker-mark { width: 45%; height: 45%; }
.here { position: absolute; left: 0; bottom: 0; padding: var(--space-xs) var(--space-md); margin: 0; }
.here:dir(rtl) { left: auto; right: 0; }
.here-mark { width: var(--space-sm); height: var(--space-sm); }
/* Over the map art, which the game draws: the scene plate and its ink. */
.legend { position: absolute; left: 50%; bottom: var(--space-sm); translate: -50% 0; padding: var(--space-xxs) var(--space-md); white-space: nowrap; }
@container (width < 760px) { .legend { left: auto; right: var(--space-sm); translate: none; } .here { bottom: auto; top: 0; } .legend:dir(rtl) { right: auto; left: var(--space-sm); } }
[part="RpgWorldMap.row"] { flex: none; }
.legend-mark { width: var(--space-sm); height: var(--space-sm); }
</style>
