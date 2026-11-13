<meta name="component" content="PuzzleMapPage">

<!-- Between levels: a two-chapter level path (a long chapter name, levels
     past a thousand) beside a won level's summary with a new best and a
     six-row breakdown. A phone: the summary over the path. -->
<div id="Page">
  <div id="Path"><PuzzleLevelPath selected={$levelPath_overflow.selected} chapters={$levelPath_overflow_chapters} levels={$levelPath_overflow_levels} /></div>
  <div id="Summary"><PuzzleSummary result={$summary_overflow.result} level={$summary_overflow.level} score={$summary_overflow.score} scoreMax={$summary_overflow.scoreMax} thresholds={$summary_overflow_thresholds} stars={$summary_overflow.stars} maxStars={$summary_overflow.maxStars} record={$summary_overflow.record} rows={$summary_overflow_rows} total={$summary_overflow.total} canNext={$summary_overflow.canNext} /></div>
</div>

<style>
#Page { display: flex; width: stretch; height: stretch; gap: var(--space-lg); padding: var(--space-xl); }
#Path { width: 45%; height: stretch; min-height: 0; }
#Summary { width: 40%; height: stretch; min-height: 0; }
@media (width < 600px) {
  #Page { flex-direction: column; gap: var(--space-sm); padding: var(--space-sm); }
  #Summary { width: stretch; height: 60%; order: -1; }
  #Path { width: stretch; height: 40%; }
}
</style>
