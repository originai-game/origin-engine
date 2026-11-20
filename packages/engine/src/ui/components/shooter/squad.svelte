<meta name="component" content="ShooterSquad">
<meta name="strings" content="./shooter.en.ogt">
<meta name="prop:members" content="list" data-doc="The player's squadmates in the game's order, keyed rows of { id, name, portrait, status, state, shield, shieldMax, health, healthMax, ghost }: portrait a texture (0: the person glyph); status the state as the game words it (Reviving), empty for none; state ok, down, reviving or out (on data-state: down and out dim the plate, reviving inks the status in the shield's blue); shieldMax 0 hides the shield bar; ghost the health before the latest loss.">
<meta name="prop:segment" content="number" data-default="25" data-doc="Shield per cell of each shield bar (25).">
<meta name="fixture:default" content='{"segment":25,"members":[{"id":"moraine","name":"Moraine","portrait":0,"status":"Reviving","state":"reviving","shield":50,"shieldMax":100,"health":38,"healthMax":100,"ghost":64},{"id":"quill","name":"Quill","portrait":0,"status":"","state":"ok","shield":100,"shieldMax":100,"health":100,"healthMax":100,"ghost":0}]}'>
<meta name="fixture:down" content='{"segment":25,"members":[{"id":"tarn","name":"Tarn","portrait":0,"status":"Down","state":"down","shield":0,"shieldMax":100,"health":0,"healthMax":100,"ghost":20},{"id":"sable","name":"Sable","portrait":0,"status":"Eliminated","state":"out","shield":0,"shieldMax":100,"health":0,"healthMax":100,"ghost":0},{"id":"ashby","name":"Ashby","portrait":0,"status":"","state":"ok","shield":25,"shieldMax":100,"health":12,"healthMax":100,"ghost":30}]}'>
<meta name="fixture:empty" content='{"segment":25,"members":[]}'>
<meta name="fixture:overflow" content='{"segment":50,"members":[{"id":"long","name":"xX_Midnight_Sniper_Of_The_Northern_Relay_Xx","portrait":0,"status":"Reviving a squadmate under heavy fire","state":"reviving","shield":250,"shieldMax":300,"health":9999,"healthMax":12000,"ghost":11000},{"id":"none","name":"Quill","portrait":0,"status":"","state":"ok","shield":0,"shieldMax":0,"health":100,"healthMax":100,"ghost":0}]}'>

<!--
  The squad at the screen's top corner: each squadmate's portrait in an
  octagon, the name and a state in words, a thin shield bar cut into cells
  over a thin health bar with its ghost. The game owns every value and
  state; read only.
