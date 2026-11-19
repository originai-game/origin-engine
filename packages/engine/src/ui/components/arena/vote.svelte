<meta name="component" content="ArenaVote">
<meta name="strings" content="./vote.en.ogt">
<meta name="parts" content="ArenaVote.frame ArenaVote.title ArenaVote.detail ArenaVote.timer ArenaVote.options ArenaVote.option ArenaVote.icon ArenaVote.name ArenaVote.description ArenaVote.tally ArenaVote.state ArenaVote.empty ArenaVote.caption ArenaVote.headline ArenaVote.figure ArenaVote.fade">
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

<section id="ArenaVote" part="ArenaVote.frame" data-layout="shell" aria-label={title || t("vote.title")} aria-busy={loading}>
  <h2 part="ArenaVote.title">{title || t("vote.title")}</h2>
  {#if detail}<p part="ArenaVote.detail ArenaVote.caption">{detail}</p>{/if}
  {#if closed}<div part="ArenaVote.state ArenaVote.caption" role="status">{t("vote.closed")}</div>{:else}{#if timeText}<div part="ArenaVote.timer ArenaVote.figure">{timeText}</div>{/if}{/if}
  {#if loading}<p part="ArenaVote.empty ArenaVote.caption" role="status">{t("vote.loading")}</p>
  {:else}{#if options.length}
    <div class="scroll-area" data-layout="stack" data-gap="none">
    <div part="ArenaVote.options" data-layout="scroll" role="group" aria-label={t("vote.options")}>
      {#each options as o (o.id)}
        <button id="Choose" part="ArenaVote.option" aria-pressed={selected == o.id} disabled={!canVote || closed || o.disabled} onclick={choose}>
          <div class="option-body" data-layout="cluster">
          {#if o.icon}<og-icon part="ArenaVote.icon" icon={o.icon}></og-icon>{/if}
          <div class="copy" data-layout="stack" data-gap="hair" data-fill>
            <div part="ArenaVote.name ArenaVote.headline">{o.name}</div>
            {#if o.detail}<div part="ArenaVote.description ArenaVote.caption">{o.detail}</div>{/if}
            {#if selected == o.id}<div part="ArenaVote.state ArenaVote.caption">{t("vote.selected")}</div>{/if}
            {#if closed && winner == o.id}<div part="ArenaVote.state ArenaVote.caption" data-tone="success">{t("vote.winner")}</div>{/if}
            {#if o.disabled && o.reason}<div part="ArenaVote.state ArenaVote.caption">{o.reason}</div>{/if}
          </div>
          <div part="ArenaVote.tally ArenaVote.figure">{t("vote.tally", { count: formatNumber(o.votes) })}</div>
          </div>
        </button>
      {/each}
    </div>
    <div class="scroll-fade" part="ArenaVote.fade" data-anchor="bottom-edge"></div>
    </div>
  {:else}<p part="ArenaVote.empty ArenaVote.caption" role="status">{t("vote.empty")}</p>{/if}{/if}
</section>
<style>
/* The vote on a smoked cut card: the clock in gold, an option a cut tile
   of dark glass with its tally at the end, the player's own ringed in
   the accent and washed in it. */
:host { inline-size: stretch; block-size: auto; max-block-size: stretch; color: var(--hud-ink); --fade-fill: var(--party-open-fill); }
[part="ArenaVote.frame"] { box-sizing: border-box; padding: var(--space-lg); border: 0; border-radius: 0; color: var(--hud-ink); background: var(--party-card-fill); box-shadow: var(--party-card-rim); clip-path: polygon(var(--party-cut) 0, 100% 0, 100% calc(100% - var(--party-cut)), calc(100% - var(--party-cut)) 100%, 0 100%, 0 var(--party-cut)); }
[part="ArenaVote.title"] { margin: 0; min-inline-size: 0; font-size: var(--hud-size-md); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-title-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); text-shadow: none; white-space: normal; overflow-wrap: break-word; }
[part~="ArenaVote.detail"], [part~="ArenaVote.empty"] { margin: 0; font-size: var(--hud-size-xs); font-weight: var(--hud-label-weight); color: var(--hud-ink-dim); overflow-wrap: break-word; }
[part="ArenaVote.title"], [part~="ArenaVote.detail"], [part~="ArenaVote.timer"] { flex: none; }
[part~="ArenaVote.timer"] { font-size: var(--hud-size-time); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-gold); text-shadow: none; }
.scroll-area { position: relative; min-block-size: 0; }
[part="ArenaVote.options"] { min-block-size: 0; padding-block-end: var(--space-lg); }
[part="ArenaVote.option"] { flex: none; min-block-size: var(--size-touch); padding: var(--space-sm) var(--space-md); text-align: start; border: 0; border-radius: 0; color: var(--hud-ink); background: var(--party-tile-fill); box-shadow: var(--party-tile-rim); clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); }
[part="ArenaVote.option"]:hover { box-shadow: var(--party-tile-hover-rim); }
[part="ArenaVote.option"]:focus-visible { box-shadow: var(--party-focus-rim); }
[part="ArenaVote.option"][aria-pressed=true] { background: var(--party-tile-on-fill); box-shadow: var(--party-on-rim); }
[part="ArenaVote.option"]:disabled { color: var(--hud-ink-dim); }
.option-body { inline-size: stretch; }
[part="ArenaVote.icon"] { flex: none; inline-size: var(--size-sm); block-size: var(--size-sm); color: inherit; }
.copy { min-inline-size: 0; }
[part~="ArenaVote.name"] { font-size: var(--hud-size-readout); line-height: 1.15; font-weight: var(--hud-figure-weight); color: inherit; overflow-wrap: break-word; }
[part~="ArenaVote.description"] { font-size: var(--hud-size-xs); color: var(--hud-ink-dim); overflow-wrap: break-word; }
[part~="ArenaVote.state"] { font-size: var(--hud-tag-size); line-height: 1.5; font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-accent); overflow-wrap: break-word; }
[part~="ArenaVote.state"][data-tone=success] { color: var(--party-ready); }
[part="ArenaVote.option"]:disabled [part~="ArenaVote.state"] { color: var(--hud-ink-dim); }
:host > section > [part~="ArenaVote.state"] { color: var(--hud-ink-dim); }
[part~="ArenaVote.tally"] { flex: 0 1 auto; font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: inherit; text-shadow: none; overflow-wrap: break-word; }
@container (width < 400px) {
  [part="ArenaVote.frame"] { padding: var(--space-sm); }
  [part="ArenaVote.option"] { padding: var(--space-xs) var(--space-sm); }
  [part~="ArenaVote.tally"] { flex-basis: 100%; text-align: end; }
}
</style>
