<meta name="component" content="ShooterSquad">
<meta name="strings" content="./shooter.en.ogt">
<meta name="parts" content="ShooterSquad.frame ShooterSquad.mate ShooterSquad.portrait ShooterSquad.callsign ShooterSquad.status ShooterSquad.shield ShooterSquad.health">
<meta name="prop:members" content="list" data-doc="The player's squadmates in the game's order, keyed rows of { id, name, portrait, status, state, shield, shieldMax, health, healthMax, ghost }: portrait a texture (0: the person glyph); status the state as the game words it (Reviving), empty for none; state ok, down, reviving or out (on data-state: down and out dim the plate, reviving inks the status in the shield's blue); shieldMax 0 hides the shield bar; ghost the health before the latest loss.">
<meta name="prop:segment" content="number" data-default="25" data-doc="Shield per cell of each shield bar (25).">
<meta name="fixture:default" content='{"segment":25,"members":[{"id":"moraine","name":"Moraine","portrait":0,"status":"Reviving","state":"reviving","shield":50,"shieldMax":100,"health":38,"healthMax":100,"ghost":64},{"id":"quill","name":"Quill","portrait":0,"status":"","state":"ok","shield":100,"shieldMax":100,"health":100,"healthMax":100,"ghost":0}]}'>
<meta name="fixture:down" content='{"segment":25,"members":[{"id":"tarn","name":"Tarn","portrait":0,"status":"Down","state":"down","shield":0,"shieldMax":100,"health":0,"healthMax":100,"ghost":20},{"id":"sable","name":"Sable","portrait":0,"status":"Eliminated","state":"out","shield":0,"shieldMax":100,"health":0,"healthMax":100,"ghost":0},{"id":"ashby","name":"Ashby","portrait":0,"status":"","state":"ok","shield":25,"shieldMax":100,"health":12,"healthMax":100,"ghost":30}]}'>
<meta name="fixture:empty" content='{"segment":25,"members":[]}'>
<meta name="fixture:overflow" content='{"segment":50,"members":[{"id":"long","name":"xX_Midnight_Sniper_Of_The_Northern_Relay_Xx","portrait":0,"status":"Reviving a squadmate under heavy fire","state":"reviving","shield":250,"shieldMax":300,"health":9999,"healthMax":12000,"ghost":11000},{"id":"none","name":"Quill","portrait":0,"status":"","state":"ok","shield":0,"shieldMax":0,"health":100,"healthMax":100,"ghost":0}]}'>

<!--
  The squad at the screen's top corner, built from the HUD's blocks: each
  squadmate on a smoked plate, an HudPortrait beside the name and a state
  in words, a thin shield HudBar cut into cells over a thin health HudBar
  with its ghost. The game owns every value and state; read only.
-->
<section id="ShooterSquad" part="ShooterSquad.frame" class="hud bare" data-layout="stack" role="group" aria-label={t("shooter.squad.name")}>
  {#each members as m (m.id)}
  <div part="ShooterSquad.mate" class="mate" data-state={m.state} role="group" aria-label={m.status ? t("shooter.squad.mate", { name: m.name, status: m.status }) : m.name}>
    <div class="portrait" part="ShooterSquad.portrait"><HudPortrait portrait={m.portrait} dimmed={m.state == 'out'} /></div>
    <span part="ShooterSquad.callsign" class="callsign">{m.name}</span>
    {#if m.status}<span part="ShooterSquad.status" class="status">{m.status}</span>{/if}
    {#if m.shieldMax > 0}<div part="ShooterSquad.shield" class="bar shield"><HudBar value={m.shield} max={m.shieldMax} kind="shield" ghost={0} segment={segment} low={false} showValue={false} name={t("shooter.vitals.shield")} /></div>{/if}
    <div part="ShooterSquad.health" class="bar health"><HudBar value={m.health} max={m.healthMax} kind="health" ghost={m.ghost} segment={0} low={false} showValue={false} name={t("shooter.vitals.health")} /></div>
  </div>
  {/each}
</section>

<style>
:host { row-gap: calc(6 * var(--font-step-px)); inline-size: calc(230 * var(--font-step-px)); max-inline-size: stretch; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
/* A plate: smoked glass with two corners cut, the portrait beside the
   name and state over the bars. */
.mate { display: grid; grid-template-columns: auto minmax(0, auto) minmax(0, 1fr); grid-template-areas: "portrait name status" "portrait shield shield" "portrait health health"; column-gap: calc(8 * var(--font-step-px)); row-gap: calc(4 * var(--font-step-px)); align-items: baseline; min-inline-size: 0; padding: calc(4 * var(--font-step-px)) calc(8 * var(--font-step-px)) calc(5 * var(--font-step-px)) calc(4 * var(--font-step-px)); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); }
.mate[data-state=down], .mate[data-state=out] { opacity: 0.6; }
.mate[data-state=down] { box-shadow: var(--hud-plate-rim), inset calc(3 * var(--font-step-px)) 0 0 var(--hud-foe); }
.mate[data-state=reviving] { box-shadow: var(--hud-plate-rim), inset calc(3 * var(--font-step-px)) 0 0 var(--hud-ally); }
/* The portrait's place: a small octagon. */
.portrait { grid-area: portrait; align-self: center; --hud-portrait-size: calc(38 * var(--font-step-px)); }
.callsign { grid-area: name; min-inline-size: 0; font-size: var(--hud-size-name); font-weight: var(--hud-label-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.status { grid-area: status; justify-self: start; max-inline-size: 100%; min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.mate[data-state=reviving] .status { color: var(--hud-ally); }
.mate[data-state=down] .status, .mate[data-state=out] .status { color: var(--hud-foe); }
/* Thin bars' places: the shield cut into cells over health and its ghost,
   in the shooter's bright health fill. */
.shield { grid-area: shield; }
.health { grid-area: health; --hud-life-fill: var(--hud-health-fill); --hud-life-glow: none; }
.bar { align-self: center; inline-size: 100%; --hud-bar-size: calc(6 * var(--font-step-px)); }
/* Short of room (a phone, or a phone on its side) each plate is compact:
   a small portrait beside the name over thinner bars. */
@media (width < 600px), (height < 500px) {
  .mate { column-gap: calc(4 * var(--font-step-px)); row-gap: calc(3 * var(--font-step-px)); padding: calc(3 * var(--font-step-px)); }
  .portrait { --hud-portrait-size: calc(26 * var(--font-step-px)); }
  .callsign { font-size: var(--hud-size-xs); }
  .bar { --hud-bar-size: calc(4 * var(--font-step-px)); }
}
/* On an upright phone the plates stand side by side, the state on a tag
   under the bars. */
@media (width < 600px) {
  :host { flex-direction: row; inline-size: stretch; column-gap: var(--space-sm); }
  .mate { flex: 1 1 0; grid-template-columns: auto minmax(0, 1fr); grid-template-areas: "portrait name" "portrait shield" "portrait health" "status status"; }
  .portrait { align-self: start; }
  .status { padding: 0 calc(3 * var(--font-step-px)); font-size: var(--hud-tag-size); letter-spacing: normal; -webkit-text-stroke-width: 0; text-shadow: none; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
}
</style>
