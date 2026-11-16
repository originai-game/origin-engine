<meta name="component" content="RpgCompass">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgCompass.frame">
<meta name="prop:heading" content="string" data-doc="The facing as the game words it (N, 12°), read out to assistive technology.">
<meta name="prop:marks" content="list" data-doc="What the strip shows, keyed rows of { id, kind, label, at }: kind is cardinal (label N, E, S, W), tick, quest, waypoint or danger; at 0 to 1 across the strip, 0.5 the centre. The game places them from its heading (marks out of view are left out); expressions compute no geometry.">
<meta name="fixture:default" content='{"heading":"N","marks":[{"id":"w","kind":"cardinal","label":"W","at":0.06},{"id":"n","kind":"cardinal","label":"N","at":0.5},{"id":"e","kind":"cardinal","label":"E","at":0.83},{"id":"q","kind":"quest","label":"","at":0.55}]}'>

<!--
  A compass strip: the cardinal letters, ticks and pins the game places,
  under a fixed centre mark. It reports nothing.
-->
<div id="RpgCompass" part="RpgCompass.frame" class="bare" role="image" aria-label={t("rpg.compass.name", { heading: heading })}>
  <div class="end start"></div>
  <div class="strip">
    {#each marks as m (m.id)}
      <div class="mark" data-kind={m.kind} style:--at={m.at}>{#if m.kind == 'cardinal'}<span class="mark-label">{m.label}</span>{/if}</div>
    {/each}
  </div>
  <div class="needle"></div>
  <div class="end finish"></div>
</div>

<style>
/* Drawn over the scene: the style's tag halo holds its text there. */
:host { text-shadow: var(--tag-halo); }
:host { position: relative; display: flex; inline-size: stretch; block-size: var(--size-lg); align-items: center; background: var(--scene-plate); color: var(--scene-ink); border-radius: var(--radius-capsule); }
.end { width: var(--size-md); align-self: stretch; flex: none; }
.strip { position: relative; flex: 1; min-inline-size: 0; align-self: stretch; overflow: hidden; }
.mark { display: flex; flex-direction: column; position: absolute; left: calc(var(--at) * 100%); top: 0; bottom: 0; translate: -50% 0; align-items: center; justify-content: center; }
.mark[data-kind=tick] { width: var(--stroke-hairline); height: 30%; top: 35%; background: var(--ink-muted); }
.mark[data-kind=quest], .mark[data-kind=waypoint], .mark[data-kind=danger] { width: var(--space-sm); height: var(--space-sm); top: var(--space-xs); background: var(--warn); border-radius: var(--radius-capsule); }
.mark[data-kind=danger] { background: var(--danger); }
.mark[data-kind=waypoint] { background: var(--info); }
.needle { position: absolute; left: 50%; top: 0; width: var(--space-xs); height: var(--space-sm); background: var(--primary); border-radius: var(--radius-capsule); translate: -50% 0; }
</style>
