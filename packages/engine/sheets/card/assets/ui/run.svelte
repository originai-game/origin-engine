<meta name="component" content="RunPage">

<!-- Between fights, over the act's map art (sample art of the sheet
     only): the run map with a visited, the current, reachable and
     out-of-reach rooms beside a card reward (one offer refused). A phone:
     the reward over the map. -->
<div id="Page">
  <img id="Art" src={$runArt} alt="">
  <div id="Map"><CardRunMap title={$mapTitle} floor={$mapFloor} nodes={$mapNodes} links={$mapLinks} view={$mapView} zoom={1} panX={0} panY={0} selected={$mapSelected} legend={$mapLegend} /></div>
  <div id="Reward"><CardReward title={$rewardTitle} choices={$rewardChoices} selected={$rewardSelected} skipLabel={$rewardSkip} /></div>
</div>

<style>
#Page { position: relative; display: flex; width: stretch; height: stretch; gap: var(--space-xl); padding: var(--space-xl); background: var(--backdrop); }
#Art { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; }
#Map { position: relative; width: 42%; height: stretch; }
#Reward { position: relative; flex: 1; min-width: 0; height: stretch; }
@media (width < 600px) {
  #Page { flex-direction: column; gap: var(--space-md); padding: var(--space-sm); }
  #Reward { width: stretch; flex: none; height: 44%; }
  #Map { order: 1; width: stretch; flex: 1; min-height: 0; height: auto; }
}
</style>
