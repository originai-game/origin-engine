<meta name="component" content="StrategyQueue">
<meta name="strings" content="./strategy.en.ogt">
<meta name="parts" content="StrategyQueue.frame StrategyQueue.band StrategyQueue.title StrategyQueue.badge StrategyQueue.row StrategyQueue.cell StrategyQueue.name StrategyQueue.reason">
<meta name="prop:title" content="string" data-doc="The queue's heading; empty takes the kit's (Production).">
<meta name="prop:items" content="list" data-doc="The queue in the game's order, first in production, keyed rows of { id, name, icon, count, progress, timeText, state, reason }: count a stack of the same order (shown above 1); progress 0 to 1 done; timeText the time left as the game words it (0:12), empty hides it; state building, waiting, paused or blocked (data-state, and a word for every state but building); reason why it is blocked (Need 2 supply), empty none.">
<meta name="prop:queued" content="number" data-doc="Orders queued, as the game counts them (stacks included), shown over limit.">
<meta name="prop:limit" content="number" data-doc="The most the queue holds; 0 hides the count.">
<meta name="prop:full" content="bool" data-doc="The game says no more can be queued: the Queue full badge and data-full on the frame.">
<meta name="fixture:default" content='{"title":"","queued":4,"limit":5,"full":false,"items":[{"id":"o1","name":"Spearman","icon":"sword","count":1,"progress":0.62,"timeText":"0:07","state":"building","reason":""},{"id":"o2","name":"Archer","icon":"bolt","count":2,"progress":0,"timeText":"0:24","state":"waiting","reason":""},{"id":"o3","name":"Ballista","icon":"bomb","count":1,"progress":0,"timeText":"","state":"blocked","reason":"Need 3 supply"}]}'>
<meta name="fixture:empty" content='{"title":"","queued":0,"limit":5,"full":false,"items":[]}'>
<meta name="fixture:full" content='{"title":"","queued":5,"limit":5,"full":true,"items":[{"id":"o1","name":"Knight","icon":"shield","count":1,"progress":0.18,"timeText":"0:33","state":"building","reason":""},{"id":"o2","name":"Knight","icon":"shield","count":2,"progress":0,"timeText":"1:20","state":"waiting","reason":""},{"id":"o3","name":"Healer","icon":"potion","count":2,"progress":0,"timeText":"0:50","state":"waiting","reason":""}]}'>
<meta name="fixture:paused" content='{"title":"Barracks","queued":2,"limit":5,"full":false,"items":[{"id":"o1","name":"Pikeman","icon":"sword","count":1,"progress":0.4,"timeText":"0:11","state":"paused","reason":""},{"id":"o2","name":"Pikeman","icon":"sword","count":1,"progress":0,"timeText":"0:18","state":"waiting","reason":""}]}'>
<meta name="fixture:overflow" content='{"title":"Royal Academy of Siegecraft and Applied Alchemy","queued":12,"limit":12,"full":true,"items":[{"id":"o1","name":"Trebuchet of the Burning Dawn, Masterwork Edition","icon":"bomb","count":1,"progress":0.91,"timeText":"0:02","state":"building","reason":""},{"id":"o2","name":"Ram","icon":"bomb","count":3,"progress":0,"timeText":"2:30","state":"waiting","reason":""},{"id":"o3","name":"Siege Tower","icon":"home","count":1,"progress":0,"timeText":"3:45","state":"blocked","reason":"Requires the Engineers Guild and a free building plot near the wall"},{"id":"o4","name":"Alchemist","icon":"potion","count":2,"progress":0,"timeText":"1:10","state":"waiting","reason":""},{"id":"o5","name":"Mortar","icon":"fire","count":1,"progress":0,"timeText":"1:55","state":"waiting","reason":""},{"id":"o6","name":"Sapper","icon":"gear","count":2,"progress":0,"timeText":"0:40","state":"waiting","reason":""},{"id":"o7","name":"Bombard","icon":"bomb","count":2,"progress":0,"timeText":"10:00:00","state":"paused","reason":""}]}'>

