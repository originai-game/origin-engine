<meta name="component" content="PuzzleSummary">
<meta name="strings" content="./puzzle.en.ogt">
<meta name="parts" content="PuzzleSummary.frame PuzzleSummary.band PuzzleSummary.title PuzzleSummary.level PuzzleSummary.stars PuzzleSummary.earned PuzzleSummary.record PuzzleSummary.breakdown PuzzleSummary.row PuzzleSummary.rowLabel PuzzleSummary.rowValue PuzzleSummary.total PuzzleSummary.actions PuzzleSummary.next PuzzleSummary.retry PuzzleSummary.menu PuzzleSummary.label PuzzleSummary.card PuzzleSummary.caption PuzzleSummary.badge PuzzleSummary.figure PuzzleSummary.fade">
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
<section id="PuzzleSummary" part="PuzzleSummary.frame PuzzleSummary.card" data-layout="stack" data-gap="loose" data-result={result} aria-label={result == 'win' ? t("puzzle.summary.win") : t("puzzle.summary.lose")} onback={menu}>
  <header part="PuzzleSummary.band" data-layout="stack" data-gap="hair" data-result={result}>
    <h2 part="PuzzleSummary.title">{result == 'win' ? t("puzzle.summary.win") : t("puzzle.summary.lose")}</h2>
    {#if level}<p class="level" part="PuzzleSummary.level PuzzleSummary.label PuzzleSummary.caption">{level}</p>{/if}
  </header>
  <div part="PuzzleSummary.stars" data-layout="stack" data-gap="tight" data-result={result}>
    <StarBar value={score} max={scoreMax} stars={thresholds} label="" valueText="" compact={false} --tag-halo="none" />
    <span part="PuzzleSummary.earned PuzzleSummary.label PuzzleSummary.caption" class="earned">{t("puzzle.summary.stars", { stars: stars, max: maxStars })}</span>
    {#if record}<span class="record" part="PuzzleSummary.record PuzzleSummary.label PuzzleSummary.badge" data-tone="success">{t("puzzle.summary.record")}</span>{/if}
  </div>
  {#if rows.length}<div class="breakdown-shell" data-layout="overlay" data-fill>
  <div part="PuzzleSummary.breakdown" data-layout="scroll" role="table" aria-label={t("puzzle.summary.breakdown")}>
    {#each rows as r (r.id)}
      <div part="PuzzleSummary.row" data-layout="row" data-gap="loose" role="row">
        <span part="PuzzleSummary.rowLabel" data-fill role="rowheader">{r.label}</span>
        <span class="row-value" part="PuzzleSummary.rowValue PuzzleSummary.figure" role="cell">{r.value}</span>
      </div>
    {/each}
    {#if total}<div class="total" part="PuzzleSummary.total PuzzleSummary.label PuzzleSummary.row" data-layout="row" data-gap="loose" role="row">
      <span class="total-label" data-fill role="rowheader">{t("puzzle.summary.total")}</span>
      <span class="total-value" part="PuzzleSummary.figure" role="cell">{total}</span>
    </div>{/if}
  </div>
  <div class="breakdown-fade" part="PuzzleSummary.fade" data-anchor="bottom-edge"></div>
  </div>{/if}
  <div part="PuzzleSummary.actions" data-layout="switcher" data-gap="tight">
    <button id="Menu" part="PuzzleSummary.menu" data-fill aria-label={t("puzzle.summary.menu")} onclick={menu}>{t("puzzle.summary.menu")}</button>
    <button id="Retry" part="PuzzleSummary.retry" data-fill class:primary={!canNext} aria-label={t("puzzle.summary.retry")} onclick={retry}>{t("puzzle.summary.retry")}</button>
    {#if canNext}<button id="Next" part="PuzzleSummary.next" data-fill class="primary" aria-label={t("puzzle.summary.next")} onclick={next}>{t("puzzle.summary.next")}</button>{/if}
  </div>
</section>

<style>
:host { inline-size: stretch; max-block-size: stretch; min-block-size: 0; padding: var(--space-lg); --layout-switch-at: var(--measure-xs); }
[part="PuzzleSummary.band"] { flex: none; align-items: center; padding: var(--space-sm) var(--space-lg); }
[part="PuzzleSummary.title"] { margin: 0; text-align: center; }
.level { margin: 0; align-self: stretch; text-align: center; line-clamp: 2; }
[part="PuzzleSummary.stars"] { flex: none; align-items: center; }
.earned, .record { white-space: nowrap; }
.record { padding: var(--space-xs) var(--space-md); }
.breakdown-shell { flex-basis: auto; }
[part="PuzzleSummary.breakdown"] { padding-block-end: var(--fade-size); }
.total { margin-block-start: var(--space-sm); }
[part~="PuzzleSummary.row"] { align-items: baseline; padding: var(--space-sm); }
[part="PuzzleSummary.rowLabel"], .total-label { text-align: start; }
.row-value, .total-value { flex: none; text-align: end; white-space: nowrap; }
[part="PuzzleSummary.actions"] { flex: none; justify-content: center; }
[part="PuzzleSummary.actions"] button { min-height: var(--size-touch); }
@media (width < 600px) {
  :host { padding: var(--space-md); --layout-gap-loose: var(--layout-gap-tight); }
  [part="PuzzleSummary.actions"] { --layout-gap-tight: var(--layout-gap-hair); }
}
</style>
