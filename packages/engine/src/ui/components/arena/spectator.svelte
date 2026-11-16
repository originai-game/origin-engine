<meta name="component" content="ArenaSpectator">
<meta name="strings" content="./spectator.en.ogt">
<meta name="parts" content="ArenaSpectator.frame ArenaSpectator.title ArenaSpectator.detail ArenaSpectator.mode ArenaSpectator.targets ArenaSpectator.target ArenaSpectator.name ArenaSpectator.team ArenaSpectator.state ArenaSpectator.empty ArenaSpectator.actions ArenaSpectator.previous ArenaSpectator.next ArenaSpectator.free ArenaSpectator.leave">
<meta name="prop:title" content="string" data-doc="Heading; empty uses Spectating.">
<meta name="prop:detail" content="string" data-doc="Match status, broadcast delay or disabled reason, as the game words it.">
<meta name="prop:targets" content="list" data-doc="Keyed { id, name, team, detail, disabled } subjects. follow reports their stable id as rowKey. detail can say Eliminated, Respawning or Disconnected; the game decides which subjects can be followed.">
<meta name="prop:selected" content="string" data-doc="The followed subject id. Empty means no chosen subject; freeCamera takes precedence over its selected state.">
<meta name="prop:freeCamera" content="bool" data-doc="Free camera active; choosing a subject only reports follow, leaving the game to change camera mode.">
<meta name="prop:canFree" content="bool" data-default="true" data-doc="Show and enable the free camera action; false omits it.">
<meta name="prop:canCycle" content="bool" data-default="true" data-doc="Enable previous and next. The game owns cycling order and wrapping.">
<meta name="prop:canLeave" content="bool" data-default="true" data-doc="Enable Leave; false keeps the exit visible but disabled.">
<meta name="prop:loading" content="bool" data-doc="Waiting for subjects; hides stale rows and disables camera actions, but not Leave.">
<meta name="fixture:default" content='{"title":"","detail":"Ranked · Round 3 · Broadcast delay 10s","targets":[{"id":"rhea","name":"Rhea","team":"Blue","detail":"Alive","disabled":false},{"id":"kai","name":"Kai","team":"Red","detail":"Respawning · 4s","disabled":false},{"id":"ana","name":"Ana","team":"Blue","detail":"Disconnected","disabled":true}],"selected":"kai","freeCamera":false,"canFree":true,"canCycle":true,"canLeave":true,"loading":false}'>
<meta name="fixture:free" content='{"freeCamera":true}'>
<meta name="fixture:disabled" content='{"canFree":false,"canCycle":false,"canLeave":false,"detail":"Camera controls locked during replay transition"}'>
<meta name="fixture:loading" content='{"loading":true}'>
<meta name="fixture:empty" content='{"targets":[],"selected":"","detail":"Waiting for players to join"}'>
<meta name="fixture:overflow" content='{"title":"Championship Series · Grand Final spectator view","targets":[{"id":"long","name":"Maximilian Vandenberghe-Okonkwo","team":"The Northern Champions","detail":"Respawning at the eastern forward base in 12 seconds","disabled":false},{"id":"rhea","name":"Rhea","team":"Blue","detail":"Alive","disabled":false},{"id":"kai","name":"Kai","team":"Red","detail":"Alive","disabled":false}],"selected":"long"}'>

<section id="ArenaSpectator" part="ArenaSpectator.frame" class="hud" aria-label={title || t("spectator.title")} aria-busy={loading}>
  <h2 part="ArenaSpectator.title">{title || t("spectator.title")}</h2>
  {#if detail}<p part="ArenaSpectator.detail">{detail}</p>{/if}
  <div part="ArenaSpectator.mode" role="status">{freeCamera ? t("spectator.free") : selected ? t("spectator.following") : t("spectator.choose")}</div>
  {#if loading}<p part="ArenaSpectator.empty" role="status">{t("spectator.loading")}</p>
  {:else}{#if targets.length}
    <div class="scroll-area">
    <div part="ArenaSpectator.targets" role="group" aria-label={t("spectator.targets")}>
      {#each targets as p (p.id)}
        <button id="Follow" part="ArenaSpectator.target" aria-pressed={!freeCamera && selected == p.id} disabled={p.disabled} onclick={follow}>
          <div class="target-body">
            <div part="ArenaSpectator.name">{p.name}</div>
            {#if p.team}<div part="ArenaSpectator.team">{p.team}</div>{/if}
            {#if p.detail}<div part="ArenaSpectator.state">{p.detail}</div>{/if}
            {#if !freeCamera && selected == p.id}<div part="ArenaSpectator.state">{t("spectator.current")}</div>{/if}
          </div>
        </button>
      {/each}
    </div>
    <div class="scroll-fade"></div>
    </div>
  {:else}<p part="ArenaSpectator.empty" role="status">{t("spectator.empty")}</p>{/if}{/if}
  <footer part="ArenaSpectator.actions">
    <button id="Previous" part="ArenaSpectator.previous" disabled={loading || !canCycle || !targets.length} onclick={previous}>{t("spectator.previous")}</button>
    <button id="Next" part="ArenaSpectator.next" disabled={loading || !canCycle || !targets.length} onclick={next}>{t("spectator.next")}</button>
    {#if canFree}<button id="Free" part="ArenaSpectator.free" aria-pressed={freeCamera} disabled={loading} onclick={free}>{t("spectator.free")}</button>{/if}
    <button id="Leave" part="ArenaSpectator.leave" variant="ghost" disabled={!canLeave} onclick={leave}>{t("spectator.leave")}</button>
  </footer>
</section>
<style>
/* A spectator HUD may be placed over the scene. Filled roles stop the halo. */
:host { text-shadow: var(--tag-halo); display: flex; flex-direction: column; inline-size: stretch; max-block-size: stretch; gap: var(--space-sm); }
[part="ArenaSpectator.title"], [part="ArenaSpectator.detail"], [part="ArenaSpectator.empty"] { margin: 0; overflow-wrap: anywhere; }
[part="ArenaSpectator.title"], [part="ArenaSpectator.detail"], [part="ArenaSpectator.mode"], [part="ArenaSpectator.actions"] { flex: none; }
[part="ArenaSpectator.detail"], [part="ArenaSpectator.empty"] { color: var(--ink-muted); }
[part="ArenaSpectator.mode"] { font-weight: var(--font-weight-semibold); }
.scroll-area { display: flex; flex-direction: column; position: relative; min-block-size: 0; }
[part="ArenaSpectator.targets"] { display: flex; flex-direction: column; min-block-size: 0; overflow-y: auto; gap: var(--space-xs); padding-block-end: var(--space-lg); }
.scroll-fade { position: absolute; inset-inline: 0; bottom: 0; block-size: var(--space-lg); pointer-events: none; background: linear-gradient(transparent, var(--panel)); }
[part="ArenaSpectator.target"] { flex: none; min-block-size: var(--size-touch); text-align: start; }
.target-body { display: flex; flex-direction: column; inline-size: stretch; gap: var(--space-xxs); }
[part="ArenaSpectator.name"] { font-weight: var(--font-weight-semibold); }
[part="ArenaSpectator.name"], [part="ArenaSpectator.team"], [part="ArenaSpectator.state"] { overflow-wrap: anywhere; }
[part="ArenaSpectator.team"], [part="ArenaSpectator.state"] { font: var(--text-sm); }
[part="ArenaSpectator.actions"] { display: flex; flex-wrap: wrap; gap: var(--space-sm); }
[part="ArenaSpectator.actions"] button { flex: 1 1 auto; min-block-size: var(--size-touch); }
</style>