<!--
  A production queue: each order with its picture, a stack count, its
  progress (ProgressBar) and time left, a word for a state other than
  building, the reason when blocked, and a cancel button. The heading
  carries count over limit and, when the game says so, Queue full. Reports
  cancel (the order's key); the game owns costs, times, refunds and the
  limit.
-->
<section id="StrategyQueue" part="StrategyQueue.frame" class="hud" aria-label={title != '' ? title : t("strategy.queue.title")} data-full={full}>
  <header part="StrategyQueue.band">
    <h2 part="StrategyQueue.title">{title != '' ? title : t("strategy.queue.title")}</h2>
    <span part="StrategyQueue.badge" class="capacity" data-full={full} hidden={!(limit > 0)} aria-label={t("strategy.queue.capacityName", { count: queued, limit: limit })}>{t("strategy.queue.capacity", { count: queued, limit: limit })}</span>
    <span part="StrategyQueue.badge" class="full" hidden={!full} role="status">{t("strategy.queue.full")}</span>
  </header>
  {#if items.length}
    <div class="items" role="list" aria-label={t("strategy.queue.items")}>
      {#each items as q (q.id)}
        <div part="StrategyQueue.row" role="listitem" aria-label={q.name} data-state={q.state}>
          <div part="StrategyQueue.cell" class="picture"><og-icon class="mark" icon={q.icon}></og-icon><span part="StrategyQueue.badge" class="stack" hidden={!(q.count > 1)}>{t("strategy.queue.stack", { count: q.count })}</span></div>
          <div class="main">
            <div class="line">
              <span part="StrategyQueue.name" class="order-name">{q.name}</span>
              <span part="StrategyQueue.badge" class="state" data-state={q.state} hidden={q.state == 'building'}>{q.state == 'paused' ? t("strategy.queue.paused") : (q.state == 'blocked' ? t("strategy.queue.blocked") : t("strategy.queue.waiting"))}</span>
              <span class="time" hidden={q.timeText == ''}>{q.timeText}</span>
            </div>
            <div class="bar"><ProgressBar value={q.progress} buffer={0} label="" valueText={q.timeText} showValue={false} indeterminate={false} tone={q.state == 'paused' ? 'warn' : (q.state == 'blocked' ? 'danger' : '')} /></div>
            <span part="StrategyQueue.reason" class="reason" hidden={q.reason == ''}>{q.reason}</span>
          </div>
          <button class="cancel" variant="ghost" aria-label={t("strategy.queue.cancel", { name: q.name })} onclick={cancel}><og-icon class="cancel-mark" icon="close"></og-icon></button>
        </div>
      {/each}
    </div>
  {:else}
    <p class="empty-text" role="status">{t("strategy.queue.empty")}</p>
  {/if}
</section>

<style>
[part="StrategyQueue.frame"][hidden], [part="StrategyQueue.frame"] [hidden] { display: none; }
[part="StrategyQueue.frame"] { display: flex; flex-direction: column; width: stretch; min-width: 0; gap: var(--space-xs); }
[part="StrategyQueue.band"] { display: flex; width: stretch; align-items: center; gap: var(--space-sm); flex: none; }
[part="StrategyQueue.title"] { margin: 0; flex: 1; min-width: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.capacity, .full { flex: none; white-space: nowrap; padding: 0 var(--space-xs); font-variant-numeric: tabular-nums; }
.items { display: flex; flex-direction: column; width: stretch; gap: var(--space-xs); }
[part="StrategyQueue.row"] { display: flex; width: stretch; align-items: center; gap: var(--space-sm); padding: var(--space-xxs) var(--space-xs); }
[part="StrategyQueue.cell"] { position: relative; flex: none; width: var(--size-lg); height: var(--size-lg); align-items: center; justify-content: center; }
.mark { width: 70%; height: 70%; }
.stack { position: absolute; inset-inline-end: 0; bottom: 0; padding: 0 var(--space-xxs); white-space: nowrap; font-variant-numeric: tabular-nums; }
.main { display: flex; flex-direction: column; flex: 1; min-width: 0; overflow: hidden; gap: var(--space-xxs); }
.line { display: flex; width: stretch; align-items: baseline; gap: var(--space-xs); }
.order-name { flex: 1; min-width: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.state { flex: none; white-space: nowrap; padding: 0 var(--space-xs); }
.time { flex: none; white-space: nowrap; font-variant-numeric: tabular-nums; }
.bar { width: stretch; }
/* A reason wraps: it is the one line that says why, so it is never cut. */
.reason { width: stretch; min-width: 0; white-space: normal; }
.cancel { flex: none; width: var(--size-touch); height: var(--size-touch); padding: 0; }
.cancel-mark { width: var(--size-xs); height: var(--size-xs); }
.empty-text { margin: 0; padding: var(--space-sm) 0; }
</style>
