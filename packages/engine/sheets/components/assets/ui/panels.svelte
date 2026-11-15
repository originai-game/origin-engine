<meta name="component" content="UngroupedPanelsPage">

<!-- The stress page, in a menu: breadcrumbs seven deep with long names, a
     fourteen-row inventory grid with long names and big values, the longest
     tooltip, the context menu with its submenu open, the overlay bar, a
     chart, a skill graph, the carousel with long pages and the radial menu
     (docs/ui.md §6.3). On a phone they stack and the page scrolls. -->
<scroll id="Page">
  <div id="Column">
    <div id="Crumbs"><Breadcrumbs items={$breadcrumbs_overflow_items} label={$breadcrumbs_overflow.label} /></div>
    <div id="Progress"><ProgressBar value={$progressBar_overflow.value} buffer={$progressBar_overflow.buffer} label={$progressBar_overflow.label} valueText={$progressBar_overflow.valueText} showValue={$progressBar_overflow.showValue} indeterminate={$progressBar_overflow.indeterminate} tone={$progressBar_overflow.tone} /></div>
    <div id="Row">
      <div id="Grid"><DataGrid columns={$dataGrid_overflow_columns} tracks={$dataGrid_overflow.tracks} rows={$dataGrid_overflow_rows} selected={$dataGrid_overflow.selected} label={$dataGrid_overflow.label} loading={$dataGrid_overflow.loading} error={$dataGrid_overflow.error} empty={$dataGrid_overflow.empty} hint={$dataGrid_overflow.hint} /></div>
      <div id="Side">
        <TooltipCard title={$tooltipCard_overflow.title} subtitle={$tooltipCard_overflow.subtitle} rarity={$tooltipCard_overflow.rarity} icon={$tooltipCard_overflow.icon} image={$tooltipCard_overflow.image} body={$tooltipCard_overflow.body} stats={$tooltipCard_overflow_stats} compareTitle={$tooltipCard_overflow.compareTitle} compareStats={$tooltipCard_overflow_compareStats} footer={$tooltipCard_overflow.footer} hints={$tooltipCard_overflow_hints} />
        <div id="MenuBox"><Menu entries={$menu_submenu_entries} submenu={$menu_submenu.submenu} label={$menu_submenu.label} /></div>
      </div>
    </div>
    <div id="Overlays"><OverlayBar overlays={$overlayBar_default_overlays} selected={$overlayBar_default.selected} label={$overlayBar_default.label} /></div>
    <div id="Row2">
      <div class="half"><Chart title={$chart_default.title} subtitle={$chart_default.subtitle} max={$chart_default.max} bars={$chart_default_bars} line={$chart_default.line} area={$chart_default.area} ticks={$chart_default_ticks} /></div>
      <div class="half"><NodeGraph nodes={$nodeGraph_default_nodes} links={$nodeGraph_default_links} view={$nodeGraph_default.view} zoom={$nodeGraph_default.zoom} panX={$nodeGraph_default.panX} panY={$nodeGraph_default.panY} selected={$nodeGraph_default.selected} name={$nodeGraph_default.name} /></div>
    </div>
    <div id="Row3">
      <div class="half"><Carousel pages={$carousel_overflow_pages} index={$carousel_overflow.index} label={$carousel_overflow.label} /></div>
      <div id="Radial"><RadialMenu items={$radialMenu_default_items} span={$radialMenu_default.span} chosen={$radialMenu_default.chosen} detail={$radialMenu_default.detail} /></div>
    </div>
  </div>
</scroll>

<style>
#Page { width: stretch; height: stretch; }
#Column { display: flex; flex-direction: column; gap: var(--space-lg); padding: var(--space-lg) var(--space-xl); width: stretch; }
#Crumbs, #Progress, #Overlays { display: flex; width: stretch; }
#Row, #Row2, #Row3 { display: flex; align-items: flex-start; gap: var(--space-xl); width: stretch; }
#Grid { flex: 3 1 0; min-width: 0; max-height: calc(var(--size-xxl) * 9); display: flex; }
#Side { flex: 2 1 0; min-width: 0; display: flex; flex-direction: column; gap: var(--space-lg); }
#MenuBox { position: relative; display: flex; }
.half { flex: 1 1 0; min-width: 0; display: flex; }
#Radial { flex: 1 1 0; min-width: 0; position: relative; height: calc(var(--size-xxl) * 5); }
@media (width < 600px) {
  #Column { gap: var(--space-md); padding: var(--space-xs); padding-top: env(safe-area-inset-top); }
  #Row, #Row2, #Row3 { flex-direction: column; align-items: stretch; gap: var(--space-md); }
  #Grid, #Side, .half, #Radial { flex: none; }
}
</style>
