<meta name="component" content="ArenaSelectPage">

<!-- The normal page, before a match: the roster with two players on it
     beside the draft's pick phase, and the lobby with its open seats. On a
     phone they stack and the page scrolls. -->
<scroll id="Page">
  <div id="Grid">
    <div id="Roster"><ArenaRoster title={$roster_default.title} selected={$roster_default.selected} characters={$roster_default_characters} players={$roster_default_players} /></div>
    <div id="Draft"><ArenaDraft title={$draft_default.title} phase={$draft_default.phase} active={$draft_default.active} nameA={$draft_default.nameA} nameB={$draft_default.nameB} timeLeft={$draft_default.timeLeft} timeTotal={$draft_default.timeTotal} canConfirm={$draft_default.canConfirm} slots={$draft_default_slots} /></div>
    <div id="Lobby"><ArenaLobby title={$lobby_default.title} detail={$lobby_default.detail} canInvite={$lobby_default.canInvite} isReady={$lobby_default.isReady} canReady={$lobby_default.canReady} isHost={$lobby_default.isHost} canStart={$lobby_default.canStart} slots={$lobby_default_slots} /></div>
  </div>
</scroll>

<style>
#Page { width: stretch; height: stretch; }
#Grid { display: flex; align-items: flex-start; gap: var(--space-xl); padding: var(--space-xl); width: stretch; }
#Roster { flex: 4; min-width: 0; }
#Draft { flex: 4; min-width: 0; }
#Lobby { flex: 3; min-width: 0; }
@media (width < 600px) {
  #Grid { flex-direction: column; flex-wrap: nowrap; align-items: stretch; gap: var(--space-md); padding: var(--space-xs); padding-top: env(safe-area-inset-top); }
  #Roster, #Draft, #Lobby { flex: none; }
}
</style>
