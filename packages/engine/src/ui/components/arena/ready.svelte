<meta name="component" content="ArenaReady">
<meta name="strings" content="./ready.en.ogt">
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
<section class="frame" id="ArenaReady" part="frame" layout="stack" aria-label={title || t("ready.title")} aria-busy={loading}>
  <h2 class="title" part="title">{title || t("ready.title")}</h2>
  {#if detail}<p class="detail" part="caption">{detail}</p>{/if}
  {#if closed}<div class="state" part="headline" role="status">{t("ready.closed")}</div>{:else}{#if timeText}<div class="timer" part="figure">{timeText}</div>{/if}{/if}
  {#if loading}<p class="empty" part="caption" role="status">{t("ready.loading")}</p>
  {:else}{#if players.length}
    <div class="scroll-area" layout="stack" gap="none">
    <div class="players" part="players" layout="scroll" gap="tight" role="list" aria-label={t("ready.players")}>
      {#each players as p (p.id)}
        <div class="row" part="row" layout="wrap" gap="tight" role="listitem" class:state-declined={(p.state) == 'declined'} class:state-offline={(p.state) == 'offline'} class:state-ready={(p.state) == 'ready'}>
          <div class="name" part="name" fill>{p.name}</div>
          <div class="state" part="headline" tone={p.state == 'ready' ? 'success' : p.state == 'declined' ? 'danger' : ''}>{p.state == 'ready' ? t("ready.accepted") : p.state == 'declined' ? t("ready.declined") : p.state == 'offline' ? t("ready.offline") : t("ready.pending")}</div>
        </div>
      {/each}
    </div>
    <div class="scroll-fade" part="fade" anchor="bottom-edge"></div>
    </div>
  {:else}<p class="empty" part="caption" role="status">{t("ready.empty")}</p>{/if}{/if}
  <footer class="actions" part="actions" layout="wrap">
    <button id="Accept" tone="confirm" class="go" part="accept" variant="primary" aria-pressed={response == 'ready'} disabled={!canRespond || closed || loading || !players.length} onclick={accept} state:selected={response == 'ready'}>{t("ready.accept")}</button>
    <button id="Decline" part="decline" aria-pressed={response == 'declined'} disabled={!canRespond || closed || loading || !players.length} onclick={decline} state:selected={response == 'declined'}>{t("ready.decline")}</button>
  </footer>
</section>
<style>
/* The ready check on a smoked cut card: the clock in gold, a row per
   player with its answer as a word, its leading edge green once ready
   and red when refused; Accept is green. */
#ArenaReady { inline-size: stretch; max-block-size: stretch; color: var(--hud-ink); --fade-fill: var(--party-open-fill); }
.frame { box-sizing: border-box; padding: var(--space-lg); border: 0; color: var(--hud-ink); background: var(--party-card-fill); box-shadow: var(--party-card-rim); border-radius: var(--party-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.title { margin: 0; min-inline-size: 0; font-size: var(--hud-size-md); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-title-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); text-shadow: none; white-space: normal; overflow-wrap: break-word; text-align: center; }
.detail, .empty { margin: 0; text-align: center; font-size: var(--hud-size-xs); font-weight: var(--hud-label-weight); color: var(--hud-ink-dim); overflow-wrap: break-word; }
.title, .detail, .timer, .actions { flex: none; }
.timer { text-align: center; font-size: var(--party-size-clock); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-gold); text-shadow: none; }
.row .state { margin-inline-start: auto; }
.state { font-size: var(--hud-size-xs); line-height: 1.3; font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink-dim); text-align: center; }
.scroll-area { position: relative; min-block-size: 0; }
.players { min-block-size: 0; padding-block-end: var(--space-lg); }
.row { justify-content: space-between; min-block-size: var(--size-md); padding: var(--space-xs) var(--space-md); border-radius: 0; color: var(--hud-ink); background: var(--result-row-fill); box-shadow: var(--party-state-edge); }
.row.state-ready { box-shadow: var(--party-ready-edge); }
.row.state-declined { box-shadow: var(--party-refused-edge); }
/* The name keeps its words whole: when its answer does not fit beside it, the answer takes the line under, at its end. */
.name { flex: 1 1 auto; min-inline-size: 0; font-size: var(--hud-size-name); font-weight: var(--hud-label-weight); color: var(--hud-ink); overflow-wrap: break-word; }
.state-offline .name { color: var(--hud-ink-dim); }
.state-ready .state { color: var(--party-ready); }
.state-declined .state { color: var(--party-refused); }
.actions button { flex: 1 1 auto; min-height: var(--size-touch); gap: var(--space-xs); padding: 0 var(--space-lg); border: 0; font-size: var(--hud-size-xs); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); background: var(--party-button-fill); box-shadow: var(--party-button-rim); text-shadow: none; border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; --button-radius: var(--hud-plate-cut) 0; --button-corner-shape: var(--cut-shape); }
.actions button:hover { background: var(--party-button-hover-fill); }
.actions button:focus-visible { box-shadow: var(--party-focus-rim); }
.actions button:state(selected) { box-shadow: var(--party-on-rim); }
.actions button:disabled { opacity: 0.45; }
</style>
