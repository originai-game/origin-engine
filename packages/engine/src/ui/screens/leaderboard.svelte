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
        <div id="Row" part="leaderboard.row" interactive="true" focusable="true" onclick={row} class:mine-other={!r.you} class:mine-you={r.you} data-place={r.rank <= 3 ? r.rank : 0} data-promote={tab == "league" && r.rank <= league.promote}>
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
/* The board over the dimmed scene: a condensed title, a segmented switch,
   the league's plate with its promote and demote lines, rows on cut dark
   glass (the top three on gold, silver and bronze, the promotion zone
   edged green) and the player's own row pinned lit at the foot. */
:root { display: flex; block-size: stretch; background: var(--result-scrim); }
h1, h2, p { margin: 0; }
[part="leaderboard.frame"] { flex: 1; min-inline-size: 0; display: flex; flex-direction: column; gap: var(--space-md); padding: var(--space-xxl); }
#Screen > * { inline-size: 100%; max-inline-size: calc(var(--measure-3xl) * 1.1); margin-inline: auto; }
[part="leaderboard.band"] { display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-sm) var(--space-md); border: 0; min-block-size: 0; padding: 0; }
[part="leaderboard.title"] { flex: 1 1 auto; min-inline-size: 0; font-size: var(--result-size-title); line-height: 1; font-weight: var(--font-weight-bold); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-title-tracking); text-transform: uppercase; color: var(--hud-ink); }
#Period, #Period::part(base) { block-size: var(--control-sm); }
#Period::part(base) { padding: 0 var(--space-md); }
/* The switch: one well, the chosen tab a lit face in it. */
#Tabs { gap: 0; inline-size: fit-content; padding: var(--stroke-thick); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
#Tabs button::part(base) { border-color: transparent; border-radius: 0; text-transform: var(--hud-label-transform); letter-spacing: var(--result-tab-tracking); }
button.selection-off::part(base) { color: var(--hud-ink-dim); }
button.selection-on::part(base) { color: var(--result-row-lit-ink); background: var(--result-row-lit); }
[part="leaderboard.league"], [part="leaderboard.me"] { display: flex; align-items: center; gap: var(--space-md); padding: var(--space-sm) var(--space-md); clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); }
[part="leaderboard.league"] { flex-wrap: wrap; background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); }
#LeagueBarName { font-size: var(--hud-size-md); font-weight: var(--font-weight-bold); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-title-tracking); text-transform: uppercase; color: var(--result-gold); }
#Lines { flex: 1 1 auto; min-inline-size: 0; font: var(--text-caption); color: var(--hud-ink-dim); }
#Ends { padding: 0 var(--space-xs); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); white-space: nowrap; }
#Body { flex: 1; min-block-size: 0; overflow-y: auto; -og-scroll-fade: var(--space-xl); }
#List { display: flex; flex-direction: column; gap: var(--space-xxs); }
/* A row: rank box, emblem, name, score; the name keeps the room. */
[part="leaderboard.row"] { display: grid; grid-template-columns: auto auto minmax(0, 1fr) auto; grid-template-areas: "rank art name score"; align-items: center; column-gap: var(--space-md); padding: var(--space-xs) var(--space-md) var(--space-xs) var(--space-xs); background: var(--result-row-fill); clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); color: var(--hud-ink); }
[part="leaderboard.row"]:hover, [part="leaderboard.row"]:focus-visible { background: var(--result-row-hover); box-shadow: var(--result-row-hover-rim); }
[part="leaderboard.row"]:active { scale: 0.99; }
[part="leaderboard.row"][data-promote=true] { box-shadow: inset calc(3 * var(--stroke-hairline)) 0 0 var(--result-promote); }
.mine-you { background: var(--result-row-lit); color: var(--result-row-lit-ink); }
#RowRank { grid-area: rank; }
#Avatar { grid-area: art; inline-size: var(--size-md); block-size: var(--size-md); clip-path: var(--hud-octagon); }
#RowName { grid-area: name; }
#RowScore { grid-area: score; }
[part="leaderboard.rank"] { display: block; text-align: center; min-inline-size: calc(52 * var(--font-step-px)); line-height: calc(38 * var(--font-step-px)); padding: 0 var(--space-xs); white-space: nowrap; font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); background: var(--hud-place-fill); clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
[data-place="1"] [part="leaderboard.rank"] { color: var(--result-medal-ink); background: var(--result-gold); }
[data-place="2"] [part="leaderboard.rank"] { color: var(--result-medal-ink); background: var(--result-silver); }
[data-place="3"] [part="leaderboard.rank"] { color: var(--result-medal-ink); background: var(--result-bronze); }
[data-place="1"] #RowScore { color: var(--result-gold); }
.name { min-inline-size: 0; font-size: var(--hud-size-sm); font-weight: var(--hud-label-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
[part="leaderboard.figure"] { font-size: var(--hud-size-md); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-align: end; }
/* The player's own row: lit white, pinned under the list. */
[part="leaderboard.me"] { padding-inline-start: var(--space-xs); background: var(--result-row-lit); color: var(--result-row-lit-ink); }
[part="leaderboard.me"] [part="leaderboard.rank"] { background: var(--result-me-rank-fill); }
#MeName { flex: 1; }
#MeScore { font-size: var(--hud-size-lg); }
/* The lit rows' figures take the lit ink, not the figure role's white. */
.mine-you [part="leaderboard.figure"], [part="leaderboard.me"] [part="leaderboard.figure"] { color: var(--result-row-lit-ink); }
@media (width < 600px), (height < 600px) {
  [part="leaderboard.frame"] { padding: var(--space-md); gap: var(--space-sm); }
  [part="leaderboard.title"] { font-size: var(--result-size-title-compact); }
  #Period, #Period::part(base) { block-size: var(--control-md); }
  #Tabs { inline-size: 100%; }
  /* Three segments share the width whole: tighter tracking and side padding.
     A label wider than its third (a wide face at large text) steps its type
     down until it fits, and only past the caption step ends in an ellipsis,
     inside its own segment, never running into its neighbours. */
  #Tabs button { flex: 1 1 0; min-inline-size: 0; }
  #Tabs button::part(base) { min-inline-size: 0; padding-inline: var(--space-xs); letter-spacing: var(--result-tab-tracking-compact); -og-fit-min: caption; text-overflow: ellipsis; }
  [part="leaderboard.row"] { column-gap: var(--space-sm); }
  [part="leaderboard.rank"] { min-inline-size: calc(38 * var(--font-step-px)); line-height: calc(32 * var(--font-step-px)); }
  #Avatar { inline-size: var(--size-sm); block-size: var(--size-sm); }
  [part="leaderboard.figure"] { font-size: var(--hud-size-sm); }
  #MeScore { font-size: var(--hud-size-md); }
}
/* A narrow row at a large text scale: the score drops under the name, so
   the name keeps the row's width and is never cut to a few letters. */
@container (width < 26rem) {
  [part="leaderboard.row"] { grid-template-columns: auto auto minmax(0, 1fr); grid-template-areas: "rank art name" "rank art score"; row-gap: 0; }
  #RowScore { text-align: start; }
}
#List { container-type: inline-size; }
</style>
