<meta name="component" content="ArenaReady">
<meta name="strings" content="./ready.en.ogt">
<meta name="parts" content="ArenaReady.frame ArenaReady.title ArenaReady.detail ArenaReady.timer ArenaReady.players ArenaReady.row ArenaReady.name ArenaReady.state ArenaReady.empty ArenaReady.actions ArenaReady.accept ArenaReady.decline ArenaReady.caption ArenaReady.headline ArenaReady.figure ArenaReady.fade">
<meta name="prop:title" content="string" data-doc="Heading; empty uses Ready check.">
<meta name="prop:detail" content="string" data-doc="Match or reason line; empty hides it.">
<meta name="prop:timeText" content="string" data-doc="Game-owned time remaining; empty hides it. This component never starts or expires a timer.">
<meta name="prop:players" content="list" data-doc="Keyed { id, name, state } rows. state: ready, declined, offline, or pending (empty). Order belongs to the game.">
<meta name="prop:response" content="string" data-doc="Local response: ready, declined or pending (empty). Both buttons remain available to change a response while canRespond.">
<meta name="prop:canRespond" content="bool" data-default="true" data-doc="The game allows a local response. False disables both actions.">
<meta name="prop:closed" content="bool" data-doc="The check has ended; no more responses. timeText is hidden.">
<meta name="prop:loading" content="bool" data-doc="Waiting for participants; stale rows are hidden and responses disabled.">
<meta name="fixture:default" content='{"title":"","detail":"Ranked · Foundry","timeText":"12s","players":[{"id":"rhea","name":"Rhea","state":"ready"},{"id":"kai","name":"Kai","state":"pending"},{"id":"ana","name":"Ana","state":"declined"}],"response":"","canRespond":true,"closed":false,"loading":false}'>
<meta name="fixture:selected" content='{"players":[{"id":"rhea","name":"Rhea","state":"ready"},{"id":"kai","name":"Kai","state":"ready"}],"response":"ready"}'>
<meta name="fixture:disabled" content='{"canRespond":false,"detail":"Only joined players can respond"}'>
<meta name="fixture:closed" content='{"closed":true,"detail":"One player declined. Returning to queue.","response":"declined"}'>
<meta name="fixture:loading" content='{"loading":true}'>
<meta name="fixture:empty" content='{"players":[],"detail":"Waiting for participants"}'>
<meta name="fixture:overflow" content='{"title":"Championship Series · Grand Final ready check","detail":"A response is needed from every player before the tournament can continue","players":[{"id":"long","name":"Maximilian Vandenberghe-Okonkwo","state":"pending"},{"id":"offline","name":"Abernathy-Whitcombe","state":"offline"},{"id":"rhea","name":"Rhea","state":"ready"},{"id":"kai","name":"Kai","state":"declined"}]}'>

<!-- The game owns readiness, timeout and acceptance. Actions report accept
     or decline from the stable component mount; no local readiness is faked. -->
