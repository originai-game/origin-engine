<meta name="component" content="StrategyQueue">
<meta name="strings" content="./strategy.en.ogt">
<meta name="parts" content="StrategyQueue.frame StrategyQueue.title StrategyQueue.slot StrategyQueue.art StrategyQueue.sweep StrategyQueue.time StrategyQueue.badge StrategyQueue.open StrategyQueue.count StrategyQueue.state StrategyQueue.reason">
<meta name="prop:title" content="string" data-doc="What produces, as the game words it (Barracks queue); empty takes the kit's (Production).">
<meta name="prop:items" content="list" data-doc="The queue in the game's order, first in production, keyed rows of { id, name, picture, icon, count, progress, timeText, state, reason }: name for a reader; picture a texture (og.load.texture), 0 draws icon from the atlas; count a stack of the same order (×2, shown above 1); progress 0 to 1 done, a dark sweep over the first order's art for what is still to run; timeText its time left as the game words it (0:08), on the first order's tile, empty none; state building, waiting, paused or blocked (data-state; paused and blocked are also words after the tiles); reason why it is blocked (Need 3 supply), empty none.">
<meta name="prop:queued" content="number" data-doc="Orders queued, as the game counts them (stacks included), shown over limit on a tag.">
<meta name="prop:limit" content="number" data-doc="The most the queue holds: its free places are drawn as empty tiles after the orders; 0 hides the count and the free places.">
<meta name="prop:full" content="bool" data-doc="The game says no more can be queued: the Queue full word on the count's tag and data-full on the frame.">
<meta name="fixture:default" content='{"title":"Barracks queue","queued":3,"limit":5,"full":false,"items":[{"id":"o1","name":"Man-at-Arms","picture":0,"icon":"sword","count":1,"progress":0.64,"timeText":"0:08","state":"building","reason":""},{"id":"o2","name":"Archer","picture":0,"icon":"bolt","count":1,"progress":0,"timeText":"","state":"waiting","reason":""},{"id":"o3","name":"Archer","picture":0,"icon":"bolt","count":1,"progress":0,"timeText":"","state":"waiting","reason":""}]}'>
<meta name="fixture:blocked" content='{"title":"","queued":4,"limit":5,"full":false,"items":[{"id":"o1","name":"Ballista","picture":0,"icon":"bomb","count":1,"progress":0,"timeText":"","state":"blocked","reason":"Need 3 supply"},{"id":"o2","name":"Archer","picture":0,"icon":"bolt","count":2,"progress":0,"timeText":"","state":"waiting","reason":""},{"id":"o3","name":"Spearman","picture":0,"icon":"sword","count":1,"progress":0,"timeText":"","state":"waiting","reason":""}]}'>
<meta name="fixture:empty" content='{"title":"","queued":0,"limit":5,"full":false,"items":[]}'>
<meta name="fixture:full" content='{"title":"","queued":5,"limit":5,"full":true,"items":[{"id":"o1","name":"Knight","picture":0,"icon":"shield","count":1,"progress":0.18,"timeText":"0:33","state":"building","reason":""},{"id":"o2","name":"Knight","picture":0,"icon":"shield","count":2,"progress":0,"timeText":"","state":"waiting","reason":""},{"id":"o3","name":"Healer","picture":0,"icon":"potion","count":2,"progress":0,"timeText":"","state":"waiting","reason":""}]}'>
<meta name="fixture:paused" content='{"title":"Barracks","queued":2,"limit":5,"full":false,"items":[{"id":"o1","name":"Pikeman","picture":0,"icon":"sword","count":1,"progress":0.4,"timeText":"0:11","state":"paused","reason":""},{"id":"o2","name":"Pikeman","picture":0,"icon":"sword","count":1,"progress":0,"timeText":"","state":"waiting","reason":""}]}'>
<meta name="fixture:overflow" content='{"title":"Royal Academy of Siegecraft and Applied Alchemy","queued":12,"limit":12,"full":true,"items":[{"id":"o1","name":"Trebuchet of the Burning Dawn, Masterwork Edition","picture":0,"icon":"bomb","count":1,"progress":0.91,"timeText":"10:00:00","state":"blocked","reason":"Requires the Engineers Guild and a free building plot near the wall"},{"id":"o2","name":"Ram","picture":0,"icon":"bomb","count":3,"progress":0,"timeText":"","state":"waiting","reason":""},{"id":"o3","name":"Siege Tower","picture":0,"icon":"home","count":1,"progress":0,"timeText":"","state":"waiting","reason":""},{"id":"o4","name":"Alchemist","picture":0,"icon":"potion","count":2,"progress":0,"timeText":"","state":"waiting","reason":""},{"id":"o5","name":"Mortar","picture":0,"icon":"fire","count":1,"progress":0,"timeText":"","state":"waiting","reason":""},{"id":"o6","name":"Sapper","picture":0,"icon":"gear","count":2,"progress":0,"timeText":"","state":"waiting","reason":""},{"id":"o7","name":"Bombard","picture":0,"icon":"bomb","count":2,"progress":0,"timeText":"","state":"waiting","reason":""}]}'>

