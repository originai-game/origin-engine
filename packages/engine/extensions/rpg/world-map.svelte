<meta name="component" content="RpgWorldMap">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="Map.frame Map.marker Map.legend Map.place">
<meta name="prop:map" content="number" data-doc="The map's texture (og.load.texture); 0 draws the frame alone.">
<meta name="prop:region" content="string" data-doc="The region's name over the map.">
<meta name="prop:markers" content="list" data-doc="The places shown, each { id, x, y, icon, label, kind, tracked, discovered }: x and y place the marker from 0 to 1 across and down the map as the game projects it (the game pans and zooms); kind is player, quest, waypoint, town or danger; an undiscovered place draws dim and cannot be travelled to.">
<meta name="prop:filters" content="list" data-doc="The legend's toggles, each { id, label, icon, on }: the game hides the kinds turned off.">
<meta name="prop:selected" content="string" data-doc="The id of the chosen place; empty hides its card.">
<meta name="prop:detailName" content="string" data-doc="The chosen place's name.">
<meta name="prop:detailText" content="string" data-doc="Its line (Waypoint · 1.2 km, or a quest's objective).">
<meta name="prop:canTravel" content="bool" data-doc="Fast travel to it is allowed; false greys it.">
<meta name="prop:canTrack" content="bool" data-doc="A marker can be set on it.">
<meta name="action:details" content="Fast travel">
<meta name="fixture:default" content='{"map":0,"region":"Saltmarsh","selected":"abbey","detailName":"Greyfen Abbey","detailText":"Waypoint · 1.2 km","canTravel":true,"canTrack":true,"markers":[{"id":"you","x":0.42,"y":0.58,"icon":"person","label":"You","kind":"player","tracked":false,"discovered":true},{"id":"beacon","x":0.62,"y":0.36,"icon":"flag","label":"Marsh beacon","kind":"quest","tracked":true,"discovered":true},{"id":"abbey","x":0.24,"y":0.3,"icon":"home","label":"Greyfen Abbey","kind":"waypoint","tracked":false,"discovered":true},{"id":"lair","x":0.78,"y":0.72,"icon":"skull","label":"Drowned Hollow","kind":"danger","tracked":false,"discovered":false}],"filters":[{"id":"quest","label":"Quests","icon":"flag","on":true},{"id":"waypoint","label":"Waypoints","icon":"home","on":true},{"id":"danger","label":"Dangers","icon":"skull","on":false}]}'>
<meta name="fixture:empty" content='{"map":0,"region":"","selected":"","detailName":"","detailText":"","canTravel":false,"canTrack":false,"markers":[],"filters":[]}'>

<!--
  The rpg kit's world map (README.md): the map the game draws, its places
  as markers it projects, the legend's toggles and the chosen place's card
  with travel and mark. The game owns the camera, discovery and travel
  rules; the map reports choose (the marker's key), filter (the toggle's
  key), travel, track and center. Markers keep a touch-sized target.
