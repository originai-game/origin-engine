<meta name="component" content="PickupFeed">
<meta name="strings" content="./pickup-feed.en.ogt">
<meta name="prop:label" content="string" data-default="" data-doc="The feed heading; empty uses Pickups.">
<meta name="prop:items" content="list" data-doc="Keyed entries { id, name, icon, quantity, detail }. The game owns ordering, expiry and retention; quantity is a signed change. No timers run in this document.">
<meta name="prop:loading" content="bool" data-default="false" data-doc="Waiting for pickup data; stale entries are hidden.">
<meta name="fixture:default" content='{"label":"Recovered supplies","items":[{"id":"ore-7","name":"Iron ore","icon":"gem","quantity":12,"detail":"Crafting material"},{"id":"key-2","name":"Observatory key","icon":"key","quantity":1,"detail":"Quest item"},{"id":"spent","name":"Gold","icon":"coin","quantity":-40,"detail":"Supplies purchased"}],"loading":false}'>
<meta name="fixture:empty" content='{"items":[]}'>
<meta name="fixture:loading" content='{"loading":true}'>
<meta name="fixture:overflow" content='{"items":[{"id":"relic","name":"Fragment of the northern observatory mechanism","icon":"gem","quantity":123456,"detail":"Recovered from the abandoned expedition storehouse"}]}'>

<section id="PickupFeed" part="frame" variant="hud" layout="stack" aria-busy={loading} aria-label={label || t("hud.pickup-feed.name")}>
  <div class="heading" part="heading">{label || t("hud.pickup-feed.name")}</div>
  {#if loading}<div part="loading" layout="row"><Spinner name={t("hud.pickup-feed.loading")} /><div class="state" part="caption" role="status">{t("hud.pickup-feed.loading")}</div></div>
  {:else}
    <ol class="entries" part="entries" layout="stack" role="log" aria-live="polite">
      {#each items as item (item.id)}<li class="entry" part="row" layout="row">
        {#if item.icon}<og-icon class="icon" part="icon" icon={item.icon}></og-icon>{/if}
        <div class="description" layout="stack" gap="hair" fill><div class="name" part="name">{item.name}</div>{#if item.detail}<div class="detail" part="caption">{item.detail}</div>{/if}</div>
        <div class="quantity" part="figure" data-trend={item.quantity < 0 ? 'down' : 'up'}>{item.quantity > 0 ? '+' : ''}{formatNumber(item.quantity)}</div>
      </li>{/each}
    </ol>
    {#if !items.length}<div class="state" part="caption">{t("hud.pickup-feed.empty")}</div>{/if}
  {/if}
</section>

<style>
#PickupFeed { max-inline-size: 100%; }
.heading { overflow-wrap: anywhere; }
.entries { padding: 0; margin: 0; list-style: none; }
.entry { align-items: start; }
.icon { flex: none; inline-size: var(--size-xs); block-size: var(--size-xs); }
.name, .detail { overflow-wrap: anywhere; }
.quantity { flex: none; }
</style>
