<meta name="component" content="RunPage">

<!-- Between fights: the run map with visited, current, reachable and
     out-of-reach rooms beside a card reward (one offer refused). A phone:
     the reward over the map. -->
<div id="Page">
  <div id="Map"><CardRunMap title={$mapTitle} floor={$mapFloor} nodes={$mapNodes} links={$mapLinks} view={$mapView} zoom={1} panX={0} panY={0} selected={$mapSelected} legend={$mapLegend} /></div>
  <div id="Reward"><CardReward title={$rewardTitle} choices={$rewardChoices} selected={$rewardSelected} skipLabel={$rewardSkip} /></div>
</div>

<style>
#Page { display: flex; width: stretch; height: stretch; gap: var(--space-xl); padding: var(--space-xl); background: var(--backdrop); }
#Map { width: 42%; height: stretch; }
#Reward { flex: 1; min-width: 0; height: stretch; }
@media (width < 600px) {
  #Page { flex-direction: column; gap: var(--space-md); padding: var(--space-sm); }
  #Reward { width: stretch; flex: none; height: 58%; }
  #Map { order: 1; width: stretch; flex: 1; min-height: 0; height: auto; }
}
</style>
