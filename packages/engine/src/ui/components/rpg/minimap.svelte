<meta name="component" content="RpgMinimap">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgMinimap.frame RpgMinimap.scope RpgMinimap.map RpgMinimap.blip RpgMinimap.self RpgMinimap.north RpgMinimap.place RpgMinimap.clock">
<meta name="prop:map" content="number" data-doc="The map texture (og.load.texture); 0 draws the dark well alone.">
<meta name="prop:centerX" content="number" data-default="0.5" data-doc="The map point under the player, 0 to 1 across.">
<meta name="prop:centerY" content="number" data-default="0.5" data-doc="The map point under the player, 0 to 1 down.">
<meta name="prop:zoom" content="number" data-default="0.5" data-doc="How much of the map the minimap shows across, 0 to 1 (0.5).">
<meta name="prop:heading" content="number" data-doc="The player's heading in degrees clockwise from north: north stays up and the player's arrow turns.">
<meta name="prop:blips" content="list" data-doc="What the minimap shows, keyed rows of { id, kind, x, y, label }: kind ally, foe, boss or objective (on data-kind, each its own shape and colour); x and y on the map, 0 to 1; label names it for a reader. A blip off the minimap's edge is clipped.">
<meta name="prop:place" content="string" data-doc="Where the player is as the game words it (Ashen Vale), under the map; empty hides it.">
<meta name="prop:clock" content="string" data-doc="The world's time of day as the game words it (06:40), on a tag beside the place; empty hides it.">
<meta name="fixture:default" content='{"map":0,"centerX":0.5,"centerY":0.5,"zoom":0.5,"heading":0,"place":"Ashen Vale","clock":"06:40","blips":[{"id":"boss","kind":"boss","x":0.56,"y":0.4,"label":"Ashen Colossus"},{"id":"gate","kind":"objective","x":0.44,"y":0.37,"label":"The Ninefold Gate"},{"id":"a1","kind":"ally","x":0.46,"y":0.55,"label":"Brannoch"},{"id":"a2","kind":"ally","x":0.53,"y":0.57,"label":"Sylvenne"}]}'>
<meta name="fixture:empty" content='{"map":0,"centerX":0.5,"centerY":0.5,"zoom":0.5,"heading":0,"place":"","clock":"","blips":[]}'>
<meta name="fixture:edge" content='{"map":0,"centerX":0.05,"centerY":0.95,"zoom":0.9,"heading":135,"place":"Saltmarsh","clock":"","blips":[{"id":"far","kind":"foe","x":0.9,"y":0.1,"label":"Wolf"},{"id":"near","kind":"objective","x":0.1,"y":0.9,"label":"Shrine"}]}'>
<meta name="fixture:overflow" content='{"map":0,"centerX":0.5,"centerY":0.5,"zoom":0.5,"heading":-30,"place":"The Drowned Abbey of Saint Wendreth beneath the Mere","clock":"Night, 23:59:59","blips":[]}'>

<!--
  The minimap at the screen's top corner: the map in a dark round well
  with a lit ring, a sweep and range rings over it, the party, the foes
  and the objective as blips, the player's arrow at the centre turned to
  the heading and N on the ring's top; the place under it with the time
  of day on a tag. Read only.
