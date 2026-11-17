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
            <div part="ArenaLobby.portrait ArenaLobby.picture" class="portrait" data-layout="ratio" data-open={s.open}>{#if s.portrait && !s.open}<img class="art" src={s.portrait} alt="">{:else}<og-icon class="glyph" icon={s.open ? 'plus' : s.icon}></og-icon>{/if}</div>
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
      <button id="Unready" variant="ghost" disabled={!canReady} onclick={unready} ondetails={unready}><kbd class="key" data-action="details"></kbd><span>{t("arena.lobby.unready")}</span></button>
    {:else}
      <button id="Ready" variant="ghost" disabled={!canReady} onclick={ready} ondetails={ready}><kbd class="key" data-action="details"></kbd><span>{t("arena.lobby.readyUp")}</span></button>
    {/if}
    {#if isHost}<button id="Start" variant="primary" disabled={!isHost || !canStart} onclick={start}><span>{t("arena.lobby.start")}</span></button>{/if}
  </footer>
</section>

<style>
:host { inline-size: stretch; min-inline-size: 0; max-block-size: stretch; }
[part="ArenaLobby.band"] { width: stretch; align-items: baseline; flex: none; }
[part="ArenaLobby.title"] { margin: 0; min-width: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.detail { min-width: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.list { overflow-y: auto; flex: 1; min-height: 0; }
.rows { width: stretch; }
[part="ArenaLobby.row"] { padding: var(--space-sm); min-height: var(--size-touch); }
[part="ArenaLobby.portrait ArenaLobby.picture"] { flex: none; width: var(--size-lg); height: var(--size-lg); overflow: hidden; }
.art { width: stretch; height: stretch; object-fit: cover; object-position: center top; }
.glyph { width: var(--size-sm); height: var(--size-sm); }
.who { min-width: 0; }
.name { text-align: start; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.tag { flex: none; white-space: nowrap; padding: 0 var(--space-xs); }
[part="ArenaLobby.state ArenaLobby.label"] { flex: none; white-space: nowrap; }
.check { width: var(--size-xs); height: var(--size-xs); }
#Invite { flex: none; min-height: var(--size-md); gap: var(--space-xxs); }
.empty-text { margin: 0; padding: var(--space-lg); text-align: center; }
.actions { width: stretch; flex: none; justify-content: flex-end; padding-bottom: env(safe-area-inset-bottom); }
.actions button { min-height: var(--size-touch); min-width: calc(var(--size-xxl) * 1.25); gap: var(--space-xs); }
@media (pointer: coarse) { .key { display: none; } }
/* The lobby sits in a panel: its own width, not the screen's, tightens it. */
@container (width < 600px) {
  [part="ArenaLobby.row"] { --layout-gap: var(--layout-gap-tight); padding: var(--space-xxs) var(--space-xs); }
  [part="ArenaLobby.portrait ArenaLobby.picture"] { width: var(--size-md); height: var(--size-md); }
  /* The actions share the width; when their words do not fit side by side, each takes a line. */
  .actions { flex-wrap: wrap; }
  .actions button { flex: 1 1 auto; }
}
</style>
