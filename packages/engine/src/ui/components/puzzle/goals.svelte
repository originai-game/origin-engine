<meta name="component" content="PuzzleGoals">
<meta name="strings" content="./puzzle.en.ogt">
<meta name="parts" content="PuzzleGoals.frame PuzzleGoals.counter PuzzleGoals.label PuzzleGoals.figure PuzzleGoals.warning PuzzleGoals.goals PuzzleGoals.goal PuzzleGoals.count PuzzleGoals.done PuzzleGoals.score">
<meta name="prop:goals" content="list" data-doc="The level's goals in order, keyed rows of { id, icon, label, left, done }: left how many are still to clear, as the game counts them; done the goal is met (a check replaces the count).">
<meta name="prop:counter" content="string" data-doc="What limits the level: moves (a move count) or time (a countdown).">
<meta name="prop:moves" content="number" data-doc="Moves left, when counter is moves.">
<meta name="prop:seconds" content="number" data-doc="Seconds left, when counter is time.">
<meta name="prop:low" content="bool" data-doc="The game says the moves or time are running low: the counter warns, in words as well as its dress.">
<meta name="prop:score" content="number" data-doc="The score so far, as the game counts it.">
<meta name="fixture:default" content='{"counter":"moves","moves":18,"seconds":0,"low":false,"score":12450,"goals":[{"id":"red","icon":"apple","label":"Red sweets","left":12,"done":false},{"id":"jelly","icon":"drop","label":"Jelly","left":4,"done":false},{"id":"leaf","icon":"leaf","label":"Leaves","left":0,"done":true}]}'>
<meta name="fixture:time" content='{"counter":"time","moves":0,"seconds":74,"low":false,"score":3200,"goals":[{"id":"gem","icon":"gem","label":"Gems","left":20,"done":false}]}'>
<meta name="fixture:low" content='{"counter":"moves","moves":3,"seconds":0,"low":true,"score":8800,"goals":[{"id":"red","icon":"apple","label":"Red sweets","left":5,"done":false},{"id":"leaf","icon":"leaf","label":"Leaves","left":0,"done":true}]}'>
<meta name="fixture:lowTime" content='{"counter":"time","moves":0,"seconds":7,"low":true,"score":15000,"goals":[{"id":"gem","icon":"gem","label":"Gems","left":2,"done":false}]}'>
<meta name="fixture:done" content='{"counter":"moves","moves":6,"seconds":0,"low":false,"score":24300,"goals":[{"id":"red","icon":"apple","label":"Red sweets","left":0,"done":true},{"id":"jelly","icon":"drop","label":"Jelly","left":0,"done":true}]}'>
<meta name="fixture:empty" content='{"counter":"moves","moves":25,"seconds":0,"low":false,"score":0,"goals":[]}'>
<meta name="fixture:overflow" content='{"counter":"moves","moves":1250,"seconds":0,"low":false,"score":123456789,"goals":[{"id":"a","icon":"apple","label":"Red sweets","left":1250,"done":false},{"id":"b","icon":"drop","label":"Jelly","left":99,"done":false},{"id":"c","icon":"leaf","label":"Leaves","left":0,"done":true},{"id":"d","icon":"gem","label":"Gems","left":340,"done":false},{"id":"e","icon":"star","label":"Stars","left":7,"done":false},{"id":"f","icon":"key","label":"Keys","left":3,"done":false}]}'>

<!--
  The in-level HUD: what limits the level (moves or time left) with a
  warning when the game says it runs low, the goals as icons with what is
  left of each and a check once met, and the score. It draws what the game
  counts; it decides nothing. Over the board it reads as one plate; where
  it is given less than a phone's width (its own, not the screen's) the
  goals wrap under the counter and the score. The low counter's and the
  met goal's looks, and any pulse, are the pack's.
