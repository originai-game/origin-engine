<meta name="component" content="ShooterRadar">
<meta name="strings" content="./shooter.en.ogt">
<meta name="prop:map" content="number" data-doc="The map texture (og.load.texture); 0 draws the dark well alone.">
<meta name="prop:centerX" content="number" data-default="0.5" data-doc="The map point under the player, 0 to 1 across.">
<meta name="prop:centerY" content="number" data-default="0.5" data-doc="The map point under the player, 0 to 1 down.">
<meta name="prop:zoom" content="number" data-default="0.4" data-doc="How much of the map the radar shows across, 0 to 1 (0.4).">
<meta name="prop:heading" content="number" data-doc="The player's heading in degrees clockwise from north: north stays up and the player's arrow turns.">
<meta name="prop:blips" content="list" data-doc="What the radar shows, keyed rows of { id, kind, x, y, label }: kind ally, foe or objective (on data-kind, each its own shape and colour); x and y on the map, 0 to 1; label names it for a reader. A blip off the radar's edge is clipped.">
<meta name="fixture:default" content='{"map":0,"centerX":0.5,"centerY":0.5,"zoom":0.4,"heading":-20,"blips":[{"id":"a1","kind":"ally","x":0.436,"y":0.54,"label":"Moraine"},{"id":"a2","kind":"ally","x":0.532,"y":0.58,"label":"Quill"},{"id":"f1","kind":"foe","x":0.548,"y":0.412,"label":"Enemy"},{"id":"f2","kind":"foe","x":0.596,"y":0.444,"label":"Enemy"},{"id":"o1","kind":"objective","x":0.476,"y":0.388,"label":"Relay tower"}]}'>
<meta name="fixture:empty" content='{"map":0,"centerX":0.5,"centerY":0.5,"zoom":0.4,"heading":0,"blips":[]}'>
<meta name="fixture:edge" content='{"map":0,"centerX":0.05,"centerY":0.95,"zoom":0.8,"heading":135,"blips":[{"id":"far","kind":"foe","x":0.9,"y":0.1,"label":"Enemy"},{"id":"near","kind":"objective","x":0.1,"y":0.9,"label":"Drop"}]}'>

<!--
  The radar at the screen's top corner: the map in a dark round well with
  a lit ring, a sweep and range rings over it, the squad and the enemies
  the game reveals as blips, the objective as a gold diamond, the player's
  arrow at the centre turned to the heading, and N on the ring's top.
  Read only.
-->
<section id="ShooterRadar" part="frame" variant="hud" layout="overlay" role="image" aria-label={t("shooter.radar.name")} style:--cx={centerX} style:--cy={centerY} style:--zoom={zoom > 0 ? zoom : 1} style:--heading={heading}>
  <div id="Scope" part="scope" anchor="fill">
    {#if map}<img id="Map" part="map" src={map} alt="">{/if}
    <div id="Sweep"></div>
    {#each blips as b (b.id)}<div part="blip" class="blip" style:--x={b.x} style:--y={b.y} aria-label={b.label} class:kind-ally={(b.kind) == 'ally'} class:kind-foe={(b.kind) == 'foe'} class:kind-objective={(b.kind) == 'objective'}></div>{/each}
    <svg id="Self" part="self" viewBox="0 0 16 16"><path d="M8 0l7 16-7-4-7 4z"></path></svg>
    <div id="Glass"></div>
  </div>
  <span class="anchor-top" id="North" part="north" anchor="top-edge">{t("shooter.radar.north")}</span>
</section>

<style>
#ShooterRadar { --side: calc(var(--size-5xl) * 0.8); inline-size: var(--side); block-size: var(--side); flex: none; --layout-anchor-inset: 0; }
#Scope { overflow: hidden; border-radius: var(--hud-radar-radius); background: var(--hud-radar-well); box-shadow: var(--hud-radar-rim); }
/* The map: the radar shows `zoom` of it across, the player's point at the centre. */
#Map { position: absolute; inline-size: calc(var(--side) / var(--zoom)); block-size: calc(var(--side) / var(--zoom)); left: calc(var(--side) / 2 - var(--cx) * var(--side) / var(--zoom)); top: calc(var(--side) / 2 - var(--cy) * var(--side) / var(--zoom)); object-fit: cover; }
#Sweep { position: absolute; inset: 0; background: var(--hud-radar-sweep); mask-image: radial-gradient(circle, #000 0 48%, transparent 49%); }
#Glass { position: absolute; inset: 0; background: var(--hud-radar-glass); }
.blip { position: absolute; --pip: calc(9 * var(--font-step-px)); inline-size: var(--pip); block-size: var(--pip); left: calc(var(--side) / 2 + (var(--x) - var(--cx)) * var(--side) / var(--zoom) - var(--pip) / 2); top: calc(var(--side) / 2 + (var(--y) - var(--cy)) * var(--side) / var(--zoom) - var(--pip) / 2); }
.blip.kind-ally { border-radius: var(--hud-radar-radius); background: var(--hud-ally); box-shadow: var(--hud-blip-rim); }
.blip.kind-foe { transform: rotate(45deg); background: var(--hud-foe); box-shadow: var(--hud-foe-glow); }
.blip.kind-objective { --pip: calc(13 * var(--font-step-px)); clip-path: polygon(50% 0, 100% 50%, 50% 100%, 0 50%); background: var(--hud-gold); }
/* The player's arrow at the centre, turned to the heading (north stays up). */
#Self { position: absolute; --me: calc(14 * var(--font-step-px)); inline-size: var(--me); block-size: var(--me); left: calc(50% - var(--me) / 2); top: calc(50% - var(--me) / 2); rotate: calc(var(--heading) * 1deg); fill: var(--hud-ink); filter: var(--hud-glyph-shadow); }
#North { padding: 0 var(--space-xs); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); white-space: nowrap; color: var(--hud-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
@media (width < 600px) { #ShooterRadar { --side: calc(var(--size-xl) * 1.25); } .blip { --pip: calc(7 * var(--font-step-px)); } }
.anchor-top { text-align: center; }
</style>
