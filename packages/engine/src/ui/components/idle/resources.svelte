<meta name="component" content="IdleResources">
<meta name="strings" content="./idle.en.ogt">
<meta name="prop:currencies" content="list" data-doc="The currencies in the game's order, keyed rows of { id, icon, name, amount, value, rate, perSecond, full }: value the amount as the game formats it (it wins; empty shows amount, a number, compacted: 12.3K, counting to each new amount); rate the income as the game words it (+1.2K/s; it wins; empty shows perSecond, a number per second, compacted; 0 hides it); perSecond also sets data-trend (up, down, flat) whether or not rate is given, so pass it with a worded rate too; full the store is at its cap, named Full.">
<meta name="prop:loading" content="bool" data-doc="The amounts are on their way: each row keeps its icon and name and shows a placeholder for its figures.">
<meta name="fixture:default" content='{"currencies":[{"id":"gold","icon":"coin","name":"Gold","amount":1234567,"value":"","rate":"","perSecond":4820,"full":false},{"id":"gems","icon":"gem","name":"Gems","amount":320,"value":"320","rate":"","perSecond":0,"full":false},{"id":"wood","icon":"tree","name":"Wood","amount":5000,"value":"5,000 / 5,000","rate":"+12/s","perSecond":12,"full":true}],"loading":false}'>
<meta name="fixture:loading" content='{"currencies":[{"id":"gold","icon":"coin","name":"Gold","amount":0,"value":"","rate":"","perSecond":0,"full":false},{"id":"gems","icon":"gem","name":"Gems","amount":0,"value":"","rate":"","perSecond":0,"full":false}],"loading":true}'>
<meta name="fixture:full" content='{"currencies":[{"id":"gold","icon":"coin","name":"Gold","amount":1000000,"value":"1M / 1M","rate":"","perSecond":0,"full":true},{"id":"mana","icon":"drop","name":"Mana","amount":250,"value":"250 / 250","rate":"+3.5/s","perSecond":3.5,"full":true}],"loading":false}'>
<meta name="fixture:draining" content='{"currencies":[{"id":"food","icon":"apple","name":"Food","amount":842,"value":"","rate":"","perSecond":-36,"full":false},{"id":"energy","icon":"bolt","name":"Energy","amount":12.5,"value":"","rate":"","perSecond":0.25,"full":false}],"loading":false}'>
<meta name="fixture:empty" content='{"currencies":[],"loading":false}'>
<meta name="fixture:overflow" content='{"currencies":[{"id":"gold","icon":"coin","name":"Gold Sovereigns of the Eastern Treasury","amount":1.234e24,"value":"","rate":"","perSecond":5.6e21,"full":false},{"id":"gems","icon":"gem","name":"Gems","amount":99999999,"value":"","rate":"+1.25 quadrillion per second","perSecond":0,"full":false},{"id":"wood","icon":"tree","name":"Wood","amount":5000,"value":"5,000 / 5,000","rate":"","perSecond":0,"full":true},{"id":"stone","icon":"shield","name":"Stone","amount":48210,"value":"","rate":"","perSecond":120,"full":false},{"id":"mana","icon":"drop","name":"Mana","amount":250,"value":"","rate":"","perSecond":3.5,"full":false},{"id":"souls","icon":"skull","name":"Souls","amount":7,"value":"","rate":"","perSecond":0,"full":false},{"id":"stars","icon":"star","name":"Stardust","amount":1.5e9,"value":"","rate":"","perSecond":2.2e6,"full":false},{"id":"crowns","icon":"crown","name":"Crowns","amount":3,"value":"","rate":"","perSecond":0,"full":false}],"loading":false}'>

<!--
  The resource bar of an idle game: one cut chip per currency with its art
  in a socket, its name, the amount large and the income under it, green
  when it rises. A store at its cap gets a gold rim and a Full tag. The
  game owns every number: a formatted value or rate it passes wins, else
  the bare number is compacted (12.3K) and counts to each new amount.
  While loading each chip keeps its art and name and shows a placeholder,
  which a pack may animate; the row's name says it is loading. It reports
  no intents. The chips wrap onto more lines when they do not fit.