<!--
  A production queue as one row: what produces in small capitals, the
  order in production as a larger tile with a dark sweep for what is still
  to run and its time left, the orders waiting as smaller tiles (a stack's
  count on a tag), the free places as empty tiles, the count over the
  limit, and Paused, Blocked with the reason, or Queue full as words.
  Reports cancel (a tile's key); the game owns costs, times, refunds and
  the limit.
-->
<section id="StrategyQueue" part="StrategyQueue.frame" class="hud bare" aria-label={title != '' ? title : t("strategy.queue.title")} data-full={full}>
  <span part="StrategyQueue.title" class="title">{title != '' ? title : t("strategy.queue.title")}</span>
  <div class="slots" role="group" aria-label={t("strategy.queue.items")}>
    {#each items as q (q.id)}
      <button part="StrategyQueue.slot" class="slot" aria-label={t("strategy.queue.cancel", { name: q.name })} aria-description={q.reason != '' ? q.reason : q.timeText} data-state={q.state} onclick={cancel}><div class="face">
        {#if q.picture}<img part="StrategyQueue.art" class="art" src={q.picture} alt="">{:else}<div part="StrategyQueue.art" class="blank-art"><og-icon class="mark" icon={q.icon}></og-icon></div>{/if}
        {#if q.state != 'waiting'}<div part="StrategyQueue.sweep" class="sweep" style:--done={q.progress <= 0 ? 0 : q.progress >= 1 ? 1 : q.progress}></div>{/if}
        {#if q.state != 'waiting' && q.timeText != ''}<div class="run" style:--chars={q.timeText.length}><span part="StrategyQueue.time" class="time">{q.timeText}</span></div>{/if}
        {#if q.count > 1}<span part="StrategyQueue.badge" class="stack">{t("strategy.queue.stack", { count: q.count })}</span>{/if}
      </div></button>
    {/each}
    {#if limit > queued}<div part="StrategyQueue.open" class="open" style:--open={limit - queued > 12 ? 12 : limit - queued}></div>{/if}
  </div>
  {#if limit > 0}<span part="StrategyQueue.count" class="capacity" data-full={full} aria-label={t("strategy.queue.capacityName", { count: queued, limit: limit })}>{full ? t("strategy.queue.fullCount", { count: queued, limit: limit }) : t("strategy.queue.capacity", { count: queued, limit: limit })}</span>{:else}{#if full}<span part="StrategyQueue.count" class="capacity" data-full="true">{t("strategy.queue.full")}</span>{/if}{/if}
  {#each items as q (q.id)}
    {#if q.state == 'paused' || q.state == 'blocked'}
      <span part="StrategyQueue.state" class="state" data-state={q.state} role="status">{q.state == 'paused' ? t("strategy.queue.paused") : t("strategy.queue.blocked")}</span>
      {#if q.reason != ''}<span part="StrategyQueue.reason" class="reason">{q.reason}</span>{/if}
    {/if}
  {/each}
  {#if !items.length}<span class="empty-text" role="status">{t("strategy.queue.empty")}</span>{/if}
</section>

<style>
:host { --first: calc(var(--size-md) * 1.1); --tile: calc(var(--size-md) * 0.84); --gap: calc(6 * var(--font-step-px)); display: flex; flex-wrap: wrap; align-items: center; column-gap: calc(10 * var(--font-step-px)); row-gap: calc(4 * var(--font-step-px)); inline-size: stretch; min-inline-size: 0; color: var(--hud-ink); }
.title { flex: 0 1 auto; min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.slots { display: flex; align-items: center; column-gap: var(--gap); flex: 0 1 auto; min-inline-size: 0; overflow: hidden; }
/* A tile: its fill is its lit edge, the art inset; the first order's tile
   is larger. The tiles are art and keep their size at any text scale. */
.slot { flex: none; inline-size: var(--tile); block-size: var(--tile); min-inline-size: 0; color: var(--hud-ink); }
.slot:first-child { inline-size: var(--first); block-size: var(--first); }
.slot::part(base) { padding: calc(2 * var(--font-step-px)); border: 0; border-radius: 0; background: var(--hud-edge); box-shadow: none; clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
.slot[data-state=blocked]::part(base) { background: var(--hud-harm); }
.face { container-type: size; position: relative; inline-size: stretch; block-size: stretch; overflow: hidden; clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
.art { position: absolute; left: 0; top: 0; inline-size: 100%; block-size: 100%; object-fit: cover; }
.blank-art { position: absolute; left: 0; top: 0; inline-size: 100%; block-size: 100%; display: flex; align-items: center; justify-content: center; background: var(--hud-plate-fill); }
.mark { inline-size: 60%; block-size: 60%; color: var(--hud-glyph); }
.slot[data-state=blocked] .art { filter: var(--hud-unready); }
/* What is still to run: a dark sweep turning clockwise from the top. */
.sweep { position: absolute; left: 0; top: 0; inline-size: 100%; block-size: 100%; background: var(--hud-sweep); mask-image: conic-gradient(transparent calc(var(--done) * 360deg), #000 0); }
/* The time on the tile: the cooldown's size, a share of its container,
   here a box narrowed by the figure's length (four characters take the
   tile), so 0:08 and 10:00:00 both stay whole on it. */
.run { container-type: inline-size; position: absolute; left: 50%; top: 0; bottom: 0; inline-size: min(100%, calc(400% / var(--chars))); translate: -50% 0; display: flex; align-items: center; justify-content: center; }
.time { flex: none; font-size: var(--hud-size-cooldown); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
.stack { position: absolute; right: 0; top: 0; padding: 0 calc(2 * var(--font-step-px)); font-size: var(--hud-size-label-compact); line-height: 1.2; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); white-space: nowrap; background: var(--hud-tag-fill); }
/* The free places: empty tiles, one per place, drawn as one strip. */
.open { flex: none; inline-size: calc(var(--open) * (var(--tile) + var(--gap)) - var(--gap)); block-size: var(--tile); background: var(--hud-slot-empty); mask-image: repeating-linear-gradient(90deg, #000 0 var(--tile), transparent var(--tile) calc(var(--tile) + var(--gap))); }
.capacity, .state { flex: none; padding: 0 calc(6 * var(--font-step-px)); font-size: var(--hud-tag-size); line-height: 1.6; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
.capacity[data-full=true] { color: var(--hud-gold); }
.state[data-state=paused] { color: var(--hud-gold); }
.state[data-state=blocked] { color: var(--hud-harm); }
.reason { flex: 0 1 auto; min-inline-size: 0; font-size: var(--hud-size-xs); font-weight: var(--hud-label-weight); color: var(--hud-harm); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.empty-text { flex: 0 1 auto; min-inline-size: 0; font-size: var(--hud-size-xs); color: var(--hud-ink-dim); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
/* On a phone the row is compact: smaller tiles, the title short. */
@media (width < 600px) {
  :host { --first: var(--size-md); --tile: calc(var(--size-md) * 0.78); --gap: calc(4 * var(--font-step-px)); column-gap: calc(6 * var(--font-step-px)); }
  .title { font-size: var(--hud-size-label-compact); letter-spacing: normal; }
}
</style>