-->
<section id="RpgMinimap" part="RpgMinimap.frame" class="hud bare" data-layout="stack" role="group" aria-label={place != '' ? place : t("rpg.map.name")} style:--cx={centerX} style:--cy={centerY} style:--zoom={zoom > 0 ? zoom : 1} style:--heading={heading}>
  <div id="Round" data-layout="overlay">
    <div id="Scope" part="RpgMinimap.scope" data-anchor="fill" role="image" aria-label={t("rpg.map.name")}>
      {#if map}<img id="Map" part="RpgMinimap.map" src={map} alt="">{/if}
      <div id="Sweep"></div>
      {#each blips as b (b.id)}<div part="RpgMinimap.blip" class="blip" data-kind={b.kind} style:--x={b.x} style:--y={b.y} aria-label={b.label}></div>{/each}
      <svg id="Self" part="RpgMinimap.self" viewBox="0 0 16 16"><path d="M8 0l7 16-7-4-7 4z"></path></svg>
      <div id="Glass"></div>
    </div>
    <span id="North" part="RpgMinimap.north" data-anchor="top">{t("rpg.minimap.north")}</span>
  </div>
  {#if place != '' || clock != ''}<div id="Where" data-layout="row" data-gap="tight">
    {#if place != ''}<span part="RpgMinimap.place" class="place">{place}</span>{/if}
    {#if clock != ''}<span part="RpgMinimap.clock" class="clock">{clock}</span>{/if}
  </div>{/if}
</section>

<style>
:host { --side: calc(var(--size-5xl) * 0.8); max-inline-size: stretch; flex: none; align-items: center; row-gap: calc(8 * var(--font-step-px)); color: var(--hud-ink); --layout-anchor-inset: 0; }
#Round { flex: none; inline-size: var(--side); block-size: var(--side); }
#Scope { overflow: hidden; border-radius: var(--hud-radar-radius); background: var(--hud-radar-well); box-shadow: var(--hud-radar-rim); }
/* The map: the minimap shows `zoom` of it across, the player's point at the centre. */
#Map { position: absolute; inline-size: calc(var(--side) / var(--zoom)); block-size: calc(var(--side) / var(--zoom)); left: calc(var(--side) / 2 - var(--cx) * var(--side) / var(--zoom)); top: calc(var(--side) / 2 - var(--cy) * var(--side) / var(--zoom)); object-fit: cover; }
#Sweep { position: absolute; inset: 0; background: var(--hud-radar-sweep); mask-image: radial-gradient(circle, #000 0 48%, transparent 49%); }
#Glass { position: absolute; inset: 0; background: var(--hud-radar-glass); }
.blip { position: absolute; --pip: calc(8 * var(--font-step-px)); inline-size: var(--pip); block-size: var(--pip); left: calc(var(--side) / 2 + (var(--x) - var(--cx)) * var(--side) / var(--zoom) - var(--pip) / 2); top: calc(var(--side) / 2 + (var(--y) - var(--cy)) * var(--side) / var(--zoom) - var(--pip) / 2); }
.blip[data-kind=ally] { border-radius: var(--hud-radar-radius); background: var(--hud-ally); box-shadow: var(--hud-blip-rim); }
.blip[data-kind=foe] { transform: rotate(45deg); background: var(--hud-foe); box-shadow: var(--hud-foe-glow); }
.blip[data-kind=boss] { --pip: calc(11 * var(--font-step-px)); transform: rotate(45deg); background: var(--hud-foe); box-shadow: var(--hud-foe-glow); }
.blip[data-kind=objective] { --pip: calc(11 * var(--font-step-px)); clip-path: polygon(50% 0, 100% 50%, 50% 100%, 0 50%); background: var(--hud-gold); }
/* The player's arrow at the centre, turned to the heading (north stays up). */
#Self { position: absolute; --me: calc(13 * var(--font-step-px)); inline-size: var(--me); block-size: var(--me); left: calc(50% - var(--me) / 2); top: calc(50% - var(--me) / 2); rotate: calc(var(--heading) * 1deg); fill: var(--hud-ink); filter: var(--hud-glyph-shadow); }
#North { padding: 0 var(--space-xs); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); white-space: nowrap; color: var(--hud-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
/* The place under the map, its time of day on a tag. */
#Where { align-items: center; justify-content: center; max-inline-size: 100%; min-inline-size: 0; }
.place { min-inline-size: 0; font-size: var(--hud-size-name); font-weight: var(--hud-label-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
.clock { flex: none; max-inline-size: 50%; padding: 0 var(--space-xs); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
@media (width < 600px), (height < 500px) {
  :host { --side: calc(var(--size-xl) * 1.6); row-gap: calc(6 * var(--font-step-px)); }
  .blip { --pip: calc(7 * var(--font-step-px)); }
  .blip[data-kind=boss], .blip[data-kind=objective] { --pip: calc(10 * var(--font-step-px)); }
  #Self { --me: calc(11 * var(--font-step-px)); }
  /* The clock goes under the place rather than widen the map's column. */
  #Where { flex-wrap: wrap; max-inline-size: var(--side); row-gap: 0; }
  .place { font-size: var(--hud-size-xs); }
}
</style>
