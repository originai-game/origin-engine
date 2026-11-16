<meta name="component" content="RpgTurnOrder">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgTurnOrder.frame RpgTurnOrder.cell RpgTurnOrder.title">
<meta name="prop:round" content="number" data-doc="The round, on the plate under the portraits.">
<meta name="prop:turns" content="list" data-doc="Keyed rows of { id, name, portrait, side, acting, down }, in the order they act: side ally, enemy or neutral; acting the one whose turn it is; portrait a texture, 0 none.">
<meta name="fixture:default" content='{"round":3,"turns":[{"id":"a","name":"Aerin","portrait":0,"side":"ally","acting":true,"down":false},{"id":"o","name":"Oarsman","portrait":0,"side":"enemy","acting":false,"down":false}]}'>

<!--
  The initiative: round portraits in acting order, the one acting first and
  ringed, between two end flourishes, and a round plate on a rule under
  them. All portraits share the width, so five fit a phone. Reports
  nothing.
-->
<section id="RpgTurnOrder" part="RpgTurnOrder.frame" class="bare" aria-label={t("rpg.turns.name")}>
  <div class="track">
    <div class="end start"></div>
    <ol class="turns">
      {#each turns as u (u.id)}
        <li part="RpgTurnOrder.cell" data-side={u.side} data-acting={u.acting} data-down={u.down} aria-label={u.name} aria-current={u.acting}>
          <div class="ring">{#if u.portrait}<img class="portrait" src={u.portrait} alt="">{/if}</div>
        </li>
      {/each}
    </ol>
    <div class="end finish"></div>
  </div>
  <div class="round-rule"><div class="rpg-rule" role="separator"></div><div part="RpgTurnOrder.title">{t("rpg.turns.round", { round: round })}</div><div class="rpg-rule" role="separator"></div></div>
</section>

<style>
/* Drawn over the scene: the style's tag halo holds its text there. */
:host { text-shadow: var(--tag-halo); }
:host { display: flex; flex-direction: column; inline-size: stretch; gap: var(--space-xs); align-items: center; }
.track { display: flex; width: stretch; align-items: center; gap: var(--space-xs); }
.end { width: var(--size-sm); height: var(--size-md); flex: none; }
.turns { display: grid; grid-auto-flow: column; grid-auto-columns: 1fr; gap: var(--space-xs); flex: 1; min-width: 0; margin: 0; padding: var(--space-xs) 0; list-style: none; }
[part="RpgTurnOrder.cell"] { display: flex; flex-direction: column; align-items: center; padding: var(--space-xs); background: var(--scene-plate); border-radius: var(--radius-card); }
[part="RpgTurnOrder.cell"][data-acting=true] { border-color: var(--focus); box-shadow: inset 0 0 0 var(--stroke-regular) var(--focus); }
[part="RpgTurnOrder.cell"][data-down=true] { opacity: var(--state-disabled-opacity); }
.ring { width: stretch; max-width: var(--size-xxl); aspect-ratio: 1; overflow: hidden; border-radius: var(--radius-capsule); }
.portrait { width: stretch; height: stretch; object-fit: cover; object-position: center top; }
.round-rule { display: flex; width: stretch; align-items: center; gap: var(--space-sm); }
.round-rule .rpg-rule { width: stretch; flex: 1; }
[part="RpgTurnOrder.title"] { flex: none; padding: var(--space-xs) var(--space-xl); white-space: nowrap; background: var(--scene-plate); color: var(--scene-ink); border-radius: var(--radius-capsule); }
@media (height < 500px) { .ring { max-width: var(--size-xl); } .turns { padding: 0; } [part="RpgTurnOrder.title"] { padding: 0 var(--space-lg); } }
</style>
