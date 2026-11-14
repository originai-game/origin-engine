<meta name="component" content="PuzzleSummary">
<meta name="strings" content="./puzzle.en.ogt">
<meta name="parts" content="PuzzleSummary.frame PuzzleSummary.band PuzzleSummary.title PuzzleSummary.level PuzzleSummary.stars PuzzleSummary.earned PuzzleSummary.record PuzzleSummary.breakdown PuzzleSummary.row PuzzleSummary.rowLabel PuzzleSummary.rowValue PuzzleSummary.total PuzzleSummary.actions PuzzleSummary.next PuzzleSummary.retry PuzzleSummary.menu">
<meta name="prop:result" content="string" data-doc="How the level ended: win or lose.">
<meta name="prop:level" content="string" data-doc="The level's name as the game words it (Level 12); empty hides it.">
<meta name="prop:score" content="number" data-doc="The final score, as the game counts it.">
<meta name="prop:scoreMax" content="number" data-default="1" data-doc="The score of the star bar's end (its last threshold).">
<meta name="prop:thresholds" content="list" data-doc="The star bar's thresholds, keyed rows of { id, at }: at a fraction of scoreMax (0 to 1), as the game's star rules place them.">
<meta name="prop:stars" content="number" data-doc="Stars the game awarded (0 to maxStars): spoken and written beside the bar.">
<meta name="prop:maxStars" content="number" data-default="3" data-doc="Stars the level can award.">
<meta name="prop:record" content="bool" data-doc="The game says the score is a new best: a badge with its word.">
<meta name="prop:rows" content="list" data-doc="The score breakdown, keyed rows of { id, label, value }: value already worded by the game (+2,400, ×3).">
<meta name="prop:total" content="string" data-doc="The breakdown's total as the game words it; empty hides the total row.">
<meta name="prop:canNext" content="bool" data-doc="Next is offered (a won level with a next one).">
<meta name="action:back" content="Map">
<meta name="fixture:default" content='{"result":"win","level":"Level 12","score":24300,"scoreMax":30000,"thresholds":[{"id":"1","at":0.3333},{"id":"2","at":0.6667},{"id":"3","at":1}],"stars":2,"maxStars":3,"record":false,"rows":[{"id":"clear","label":"Goals cleared","value":"18,300"},{"id":"moves","label":"Moves left × 6","value":"+6,000"}],"total":"24,300","canNext":true}'>
<meta name="fixture:full" content='{"result":"win","level":"Level 12","score":31500,"scoreMax":30000,"thresholds":[{"id":"1","at":0.3333},{"id":"2","at":0.6667},{"id":"3","at":1}],"stars":3,"maxStars":3,"record":true,"rows":[{"id":"clear","label":"Goals cleared","value":"21,000"},{"id":"moves","label":"Moves left × 7","value":"+7,000"},{"id":"combo","label":"Best combo","value":"+3,500"}],"total":"31,500","canNext":true}'>
<meta name="fixture:lose" content='{"result":"lose","level":"Level 12","score":6400,"scoreMax":30000,"thresholds":[{"id":"1","at":0.3333},{"id":"2","at":0.6667},{"id":"3","at":1}],"stars":0,"maxStars":3,"record":false,"rows":[{"id":"clear","label":"Goals cleared","value":"6,400"}],"total":"","canNext":false}'>
<meta name="fixture:empty" content='{"result":"win","level":"","score":0,"scoreMax":30000,"thresholds":[],"stars":0,"maxStars":3,"record":false,"rows":[],"total":"","canNext":false}'>
<meta name="fixture:overflow" content='{"result":"win","level":"Level 1016: The Very Long Caramel Causeway Beyond the Whispering Lemonade Falls","score":123456789,"scoreMax":150000000,"thresholds":[{"id":"1","at":0.3333},{"id":"2","at":0.6667},{"id":"3","at":1}],"stars":2,"maxStars":3,"record":true,"rows":[{"id":"a","label":"Goals cleared across every layer of frosting and caramel","value":"98,000,000"},{"id":"b","label":"Moves left × 12","value":"+12,000,000"},{"id":"c","label":"Best combo","value":"+8,456,789"},{"id":"d","label":"Special sweets","value":"+3,000,000"},{"id":"e","label":"Jelly bonus","value":"+1,500,000"},{"id":"f","label":"Daily streak","value":"+500,000"}],"total":"123,456,789","canNext":true}'>

