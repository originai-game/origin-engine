<meta name="component" content="Currency">
<meta name="strings" content="./currency.en.ogt">
<meta name="parts" content="Currency.frame Currency.icon Currency.name Currency.amount Currency.limit Currency.rate Currency.state">
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
     purchase control; Currency never invents an economy or sends an intent. -->
<div id="Currency" part="Currency.frame" class="bare" aria-label={name || t("currency.name")} aria-busy={loading} data-full={full} data-unavailable={unavailable}>
  {#if icon}<og-icon part="Currency.icon" icon={icon}></og-icon>{/if}
  <div class="body">
    <div part="Currency.name">{name || t("currency.name")}</div>
    {#if loading || unavailable}
      <div part="Currency.state" role="status">{loading ? t("currency.loading") : t("currency.unavailable")}</div>
    {:else}
      <div class="figures">
        <div part="Currency.amount">{valueText || formatCompact(amount)}</div>
        {#if limit}<div part="Currency.limit">/ {limit}</div>{/if}
      </div>
      {#if rate}<div part="Currency.rate">{rate}</div>{/if}
      {#if full}<div part="Currency.state">{t("currency.full")}</div>{/if}
    {/if}
  </div>
</div>

<style>
:host { display: flex; align-items: center; gap: var(--space-sm); max-inline-size: stretch; padding: var(--space-xs) var(--space-sm); border: var(--stroke-hairline) solid var(--line); border-radius: var(--radius-control); background: var(--panel); color: var(--ink); }
[part="Currency.icon"] { flex: none; inline-size: var(--size-sm); block-size: var(--size-sm); }
.body { display: flex; flex-direction: column; gap: var(--space-xxs); min-inline-size: 0; }
[part="Currency.name"] { font: var(--text-sm); color: var(--ink-muted); overflow-wrap: anywhere; }
.figures { display: flex; flex-wrap: wrap; align-items: baseline; gap: 0 var(--space-xs); }
[part="Currency.amount"] { font-weight: var(--font-weight-semibold); }
[part="Currency.amount"], [part="Currency.limit"], [part="Currency.rate"] { font-family: var(--figure-font); overflow-wrap: anywhere; }
[part="Currency.limit"], [part="Currency.rate"], [part="Currency.state"] { font: var(--text-sm); }
[part="Currency.limit"], [part="Currency.rate"] { color: var(--ink-muted); }
[part="Currency.state"] { font-weight: var(--font-weight-semibold); }
</style>
