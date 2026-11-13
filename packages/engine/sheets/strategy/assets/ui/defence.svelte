<meta name="component" content="StrategyDefencePage">

<!-- The stress page, tower defence at its limits: an endless wave 1284 with a
     long boss name and five-digit counts, the overflowing production queue,
     and a tier-ten tower inspector with long path names, every upgrade
     state and a nine-digit refund. On a phone they stack and scroll. -->
<scroll id="Page">
  <div id="Columns">
    <div id="Left">
      <StrategyWave wave={$wave_overflow.wave} waves={$wave_overflow.waves} phase={$wave_overflow.phase} next={$wave_overflow.next} nextTotal={$wave_overflow.nextTotal} finalWave={$wave_overflow.finalWave} canCallEarly={$wave_overflow.canCallEarly} reward={$wave_overflow.reward} enemies={$wave_overflow_enemies} />
      <StrategyQueue title={$queue_overflow.title} queued={$queue_overflow.queued} limit={$queue_overflow.limit} full={$queue_overflow.full} items={$queue_overflow_items} />
    </div>
    <div id="Inspect"><StrategyTower name={$tower_overflow.name} icon={$tower_overflow.icon} portrait={$tower_overflow.portrait} level={$tower_overflow.level} canSell={$tower_overflow.canSell} sellText={$tower_overflow.sellText} stats={$tower_overflow_stats} paths={$tower_overflow_paths} /></div>
  </div>
</scroll>

<style>
#Page { width: stretch; height: stretch; }
#Columns { display: flex; width: stretch; gap: var(--space-xl); padding: var(--space-xl); align-items: flex-start; }
#Left { display: flex; flex-direction: column; flex: 1; min-width: 0; gap: var(--space-lg); }
#Inspect { flex: 1; min-width: 0; }
@media (width < 600px) {
  #Columns { flex-direction: column; gap: var(--space-md); padding: var(--space-sm); }
  #Left, #Inspect { width: stretch; flex: none; }
}
</style>
