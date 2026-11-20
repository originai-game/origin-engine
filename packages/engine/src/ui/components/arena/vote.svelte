<meta name="component" content="ArenaVote">
<meta name="strings" content="./vote.en.ogt">
<meta name="prop:title" content="string" data-doc="Heading; empty uses Map vote.">
<meta name="prop:detail" content="string" data-doc="Voting rules or a disabled reason, supplied by the game.">
<meta name="prop:timeText" content="string" data-doc="Game-owned time remaining; empty hides it. Hidden once closed.">
<meta name="prop:options" content="list" data-doc="Keyed { id, name, icon, detail, votes, disabled, reason } options. choose reports the stable id as rowKey, never the index. The game counts votes and decides winners.">
<meta name="prop:selected" content="string" data-doc="The local vote's option id; empty means no vote. A click reports choose but does not change it locally.">
<meta name="prop:winner" content="string" data-doc="The winning option id, supplied by the game. Shown only when closed; empty means no announced result.">
<meta name="prop:canVote" content="bool" data-default="true" data-doc="Whether the local player can vote or change a vote.">
<meta name="prop:closed" content="bool" data-doc="Voting is over; options remain readable but disabled.">
<meta name="prop:loading" content="bool" data-doc="Waiting for options; hides stale options and disables voting.">
<meta name="fixture:default" content='{"title":"","detail":"Choose the next map. You can change your vote.","timeText":"18s","options":[{"id":"foundry","name":"Foundry","icon":"fire","detail":"Close quarters · 2 v 2","votes":3,"disabled":false,"reason":""},{"id":"grove","name":"Moonlit Grove","icon":"leaf","detail":"Open lanes · 2 v 2","votes":1,"disabled":false,"reason":""},{"id":"citadel","name":"Citadel","icon":"crown","detail":"Tournament map","votes":0,"disabled":true,"reason":"Not installed"}],"selected":"foundry","winner":"","canVote":true,"closed":false,"loading":false}'>
<meta name="fixture:closed" content='{"closed":true,"winner":"grove","detail":"Moonlit Grove won the vote.","options":[{"id":"foundry","name":"Foundry","icon":"fire","detail":"Close quarters · 2 v 2","votes":3,"disabled":false,"reason":""},{"id":"grove","name":"Moonlit Grove","icon":"leaf","detail":"Open lanes · 2 v 2","votes":5,"disabled":false,"reason":""},{"id":"citadel","name":"Citadel","icon":"crown","detail":"Tournament map","votes":0,"disabled":true,"reason":"Not installed"}]}'>
<meta name="fixture:disabled" content='{"canVote":false,"detail":"Spectators cannot vote","selected":""}'>
<meta name="fixture:loading" content='{"loading":true}'>
<meta name="fixture:empty" content='{"options":[],"selected":"","detail":"Waiting for the host to choose maps"}'>
<meta name="fixture:overflow" content='{"title":"Championship Series · Deciding map selection","options":[{"id":"isles","name":"The Shattered Isles of the Unforgotten Ancients","icon":"map","detail":"An extended description that must remain readable without pushing a vote count off the phone","votes":1234567,"disabled":false,"reason":""},{"id":"grove","name":"Moonlit Grove","icon":"leaf","detail":"Open lanes","votes":99,"disabled":false,"reason":""}],"selected":"isles"}'>

