<meta name="component" content="PuzzleSummary">
<meta name="strings" content="./puzzle.en.ogt">
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
<section class="frame" id="PuzzleSummary" part="card" layout="stack" gap="loose" aria-label={result == 'win' ? t("puzzle.summary.win") : t("puzzle.summary.lose")} onback={menu} class:result-lose={(result) == 'lose'}>
  <header class="band" part="band" layout="stack" gap="hair" class:result-lose={(result) == 'lose'}>
    <h2 class="title" part="title">{result == 'win' ? t("puzzle.summary.win") : t("puzzle.summary.lose")}</h2>
    {#if level}<p class="level label" part="caption">{level}</p>{/if}
  </header>
  <div class="stars" part="stars" layout="stack" gap="tight" class:result-lose={(result) == 'lose'}>
    {#if thresholds.length}<div class="medals">{#each thresholds as th, i (th.id)}<svg class="medal" class:lit={i < stars} class:centre={i == 1 && thresholds.length == 3} style:--i={i} viewBox="0 0 24 24"><path d="M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z"></path></svg>{/each}</div>{/if}
    <StarBar value={score} max={scoreMax} stars={thresholds} label="" valueText="" compact={false} --tag-halo="none" />
    <span part="caption" class="earned label">{t("puzzle.summary.stars", { stars: stars, max: maxStars })}</span>
    {#if record}<span class="record label" part="badge" tone="success">{t("puzzle.summary.record")}</span>{/if}
  </div>
  {#if rows.length}<div class="breakdown-shell" layout="overlay" fill>
  <div class="breakdown" part="breakdown" layout="scroll" role="table" aria-label={t("puzzle.summary.breakdown")}>
    {#each rows as r (r.id)}
      <div class="row" part="row" layout="row" gap="loose" role="row">
        <span class="rowLabel" part="rowLabel" fill role="rowheader">{r.label}</span>
        <span class="row-value rowValue" part="figure" role="cell">{r.value}</span>
      </div>
    {/each}
    {#if total}<div class="total label row" part="row" layout="row" gap="loose" role="row">
      <span class="total-label" fill role="rowheader">{t("puzzle.summary.total")}</span>
      <span class="total-value" part="figure" role="cell">{total}</span>
    </div>{/if}
  </div>
  </div>{/if}
  <div class="actions" part="actions" layout="wrap" gap="tight">
    <button id="Menu" part="menu" fill aria-label={t("puzzle.summary.menu")} onclick={menu}>{t("puzzle.summary.menu")}</button>
    <button id="Retry" part="retry" fill class:primary={!canNext} aria-label={t("puzzle.summary.retry")} onclick={retry}>{t("puzzle.summary.retry")}</button>
    {#if canNext}<button id="Next" part="next" fill class="go" variant="primary" aria-label={t("puzzle.summary.next")} onclick={next}>{t("puzzle.summary.next")}</button>{/if}
  </div>
</section>

<style>
/* The end of a level on a cut card of dense smoked glass that holds its
   white text over the brightest key art: the verdict in
   the result's lit fill, the level's name, the stars the game awarded as
   large lit glyphs (the middle one raised, each popping in turn), the
   score on the star bar, the breakdown in a recess, and Next in green. */
#PuzzleSummary { inline-size: stretch; max-inline-size: calc(var(--measure-lg) * 1.1); max-block-size: stretch; min-block-size: 0; padding: var(--space-xl) var(--space-xl) var(--space-lg); --layout-switch-at: var(--measure-xs); background: var(--result-card-fill); box-shadow: var(--hud-plate-rim); border-radius: var(--result-card-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.band { flex: none; align-items: center; padding: 0; border: 0; min-block-size: 0; background: none; }
.title { margin: 0; text-align: center; font-size: var(--result-size-title-compact); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-title-tracking); text-transform: var(--hud-label-transform); color: transparent; background: var(--result-won-fill); background-clip: var(--hud-text-clip); filter: var(--result-won-glow); }
.result-lose .title { background: var(--result-lost-fill); background-clip: var(--hud-text-clip); filter: var(--result-lost-glow); }
.level { margin: 0; align-self: stretch; text-align: center; line-clamp: 2; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); }
.stars { flex: none; align-items: center; }
/* The stars: the style's star glyph, an earned one filled and glowing. */
.medals { display: flex; align-items: flex-end; justify-content: center; gap: var(--space-sm); }
.medal { inline-size: var(--result-size-star); block-size: var(--result-size-star); fill: var(--star-empty-fill); stroke: var(--star-empty-edge); stroke-width: var(--star-edge-width); stroke-linejoin: round; }
.medal.centre { inline-size: var(--result-size-star-lead); block-size: var(--result-size-star-lead); margin-block-end: var(--space-md); }
.medal.lit { fill: var(--star-fill-solid); stroke: var(--star-edge); filter: var(--star-glow); animation: var(--pop-motion); animation-delay: var(--result-star-delay); }
.earned, .record { white-space: nowrap; }
.earned { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); }
.record { padding: 0 var(--space-sm); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--result-medal-ink); background: var(--hud-key-gold-fill); box-shadow: none; border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.breakdown-shell { flex-basis: auto; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); }
/* The breakdown scrolls only when it has more than fits: its edge then fades
   into the recess, so more below shows, and at rest its last row is whole. */
.breakdown { padding: var(--space-xs) var(--space-md); -og-scroll-fade: var(--space-xl); }
.total { margin-block-start: var(--space-xs); border-block-start: var(--result-divider); }
.row { align-items: baseline; padding: var(--space-xs) 0; background: none; }
.row:hover { background: none; }
.rowLabel, .total-label { text-align: start; color: var(--hud-ink); }
.total-label { font-weight: var(--hud-figure-weight); }
.row-value, .total-value { flex: none; text-align: end; white-space: nowrap; font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink); }
.total-value { font-size: var(--hud-size-md); color: var(--hud-gold); }
.actions { flex: none; justify-content: center; }
.actions button { min-height: var(--size-touch); }
/* Next: the way on, in green. */
#PuzzleSummary:size(compact) { padding: var(--space-lg) var(--space-md) var(--space-md); --layout-gap-loose: var(--layout-gap-tight); }
.actions:size(compact) { --layout-gap-tight: var(--layout-gap-hair); }
.medal:size(compact) { inline-size: var(--result-size-star-compact); block-size: var(--result-size-star-compact); }
.medal.centre:size(compact) { inline-size: var(--result-size-star-lead-compact); block-size: var(--result-size-star-lead-compact); margin-block-end: var(--space-sm); }
</style>
