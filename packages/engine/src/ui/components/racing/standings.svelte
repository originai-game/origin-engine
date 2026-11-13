<meta name="component" content="RacingStandings">
<meta name="strings" content="./racing.en.ogt">
<meta name="parts" content="RacingStandings.frame RacingStandings.band RacingStandings.title RacingStandings.row RacingStandings.cell RacingStandings.name RacingStandings.gap RacingStandings.badge">
<meta name="prop:title" content="string" data-doc="The board's heading; empty takes the kit's (Standings).">
<meta name="prop:rows" content="list" data-doc="The running order as the game ranks it, keyed rows of { id, position, name, gap, player, state }: position the place shown; gap the interval as the game words it (+1.204, +1 lap; empty for the leader); player marks the local player's row (You); state racing, pitting, retired or finished, each with a word as well as its dress.">
<meta name="fixture:default" content='{"title":"","rows":[{"id":"marlow","position":1,"name":"Marlow","gap":"","player":false,"state":"racing"},{"id":"okafor","position":2,"name":"Okafor","gap":"+0.884","player":false,"state":"racing"},{"id":"you","position":3,"name":"Rhea Vance","gap":"+1.512","player":true,"state":"racing"},{"id":"lindqvist","position":4,"name":"Lindqvist","gap":"+3.020","player":false,"state":"pitting"},{"id":"sato","position":5,"name":"Sato","gap":"+1 lap","player":false,"state":"racing"},{"id":"brandt","position":6,"name":"Brandt","gap":"","player":false,"state":"retired"}]}'>
<meta name="fixture:empty" content='{"title":"","rows":[]}'>
<meta name="fixture:single" content='{"title":"Time trial","rows":[{"id":"you","position":1,"name":"Rhea Vance","gap":"","player":true,"state":"racing"}]}'>
<meta name="fixture:finished" content='{"title":"","rows":[{"id":"marlow","position":1,"name":"Marlow","gap":"","player":false,"state":"finished"},{"id":"you","position":2,"name":"Rhea Vance","gap":"+0.207","player":true,"state":"finished"},{"id":"okafor","position":3,"name":"Okafor","gap":"+2.940","player":false,"state":"racing"}]}'>
<meta name="fixture:overflow" content='{"title":"","rows":[{"id":"r1","position":1,"name":"Maximilian Vandenberghe-Okonkwo","gap":"","player":false,"state":"racing"},{"id":"r2","position":2,"name":"Okafor","gap":"+0.884","player":false,"state":"racing"},{"id":"r3","position":3,"name":"Lindqvist","gap":"+1.020","player":false,"state":"racing"},{"id":"r4","position":4,"name":"Sato","gap":"+2.310","player":false,"state":"racing"},{"id":"r5","position":5,"name":"Brandt","gap":"+3.007","player":false,"state":"pitting"},{"id":"r6","position":6,"name":"Duarte","gap":"+4.551","player":false,"state":"racing"},{"id":"r7","position":7,"name":"Nakamura","gap":"+5.902","player":false,"state":"racing"},{"id":"r8","position":8,"name":"Haddad","gap":"+7.118","player":false,"state":"racing"},{"id":"r9","position":9,"name":"Kowalczyk","gap":"+8.640","player":false,"state":"racing"},{"id":"r10","position":10,"name":"Moreau","gap":"+9.215","player":false,"state":"racing"},{"id":"r11","position":11,"name":"Rhea Vance","gap":"+10.774","player":true,"state":"racing"},{"id":"r12","position":12,"name":"Ferreira","gap":"+12.330","player":false,"state":"racing"},{"id":"r13","position":13,"name":"Johansson","gap":"+15.004","player":false,"state":"racing"},{"id":"r14","position":14,"name":"Abernathy-Whitcombe","gap":"+1 lap","player":false,"state":"racing"},{"id":"r15","position":15,"name":"Ivanova","gap":"+1 lap","player":false,"state":"racing"},{"id":"r16","position":16,"name":"Quinn","gap":"+2 laps","player":false,"state":"racing"},{"id":"r17","position":17,"name":"Petrov","gap":"","player":false,"state":"retired"},{"id":"r18","position":18,"name":"Laurent","gap":"","player":false,"state":"retired"}]}'>

<!--
  The running order: a heading and one row per car (place, name, gap) in
  the order the game ranks them. The player's row is marked You and
  aria-current; pitting, retired and finished cars carry a short word
  badge. It fills the height it is given and scrolls inside it. Read only.
-->
<section id="RacingStandings" part="RacingStandings.frame" class="hud" aria-label={title != '' ? title : t("racing.standings.name")}>
  <header part="RacingStandings.band"><h2 part="RacingStandings.title">{title != '' ? title : t("racing.standings.name")}</h2></header>
  {#if rows.length}
    <scroll class="list">
      <div class="rows" role="list" aria-label={title != '' ? title : t("racing.standings.name")}>
        {#each rows as r (r.id)}
          <div part="RacingStandings.row" role="listitem" aria-current={r.player} aria-label={r.name} data-player={r.player} data-state={r.state}>
            <span part="RacingStandings.cell" class="place">{formatNumber(r.position)}</span>
            <span part="RacingStandings.name" class="name">{r.name}</span>
            <span class="you" hidden={!r.player}>{t("racing.standings.you")}</span>
            <span part="RacingStandings.badge" class="state" data-state={r.state} hidden={r.state == 'racing'} aria-label={r.state == 'pitting' ? t("racing.standings.pittingName") : (r.state == 'retired' ? t("racing.standings.retiredName") : t("racing.standings.finishedName"))}>{r.state == 'pitting' ? t("racing.standings.pitting") : (r.state == 'retired' ? t("racing.standings.retired") : t("racing.standings.finished"))}</span>
            <span part="RacingStandings.gap" class="gap" hidden={r.gap == ''}>{r.gap}</span>
          </div>
        {/each}
      </div>
    </scroll>
  {:else}
    <p class="empty-text" role="status">{t("racing.standings.empty")}</p>
  {/if}
</section>

<style>
[part="RacingStandings.frame"][hidden], [part="RacingStandings.frame"] [hidden] { display: none; }
[part="RacingStandings.frame"] { display: flex; flex-direction: column; width: stretch; max-height: stretch; gap: var(--space-xxs); }
[part="RacingStandings.band"] { display: flex; width: stretch; align-items: center; flex: none; }
[part="RacingStandings.title"] { margin: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.list { width: stretch; flex: 1; min-height: 0; }
.rows { display: flex; flex-direction: column; width: stretch; gap: var(--space-xxs); }
[part="RacingStandings.row"] { display: flex; width: stretch; align-items: center; gap: var(--space-sm); padding: var(--space-xxs) var(--space-sm); min-height: var(--size-sm); }
.place { flex: none; width: var(--size-sm); text-align: center; font-variant-numeric: tabular-nums; }
.name { flex: 1; min-width: 0; text-align: start; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.you, .state { flex: none; white-space: nowrap; padding: 0 var(--space-xs); }
.gap { flex: none; text-align: end; white-space: nowrap; font-variant-numeric: tabular-nums; }
.empty-text { margin: 0; padding: var(--space-sm); }
</style>
