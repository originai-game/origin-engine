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

<div id="Screen" part="leaderboard.frame" class="bare">
  <header id="Header" part="leaderboard.band">
    <button id="Back" variant="quiet" onclick={back}>{t("screen.common.back")}</button>
    <h1 id="Title" part="leaderboard.title">{t("screen.leaderboard.title")}</h1>
    <button id="Period" onclick={period}>{period}</button>
  </header>
  <div id="Tabs" data-layout="cluster" focus-group="wrap">
    <button id="Global" variant="ghost" class:selection-off={tab != "global"} class:selection-on={tab == "global"}>{t("screen.leaderboard.global")}</button>
    <button id="Friends" variant="ghost" class:selection-off={tab != "friends"} class:selection-on={tab == "friends"}>{t("screen.leaderboard.friends")}</button>
    <button id="League" variant="ghost" class:selection-off={tab != "league"} class:selection-on={tab == "league"}>{t("screen.leaderboard.league")}</button>
  </div>
  {#if tab == "league"}
    <div id="LeagueBar" part="leaderboard.league">
      <h2 id="LeagueBarName">{league.name}</h2>
      <p id="Lines">{t("screen.leaderboard.Lines", { league_demote: league.demote, league_promote: league.promote })}</p>
      <p id="Ends">{t("screen.leaderboard.Ends", { league_ends: formatDuration(league.ends) })}</p>
    </div>
  {/if}
  <div id="Body">
    <div id="List" focus-group="stop">
      <div id="Board">{@render board?.()}</div>
      {#each rows as r (r.id)}
        <div id="Row" part="leaderboard.row" interactive="true" focusable="true" onclick={row} class:mine-other={!r.you} class:mine-you={r.you}>
          <p id="RowRank" part="leaderboard.rank">{t("screen.leaderboard.RowRank", { value: formatOrdinal(r.rank) })}</p>
          <art id="Avatar" kind="emblem" src={r.texture}></art>
          <p id="RowName" class="name">{r.name}</p>
          <p id="RowScore" part="leaderboard.figure">{t("screen.leaderboard.RowScore", { value: r.score })}</p>
        </div>
      {/each}
    </div>
  </div>
  <div id="Me" part="leaderboard.me">
    <p id="MeRank" part="leaderboard.rank">{t("screen.leaderboard.MeRank", { value: formatOrdinal(me.rank) })}</p>
    <p id="MeName" class="name">{me.name}</p>
    <p id="MeScore" part="leaderboard.figure">{t("screen.leaderboard.MeScore", { value: me.score })}</p>
  </div>
</div>

<style>
:root { display: flex; flex-direction: column; gap: var(--space-md); block-size: stretch; padding: var(--space-xxl); }
h1, h2, p { margin: 0; }
[part="leaderboard.band"] { display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-md); }
[part="leaderboard.title"] { flex: 1; font-size: var(--type-xl2-size); letter-spacing: var(--type-xl2-tracking); }
#Period, #Period::part(base) { block-size: var(--control-sm); }
#Period::part(base) { padding: 0 var(--space-md); }
/* The tabs wrap onto a second line where the row is too narrow for them. */
#Tabs { gap: var(--space-sm); }
/* An unselected tab keeps its words' ink; only its rim goes. */
button.selection-off::part(base) { border-color: transparent; }
/* The league bar and the player's own row: raised plates. */
[part="leaderboard.league"], [part="leaderboard.me"] { display: flex; align-items: center; gap: var(--space-md); padding: var(--space-md); background: var(--panel-raised); border-radius: var(--radius-card); outline: var(--stroke-hairline) solid var(--line); outline-offset: calc(var(--stroke-hairline) * -1); }
[part="leaderboard.league"] { flex-wrap: wrap; }
#LeagueBarName { font: var(--text-lg); letter-spacing: var(--type-lg-tracking); }
#Lines { flex: 1; font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); }
#Ends { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); }
#Body { flex: 1; min-block-size: 0; overflow-y: auto; }
#List { display: flex; flex-direction: column; gap: var(--space-xs); }
[part="leaderboard.row"] { display: flex; align-items: center; gap: var(--space-md); padding: var(--space-sm); }
[part="leaderboard.row"]:hover, [part="leaderboard.row"]:focus-visible { scale: 1.01; }
[part="leaderboard.row"]:active { scale: 0.99; }
.mine-you { background: var(--primary-soft); }
[part="leaderboard.rank"] { min-inline-size: var(--size-xl); white-space: nowrap; font: var(--text-lg); letter-spacing: var(--type-lg-tracking); }
#Avatar { inline-size: var(--size-md); block-size: var(--size-md); }
.name { flex: 1; min-inline-size: 0; line-clamp: 1; text-overflow: ellipsis; }
#MeScore { font: var(--text-lg); letter-spacing: var(--type-lg-tracking); }
@media (width < 600px), (height < 600px) {
  :root { padding: var(--space-lg); }
  [part="leaderboard.title"] { font-size: var(--type-xl-size); letter-spacing: var(--type-xl-tracking); }
  #Period, #Period::part(base) { block-size: var(--control-md); }
  #Period::part(base) { padding: 0 var(--space-lg); }
}
</style>
