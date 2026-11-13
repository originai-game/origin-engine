<meta name="component" content="GaragePage">

<!-- The stress page: the ten-car garage with long names, a locked car and
     the car driven, beside the fourteen-row classification with long names,
     a DNF and a DSQ. On a phone they stack, each scrolling in its half. -->
<div id="Page">
  <div id="Garage"><RacingGarage title={$garage_overflow.title} vehicles={$garage_overflow_vehicles} shown={$garage_overflow.shown} index={$garage_overflow.index} chosen={$garage_overflow.chosen} stats={$garage_overflow_stats} /></div>
  <div id="Results"><RacingResults title={$results_overflow.title} rows={$results_overflow_rows} bestLap={$results_overflow.bestLap} bestBy={$results_overflow.bestBy} canRetry={$results_overflow.canRetry} /></div>
</div>

<style>
#Page { display: flex; width: stretch; height: stretch; gap: var(--space-xl); padding: var(--space-xl); }
#Garage { flex: 3; min-width: 0; height: stretch; }
#Results { flex: 2; min-width: 0; height: stretch; }
@media (width < 600px) {
  #Page { flex-direction: column; gap: var(--space-md); padding: var(--space-sm); }
  #Garage, #Results { flex: 1; min-height: 0; width: stretch; height: auto; }
}
</style>
