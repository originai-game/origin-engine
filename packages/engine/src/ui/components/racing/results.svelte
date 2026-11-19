<meta name="component" content="RacingResults">
<meta name="strings" content="./racing.en.ogt">
<meta name="parts" content="RacingResults.frame RacingResults.band RacingResults.title RacingResults.heading RacingResults.row RacingResults.place RacingResults.name RacingResults.badge RacingResults.label RacingResults.figure RacingResults.caption">
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
<section id="RacingResults" part="RacingResults.frame" class="bare" data-layout="shell" aria-label={title != '' ? title : t("racing.results.title")}>
  <header part="RacingResults.band" data-layout="row"><h2 part="RacingResults.title" data-fill>{title != '' ? title : t("racing.results.title")}</h2></header>
  {#if rows.length}
    <div class="board" data-fill>
    <div class="hero" role="group" aria-label={t("racing.results.you")}>
      {#each rows as r (r.id)}{#if r.player}
        <p class="mine-place">{r.state == 'dnf' || r.state == 'dsq' ? (r.state == 'dsq' ? t("racing.results.dsq") : t("racing.results.dnf")) : formatOrdinal(r.position)}</p>
        <div class="mine" data-layout="row"><span class="mine-label">{t("racing.results.time")}</span><span class="mine-figure">{r.time}</span></div>
        {#if r.points != ''}<div class="mine" data-layout="row"><span class="mine-label">{t("racing.results.points")}</span><span class="mine-figure gold">{r.points}</span></div>{/if}
      {/if}{/each}
    </div>
    <div class="tower" data-layout="shell">
    <div part="RacingResults.heading" class="heading" data-layout="row" data-gap="loose">
      <span class="place">{t("racing.results.position")}</span>
      <span class="head-name" data-fill>{t("racing.results.driver")}</span>
      <span class="time">{t("racing.results.time")}</span>
      <span class="points">{t("racing.results.points")}</span>
    </div>
    <div class="list" data-layout="scroll" data-fill>
      <div class="rows" data-layout="stack" data-gap="hair" role="list" aria-label={title != '' ? title : t("racing.results.title")}>
        {#each rows as r (r.id)}
          <div part="RacingResults.row" data-layout="row" data-gap="loose" role="listitem" aria-current={r.player} aria-label={r.name} data-player={r.player} data-state={r.state} data-fastest={r.fastest}>
            <span part="RacingResults.place RacingResults.label RacingResults.figure" class="place">{r.state == 'dnf' || r.state == 'dsq' ? t("racing.results.unplaced") : formatNumber(r.position)}</span>
            <div class="name" data-layout="cluster" data-gap="tight" data-fill><span part="RacingResults.name" class="driver">{r.name}</span>{#if r.player}<span class="you" part="RacingResults.caption">{t("racing.results.you")}</span>{/if}{#if r.fastest}<span part="RacingResults.badge" class="fastest" aria-label={t("racing.results.fastest")}>{t("racing.results.fastest")}</span>{/if}</div>
            {#if r.state == 'dnf' || r.state == 'dsq'}<span part="RacingResults.badge" class="out" data-state={r.state} aria-label={r.state == 'dsq' ? t("racing.results.dsqName") : t("racing.results.dnfName")}>{r.state == 'dsq' ? t("racing.results.dsq") : t("racing.results.dnf")}</span>
            {:else}<span class="time" part="RacingResults.figure">{r.time}</span>{/if}
            <span class="points" part="RacingResults.figure">{r.points}</span>
          </div>
        {/each}
      </div>
    </div>
    {#if bestLap != ''}<p class="best" part="RacingResults.caption">{bestBy != '' ? t("racing.results.bestLap", { time: bestLap, name: bestBy }) : t("racing.results.bestLapTime", { time: bestLap })}</p>{/if}
    </div>
    </div>
  {:else}
    <p class="empty-text" part="RacingResults.caption" data-fill role="status">{t("racing.results.empty")}</p>
  {/if}
  <footer class="actions" data-layout="row">
    <button id="Retry" variant="ghost" disabled={!canRetry} aria-label={t("racing.results.retry")} onclick={retry} ondetails={retry}><kbd class="key" data-action="details"></kbd><span>{t("racing.results.retry")}</span></button>
    <button id="Continue" variant="primary" onclick={continue}><span>{t("racing.results.continue")}</span></button>
  </footer>
</section>

<style>
/* The classification after the flag, as the racing HUD's tower: the
   player's place set huge in the HUD's accent beside their time and
   points on cut plates, and the order on cut dark glass, the leader's
   place on gold, the player's row lit in the accent, a car out named on
   a tag. Bare: the plates hold over the race. */
:host { inline-size: stretch; min-inline-size: 0; block-size: stretch; color: var(--hud-ink); }
[part="RacingResults.band"] { width: stretch; flex: none; border: 0; padding: 0; min-height: 0; background: none; box-shadow: none; }
[part="RacingResults.title"] { margin: 0; font-size: var(--result-size-title); line-height: 1.05; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-title-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); overflow-wrap: anywhere; }
/* The board scrolls in its share of the frame: a short frame (a phone at a
   large text scale) never pushes the hero or the order under the actions. */
.board { display: grid; grid-template-columns: minmax(0, 0.8fr) minmax(0, 1.4fr); column-gap: var(--space-xxl); row-gap: var(--space-md); align-items: start; min-height: 0; width: stretch; overflow-y: auto; }
.hero { display: flex; flex-direction: column; gap: var(--space-sm); min-width: 0; }
.hero:empty { display: none; }
.mine-place { margin: 0; font-size: var(--hud-size-place); line-height: 0.9; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-style: var(--judge-slant); color: var(--hud-accent); filter: var(--result-grade-glow); white-space: nowrap; }
.mine { justify-content: space-between; align-items: baseline; column-gap: var(--space-md); flex-wrap: wrap; padding: var(--space-sm) var(--space-md); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); }
.mine-label, [part="RacingResults.heading"] { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); }
.mine-figure { font-size: var(--hud-size-lg); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.gold { color: var(--hud-gold); }
.tower { min-height: 0; min-width: 0; max-height: stretch; }
[part="RacingResults.heading"], [part="RacingResults.row"] { width: stretch; padding: var(--space-xxs) var(--space-md) var(--space-xxs) 0; }
[part="RacingResults.heading"] { flex: none; border: 0; }
.rows { width: stretch; }
[part="RacingResults.row"] { min-height: var(--size-touch); align-items: center; border-radius: 0; color: var(--hud-ink); background: var(--hud-feed-fill-start); clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
[part="RacingResults.row"]:hover { background: var(--hud-feed-fill-start); }
.place { flex: none; width: calc(32 * var(--font-step-px)); text-align: center; }
[part="RacingResults.row"] .place { align-self: stretch; display: block; line-height: var(--size-touch); font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); letter-spacing: 0; color: var(--hud-ink); background: var(--hud-place-fill); }
[part="RacingResults.row"]:first-child[data-state=finished] .place { color: var(--result-medal-ink); background: var(--result-gold); }
.head-name { text-align: start; }
.driver { flex: 1 0 100%; text-align: start; overflow-wrap: anywhere; font-size: var(--hud-size-name); font-weight: var(--hud-label-weight); }
.you, .fastest, .out { flex: none; white-space: nowrap; padding: 0 var(--space-xs); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); border-radius: 0; clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
.you { color: var(--hud-ink); }
.fastest { color: var(--hud-best-ink); }
.out { color: var(--hud-harm); }
[part="RacingResults.row"][data-state=dnf], [part="RacingResults.row"][data-state=dsq] { opacity: 0.6; }
.time { flex: none; width: calc(var(--size-xxl) * 1.15); text-align: end; white-space: nowrap; }
[part="RacingResults.row"] .time, [part="RacingResults.row"] .points { font-size: var(--hud-size-time); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); }
.out { width: auto; margin-inline-start: auto; text-align: end; }
.points { flex: none; width: calc(48 * var(--font-step-px)); text-align: end; white-space: nowrap; }
[part="RacingResults.row"] .points { color: var(--hud-gold); }
/* The player's row, lit in the accent with ink to match. */
[part="RacingResults.row"][data-player=true] { color: var(--hud-accent-ink); background: var(--hud-player-fill); box-shadow: none; }
[part="RacingResults.row"][data-player=true] .place { color: var(--hud-accent-ink); background: var(--hud-place-player-fill); }
[part="RacingResults.row"][data-player=true] .points { color: var(--hud-accent-ink); }
[part="RacingResults.row"][data-player=true] .you, [part="RacingResults.row"][data-player=true] .fastest { color: var(--hud-accent-ink); background: var(--hud-place-player-fill); box-shadow: none; }
.best { margin: 0; flex: none; padding-block-start: var(--space-sm); letter-spacing: 0; text-transform: none; }
.empty-text { margin: 0; text-align: center; }
.actions { width: stretch; flex: none; justify-content: flex-end; padding-bottom: env(safe-area-inset-bottom); }
.actions button { min-height: var(--size-touch); min-width: calc(var(--size-xxl) * 1.25); }
/* A phone has no X key: its key caps go. */
@media (pointer: coarse) { .key { display: none; } }
@container (width < 760px) {
  .board { grid-template-columns: minmax(0, 1fr); }
  .hero { flex-direction: row; flex-wrap: wrap; align-items: center; }
  .mine-place { font-size: var(--hud-size-place-compact); }
  .mine { flex: 1 1 calc(110 * var(--font-step-px)); flex-direction: column; align-items: flex-start; }
  .mine-figure { font-size: var(--hud-size-md); }
}
@container (width < 600px) {
  [part="RacingResults.title"] { font-size: var(--result-size-title-compact); }
  [part="RacingResults.heading"], [part="RacingResults.row"] { --layout-gap-loose: var(--layout-gap-tight); padding: var(--space-xxs) var(--space-xs) var(--space-xxs) 0; }
  [part="RacingResults.row"] .time, [part="RacingResults.row"] .points, .driver { font-size: var(--hud-size-xs); }
  /* The captions in their columns' widths at any text scale: tight tracking, the tag size. */
  [part="RacingResults.heading"] { font-size: var(--hud-tag-size); letter-spacing: var(--result-tab-tracking-compact); }
  .time { width: calc(var(--size-xxl) * 0.9); }
  .points { width: calc(36 * var(--font-step-px)); }
  .actions { justify-content: stretch; }
  .actions button { flex: 1; min-width: 0; }
}
</style>