-->
<section id="ShooterSquad" part="frame" variant="hud" layout="stack" role="group" aria-label={t("shooter.squad.name")}>
  {#each members as m (m.id)}
  <div part="mate" class="mate" role="group" aria-label={m.status ? t("shooter.squad.mate", { name: m.name, status: m.status }) : m.name} class:state-down={(m.state) == 'down'} class:state-out={(m.state) == 'out'} class:state-reviving={(m.state) == 'reviving'}>
    <div class="portrait" part="portrait" layout="overlay">
      {#if m.portrait}<img class="face" part="face" anchor="fill" src={m.portrait} alt="">{:else}<div class="blank" anchor="fill" layout="center"><og-icon class="glyph" icon="person"></og-icon></div>{/if}
    </div>
    <span part="callsign" class="callsign">{m.name}</span>
    {#if m.status}<span part="status" class="status">{m.status}</span>{/if}
      {#if m.shieldMax > 0}
      <div part="shield" class="bar shield" layout="overlay" style:--value={m.shield <= 0 ? 0 : m.shield >= m.shieldMax ? 1 : m.shield / m.shieldMax} style:--cells={segment > 0 ? m.shieldMax / segment : 1}>
        <div class="fill" anchor="top-start"></div>
        <div class="cuts" anchor="fill"></div>
      </div>
      {/if}
      <div part="health" class="bar health" layout="overlay" role="meter" aria-label={t("shooter.vitals.health")} aria-valuemin="0" aria-valuemax={m.healthMax} aria-valuenow={m.health} style:--value={m.health <= 0 ? 0 : m.health >= m.healthMax ? 1 : m.health / m.healthMax} style:--ghost={m.ghost <= m.health ? 0 : m.ghost >= m.healthMax ? 1 : m.ghost / m.healthMax}>
        <div part="ghost" class="ghost" anchor="top-start"></div>
        <div class="fill" anchor="top-start"></div>
      </div>
  </div>
  {/each}
</section>

<style>
#ShooterSquad { row-gap: calc(10 * var(--font-step-px)); inline-size: calc(220 * var(--font-step-px)); max-inline-size: stretch; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
/* A plate: the portrait beside the name and state over the bars. */
.mate { display: grid; grid-template-columns: auto minmax(0, auto) minmax(0, 1fr); grid-template-areas: "portrait name status" "portrait shield shield" "portrait health health"; column-gap: calc(10 * var(--font-step-px)); row-gap: calc(4 * var(--font-step-px)); align-items: baseline; min-inline-size: 0; --layout-anchor-inset: 0; }
.mate.state-down, .mate.state-out { opacity: 0.6; }
/* The portrait: an octagon with a lit edge, the art inset inside it. */
.portrait { grid-area: portrait; align-self: center; inline-size: calc(40 * var(--font-step-px)); block-size: calc(40 * var(--font-step-px)); clip-path: var(--hud-octagon); background: var(--hud-edge); --layout-anchor-inset: calc(2 * var(--font-step-px)); }
.face, .blank { clip-path: var(--hud-octagon); background: var(--hud-plate-fill); }
.face { object-fit: cover; }
.mate.state-out .face { filter: var(--hud-unready); }
.glyph { inline-size: 50%; block-size: 50%; color: var(--hud-ink-dim); }
.callsign { grid-area: name; min-inline-size: 0; font-size: var(--hud-size-name); font-weight: var(--hud-label-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.status { grid-area: status; justify-self: start; max-inline-size: 100%; min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.mate.state-reviving .status { color: var(--hud-ally); }
.mate.state-down .status, .mate.state-out .status { color: var(--hud-foe); }
/* Thin leaning bars: the shield cut into cells over health and its ghost. */
.shield { grid-area: shield; }
.health { grid-area: health; }
.bar { align-self: center; inline-size: 100%; block-size: calc(6 * var(--font-step-px)); overflow: hidden; transform: skewX(var(--bar-skew)); background: var(--bar-well); box-shadow: var(--bar-rim); }
.fill { inline-size: calc(var(--value) * 100%); block-size: 100%; }
.shield .fill { background: var(--hud-shield-fill); box-shadow: var(--hud-shield-glow); }
.health .fill { background: var(--hud-health-fill); }
.cuts { background: var(--bar-notch); mask-image: repeating-linear-gradient(90deg, transparent 0 calc(100% / var(--cells) - 2 * var(--font-step-px)), #000 calc(100% / var(--cells) - 2 * var(--font-step-px)) calc(100% / var(--cells))); }
.ghost { inline-size: calc(var(--ghost) * 100%); block-size: 100%; background: var(--bar-ghost); transition: var(--bar-ghost-motion); }
/* On a phone the plates stand side by side, each compact: a small
   portrait beside the name over the bars, the state on a tag under
   them. */
@media (width < 600px) {
  #ShooterSquad { flex-direction: row; inline-size: stretch; column-gap: var(--space-md); }
  .mate { flex: 1 1 0; }
  .mate { grid-template-columns: auto minmax(0, 1fr); grid-template-areas: "portrait name" "portrait shield" "portrait health" "status status"; column-gap: calc(4 * var(--font-step-px)); row-gap: calc(3 * var(--font-step-px)); }
  .portrait { align-self: start; inline-size: calc(28 * var(--font-step-px)); block-size: calc(28 * var(--font-step-px)); }
  .callsign { font-size: var(--hud-size-xs); }
  .bar { block-size: calc(4 * var(--font-step-px)); }
  .status { padding: 0 calc(3 * var(--font-step-px)); font-size: var(--hud-tag-size); letter-spacing: normal; -webkit-text-stroke-width: 0; text-shadow: none; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
}
</style>
