<meta name="screen" content="leaderboard" data-layer="menu" data-focus="Body/List" data-under="dim" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="pop">
<meta name="strings" content="./leaderboard.en.ogt">
<meta name="prop:tab" content="string" data-doc="global, friends or league.">
<meta name="prop:period" content="string" data-doc="The period, as text (This week).">
<meta name="prop:rows" content="list" data-doc="The top rows: { rank, name, score, you (bool), texture }.">
<meta name="prop:selected" content="string" data-doc="The stable collection key selected now.">
<meta name="prop:me" content="object" data-doc="The player's row: { rank, name, score }.">
<meta name="prop:league" content="object" data-doc="The league: { name, promote (rank), demote (rank), ends (seconds) }.">
<meta name="action:confirm" content="Profile">
<meta name="action:back" content="Back">
<meta name="action:tabNext" content="Next tab">
<meta name="action:tabPrev" content="Previous tab">
<meta name="fixture:default" content="{&quot;league&quot;:{&quot;demote&quot;:25,&quot;ends&quot;:259200,&quot;name&quot;:&quot;Gold league&quot;,&quot;promote&quot;:10},&quot;me&quot;:{&quot;name&quot;:&quot;You&quot;,&quot;rank&quot;:1284,&quot;score&quot;:402110},&quot;period&quot;:&quot;This week&quot;,&quot;rows&quot;:[{&quot;id&quot;:&quot;item-0&quot;,&quot;name&quot;:&quot;Nyx&quot;,&quot;rank&quot;:1,&quot;score&quot;:982140,&quot;texture&quot;:0,&quot;you&quot;:false},{&quot;id&quot;:&quot;item-1&quot;,&quot;name&quot;:&quot;Ferro&quot;,&quot;rank&quot;:2,&quot;score&quot;:951300,&quot;texture&quot;:0,&quot;you&quot;:false},{&quot;id&quot;:&quot;item-2&quot;,&quot;name&quot;:&quot;Lumen&quot;,&quot;rank&quot;:3,&quot;score&quot;:948020,&quot;texture&quot;:0,&quot;you&quot;:false},{&quot;id&quot;:&quot;item-3&quot;,&quot;name&quot;:&quot;Quill&quot;,&quot;rank&quot;:4,&quot;score&quot;:902210,&quot;texture&quot;:0,&quot;you&quot;:false}],&quot;selected&quot;:-1,&quot;tab&quot;:&quot;global&quot;}">

<column id="Screen">
  <row id="Header">
    <button id="Back" variant="quiet" onclick={back}>{t("screen.common.back")}</button>
    <text id="Title" heading="true">{t("screen.leaderboard.title")}</text>
    <spacer id="Node4"></spacer>
    <button id="Period" size="small" onclick={period}>{$period}</button>
  </row>
  <row id="Tabs" focus-group="wrap">
    <button id="Global" variant="ghost" class:selection-off={!(($tab == "global"))} class:selection-on={($tab == "global")}>{t("screen.leaderboard.global")}</button>
    <button id="Friends" variant="ghost" class:selection-off={!(($tab == "friends"))} class:selection-on={($tab == "friends")}>{t("screen.leaderboard.friends")}</button>
    <button id="League" variant="ghost" class:selection-off={!(($tab == "league"))} class:selection-on={($tab == "league")}>{t("screen.leaderboard.league")}</button>
  </row>
  <row id="LeagueBar" hidden={!(($tab == "league"))}>
    <text id="LeagueBarName" heading="true">{$league.name}</text>
    <text id="Lines">{t("screen.leaderboard.Lines", { league_demote: $league.demote, league_promote: $league.promote })}</text>
    <spacer id="Node13"></spacer>
    <text id="Ends">{t("screen.leaderboard.Ends", { league_ends: formatDuration($league.ends) })}</text>
  </row>
  <scroll id="Body">
    <column id="List" focus-group="stop">
      <column id="Board">{@render board?.()}</column>
      {#each $rows as r (r.id)}
        <row id="Row" interactive="true" focusable="true" onclick={row} class:mine-other={!(r.you)} class:mine-you={r.you}>
          <text id="RowRank">{t("screen.leaderboard.RowRank", { value: formatOrdinal(r.rank) })}</text>
          <art id="Avatar" kind="emblem" src={r.texture}></art>
          <text id="RowName">{r.name}</text>
          <text id="RowScore">{t("screen.leaderboard.RowScore", { value: r.score })}</text>
        </row>
      {/each}
    </column>
  </scroll>
  <row id="Me">
    <text id="MeRank">{t("screen.leaderboard.MeRank", { value: formatOrdinal($me.rank) })}</text>
    <text id="MeName">{$me.name}</text>
    <text id="MeScore">{t("screen.leaderboard.MeScore", { value: $me.score })}</text>
  </row>
</column>

<style>
#Screen { width: stretch; height: stretch; padding: 32px; gap: 12px; }
@media (width < 600px), (height < 600px) {
#Screen { padding: 16px; }
}
#Header { width: stretch; gap: 12px; align-items: center; flex-wrap: wrap; }
#Title { font: var(--text-xl2); letter-spacing: var(--type-xl2-tracking); }
@media (width < 600px), (height < 600px) {
#Title { font: var(--text-xl); letter-spacing: var(--type-xl-tracking); }
}
@media (width < 600px), (height < 600px) {
#Period { -og-control-size: medium; }
}
#Tabs { gap: 6px; }
#Global.selection-off { opacity: 0.85; }
#Global.selection-on { opacity: 1; }
#Friends.selection-off { opacity: 0.85; }
#Friends.selection-on { opacity: 1; }
#League.selection-off { opacity: 0.85; }
#League.selection-on { opacity: 1; }
#LeagueBar { width: stretch; gap: 12px; padding: 10px; align-items: center; flex-wrap: wrap; background: var(--panel-raised); border-radius: var(--radius-card); outline: 1px solid var(--line); outline-offset: -1px; }
#LeagueBarName { font: var(--text-lg); letter-spacing: var(--type-lg-tracking); }
#Lines { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); }
#Ends { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); }
#Body { width: stretch; height: stretch; }
#List { width: stretch; gap: 4px; }
#Board { width: stretch; }
#Row { width: stretch; padding: 8px; gap: 12px; align-items: center; border-radius: 8px; }
#Row.mine-other { background: var(--panel); }
#Row.mine-you { background: var(--primary-soft); }
#Row { scale: 1; }
#Row:hover { scale: 1.01; }
#Row:active { scale: 0.99; }
#Row:focus-visible { scale: 1.01; }
#RowRank { min-width: 56px; white-space: nowrap; font: var(--text-lg); letter-spacing: var(--type-lg-tracking); }
#Avatar { width: 36px; height: 36px; }
#RowName { width: stretch; line-clamp: 1; text-overflow: ellipsis; }
#Me { width: stretch; padding: 10px; gap: 12px; align-items: center; background: var(--panel-raised); border-radius: var(--radius-card); outline: 1px solid var(--line); outline-offset: -1px; }
#MeRank { min-width: 56px; white-space: nowrap; font: var(--text-lg); letter-spacing: var(--type-lg-tracking); }
#MeName { width: stretch; line-clamp: 1; text-overflow: ellipsis; }
#MeScore { font: var(--text-lg); letter-spacing: var(--type-lg-tracking); }
</style>
