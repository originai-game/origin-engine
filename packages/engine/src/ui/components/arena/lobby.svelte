<meta name="component" content="ArenaLobby">
<meta name="strings" content="./arena.en.ogt">
<meta name="parts" content="ArenaLobby.frame ArenaLobby.band ArenaLobby.title ArenaLobby.row ArenaLobby.portrait ArenaLobby.name ArenaLobby.badge ArenaLobby.cell">
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
<section id="ArenaLobby" part="ArenaLobby.frame" aria-label={title != '' ? title : t("arena.lobby.title")}>
  <header part="ArenaLobby.band" class="band"><h2 part="ArenaLobby.title">{title != '' ? title : t("arena.lobby.title")}</h2><span class="detail" hidden={detail == ''}>{detail}</span></header>
  {#if slots.length}
    <scroll class="list">
      <div class="rows" role="list" aria-label={t("arena.lobby.players")}>
        {#each slots as s (s.id)}
          <div part="ArenaLobby.row" role="listitem" aria-current={s.local} data-open={s.open} data-local={s.local}>
            <div part="ArenaLobby.portrait" class="portrait" data-open={s.open}><img class="art" src={s.portrait} alt="" hidden={!s.portrait || s.open}><og-icon class="glyph" icon={s.open ? 'plus' : s.icon} hidden={!!s.portrait && !s.open}></og-icon></div>
            <div class="who">
              <span part="ArenaLobby.name" class="name">{s.open ? t("arena.lobby.open") : s.name}</span>
              <div class="tags" hidden={!(s.host || s.local)}>
                <span part="ArenaLobby.badge" class="tag" data-tag="host" hidden={!s.host}>{t("arena.lobby.host")}</span>
                <span part="ArenaLobby.badge" class="tag" data-tag="you" hidden={!s.local}>{t("arena.you")}</span>
              </div>
            </div>
            <div part="ArenaLobby.cell" class="state" data-ready={s.ready} hidden={s.open}><og-icon class="check" icon={s.ready ? 'check' : 'clock'}></og-icon><span>{s.ready ? t("arena.lobby.ready") : t("arena.lobby.notReady")}</span></div>
            <button id="Invite" variant="ghost" hidden={!(s.open && canInvite)} onclick={invite}><og-icon class="check" icon="plus"></og-icon><span>{t("arena.lobby.invite")}</span></button>
          </div>
        {/each}
      </div>
    </scroll>
  {:else}
    <p class="empty-text" role="status">{t("arena.lobby.empty")}</p>
  {/if}
  <footer class="actions">
    {#if isReady}
      <button id="Unready" variant="ghost" disabled={!canReady} onclick={unready} ondetails={unready}><kbd class="key" data-action="details"></kbd><span>{t("arena.lobby.unready")}</span></button>
    {:else}
      <button id="Ready" variant="ghost" disabled={!canReady} onclick={ready} ondetails={ready}><kbd class="key" data-action="details"></kbd><span>{t("arena.lobby.readyUp")}</span></button>
    {/if}
    <button id="Start" variant="primary" hidden={!isHost} disabled={!isHost || !canStart} onclick={start}><span>{t("arena.lobby.start")}</span></button>
  </footer>
</section>

<style>
[part="ArenaLobby.frame"][hidden], [part="ArenaLobby.frame"] [hidden] { display: none; }
[part="ArenaLobby.frame"] { display: flex; flex-direction: column; width: stretch; min-width: 0; max-height: stretch; gap: var(--space-sm); container-type: inline-size; }
[part="ArenaLobby.band"] { display: flex; width: stretch; align-items: baseline; flex: none; flex-wrap: wrap; gap: var(--space-xxs) var(--space-md); }
[part="ArenaLobby.title"] { margin: 0; min-width: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.detail { min-width: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.list { width: stretch; flex: 1; min-height: 0; }
.rows { display: flex; flex-direction: column; width: stretch; gap: var(--space-xs); }
[part="ArenaLobby.row"] { display: flex; width: stretch; align-items: center; gap: var(--space-sm); padding: var(--space-xxs) var(--space-sm); min-height: var(--size-touch); }
[part="ArenaLobby.portrait"] { display: flex; flex: none; width: var(--size-lg); height: var(--size-lg); align-items: center; justify-content: center; overflow: hidden; }
.art { width: stretch; height: stretch; object-fit: cover; object-position: center top; }
.glyph { width: var(--size-sm); height: var(--size-sm); }
.who { display: flex; flex-direction: column; flex: 1; min-width: 0; gap: var(--space-xxs); }
.name { width: stretch; text-align: start; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.tags { display: flex; gap: var(--space-xxs); }
.tag { flex: none; white-space: nowrap; padding: 0 var(--space-xs); }
[part="ArenaLobby.cell"] { display: flex; flex: none; align-items: center; gap: var(--space-xxs); white-space: nowrap; }
.check { width: var(--size-xs); height: var(--size-xs); }
#Invite { flex: none; min-height: var(--size-md); gap: var(--space-xxs); }
.empty-text { margin: 0; padding: var(--space-lg); text-align: center; }
.actions { display: flex; width: stretch; flex: none; justify-content: flex-end; gap: var(--space-sm); padding-bottom: env(safe-area-inset-bottom); }
.actions button { min-height: var(--size-touch); min-width: calc(var(--size-xxl) * 1.25); gap: var(--space-xs); }
@media (pointer: coarse) { .key { display: none; } }
/* The lobby sits in a panel: its own width, not the screen's, tightens it. */
@container (width < 600px) {
  [part="ArenaLobby.row"] { gap: var(--space-xs); padding: var(--space-xxs) var(--space-xs); }
  [part="ArenaLobby.portrait"] { width: var(--size-md); height: var(--size-md); }
  /* The actions share the width; when their words do not fit side by side, each takes a line. */
  .actions { flex-wrap: wrap; }
  .actions button { flex: 1 1 auto; }
}
</style>
