<meta name="component" content="ArenaLobby">
<meta name="strings" content="./arena.en.ogt">
<meta name="parts" content="ArenaLobby.frame ArenaLobby.band ArenaLobby.title ArenaLobby.row ArenaLobby.portrait ArenaLobby.name ArenaLobby.badge ArenaLobby.state ArenaLobby.picture ArenaLobby.label ArenaLobby.headline ArenaLobby.caption">
<meta name="prop:title" content="string" data-doc="The lobby's heading (the match's name); empty takes the kit's (Lobby).">
<meta name="prop:detail" content="string" data-doc="A line beside the heading as the game words it (Ranked · 2 v 2, Waiting for the host); empty hides it.">
<meta name="prop:slots" content="list" data-doc="The lobby's seats in the game's order, keyed rows of { id, name, portrait, icon, open, host, ready, local }: open the seat is free (an Invite button reporting invite with the seat's key, when canInvite); portrait a texture (0 shows the icon); host marks the host (Host); ready Ready or Not ready in words; local the local player (You).">
<meta name="prop:canInvite" content="bool" data-doc="Open seats offer Invite (false: they read Open slot only).">
<meta name="prop:isReady" content="bool" data-doc="The local player is ready: the button reads Cancel ready (unready) instead of Ready up (ready).">
<meta name="prop:canReady" content="bool" data-doc="The ready button is offered (false: disabled).">
<meta name="prop:isHost" content="bool" data-doc="The local player hosts: Start match is shown.">
<meta name="prop:canStart" content="bool" data-doc="Start match is offered (false: disabled; the game decides when the match can start).">
<meta name="action:details" content="Ready">
<meta name="fixture:default" content='{"title":"","detail":"Ranked · 2 v 2","canInvite":true,"isReady":false,"canReady":true,"isHost":true,"canStart":false,"slots":[{"id":"s1","name":"Rhea","portrait":0,"icon":"sword","open":false,"host":true,"ready":false,"local":true},{"id":"s2","name":"Kai","portrait":0,"icon":"shield","open":false,"host":false,"ready":true,"local":false},{"id":"s3","name":"","portrait":0,"icon":"","open":true,"host":false,"ready":false,"local":false},{"id":"s4","name":"","portrait":0,"icon":"","open":true,"host":false,"ready":false,"local":false}]}'>
<meta name="fixture:full" content='{"title":"","detail":"","canInvite":true,"isReady":true,"canReady":true,"isHost":true,"canStart":true,"slots":[{"id":"s1","name":"Rhea","portrait":0,"icon":"sword","open":false,"host":true,"ready":true,"local":true},{"id":"s2","name":"Kai","portrait":0,"icon":"shield","open":false,"host":false,"ready":true,"local":false},{"id":"s3","name":"Ana","portrait":0,"icon":"heart","open":false,"host":false,"ready":true,"local":false},{"id":"s4","name":"Rook","portrait":0,"icon":"bolt","open":false,"host":false,"ready":true,"local":false}]}'>
<meta name="fixture:guest" content='{"title":"","detail":"Waiting for the host","canInvite":false,"isReady":true,"canReady":true,"isHost":false,"canStart":false,"slots":[{"id":"s1","name":"Kai","portrait":0,"icon":"shield","open":false,"host":true,"ready":false,"local":false},{"id":"s2","name":"Rhea","portrait":0,"icon":"sword","open":false,"host":false,"ready":true,"local":true},{"id":"s3","name":"","portrait":0,"icon":"","open":true,"host":false,"ready":false,"local":false}]}'>
<meta name="fixture:empty" content='{"title":"","detail":"","canInvite":true,"isReady":false,"canReady":false,"isHost":true,"canStart":false,"slots":[]}'>
<meta name="fixture:disabled" content='{"title":"","detail":"Matchmaking is down for maintenance","canInvite":false,"isReady":false,"canReady":false,"isHost":true,"canStart":false,"slots":[{"id":"s1","name":"Rhea","portrait":0,"icon":"sword","open":false,"host":true,"ready":false,"local":true},{"id":"s2","name":"","portrait":0,"icon":"","open":true,"host":false,"ready":false,"local":false}]}'>
<meta name="fixture:overflow" content='{"title":"Championship Series · Grand Finals · Deciding match lobby","detail":"Custom · 5 v 5 · Draft pick · Tournament rules · Spectators allowed","canInvite":true,"isReady":false,"canReady":true,"isHost":true,"canStart":false,"slots":[{"id":"s1","name":"Maximilian Vandenberghe-Okonkwo","portrait":0,"icon":"crown","open":false,"host":true,"ready":true,"local":false},{"id":"s2","name":"Rhea","portrait":0,"icon":"sword","open":false,"host":false,"ready":false,"local":true},{"id":"s3","name":"Ana","portrait":0,"icon":"heart","open":false,"host":false,"ready":true,"local":false},{"id":"s4","name":"Kai","portrait":0,"icon":"shield","open":false,"host":false,"ready":false,"local":false},{"id":"s5","name":"Jo","portrait":0,"icon":"music","open":false,"host":false,"ready":true,"local":false},{"id":"s6","name":"Abernathy-Whitcombe","portrait":0,"icon":"bolt","open":false,"host":false,"ready":true,"local":false},{"id":"s7","name":"Wren","portrait":0,"icon":"leaf","open":false,"host":false,"ready":false,"local":false},{"id":"s8","name":"","portrait":0,"icon":"","open":true,"host":false,"ready":false,"local":false},{"id":"s9","name":"","portrait":0,"icon":"","open":true,"host":false,"ready":false,"local":false},{"id":"s10","name":"","portrait":0,"icon":"","open":true,"host":false,"ready":false,"local":false}]}'>