-->
<section id="RpgWorldMap" part="Map.frame" aria-label={region || t("rpg.map.name")}>
  <div id="Field">
    <img id="Chart" src={map} alt="" hidden={!map}>
    <div id="Markers" role="list" aria-label={t("rpg.map.places")}>
      {#each markers as m (m.id)}
        <button id="Marker" part="Map.marker" class="marker" class:player={m.kind == "player"} class:quest={m.kind == "quest"} class:waypoint={m.kind == "waypoint"} class:town={m.kind == "town"} class:danger={m.kind == "danger"} class:unknown={!m.discovered} class:tracked={m.tracked} variant="quiet" role="listitem" aria-selected={m.id == selected} aria-label={m.label} style:left={m.x * 100 + "%"} style:top={m.y * 100 + "%"} onclick={choose}>
          <div class="pip"><icon class="glyph" icon={m.icon}></icon></div>
        </button>
      {/each}
    </div>
    <h2 id="Region" hidden={!region}>{region}</h2>
    <div id="Tools">
      <button id="Center" variant="secondary" aria-label={t("rpg.map.center")} onclick={center}><icon icon="person"></icon></button>
    </div>
  </div>
  <div id="Below">
    {#if filters.length}
    <div id="Legend" part="Map.legend" role="group" aria-label={t("rpg.map.legend")}>
      {#each filters as f (f.id)}
        <button id="Filter" class="filter" variant="quiet" aria-pressed={f.on} onclick={filter}><icon class="fglyph" icon={f.icon}></icon><span>{f.label}</span></button>
      {/each}
    </div>
    {/if}
    {#if detailName}
    <aside id="Place" part="Map.place" aria-label={detailName}>
      <div id="PlaceText">
        <h3 id="PlaceName">{detailName}</h3>
        <p class="quiet">{detailText}</p>
      </div>
      <div id="Actions">
        <button id="Track" variant="secondary" disabled={!canTrack} onclick={track}><icon icon="star"></icon><span>{t("rpg.map.track")}</span></button>
        <button id="Travel" variant="primary" disabled={!canTravel} onclick={travel} ondetails={travel}><kbd id="TravelKey" data-action="details"></kbd><span>{t("rpg.map.travel")}</span></button>
      </div>
    </aside>
    {/if}
  </div>
</section>

<style>
#RpgWorldMap { width: stretch; height: stretch; gap: var(--space-sm); padding: var(--space-md); background: var(--panel); border-radius: var(--radius-lg); container-type: inline-size; }
#Field { position: relative; width: stretch; height: stretch; min-height: calc(10 * var(--space-lg)); overflow: hidden; border-radius: var(--radius-sm); background: var(--well); border: var(--stroke-thin) solid var(--line-strong); }
#Chart { position: absolute; left: 0; top: 0; width: stretch; height: stretch; object-fit: cover; }
#Markers { position: absolute; left: 0; top: 0; width: stretch; height: stretch; }
.marker { position: absolute; width: var(--size-touch); height: var(--size-touch); min-width: var(--size-touch); min-height: var(--size-touch); padding: 0; translate: -50% -50%; align-items: center; justify-content: center; }
.pip { width: var(--space-xl); height: var(--space-xl); align-items: center; justify-content: center; border-radius: var(--radius-pill); background: var(--panel-raised); border: var(--stroke-thin) solid var(--ink-muted); box-shadow: 0 1px 3px var(--shadow); }
.glyph { width: var(--space-md); height: var(--space-md); color: var(--ink); }
.marker.quest .pip { border-color: var(--gold); }
.marker.quest .glyph { color: var(--gold); }
.marker.danger .glyph { color: var(--danger); }
.marker.player .pip { background: var(--highlight); border-color: var(--ink); }
.marker.player .glyph { color: var(--ink-inverse); }
.marker.tracked .pip { border-width: var(--stroke-thick); border-color: var(--highlight); }
.marker.unknown .pip { opacity: 0.55; }
.marker[aria-selected=true] .pip { scale: 1.25; border-color: var(--focus); }
#Region { position: absolute; left: var(--space-md); top: var(--space-sm); margin: 0; font: var(--text-xl); font-family: fantasy; color: var(--ink); text-shadow: 0 2px 4px var(--shadow); }
#Tools { position: absolute; right: var(--space-sm); top: var(--space-sm); gap: var(--space-xs); }
#Tools button { width: var(--size-touch); height: var(--size-touch); padding: 0; }
#Below { display: flex; width: stretch; gap: var(--space-md); align-items: center; }
#Legend { display: flex; flex-wrap: wrap; gap: var(--space-xs); width: stretch; }
.filter { min-height: var(--size-touch); gap: var(--space-xs); padding: 0 var(--space-sm); }
.filter[aria-pressed=false] { opacity: 0.55; }
.fglyph { width: var(--space-md); height: var(--space-md); }
#Place { display: flex; gap: var(--space-md); align-items: center; padding: var(--space-sm) var(--space-md); flex-shrink: 0; background: var(--panel-raised); border-radius: var(--radius-md); }
#PlaceText { gap: 0; }
#PlaceName { margin: 0; font: var(--text-body); font-family: fantasy; color: var(--ink); }
.quiet { margin: 0; color: var(--ink-muted); font: var(--text-sm); }
#Actions { display: flex; gap: var(--space-xs); }
#Actions button { min-height: var(--size-touch); }
@container (width < 640px) {
  #Below { flex-direction: column; align-items: stretch; }
  #Place { flex-wrap: wrap; }
  #PlaceText { width: stretch; }
  #Actions { width: stretch; }
  #Actions button { flex: 1 1 auto; }
}
@media (pointer: coarse) { kbd { display: none; } }
</style>
