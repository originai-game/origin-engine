<meta name="component" content="RhythmDifficulty">
<meta name="strings" content="./rhythm.en.ogt">
<meta name="parts" content="RhythmDifficulty.frame RhythmDifficulty.band RhythmDifficulty.title RhythmDifficulty.row RhythmDifficulty.name RhythmDifficulty.cell RhythmDifficulty.badge">
<meta name="prop:title" content="string" data-doc="The heading (the song's title); empty takes the kit's (Difficulty).">
<meta name="prop:levels" content="list" data-doc="The song's charts in the game's order, keyed rows of { id, tier, label, rating, grade, score, locked }: tier the chart's family (easy, normal, hard, expert, master, special; on data-tier for the pack's colour, never alone); label its name as the game words it (Hard); rating its level number; grade the player's best grade on it (S, AA; empty unplayed); score the best score as the game formats it (981,200; empty Not played); locked it cannot be played yet (disabled).">
<meta name="prop:selected" content="string" data-doc="The chart the game holds as chosen (selected, named Selected); empty is none.">
<meta name="fixture:default" content='{"title":"","selected":"hard","levels":[{"id":"easy","tier":"easy","label":"Easy","rating":3,"grade":"S","score":"995,120","locked":false},{"id":"normal","tier":"normal","label":"Normal","rating":6,"grade":"A","score":"912,400","locked":false},{"id":"hard","tier":"hard","label":"Hard","rating":9,"grade":"","score":"","locked":false},{"id":"expert","tier":"expert","label":"Expert","rating":12,"grade":"","score":"","locked":true}]}'>
<meta name="fixture:selected" content='{"title":"Skyline Drive","selected":"easy","levels":[{"id":"easy","tier":"easy","label":"Easy","rating":3,"grade":"S","score":"995,120","locked":false},{"id":"normal","tier":"normal","label":"Normal","rating":6,"grade":"A","score":"912,400","locked":false}]}'>
<meta name="fixture:disabled" content='{"title":"","selected":"","levels":[{"id":"expert","tier":"expert","label":"Expert","rating":12,"grade":"","score":"","locked":true},{"id":"master","tier":"master","label":"Master","rating":14,"grade":"","score":"","locked":true}]}'>
<meta name="fixture:empty" content='{"title":"","selected":"","levels":[]}'>
<meta name="fixture:overflow" content='{"title":"The Unbearably Long Extended Twelve-Minute Midnight Orchestral Remix of a Song","selected":"special","levels":[{"id":"beginner","tier":"easy","label":"Beginner","rating":1,"grade":"SSS+","score":"1,000,000","locked":false},{"id":"easy","tier":"easy","label":"Easy","rating":4,"grade":"SS","score":"998,870","locked":false},{"id":"normal","tier":"normal","label":"Normal","rating":7,"grade":"S","score":"975,010","locked":false},{"id":"hard","tier":"hard","label":"Hard","rating":10,"grade":"AA","score":"931,445","locked":false},{"id":"expert","tier":"expert","label":"Expert","rating":13,"grade":"B","score":"801,992","locked":false},{"id":"master","tier":"master","label":"Master","rating":15,"grade":"","score":"","locked":false},{"id":"special","tier":"special","label":"Remaster Ultimate Challenge","rating":16.5,"grade":"","score":"","locked":false},{"id":"world","tier":"special","label":"Final Ascent","rating":99,"grade":"","score":"","locked":true}]}'>

<!--
  Choosing a chart: one tile per difficulty with its name, level, the
  player's best grade and score, and Locked or Selected in words. A tile
  reports choose with its key; a locked chart is disabled and reports
  nothing. The game owns the charts, unlocking, grades and scores.
-->
<section id="RhythmDifficulty" part="RhythmDifficulty.frame" aria-label={title != '' ? title : t("rhythm.difficulty.title")}>
  <header part="RhythmDifficulty.band"><h2 part="RhythmDifficulty.title">{title != '' ? title : t("rhythm.difficulty.title")}</h2></header>
  {#if levels.length}
    <scroll class="list">
      <div class="tiles" role="listbox" aria-label={t("rhythm.difficulty.list")}>
        {#each levels as l (l.id)}
          <button part="RhythmDifficulty.row" role="option" aria-selected={l.id == selected} aria-label={t("rhythm.difficulty.ratingName", { label: l.label, rating: l.rating })} disabled={l.locked} data-tier={l.tier} data-locked={l.locked} onclick={choose}><div class="face">
            <span part="RhythmDifficulty.name" class="name">{l.label}</span>
            <span part="RhythmDifficulty.cell" class="rating">{t("rhythm.difficulty.rating", { rating: l.rating })}</span>
            <div class="best">
              <span part="RhythmDifficulty.badge" class="grade" data-tag="grade" hidden={l.grade == ''}>{l.grade}</span>
              <span class="score">{l.score != '' ? l.score : t("rhythm.difficulty.unplayed")}</span>
            </div>
            <span part="RhythmDifficulty.badge" data-tag="locked" hidden={!l.locked}>{t("rhythm.difficulty.locked")}</span>
            <span part="RhythmDifficulty.badge" data-tag="selected" hidden={l.id != selected}>{t("rhythm.difficulty.selected")}</span>
          </div></button>
        {/each}
      </div>
    </scroll>
  {:else}
    <p class="empty-text" role="status">{t("rhythm.difficulty.empty")}</p>
  {/if}
</section>

<style>
[part="RhythmDifficulty.frame"][hidden], [part="RhythmDifficulty.frame"] [hidden] { display: none; }
[part="RhythmDifficulty.frame"] { display: flex; flex-direction: column; width: stretch; min-width: 0; gap: var(--space-sm); }
[part="RhythmDifficulty.band"] { display: flex; width: stretch; align-items: center; flex: none; }
[part="RhythmDifficulty.title"] { margin: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.list { width: stretch; min-width: 0; min-height: 0; flex: 0 1 auto; }
.tiles { display: flex; width: stretch; flex-wrap: wrap; gap: var(--space-sm); padding: var(--space-xxs); }
[part="RhythmDifficulty.row"] { flex: 1; min-width: calc(var(--size-xxl) * 1.6); max-width: calc(var(--size-xxl) * 2.6); min-height: var(--size-touch); padding: var(--space-sm); }
.face { display: flex; flex-direction: column; width: stretch; align-items: flex-start; gap: var(--space-xxs); }
.name { max-width: stretch; text-align: start; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.rating { align-self: flex-start; text-align: start; white-space: nowrap; font-variant-numeric: tabular-nums; }
.best { display: flex; max-width: stretch; align-items: center; gap: var(--space-xs); }
.score { min-width: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; font-variant-numeric: tabular-nums; }
[part="RhythmDifficulty.badge"] { flex: none; white-space: nowrap; padding: 0 var(--space-xs); }
.empty-text { margin: 0; padding: var(--space-md); text-align: center; }
/* A phone lists the charts one under another, each a full-width row. */
@media (width < 600px) {
  .tiles { flex-direction: column; flex-wrap: nowrap; gap: var(--space-xs); }
  [part="RhythmDifficulty.row"] { width: stretch; min-width: 0; max-width: none; padding: var(--space-xs) var(--space-sm); }
  .face { flex-direction: row; flex-wrap: wrap; align-items: center; gap: var(--space-xxs) var(--space-sm); }
  .name { flex: 1 1 auto; min-width: calc(var(--size-xxl) * 1.2); }
  .rating { align-self: center; }
}
</style>
