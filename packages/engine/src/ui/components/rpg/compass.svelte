<meta name="component" content="RpgCompass">
<meta name="strings" content="./rpg.en.ogt">
<meta name="prop:heading" content="string" data-doc="The facing as the game words it (N, 12°), read out to assistive technology.">
<meta name="prop:marks" content="list" data-doc="What the strip shows, keyed rows of { id, kind, label, at }: kind is cardinal (label N, E, S, W), tick, quest, waypoint or danger; at 0 to 1 across the strip, 0.5 the centre. The game places them from its heading (marks out of view are left out); expressions compute no geometry.">
<meta name="fixture:default" content='{"heading":"N","marks":[{"id":"w","kind":"cardinal","label":"W","at":0.06},{"id":"n","kind":"cardinal","label":"N","at":0.5},{"id":"e","kind":"cardinal","label":"E","at":0.83},{"id":"q","kind":"quest","label":"","at":0.55}]}'>

<!--
  A compass strip: the cardinal letters, ticks and pins the game places,
  under a fixed centre mark. It reports nothing.
-->
<div id="RpgCompass" class="rpg-compass" role="image" aria-label={t("rpg.compass.name", { heading: heading })}>
  <div class="end start"></div>
  <div class="strip">
    {#each marks as m (m.id)}
      <div class="mark" data-kind={m.kind} style:--at={m.at}><span class="mark-label" hidden={m.kind != 'cardinal'}>{m.label}</span></div>
    {/each}
  </div>
  <div class="needle"></div>
  <div class="end finish"></div>
</div>

<style>
.rpg-compass[hidden], .rpg-compass [hidden] { display: none; }
.rpg-compass { position: relative; display: flex; width: stretch; height: var(--size-lg); align-items: center; }
.end { width: var(--size-md); height: stretch; flex: none; }
.strip { position: relative; width: stretch; height: stretch; }
.mark { position: absolute; left: calc(var(--at) * 100%); top: 0; height: stretch; translate: -50% 0; align-items: center; justify-content: center; }
.mark[data-kind=tick] { width: 1px; height: 30%; top: 35%; }
.mark[data-kind=quest], .mark[data-kind=waypoint], .mark[data-kind=danger] { width: var(--space-lg); height: var(--space-lg); top: 0; }
.needle { position: absolute; left: 50%; top: 0; width: var(--size-md); height: stretch; translate: -50% 0; }
</style>
