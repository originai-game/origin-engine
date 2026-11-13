<meta name="component" content="IdleResources">
<meta name="strings" content="./idle.en.ogt">
<meta name="parts" content="IdleResources.frame IdleResources.row IdleResources.name IdleResources.figure IdleResources.rate IdleResources.badge IdleResources.placeholder">
<meta name="prop:currencies" content="list" data-doc="The currencies in the game's order, keyed rows of { id, icon, name, amount, value, rate, perSecond, full }: value the amount as the game formats it (it wins; empty shows amount, a number, compacted: 12.3K); rate the income as the game words it (+1.2K/s; it wins; empty shows perSecond, a number per second, compacted; 0 hides it); perSecond also sets data-trend (up, down, flat) whether or not rate is given, so pass it with a worded rate too; full the store is at its cap, named Full.">
<meta name="prop:loading" content="bool" data-doc="The amounts are on their way: each row keeps its icon and name and shows a placeholder for its figures.">
<meta name="fixture:default" content='{"currencies":[{"id":"gold","icon":"coin","name":"Gold","amount":1234567,"value":"","rate":"","perSecond":4820,"full":false},{"id":"gems","icon":"gem","name":"Gems","amount":320,"value":"320","rate":"","perSecond":0,"full":false},{"id":"wood","icon":"leaf","name":"Wood","amount":5000,"value":"5,000 / 5,000","rate":"+12/s","perSecond":12,"full":true}],"loading":false}'>
<meta name="fixture:loading" content='{"currencies":[{"id":"gold","icon":"coin","name":"Gold","amount":0,"value":"","rate":"","perSecond":0,"full":false},{"id":"gems","icon":"gem","name":"Gems","amount":0,"value":"","rate":"","perSecond":0,"full":false}],"loading":true}'>
<meta name="fixture:full" content='{"currencies":[{"id":"gold","icon":"coin","name":"Gold","amount":1000000,"value":"1M / 1M","rate":"","perSecond":0,"full":true},{"id":"mana","icon":"drop","name":"Mana","amount":250,"value":"250 / 250","rate":"+3.5/s","perSecond":3.5,"full":true}],"loading":false}'>
<meta name="fixture:draining" content='{"currencies":[{"id":"food","icon":"apple","name":"Food","amount":842,"value":"","rate":"","perSecond":-36,"full":false},{"id":"energy","icon":"bolt","name":"Energy","amount":12.5,"value":"","rate":"","perSecond":0.25,"full":false}],"loading":false}'>
<meta name="fixture:empty" content='{"currencies":[],"loading":false}'>
<meta name="fixture:overflow" content='{"currencies":[{"id":"gold","icon":"coin","name":"Gold Sovereigns of the Eastern Treasury","amount":1.234e24,"value":"","rate":"","perSecond":5.6e21,"full":false},{"id":"gems","icon":"gem","name":"Gems","amount":99999999,"value":"","rate":"+1.25 quadrillion per second","perSecond":0,"full":false},{"id":"wood","icon":"leaf","name":"Wood","amount":5000,"value":"5,000 / 5,000","rate":"","perSecond":0,"full":true},{"id":"stone","icon":"shield","name":"Stone","amount":48210,"value":"","rate":"","perSecond":120,"full":false},{"id":"mana","icon":"drop","name":"Mana","amount":250,"value":"","rate":"","perSecond":3.5,"full":false},{"id":"souls","icon":"skull","name":"Souls","amount":7,"value":"","rate":"","perSecond":0,"full":false},{"id":"stars","icon":"star","name":"Stardust","amount":1.5e9,"value":"","rate":"","perSecond":2.2e6,"full":false},{"id":"crowns","icon":"crown","name":"Crowns","amount":3,"value":"","rate":"","perSecond":0,"full":false}],"loading":false}'>

<!--
  The resource bar of an idle game: one chip per currency with its icon,
  name, amount and income, and a Full badge when its store is at the cap.
  The game owns every number: a formatted value or rate it passes wins,
  else the bare number is compacted (12.3K). While loading each chip keeps
  its icon and name and shows a placeholder, which a pack may animate; the
  row's name says it is loading. It reports no intents. The
  chips wrap onto more lines when they do not fit.
-->
<section id="IdleResources" part="IdleResources.frame" class="bare" aria-label={t("idle.resources.name")} aria-busy={loading}>
  {#if currencies.length}
    <ul class="chips">
      {#each currencies as c (c.id)}
        <li part="IdleResources.row" data-full={c.full} data-loading={loading} data-trend={c.perSecond < 0 ? 'down' : c.perSecond > 0 ? 'up' : 'flat'} aria-label={loading ? t("idle.resources.loading", { name: c.name }) : t("idle.resources.row", { name: c.name, amount: c.value != '' ? c.value : formatCompact(c.amount) })}>
          <og-icon class="icon" icon={c.icon}></og-icon>
          <div class="text">
            <span part="IdleResources.name" class="name">{c.name}</span>
            <div class="figures" hidden={loading}>
              <span part="IdleResources.figure" class="figure">{c.value != '' ? c.value : formatCompact(c.amount)}</span>
              <span part="IdleResources.rate" data-trend={c.perSecond < 0 ? 'down' : c.perSecond > 0 ? 'up' : 'flat'} hidden={c.rate == '' && c.perSecond == 0}>{c.rate != '' ? c.rate : c.perSecond > 0 ? t("idle.resources.perSecond", { rate: formatCompact(c.perSecond) }) : t("idle.resources.perSecondDown", { rate: formatCompact(c.perSecond) })}</span>
              <span part="IdleResources.badge" hidden={!c.full}>{t("idle.resources.full")}</span>
            </div>
            <div id="Placeholder" part="IdleResources.placeholder" hidden={!loading}></div>
          </div>
        </li>
      {/each}
    </ul>
  {:else}
    <p class="empty-text" role="status">{t("idle.resources.empty")}</p>
  {/if}
</section>

<style>
[part="IdleResources.frame"][hidden], [part="IdleResources.frame"] [hidden] { display: none; }
[part="IdleResources.frame"] { display: flex; width: stretch; }
.chips { display: flex; flex-wrap: wrap; width: stretch; margin: 0; padding: 0; list-style: none; gap: var(--space-xs); }
[part="IdleResources.row"] { display: flex; align-items: center; gap: var(--space-xs); padding: var(--space-xxs) var(--space-sm); min-height: var(--size-touch); min-width: 0; max-width: stretch; }
.icon { width: var(--size-sm); height: var(--size-sm); flex: none; }
.text { display: flex; flex-direction: column; min-width: 0; gap: 0; }
[part="IdleResources.name"] { white-space: nowrap; text-overflow: ellipsis; overflow: hidden; text-align: start; }
.figures { display: flex; align-items: baseline; flex-wrap: wrap; gap: 0 var(--space-xs); min-width: 0; }
[part="IdleResources.figure"] { font-variant-numeric: tabular-nums; white-space: nowrap; }
[part="IdleResources.rate"] { font-variant-numeric: tabular-nums; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; min-width: 0; }
[part="IdleResources.badge"] { white-space: nowrap; padding: 0 var(--space-xxs); }
[part="IdleResources.placeholder"] { width: var(--size-xl); height: var(--space-sm); margin-block: var(--space-xxs); border-radius: var(--radius-xs); background: var(--control-strong); }
.empty-text { margin: 0; }
@media (width < 600px) {
  [part="IdleResources.row"] { padding: var(--space-xxs) var(--space-xs); }
}
</style>
