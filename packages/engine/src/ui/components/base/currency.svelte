<meta name="component" content="Currency">
<meta name="strings" content="./currency.en.ogt">
<meta name="prop:name" content="string" data-doc="Resource name; empty uses Currency.">
<meta name="prop:icon" content="string" data-doc="Atlas icon; empty omits it.">
<meta name="prop:amount" content="number" data-doc="Amount held; zero remains visible. Ignored when valueText is supplied.">
<meta name="prop:valueText" content="string" data-doc="Game-formatted amount; empty uses the compact amount.">
<meta name="prop:limit" content="string" data-doc="Game-formatted capacity; empty omits it. The game decides whether the store is full.">
<meta name="prop:rate" content="string" data-doc="Optional income or drain, including its sign and unit.">
<meta name="prop:full" content="bool" data-doc="At capacity; reads Full in words.">
<meta name="prop:loading" content="bool" data-doc="Awaiting the amount; hides stale figures and reads Loading.">
<meta name="prop:unavailable" content="bool" data-doc="The balance could not be read; hides stale figures and reads Unavailable.">
<meta name="fixture:default" content='{"name":"Gold","icon":"coin","amount":12540,"valueText":"","limit":"","rate":"+48/s","full":false,"loading":false,"unavailable":false}'>
<meta name="fixture:empty" content='{"name":"Gems","icon":"gem","amount":0}'>
<meta name="fixture:full" content='{"name":"Energy","icon":"bolt","amount":120,"limit":"120","full":true}'>
<meta name="fixture:loading" content='{"name":"Gold","icon":"coin","amount":999,"loading":true}'>
<meta name="fixture:unavailable" content='{"name":"Gold","icon":"coin","amount":999,"unavailable":true}'>
<meta name="fixture:overflow" content='{"name":"Sovereigns of the Eastern Treasury","icon":"crown","valueText":"123.45 quattuordecillion","limit":"500 quattuordecillion","rate":"−4.2 million per second"}'>

<!-- A readout, not a button. A game can put it beside a separately named
     purchase control; Currency never invents an economy or sends an intent.
     A chip: the resource's icon in a socket, its name over the amount, and
     beside the amount its capacity, its rate and Full. It is as wide as
     what it holds. -->
<div id="Currency" part="frame" variant="hud" layout="row" aria-label={name || t("currency.name")} aria-busy={loading} aria-disabled={unavailable} data-full={full} data-unavailable={unavailable}>
  {#if icon}<div part="well" class="socket cover" layout="stack" fill><og-icon class="icon" part="icon" icon={icon}></og-icon></div>{/if}
  <div class="body" layout="stack" gap="none">
    <div part="caption" class="name">{name || t("currency.name")}</div>
    {#if loading}
      <div class="wait" layout="row" gap="tight"><div part="spinner"><Spinner size={18} name={t("currency.loading")} /></div><div part="headline" class="state" role="status">{t("currency.loading")}</div></div>
    {:else if unavailable}
      <div part="headline" class="state" role="status">{t("currency.unavailable")}</div>
    {:else}
      <div class="figures" layout="wrap" gap="tight">
        <div part="figure" class="amount">{valueText || formatCompact(amount)}</div>
        {#if limit}<div class="limit" part="caption">{t("currency.capacity", { limit })}</div>{/if}
        {#if full}<div part="ribbon" class="full" tone="warn" play="pop">{t("currency.full")}</div>{/if}
        {#if rate}<div part="badge" class="rate">{rate}</div>{/if}
      </div>
    {/if}
  </div>
</div>

<style>
.cover { justify-content: center; align-items: center; }
#Currency { inline-size: fit-content; max-inline-size: 100%; padding: var(--space-xs) var(--space-md) var(--space-xs) var(--space-xs); }
.socket { flex: none; inline-size: calc(var(--size-md) + var(--space-xs)); block-size: calc(var(--size-md) + var(--space-xs)); }
.icon { inline-size: var(--size-sm); block-size: var(--size-sm); }
.body { flex: 0 1 auto; min-inline-size: 0; overflow-wrap: anywhere; }
.figures { align-items: baseline; }
.amount { --figure-size: var(--type-lg-size); --figure-weight: var(--font-weight-bold); }
.state { --headline-ink: var(--ink-label); }
.full { --ribbon-radius: var(--radius-capsule); }
.rate { --badge-radius: var(--radius-control); }
</style>
