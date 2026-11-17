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
:host { inline-size: stretch; max-block-size: stretch; --headline-text: var(--text-sm); }
[part="ArenaReady.title"], [part~="ArenaReady.detail"], [part~="ArenaReady.empty"] { margin: 0; overflow-wrap: anywhere; }
[part="ArenaReady.title"], [part~="ArenaReady.detail"], [part~="ArenaReady.timer"], [part="ArenaReady.actions"] { flex: none; }
[part~="ArenaReady.timer"] { --figure-weight: var(--font-weight-semibold); }
.scroll-area { position: relative; min-block-size: 0; }
[part="ArenaReady.players"] { min-block-size: 0; padding-block-end: var(--space-lg); }
[part="ArenaReady.row"] { padding: var(--space-sm); }
[part="ArenaReady.name"] { overflow-wrap: anywhere; }
[part="ArenaReady.actions"] button { flex: 1 1 auto; min-block-size: var(--size-touch); }
</style>
