<meta name="component" content="PuzzleMapPage">

<!-- Between levels, over the game's level map art (sample art of the sheet
     only): a two-chapter level path (a long chapter name, levels past a
     thousand) beside a won level's summary with a new best and a six-row
     breakdown. A phone: the path alone takes the screen, as a level map
     does; the summary is on the states page. -->
<div id="Page">
  <img id="Map" src={$meadow} alt="">
  <div id="Path"><PuzzleLevelPath selected={$levelPath_overflow.selected} chapters={$levelPath_overflow_chapters} levels={$levelPath_overflow_levels} /></div>
  <div id="Shade"></div>
  <div id="Summary"><PuzzleSummary result={$summary_overflow.result} level={$summary_overflow.level} score={$summary_overflow.score} scoreMax={$summary_overflow.scoreMax} thresholds={$summary_overflow_thresholds} stars={$summary_overflow.stars} maxStars={$summary_overflow.maxStars} record={$summary_overflow.record} rows={$summary_overflow_rows} total={$summary_overflow.total} canNext={$summary_overflow.canNext} /></div>
</div>

<style>
#Page { position: relative; display: flex; width: stretch; height: stretch; gap: var(--space-lg); padding: calc(env(-og-hud-inset-top) + var(--space-xl)) calc(env(-og-hud-inset-right) + var(--space-xl)) calc(env(-og-hud-inset-bottom) + var(--space-xl)) calc(env(-og-hud-inset-left) + var(--space-xl)); justify-content: center; }
#Map { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; }
#Path { position: relative; width: 45%; height: stretch; min-height: 0; }
#Summary { position: relative; width: 40%; height: stretch; min-height: 0; }
/* A results panel over key art: the game shades the art behind it so its
   white text holds whatever the art is (the meadow's sky, river and
   flowers), and leaves the map path's side at full brightness. */
#Shade { position: absolute; inset-block: 0; inset-inline-end: 0; width: 58%; background: linear-gradient(90deg, rgb(4 8 16 / 0), rgb(4 8 16 / 0.78) 22%, rgb(4 8 16 / 0.84)); }
#Shade:dir(rtl) { background: linear-gradient(270deg, rgb(4 8 16 / 0), rgb(4 8 16 / 0.78) 22%, rgb(4 8 16 / 0.84)); }
@media (width < 600px) {
  #Page { padding: calc(env(-og-hud-inset-top) + var(--space-sm)) calc(env(-og-hud-inset-right) + var(--space-sm)) calc(env(-og-hud-inset-bottom) + var(--space-sm)) calc(env(-og-hud-inset-left) + var(--space-sm)); }
  #Summary, #Shade { display: none; }
  #Path { width: stretch; }
}
</style>
