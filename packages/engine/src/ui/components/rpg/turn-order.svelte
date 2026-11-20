<meta name="component" content="RpgTurnOrder">
<meta name="strings" content="./rpg.en.ogt">
<meta name="prop:round" content="number" data-doc="The round, on the plate under the portraits.">
<meta name="prop:turns" content="list" data-doc="Keyed rows of { id, name, portrait, side, acting, down }, in the order they act: side ally, enemy or neutral; acting the one whose turn it is; portrait a texture, 0 none.">
<meta name="fixture:default" content='{"round":3,"turns":[{"id":"a","name":"Aerin","portrait":0,"side":"ally","acting":true,"down":false},{"id":"o","name":"Oarsman","portrait":0,"side":"enemy","acting":false,"down":false}]}'>

<!--
  The initiative: round portraits in acting order, the one acting first and
  ringed, between two end flourishes, and a round plate on a rule under
  them. All portraits share the width, so five fit a phone. Reports
  nothing.
-->
<section id="RpgTurnOrder" part="frame" variant="bare" layout="stack" gap="tight" aria-label={t("rpg.turns.name")}>
  <div class="track" layout="row" gap="tight">
    <div class="end start"></div>
    <ol class="turns grid" gap="tight" fill>
      {#each turns as u (u.id)}
        <li class="cell" part="cell" layout="stack" data-side={u.side} data-acting={u.acting} data-down={u.down} aria-label={u.name} aria-current={u.acting}>
          <div class="ring" part="picture">{#if u.portrait}<img class="portrait" src={u.portrait} alt="">{/if}</div>
        </li>
      {/each}
    </ol>
    <div class="end finish"></div>
  </div>
  <div class="round-rule" layout="row"><div class="rpg-rule" fill role="separator"></div><div part="badge" class="round-title title">{t("rpg.turns.round", { round: round })}</div><div class="rpg-rule" fill role="separator"></div></div>
</section>

<style>
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(var(--layout-tile), 100%), 1fr)); }
/* Drawn over the scene: the style's tag halo holds its text there. */
#RpgTurnOrder { text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
#RpgTurnOrder { inline-size: stretch; align-items: center; }
.track { width: stretch; }
.end { width: var(--size-sm); height: var(--size-md); flex: none; }
.turns { grid-template-columns: none; grid-auto-flow: column; grid-auto-columns: 1fr; margin: 0; padding: var(--space-xs) 0; list-style: none; }
.cell { align-items: center; padding: var(--space-xs); }
.ring { width: stretch; max-width: var(--size-xxl); aspect-ratio: 1; overflow: hidden; padding: calc(2 * var(--font-step-px)); border-radius: 0; clip-path: var(--hud-octagon); background: var(--hud-edge); filter: var(--hud-lift); }
.ring .portrait { clip-path: var(--hud-octagon); background: var(--hud-plate-fill); }
.portrait { width: stretch; height: stretch; object-fit: cover; object-position: center top; }
.round-rule { width: stretch; }
.round-title { flex: none; padding: var(--space-xs) var(--space-xl); white-space: nowrap; --badge-text: var(--text-lg); background: var(--hud-tag-fill); box-shadow: var(--hud-plate-lit-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
@media (height < 500px) { .ring { max-width: var(--size-xl); } .turns { padding: 0; } .round-title { padding: 0 var(--space-lg); } }
</style>