-->
<section id="PuzzleGoals" part="PuzzleGoals.frame" class="hud" aria-label={t("puzzle.goals.name")} data-low={low}>
  <div class="hud">
    <div part="PuzzleGoals.counter" role={counter == 'time' ? 'timer' : 'status'} data-kind={counter} data-low={low} aria-label={counter == 'time' ? t("puzzle.goals.timeLeft", { time: formatCountdown(seconds) }) : t("puzzle.goals.movesLeft", { moves: moves })}>
      <span part="PuzzleGoals.label" class="label">{counter == 'time' ? t("puzzle.goals.time") : t("puzzle.goals.moves")}</span>
      <span part="PuzzleGoals.figure" data-low={low}>{counter == 'time' ? formatCountdown(seconds) : formatNumber(moves)}</span>
      <span part="PuzzleGoals.warning" hidden={!low}>{t("puzzle.goals.low")}</span>
    </div>
    <div part="PuzzleGoals.goals" role="list" aria-label={t("puzzle.goals.name")}>
      {#each goals as g (g.id)}
        <div part="PuzzleGoals.goal" role="listitem" data-done={g.done} aria-label={g.done ? t("puzzle.goals.done", { label: g.label }) : t("puzzle.goals.left", { label: g.label, left: g.left })}>
          <og-icon class="goal-art" icon={g.icon}></og-icon>
          <span part="PuzzleGoals.count" hidden={g.done}>{formatNumber(g.left)}</span>
          <svg part="PuzzleGoals.done" class="check" viewBox="0 0 24 24" hidden={!g.done}><path d="M4.5 12.5l5 5 10-11"></path></svg>
        </div>
      {:else}
        <p class="none">{t("puzzle.goals.none")}</p>
      {/each}
    </div>
    <div part="PuzzleGoals.score" role="status" aria-label={t("puzzle.goals.scoreName", { score: formatNumber(score) })}>
      <span class="score-label">{t("puzzle.goals.score")}</span>
      <span class="score-figure">{formatNumber(score)}</span>
    </div>
  </div>
</section>

<style>
[part="PuzzleGoals.frame"] [hidden] { display: none; }
[part="PuzzleGoals.frame"] { display: flex; width: stretch; container-type: inline-size; }
.hud { display: flex; width: stretch; align-items: stretch; gap: var(--space-md); padding: var(--space-sm) var(--space-md); }
[part="PuzzleGoals.counter"] { display: flex; flex: none; flex-direction: column; align-items: center; justify-content: center; min-width: calc(var(--size-xl) * 1.4); padding: var(--space-xs) var(--space-sm); }
[part="PuzzleGoals.label"], .score-label { white-space: nowrap; }
[part="PuzzleGoals.figure"] { white-space: nowrap; }
[part="PuzzleGoals.warning"] { white-space: nowrap; }
[part="PuzzleGoals.goals"] { display: flex; flex: 1; min-width: 0; flex-wrap: wrap; align-items: center; justify-content: center; gap: var(--space-sm) var(--space-md); }
[part="PuzzleGoals.goal"] { position: relative; display: flex; flex-direction: column; align-items: center; justify-content: center; width: var(--size-xl); padding: var(--space-xxs); }
.goal-art { width: var(--size-lg); height: var(--size-lg); }
[part="PuzzleGoals.count"] { white-space: nowrap; }
.check { width: var(--size-sm); height: var(--size-sm); fill: none; stroke-linecap: round; stroke-linejoin: round; }
.none { margin: 0; text-align: center; }
[part="PuzzleGoals.score"] { display: flex; flex: none; flex-direction: column; align-items: flex-end; justify-content: center; min-width: calc(var(--size-xl) * 1.6); padding: var(--space-xs) var(--space-sm); }
.score-figure { white-space: nowrap; }
/* The frame is the container, so a narrow panel on a wide screen wraps as a phone does. */
@container (width < 600px) {
  .hud { flex-wrap: wrap; gap: var(--space-xs) var(--space-sm); padding: var(--space-xs) var(--space-sm); }
  [part="PuzzleGoals.counter"] { min-width: var(--size-xl); }
  [part="PuzzleGoals.goals"] { order: 3; flex-basis: 100%; }
  [part="PuzzleGoals.score"] { flex: 1; min-width: 0; }
  [part="PuzzleGoals.goal"] { width: var(--size-lg); }
  .goal-art { width: var(--size-md); height: var(--size-md); }
}
</style>
