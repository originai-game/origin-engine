<meta name="component" content="RacingResults">
<meta name="strings" content="./racing.en.ogt">
<meta name="parts" content="RacingResults.frame RacingResults.band RacingResults.title RacingResults.heading RacingResults.row RacingResults.cell RacingResults.name RacingResults.badge">
<meta name="prop:title" content="string" data-doc="The page's heading (the event's name); empty takes the kit's (Results).">
<meta name="prop:rows" content="list" data-doc="The finishing order as the game classifies it, keyed rows of { id, position, name, time, points, fastest, player, state }: time the race time or gap as the game words it; points as text (empty hides it); fastest marks the fastest lap; player the local player's row; state finished, dnf or dsq (dnf and dsq are named in words and show no place or time).">
<meta name="prop:bestLap" content="string" data-doc="The race's fastest lap time, formatted; empty hides the line under the table.">
<meta name="prop:bestBy" content="string" data-doc="Who set the fastest lap; empty leaves the name out of the line.">
<meta name="prop:canRetry" content="bool" data-doc="Retry is offered (false: disabled).">
<meta name="action:details" content="Retry">
<meta name="fixture:default" content='{"title":"","rows":[{"id":"marlow","position":1,"name":"Marlow","time":"24:31.207","points":"25","fastest":false,"player":false,"state":"finished"},{"id":"you","position":2,"name":"Rhea Vance","time":"+0.207","points":"18","fastest":true,"player":true,"state":"finished"},{"id":"okafor","position":3,"name":"Okafor","time":"+2.940","points":"15","fastest":false,"player":false,"state":"finished"},{"id":"sato","position":4,"name":"Sato","time":"+1 lap","points":"12","fastest":false,"player":false,"state":"finished"},{"id":"brandt","position":5,"name":"Brandt","time":"","points":"","fastest":false,"player":false,"state":"dnf"}],"bestLap":"1:20.871","bestBy":"Rhea Vance","canRetry":true}'>
<meta name="fixture:empty" content='{"title":"","rows":[],"bestLap":"","bestBy":"","canRetry":true}'>
<meta name="fixture:disabled" content='{"title":"Coastal Cup · Final","rows":[{"id":"you","position":1,"name":"Rhea Vance","time":"8:02.114","points":"","fastest":true,"player":true,"state":"finished"},{"id":"okafor","position":2,"name":"Okafor","time":"","points":"","fastest":false,"player":false,"state":"dsq"}],"bestLap":"","bestBy":"","canRetry":false}'>
<meta name="fixture:overflow" content='{"title":"Grand Touring Championship · Round 11 · Nordschleife Endurance","rows":[{"id":"r1","position":1,"name":"Maximilian Vandenberghe-Okonkwo","time":"2:01:31.207","points":"25","fastest":false,"player":false,"state":"finished"},{"id":"r2","position":2,"name":"Okafor","time":"+0.884","points":"18","fastest":false,"player":false,"state":"finished"},{"id":"r3","position":3,"name":"Lindqvist","time":"+1.020","points":"15","fastest":true,"player":false,"state":"finished"},{"id":"r4","position":4,"name":"Sato","time":"+2.310","points":"12","fastest":false,"player":false,"state":"finished"},{"id":"r5","position":5,"name":"Brandt","time":"+3.007","points":"10","fastest":false,"player":false,"state":"finished"},{"id":"r6","position":6,"name":"Duarte","time":"+4.551","points":"8","fastest":false,"player":false,"state":"finished"},{"id":"r7","position":7,"name":"Nakamura","time":"+5.902","points":"6","fastest":false,"player":false,"state":"finished"},{"id":"r8","position":8,"name":"Haddad","time":"+7.118","points":"4","fastest":false,"player":false,"state":"finished"},{"id":"r9","position":9,"name":"Rhea Vance","time":"+8.640","points":"2","fastest":false,"player":true,"state":"finished"},{"id":"r10","position":10,"name":"Moreau","time":"+9.215","points":"1","fastest":false,"player":false,"state":"finished"},{"id":"r11","position":11,"name":"Ferreira","time":"+1 lap","points":"","fastest":false,"player":false,"state":"finished"},{"id":"r12","position":12,"name":"Abernathy-Whitcombe","time":"+2 laps","points":"","fastest":false,"player":false,"state":"finished"},{"id":"r13","position":13,"name":"Petrov","time":"","points":"","fastest":false,"player":false,"state":"dnf"},{"id":"r14","position":14,"name":"Laurent","time":"","points":"","fastest":false,"player":false,"state":"dsq"}],"bestLap":"8:14.390","bestBy":"Lindqvist","canRetry":true}'>

<!--
  The classification after the flag: place, driver, time or gap, points
  and the fastest-lap mark per row, the player's row marked You, cars
  that did not finish or were disqualified named so. The game computes
  every time and point. Continue (continue) and Retry (retry; also
  the details action, X or pad Y) report once; a Retry the game refuses is disabled.
