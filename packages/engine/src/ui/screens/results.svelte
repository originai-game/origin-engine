<meta name="screen" content="results" data-layer="menu" data-focus="Panel/Footer/Continue" data-under="hide" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="none">
<meta name="strings" content="./results.en.ogt">
<meta name="prop:won" content="bool" data-doc="The player won.">
<meta name="prop:grade" content="string" data-doc="The grade (S, A, B …; empty: none).">
<meta name="prop:title" content="string" data-doc="The result's line (Victory, Stage clear).">
<meta name="prop:stats" content="list" data-doc="The stats: { name, value, best (bool), format (number or percent; defaults to number) }.">
<meta name="prop:level" content="number" data-doc="The level after the match.">
<meta name="prop:xp" content="number" data-doc="Progress to the next level after the match, 0 to 1.">
<meta name="prop:gained" content="number" data-doc="XP gained.">
<meta name="prop:rewards" content="list" data-doc="The rewards: { name, amount, texture }.">
<meta name="prop:tab" content="string" data-doc="summary, graphs or board.">
<meta name="action:confirm" content="Continue">
<meta name="fixture:default" content="{&quot;gained&quot;:1240,&quot;grade&quot;:&quot;A&quot;,&quot;level&quot;:18,&quot;rewards&quot;:[{&quot;amount&quot;:320,&quot;id&quot;:&quot;item-0&quot;,&quot;name&quot;:&quot;Gold&quot;,&quot;texture&quot;:0},{&quot;amount&quot;:1,&quot;id&quot;:&quot;item-1&quot;,&quot;name&quot;:&quot;Crate&quot;,&quot;texture&quot;:0}],&quot;stats&quot;:[{&quot;best&quot;:true,&quot;id&quot;:&quot;item-0&quot;,&quot;name&quot;:&quot;Score&quot;,&quot;value&quot;:48210},{&quot;best&quot;:false,&quot;id&quot;:&quot;item-1&quot;,&quot;name&quot;:&quot;Kills&quot;,&quot;value&quot;:14},{&quot;best&quot;:false,&quot;format&quot;:&quot;percent&quot;,&quot;id&quot;:&quot;item-2&quot;,&quot;name&quot;:&quot;Accuracy&quot;,&quot;value&quot;:0.62}],&quot;tab&quot;:&quot;summary&quot;,&quot;title&quot;:&quot;Victory&quot;,&quot;won&quot;:true,&quot;xp&quot;:0.45}">

