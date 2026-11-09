<meta name="component" content="RpgCompass">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="Compass.frame Compass.marker">
<meta name="prop:facing" content="string" data-doc="The facing as the game words it (NE, 045°).">
<meta name="prop:points" content="list" data-doc="The cardinal and ordinal letters in view: each { id, x, label }, x 0 to 1 across the strip as the game projects the facing.">
<meta name="prop:markers" content="list" data-doc="The places in view: each { id, x, icon, kind, distance, above, below }. x is 0 to 1 across the strip; kind quest, waypoint, danger or ally; distance as words (240 m), empty hides it; above or below marks a place up or down a level.">
<meta name="fixture:default" content='{"facing":"NE","points":[{"id":"n","x":0.18,"label":"N"},{"id":"ne","x":0.5,"label":"NE"},{"id":"e","x":0.82,"label":"E"}],"markers":[{"id":"beacon","x":0.62,"icon":"flag","kind":"quest","distance":"240 m","above":false,"below":false},{"id":"abbey","x":0.31,"icon":"home","kind":"waypoint","distance":"1.2 km","above":false,"below":false},{"id":"lair","x":0.9,"icon":"skull","kind":"danger","distance":"","above":false,"below":true}]}'>

<!--
  The rpg kit's compass (README.md), the open-world HUD's bearing strip:
  the facing, its letters and the places in view, each where the game
  projects it, with its distance. It sends nothing; the game computes
  every bearing and decides what is in view.
-->
<div id="RpgCompass" part="Compass.frame" role="group" aria-label={t("rpg.compass.name", { heading: facing })}>
  <div id="Strip">
    {#each points as p (p.id)}
      <span class="point" class:cardinal={p.label.length == 1} style:left={p.x * 100 + "%"}>{p.label}</span>
    {/each}
    {#each markers as m (m.id)}
      <div class="marker" part="Compass.marker" class:quest={m.kind == "quest"} class:waypoint={m.kind == "waypoint"} class:danger={m.kind == "danger"} class:ally={m.kind == "ally"} style:left={m.x * 100 + "%"}>
        <icon class="glyph" icon={m.icon}></icon>
        <span class="distance number" hidden={!m.distance}>{m.distance}</span>
        <div class="level" class:above={m.above} hidden={!(m.above || m.below)} aria-label={m.above ? t("rpg.compass.above") : t("rpg.compass.below")}><icon class="lv" icon="play"></icon></div>
      </div>
    {/each}
    <div id="Needle"></div>
  </div>
</div>

<style>
#RpgCompass { width: stretch; max-width: calc(24 * var(--space-lg)); height: calc(3 * var(--space-lg)); padding: 0 var(--space-md); }
#Strip { position: relative; width: stretch; height: stretch; overflow: hidden; border-block-end: var(--stroke-thin) solid var(--ink-muted); }
.point { position: absolute; top: var(--space-xxs); translate: -50% 0; font: var(--text-caption); color: var(--ink-muted); text-shadow: 0 1px 2px var(--shadow); }
.point.cardinal { font: var(--text-sm); font-family: fantasy; color: var(--ink); }
.marker { position: absolute; top: var(--space-md); translate: -50% 0; gap: 0; align-items: center; }
.glyph { width: var(--space-lg); height: var(--space-lg); color: var(--ink); filter: drop-shadow(0 1px 2px var(--shadow)); }
.marker.quest .glyph { color: var(--gold); }
.marker.danger .glyph { color: var(--danger); }
.marker.ally .glyph { color: var(--team1); }
.number { font-family: monospace; font-variant-numeric: tabular-nums; white-space: nowrap; }
.distance { font: var(--text-caption); font-family: monospace; color: var(--ink); text-shadow: 0 1px 2px var(--shadow); }
.level { position: absolute; right: calc(-1 * var(--space-sm)); top: 0; }
.lv { width: var(--space-sm); height: var(--space-sm); color: var(--ink-muted); rotate: 90deg; }
.level.above .lv { rotate: -90deg; }
#Needle { position: absolute; left: 50%; bottom: 0; width: var(--stroke-thick); height: var(--space-sm); translate: -50% 0; background: var(--highlight); }
</style>
