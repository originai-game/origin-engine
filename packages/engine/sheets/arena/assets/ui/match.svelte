<meta name="component" content="ArenaMatchPage">

<!-- The stress page, in and after a round: the round HUD with its won
     banner along the top over a bright sky and a dark floor, a
     ten-player scoreboard with long names and disconnected players, and
     the five-a-side versus with long team, player and champion names
     (docs/ui.md §6.3). On a phone they stack and the page scrolls. -->
<div id="Page">
  <div id="Sky"></div>
  <div id="Floor"></div>
  <scroll id="Over">
    <div id="Column">
      <div id="Hud"><ArenaRoundHud round={$roundHud_overflow.round} rounds={$roundHud_overflow.rounds} nameA={$roundHud_overflow.nameA} nameB={$roundHud_overflow.nameB} scoreA={$roundHud_overflow.scoreA} scoreB={$roundHud_overflow.scoreB} localTeam={$roundHud_overflow.localTeam} timeLeft={$roundHud_overflow.timeLeft} timeTotal={$roundHud_overflow.timeTotal} outcome={$roundHud_overflow.outcome} at={$roundHud_overflow.at} detail={$roundHud_overflow.detail} /></div>
      <div id="Boards">
        <div id="Score"><ArenaScoreboard title={$scoreboard_overflow.title} columns={$scoreboard_overflow.columns} nameA={$scoreboard_overflow.nameA} nameB={$scoreboard_overflow.nameB} scoreA={$scoreboard_overflow.scoreA} scoreB={$scoreboard_overflow.scoreB} rows={$scoreboard_overflow_rows} /></div>
        <div id="Versus"><ArenaVersus nameA={$versus_overflow.nameA} nameB={$versus_overflow.nameB} at={$versus_overflow.at} sideA={$versus_overflow_sideA} sideB={$versus_overflow_sideB} /></div>
      </div>
    </div>
  </scroll>
</div>

<style>
#Page { position: relative; width: stretch; height: stretch; overflow: hidden; }
#Sky { position: absolute; left: 0; top: 0; width: stretch; height: 45%; background: linear-gradient(180deg, #9cc7e8, #e2e6df); }
#Floor { position: absolute; left: 0; bottom: 0; width: stretch; height: 55%; background: linear-gradient(180deg, #4b4038, #16120e); }
#Over { position: absolute; left: 0; top: 0; width: stretch; height: stretch; }
#Column { display: flex; flex-direction: column; gap: var(--space-xl); padding: var(--space-lg) var(--space-xl); width: stretch; }
#Boards { display: flex; align-items: flex-start; gap: var(--space-xl); width: stretch; }
#Hud { display: flex; width: stretch; }
#Score { flex: 1 1 0; min-width: 0; }
#Versus { flex: 1 1 0; min-width: 0; }
@media (width < 600px) {
  #Column { gap: var(--space-md); padding: var(--space-xs); padding-top: env(safe-area-inset-top); }
  #Boards { flex-direction: column; align-items: stretch; gap: var(--space-md); }
  #Score, #Versus { flex: none; }
}
</style>
