<meta name="screen" content="results" data-layer="menu" data-focus="Panel/Footer/Continue" data-under="hide" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="none">
<meta name="strings" content="./results.en.ogt">
<meta name="parts" content="results.frame results.title results.grade results.row results.label results.figure results.cell">
<meta name="prop:won" content="bool" data-doc="The player won.">
<meta name="prop:grade" content="string" data-doc="The grade (S, A, B …; empty: none).">
<meta name="prop:title" content="string" data-doc="The result's line (Victory, Stage clear).">
<meta name="prop:stats" content="list" data-doc="The stats: { name, value, best (bool), format (number or percent; defaults to number), count (bool), decimals }. A stat with count counts up from 0 as the screen opens, written whole or with decimals digits after the point; the others show their value as it is, fractions kept.">
<meta name="prop:level" content="number" data-doc="The level after the match.">
<meta name="prop:xp" content="number" data-doc="Progress to the next level after the match, 0 to 1.">
<meta name="prop:gained" content="number" data-doc="XP gained.">
<meta name="prop:rewards" content="list" data-doc="The rewards: { name, amount, texture }.">
<meta name="prop:tab" content="string" data-doc="summary, graphs or board.">
<meta name="action:confirm" content="Continue">
<meta name="fixture:default" content="{&quot;gained&quot;:1240,&quot;grade&quot;:&quot;A&quot;,&quot;level&quot;:18,&quot;rewards&quot;:[{&quot;amount&quot;:320,&quot;id&quot;:&quot;item-0&quot;,&quot;name&quot;:&quot;Gold&quot;,&quot;texture&quot;:0},{&quot;amount&quot;:1,&quot;id&quot;:&quot;item-1&quot;,&quot;name&quot;:&quot;Crate&quot;,&quot;texture&quot;:0}],&quot;stats&quot;:[{&quot;best&quot;:true,&quot;count&quot;:true,&quot;id&quot;:&quot;item-0&quot;,&quot;name&quot;:&quot;Score&quot;,&quot;value&quot;:48210},{&quot;best&quot;:false,&quot;id&quot;:&quot;item-1&quot;,&quot;name&quot;:&quot;Kills&quot;,&quot;value&quot;:14},{&quot;best&quot;:false,&quot;format&quot;:&quot;percent&quot;,&quot;id&quot;:&quot;item-2&quot;,&quot;name&quot;:&quot;Accuracy&quot;,&quot;value&quot;:0.62}],&quot;tab&quot;:&quot;summary&quot;,&quot;title&quot;:&quot;Victory&quot;,&quot;won&quot;:true,&quot;xp&quot;:0.45}">

<!--
  After a match: the verdict set large in a lit fill (gold for a win, red
  for a loss) over the dimmed scene, a rule with a lit diamond, the level
  in an octagon beside the XP gained and the leaning progress bar, the
  grade on a hexagon, then the tabs. Summary shows each stat on a cut
  plate (a best in gold) and each reward as octagon art with its amount
  on a tag and its name under it. The page scrolls over the actions.
