<meta name="component" content="PuzzleLevelPath">
<meta name="strings" content="./puzzle.en.ogt">
<meta name="parts" content="PuzzleLevelPath.frame PuzzleLevelPath.chapter PuzzleLevelPath.title PuzzleLevelPath.badge PuzzleLevelPath.lane PuzzleLevelPath.trail PuzzleLevelPath.node PuzzleLevelPath.number PuzzleLevelPath.stars PuzzleLevelPath.here PuzzleLevelPath.empty">
<meta name="prop:chapters" content="list" data-doc="The chapters in path order, keyed rows of { id, label, earned, total }: earned and total the chapter's stars as the game counts them.">
<meta name="prop:levels" content="list" data-doc="The levels in path order, keyed rows of { id, chapter, number, state, stars, maxStars, x }: chapter a chapter's id; state locked, open, current or done; stars earned (0 to 3) of maxStars (1 to 3); x the node's place across the path, 0 (start) to 1 (end), as the game lays the path out.">
<meta name="prop:selected" content="string" data-doc="The level the player has picked (its details shown elsewhere); empty is none.">
<meta name="action:details" content="Play">
<meta name="fixture:default" content='{"selected":"","chapters":[{"id":"c1","label":"Candy Meadow","earned":7,"total":12}],"levels":[{"id":"l1","chapter":"c1","number":1,"state":"done","stars":3,"maxStars":3,"x":0.5},{"id":"l2","chapter":"c1","number":2,"state":"done","stars":2,"maxStars":3,"x":0.75},{"id":"l3","chapter":"c1","number":3,"state":"done","stars":2,"maxStars":3,"x":0.5},{"id":"l4","chapter":"c1","number":4,"state":"current","stars":0,"maxStars":3,"x":0.25},{"id":"l5","chapter":"c1","number":5,"state":"locked","stars":0,"maxStars":3,"x":0.5}]}'>
<meta name="fixture:empty" content='{"selected":"","chapters":[],"levels":[]}'>
<meta name="fixture:selected" content='{"selected":"l2","chapters":[{"id":"c1","label":"Candy Meadow","earned":7,"total":12}],"levels":[{"id":"l1","chapter":"c1","number":1,"state":"done","stars":3,"maxStars":3,"x":0.5},{"id":"l2","chapter":"c1","number":2,"state":"done","stars":2,"maxStars":3,"x":0.75},{"id":"l3","chapter":"c1","number":3,"state":"open","stars":0,"maxStars":3,"x":0.5},{"id":"l4","chapter":"c1","number":4,"state":"current","stars":0,"maxStars":3,"x":0.25}]}'>
<meta name="fixture:locked" content='{"selected":"","chapters":[{"id":"c9","label":"Frosted Peaks","earned":0,"total":9}],"levels":[{"id":"l41","chapter":"c9","number":41,"state":"locked","stars":0,"maxStars":3,"x":0.5},{"id":"l42","chapter":"c9","number":42,"state":"locked","stars":0,"maxStars":3,"x":0.25},{"id":"l43","chapter":"c9","number":43,"state":"locked","stars":0,"maxStars":3,"x":0.5}]}'>
<meta name="fixture:overflow" content='{"selected":"l12","chapters":[{"id":"c1","label":"Candy Meadow","earned":12,"total":12},{"id":"c2","label":"The Very Long Caramel Causeway Beyond the Whispering Lemonade Falls","earned":4,"total":18}],"levels":[{"id":"l9","chapter":"c1","number":9,"state":"done","stars":3,"maxStars":3,"x":0.25},{"id":"l10","chapter":"c1","number":10,"state":"done","stars":3,"maxStars":3,"x":0.5},{"id":"l11","chapter":"c1","number":11,"state":"done","stars":3,"maxStars":3,"x":0.75},{"id":"l12","chapter":"c1","number":12,"state":"done","stars":3,"maxStars":3,"x":0.5},{"id":"l13","chapter":"c2","number":13,"state":"done","stars":1,"maxStars":3,"x":0.25},{"id":"l14","chapter":"c2","number":14,"state":"done","stars":3,"maxStars":3,"x":0.5},{"id":"l15","chapter":"c2","number":15,"state":"open","stars":0,"maxStars":3,"x":0.75},{"id":"l16","chapter":"c2","number":1016,"state":"current","stars":0,"maxStars":3,"x":0.5},{"id":"l17","chapter":"c2","number":1017,"state":"locked","stars":0,"maxStars":3,"x":0.25},{"id":"l18","chapter":"c2","number":1018,"state":"locked","stars":0,"maxStars":3,"x":0.5}]}'>

