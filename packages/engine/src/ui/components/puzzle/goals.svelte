<meta name="component" content="PuzzleGoals">
<meta name="strings" content="./puzzle.en.ogt">
<meta name="parts" content="PuzzleGoals.frame PuzzleGoals.counter PuzzleGoals.label PuzzleGoals.figure PuzzleGoals.warning PuzzleGoals.goals PuzzleGoals.goal PuzzleGoals.socket PuzzleGoals.count PuzzleGoals.name PuzzleGoals.done PuzzleGoals.glyph PuzzleGoals.score PuzzleGoals.track PuzzleGoals.fill PuzzleGoals.star">
<meta name="prop:goals" content="list" data-doc="The level's goals in order, keyed rows of { id, icon, label, left, done }: left how many are still to clear, as the game counts them; done the goal is met (a check replaces the count).">
<meta name="prop:counter" content="string" data-doc="What limits the level: moves (a move count) or time (a countdown).">
<meta name="prop:moves" content="number" data-doc="Moves left, when counter is moves.">
<meta name="prop:seconds" content="number" data-doc="Seconds left, when counter is time.">
<meta name="prop:low" content="bool" data-doc="The game says the moves or time are running low: the counter warns, in words as well as its dress.">
<meta name="prop:score" content="number" data-doc="The score so far, as the game counts it.">
<meta name="prop:progress" content="number" data-default="-1" data-doc="The score toward the level's top star, 0 to 1, for the bar under it; negative hides the bar.">
<meta name="prop:stars" content="list" data-doc="The star marks on that bar, keyed rows of { id, at }: at where the star sits along the bar, 0 to 1; a star at or under progress is lit.">
<meta name="fixture:default" content='{"progress":0.62,"stars":[{"id":"s1","at":0.4},{"id":"s2","at":0.7},{"id":"s3","at":1}],"counter":"moves","moves":18,"seconds":0,"low":false,"score":12450,"goals":[{"id":"red","icon":"apple","label":"Red sweets","left":12,"done":false},{"id":"jelly","icon":"drop","label":"Jelly","left":4,"done":false},{"id":"leaf","icon":"leaf","label":"Leaves","left":0,"done":true}]}'>
<meta name="fixture:time" content='{"progress":0.2,"stars":[{"id":"s1","at":0.4},{"id":"s2","at":0.7},{"id":"s3","at":1}],"counter":"time","moves":0,"seconds":74,"low":false,"score":3200,"goals":[{"id":"gem","icon":"gem","label":"Gems","left":20,"done":false}]}'>
<meta name="fixture:low" content='{"progress":0.45,"stars":[{"id":"s1","at":0.4},{"id":"s2","at":0.7},{"id":"s3","at":1}],"counter":"moves","moves":3,"seconds":0,"low":true,"score":8800,"goals":[{"id":"red","icon":"apple","label":"Red sweets","left":5,"done":false},{"id":"leaf","icon":"leaf","label":"Leaves","left":0,"done":true}]}'>
<meta name="fixture:lowTime" content='{"progress":0.75,"stars":[{"id":"s1","at":0.4},{"id":"s2","at":0.7},{"id":"s3","at":1}],"counter":"time","moves":0,"seconds":7,"low":true,"score":15000,"goals":[{"id":"gem","icon":"gem","label":"Gems","left":2,"done":false}]}'>
<meta name="fixture:done" content='{"progress":1,"stars":[{"id":"s1","at":0.4},{"id":"s2","at":0.7},{"id":"s3","at":1}],"counter":"moves","moves":6,"seconds":0,"low":false,"score":24300,"goals":[{"id":"red","icon":"apple","label":"Red sweets","left":0,"done":true},{"id":"jelly","icon":"drop","label":"Jelly","left":0,"done":true}]}'>
<meta name="fixture:empty" content='{"progress":-1,"stars":[],"counter":"moves","moves":25,"seconds":0,"low":false,"score":0,"goals":[]}'>
<meta name="fixture:overflow" content='{"progress":1,"stars":[{"id":"s1","at":0.4},{"id":"s2","at":0.7},{"id":"s3","at":1}],"counter":"moves","moves":1250,"seconds":0,"low":false,"score":123456789,"goals":[{"id":"a","icon":"apple","label":"Red sweets","left":1250,"done":false},{"id":"b","icon":"drop","label":"Jelly","left":99,"done":false},{"id":"c","icon":"leaf","label":"Leaves","left":0,"done":true},{"id":"d","icon":"gem","label":"Gems","left":340,"done":false},{"id":"e","icon":"star","label":"Stars","left":7,"done":false},{"id":"f","icon":"key","label":"Keys","left":3,"done":false}]}'>

