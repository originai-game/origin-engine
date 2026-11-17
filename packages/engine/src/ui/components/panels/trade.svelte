<meta name="component" content="RpgTrade">
<meta name="strings" content="./economy.en.ogt">
<meta name="parts" content="RpgTrade.frame RpgTrade.title RpgTrade.wallet RpgTrade.mode RpgTrade.goods RpgTrade.row RpgTrade.face RpgTrade.icon RpgTrade.name RpgTrade.stock RpgTrade.reason RpgTrade.detail RpgTrade.description RpgTrade.price RpgTrade.buy RpgTrade.sell RpgTrade.status RpgTrade.empty RpgTrade.figure RpgTrade.caption RpgTrade.well">
<meta name="prop:title" content="string" data-default="" data-doc="The merchant or trade title; empty uses Trade.">
<meta name="prop:mode" content="string" data-default="buy" data-doc="buy or sell. The game changes this in response to switchMode and supplies that side's goods and prices.">
<meta name="prop:wallet" content="string" data-default="" data-doc="The player's balance as the game words it; empty hides it.">
<meta name="prop:goods" content="list" data-default="[]" data-doc="Keyed { id, name, icon, stock, description, price, available, reason }. stock and the total price are game-formatted strings; available includes balance, capacity, quantity and legality checks.">
<meta name="prop:selected" content="string" data-default="" data-doc="The inspected good's id. Unavailable goods remain selectable so their restriction can be read.">
<meta name="prop:loading" content="bool" data-default="false" data-doc="Goods are loading; selection and transactions are disabled.">
<meta name="prop:busy" content="bool" data-default="false" data-doc="A transaction is pending; selection, mode switching and transactions are disabled.">
<meta name="prop:error" content="string" data-default="" data-doc="The game's transaction failure, kept visible alongside the selection.">
<meta name="fixture:default" content='{"title":"Mira’s supplies","wallet":"120 gold","mode":"buy","goods":[{"id":"potion","name":"Healing draught ×2","icon":"potion","stock":"8 in stock","description":"Two draughts for the road.","price":"20 gold","available":true,"reason":""},{"id":"gem","name":"Storm crystal","icon":"gem","stock":"1 in stock","description":"A charged crafting catalyst.","price":"240 gold","available":false,"reason":"Not enough gold"}],"selected":"potion","loading":false,"busy":false,"error":""}'>
<meta name="fixture:empty" content='{}'>
<meta name="fixture:loading" content='{"loading":true}'>
<meta name="fixture:sell" content='{"mode":"sell","wallet":"120 gold","selected":"gem","goods":[{"id":"gem","name":"Storm crystal","icon":"gem","stock":"3 owned","description":"Sell one crystal.","price":"80 gold","available":true,"reason":""}]}'>
<meta name="fixture:unavailable" content='{"selected":"gem","goods":[{"id":"gem","name":"Storm crystal","icon":"gem","stock":"1 in stock","description":"A charged crafting catalyst.","price":"240 gold","available":false,"reason":"Not enough gold"}],"error":"The price changed. Review it before trying again."}'>

<section id="RpgTrade" part="RpgTrade.frame" data-layout="scroll" data-gap="loose" aria-label={title || t("rpg.trade.title")} aria-busy={loading || busy}>
  <h2 part="RpgTrade.title">{title || t("rpg.trade.title")}</h2>
  {#if wallet}<p part="RpgTrade.wallet RpgTrade.figure">{wallet}</p>{/if}
  <button part="RpgTrade.mode" disabled={loading || busy} onclick={switchMode}>{mode == 'sell' ? t("rpg.trade.toBuy") : t("rpg.trade.toSell")}</button>
  {#if error}<p part="RpgTrade.status RpgTrade.well" data-tone="danger" role="alert">{error}</p>{/if}
  {#if loading || busy}<div part="RpgTrade.status RpgTrade.well" data-layout="row" role="status"><Spinner name={busy ? t("rpg.economy.busy") : t("rpg.economy.loading")} /><span>{busy ? t("rpg.economy.busy") : t("rpg.economy.loading")}</span></div>{/if}
  {#if goods.length}<div part="RpgTrade.goods" data-layout="stack" data-gap="tight" role="listbox" aria-label={mode == 'sell' ? t("rpg.trade.sell") : t("rpg.trade.buy")}>
    {#each goods as good (good.id)}<button part="RpgTrade.row" role="option" aria-selected={good.id == selected} disabled={loading || busy} onclick={choose}><div part="RpgTrade.face" data-layout="cluster"><og-icon part="RpgTrade.icon" icon={good.icon}></og-icon><div part="RpgTrade.name" data-fill>{good.name}</div><div part="RpgTrade.stock RpgTrade.caption">{good.stock}</div></div></button>{/each}
  </div>
  {#if !selected}<p part="RpgTrade.empty RpgTrade.caption">{t("rpg.trade.choose")}</p>{/if}
  {#each goods as good (good.id)}{#if good.id == selected}<section part="RpgTrade.detail RpgTrade.well" data-layout="stack" aria-label={good.name}>
    {#if good.description}<p part="RpgTrade.description">{good.description}</p>{/if}
    <p part="RpgTrade.price RpgTrade.figure">{t("rpg.trade.price", { price: good.price })}</p>
    {#if !good.available}<p part="RpgTrade.reason RpgTrade.caption">{good.reason || t("rpg.economy.unavailable")}</p>{/if}
    {#if mode == 'sell'}<button part="RpgTrade.sell" class="primary" disabled={!good.available || loading || busy} onclick={sell}>{t("rpg.trade.sell")}</button>{:else}<button part="RpgTrade.buy" class="primary" disabled={!good.available || loading || busy} onclick={buy}>{t("rpg.trade.buy")}</button>{/if}
  </section>{/if}{/each}
  {:else if !loading}<p part="RpgTrade.empty RpgTrade.caption" role="status">{t("rpg.trade.empty")}</p>{/if}
</section>

<style>
:host { inline-size: stretch; max-block-size: stretch; }
h2, p { margin: 0; }
[part="RpgTrade.row"] { min-block-size: var(--size-touch); padding: var(--space-sm); --row-fill: var(--well); }
[part="RpgTrade.icon"] { inline-size: var(--size-sm); block-size: var(--size-sm); flex: none; }
[part="RpgTrade.name"] { text-align: start; }
[part~="RpgTrade.detail"] { padding: var(--space-md); }
[part="RpgTrade.mode"], [part="RpgTrade.buy"], [part="RpgTrade.sell"] { min-block-size: var(--size-touch); }
[part~="RpgTrade.status"] { padding: var(--space-sm); }
</style>