-->
<div id="Results">
  <section id="Panel" part="results.frame" class="bare">
    <div id="Page">
      <header id="Head">
        <div id="Text">
          <h1 id="Title" part="results.title" class:outcome-lost={!won} class:outcome-won={won}>{title}</h1>
          <div id="Rule"><span id="Diamond"></span></div>
          <div id="Level">
            <span id="LevelBadge">{level}</span>
            <div id="Progress">
              <div id="LevelLine">
                <p id="LevelValue">{t("screen.results.LevelValue", { level: level })}</p>
                <p id="Gained">{t("screen.results.Gained", { gained: gained })}</p>
              </div>
              <progress id="Xp" bind:value={xp}></progress>
            </div>
          </div>
        </div>
        {#if grade != ""}<div id="GradeCell"><div id="GradeEdge"><p id="Grade" part="results.grade" role="heading">{grade}</p></div><p id="GradeCaption">{t("screen.results.grade")}</p></div>{/if}
      </header>
      <div id="Celebration">{@render celebration?.()}</div>
      <tabs id="Tabs" items="Summary Graphs Scoreboard" keys="screen.results.summary screen.results.graphs screen.results.board" values="summary graphs board" bind:current={tab} fit="true"></tabs>
      {#if tab == "summary"}
        <div id="Summary">
          <div id="Stats">
            {#each stats as s (s.id)}
              <div id="Stat" part="results.row" data-best={s.best}>
                <div id="StatHead">
                  <p id="Name" part="results.label">{s.name}</p>
                  {#if s.best}<span id="Best">{t("screen.results.best")}</span>{/if}
                </div>
                {#if s.count}
                  <og-counter id="StatCount" part="results.figure" value={s.value} from="0" format={s.format == 'percent' ? 'percent' : 'grouped'} decimals={s.decimals}></og-counter>
                {:else}
                  <p id="StatValue" part="results.figure">{t("screen.results.StatValue", { format: s.format, value: s.format == 'percent' ? s.value * 100 : s.value })}</p>
                {/if}
              </div>
            {/each}
          </div>
          {#if rewards.length}
            <div id="RewardsColumn">
              <p id="RewardsHeading">{t("screen.results.rewards")}</p>
              <div id="Rewards">
                {#each rewards as r (r.id)}
                  <div id="Reward" part="results.cell">
                    <div id="Socket" data-layout="overlay">
                      <art id="Art" kind="itemIcon" src={r.texture} data-anchor="fill"></art>
                    </div>
                    <p id="Amount">{t("screen.results.Amount", { r_amount: formatCompact(r.amount) })}</p>
                    <p id="RewardName">{r.name}</p>
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        </div>
      {:else if tab == "graphs"}
        <div id="Graph">{@render graph?.()}</div>
      {:else if tab == "board"}
        <div id="Scores">{@render scores?.()}</div>
      {/if}
    </div>
    <footer id="Footer">
      <button id="Replay" variant="quiet" onclick={replay}>{t("screen.results.replay")}</button>
      <button id="Again" onclick={again}>{t("screen.results.again")}</button>
      <button id="Continue" class="go" variant="primary" onclick={continue} onconfirm={continue}>{t("screen.results.continue")}</button>
    </footer>
  </section>
</div>

<style>
/* The scene dimmed under the page; the page centred in the height left,
   so a short summary sits in the middle of the screen, not at its top. */
:root { display: flex; block-size: stretch; padding: var(--space-xxl) calc(var(--space-xxl) * 2); background: var(--result-scrim); }
h1, p { margin: 0; }
[part="results.frame"] { display: flex; flex-direction: column; gap: var(--space-lg); inline-size: 100%; max-inline-size: calc(var(--measure-3xl) * 1.6); margin-inline: auto; block-size: 100%; }
#Page { flex: 1; min-block-size: 0; display: flex; flex-direction: column; justify-content: safe center; gap: var(--space-xl); overflow-y: auto; -og-scroll-fade: var(--space-xxl); }
#Head { display: flex; align-items: flex-end; gap: var(--space-xl); }
#Text { flex: 1; min-inline-size: 0; display: flex; flex-direction: column; gap: var(--space-md); }
/* The verdict: condensed capitals in the result's lit fill. The verdict,
   the grade and a stat's plate follow the player's text scale up to a
   share of the screen's height, so at large text the head gives the
   stats under it room and every figure shows before a scroll. */
[part="results.title"] { font-size: min(var(--result-size-hero), var(--party-hold-verdict)); line-height: 0.9; font-weight: var(--font-weight-bold); font-stretch: var(--hud-figure-stretch); text-transform: uppercase; letter-spacing: var(--result-verdict-tracking); overflow-wrap: anywhere; color: transparent; background: var(--result-won-fill); background-clip: var(--hud-text-clip); filter: var(--result-won-glow); }
.outcome-lost { background: var(--result-lost-fill); background-clip: var(--hud-text-clip); filter: var(--result-lost-glow); }
#Rule { position: relative; inline-size: min(100%, calc(560 * var(--font-step-px))); block-size: var(--stroke-thin); background: var(--result-rule); }
#Diamond { position: absolute; inset-inline-start: 50%; inset-block-start: 50%; inline-size: calc(9 * var(--font-step-px)); block-size: calc(9 * var(--font-step-px)); translate: -50% -50%; rotate: 45deg; background: var(--hud-ink); box-shadow: var(--result-diamond-glow); }
#Level { display: flex; align-items: center; gap: var(--space-md); max-inline-size: calc(640 * var(--font-step-px)); }
#LevelBadge { flex: none; display: block; text-align: center; inline-size: calc(54 * var(--font-step-px)); block-size: calc(54 * var(--font-step-px)); line-height: calc(54 * var(--font-step-px)); clip-path: var(--hud-octagon); background: var(--result-level-fill); color: var(--hud-ink); font-size: var(--hud-size-md); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); }
#Progress { flex: 1; min-inline-size: 0; display: flex; flex-direction: column; gap: var(--space-xs); }
#LevelLine { display: flex; flex-wrap: wrap; align-items: baseline; justify-content: space-between; column-gap: var(--space-md); }
#LevelValue { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); }
#Gained { font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-epic-ink); }
/* The XP bar leans as the HUD's bars do. */
#Xp { inline-size: stretch; block-size: calc(10 * var(--font-step-px)); transform: skewX(var(--bar-skew)); --meter-radius: 0px; --meter-track-fill: var(--bar-well); --meter-track-shadow: var(--bar-rim); --meter-shadow: none; --progress-fill: var(--hud-xp-fill); --meter-fill-shadow: var(--hud-xp-glow); }
/* The grade on a hexagon with a lit edge. */
#GradeCell { flex: none; display: flex; flex-direction: column; align-items: center; gap: var(--space-xs); }
#GradeEdge { padding: var(--stroke-thick); clip-path: var(--hud-hexagon); background: var(--hud-edge); filter: var(--result-grade-glow); }
[part="results.grade"] { --grade: min(var(--result-size-grade), var(--party-hold-grade)); display: block; text-align: center; line-height: calc(var(--grade) * 1.72); inline-size: calc(var(--grade) * 1.5); block-size: calc(var(--grade) * 1.72); clip-path: var(--hud-hexagon); background: var(--result-grade-face); font-size: var(--grade); line-height: 1; font-weight: var(--font-weight-bold); font-stretch: var(--hud-figure-stretch); color: var(--hud-ink); }
#GradeCaption { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); }
/* Tabs: a rule under them and the chosen tab underlined in the accent. */
#Tabs { inline-size: stretch; --tabs-track-fill: none; --tabs-track-shadow: none; --tabs-track-rule: var(--stroke-hairline) solid var(--result-tab-rule); --tabs-tab-fill: none; --tabs-tab-hover-fill: var(--result-tab-hover); --tabs-tab-selected-fill: var(--result-tab-selected); --tabs-tab-selected-shadow: var(--result-tab-mark); --tabs-tab-ink: var(--hud-ink-dim); --tabs-tab-selected-ink: var(--hud-ink); --tabs-transform: uppercase; --tabs-tracking: var(--result-tab-tracking); --tabs-radius: 0px; }
#Summary { display: grid; grid-template-columns: minmax(0, 1.7fr) minmax(0, 1fr); gap: var(--space-xxl); align-items: start; }
#Stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(calc(170 * var(--font-step-px)), 1fr)); gap: var(--space-md); }
/* A stat: a cut plate, its name small above its figure; a best in gold. */
[part="results.row"] { display: flex; flex-direction: column; justify-content: space-between; gap: var(--space-sm); min-inline-size: 0; min-block-size: min(calc(140 * var(--font-step-px)), var(--party-hold-plate)); padding: var(--space-md) var(--space-lg); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; container-type: inline-size; }
[part="results.row"][data-best=true] { box-shadow: var(--result-best-rim); }
#StatHead { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: var(--space-xs); }
[part="results.label"] { min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); overflow-wrap: anywhere; }
#Best { flex: none; padding: 0 var(--space-xs); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-accent-ink); background: var(--hud-key-gold-fill); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; white-space: nowrap; }
/* The figure fills its plate up to the size step, never wider than it. */
[part="results.figure"] { font-size: min(var(--result-size-figure), 28cqi, var(--party-hold-figure)); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink); white-space: nowrap; }
#RewardsColumn { display: flex; flex-direction: column; gap: var(--space-sm); min-inline-size: 0; }
#RewardsHeading { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); }
#Rewards { display: flex; flex-wrap: wrap; gap: var(--space-md); }
/* A reward: its art in a lit octagon, the amount on a tag across its
   foot, the name under it (wrapping, never dropped). */
[part="results.cell"] { display: flex; flex-direction: column; align-items: center; inline-size: calc(116 * var(--font-step-px)); background: none; box-shadow: none; border: 0; }
#Socket { inline-size: calc(104 * var(--font-step-px)); block-size: calc(104 * var(--font-step-px)); padding: var(--stroke-thick); clip-path: var(--hud-octagon); background: var(--hud-edge); --layout-anchor-inset: var(--stroke-thick); }
#Art { clip-path: var(--hud-octagon); background: var(--hud-plate-fill); object-fit: cover; }
#Amount { margin-block-start: calc(var(--space-md) * -1); padding: 0 var(--space-xs); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); white-space: nowrap; }
#RewardName { margin-block-start: var(--space-xs); inline-size: stretch; text-align: center; font-size: var(--hud-size-xs); font-weight: var(--hud-label-weight); color: var(--hud-ink); overflow-wrap: anywhere; }
#Footer { flex: none; display: flex; flex-wrap: wrap; justify-content: end; gap: var(--space-sm); }
#Footer button { min-block-size: var(--size-touch); min-inline-size: calc(var(--size-xxl) * 1.5); }
#Replay { min-inline-size: 0; }
@media (width < 1000px) {
  :root { padding: var(--space-xl); }
  #Summary { grid-template-columns: minmax(0, 1fr); gap: var(--space-lg); }
}
/* A phone: the verdict held to its column, the stats one plate a line
   (name, then the figure at the end), the actions two up with replay
   under them, every label kept whole. */
@media (width < 600px) {
  :root { padding: var(--space-md); }
  [part="results.frame"], #Page { gap: var(--space-md); }
  #Page { justify-content: flex-start; }
  #Head { align-items: flex-start; gap: var(--space-md); }
  #Text { container-type: inline-size; gap: var(--space-sm); }
  [part="results.title"] { font-size: min(var(--result-size-hero-compact), 22cqi); }
  #LevelBadge { inline-size: calc(40 * var(--font-step-px)); block-size: calc(40 * var(--font-step-px)); line-height: calc(40 * var(--font-step-px)); font-size: var(--hud-size-sm); }
  #LevelValue { display: none; }
  [part="results.grade"] { font-size: var(--result-size-grade-compact); inline-size: calc(var(--result-size-grade-compact) * 1.5); block-size: calc(var(--result-size-grade-compact) * 1.72); line-height: calc(var(--result-size-grade-compact) * 1.72); }
  #GradeCaption { display: none; }
  /* The tabs keep every label whole on a phone at large text: the type a
     step smaller and the tracking tight, so three fit the track. */
  #Tabs { --tabs-font-size: var(--result-tab-size-compact); --tabs-tracking: var(--result-tab-tracking-compact); }
  #Stats { grid-template-columns: minmax(0, 1fr); gap: var(--space-xs); }
  [part="results.row"] { flex-direction: row; align-items: center; min-block-size: 0; padding: var(--space-sm) var(--space-md); container-type: normal; }
  #StatHead { flex: 1; justify-content: flex-start; }
  [part="results.figure"] { font-size: var(--result-size-figure-compact); }
  [part="results.cell"] { inline-size: calc(84 * var(--font-step-px)); }
  #Socket { inline-size: calc(68 * var(--font-step-px)); block-size: calc(68 * var(--font-step-px)); }
  #Footer { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); }
  #Footer button { min-inline-size: 0; }
  #Replay { grid-column: 1 / -1; grid-row: 2; }
}
/* A short landscape phone: the head beside the page's lower half, the
   actions in a column at the end, so every stat shows before a scroll. */
@media (height < 600px) and (orientation: landscape) {
  :root { padding: var(--space-md) var(--space-lg); }
  [part="results.frame"] { flex-direction: row; }
  #Page { justify-content: flex-start; gap: var(--space-sm); -og-scroll-fade: var(--space-lg); }
  [part="results.title"] { font-size: var(--result-size-hero-compact); }
  #Summary { grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr); }
  #Stats { grid-template-columns: minmax(0, 1fr); gap: var(--space-xs); }
  [part="results.row"] { flex-direction: row; align-items: center; min-block-size: 0; padding: var(--space-xs) var(--space-md); container-type: normal; }
  #StatHead { flex: 1; justify-content: flex-start; }
  [part="results.figure"] { font-size: var(--result-size-figure-compact); }
  [part="results.grade"] { font-size: var(--result-size-grade-compact); inline-size: calc(var(--result-size-grade-compact) * 1.5); block-size: calc(var(--result-size-grade-compact) * 1.72); line-height: calc(var(--result-size-grade-compact) * 1.72); }
  #Footer { flex-direction: column; flex-wrap: nowrap; justify-content: flex-end; }
}
@media (prefers-reduced-motion: reduce) {
  [part="results.title"] { animation: none; }
}
</style>
