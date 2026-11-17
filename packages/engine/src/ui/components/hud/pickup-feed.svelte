<meta name="component" content="PickupFeed">
<meta name="strings" content="./pickup-feed.en.ogt">
<meta name="parts" content="PickupFeed.frame PickupFeed.heading PickupFeed.entries PickupFeed.entry PickupFeed.icon PickupFeed.name PickupFeed.quantity PickupFeed.detail PickupFeed.state PickupFeed.loading PickupFeed.caption PickupFeed.figure PickupFeed.row">
<meta name="prop:label" content="string" data-default="" data-doc="The feed heading; empty uses Pickups.">
<meta name="prop:items" content="list" data-doc="Keyed entries { id, name, icon, quantity, detail }. The game owns ordering, expiry and retention; quantity is a signed change. No timers run in this document.">
<meta name="prop:loading" content="bool" data-default="false" data-doc="Waiting for pickup data; stale entries are hidden.">
<meta name="fixture:default" content='{"label":"Recovered supplies","items":[{"id":"ore-7","name":"Iron ore","icon":"gem","quantity":12,"detail":"Crafting material"},{"id":"key-2","name":"Observatory key","icon":"key","quantity":1,"detail":"Quest item"},{"id":"spent","name":"Gold","icon":"coin","quantity":-40,"detail":"Supplies purchased"}],"loading":false}'>
<meta name="fixture:empty" content='{"items":[]}'>
<meta name="fixture:loading" content='{"loading":true}'>
<meta name="fixture:overflow" content='{"items":[{"id":"relic","name":"Fragment of the northern observatory mechanism","icon":"gem","quantity":123456,"detail":"Recovered from the abandoned expedition storehouse"}]}'>

<section id="PickupFeed" part="PickupFeed.frame" class="hud" data-layout="stack" aria-busy={loading} aria-label={label || t("hud.pickup-feed.name")}>
  <div part="PickupFeed.heading">{label || t("hud.pickup-feed.name")}</div>
  {#if loading}<div part="PickupFeed.loading" data-layout="row"><Spinner name={t("hud.pickup-feed.loading")} /><div part="PickupFeed.state PickupFeed.caption" role="status">{t("hud.pickup-feed.loading")}</div></div>
  {:else}
    <ol part="PickupFeed.entries" data-layout="stack" role="log" aria-live="polite">
      {#each items as item (item.id)}<li part="PickupFeed.entry PickupFeed.row" data-layout="row">
        {#if item.icon}<og-icon part="PickupFeed.icon" icon={item.icon}></og-icon>{/if}
        <div class="description" data-layout="stack" data-gap="hair" data-fill><div part="PickupFeed.name">{item.name}</div>{#if item.detail}<div part="PickupFeed.detail PickupFeed.caption">{item.detail}</div>{/if}</div>
        <div part="PickupFeed.quantity PickupFeed.figure" data-trend={item.quantity < 0 ? 'down' : 'up'}>{item.quantity > 0 ? '+' : ''}{formatNumber(item.quantity)}</div>
      </li>{/each}
    </ol>
    {#if !items.length}<div part="PickupFeed.state PickupFeed.caption">{t("hud.pickup-feed.empty")}</div>{/if}
  {/if}
</section>

<style>
:host { max-inline-size: 100%; }
[part="PickupFeed.heading"] { overflow-wrap: anywhere; }
[part="PickupFeed.entries"] { padding: 0; margin: 0; list-style: none; }
[part~="PickupFeed.entry"] { align-items: start; }
[part="PickupFeed.icon"] { flex: none; inline-size: var(--size-xs); block-size: var(--size-xs); }
[part="PickupFeed.name"], [part~="PickupFeed.detail"] { overflow-wrap: anywhere; }
[part~="PickupFeed.quantity"] { flex: none; }
</style>
