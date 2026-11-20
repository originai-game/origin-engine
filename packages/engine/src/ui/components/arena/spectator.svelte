<meta name="component" content="ArenaSpectator">
<meta name="strings" content="./spectator.en.ogt">
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

<section class="frame" id="ArenaSpectator" part="frame" variant="hud" layout="stack" aria-label={title || t("spectator.title")} aria-busy={loading}>
  <h2 class="title" part="title">{title || t("spectator.title")}</h2>
  {#if detail}<p class="detail" part="caption">{detail}</p>{/if}
  <div class="mode" part="headline" role="status">{freeCamera ? t("spectator.free") : selected ? t("spectator.following") : t("spectator.choose")}</div>
  {#if loading}<p class="empty" part="caption" role="status">{t("spectator.loading")}</p>
  {:else}{#if targets.length}
    <div class="scroll-area" layout="stack" gap="none">
    <div class="targets" part="targets" layout="scroll" gap="tight" role="group" aria-label={t("spectator.targets")}>
      {#each targets as p (p.id)}
        <button class="target" id="Follow" part="target" aria-pressed={!freeCamera && selected == p.id} disabled={p.disabled} onclick={follow} state:selected={!freeCamera && selected == p.id}>
          <div class="target-body" layout="stack" gap="hair">
            <div class="name" part="headline">{p.name}</div>
            {#if p.team}<div class="team" part="caption">{p.team}</div>{/if}
            {#if p.detail}<div class="state" part="caption">{p.detail}</div>{/if}
            {#if !freeCamera && selected == p.id}<div class="state" part="caption">{t("spectator.current")}</div>{/if}
          </div>
        </button>
      {/each}
    </div>
    <div class="scroll-fade" part="fade" anchor="bottom-edge"></div>
    </div>
  {:else}<p class="empty" part="caption" role="status">{t("spectator.empty")}</p>{/if}{/if}
  <footer class="actions" part="actions" layout="wrap">
    <button id="Previous" part="previous" disabled={loading || !canCycle || !targets.length} onclick={previous}>{t("spectator.previous")}</button>
    <button id="Next" part="next" disabled={loading || !canCycle || !targets.length} onclick={next}>{t("spectator.next")}</button>
    {#if canFree}<button id="Free" part="free" aria-pressed={freeCamera} disabled={loading} onclick={free} state:selected={freeCamera}>{t("spectator.free")}</button>{/if}
    <button id="Leave" part="leave" variant="ghost" disabled={!canLeave} onclick={leave}>{t("spectator.leave")}</button>
  </footer>
</section>
<style>
/* The spectator's plate over the match: smoked glass with a cut corner,
   the mode in condensed capitals, a row per player with a hairline edge,
   the one being watched washed in the accent with its edge lit. */
#ArenaSpectator { inline-size: stretch; max-block-size: stretch; color: var(--hud-ink); --fade-fill: var(--party-open-fill); }
.frame { box-sizing: border-box; padding: var(--space-md); border: 0; color: var(--hud-ink); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); border-radius: var(--party-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.title { margin: 0; font-size: var(--hud-label-size); line-height: 1.3; font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); overflow-wrap: break-word; text-shadow: none; }
.detail, .empty { margin: 0; font-size: var(--hud-size-xs); font-weight: var(--hud-label-weight); color: var(--hud-ink-dim); overflow-wrap: break-word; }
.title, .detail, .mode, .actions { flex: none; }
.mode { font-size: var(--hud-size-md); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-title-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); overflow-wrap: break-word; }
.scroll-area { position: relative; min-block-size: 0; }
.targets { min-block-size: 0; padding-block-end: var(--space-lg); }
.target { flex: none; min-block-size: var(--size-touch); text-align: start; padding: var(--space-xs) var(--space-md); border: 0; border-radius: 0; color: var(--hud-ink); background: var(--result-row-fill); box-shadow: var(--party-state-edge); }
.target:hover { background: var(--party-button-hover-fill); }
.target:focus-visible { box-shadow: var(--party-focus-rim); }
.target:state(selected) { background: var(--party-on-row); box-shadow: var(--party-on-edge); }
.target:disabled { color: var(--hud-ink-dim); }
.target-body { inline-size: stretch; }
.name { font-size: var(--hud-size-name); line-height: 1.15; font-weight: var(--hud-figure-weight); color: inherit; overflow-wrap: break-word; }
.team { font-size: var(--hud-label-size); line-height: 1.3; font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); overflow-wrap: break-word; }
.state { font-size: var(--hud-size-xs); color: var(--hud-ink-dim); overflow-wrap: break-word; }
.actions button { flex: 1 1 auto; min-height: var(--size-touch); gap: var(--space-xs); padding: 0 var(--space-lg); border: 0; font-size: var(--hud-size-xs); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); background: var(--party-button-fill); box-shadow: var(--party-button-rim); text-shadow: none; border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; --button-radius: var(--hud-plate-cut) 0; --button-corner-shape: var(--cut-shape); padding: 0 var(--space-md); }
.actions button:hover { background: var(--party-button-hover-fill); }
.actions button:focus-visible { box-shadow: var(--party-focus-rim); }
.actions button:state(selected) { box-shadow: var(--party-on-rim); }
.actions button:disabled { opacity: 0.45; }
</style>
