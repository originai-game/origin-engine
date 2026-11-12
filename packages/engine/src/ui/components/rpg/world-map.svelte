<meta name="component" content="RpgWorldMap">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgWorldMap.frame RpgWorldMap.marker RpgWorldMap.row">
<meta name="prop:map" content="number" data-doc="The map's texture (og.load.texture); markers sit on it in fractions of its size.">
<meta name="prop:markers" content="list" data-doc="Keyed rows of { id, kind, x, y, label }: kind is quest, waypoint, danger or player; x and y 0 to 1 across and down the map.">
<meta name="prop:selected" content="string" data-doc="The marker selected; empty is none.">
<meta name="prop:legend" content="list" data-doc="The legend, keyed rows of { id, kind, label }, one line along the map's foot; empty hides it.">
<meta name="prop:here" content="string" data-doc="A caption pinned to the map's lower corner (Current location); empty hides it.">
<meta name="fixture:default" content='{"map":0,"markers":[{"id":"crown","kind":"quest","x":0.48,"y":0.3,"label":"Dawnhold"},{"id":"me","kind":"player","x":0.3,"y":0.55,"label":"You"}],"selected":"crown","legend":[{"id":"quest","kind":"quest","label":"Quest"},{"id":"danger","kind":"danger","label":"Danger"}],"here":""}'>

<!--
  A map with its markers, a one-line legend and an optional caption. The
  map is drawn to cover the box it is given; markers sit at fractions of
  it and report choose (a marker's key). The player's marker is not a
  button.
-->
<figure id="RpgWorldMap" part="RpgWorldMap.frame" aria-label={t("rpg.map.name")}>
  <div class="sheet">
    <img class="terrain" src={map} alt="">
    <div class="markers" role="group" aria-label={t("rpg.map.places")}>
      {#each markers as m (m.id)}
        <button part="RpgWorldMap.marker" data-kind={m.kind} disabled={m.kind == 'player'} aria-selected={m.id == selected} aria-label={m.label} style:--x={m.x} style:--y={m.y} onclick={choose}><div class="face">
          <div class="marker-mark"></div>
        </div></button>
      {/each}
    </div>
    <div class="here" hidden={!here}><div class="here-mark"></div><span class="here-text">{here}</span></div>
  </div>
  <div class="legend" hidden={!legend.length} aria-label={t("rpg.map.legend")}>
    {#each legend as l (l.id)}
      <div part="RpgWorldMap.row" data-kind={l.kind}><div class="legend-mark"></div><span class="legend-label">{l.label}</span></div>
    {/each}
  </div>
</figure>

<style>
[part="RpgWorldMap.frame"][hidden], [part="RpgWorldMap.frame"] [hidden] { display: none; }
[part="RpgWorldMap.frame"] { position: relative; width: stretch; height: stretch; margin: 0; container-type: inline-size; }
.sheet { position: relative; width: stretch; height: stretch; overflow: hidden; }
.terrain { position: absolute; left: 0; top: 0; width: stretch; height: stretch; object-fit: cover; }
.markers { position: absolute; left: 0; top: 0; width: stretch; height: stretch; }
[part="RpgWorldMap.marker"] { position: absolute; left: calc(var(--x) * 100%); top: calc(var(--y) * 100%); width: var(--size-md); height: var(--size-md); padding: 0; translate: -50% -50%; align-items: center; justify-content: center; }
[part="RpgWorldMap.marker"] .face { width: stretch; height: stretch; align-items: center; justify-content: center; }
.marker-mark { width: 62%; height: 62%; }
.here { position: absolute; left: 0; bottom: 0; display: flex; align-items: center; gap: var(--space-sm); padding: var(--space-xs) var(--space-md); margin: 0; }
.here:dir(rtl) { left: auto; right: 0; }
.here-mark { width: var(--space-lg); height: var(--space-lg); }
.legend { position: absolute; left: 50%; bottom: var(--space-sm); translate: -50% 0; display: flex; align-items: center; gap: var(--space-md); padding: var(--space-xxs) var(--space-md); white-space: nowrap; }
@container (width < 760px) { .legend { left: auto; right: var(--space-sm); translate: none; } .here { bottom: auto; top: 0; } .legend:dir(rtl) { right: auto; left: var(--space-sm); } }
[part="RpgWorldMap.row"] { display: flex; align-items: center; gap: var(--space-xs); flex: none; }
.legend-mark { width: var(--space-lg); height: var(--space-lg); }
</style>
