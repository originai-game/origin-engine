<meta name="component" content="PuzzleStatesPage">

<!-- Every puzzle component in every fixture state it declares, a captioned
     cell each, in a grid that scrolls (docs/ui.md §6.1). -->
<scroll id="Page">
  <div id="Cells">
    <section class="cell" data-member="goals"><span class="caption">{$captions.goals_default}</span><div class="stage"><PuzzleGoals goals={$goals_default_goals} counter={$goals_default.counter} moves={$goals_default.moves} seconds={$goals_default.seconds} low={$goals_default.low} score={$goals_default.score} /></div></section>
    <section class="cell" data-member="goals"><span class="caption">{$captions.goals_time}</span><div class="stage"><PuzzleGoals goals={$goals_time_goals} counter={$goals_time.counter} moves={$goals_time.moves} seconds={$goals_time.seconds} low={$goals_time.low} score={$goals_time.score} /></div></section>
    <section class="cell" data-member="goals"><span class="caption">{$captions.goals_low}</span><div class="stage"><PuzzleGoals goals={$goals_low_goals} counter={$goals_low.counter} moves={$goals_low.moves} seconds={$goals_low.seconds} low={$goals_low.low} score={$goals_low.score} /></div></section>
    <section class="cell" data-member="goals"><span class="caption">{$captions.goals_lowTime}</span><div class="stage"><PuzzleGoals goals={$goals_lowTime_goals} counter={$goals_lowTime.counter} moves={$goals_lowTime.moves} seconds={$goals_lowTime.seconds} low={$goals_lowTime.low} score={$goals_lowTime.score} /></div></section>
    <section class="cell" data-member="goals"><span class="caption">{$captions.goals_done}</span><div class="stage"><PuzzleGoals goals={$goals_done_goals} counter={$goals_done.counter} moves={$goals_done.moves} seconds={$goals_done.seconds} low={$goals_done.low} score={$goals_done.score} /></div></section>
    <section class="cell" data-member="goals"><span class="caption">{$captions.goals_empty}</span><div class="stage"><PuzzleGoals goals={$goals_empty_goals} counter={$goals_empty.counter} moves={$goals_empty.moves} seconds={$goals_empty.seconds} low={$goals_empty.low} score={$goals_empty.score} /></div></section>
    <section class="cell" data-member="goals"><span class="caption">{$captions.goals_overflow}</span><div class="stage"><PuzzleGoals goals={$goals_overflow_goals} counter={$goals_overflow.counter} moves={$goals_overflow.moves} seconds={$goals_overflow.seconds} low={$goals_overflow.low} score={$goals_overflow.score} /></div></section>
    <section class="cell" data-member="boosters"><span class="caption">{$captions.boosters_default}</span><div class="stage"><PuzzleBoosters boosters={$boosters_default_boosters} armed={$boosters_default.armed} /></div></section>
    <section class="cell" data-member="boosters"><span class="caption">{$captions.boosters_selected}</span><div class="stage"><PuzzleBoosters boosters={$boosters_selected_boosters} armed={$boosters_selected.armed} /></div></section>
    <section class="cell" data-member="boosters"><span class="caption">{$captions.boosters_disabled}</span><div class="stage"><PuzzleBoosters boosters={$boosters_disabled_boosters} armed={$boosters_disabled.armed} /></div></section>
    <section class="cell" data-member="boosters"><span class="caption">{$captions.boosters_empty}</span><div class="stage"><PuzzleBoosters boosters={$boosters_empty_boosters} armed={$boosters_empty.armed} /></div></section>
    <section class="cell" data-member="boosters"><span class="caption">{$captions.boosters_overflow}</span><div class="stage"><PuzzleBoosters boosters={$boosters_overflow_boosters} armed={$boosters_overflow.armed} /></div></section>
    <section class="cell" data-member="level-path"><span class="caption">{$captions.levelPath_default}</span><div class="stage"><PuzzleLevelPath chapters={$levelPath_default_chapters} levels={$levelPath_default_levels} selected={$levelPath_default.selected} /></div></section>
    <section class="cell" data-member="level-path"><span class="caption">{$captions.levelPath_empty}</span><div class="stage"><PuzzleLevelPath chapters={$levelPath_empty_chapters} levels={$levelPath_empty_levels} selected={$levelPath_empty.selected} /></div></section>
    <section class="cell" data-member="level-path"><span class="caption">{$captions.levelPath_selected}</span><div class="stage"><PuzzleLevelPath chapters={$levelPath_selected_chapters} levels={$levelPath_selected_levels} selected={$levelPath_selected.selected} /></div></section>
    <section class="cell" data-member="level-path"><span class="caption">{$captions.levelPath_locked}</span><div class="stage"><PuzzleLevelPath chapters={$levelPath_locked_chapters} levels={$levelPath_locked_levels} selected={$levelPath_locked.selected} /></div></section>
    <section class="cell" data-member="level-path"><span class="caption">{$captions.levelPath_overflow}</span><div class="stage"><PuzzleLevelPath chapters={$levelPath_overflow_chapters} levels={$levelPath_overflow_levels} selected={$levelPath_overflow.selected} /></div></section>
    <section class="cell" data-member="summary"><span class="caption">{$captions.summary_default}</span><div class="stage"><PuzzleSummary result={$summary_default.result} level={$summary_default.level} score={$summary_default.score} scoreMax={$summary_default.scoreMax} thresholds={$summary_default_thresholds} stars={$summary_default.stars} maxStars={$summary_default.maxStars} record={$summary_default.record} rows={$summary_default_rows} total={$summary_default.total} canNext={$summary_default.canNext} /></div></section>
    <section class="cell" data-member="summary"><span class="caption">{$captions.summary_full}</span><div class="stage"><PuzzleSummary result={$summary_full.result} level={$summary_full.level} score={$summary_full.score} scoreMax={$summary_full.scoreMax} thresholds={$summary_full_thresholds} stars={$summary_full.stars} maxStars={$summary_full.maxStars} record={$summary_full.record} rows={$summary_full_rows} total={$summary_full.total} canNext={$summary_full.canNext} /></div></section>
    <section class="cell" data-member="summary"><span class="caption">{$captions.summary_lose}</span><div class="stage"><PuzzleSummary result={$summary_lose.result} level={$summary_lose.level} score={$summary_lose.score} scoreMax={$summary_lose.scoreMax} thresholds={$summary_lose_thresholds} stars={$summary_lose.stars} maxStars={$summary_lose.maxStars} record={$summary_lose.record} rows={$summary_lose_rows} total={$summary_lose.total} canNext={$summary_lose.canNext} /></div></section>
    <section class="cell" data-member="summary"><span class="caption">{$captions.summary_empty}</span><div class="stage"><PuzzleSummary result={$summary_empty.result} level={$summary_empty.level} score={$summary_empty.score} scoreMax={$summary_empty.scoreMax} thresholds={$summary_empty_thresholds} stars={$summary_empty.stars} maxStars={$summary_empty.maxStars} record={$summary_empty.record} rows={$summary_empty_rows} total={$summary_empty.total} canNext={$summary_empty.canNext} /></div></section>
    <section class="cell" data-member="summary"><span class="caption">{$captions.summary_overflow}</span><div class="stage"><PuzzleSummary result={$summary_overflow.result} level={$summary_overflow.level} score={$summary_overflow.score} scoreMax={$summary_overflow.scoreMax} thresholds={$summary_overflow_thresholds} stars={$summary_overflow.stars} maxStars={$summary_overflow.maxStars} record={$summary_overflow.record} rows={$summary_overflow_rows} total={$summary_overflow.total} canNext={$summary_overflow.canNext} /></div></section>
  </div>
</scroll>

<style>
#Page { width: stretch; height: stretch; }
#Cells { display: grid; grid-template-columns: repeat(auto-fill, minmax(calc(var(--size-xxl) * 4.4), 1fr)); gap: var(--space-lg); padding: var(--space-lg); }
.cell { display: flex; flex-direction: column; gap: var(--space-xs); min-width: 0; }
.cell[data-member=goals] { grid-column: span 2; }
.caption { font: var(--text-caption); color: var(--ink-muted); }
.stage { display: flex; min-width: 0; min-height: calc(var(--size-xxl) * 1.6); align-items: flex-start; padding: var(--space-sm); border-radius: var(--radius-sm); background: var(--well); }
.cell[data-member=level-path] .stage, .cell[data-member=summary] .stage { height: calc(var(--size-xxl) * 6); }
@media (width < 600px) {
  #Cells { grid-template-columns: minmax(0, 1fr); padding: var(--space-sm); gap: var(--space-md); }
  .cell[data-member=goals] { grid-column: span 1; }
  .cell[data-member=level-path] .stage, .cell[data-member=summary] .stage { height: calc(var(--size-xxl) * 7); }
}
</style>
