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

<div id="Results">
  <section id="Panel" part="results.frame">
    <div id="Page">
      <header id="Head">
        <div id="Text">
          <h1 id="Title" part="results.title" class:outcome-lost={!won} class:outcome-won={won}>{title}</h1>
          <p id="Gained">{t("screen.results.Gained", { gained: gained })}</p>
          <div id="Level">
            <p id="LevelValue">{t("screen.results.LevelValue", { level: level })}</p>
            <progress id="Xp" bind:value={xp}></progress>
          </div>
        </div>
        {#if grade != ""}<p id="Grade" part="results.grade" role="heading">{grade}</p>{/if}
      </header>
      <div id="Celebration">{@render celebration?.()}</div>
      <tabs id="Tabs" items="Summary Graphs Scoreboard" keys="screen.results.summary screen.results.graphs screen.results.board" values="summary graphs board" bind:current={tab} fill="true"></tabs>
      {#if tab == "summary"}
        <div id="Summary">
          <div id="Stats">
            {#each stats as s (s.id)}
              <div id="Stat" part="results.row">
                <p id="Name" part="results.label">{s.name}</p>
                {#if s.best}<badge id="Best" tone="accent">{t("screen.results.best")}</badge>{/if}
                {#if s.count}
                  <og-counter id="StatCount" part="results.figure" value={s.value} from="0" format={s.format == 'percent' ? 'percent' : 'grouped'} decimals={s.decimals}></og-counter>
                {:else}
                  <p id="StatValue" part="results.figure">{t("screen.results.StatValue", { format: s.format, value: s.format == 'percent' ? s.value * 100 : s.value })}</p>
                {/if}
              </div>
            {/each}
          </div>
          <div id="Rewards">
            {#each rewards as r (r.id)}
              <div id="Reward" part="results.cell">
                <art id="Art" kind="itemIcon" src={r.texture}></art>
                <p id="Amount">{t("screen.results.Amount", { r_amount: formatCompact(r.amount), r_name: r.name })}</p>
              </div>
            {/each}
          </div>
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
      <button id="Continue" variant="primary" onclick={continue} onconfirm={continue}>{t("screen.results.continue")}</button>
    </footer>
  </section>
</div>

<style>
:root { display: flex; align-items: center; justify-content: center; block-size: stretch; padding: var(--space-xxl); }
h1, p { margin: 0; }
/* A page that scrolls over a footer of actions; beside it on a short
   landscape phone. */
[part="results.frame"] { display: flex; flex-direction: column; gap: var(--space-lg); inline-size: 100%; max-inline-size: var(--measure-3xl); block-size: 100%; max-block-size: var(--measure-3xl); }
#Page { flex: 1; min-block-size: 0; display: flex; flex-direction: column; gap: var(--space-lg); overflow-y: auto; -og-scroll-fade: var(--space-xxl); }
#Head { display: flex; align-items: center; gap: var(--space-lg); }
#Text { flex: 1; min-inline-size: 0; display: flex; flex-direction: column; gap: var(--space-xs); }
[part="results.title"] { font-size: var(--type-xl4-size); letter-spacing: var(--type-xl4-tracking); }
.outcome-lost { color: var(--danger); }
.outcome-won { color: var(--gold); }
#Gained { color: var(--xp); }
#Level, [part="results.row"] { display: flex; align-items: center; }
#Level { gap: var(--space-md); }
#LevelValue, [part="results.figure"] { font: var(--text-lg); letter-spacing: var(--type-lg-tracking); }
#Xp { flex: 1; }
[part="results.grade"] { font: var(--text-display-lg); letter-spacing: var(--type-display-lg-tracking); }
#Summary { display: flex; flex-direction: column; gap: var(--space-md); }
#Stats { display: flex; flex-direction: column; gap: var(--space-sm); }
[part="results.row"] { gap: var(--space-sm); }
[part="results.figure"] { margin-inline-start: auto; }
#Rewards { display: flex; flex-wrap: wrap; gap: var(--space-sm); }
[part="results.cell"] { display: flex; flex-direction: column; align-items: center; gap: var(--space-xs); padding: var(--space-sm); }
#Art { inline-size: var(--size-xl); block-size: var(--size-xl); }
#Amount { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); }
#Footer { display: flex; flex-wrap: wrap; justify-content: end; gap: var(--space-sm); }
@media (width < 600px), (height < 600px) {
  :root { padding: var(--space-md); }
  [part="results.frame"], #Page { gap: var(--space-md); }
  [part="results.title"] { font-size: var(--type-xl2-size); letter-spacing: var(--type-xl2-tracking); }
  [part="results.grade"] { font: var(--text-xl4); letter-spacing: var(--type-xl4-tracking); }
}
/* A short landscape phone: the result's line and its XP share a line and
   the stats close up, so every stat shows before the page scrolls; the
   rewards follow under a shorter fade. */
@media (height < 600px) and (orientation: landscape) {
  [part="results.frame"] { flex-direction: row; }
  #Page { gap: var(--space-sm); -og-scroll-fade: var(--space-lg); }
  #Text { flex-direction: row; flex-wrap: wrap; align-items: baseline; column-gap: var(--space-md); }
  #Level { flex: 1 0 100%; }
  #Stats { gap: var(--space-xs); }
  #Footer { flex-direction: column; flex-wrap: nowrap; }
}
</style>