<section id="ArenaReady" part="ArenaReady.frame" data-layout="stack" aria-label={title || t("ready.title")} aria-busy={loading}>
  <h2 part="ArenaReady.title">{title || t("ready.title")}</h2>
  {#if detail}<p part="ArenaReady.detail ArenaReady.caption">{detail}</p>{/if}
  {#if closed}<div part="ArenaReady.state ArenaReady.headline" role="status">{t("ready.closed")}</div>{:else}{#if timeText}<div part="ArenaReady.timer ArenaReady.figure">{timeText}</div>{/if}{/if}
  {#if loading}<p part="ArenaReady.empty ArenaReady.caption" role="status">{t("ready.loading")}</p>
  {:else}{#if players.length}
    <div class="scroll-area" data-layout="stack" data-gap="none">
    <div part="ArenaReady.players" data-layout="scroll" data-gap="tight" role="list" aria-label={t("ready.players")}>
      {#each players as p (p.id)}
        <div part="ArenaReady.row" data-layout="cluster" data-gap="tight" role="listitem" data-state={p.state}>
          <div part="ArenaReady.name" data-fill>{p.name}</div>
          <div part="ArenaReady.state ArenaReady.headline" data-tone={p.state == 'ready' ? 'success' : p.state == 'declined' ? 'danger' : ''}>{p.state == 'ready' ? t("ready.accepted") : p.state == 'declined' ? t("ready.declined") : p.state == 'offline' ? t("ready.offline") : t("ready.pending")}</div>
        </div>
      {/each}
    </div>
    <div class="scroll-fade" part="ArenaReady.fade" data-anchor="bottom-edge"></div>
    </div>
  {:else}<p part="ArenaReady.empty ArenaReady.caption" role="status">{t("ready.empty")}</p>{/if}{/if}
  <footer part="ArenaReady.actions" data-layout="cluster">
    <button id="Accept" part="ArenaReady.accept" variant="primary" aria-pressed={response == 'ready'} disabled={!canRespond || closed || loading || !players.length} onclick={accept}>{t("ready.accept")}</button>
    <button id="Decline" part="ArenaReady.decline" aria-pressed={response == 'declined'} disabled={!canRespond || closed || loading || !players.length} onclick={decline}>{t("ready.decline")}</button>
  </footer>
</section>
<style>
/* The ready check on a smoked cut card: the clock in gold, a row per
   player with its answer as a word, its leading edge green once ready
   and red when refused; Accept is green. */
:host { inline-size: stretch; max-block-size: stretch; color: var(--hud-ink); --fade-fill: var(--party-open-fill); }
[part="ArenaReady.frame"] { box-sizing: border-box; padding: var(--space-lg); border: 0; border-radius: 0; color: var(--hud-ink); background: var(--party-card-fill); box-shadow: var(--party-card-rim); clip-path: polygon(var(--party-cut) 0, 100% 0, 100% calc(100% - var(--party-cut)), calc(100% - var(--party-cut)) 100%, 0 100%, 0 var(--party-cut)); }
[part="ArenaReady.title"] { margin: 0; min-inline-size: 0; font-size: var(--hud-size-md); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-title-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); text-shadow: none; white-space: normal; overflow-wrap: break-word; text-align: center; }
[part~="ArenaReady.detail"], [part~="ArenaReady.empty"] { margin: 0; text-align: center; font-size: var(--hud-size-xs); font-weight: var(--hud-label-weight); color: var(--hud-ink-dim); overflow-wrap: break-word; }
[part="ArenaReady.title"], [part~="ArenaReady.detail"], [part~="ArenaReady.timer"], [part="ArenaReady.actions"] { flex: none; }
[part~="ArenaReady.timer"] { text-align: center; font-size: var(--party-size-clock); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-gold); text-shadow: none; }
[part="ArenaReady.row"] [part="ArenaReady.state ArenaReady.headline"] { margin-inline-start: auto; }
[part="ArenaReady.state ArenaReady.headline"] { font-size: var(--hud-size-xs); line-height: 1.3; font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink-dim); text-align: center; }
.scroll-area { position: relative; min-block-size: 0; }
[part="ArenaReady.players"] { min-block-size: 0; padding-block-end: var(--space-lg); }
[part="ArenaReady.row"] { justify-content: space-between; min-block-size: var(--size-md); padding: var(--space-xs) var(--space-md); border-radius: 0; color: var(--hud-ink); background: var(--result-row-fill); box-shadow: var(--party-state-edge); }
[part="ArenaReady.row"][data-state=ready] { box-shadow: var(--party-ready-edge); }
[part="ArenaReady.row"][data-state=declined] { box-shadow: var(--party-refused-edge); }
/* The name keeps its words whole: when its answer does not fit beside it, the answer takes the line under, at its end. */
[part="ArenaReady.name"] { flex: 1 1 auto; min-inline-size: 0; font-size: var(--hud-size-name); font-weight: var(--hud-label-weight); color: var(--hud-ink); overflow-wrap: break-word; }
[data-state=offline] [part="ArenaReady.name"] { color: var(--hud-ink-dim); }
[data-state=ready] [part="ArenaReady.state ArenaReady.headline"] { color: var(--party-ready); }
[data-state=declined] [part="ArenaReady.state ArenaReady.headline"] { color: var(--party-refused); }
[part="ArenaReady.actions"] button { flex: 1 1 auto; min-height: var(--size-touch); gap: var(--space-xs); padding: 0 var(--space-lg); border: 0; border-radius: 0; font-size: var(--hud-size-xs); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); background: var(--party-button-fill); box-shadow: var(--party-button-rim); text-shadow: none; clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); }
[part="ArenaReady.actions"] button:hover { background: var(--party-button-hover-fill); }
[part="ArenaReady.actions"] button:focus-visible { box-shadow: var(--party-focus-rim); }
[part="ArenaReady.actions"] #Accept { color: var(--party-ready-ink); background: var(--party-ready-fill); box-shadow: none; }
[part="ArenaReady.actions"] button[aria-pressed=true] { box-shadow: var(--party-on-rim); }
[part="ArenaReady.actions"] button:disabled { opacity: 0.45; }
</style>
