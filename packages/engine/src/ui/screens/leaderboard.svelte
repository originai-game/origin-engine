<meta name="screen" content="leaderboard" data-layer="menu" data-focus="Body/List" data-under="dim" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="pop">
<meta name="strings" content="./leaderboard.en.ogt">
<meta name="parts" content="leaderboard.frame leaderboard.band leaderboard.title leaderboard.league leaderboard.row leaderboard.rank leaderboard.figure leaderboard.me">
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

<column id="Screen" part="leaderboard.frame" class="bare">
  <row id="Header" part="leaderboard.band">
    <button id="Back" variant="quiet" onclick={back}>{t("screen.common.back")}</button>
    <text id="Title" part="leaderboard.title" heading="true">{t("screen.leaderboard.title")}</text>
    <spacer id="Node4"></spacer>
    <button id="Period" onclick={period}>{$period}</button>
  </row>
  <row id="Tabs" focus-group="wrap">
    <button id="Global" variant="ghost" class:selection-off={!(($tab == "global"))} class:selection-on={($tab == "global")}>{t("screen.leaderboard.global")}</button>
    <button id="Friends" variant="ghost" class:selection-off={!(($tab == "friends"))} class:selection-on={($tab == "friends")}>{t("screen.leaderboard.friends")}</button>
    <button id="League" variant="ghost" class:selection-off={!(($tab == "league"))} class:selection-on={($tab == "league")}>{t("screen.leaderboard.league")}</button>
  </row>
  <row id="LeagueBar" part="leaderboard.league" hidden={!(($tab == "league"))}>
    <text id="LeagueBarName" heading="true">{$league.name}</text>
    <text id="Lines">{t("screen.leaderboard.Lines", { league_demote: $league.demote, league_promote: $league.promote })}</text>
    <spacer id="Node13"></spacer>
    <text id="Ends">{t("screen.leaderboard.Ends", { league_ends: formatDuration($league.ends) })}</text>
  </row>
  <scroll id="Body">
    <column id="List" focus-group="stop">
      <column id="Board">{@render board?.()}</column>
      {#each $rows as r (r.id)}
        <row id="Row" part="leaderboard.row" interactive="true" focusable="true" onclick={row} class:mine-other={!(r.you)} class:mine-you={r.you}>
          <text id="RowRank" part="leaderboard.rank">{t("screen.leaderboard.RowRank", { value: formatOrdinal(r.rank) })}</text>
          <art id="Avatar" kind="emblem" src={r.texture}></art>
          <text id="RowName">{r.name}</text>
          <text id="RowScore" part="leaderboard.figure">{t("screen.leaderboard.RowScore", { value: r.score })}</text>
        </row>
      {/each}
    </column>
  </scroll>
  <row id="Me" part="leaderboard.me">
    <text id="MeRank" part="leaderboard.rank">{t("screen.leaderboard.MeRank", { value: formatOrdinal($me.rank) })}</text>
    <text id="MeName">{$me.name}</text>
    <text id="MeScore" part="leaderboard.figure">{t("screen.leaderboard.MeScore", { value: $me.score })}</text>
  </row>
</column>

<style>
#Period, #Period::part(base) { height: var(--control-sm); }
#Period::part(base) { padding: 0 var(--space-md); }
@media (width < 600px), (height < 600px) {
#Period, #Period::part(base) { height: var(--control-md); }
#Period::part(base) { padding: 0 var(--space-lg); }
}
#Screen { width: stretch; height: stretch; padding: var(--space-xxl); gap: var(--space-md); }
@media (width < 600px), (height < 600px) {
#Screen { padding: var(--space-lg); }
}
#Header { width: stretch; gap: var(--space-md); align-items: center; flex-wrap: wrap; }
#Title { font-size: var(--type-xl2-size); }
@media (width < 600px), (height < 600px) {
#Title { font-size: var(--type-xl-size); }
}
#Tabs { gap: var(--space-sm); }
#Global.selection-off { opacity: var(--opacity-soft); }
#Global.selection-on { opacity: 1; }
#Friends.selection-off { opacity: var(--opacity-soft); }
#Friends.selection-on { opacity: 1; }
#League.selection-off { opacity: var(--opacity-soft); }
#League.selection-on { opacity: 1; }
#LeagueBar { width: stretch; gap: var(--space-md); padding: var(--space-md); align-items: center; flex-wrap: wrap; background: var(--panel-raised); border-radius: var(--radius-card); outline: var(--stroke-hairline) solid var(--line); outline-offset: calc(var(--stroke-hairline) * -1); }
#LeagueBarName { font: var(--text-lg); letter-spacing: var(--type-lg-tracking); }
#Lines { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); }
#Ends { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); }
#Body { width: stretch; height: stretch; }
#List { width: stretch; gap: var(--space-xs); }
#Board { width: stretch; }
#Row { width: stretch; padding: var(--space-sm); gap: var(--space-md); align-items: center; }
#Row.mine-you { background: var(--primary-soft); }
#Row { scale: 1; }
#Row:hover { scale: 1.01; }
#Row:active { scale: 0.99; }
#Row:focus-visible { scale: 1.01; }
#RowRank { min-width: var(--size-xl); white-space: nowrap; font: var(--text-lg); letter-spacing: var(--type-lg-tracking); }
#Avatar { width: var(--size-md); height: var(--size-md); }
#RowName { width: stretch; line-clamp: 1; text-overflow: ellipsis; }
#Me { width: stretch; padding: var(--space-md); gap: var(--space-md); align-items: center; background: var(--panel-raised); border-radius: var(--radius-card); outline: var(--stroke-hairline) solid var(--line); outline-offset: calc(var(--stroke-hairline) * -1); }
#MeRank { min-width: var(--size-xl); white-space: nowrap; font: var(--text-lg); letter-spacing: var(--type-lg-tracking); }
#MeName { width: stretch; line-clamp: 1; text-overflow: ellipsis; }
#MeScore { font: var(--text-lg); letter-spacing: var(--type-lg-tracking); }
</style>