<column id="Results">
  <column id="Panel">
    <scroll id="Page">
      <row id="Head">
        <column id="Text">
          <text id="Title" heading="true" class:outcome-lost={!($won)} class:outcome-won={$won}>{$title}</text>
          <text id="Gained">{t("screen.results.Gained", { gained: $gained })}</text>
          <row id="Level">
            <text id="LevelValue">{t("screen.results.LevelValue", { level: $level })}</text>
            <progress id="Xp" bind:value={$xp}></progress>
          </row>
        </column>
        <text id="Grade" heading="true" hidden={!(($grade != ""))}>{$grade}</text>
      </row>
      <column id="Celebration">{@render celebration?.()}</column>
      <tabs id="Tabs" items="Summary Graphs Scoreboard" keys="screen.results.summary screen.results.graphs screen.results.board" values="summary graphs board" bind:current={$tab} fill="true"></tabs>
      <column id="Summary" hidden={!(($tab == "summary"))}>
        <column id="Stats">
          {#each $stats as s (s.id)}
            <row id="Stat">
              <text id="Name">{s.name}</text>
              <badge id="Best" tone="accent" hidden={!(s.best)}>{t("screen.results.best")}</badge>
              <spacer id="Node19"></spacer>
              <text id="StatValue">{t("screen.results.StatValue", { format: s.format, value: s.format == 'percent' ? s.value * 100 : s.value })}</text>
            </row>
          {/each}
        </column>
        <row id="Rewards">
          {#each $rewards as r (r.id)}
            <column id="Reward">
              <art id="Art" kind="itemIcon" src={r.texture}></art>
              <text id="Amount">{t("screen.results.Amount", { r_amount: formatCompact(r.amount), r_name: r.name })}</text>
            </column>
          {/each}
        </row>
      </column>
      <column id="Graph" hidden={!(($tab == "graphs"))}>{@render graph?.()}</column>
      <column id="Scores" hidden={!(($tab == "board"))}>{@render scores?.()}</column>
    </scroll>
    <row id="Footer">
      <button id="Replay" variant="quiet" onclick={replay}>{t("screen.results.replay")}</button>
      <button id="Again" onclick={again}>{t("screen.results.again")}</button>
      <button id="Continue" variant="primary" onclick={continue} onconfirm={continue}>{t("screen.results.continue")}</button>
    </row>
  </column>
</column>

<style>
:root { --surface-saturation: 120%; }
@media (prefers-color-scheme: light) { :root { --surface-saturation: 200%; } }
#Results { width: stretch; height: stretch; padding: 32px; justify-content: center; align-items: center; }
@media (width < 600px), (height < 600px) {
#Results { padding: 12px; }
}
#Panel { width: stretch; max-width: 960px; height: stretch; max-height: 900px; padding: 24px; gap: 16px;  border-radius: var(--radius-panel); backdrop-filter: blur(32px) saturate(var(--surface-saturation)); -og-material: glass thick lift; }
@media (width < 600px), (height < 600px) {
#Panel { padding: 16px; gap: 12px; }
}
@media (height < 600px) and (orientation: landscape) {
#Panel { flex-direction: row; padding: 12px; gap: 12px; }
}
#Page { width: stretch; height: stretch; gap: 16px; -og-scroll-fade: 40px; }
@media (width < 600px), (height < 600px) {
#Page { gap: 12px; }
}
@media (height < 600px) and (orientation: landscape) {
#Page { gap: 8px; }
}
#Head { width: stretch; gap: 16px; align-items: center; }
#Text { width: stretch; gap: 4px; }
#Title { font: var(--text-xl4); letter-spacing: var(--type-xl4-tracking); }
@media (width < 600px), (height < 600px) {
#Title { font: var(--text-xl2); letter-spacing: var(--type-xl2-tracking); }
}
#Title.outcome-lost { color: var(--danger); }
#Title.outcome-won { color: var(--gold); }
#Gained { color: var(--xp); }
#Level { width: stretch; gap: 12px; align-items: center; }
#LevelValue { font: var(--text-lg); letter-spacing: var(--type-lg-tracking); }
#Xp { width: stretch; }
#Grade { font: var(--text-display-lg); letter-spacing: var(--type-display-lg-tracking); }
@media (width < 600px), (height < 600px) {
#Grade { font: var(--text-xl4); letter-spacing: var(--type-xl4-tracking); }
}
#Celebration { width: stretch; }
#Tabs { width: stretch; }
#Summary { width: stretch; gap: 12px; }
#Stats { width: stretch; gap: 6px; }
#Stat { width: stretch; gap: 8px; align-items: center; }
#Name { color: var(--ink-muted); }
#StatValue { font: var(--text-lg); letter-spacing: var(--type-lg-tracking); }
#Rewards { width: stretch; gap: 8px; flex-wrap: wrap; }
#Reward { padding: 8px; gap: 4px; align-items: center; background: var(--panel-raised); border-radius: var(--radius-card); outline: 1px solid var(--line); outline-offset: -1px; }
#Art { width: 56px; height: 56px; }
#Amount { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); }
#Graph { width: stretch; }
#Scores { width: stretch; }
#Footer { width: stretch; gap: 8px; justify-content: end; flex-wrap: wrap; }
@media (height < 600px) and (orientation: landscape) {
#Footer { flex-direction: column; width: fit-content; height: stretch; align-items: stretch; flex-wrap: nowrap; }
}
</style>