-->
<section id="IdleResources" part="frame" variant="bare" layout="row" aria-label={t("idle.resources.name")} aria-busy={loading}>
  {#if currencies.length}
    <ul class="chips" layout="wrap" gap="tight">
      {#each currencies as c (c.id)}
        <li class="row" part="row" layout="row" gap="tight" data-loading={loading} trend={c.perSecond < 0 ? 'down' : c.perSecond > 0 ? 'up' : 'flat'} aria-label={loading ? t("idle.resources.loading", { name: c.name }) : t("idle.resources.row", { name: c.name, amount: c.value ? c.value : formatCompact(c.amount) })} class:full={c.full} class:trend-down={(c.perSecond < 0 ? 'down' : c.perSecond > 0 ? 'up' : 'flat') == 'down'} class:trend-up={(c.perSecond < 0 ? 'down' : c.perSecond > 0 ? 'up' : 'flat') == 'up'}>
          <div part="socket" class="socket cover" layout="stack" fill><og-icon class="icon" icon={c.icon}></og-icon></div>
          <div class="text" layout="stack" gap="none">
            <span part="name" class="name">{c.name}</span>
            {#if !loading}
            <div class="figures" layout="row" gap="tight">
              {#if c.value}
                <span part="figure" class="figure">{c.value}</span>
              {:else}
                <og-counter part="figure" class="figure" value={c.amount} format="compact" duration="300"></og-counter>
              {/if}
              {#if c.full}<span part="tag" class="full">{t("idle.resources.full")}</span>{/if}
            </div>
            {#if c.rate || c.perSecond != 0}<span part="rate" class="rate" trend={c.perSecond < 0 ? 'down' : c.perSecond > 0 ? 'up' : 'flat'} class:trend-down={(c.perSecond < 0 ? 'down' : c.perSecond > 0 ? 'up' : 'flat') == 'down'} class:trend-up={(c.perSecond < 0 ? 'down' : c.perSecond > 0 ? 'up' : 'flat') == 'up'}>{c.rate ? c.rate : c.perSecond > 0 ? t("idle.resources.perSecond", { rate: formatCompact(c.perSecond) }) : t("idle.resources.perSecondDown", { rate: formatCompact(c.perSecond) })}</span>{/if}
            {:else}
            <div class="placeholder" id="Placeholder" part="placeholder"></div>
            {/if}
          </div>
        </li>
      {/each}
    </ul>
  {:else}
    <p class="empty-text" part="name" role="status">{t("idle.resources.empty")}</p>
  {/if}
</section>

<style>
.cover { justify-content: center; align-items: center; }
/* Drawn over the scene: the style's tag halo holds its text there. */
#IdleResources { text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
#IdleResources { inline-size: stretch; }
.chips { width: stretch; margin: 0; padding: 0; list-style: none; min-width: 0; }
/* A chip: cut at two corners; a full store rimmed in gold. */
.row { flex: 0 1 auto; align-items: center; padding: var(--space-xxs) var(--space-sm) var(--space-xxs) var(--space-xxs); min-height: var(--size-touch); min-width: 0; max-width: stretch; background: var(--play-chip-fill); box-shadow: var(--play-chip-rim); color: var(--ink-bright); border-radius: var(--inv-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.row.full { box-shadow: var(--play-full-rim); }
.socket { flex: none; inline-size: var(--size-lg); block-size: var(--size-lg); background: var(--inv-socket-fill); box-shadow: var(--inv-socket-rim); border-radius: var(--inv-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.icon { width: 72%; height: 72%; filter: var(--inv-slot-art-shadow); }
.text { min-width: 0; }
.name { white-space: nowrap; text-overflow: ellipsis; overflow: hidden; text-align: start; font-size: var(--hud-tag-size); font-weight: var(--hud-label-weight); color: var(--hud-label-ink); }
.figures { align-items: center; min-width: 0; }
.figure { white-space: nowrap; font-size: var(--play-figure-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--ink-bright); }
.full { flex: none; padding: 0 var(--space-xxs); white-space: nowrap; font-size: var(--hud-tag-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--play-boost-ink); background: var(--play-boost-fill); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.rate { white-space: nowrap; text-overflow: ellipsis; overflow: hidden; min-width: 0; font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-label-ink); }
.rate.trend-up { color: var(--play-gain-ink); }
.rate.trend-down { color: var(--play-low-ink); }
.placeholder { width: var(--size-xl); height: var(--space-sm); margin-block: var(--space-xxs); background: var(--play-well-fill); }
.empty-text { margin: 0; padding: var(--space-md); }
@container (width < 600px) {
  .row { flex: 1 1 0; padding-inline-end: var(--space-xs); }
  .socket { inline-size: var(--size-md); block-size: var(--size-md); }
}
</style>
