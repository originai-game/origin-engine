<meta name="component" content="CombatPage">

<!-- Turn-based combat at desktop: the turn order across the top over the
     battlefield, the party along the foot. -->
<div id="Page">
  <img id="Scene" src={$exploreScene} alt="">
  <div id="Turns"><RpgTurnOrder round={$turnRound} turns={$turns} /></div>
  <div id="Party"><RpgParty members={$partyMembers} selected={$partySelected} /></div>
</div>

<style>
#Page { position: relative; display: flex; flex-direction: column; justify-content: space-between; width: stretch; height: stretch; overflow: hidden; padding: var(--space-md) var(--space-lg) var(--space-lg); }
#Scene { position: absolute; left: 0; top: 0; width: stretch; height: stretch; object-fit: cover; }
#Turns { position: relative; width: 50%; align-self: center; }
#Party { position: relative; width: 40%; }
@media (width < 600px) {
  #Page { padding: var(--space-sm); }
  #Turns, #Party { width: stretch; }
}
</style>