<section class="frame" id="ArenaVote" part="frame" layout="stack" fill aria-label={title || t("vote.title")} aria-busy={loading}>
  <h2 class="title" part="title">{title || t("vote.title")}</h2>
  {#if detail}<p class="detail" part="caption">{detail}</p>{/if}
  {#if closed}<div class="state" part="caption" role="status">{t("vote.closed")}</div>{:else}{#if timeText}<div class="timer" part="figure">{timeText}</div>{/if}{/if}
  {#if loading}<p class="empty" part="caption" role="status">{t("vote.loading")}</p>
  {:else}{#if options.length}
    <div class="scroll-area" layout="stack" gap="none">
    <div class="options" part="options" layout="scroll" role="group" aria-label={t("vote.options")}>
      {#each options as o (o.id)}
        <button class="option" id="Choose" part="option" aria-pressed={selected == o.id} disabled={!canVote || closed || o.disabled} onclick={choose} state:selected={selected == o.id}>
          <div class="option-body" layout="wrap">
          {#if o.icon}<og-icon class="icon" part="icon" icon={o.icon}></og-icon>{/if}
          <div class="copy" layout="stack" gap="hair" fill>
            <div class="name" part="headline">{o.name}</div>
            {#if o.detail}<div class="description" part="caption">{o.detail}</div>{/if}
            {#if selected == o.id}<div class="state" part="caption">{t("vote.selected")}</div>{/if}
            {#if closed && winner == o.id}<div class="state" part="caption" tone="success" class:tone-success={true}>{t("vote.winner")}</div>{/if}
            {#if o.disabled && o.reason}<div class="state" part="caption">{o.reason}</div>{/if}
          </div>
          <div class="tally" part="figure">{t("vote.tally", { count: formatNumber(o.votes) })}</div>
          </div>
        </button>
      {/each}
    </div>
    <div class="scroll-fade" part="fade" anchor="bottom-edge"></div>
    </div>
  {:else}<p class="empty" part="caption" role="status">{t("vote.empty")}</p>{/if}{/if}
</section>
<style>
/* The vote on a smoked cut card: the clock in gold, an option a cut tile
   of dark glass with its tally at the end, the player's own ringed in
   the accent and washed in it. */
#ArenaVote { inline-size: stretch; block-size: auto; max-block-size: stretch; color: var(--hud-ink); --fade-fill: var(--party-open-fill); }
.frame { box-sizing: border-box; padding: var(--space-lg); border: 0; color: var(--hud-ink); background: var(--party-card-fill); box-shadow: var(--party-card-rim); border-radius: var(--party-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.title { margin: 0; min-inline-size: 0; font-size: var(--hud-size-md); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-title-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); text-shadow: none; white-space: normal; overflow-wrap: break-word; }
.detail, .empty { margin: 0; font-size: var(--hud-size-xs); font-weight: var(--hud-label-weight); color: var(--hud-ink-dim); overflow-wrap: break-word; }
.title, .detail, .timer { flex: none; }
.timer { font-size: var(--hud-size-time); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-gold); text-shadow: none; }
.scroll-area { position: relative; min-block-size: 0; }
.options { min-block-size: 0; padding-block-end: var(--space-lg); }
.option { flex: none; min-block-size: var(--size-touch); padding: var(--space-sm) var(--space-md); text-align: start; border: 0; color: var(--hud-ink); background: var(--party-tile-fill); box-shadow: var(--party-tile-rim); border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.option:hover { box-shadow: var(--party-tile-hover-rim); }
.option:focus-visible { box-shadow: var(--party-focus-rim); }
.option:state(selected) { background: var(--party-tile-on-fill); box-shadow: var(--party-on-rim); }
.option:disabled { color: var(--hud-ink-dim); }
.option-body { inline-size: stretch; }
.icon { flex: none; inline-size: var(--size-sm); block-size: var(--size-sm); color: inherit; }
.copy { min-inline-size: 0; }
.name { font-size: var(--hud-size-readout); line-height: 1.15; font-weight: var(--hud-figure-weight); color: inherit; overflow-wrap: break-word; }
.description { font-size: var(--hud-size-xs); color: var(--hud-ink-dim); overflow-wrap: break-word; }
.state { font-size: var(--hud-tag-size); line-height: 1.5; font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-accent); overflow-wrap: break-word; }
.state.tone-success { color: var(--party-ready); }
.option:disabled .state { color: var(--hud-ink-dim); }
#ArenaVote section .state { color: var(--hud-ink-dim); }
.tally { flex: 0 1 auto; font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: inherit; text-shadow: none; overflow-wrap: break-word; }
.frame:size(compact) { padding: var(--space-sm); }
.option:size(compact) { padding: var(--space-xs) var(--space-sm); }
.tally:size(compact) { flex-basis: 100%; text-align: end; }
</style>