<!--
  The in-level HUD: what limits the level (moves or time left) in a cut
  well, amber with a tag in words when the game says it runs low; the goals
  as sockets with what is left of each, its name under it and a check once
  met; and the score over a leaning bar with the game's star marks, lit up
  to its progress. It draws what the game counts; it decides nothing. Where
  it is given less than a phone's width (its own, not the screen's) the
  moves and the score share the top line and the goals wrap under them.
-->
<section id="PuzzleGoals" part="PuzzleGoals.frame" class="hud" data-layout="shell" aria-label={t("puzzle.goals.name")} data-low={low}>
  <div class="counters" data-layout="cluster" data-gap="loose">
    <div part="PuzzleGoals.counter" data-layout="stack" data-gap="none" role={counter == 'time' ? 'timer' : 'status'} data-kind={counter} data-low={low} aria-label={counter == 'time' ? t("puzzle.goals.timeLeft", { time: formatCountdown(seconds) }) : t("puzzle.goals.movesLeft", { moves: moves })}>
      <span class="counter-figure" part="PuzzleGoals.figure" data-low={low}>{counter == 'time' ? formatCountdown(seconds) : formatNumber(moves)}</span>
      <span part="PuzzleGoals.label" class="label">{counter == 'time' ? t("puzzle.goals.time") : t("puzzle.goals.moves")}</span>
      {#if low}<span class="warning" part="PuzzleGoals.warning">{t("puzzle.goals.low")}</span>{/if}
    </div>
    <div part="PuzzleGoals.goals" data-layout="cluster" data-gap="tight" data-fill role="list" aria-label={t("puzzle.goals.name")}>
      {#each goals as g (g.id)}
        <div part="PuzzleGoals.goal" data-layout="stack" data-gap="none" role="listitem" data-done={g.done} aria-label={g.done ? t("puzzle.goals.done", { label: g.label }) : t("puzzle.goals.left", { label: g.label, left: g.left })}>
          <div part="PuzzleGoals.socket" class="socket" data-layout="cover"><og-icon class="goal-art" icon={g.icon}></og-icon></div>
          {#if g.done}<svg part="PuzzleGoals.done" class="check" viewBox="0 0 24 24"><path part="PuzzleGoals.glyph" d="M5.5 12.5l4.5 4.5 8.5-9.5"></path></svg>{/if}
          <span part="PuzzleGoals.count" class="count" data-done={g.done}>{formatNumber(g.left)}</span>
          <span part="PuzzleGoals.name" class="name">{g.label}</span>
        </div>
      {:else}
        <p class="none" part="PuzzleGoals.label">{t("puzzle.goals.none")}</p>
      {/each}
    </div>
    <div part="PuzzleGoals.score" data-layout="stack" data-gap="hair" role="status" aria-label={t("puzzle.goals.scoreName", { score: formatNumber(score) })}>
      <div class="score-line" data-layout="row" data-gap="tight">
        <span class="score-label" part="PuzzleGoals.label" data-fill>{t("puzzle.goals.score")}</span>
        <span class="score-figure" part="PuzzleGoals.figure">{formatNumber(score)}</span>
      </div>
      {#if progress >= 0}
      <div class="bar">
        <div part="PuzzleGoals.track" class="track"><div part="PuzzleGoals.fill" class="fill" style:--value={progress > 1 ? 1 : progress}></div></div>
        {#each stars as s (s.id)}<span part="PuzzleGoals.star" class="star" data-lit={progress >= s.at} style:--at={s.at}><og-icon class="star-icon" icon="star"></og-icon></span>{/each}
      </div>
      {/if}
    </div>
  </div>
</section>

<style>
/* Drawn over the scene: the style's tag halo holds its text there. */
:host { text-shadow: var(--tag-halo); }
:host { block-size: auto; }
.counters { align-items: stretch; padding: var(--space-xs); }
/* Moves or time: a big figure over its word in a cut well; low, amber. */
[part="PuzzleGoals.counter"] { position: relative; flex: none; align-items: center; justify-content: center; min-width: calc(var(--size-xl) * 1.6); padding: var(--space-xs) var(--space-sm); background: var(--play-well-fill); box-shadow: var(--play-well-rim); clip-path: polygon(0 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%); }
[part="PuzzleGoals.counter"][data-low=true] { box-shadow: var(--play-low-rim); }
.counter-figure { white-space: nowrap; font-size: var(--play-counter-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--ink-bright); }
.counter-figure[data-low=true] { color: var(--play-low-ink); }
.label, .score-label { white-space: nowrap; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); }
.warning { margin-block-start: var(--space-xxs); padding: 0 var(--space-xs); white-space: nowrap; font-size: var(--hud-tag-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--play-low-tag-ink); background: var(--play-low-ink); clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
/* A goal: its socket, what is left under it, its name; met, a check. */
[part="PuzzleGoals.goals"] { justify-content: center; align-items: stretch; }
[part="PuzzleGoals.goal"] { position: relative; align-items: center; justify-content: flex-start; min-width: calc(var(--size-xl) * 1.3); max-width: calc(var(--size-xl) * 2); padding: var(--space-xxs) var(--space-xs); background: var(--play-goal-fill); box-shadow: var(--play-goal-rim); }
.socket { inline-size: var(--size-lg); block-size: var(--size-lg); }
.goal-art { width: 100%; height: 100%; filter: var(--inv-slot-art-shadow); }
[data-done=true] .goal-art { filter: var(--play-met-art); }
.check { position: absolute; top: var(--space-xxs); inset-inline-end: var(--space-xxs); inline-size: var(--size-xs); block-size: var(--size-xs); border-radius: var(--tag-radius); background: var(--play-met-fill); box-shadow: var(--play-check-glow); }
[part="PuzzleGoals.glyph"] { fill: none; stroke: var(--play-check-ink); stroke-width: 3; stroke-linecap: round; stroke-linejoin: round; }
.count { white-space: nowrap; font-size: var(--play-goal-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--ink-bright); }
.count[data-done=true] { color: var(--play-met-ink); }
.name { max-inline-size: 100%; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; font-size: var(--hud-tag-size); font-weight: var(--hud-label-weight); color: var(--hud-label-ink); }
.none { margin: 0; align-self: center; text-align: center; }
/* The score over its bar, the star marks along it. */
[part="PuzzleGoals.score"] { flex: none; justify-content: center; min-width: calc(var(--size-xl) * 3.6); padding: var(--space-xs) var(--space-sm); }
.score-line { align-items: baseline; }
.score-figure { white-space: nowrap; font-size: var(--play-score-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--ink-bright); }
.bar { position: relative; inline-size: stretch; block-size: var(--size-xs); }
.track { position: absolute; inset-inline: 0; top: 30%; block-size: calc(8 * var(--font-step-px)); overflow: hidden; transform: skewX(var(--bar-skew)); background: var(--bar-well); box-shadow: var(--bar-rim); }
.fill { inline-size: calc(var(--value) * 100%); block-size: 100%; background: var(--play-star-fill); }
.star { position: absolute; top: 0; inset-inline-start: calc(var(--at) * 100% - var(--size-xs) / 2); inline-size: var(--size-xs); block-size: var(--size-xs); color: var(--play-star-off); }
.star[data-lit=true] { color: var(--play-star-on); filter: var(--hud-gold-glow); }
.star-icon { width: 100%; height: 100%; }
/* The frame is the container, so a narrow panel on a wide screen wraps as a phone does. */
@container (width < 600px) {
  .counters { --layout-gap-loose: var(--layout-gap-tight); }
  [part="PuzzleGoals.counter"] { min-width: calc(var(--size-xl) * 1.3); }
  .counter-figure { font-size: var(--play-counter-compact-size); }
  [part="PuzzleGoals.goals"] { order: 3; flex-basis: 100%; }
  [part="PuzzleGoals.score"] { flex: 1; min-width: 0; }
  [part="PuzzleGoals.goal"] { flex: 1 1 0; min-width: 0; }
  .socket { inline-size: var(--size-md); block-size: var(--size-md); }
}
</style>
