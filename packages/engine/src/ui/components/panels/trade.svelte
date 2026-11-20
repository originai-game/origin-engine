<meta name="component" content="RpgTrade">
<meta name="strings" content="./economy.en.ogt">
<meta name="prop:title" content="string" data-default="" data-doc="The merchant or trade title; empty uses Trade.">
<meta name="prop:mode" content="string" data-default="buy" data-doc="buy or sell. The game changes this in response to switchMode and supplies that side's goods and prices.">
<meta name="prop:wallet" content="string" data-default="" data-doc="The player's balance as the game words it; empty hides it.">
<meta name="prop:goods" content="list" data-default="[]" data-doc="Keyed { id, name, icon, stock, description, price, was, after, available, reason, rarity }. stock, price (the total), was (the struck price before a discount, empty none) and after (the balance once paid, empty hides it) are game-formatted strings; available includes balance, capacity, quantity and legality checks; rarity (common … mythic, empty common) colours the row's edge, the socket and the name.">
<meta name="prop:selected" content="string" data-default="" data-doc="The inspected good's id. Unavailable goods remain selectable so their restriction can be read.">
<meta name="prop:loading" content="bool" data-default="false" data-doc="Goods are loading; selection and transactions are disabled.">
<meta name="prop:busy" content="bool" data-default="false" data-doc="A transaction is pending; selection, mode switching and transactions are disabled.">
<meta name="prop:error" content="string" data-default="" data-doc="The game's transaction failure, kept visible alongside the selection.">
<meta name="fixture:default" content='{"title":"Mira’s supplies","wallet":"120 gold","mode":"buy","goods":[{"id":"potion","name":"Healing draught ×2","icon":"potion","stock":"8 in stock","description":"Two draughts for the road.","price":"20 gold","was":"","after":"120 → 100","available":true,"reason":"","rarity":"uncommon"},{"id":"lamp","name":"Hooded lantern","icon":"fire","stock":"2 in stock","description":"Burns through the night.","price":"45 gold","was":"60 gold","after":"120 → 75","available":true,"reason":"","rarity":"rare"},{"id":"gem","name":"Storm crystal","icon":"gem","stock":"1 in stock","description":"A charged crafting catalyst.","price":"240 gold","was":"","after":"","available":false,"reason":"Not enough gold","rarity":"epic"}],"selected":"potion","loading":false,"busy":false,"error":""}'>
<meta name="fixture:empty" content='{}'>
<meta name="fixture:loading" content='{"loading":true}'>
<meta name="fixture:sell" content='{"mode":"sell","wallet":"120 gold","selected":"gem","goods":[{"id":"gem","name":"Storm crystal","icon":"gem","stock":"3 owned","description":"Sell one crystal.","price":"80 gold","was":"","after":"120 → 200","available":true,"reason":"","rarity":"epic"}]}'>
<meta name="fixture:unavailable" content='{"selected":"gem","goods":[{"id":"gem","name":"Storm crystal","icon":"gem","stock":"1 in stock","description":"A charged crafting catalyst.","price":"240 gold","was":"","after":"","available":false,"reason":"Not enough gold","rarity":"epic"}],"error":"The price changed. Review it before trying again."}'>

<!--
  A merchant: a band with the merchant's name, the Buy / Sell switch and
  the purse; an alert when a deal failed; the goods as rows (the item's
  icon in a socket, its rarity on the row's edge and in its name, its stock
  or why it cannot be had, and its price, a discount struck before it);
  and at the foot the selected good with its total, the balance after, and
  Buy or Sell. The game decides every price and verdict; the panel reports
  choose, switchMode, buy and sell, each with the good's key.
