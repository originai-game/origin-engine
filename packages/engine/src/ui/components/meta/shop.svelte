<meta name="component" content="Shop">
<meta name="strings" content="./general.en.ogt">
<meta name="parts" content="Shop.frame Shop.band Shop.title Shop.wallet Shop.balance Shop.tab Shop.offer Shop.name Shop.badge Shop.price Shop.status Shop.figure Shop.cell Shop.label Shop.headline Shop.caption Shop.well Shop.former">
<meta name="prop:title" content="string" data-doc="The shop's heading; empty takes the group's (Shop).">
<meta name="prop:wallet" content="list" data-doc="The player's balances shown in the band, keyed rows of { id, icon, label, amount }: amount as the game formats it; label names the currency for a screen reader.">
<meta name="prop:tabs" content="list" data-doc="The categories, keyed rows of { id, label }; empty shows no tab row.">
<meta name="prop:tab" content="string" data-doc="The category shown.">
<meta name="prop:offers" content="list" data-doc="The offers in the shown category, in the game's order: keyed rows of { id, name, icon, detail, badge, featured, soldOut, owned, affordable }: detail a second line (x500, 7 days); badge a corner tag the game words (-30%, New), empty none; featured a wide tile; soldOut, owned and affordable the game's verdicts: a sold out, owned or unaffordable (affordable false) tile keeps the focus, is marked disabled, says why in words and reports nothing.">
<meta name="prop:prices" content="list" data-doc="Every offer's price, one row per currency: keyed rows of { id, offer, icon, label, amount, was, short }: offer an offer's id; amount and was (the struck price before a discount, empty none) as the game formats them; label the currency's name; short marks the currency the player lacks.">
<meta name="prop:selected" content="string" data-doc="The offer the focus rests on; empty is none.">
<meta name="prop:loading" content="bool" data-doc="The catalogue is on its way: a spinner stands in.">
<meta name="prop:error" content="string" data-doc="Why the catalogue could not load, as the game words it; empty is none. Shows a Try again button.">
<meta name="action:tabPrev" content="Previous category">
<meta name="action:tabNext" content="Next category">
<meta name="fixture:default" content='{"title":"","wallet":[{"id":"gold","icon":"coin","label":"Gold","amount":"12,450"},{"id":"gems","icon":"gem","label":"Gems","amount":"320"}],"tabs":[{"id":"featured","label":"Featured"},{"id":"items","label":"Items"},{"id":"looks","label":"Looks"}],"tab":"featured","offers":[{"id":"bundle","name":"Founder Bundle","icon":"crown","detail":"3 items, 500 gems","badge":"-30%","featured":true,"soldOut":false,"owned":false,"affordable":true},{"id":"potion","name":"Potion Pack","icon":"potion","detail":"x10","badge":"","featured":false,"soldOut":false,"owned":false,"affordable":true},{"id":"cape","name":"Ember Cape","icon":"fire","detail":"Look","badge":"New","featured":false,"soldOut":false,"owned":true,"affordable":true},{"id":"key","name":"Vault Key","icon":"key","detail":"x1","badge":"","featured":false,"soldOut":true,"owned":false,"affordable":true},{"id":"dragon","name":"Dragon Mount","icon":"star","detail":"Mount","badge":"","featured":false,"soldOut":false,"owned":false,"affordable":false}],"prices":[{"id":"bundle-gems","offer":"bundle","icon":"gem","label":"Gems","amount":"700","was":"1,000","short":false},{"id":"potion-gold","offer":"potion","icon":"coin","label":"Gold","amount":"1,200","was":"","short":false},{"id":"potion-gems","offer":"potion","icon":"gem","label":"Gems","amount":"40","was":"","short":false},{"id":"cape-gems","offer":"cape","icon":"gem","label":"Gems","amount":"250","was":"","short":false},{"id":"key-gold","offer":"key","icon":"coin","label":"Gold","amount":"5,000","was":"","short":false},{"id":"dragon-gems","offer":"dragon","icon":"gem","label":"Gems","amount":"2,400","was":"","short":true}],"selected":"bundle","loading":false,"error":""}'>
<meta name="fixture:empty" content='{"title":"","wallet":[{"id":"gold","icon":"coin","label":"Gold","amount":"0"}],"tabs":[{"id":"items","label":"Items"}],"tab":"items","offers":[],"prices":[],"selected":"","loading":false,"error":""}'>
<meta name="fixture:loading" content='{"title":"","wallet":[],"tabs":[],"tab":"","offers":[],"prices":[],"selected":"","loading":true,"error":""}'>
<meta name="fixture:error" content='{"title":"","wallet":[],"tabs":[],"tab":"","offers":[],"prices":[],"selected":"","loading":false,"error":"The shop could not be reached."}'>
<meta name="fixture:overflow" content='{"title":"The Wandering Merchant of the Far Eastern Reaches","wallet":[{"id":"gold","icon":"coin","label":"Gold","amount":"1,234,567,890"},{"id":"gems","icon":"gem","label":"Gems","amount":"99,999"},{"id":"tokens","icon":"star","label":"Tokens","amount":"12"}],"tabs":[{"id":"a","label":"Featured"},{"id":"b","label":"Consumables"},{"id":"c","label":"Equipment"},{"id":"d","label":"Cosmetics"},{"id":"e","label":"Bundles"},{"id":"f","label":"Seasonal"}],"tab":"b","offers":[{"id":"o1","name":"An Extraordinarily Long Offer Name That Must Wrap","icon":"potion","detail":"x10 with a long detail line","badge":"Limited time","featured":false,"soldOut":false,"owned":false,"affordable":true},{"id":"o2","name":"Potion","icon":"potion","detail":"x1","badge":"","featured":false,"soldOut":false,"owned":false,"affordable":false},{"id":"o3","name":"Elixir","icon":"drop","detail":"x5","badge":"","featured":false,"soldOut":false,"owned":false,"affordable":true},{"id":"o4","name":"Bomb","icon":"bomb","detail":"x3","badge":"","featured":false,"soldOut":true,"owned":false,"affordable":true},{"id":"o5","name":"Apple","icon":"apple","detail":"x20","badge":"","featured":false,"soldOut":false,"owned":false,"affordable":true},{"id":"o6","name":"Leaf","icon":"leaf","detail":"x8","badge":"","featured":false,"soldOut":false,"owned":false,"affordable":true},{"id":"o7","name":"Bolt","icon":"bolt","detail":"x2","badge":"","featured":false,"soldOut":false,"owned":false,"affordable":true},{"id":"o8","name":"Shield","icon":"shield","detail":"x1","badge":"","featured":false,"soldOut":false,"owned":false,"affordable":true}],"prices":[{"id":"p1a","offer":"o1","icon":"coin","label":"Gold","amount":"123,456,789","was":"200,000,000","short":false},{"id":"p1b","offer":"o1","icon":"gem","label":"Gems","amount":"9,999","was":"","short":false},{"id":"p1c","offer":"o1","icon":"star","label":"Tokens","amount":"12","was":"","short":false},{"id":"p2","offer":"o2","icon":"star","label":"Tokens","amount":"99","was":"","short":true},{"id":"p3","offer":"o3","icon":"coin","label":"Gold","amount":"800","was":"","short":false},{"id":"p4","offer":"o4","icon":"coin","label":"Gold","amount":"300","was":"","short":false},{"id":"p5","offer":"o5","icon":"coin","label":"Gold","amount":"40","was":"","short":false},{"id":"p6","offer":"o6","icon":"coin","label":"Gold","amount":"90","was":"","short":false},{"id":"p7","offer":"o7","icon":"gem","label":"Gems","amount":"15","was":"","short":false},{"id":"p8","offer":"o8","icon":"coin","label":"Gold","amount":"2,000","was":"","short":false}],"selected":"o3","loading":false,"error":""}'>