<!--
  The pre-match lobby: a heading and the match's line, one row per seat
  (portrait, name, Host, You, and Ready or Not ready in words), an open
  seat reading Open slot with an Invite button (invite, the seat's key),
  and the local player's actions: Ready up (ready; also the details action, X or pad Y) or Cancel ready
  (unready), and for the host Start match (start), disabled until the game
  allows it. The game owns seats, readiness and when a match may start.
-->
<section id="ArenaLobby" part="ArenaLobby.frame" data-layout="shell" data-gap="loose" aria-label={title != '' ? title : t("arena.lobby.title")}>
  <header part="ArenaLobby.band" class="band" data-layout="cluster" data-gap="loose"><h2 part="ArenaLobby.title">{title != '' ? title : t("arena.lobby.title")}</h2>{#if detail != ''}<span class="detail" part="ArenaLobby.caption">{detail}</span>{/if}</header>
  {#if slots.length}
    <div class="list" data-layout="scroll" data-fill>
      <div class="rows" data-layout="stack" data-gap="tight" role="list" aria-label={t("arena.lobby.players")}>
        {#each slots as s (s.id)}
          <div part="ArenaLobby.row" data-layout="row" role="listitem" aria-current={s.local} data-open={s.open} data-local={s.local}>
            <div part="ArenaLobby.portrait ArenaLobby.picture" class="portrait" data-layout="ratio" data-open={s.open}><div class="well" data-layout="ratio">{#if s.portrait && !s.open}<img class="art" src={s.portrait} alt="">{:else}<og-icon class="glyph" icon={s.open ? 'plus' : s.icon}></og-icon>{/if}</div></div>
            <div class="who" data-layout="stack" data-gap="hair" data-fill>
              <span part="ArenaLobby.name ArenaLobby.headline" class="name">{s.open ? t("arena.lobby.open") : s.name}</span>
              {#if s.host || s.local}
              <div class="tags" data-layout="row" data-gap="hair">
                {#if s.host}<span part="ArenaLobby.badge" class="tag" data-tag="host">{t("arena.lobby.host")}</span>{/if}
                {#if s.local}<span part="ArenaLobby.badge" class="tag" data-tag="you">{t("arena.you")}</span>{/if}
              </div>
              {/if}
            </div>
            {#if !s.open}<div part="ArenaLobby.state ArenaLobby.label" class="state" data-layout="row" data-gap="hair" data-tone={s.ready ? 'success' : 'neutral'} data-ready={s.ready}><og-icon class="check" icon={s.ready ? 'check' : 'clock'}></og-icon><span>{s.ready ? t("arena.lobby.ready") : t("arena.lobby.notReady")}</span></div>{/if}
            {#if s.open && canInvite}<button id="Invite" variant="ghost" onclick={invite}><og-icon class="check" icon="plus"></og-icon><span>{t("arena.lobby.invite")}</span></button>{/if}
          </div>
        {/each}
      </div>
    </div>
  {:else}
    <p class="empty-text" part="ArenaLobby.caption" role="status">{t("arena.lobby.empty")}</p>
  {/if}
  <footer class="actions" data-layout="cluster">
    {#if isReady}
      <button id="Unready" variant="ghost" aria-label={t("arena.lobby.unready")} disabled={!canReady} onclick={unready} ondetails={unready}><kbd class="key" data-action="details"></kbd><span>{t("arena.lobby.unready")}</span></button>
    {:else}
      <button id="Ready" variant="ghost" aria-label={t("arena.lobby.readyUp")} disabled={!canReady} onclick={ready} ondetails={ready}><kbd class="key" data-action="details"></kbd><span>{t("arena.lobby.readyUp")}</span></button>
    {/if}
    {#if isHost}<button id="Start" variant="primary" disabled={!isHost || !canStart} onclick={start}><span>{t("arena.lobby.start")}</span></button>{/if}
  </footer>
</section>

<style>
/* The lobby on a smoked cut card: a seat is a cut row of dark glass with
   an octagon portrait, the local player's lit along its top in the
   accent, an open seat a faint wash; Ready is green, with a glow. */
:host { inline-size: stretch; min-inline-size: 0; max-block-size: stretch; color: var(--hud-ink); }
[part="ArenaLobby.frame"] { box-sizing: border-box; padding: var(--space-lg); border: 0; border-radius: 0; color: var(--hud-ink); background: var(--party-card-fill); box-shadow: var(--party-card-rim); clip-path: polygon(var(--party-cut) 0, 100% 0, 100% calc(100% - var(--party-cut)), calc(100% - var(--party-cut)) 100%, 0 100%, 0 var(--party-cut)); }
[part="ArenaLobby.band"] { width: stretch; align-items: baseline; flex: none; min-height: 0; padding: 0; border: 0; border-radius: 0; background: none; box-shadow: none; }
[part="ArenaLobby.title"] { margin: 0; min-inline-size: 0; font-size: var(--hud-size-md); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-title-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); text-shadow: none; white-space: normal; overflow-wrap: break-word; }
.detail { min-width: 0; font-size: var(--hud-size-xs); font-weight: var(--hud-label-weight); color: var(--hud-ink-dim); overflow-wrap: break-word; }
.list { overflow-y: auto; flex: 1; min-height: 0; -og-scroll-fade: var(--space-xl); }
.rows { width: stretch; }
[part="ArenaLobby.row"] { padding: var(--space-xs) var(--space-sm); min-height: var(--size-touch); border-radius: 0; color: var(--hud-ink); background: var(--result-row-fill); box-shadow: var(--party-seat-rim); clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); }
[part="ArenaLobby.row"][data-local=true] { box-shadow: var(--hud-plate-lit-rim); }
[part="ArenaLobby.row"][data-open=true] { background: var(--party-open-row); }
[part="ArenaLobby.portrait ArenaLobby.picture"] { flex: none; box-sizing: border-box; padding: var(--stroke-thick); border: 0; border-radius: 0; background: var(--hud-edge); box-shadow: none; clip-path: var(--hud-octagon); overflow: hidden; inline-size: var(--party-portrait); block-size: var(--party-portrait); }
[part="ArenaLobby.portrait ArenaLobby.picture"][data-open=true] { padding: var(--stroke-hairline); background: var(--party-open-edge); }
.well { inline-size: stretch; block-size: stretch; clip-path: var(--hud-octagon); background: var(--party-open-fill); }
.art { width: stretch; height: stretch; object-fit: cover; object-position: center top; }
.glyph { width: var(--size-sm); height: var(--size-sm); color: var(--hud-ink); }
[data-open=true] .glyph { color: var(--hud-ink-dim); }
.who { min-width: 0; }
.name { text-align: start; font-size: var(--hud-size-name); line-height: 1.15; font-weight: var(--hud-figure-weight); color: var(--hud-ink); overflow-wrap: break-word; }
[data-open=true] .name { color: var(--hud-ink-dim); }
.tags { flex-wrap: wrap; }
.tag { flex: none; padding: 0 var(--space-xs); border: 0; border-radius: 0; font-size: var(--hud-tag-size); line-height: 1.5; font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); text-shadow: none; clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); white-space: nowrap; }
.tag[data-tag=host] { color: var(--party-gold-ink); background: var(--hud-key-gold-fill); box-shadow: none; }
.tag[data-tag=you] { color: var(--party-go-ink); background: var(--hud-accent); box-shadow: none; }
[part="ArenaLobby.state ArenaLobby.label"] { flex: none; white-space: nowrap; font-size: var(--hud-size-xs); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink-dim); }
[part="ArenaLobby.state ArenaLobby.label"][data-ready=true] { color: var(--party-ready); filter: var(--party-ready-glow); }
.check { width: var(--size-xs); height: var(--size-xs); }
button { min-height: var(--size-touch); gap: var(--space-xs); padding: 0 var(--space-lg); border: 0; border-radius: 0; font-size: var(--hud-size-xs); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); background: var(--party-button-fill); box-shadow: var(--party-button-rim); text-shadow: none; clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); }
button:hover { background: var(--party-button-hover-fill); }
button:focus-visible { box-shadow: var(--party-focus-rim); }
button[variant=primary] { color: var(--party-go-ink); background: var(--front-play-fill); box-shadow: none; }
button[variant=primary]:hover { background: var(--party-go-hover-fill); }
button:disabled { opacity: 0.45; }
button > span { color: inherit; font-weight: inherit; }
#Invite { flex: none; min-height: var(--size-md); gap: var(--space-xxs); padding: 0 var(--space-md); }
.empty-text { margin: 0; padding: var(--space-lg); text-align: center; font-size: var(--hud-size-sm); color: var(--hud-ink-dim); }
.actions { width: stretch; flex: none; justify-content: flex-end; padding-bottom: env(safe-area-inset-bottom); }
.actions button { min-width: calc(var(--size-xxl) * 1.25); }
.key { color: var(--hud-key-ink); background: var(--hud-key-fill); box-shadow: var(--hud-key-shadow); border-radius: var(--hud-key-radius); }
@media (pointer: coarse) { .key { display: none; } }
/* The lobby sits in a panel: its own width, not the screen's, tightens it. */
@container (width < 600px) {
  [part="ArenaLobby.frame"] { padding: var(--space-sm); }
  [part="ArenaLobby.row"] { --layout-gap: var(--layout-gap-tight); flex-wrap: wrap; padding: var(--space-xxs) var(--space-xs); }
  [part="ArenaLobby.portrait ArenaLobby.picture"] { inline-size: var(--party-portrait-compact); block-size: var(--party-portrait-compact); }
  /* The name and its tags keep a width of their own; when the state or
     Invite does not fit beside them it takes the line under, at its end. */
  .who { flex: 1 1 calc(96 * var(--font-step-px)); }
  [part="ArenaLobby.state ArenaLobby.label"], #Invite { margin-inline-start: auto; }
  /* The actions share the width; when their words do not fit side by side, each takes a line. */
  .actions { flex-wrap: wrap; }
  .actions button { flex: 1 1 auto; }
}
</style>