<!--
  A scrolling level path: chapters in order, each a header with its name
  and its stars, then its levels as round nodes stepping down the path at
  the place across it the game gives (x). A node shows its number, a lock
  when it is locked, its stars once done, and a Play marker on the
  current one; each state has a mark as well as a dress. Reports choose (a
  level's key; a locked level is disabled and sends nothing) and, with X
  or pad Y on the focused node, details. The game decides what is open
  and counts the stars.
-->
<section id="PuzzleLevelPath" part="PuzzleLevelPath.frame" aria-label={t("puzzle.path.name")}>
  {#if levels.length}
    <scroll class="path">
      {#each chapters as c (c.id)}
        <section class="chapter" aria-label={c.label}>
          <header part="PuzzleLevelPath.chapter">
            <h3 part="PuzzleLevelPath.title">{c.label}</h3>
            <div part="PuzzleLevelPath.badge" aria-label={t("puzzle.path.chapterStarsName", { earned: c.earned, total: c.total })}>
              <svg class="badge-star" viewBox="0 0 24 24"><path d="M12 2.5l2.9 6.1 6.6.8-4.9 4.6 1.3 6.6L12 17.3l-5.9 3.3 1.3-6.6-4.9-4.6 6.6-.8z"></path></svg>
              <span class="badge-figure">{t("puzzle.path.chapterStars", { earned: c.earned, total: c.total })}</span>
            </div>
          </header>
          <div class="lanes" role="listbox" aria-label={c.label}>
            {#each levels as l (l.id)}
              {#if l.chapter == c.id}
                <div part="PuzzleLevelPath.lane" data-state={l.state} style:--x={l.x}>
                  <div class="lead"></div>
                  <div class="stop">
                    <div part="PuzzleLevelPath.trail" data-state={l.state}></div>
                    <button part="PuzzleLevelPath.node" role="option" data-state={l.state} aria-selected={l.id == selected} disabled={l.state == 'locked'} aria-label={l.state == 'locked' ? t("puzzle.path.levelLocked", { number: l.number }) : l.state == 'done' ? t("puzzle.path.levelDone", { number: l.number, stars: l.stars, max: l.maxStars }) : l.state == 'current' ? t("puzzle.path.levelCurrent", { number: l.number }) : t("puzzle.path.levelOpen", { number: l.number })} onclick={choose} ondetails={details}><div class="face">
                      <span part="PuzzleLevelPath.number" class="number">{formatNumber(l.number)}</span>
                      {#if l.state == 'locked'}<og-icon class="lock" icon="lock"></og-icon>{/if}
                    </div></button>
                    <div part="PuzzleLevelPath.stars" data-state={l.state} hidden={l.state != 'done'}>
                      <svg class="star" class:lit={l.stars >= 1} viewBox="0 0 24 24"><path d="M12 2.5l2.9 6.1 6.6.8-4.9 4.6 1.3 6.6L12 17.3l-5.9 3.3 1.3-6.6-4.9-4.6 6.6-.8z"></path></svg>
                      <svg class="star" class:lit={l.stars >= 2} hidden={l.maxStars < 2} viewBox="0 0 24 24"><path d="M12 2.5l2.9 6.1 6.6.8-4.9 4.6 1.3 6.6L12 17.3l-5.9 3.3 1.3-6.6-4.9-4.6 6.6-.8z"></path></svg>
                      <svg class="star" class:lit={l.stars >= 3} hidden={l.maxStars < 3} viewBox="0 0 24 24"><path d="M12 2.5l2.9 6.1 6.6.8-4.9 4.6 1.3 6.6L12 17.3l-5.9 3.3 1.3-6.6-4.9-4.6 6.6-.8z"></path></svg>
                    </div>
                    <span part="PuzzleLevelPath.here" hidden={l.state != 'current'}>{t("puzzle.path.play")}</span>
                  </div>
                  <div class="tail"></div>
                </div>
              {/if}
            {/each}
          </div>
        </section>
      {/each}
    </scroll>
  {:else}
    <div part="PuzzleLevelPath.empty" role="status"><p class="empty-text">{t("puzzle.path.empty")}</p></div>
  {/if}
</section>

<style>
[part="PuzzleLevelPath.frame"] [hidden] { display: none; }
[part="PuzzleLevelPath.frame"] { display: flex; flex-direction: column; width: stretch; height: stretch; min-height: 0; background: var(--panel); container-type: inline-size; }
.path { width: stretch; flex: 1; min-height: 0; gap: var(--space-lg); padding: var(--space-sm) var(--space-md) var(--space-xl); }
.chapter { display: flex; flex-direction: column; width: stretch; gap: var(--space-xs); }
[part="PuzzleLevelPath.chapter"] { display: flex; width: stretch; align-items: center; gap: var(--space-sm); padding: var(--space-xs) var(--space-md); min-height: var(--size-touch); }
[part="PuzzleLevelPath.title"] { margin: 0; flex: 1; min-width: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; text-align: start; }
[part="PuzzleLevelPath.badge"] { display: flex; flex: none; align-items: center; gap: var(--space-xxs); padding: var(--space-xxs) var(--space-sm); white-space: nowrap; }
.badge-star { width: var(--size-xs); height: var(--size-xs); flex: none; }
.lanes { display: flex; flex-direction: column; width: stretch; }
[part="PuzzleLevelPath.lane"] { display: flex; width: stretch; align-items: flex-start; }
.lead { flex-grow: var(--x); flex-shrink: 1; flex-basis: 0; }
.tail { flex-grow: calc(1 - var(--x)); flex-shrink: 1; flex-basis: 0; }
.stop { display: flex; flex: none; flex-direction: column; align-items: center; width: calc(var(--size-xl) * 1.5); }
[part="PuzzleLevelPath.trail"] { width: var(--stroke-thick); height: var(--space-lg); }
[part="PuzzleLevelPath.lane"]:first-child [part="PuzzleLevelPath.trail"] { visibility: hidden; }
[part="PuzzleLevelPath.node"] { width: var(--size-xl); height: var(--size-xl); min-height: var(--size-touch); padding: 0; border-radius: var(--radius-pill); }
[part="PuzzleLevelPath.node"] .face { position: relative; display: flex; width: stretch; height: stretch; align-items: center; justify-content: center; }
[part="PuzzleLevelPath.number"] { white-space: nowrap; }
[part="PuzzleLevelPath.node"][data-state=locked] [part="PuzzleLevelPath.number"] { display: none; }
.lock { width: var(--size-sm); height: var(--size-sm); }
[part="PuzzleLevelPath.stars"] { display: flex; align-items: center; justify-content: center; gap: var(--space-xxs); margin-top: calc(var(--space-sm) * -1); }
.star { width: var(--size-xs); height: var(--size-xs); }
[part="PuzzleLevelPath.here"] { margin-top: var(--space-xxs); padding: var(--space-xxs) var(--space-sm); white-space: nowrap; }
[part="PuzzleLevelPath.empty"] { display: flex; width: stretch; flex: 1; align-items: center; justify-content: center; padding: var(--space-xl); }
.empty-text { margin: 0; text-align: center; }
@container (width < 600px) { .stop { width: calc(var(--size-xl) * 1.25); } [part="PuzzleLevelPath.node"] { width: var(--size-lg); height: var(--size-lg); } }
</style>
