<meta name="component" content="StrategyDefencePage">

<!-- The stress page, tower defence at its limits over the field's scene: an
     endless wave 1284 with a long name, a long boss and five-digit counts,
     the overflowing production queue, nine-digit stockpiles, and a
     tier-ten tower inspector with long path names, every upgrade state and
     a nine-digit refund. On a phone they stack and scroll. -->
<div id="Page">
  <img id="Stage" src={$hudField} alt="">
  <div id="Scrim"></div>
  <scroll id="Scroll">
    <div id="Columns">
      <div id="Left">
        <StrategyResources tier={$resources_overflow.tier} resources={$resources_overflow_resources} />
        <StrategyWave wave={$wave_overflow.wave} waves={$wave_overflow.waves} name={$wave_overflow.name} icon={$wave_overflow.icon} phase={$wave_overflow.phase} next={$wave_overflow.next} nextTotal={$wave_overflow.nextTotal} warnAt={$wave_overflow.warnAt} finalWave={$wave_overflow.finalWave} canCallEarly={$wave_overflow.canCallEarly} reward={$wave_overflow.reward} enemies={$wave_overflow_enemies} />
        <div class="plate"><StrategyQueue title={$queue_overflow.title} queued={$queue_overflow.queued} limit={$queue_overflow.limit} full={$queue_overflow.full} items={$queue_overflow_items} /></div>
      </div>
      <div id="Inspect"><StrategyTower name={$tower_overflow.name} icon={$tower_overflow.icon} portrait={$tower_overflow.portrait} level={$tower_overflow.level} canSell={$tower_overflow.canSell} sellText={$tower_overflow.sellText} stats={$tower_overflow_stats} paths={$tower_overflow_paths} /></div>
    </div>
  </scroll>
</div>

<style>
#Page { position: fixed; inset: 0; overflow: hidden; background: #05070c; }
#Stage { position: absolute; left: 0; top: 0; width: stretch; height: stretch; object-fit: cover; }
#Scrim { position: absolute; left: 0; top: 0; width: stretch; height: stretch; background: var(--hud-veil); }
#Scroll { position: absolute; left: 0; top: 0; width: stretch; height: stretch; }
#Columns { display: flex; width: stretch; box-sizing: border-box; gap: var(--space-xl); align-items: flex-start;
  padding: calc(env(-og-hud-inset-top) + var(--space-xl)) calc(env(-og-hud-inset-right) + var(--space-xl)) calc(env(-og-hud-inset-bottom) + var(--space-xl)) calc(env(-og-hud-inset-left) + var(--space-xl)); }
#Left { display: flex; flex-direction: column; align-items: flex-start; flex: 1; min-width: 0; gap: var(--space-lg); }
#Inspect { flex: 1; min-width: 0; }
.plate { display: flex; width: stretch; min-width: 0; box-sizing: border-box; padding: var(--space-sm) var(--space-md); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); }
@media (width < 600px) {
  #Columns { flex-direction: column; gap: var(--space-md);
    padding: calc(env(-og-hud-inset-top) + var(--space-sm)) calc(env(-og-hud-inset-right) + var(--space-sm)) calc(env(-og-hud-inset-bottom) + var(--space-sm)) calc(env(-og-hud-inset-left) + var(--space-sm)); }
  #Left, #Inspect { width: stretch; flex: none; }
  #Left { gap: var(--space-md); }
}
</style>