<!--
  A storefront: a band with the title and the player's balances, category
  tabs, and a grid of offer tiles, each with its art, name, detail, a corner
  badge and its price in one or more currencies. The game owns the
  catalogue, every price and every verdict (sold out, owned, unaffordable);
  the shop reports buy (a tile's key, which the game answers with a
  ShopConfirm), choosetab (a category's key), tabNext and tabPrev (E/Q,
  bumpers) and retry. A tile the game refused has no buy handler: it keeps
  the focus so a player can read it, is marked disabled (aria-disabled) and
  says why in words. The grid fills the height it is given and scrolls
  inside it.
-->
<section id="Shop" part="Shop.frame" data-layout="shell" aria-label={title != '' ? title : t("general.shop.name")} aria-busy={loading}>
  <header part="Shop.band" data-layout="cluster" data-gap="loose">
    <h2 part="Shop.title" data-fill>{title != '' ? title : t("general.shop.name")}</h2>
    {#if wallet.length}<ul part="Shop.wallet" data-layout="cluster" aria-label={t("general.shop.wallet")}>
      {#each wallet as w (w.id)}
        <li part="Shop.balance Shop.figure" data-layout="row" data-gap="hair" aria-label={t("general.shop.amount", { amount: w.amount, currency: w.label })}><og-icon class="currency" icon={w.icon}></og-icon><span class="figure">{w.amount}</span></li>
      {/each}
    </ul>{/if}
  </header>
  {#if tabs.length}<div class="tabs" role="tablist" aria-label={t("general.shop.tabs")} ontabNext={tabNext} ontabPrev={tabPrev}>
    <div class="tabs-scroll">
      <div class="tabs-row" data-layout="row" data-gap="tight">
        {#each tabs as c (c.id)}
          <button part="Shop.tab" variant="quiet" role="tab" aria-selected={c.id == tab} onclick={choosetab}>{c.label}</button>
        {/each}
      </div>
    </div>
  </div>{/if}
  {#if error != ''}
    <div class="state" data-layout="cover" data-fill role="alert"><og-icon class="state-mark" icon="close"></og-icon><p class="state-text">{error}</p><button id="Retry" class="secondary" onclick={retry}>{t("general.shop.retry")}</button></div>
  {:else if loading}
    <div class="state" data-layout="cover" data-fill role="status"><Spinner name={t("general.shop.loading")} /><p class="state-text">{t("general.shop.loading")}</p></div>
  {:else if offers.length}
    <div class="grid-scroll" data-layout="scroll" data-fill>
      <div class="grid" data-layout="grid" role="grid" aria-label={title != '' ? title : t("general.shop.name")}>
        {#each offers as o (o.id)}
          {#if !o.soldOut && !o.owned && o.affordable}
            <button part="Shop.offer Shop.cell" role="gridcell" data-featured={o.featured} data-sold-out={o.soldOut} data-owned={o.owned} data-affordable={o.affordable} aria-selected={o.id == selected} aria-label={o.name} onclick={buy}><div class="face" data-layout="stack" data-gap="tight">
              {#if o.badge || o.featured}<div class="tags" data-layout="row" data-gap="tight">
                {#if o.featured}<span class="featured">{t("general.shop.featured")}</span>{/if}
                {#if o.badge}<p part="Shop.badge">{o.badge}</p>{/if}
              </div>{/if}
              <div part="Shop.well" class="art" data-layout="row" data-fill><og-icon class="art-icon" icon={o.icon}></og-icon></div>
              <div class="text" data-layout="stack" data-gap="hair">
                <p part="Shop.name Shop.headline">{o.name}</p>
                {#if o.detail}<span part="Shop.caption" class="detail">{o.detail}</span>{/if}
              </div>
              <div class="prices" data-layout="cluster" aria-label={t("general.shop.price")}>
                {#each prices as p (p.id)}
                  {#if p.offer == o.id}
                    <div part="Shop.price Shop.figure" data-layout="row" data-gap="hair" data-tone={p.short ? 'danger' : ''} data-short={p.short} aria-label={t("general.shop.amount", { amount: p.amount, currency: p.label })}>
                      <og-icon class="currency" icon={p.icon}></og-icon>
                      {#if p.was}<span part="Shop.former" class="was" aria-label={t("general.shop.was", { price: p.was })}>{p.was}</span>{/if}
                      <span class="figure">{p.amount}</span>
                    </div>
                  {/if}
                {/each}
              </div>
            </div></button>
          {:else}
            <button part="Shop.offer Shop.cell" role="gridcell" data-featured={o.featured} data-sold-out={o.soldOut} data-owned={o.owned} data-affordable={o.affordable} aria-selected={o.id == selected} aria-disabled={true} aria-label={t("general.shop.refused", { name: o.name, status: o.soldOut ? t("general.shop.soldOut") : o.owned ? t("general.shop.owned") : t("general.shop.short") })}><div class="face" data-layout="stack" data-gap="tight">
              {#if o.badge || o.featured}<div class="tags" data-layout="row" data-gap="tight">
                {#if o.featured}<span class="featured">{t("general.shop.featured")}</span>{/if}
                {#if o.badge}<p part="Shop.badge">{o.badge}</p>{/if}
              </div>{/if}
              <div part="Shop.well" class="art" data-layout="row" data-fill><og-icon class="art-icon" icon={o.icon}></og-icon></div>
              <div class="text" data-layout="stack" data-gap="hair">
                <p part="Shop.name Shop.headline">{o.name}</p>
                {#if o.detail}<span part="Shop.caption" class="detail">{o.detail}</span>{/if}
              </div>
              <div class="prices" data-layout="cluster" aria-label={t("general.shop.price")}>
                {#each prices as p (p.id)}
                  {#if p.offer == o.id}
                    <div part="Shop.price Shop.figure" data-layout="row" data-gap="hair" data-tone={p.short ? 'danger' : ''} data-short={p.short} aria-label={t("general.shop.amount", { amount: p.amount, currency: p.label })}>
                      <og-icon class="currency" icon={p.icon}></og-icon>
                      {#if p.was}<span part="Shop.former" class="was" aria-label={t("general.shop.was", { price: p.was })}>{p.was}</span>{/if}
                      <span class="figure">{p.amount}</span>
                    </div>
                  {/if}
                {/each}
              </div>
              {#if o.soldOut}<div part="Shop.status Shop.label" data-layout="row" data-gap="hair" data-kind="soldOut"><og-icon class="status-mark" icon="close"></og-icon><span>{t("general.shop.soldOut")}</span></div>{/if}
              {#if o.owned}<div part="Shop.status Shop.label" data-layout="row" data-gap="hair" data-kind="owned"><og-icon class="status-mark" icon="check"></og-icon><span>{t("general.shop.owned")}</span></div>{/if}
              {#if !o.affordable && !o.soldOut && !o.owned}<div part="Shop.status Shop.label" data-layout="row" data-gap="hair" data-kind="short"><og-icon class="status-mark" icon="lock"></og-icon><span>{t("general.shop.short")}</span></div>{/if}
            </div></button>
          {/if}
        {/each}
      </div>
    </div>
  {:else}
    <div class="state" data-layout="cover" data-fill role="status"><og-icon class="state-mark" icon="bag"></og-icon><p class="state-text">{t("general.shop.empty")}</p></div>
  {/if}
</section>

<style>
[part="Shop.band"] { width: stretch; flex: none; }
[part="Shop.title"] { margin: 0; }
[part="Shop.wallet"] { margin: 0; padding: 0; list-style: none; justify-content: flex-end; }
[part="Shop.balance Shop.figure"] { padding: var(--space-xxs) var(--space-sm); }
.currency { width: var(--size-xs); height: var(--size-xs); flex: none; }
.figure { white-space: nowrap; }
.tabs { width: stretch; flex: none; }
.tabs-scroll { width: stretch; overflow-x: auto; overflow-y: hidden; min-width: 0; }
[part="Shop.tab"] { flex: none; min-height: var(--size-touch); padding: 0 var(--space-md); }
.grid { --layout-tile: calc(var(--size-xxl) * 2.6); width: stretch; padding: var(--space-xs); }
[part="Shop.offer Shop.cell"] { position: relative; width: stretch; height: stretch; min-height: calc(var(--size-xxl) * 2.2); padding: var(--space-sm); align-items: stretch; justify-content: flex-start; min-width: 0; }
[part="Shop.offer Shop.cell"][data-featured=true] { grid-column: span 2; }
[part="Shop.offer Shop.cell"] .face { width: stretch; height: stretch; }
.art { min-height: calc(var(--size-xl) * 1.5); justify-content: center; }
.art-icon { width: var(--size-xl); height: var(--size-xl); }
.tags { width: stretch; flex: none; }
[part="Shop.badge"] { flex: 0 1 auto; min-width: 0; max-width: 60%; margin: 0; margin-inline-start: auto; padding: var(--space-xxs) var(--space-xs); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.featured { flex: 0 1 auto; min-width: 0; overflow: hidden; text-overflow: ellipsis; padding: var(--space-xxs) var(--space-xs); white-space: nowrap; }
.text { width: stretch; }
[part~="Shop.name"] { margin: 0; text-align: start; overflow-wrap: anywhere; }
.detail { text-align: start; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.prices { width: stretch; }
[part="Shop.price Shop.figure"] { min-width: 0; }
.was { white-space: nowrap; }
[part="Shop.status Shop.label"] { width: stretch; text-align: start; }
.status-mark { width: var(--size-xs); height: var(--size-xs); flex: none; }
.state { width: stretch; padding: var(--space-lg); }
.state-mark { width: var(--size-lg); height: var(--size-lg); }
.state-text { margin: 0; text-align: center; }
#Retry { min-height: var(--size-touch); }
@container (width < 480px) {
  .grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
}
</style>
