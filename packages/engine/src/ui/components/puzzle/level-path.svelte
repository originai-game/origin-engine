<meta name="component" content="PuzzleLevelPath">
<meta name="strings" content="./puzzle.en.ogt">
<meta name="parts" content="PuzzleLevelPath.frame PuzzleLevelPath.chapter PuzzleLevelPath.title PuzzleLevelPath.badge PuzzleLevelPath.meter PuzzleLevelPath.lane PuzzleLevelPath.trail PuzzleLevelPath.node PuzzleLevelPath.number PuzzleLevelPath.stars PuzzleLevelPath.here PuzzleLevelPath.empty PuzzleLevelPath.figure PuzzleLevelPath.caption">
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
  A scrolling level path over the game's own map art, which the component
  leaves at full brightness: chapters in order, each a glass header with
  its name, its stars and a gold line of them, then its levels as discs on
  a road, each at the place across the path the game gives (x). The road
  is lit gold up to the level to play and pale past it. A disc shows its
  number, a lock when it is locked, its stars on a dark capsule at its
  foot once done, and the Play plate under the current one, the road leaving
  only below it after a gap; the chosen
  one is ringed and says Selected, an open one says Open. Reports choose
  (a level's key; a locked level is disabled and sends nothing) and, with
  X or pad Y on the focused disc, details. The game decides what is open
  and counts the stars. The path opens scrolled to the level to play.
-->
<section id="PuzzleLevelPath" part="PuzzleLevelPath.frame" class="hud bare" data-layout="shell" aria-label={t("puzzle.path.name")}>
  <!-- The scroller is built once its lanes can be: a level to play that
       arrives in a scroller already shown no longer moves it. -->
  {#if levels.length && chapters.length}
    <div class="path" data-layout="scroll" data-fill>
      {#each chapters as c (c.id)}
        <section class="chapter" data-layout="stack" data-gap="none" aria-label={c.label}>
          <header part="PuzzleLevelPath.chapter" class="head" data-layout="stack" data-gap="tight">
            <div class="head-row" data-layout="row" data-gap="tight">
              <h3 part="PuzzleLevelPath.title" class="title" data-fill>{c.label}</h3>
              <div part="PuzzleLevelPath.badge" class="total" data-layout="row" data-gap="hair" aria-label={t("puzzle.path.chapterStarsName", { earned: c.earned, total: c.total })}>
                <svg class="total-star" part="PuzzleLevelPath.figure" viewBox="0 0 24 24"><path d="M12 2.5l2.9 6.1 6.6.8-4.9 4.6 1.3 6.6L12 17.3l-5.9 3.3 1.3-6.6-4.9-4.6 6.6-.8z"></path></svg>
                <span class="total-figure">{t("puzzle.path.chapterStars", { earned: c.earned, total: c.total })}</span>
              </div>
            </div>
            <div part="PuzzleLevelPath.meter" class="meter" data-layout="overlay" style:--got={c.total > 0 ? (c.earned >= c.total ? 1 : c.earned / c.total) : 0}><div class="meter-fill" data-anchor="top-start"></div></div>
          </header>
          <div class="lanes" data-layout="stack" data-gap="none" role="listbox" aria-label={c.label}>
            {#each levels as l, i (l.id)}
              {#if l.chapter == c.id}
                <div part="PuzzleLevelPath.lane" class="lane" data-layout="row" data-gap="none" data-state={l.state} data-selected={l.id == selected} style:--x={l.x} style:--px={i > 0 ? levels[i - 1]['x'] : l.x} style:--lo={i > 0 ? (levels[i - 1]['x'] < l.x ? levels[i - 1]['x'] : l.x) : l.x} style:--span={i > 0 ? (levels[i - 1]['x'] < l.x ? l.x - levels[i - 1]['x'] : levels[i - 1]['x'] - l.x) : 0}>
                  {#if i > 0 && levels[i - 1]['chapter'] == l.chapter}
                    <div part="PuzzleLevelPath.trail" class="road across" data-lit={l.state == 'done' || l.state == 'current'}></div>
                    <div part="PuzzleLevelPath.trail" class="road down-in" data-lit={l.state == 'done' || l.state == 'current'}></div>
                  {/if}
                  {#if i + 1 < levels.length && levels[i + 1]['chapter'] == l.chapter}
                    <div part="PuzzleLevelPath.trail" class="road down-out" data-lit={levels[i + 1]['state'] == 'done' || levels[i + 1]['state'] == 'current'}></div>
                  {/if}
                  <div class="lead"></div>
                  <div class="stop" data-layout="center">
                    <button part="PuzzleLevelPath.node" class="node" role="option" data-state={l.state} aria-selected={l.id == selected} disabled={l.state == 'locked'} aria-label={l.state == 'locked' ? t("puzzle.path.levelLocked", { number: l.number }) : l.state == 'done' ? t("puzzle.path.levelDone", { number: l.number, stars: l.stars, max: l.maxStars }) : l.state == 'current' ? t("puzzle.path.levelCurrent", { number: l.number }) : t("puzzle.path.levelOpen", { number: l.number })} onclick={choose} ondetails={details}><div class="face" data-layout="center">
                      {#if l.state == 'locked'}<og-icon class="lock" icon="lock"></og-icon>{:else}<span part="PuzzleLevelPath.number" class="number" data-length={l.number >= 10000 ? 'longer' : l.number >= 1000 ? 'long' : null}>{formatNumber(l.number)}</span>{/if}
                    </div></button>
                    {#if l.state == 'done'}<div part="PuzzleLevelPath.stars" class="stars" data-layout="row" data-gap="none" data-state={l.state}>
                      <svg class="star" data-lit={l.stars >= 1} viewBox="0 0 24 24"><path d="M12 2.5l2.9 6.1 6.6.8-4.9 4.6 1.3 6.6L12 17.3l-5.9 3.3 1.3-6.6-4.9-4.6 6.6-.8z"></path></svg>
                      {#if l.maxStars >= 2}<svg class="star big" data-lit={l.stars >= 2} viewBox="0 0 24 24"><path d="M12 2.5l2.9 6.1 6.6.8-4.9 4.6 1.3 6.6L12 17.3l-5.9 3.3 1.3-6.6-4.9-4.6 6.6-.8z"></path></svg>{/if}
                      {#if l.maxStars >= 3}<svg class="star" data-lit={l.stars >= 3} viewBox="0 0 24 24"><path d="M12 2.5l2.9 6.1 6.6.8-4.9 4.6 1.3 6.6L12 17.3l-5.9 3.3 1.3-6.6-4.9-4.6 6.6-.8z"></path></svg>{/if}
                    </div>{/if}
                    {#if l.state == 'current'}<span class="here" part="PuzzleLevelPath.here">{t("puzzle.path.play")}</span>{:else}{#if l.id == selected}<span class="word" part="PuzzleLevelPath.caption">{t("puzzle.path.selected")}</span>{:else}{#if l.state == 'open'}<span class="word" part="PuzzleLevelPath.caption">{t("puzzle.path.open")}</span>{/if}{/if}{/if}
                  </div>
                  <div class="tail"></div>
                </div>
              {/if}
            {/each}
          </div>
        </section>
      {/each}
    </div>
  {:else}
    <div part="PuzzleLevelPath.empty" data-layout="cover" data-fill role="status"><p class="empty-text" part="PuzzleLevelPath.caption">{t("puzzle.path.empty")}</p></div>
  {/if}
</section>

<style>
:host { inline-size: stretch; min-inline-size: 0; block-size: stretch; min-block-size: 0; color: var(--hud-ink); --node: var(--select-node-size); --node-on: var(--select-node-current-size); --road: var(--select-road-width); --stop: calc(var(--node-on) * 1.5); --pad-top: var(--space-xs); --pad-end: calc(var(--hud-size-xs) * 1.8 + var(--space-sm)); --mid: calc(var(--pad-top) + var(--node-on) / 2); }
.path { overflow-y: auto; padding: var(--space-sm) var(--space-sm) var(--space-xl); }
.chapter { flex: none; margin-block-end: var(--space-lg); }
/* The chapter's header: the only glass the path draws. */
.head { flex: none; width: stretch; padding: var(--space-sm) var(--space-md) var(--space-md); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); clip-path: polygon(var(--select-row-cut) 0, 100% 0, 100% calc(100% - var(--select-row-cut)), calc(100% - var(--select-row-cut)) 100%, 0 100%, 0 var(--select-row-cut)); }
.head-row { width: stretch; align-items: center; }
.title { margin: 0; min-inline-size: 0; text-align: start; font-size: var(--select-size-name-on); line-height: 1.15; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); color: var(--hud-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.total { flex: none; align-items: center; padding: calc(2 * var(--font-step-px)) var(--space-sm); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); white-space: nowrap; }
.total-star { inline-size: var(--select-glyph-lg); block-size: var(--select-glyph-lg); flex: none; fill: var(--select-star-on); filter: var(--star-glow); }
.total-figure { font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); }
.meter { width: stretch; block-size: calc(5 * var(--font-step-px)); background: var(--select-meter-well); }
.meter-fill { inline-size: calc(var(--got) * 100%); block-size: 100%; background: var(--select-meter-fill); box-shadow: var(--select-meter-glow); }
/* A level's lane: its disc at the place across the path the game gives,
   centred in the lane, so the road meets it at the lane's middle. */
/* Under each disc the lane keeps room for its words, so they never lie
   on the road that crosses to the next level. */
.lane { position: relative; width: stretch; align-items: center; padding-block: var(--pad-top) var(--pad-end); }
.lane[data-state=current] { scroll-initial-target: nearest; scroll-margin-block-start: calc(var(--node) * 2.5); }
.lead { flex-grow: var(--x); flex-shrink: 1; flex-basis: 0; }
.tail { flex-grow: calc(1 - var(--x)); flex-shrink: 1; flex-basis: 0; }
.stop { position: relative; flex: none; inline-size: var(--stop); block-size: var(--node-on); }
/* The road: round-capped runs from the last disc down to the lane's top,
   across it and down to this disc; lit gold where the player has been. */
.road { position: absolute; border-radius: var(--select-road-radius); background: var(--select-road-ahead); box-shadow: var(--select-road-shadow); }
.road[data-lit=true] { background: var(--select-road-lit); }
.across { inset-block-start: calc(var(--road) / -2); block-size: var(--road); inset-inline-start: calc(var(--lo) * (100% - var(--stop)) + var(--stop) / 2 - var(--road) / 2); inline-size: calc(var(--span) * (100% - var(--stop)) + var(--road)); }
.down-in { inset-block-start: calc(var(--road) / -2); block-size: calc(var(--mid) + var(--road) / 2); inset-inline-start: calc(var(--x) * (100% - var(--stop)) + var(--stop) / 2 - var(--road) / 2); inline-size: var(--road); }
.down-out { inset-block-start: var(--mid); block-size: calc(100% - var(--mid)); inset-inline-start: calc(var(--x) * (100% - var(--stop)) + var(--stop) / 2 - var(--road) / 2); inline-size: var(--road); }
/* Under a disc with words (Play, Open, Selected) the road leaves only below
   them, after a gap, so it never touches the plate or the tag. */
.lane:is([data-state=current], [data-state=open], [data-selected=true]) { padding-block-end: calc(var(--pad-end) + var(--space-lg)); }
.lane:is([data-state=current], [data-state=open], [data-selected=true]) .down-out { inset-block-start: auto; inset-block-end: 0; block-size: calc(var(--space-lg) - var(--space-xs)); }
/* A disc: slate when done, the accent with rings when it is the one to
   play, rimmed in the accent when open, dark with a lock when locked. */
.node { inline-size: var(--node); block-size: var(--node); min-block-size: var(--size-touch); min-inline-size: var(--size-touch); padding: 0; border: 0; border-radius: var(--select-node-radius); color: var(--hud-ink); background: var(--select-node-fill); box-shadow: var(--select-node-rim); transition: var(--change-motion); }
.node:hover { box-shadow: var(--select-node-hover-rim); }
.node[data-state=open] { background: var(--select-node-open-fill); box-shadow: var(--select-node-open-rim); }
.node[data-state=current] { inline-size: var(--node-on); block-size: var(--node-on); color: var(--select-accent-ink); background: var(--select-node-current-fill); box-shadow: var(--select-node-current-rim); }
.node[data-state=locked] { color: var(--select-node-locked-ink); background: var(--select-node-locked-fill); box-shadow: var(--select-node-locked-rim); }
.node[aria-selected=true] { box-shadow: var(--select-node-selected-rim); }
.node:focus-visible { box-shadow: var(--select-node-focus-rim); }
/* The number sits in the disc's middle both ways. */
.face { inline-size: 100%; block-size: 100%; justify-content: safe center; container-type: inline-size; }
.number { font-size: var(--select-size-node); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-shadow: var(--hud-shadow); }
.node[data-state=current] .number { font-size: var(--select-size-node-current); text-shadow: none; }
/* A long number takes a share of its disc (the face is its container),
   at any state and text size. */
.number[data-length=long], .node[data-state=current] .number[data-length=long] { font-size: var(--select-size-node-long); }
.number[data-length=longer], .node[data-state=current] .number[data-length=longer] { font-size: var(--select-size-node-longer); }
.lock { inline-size: 38%; block-size: 38%; }
/* A done level's stars, on a dark capsule across its disc's foot. */
.stars { position: absolute; inset-block-start: calc(50% + var(--node) / 2 - var(--space-md)); inset-inline-start: 50%; translate: -50% 0; align-items: flex-end; padding: calc(2 * var(--font-step-px)) var(--space-xs); border-radius: var(--select-node-radius); background: var(--select-capsule-fill); box-shadow: var(--select-capsule-rim); }
.star { inline-size: calc(var(--node) * 0.26); block-size: calc(var(--node) * 0.26); fill: var(--select-star-off); }
.star.big { inline-size: calc(var(--node) * 0.32); block-size: calc(var(--node) * 0.32); }
.star[data-lit=true] { fill: var(--select-star-on); filter: var(--star-glow); }
/* Under the disc: Play on the accent plate, or the state in words on a
   dark tag. */
.here, .word { position: absolute; inset-block-start: calc(50% + var(--node-on) / 2 + var(--space-xs)); inset-inline-start: 50%; translate: -50% 0; white-space: nowrap; font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); }
.lane:has(.stars) .word { inset-block-start: calc(50% + var(--node) / 2 + var(--space-md)); }
.here { padding: calc(3 * var(--font-step-px)) var(--space-md); font-size: var(--hud-size-xs); color: var(--select-accent-ink); background: var(--select-button-on-fill); box-shadow: var(--select-button-on-rim); clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
.word { inset-block-start: calc(50% + var(--node) / 2 + var(--space-xs)); padding: 0 var(--space-xs); font-size: var(--hud-tag-size); line-height: 1.6; color: var(--hud-ink); background: var(--hud-tag-fill); }
[part="PuzzleLevelPath.empty"] { padding: var(--space-xl); }
.empty-text { margin: 0; padding: var(--space-sm) var(--space-md); text-align: center; font-size: var(--hud-size-sm); color: var(--hud-ink); background: var(--hud-tag-fill); }
@container (width < 600px) { :host { --node: calc(var(--select-node-size) * 0.8); --node-on: calc(var(--select-node-current-size) * 0.8); --road: calc(var(--select-road-width) * 0.8); } .title { font-size: var(--select-size-name); } }
</style>
