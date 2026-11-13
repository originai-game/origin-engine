<meta name="component" content="PuzzlePlayPage">

<!-- In a level: the goals and moves over the board, the board itself a
     stand-in well (the game draws it), the booster tray under it with the
     hammer armed and one booster unavailable. -->
<div id="Page">
  <div id="Hud"><PuzzleGoals counter={$goals_default.counter} moves={$goals_default.moves} seconds={$goals_default.seconds} low={$goals_default.low} score={$goals_default.score} goals={$goals_default_goals} /></div>
  <div id="Board"></div>
  <div id="Tray"><PuzzleBoosters armed="hammer" boosters={$boosters_default_boosters} /></div>
</div>

<style>
#Page { display: flex; flex-direction: column; width: stretch; height: stretch; align-items: center; gap: var(--space-md); padding: var(--space-lg); }
#Hud, #Tray { width: 60%; flex: none; }
#Board { width: 60%; flex: 1; min-height: 0; border-radius: var(--radius-md); background: var(--well); }
@media (width < 600px) {
  #Page { gap: var(--space-sm); padding: var(--space-sm); }
  #Hud, #Tray, #Board { width: stretch; }
}
</style>