-->
<section id="RacingResults" part="RacingResults.frame" aria-label={title != '' ? title : t("racing.results.title")}>
  <header part="RacingResults.band"><h2 part="RacingResults.title">{title != '' ? title : t("racing.results.title")}</h2></header>
  {#if rows.length}
    <div part="RacingResults.heading" class="heading">
      <span class="place">{t("racing.results.position")}</span>
      <span class="head-name">{t("racing.results.driver")}</span>
      <span class="time">{t("racing.results.time")}</span>
      <span class="points">{t("racing.results.points")}</span>
    </div>
    <scroll class="list">
      <div class="rows" role="list" aria-label={title != '' ? title : t("racing.results.title")}>
        {#each rows as r (r.id)}
          <div part="RacingResults.row" role="listitem" aria-current={r.player} aria-label={r.name} data-player={r.player} data-state={r.state} data-fastest={r.fastest}>
            <span part="RacingResults.cell" class="place">{r.state == 'dnf' || r.state == 'dsq' ? t("racing.results.unplaced") : formatNumber(r.position)}</span>
            <div class="name"><span part="RacingResults.name" class="driver">{r.name}</span><span class="you" hidden={!r.player}>{t("racing.results.you")}</span><span part="RacingResults.badge" class="fastest" hidden={!r.fastest} aria-label={t("racing.results.fastest")}>{t("racing.results.fastest")}</span></div>
            <span class="time" hidden={r.state == 'dnf' || r.state == 'dsq'}>{r.time}</span>
            <span part="RacingResults.badge" class="out" data-state={r.state} hidden={!(r.state == 'dnf' || r.state == 'dsq')} aria-label={r.state == 'dsq' ? t("racing.results.dsqName") : t("racing.results.dnfName")}>{r.state == 'dsq' ? t("racing.results.dsq") : t("racing.results.dnf")}</span>
            <span class="points">{r.points}</span>
          </div>
        {/each}
      </div>
    </scroll>
    <p class="best" hidden={bestLap == ''}>{bestBy != '' ? t("racing.results.bestLap", { time: bestLap, name: bestBy }) : t("racing.results.bestLapTime", { time: bestLap })}</p>
  {:else}
    <p class="empty-text" role="status">{t("racing.results.empty")}</p>
  {/if}
  <footer class="actions">
    <button id="Retry" variant="ghost" disabled={!canRetry} onclick={retry} ondetails={retry}><kbd class="key" data-action="details"></kbd><span>{t("racing.results.retry")}</span></button>
    <button id="Continue" variant="primary" onclick={continue}><span>{t("racing.results.continue")}</span></button>
  </footer>
</section>

<style>
[part="RacingResults.frame"][hidden], [part="RacingResults.frame"] [hidden] { display: none; }
[part="RacingResults.frame"] { display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-sm); container-type: inline-size; }
[part="RacingResults.band"] { display: flex; width: stretch; align-items: center; flex: none; }
[part="RacingResults.title"] { margin: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
[part="RacingResults.heading"], [part="RacingResults.row"] { display: flex; width: stretch; align-items: center; gap: var(--space-md); padding: var(--space-xs) var(--space-md); }
[part="RacingResults.heading"] { flex: none; }
.list { width: stretch; flex: 1; min-height: 0; }
.rows { display: flex; flex-direction: column; width: stretch; gap: var(--space-xxs); }
[part="RacingResults.row"] { min-height: var(--size-touch); }
.place { flex: none; width: var(--size-md); text-align: center; font-variant-numeric: tabular-nums; }
.name { display: flex; flex: 1; min-width: 0; align-items: center; gap: var(--space-sm); }
.head-name { flex: 1; min-width: 0; text-align: start; }
.driver { min-width: 0; text-align: start; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.you, .fastest, .out { flex: none; white-space: nowrap; padding: 0 var(--space-xs); }
.time { flex: none; width: calc(var(--size-xxl) * 1.15); text-align: end; white-space: nowrap; font-variant-numeric: tabular-nums; }
.out { width: calc(var(--size-xxl) * 1.15); text-align: end; }
.points { flex: none; width: var(--size-lg); text-align: end; white-space: nowrap; font-variant-numeric: tabular-nums; }
.best { margin: 0; flex: none; }
.empty-text { margin: 0; flex: 1; text-align: center; }
.actions { display: flex; width: stretch; flex: none; justify-content: flex-end; gap: var(--space-sm); padding-bottom: env(safe-area-inset-bottom); }
.actions button { min-height: var(--size-touch); min-width: calc(var(--size-xxl) * 1.25); gap: var(--space-xs); }
@media (pointer: coarse) { .key { display: none; } }
@container (width < 600px) {
  [part="RacingResults.heading"], [part="RacingResults.row"] { gap: var(--space-xs); padding: var(--space-xxs) var(--space-xs); }
  .time, .out { width: calc(var(--size-xxl) * 0.9); }
  .points { width: var(--size-md); }
  .actions { justify-content: stretch; }
  .actions button { flex: 1; min-width: 0; }
}
</style>