-->
<section class="frame" id="RpgTrade" part="frame" layout="stack" fill gap="loose" aria-label={title || t("rpg.trade.title")} aria-busy={loading || busy}>
  <div class="lit"></div>
  <header part="band" class="band" layout="wrap" gap="loose">
    <div class="heading" layout="stack" gap="hair" fill><span part="kicker" class="kicker">{t("rpg.trade.kicker")}</span><h2 class="title" part="title">{title || t("rpg.trade.title")}</h2></div>
    <button part="mode" class="mode" role="switch" aria-checked={mode == 'sell'} aria-label={mode == 'sell' ? t("rpg.trade.toBuy") : t("rpg.trade.toSell")} disabled={loading || busy} onclick={switchMode}><span class="seg" class:on={mode != 'sell'}>{t("rpg.trade.buy")}</span><span class="seg" class:on={mode == 'sell'}>{t("rpg.trade.sell")}</span></button>
    {#if wallet}<div part="figure" class="purse wallet" layout="row" gap="tight"><span class="coin"></span><span>{wallet}</span></div>{/if}
  </header>
  {#if error}<p part="status" class="alert" tone="danger" role="alert">{error}</p>{/if}
  {#if loading || busy}<div part="well" class="wait status" layout="row" role="status"><Spinner name={busy ? t("rpg.economy.busy") : t("rpg.economy.loading")} /><span>{busy ? t("rpg.economy.busy") : t("rpg.economy.loading")}</span></div>{/if}
  {#if goods.length}
    <div part="goods" class="goods" layout="scroll" fill role="listbox" aria-label={mode == 'sell' ? t("rpg.trade.sell") : t("rpg.trade.buy")}>
      {#each goods as good (good.id)}<button part="row" class="row" role="option" tier={good.rarity || 'common'} aria-selected={good.id == selected} disabled={loading || busy} onclick={choose} state:selected={good.id == selected} class:not-available={!(good.available)}><div part="face" class="face">
        <div part="socket" class="socket cover" layout="stack" fill><og-icon part="icon" icon={good.icon}></og-icon></div>
        <div class="words" layout="stack" gap="hair"><span part="name" class="name">{good.name}</span>{#if !good.available}<span part="reason" class="reason">{good.reason || t("rpg.economy.unavailable")}</span>{:else}<span class="stock" part="caption">{good.stock}</span>{/if}</div>
        <div part="figure" class="price">{#if good.was}<span part="former" class="was" aria-label={t("rpg.trade.was", { price: good.was })}>{good.was}</span>{/if}<span>{good.price}</span></div>
      </div></button>{/each}
    </div>
    {#if !selected}<p part="caption" class="guide empty">{t("rpg.trade.choose")}</p>{/if}
    {#each goods as good (good.id)}{#if good.id == selected}<footer part="detail" class="detail" tier={good.rarity || 'common'} aria-label={good.name}>
      <div class="summary">
        <div class="socket big cover" layout="stack" fill><og-icon icon={good.icon}></og-icon></div>
        <div class="words" layout="stack" gap="hair"><span class="name">{good.name}</span>{#if !good.available}<span class="reason">{good.reason || t("rpg.economy.unavailable")}</span>{:else if good.description}<span class="description" part="caption">{good.description}</span>{/if}</div>
        <div class="sum" layout="stack" gap="hair"><span class="kicker">{t("rpg.trade.total")}</span><span part="figure" class="total" aria-label={t("rpg.trade.price", { price: good.price })}>{good.price}</span>{#if good.after}<span part="caption" class="after">{good.after}</span>{/if}</div>
      </div>
      {#if mode == 'sell'}<button part="sell" class="go deal" variant="primary" disabled={!good.available || loading || busy} aria-label={t("rpg.trade.sell")} onclick={sell}><kbd class="cap" data-action="details"></kbd><span>{t("rpg.trade.sell")}</span></button>{:else}<button part="buy" class="go deal" variant="primary" disabled={!good.available || loading || busy} aria-label={t("rpg.trade.buy")} onclick={buy}><kbd class="cap" data-action="details"></kbd><span>{t("rpg.trade.buy")}</span></button>{/if}
    </footer>{/if}{/each}
  {:else if !loading}<p part="caption" class="guide empty" role="status">{t("rpg.trade.empty")}</p>{/if}
</section>

<style>
.cover { justify-content: center; align-items: center; }
.frame { position: relative; inline-size: stretch; max-block-size: stretch; min-block-size: 0; border-radius: var(--inv-frame-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.lit { position: absolute; inset-inline: var(--inv-frame-cut) 30%; top: 0; block-size: var(--stroke-hairline); background: var(--inv-lit-edge); pointer-events: none; }
h2, p { margin: 0; }
.band { flex: none; width: stretch; align-items: flex-end; }
.heading { min-width: 0; }
.kicker { font-size: var(--inv-kicker-size); font-weight: var(--inv-name-weight); letter-spacing: var(--inv-kicker-tracking); text-transform: var(--inv-kicker-transform); color: var(--inv-kicker-ink); white-space: nowrap; }
.title { overflow-wrap: anywhere; }
/* The switch: Buy and Sell as two segments in a well, the side shown lit. */
.mode { flex: none; min-block-size: var(--size-touch); padding: var(--space-xxs); gap: var(--space-xxs); border-radius: var(--inv-seg-radius); background: var(--inv-chip-fill); box-shadow: var(--inv-chip-rim); }
.seg { padding: var(--space-xs) var(--space-sm); border-radius: var(--inv-seg-radius); color: var(--inv-tab-ink); }
.seg.on { color: var(--inv-tab-on-ink); background: var(--inv-tab-count-fill); box-shadow: var(--inv-tag-rim); }
.purse { flex: none; align-items: center; padding: var(--space-xxs) var(--space-sm) var(--space-xxs) var(--space-xxs); border-radius: var(--inv-chip-radius); background: var(--inv-chip-fill); box-shadow: var(--inv-chip-rim); white-space: nowrap; font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); }
.coin { flex: none; inline-size: calc(22 * var(--font-step-px)); block-size: calc(22 * var(--font-step-px)); border-radius: var(--inv-chip-radius); background: var(--inv-coin); }
.alert { flex: none; padding: var(--space-xs) var(--space-sm); background: var(--inv-alert-fill); box-shadow: var(--inv-alert-rim); }
.wait { flex: none; padding: var(--space-sm); }
.goods { gap: var(--space-xs); min-block-size: 0; }
.row { --inv-rarity: var(--common); --inv-socket-fill: var(--cell-common-fill); --inv-socket-rim: var(--cell-common-shadow); flex: none; width: stretch; min-block-size: var(--size-touch); padding: var(--space-xs) var(--space-sm) var(--space-xs) var(--space-md); justify-content: flex-start; border: none; background: var(--inv-row-fill); box-shadow: var(--inv-row-rim); border-inline-start: var(--inv-edge); border-inline-start-color: var(--inv-rarity); border-radius: var(--inv-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.row:hover { background: var(--inv-row-hover-fill); }
.row:state(selected) { background: var(--inv-row-on-fill); box-shadow: var(--inv-row-on-rim); }
.row.rarity-uncommon, .detail.rarity-uncommon { --inv-rarity: var(--uncommon); --inv-socket-fill: var(--cell-uncommon-fill); --inv-socket-rim: var(--cell-uncommon-shadow); }
.row.rarity-rare, .detail.rarity-rare { --inv-rarity: var(--rare); --inv-socket-fill: var(--cell-rare-fill); --inv-socket-rim: var(--cell-rare-shadow); }
.row.rarity-epic, .detail.rarity-epic { --inv-rarity: var(--epic); --inv-socket-fill: var(--cell-epic-fill); --inv-socket-rim: var(--cell-epic-shadow); }
.row.rarity-legendary, .detail.rarity-legendary { --inv-rarity: var(--legendary); --inv-socket-fill: var(--cell-legendary-fill); --inv-socket-rim: var(--cell-legendary-shadow); }
.row.rarity-mythic, .detail.rarity-mythic { --inv-rarity: var(--mythic); --inv-socket-fill: var(--cell-mythic-fill); --inv-socket-rim: var(--cell-mythic-shadow); }
.face { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; align-items: center; column-gap: var(--space-sm); row-gap: var(--space-xxs); width: stretch; min-width: 0; }
.summary { display: flex; align-items: center; gap: var(--space-sm); width: stretch; min-width: 0; }
.socket { flex: none; inline-size: calc(44 * var(--font-step-px)); block-size: calc(44 * var(--font-step-px)); background: var(--inv-socket-fill); box-shadow: var(--inv-socket-rim); border-radius: var(--inv-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.socket og-icon { inline-size: 74%; block-size: 74%; filter: var(--inv-slot-art-shadow); }
.socket.big { inline-size: calc(60 * var(--font-step-px)); block-size: calc(60 * var(--font-step-px)); }
.row.not-available .socket { opacity: var(--disabled-content-opacity); }
.words { flex: 1 1 0; min-width: 0; align-items: flex-start; text-align: start; }
.name { overflow-wrap: anywhere; font-weight: var(--inv-name-weight); color: var(--inv-rarity); }
.row.rarity-common .name, .detail.rarity-common .name { color: var(--ink-bright); }
.reason { color: var(--inv-broken-ink); overflow-wrap: anywhere; }
.price { flex: none; display: flex; align-items: baseline; gap: var(--space-xs); white-space: nowrap; font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); }
.row.not-available .price { color: var(--inv-broken-ink); }
.was { font-weight: var(--inv-name-weight); }
.guide { padding: var(--space-lg); text-align: center; }
/* The deal at the foot: the good, its total and the balance after, then Buy or Sell. */
.detail { flex: none; display: flex; flex-direction: column; gap: var(--space-sm); width: stretch; padding-block-start: var(--space-sm); border-block-start: var(--inv-rule); }
.sum { flex: none; align-items: flex-end; text-align: end; }
.total { font-size: var(--inv-figure-xl-size); font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); white-space: nowrap; }
.after { white-space: nowrap; }
.deal { width: stretch; min-block-size: calc(var(--size-touch) * 1.15); gap: var(--space-xs); }
.cap { flex: none; }
/* Narrow: the switch and the purse go under the title. */
.heading:size(compact) { flex-basis: 100%; }
.face .socket:size(compact) { grid-row: span 2; }
.price:size(compact) { grid-column: 2; justify-self: start; }
.summary:size(compact) { flex-wrap: wrap; }
.sum:size(compact) { flex-basis: 100%; flex-direction: row; justify-content: flex-end; align-items: baseline; gap: var(--space-sm); }
</style>