<!--
  The end of a level: won or failed, the level's name, the stars on the
  shared StarBar with the stars the game awarded in words, a new-best badge,
  the score breakdown the game words, and the next steps. Reports next
  (only when canNext), retry and menu (also Escape or pad B). The game
  computes the score, the stars and what comes next.
-->
<section id="PuzzleSummary" part="PuzzleSummary.frame" data-result={result} aria-label={result == 'win' ? t("puzzle.summary.win") : t("puzzle.summary.lose")} onback={menu}>
  <header part="PuzzleSummary.band" data-result={result}>
    <h2 part="PuzzleSummary.title">{result == 'win' ? t("puzzle.summary.win") : t("puzzle.summary.lose")}</h2>
    <p part="PuzzleSummary.level" hidden={!level}>{level}</p>
  </header>
  <div part="PuzzleSummary.stars" data-result={result}>
    <StarBar value={score} max={scoreMax} stars={thresholds} label="" valueText="" compact={false} />
    <span part="PuzzleSummary.earned" class="earned">{t("puzzle.summary.stars", { stars: stars, max: maxStars })}</span>
    <span part="PuzzleSummary.record" hidden={!record}>{t("puzzle.summary.record")}</span>
  </div>
  <scroll part="PuzzleSummary.breakdown" role="table" aria-label={t("puzzle.summary.breakdown")} hidden={!rows.length}>
    {#each rows as r (r.id)}
      <div part="PuzzleSummary.row" role="row">
        <span part="PuzzleSummary.rowLabel" role="rowheader">{r.label}</span>
        <span part="PuzzleSummary.rowValue" role="cell">{r.value}</span>
      </div>
    {/each}
    <div part="PuzzleSummary.total" role="row" hidden={!total}>
      <span class="total-label" role="rowheader">{t("puzzle.summary.total")}</span>
      <span class="total-value" role="cell">{total}</span>
    </div>
  </scroll>
  <div part="PuzzleSummary.actions">
    <button id="Menu" part="PuzzleSummary.menu" aria-label={t("puzzle.summary.menu")} onclick={menu}>{t("puzzle.summary.menu")}</button>
    <button id="Retry" part="PuzzleSummary.retry" class:primary={!canNext} aria-label={t("puzzle.summary.retry")} onclick={retry}>{t("puzzle.summary.retry")}</button>
    <button id="Next" part="PuzzleSummary.next" class="primary" aria-label={t("puzzle.summary.next")} hidden={!canNext} disabled={!canNext} onclick={next}>{t("puzzle.summary.next")}</button>
  </div>
</section>

<style>
[part="PuzzleSummary.frame"] [hidden], [part="PuzzleSummary.actions"] [hidden] { display: none; }
[part="PuzzleSummary.frame"] { display: flex; flex-direction: column; width: stretch; max-height: stretch; min-height: 0; gap: var(--space-md); padding: var(--space-lg); background: var(--panel); }
[part="PuzzleSummary.band"] { display: flex; flex: none; flex-direction: column; align-items: center; gap: var(--space-xxs); padding: var(--space-sm) var(--space-lg); }
[part="PuzzleSummary.title"] { margin: 0; text-align: center; }
[part="PuzzleSummary.level"] { margin: 0; width: stretch; text-align: center; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
[part="PuzzleSummary.stars"] { display: flex; flex: none; width: stretch; flex-direction: column; align-items: center; gap: var(--space-xs); }
[part="PuzzleSummary.earned"], [part="PuzzleSummary.record"] { white-space: nowrap; }
[part="PuzzleSummary.record"] { padding: var(--space-xxs) var(--space-sm); }
[part="PuzzleSummary.breakdown"] { width: stretch; flex: 1 1 auto; min-height: 0; }
[part="PuzzleSummary.row"], [part="PuzzleSummary.total"] { display: flex; width: stretch; align-items: baseline; gap: var(--space-md); padding: var(--space-xs) var(--space-sm); }
[part="PuzzleSummary.rowLabel"], .total-label { flex: 1; min-width: 0; text-align: start; }
[part="PuzzleSummary.rowValue"], .total-value { flex: none; text-align: end; white-space: nowrap; }
[part="PuzzleSummary.actions"] { display: flex; flex: none; width: stretch; gap: var(--space-sm); justify-content: center; }
[part="PuzzleSummary.actions"] button { flex: 1; min-width: 0; min-height: var(--size-touch); }
@media (width < 600px) {
  [part="PuzzleSummary.frame"] { padding: var(--space-md); gap: var(--space-sm); }
  [part="PuzzleSummary.actions"] { gap: var(--space-xs); }
}
</style>
