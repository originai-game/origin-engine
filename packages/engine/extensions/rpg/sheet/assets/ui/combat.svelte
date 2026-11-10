<meta name="component" content="CombatPage">

<!-- Turn-based combat, as on the board: the turn order and Round plate at
     the top, the combat log top left, the objective top right, the targeted
     enemy on the right, the party along the foot and the commands bottom
     right. -->
<div id="Page">
  <img id="Scene" src={$exploreScene} alt="">
  <div id="Hud">
    <div id="Top">
      <div id="Log"><RpgCombatLog lines={$logLines} /></div>
      <div id="Turns"><RpgTurnOrder round={$turnRound} turns={$turns} /></div>
      <div id="Goal"><RpgQuestTracker name={$objectiveName} steps={$objectiveSteps} distance="" /></div>
    </div>
    <div id="Mid"><div id="Enemy"><RpgEnemyInfo name={$enemyName} portrait={$enemyPortrait} level={$enemyLevel} health={$enemyHealth} healthMax={$enemyHealthMax} effects={$enemyEffects} /></div></div>
    <div id="Foot">
      <div id="Party"><RpgParty members={$partyMembers} selected={$partySelected} /></div>
      <div id="Commands"><RpgCommands commands={$commands} selected={$commandSelected} /></div>
    </div>
  </div>
</div>

<style>
#Page { position: relative; width: stretch; height: stretch; overflow: hidden; }
#Scene { position: absolute; left: 0; top: 0; width: stretch; height: stretch; object-fit: cover; }
#Hud { position: relative; display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-md); padding: var(--space-md) var(--space-lg) var(--space-lg); }
#Top { display: flex; width: stretch; align-items: flex-start; gap: var(--space-lg); flex: none; }
#Log { flex: none; width: 22%; height: calc(var(--size-xxl) * 1.9); }
#Turns { flex: 1; min-width: 0; }
#Goal { flex: none; width: 18%; }
#Mid { display: flex; width: stretch; flex: 1; min-height: 0; justify-content: flex-end; align-items: flex-end; }
#Enemy { width: 18%; }
#Foot { display: flex; width: stretch; flex: none; align-items: flex-end; gap: var(--space-lg); }
#Party { flex: 1; min-width: 0; }
#Commands { flex: none; width: 44%; }
@media (width < 600px) {
  #Hud { padding: var(--space-sm); gap: var(--space-sm); }
  #Top { flex-wrap: wrap; gap: var(--space-sm); }
  #Log { display: none; }
  #Turns { width: 100%; flex: none; }
  #Goal { width: 100%; }
  #Enemy { width: 100%; }
  #Foot { flex-direction: column; gap: var(--space-sm); }
  #Party, #Commands { width: 100%; flex: none; }
}
@media (height < 500px) {
  #Hud { display: grid; grid-template-columns: minmax(0, 1fr) 34%; grid-template-rows: auto minmax(0, 1fr) auto; grid-template-areas: "turns enemy" ". enemy" "party commands"; column-gap: var(--space-md); row-gap: var(--space-xs); padding: var(--space-sm) var(--space-md); }
  #Top, #Mid, #Foot { display: contents; }
  #Log, #Goal { display: none; }
  #Turns { grid-area: turns; }
  #Enemy { grid-area: enemy; width: stretch; align-self: start; }
  #Party { grid-area: party; align-self: end; }
  #Commands { grid-area: commands; width: stretch; align-self: end; }
}
</style>
