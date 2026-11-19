<meta name="component" content="Shop">
<meta name="strings" content="./general.en.ogt">
<meta name="parts" content="Shop.frame Shop.band Shop.kicker Shop.title Shop.wallet Shop.balance Shop.tab Shop.count Shop.offer Shop.art Shop.name Shop.detail Shop.featured Shop.badge Shop.foot Shop.price Shop.former Shop.status Shop.bracket Shop.figure Shop.caption">
<meta name="prop:title" content="string" data-doc="The shop's heading; empty takes the group's (Shop).">
<meta name="prop:kicker" content="string" data-default="" data-doc="A small line over the title naming the place (Store, a merchant's town); empty hides it.">
<meta name="prop:wallet" content="list" data-doc="The player's balances shown in the band, keyed rows of { id, icon, label, amount }: amount as the game formats it; label names the currency for a screen reader.">
<meta name="prop:tabs" content="list" data-doc="The categories, keyed rows of { id, label, count }: count, how many offers the category holds, is shown beside its label above 0; empty shows no tab row.">
<meta name="prop:tab" content="string" data-doc="The category shown.">
<meta name="prop:offers" content="list" data-doc="The offers in the shown category, in the game's order: keyed rows of { id, name, icon, detail, rarity, badge, badgeTone, featured, soldOut, owned, affordable }: detail a second line (×500, 7 days); rarity common, uncommon, rare, epic, legendary or mythic, the light behind the art (empty is common); badge a corner tag the game words (−30%, New), empty none, on gold unless badgeTone is success; featured a wide tile; soldOut, owned and affordable the game's verdicts: a sold out, owned or unaffordable (affordable false) tile keeps the focus, is marked disabled, says why in words on its foot and reports nothing.">
<meta name="prop:prices" content="list" data-doc="Every offer's price, one row per currency: keyed rows of { id, offer, icon, label, amount, was, short }: offer an offer's id; amount and was (the struck price before a discount, empty none) as the game formats them; label the currency's name; short marks the currency the player lacks.">
<meta name="prop:selected" content="string" data-doc="The offer the focus rests on; empty is none.">
<meta name="prop:loading" content="bool" data-doc="The catalogue is on its way: a spinner stands in.">
<meta name="prop:error" content="string" data-doc="Why the catalogue could not load, as the game words it; empty is none. Shows a Try again button.">
<meta name="action:tabPrev" content="Previous category">
<meta name="action:tabNext" content="Next category">
<meta name="fixture:default" content='{"title":"","kicker":"Store","wallet":[{"id":"gold","icon":"coin","label":"Gold","amount":"12,450"},{"id":"gems","icon":"gem","label":"Gems","amount":"3,200"}],"tabs":[{"id":"featured","label":"Featured","count":3},{"id":"items","label":"Items","count":12},{"id":"looks","label":"Looks","count":8},{"id":"bundles","label":"Bundles","count":2}],"tab":"featured","offers":[{"id":"bundle","name":"Founder Bundle","icon":"crown","detail":"3 items, 500 gems","rarity":"legendary","badge":"−30%","badgeTone":"","featured":true,"soldOut":false,"owned":false,"affordable":true},{"id":"potion","name":"Potion Pack","icon":"potion","detail":"×10","rarity":"uncommon","badge":"","badgeTone":"","featured":false,"soldOut":false,"owned":false,"affordable":true},{"id":"cape","name":"Ember Cape","icon":"fire","detail":"Look","rarity":"epic","badge":"New","badgeTone":"success","featured":false,"soldOut":false,"owned":true,"affordable":true},{"id":"key","name":"Vault Key","icon":"key","detail":"×1","rarity":"rare","badge":"","badgeTone":"","featured":false,"soldOut":true,"owned":false,"affordable":true},{"id":"shield","name":"Dragonscale Shield","icon":"shield","detail":"Shield","rarity":"legendary","badge":"","badgeTone":"","featured":false,"soldOut":false,"owned":false,"affordable":false},{"id":"map","name":"Map of the Reaches","icon":"map","detail":"Reveals the east","rarity":"common","badge":"","badgeTone":"","featured":false,"soldOut":false,"owned":false,"affordable":true},{"id":"band","name":"Moonlit Band","icon":"gem","detail":"Ring · +4 Intellect","rarity":"rare","badge":"","badgeTone":"","featured":false,"soldOut":false,"owned":false,"affordable":true}],"prices":[{"id":"bundle-gems","offer":"bundle","icon":"gem","label":"Gems","amount":"700","was":"1,000","short":false},{"id":"potion-gold","offer":"potion","icon":"coin","label":"Gold","amount":"1,200","was":"","short":false},{"id":"potion-gems","offer":"potion","icon":"gem","label":"Gems","amount":"40","was":"","short":false},{"id":"cape-gems","offer":"cape","icon":"gem","label":"Gems","amount":"250","was":"","short":false},{"id":"key-gold","offer":"key","icon":"coin","label":"Gold","amount":"5,000","was":"","short":false},{"id":"shield-gems","offer":"shield","icon":"gem","label":"Gems","amount":"4,800","was":"","short":true},{"id":"map-gold","offer":"map","icon":"coin","label":"Gold","amount":"800","was":"","short":false},{"id":"band-gold","offer":"band","icon":"coin","label":"Gold","amount":"3,250","was":"","short":false}],"selected":"bundle","loading":false,"error":""}'>
<meta name="fixture:empty" content='{"title":"","wallet":[{"id":"gold","icon":"coin","label":"Gold","amount":"0"}],"tabs":[{"id":"items","label":"Items","count":0}],"tab":"items","offers":[],"prices":[],"selected":"","loading":false,"error":"","kicker":""}'>
<meta name="fixture:loading" content='{"title":"","wallet":[],"tabs":[],"tab":"","offers":[],"prices":[],"selected":"","loading":true,"error":"","kicker":""}'>
<meta name="fixture:error" content='{"title":"","wallet":[],"tabs":[],"tab":"","offers":[],"prices":[],"selected":"","loading":false,"error":"The shop could not be reached.","kicker":""}'>
<meta name="fixture:overflow" content='{"title":"The Wandering Merchant of the Far Eastern Reaches","wallet":[{"id":"gold","icon":"coin","label":"Gold","amount":"1,234,567,890"},{"id":"gems","icon":"gem","label":"Gems","amount":"99,999"},{"id":"tokens","icon":"star","label":"Tokens","amount":"12"}],"tabs":[{"id":"a","label":"Featured","count":0},{"id":"b","label":"Consumables","count":0},{"id":"c","label":"Equipment","count":0},{"id":"d","label":"Cosmetics","count":0},{"id":"e","label":"Bundles","count":0},{"id":"f","label":"Seasonal","count":0}],"tab":"b","offers":[{"id":"o1","name":"An Extraordinarily Long Offer Name That Must Wrap","icon":"potion","detail":"x10 with a long detail line","badge":"Limited time","featured":false,"soldOut":false,"owned":false,"affordable":true,"rarity":"rare","badgeTone":""},{"id":"o2","name":"Potion","icon":"potion","detail":"x1","badge":"","featured":false,"soldOut":false,"owned":false,"affordable":false,"rarity":"common","badgeTone":""},{"id":"o3","name":"Elixir","icon":"drop","detail":"x5","badge":"","featured":false,"soldOut":false,"owned":false,"affordable":true,"rarity":"uncommon","badgeTone":""},{"id":"o4","name":"Bomb","icon":"bomb","detail":"x3","badge":"","featured":false,"soldOut":true,"owned":false,"affordable":true,"rarity":"epic","badgeTone":""},{"id":"o5","name":"Apple","icon":"apple","detail":"x20","badge":"","featured":false,"soldOut":false,"owned":false,"affordable":true,"rarity":"rare","badgeTone":""},{"id":"o6","name":"Leaf","icon":"leaf","detail":"x8","badge":"","featured":false,"soldOut":false,"owned":false,"affordable":true,"rarity":"common","badgeTone":""},{"id":"o7","name":"Bolt","icon":"bolt","detail":"x2","badge":"","featured":false,"soldOut":false,"owned":false,"affordable":true,"rarity":"uncommon","badgeTone":""},{"id":"o8","name":"Shield","icon":"shield","detail":"x1","badge":"","featured":false,"soldOut":false,"owned":false,"affordable":true,"rarity":"epic","badgeTone":""}],"prices":[{"id":"p1a","offer":"o1","icon":"coin","label":"Gold","amount":"123,456,789","was":"200,000,000","short":false},{"id":"p1b","offer":"o1","icon":"gem","label":"Gems","amount":"9,999","was":"","short":false},{"id":"p1c","offer":"o1","icon":"star","label":"Tokens","amount":"12","was":"","short":false},{"id":"p2","offer":"o2","icon":"star","label":"Tokens","amount":"99","was":"","short":true},{"id":"p3","offer":"o3","icon":"coin","label":"Gold","amount":"800","was":"","short":false},{"id":"p4","offer":"o4","icon":"coin","label":"Gold","amount":"300","was":"","short":false},{"id":"p5","offer":"o5","icon":"coin","label":"Gold","amount":"40","was":"","short":false},{"id":"p6","offer":"o6","icon":"coin","label":"Gold","amount":"90","was":"","short":false},{"id":"p7","offer":"o7","icon":"gem","label":"Gems","amount":"15","was":"","short":false},{"id":"p8","offer":"o8","icon":"coin","label":"Gold","amount":"2,000","was":"","short":false}],"selected":"o3","loading":false,"error":"","kicker":"The Far Eastern Reaches Trading Company"}'>

<!--
  A storefront: a band with a kicker over the title and the player's
  balances as chips, category tabs with their counts between the bumper
  caps, and a grid of offer tiles. A tile is its art in a well over a glow
  of its rarity, corner tags (Featured, and a badge the game words), its
  name and detail, and one ruled foot with its price in one or more
  currencies (a discount's old price struck before it, and named: 700
  Gems, was 1,000). The game owns the catalogue, every price and every
  verdict (sold out, owned, unaffordable); the shop reports buy (a tile's
  key, which the game answers with a ShopConfirm), choosetab (a category's
  key), tabNext and tabPrev (E/Q, bumpers) and retry. A tile the game
  refused has no buy handler: it keeps the focus so a player can read it,
  is marked disabled (aria-disabled), dims its art and says why in words
  on its foot. The selected tile wears corner brackets. The grid fills the
  height it is given and scrolls inside it; narrow, it is two columns and
  the tabs scroll sideways.
-->
<section id="Shop" part="Shop.frame" data-layout="shell" aria-label={title != '' ? title : t("general.shop.name")} aria-busy={loading}>
  <div class="lit"></div>
  <header part="Shop.band" data-layout="cluster" data-gap="loose">
    <div class="heading" data-layout="stack" data-gap="hair" data-fill>
      {#if kicker}<span part="Shop.kicker" class="kicker">{kicker}</span>{/if}
      <h2 part="Shop.title">{title != '' ? title : t("general.shop.name")}</h2>
    </div>
    {#if wallet.length}<ul part="Shop.wallet" data-layout="cluster" data-gap="tight" aria-label={t("general.shop.wallet")}>
      {#each wallet as w (w.id)}
        <li part="Shop.balance" class="chip" data-layout="row" data-gap="tight" aria-label={t("general.shop.amount", { amount: w.amount, currency: w.label })}><og-icon class="chip-icon" icon={w.icon}></og-icon><span part="Shop.figure" class="figure">{w.amount}</span></li>
      {/each}
    </ul>{/if}
  </header>
  {#if tabs.length}<div class="tabs" data-layout="row" data-gap="tight" role="tablist" aria-label={t("general.shop.tabs")} ontabNext={tabNext} ontabPrev={tabPrev}>
    <kbd class="cap" data-action="tabPrev"></kbd>
    <div class="tabs-scroll" data-fill>
      <div class="tabs-row" data-layout="row" data-gap="none">
        {#each tabs as c (c.id)}
          <button part="Shop.tab" variant="quiet" role="tab" aria-selected={c.id == tab} aria-label={c.label} onclick={choosetab}><span class="tab-label">{c.label}</span>{#if c.count > 0}<span part="Shop.count" class="count">{formatNumber(c.count)}</span>{/if}<span class="line"></span></button>
        {/each}
      </div>
    </div>
    <kbd class="cap" data-action="tabNext"></kbd>
  </div>{/if}
  {#if error != ''}
    <div class="state" data-layout="cover" data-fill role="alert"><og-icon class="state-mark" icon="close"></og-icon><p class="state-text">{error}</p><button id="Retry" class="secondary" onclick={retry}>{t("general.shop.retry")}</button></div>
  {:else if loading}
    <div class="state" data-layout="cover" data-fill role="status"><Spinner name={t("general.shop.loading")} /><p class="state-text">{t("general.shop.loading")}</p></div>
  {:else if offers.length}
    <div class="grid-scroll" data-layout="scroll" data-fill>
      <div class="grid" role="grid" aria-label={title != '' ? title : t("general.shop.name")}>
        {#each offers as o (o.id)}
          {#if !o.soldOut && !o.owned && o.affordable}
            <button part="Shop.offer" class="offer" role="gridcell" data-rarity={o.rarity || 'common'} data-featured={o.featured} data-sold-out={o.soldOut} data-owned={o.owned} data-affordable={o.affordable} aria-selected={o.id == selected} aria-label={o.name} onclick={buy}><div class="face" data-layout="stack" data-gap="tight">
              <div part="Shop.art" class="art" data-layout="cover"><og-icon class="art-icon" icon={o.icon}></og-icon></div>
              {#if o.featured}<span part="Shop.featured" class="ctag lead">{t("general.shop.featured")}</span>{/if}
              {#if o.badge}<span part="Shop.badge" class="ctag end" data-tone={o.badgeTone}>{o.badge}</span>{/if}
              <div class="text" data-layout="stack" data-gap="hair">
                <p part="Shop.name" class="name">{o.name}</p>
                {#if o.detail}<span part="Shop.detail Shop.caption" class="detail">{o.detail}</span>{/if}
              </div>
              <div part="Shop.foot" class="foot" data-layout="cluster" data-gap="tight" aria-label={t("general.shop.price")}>
                {#each prices as p (p.id)}
                  {#if p.offer == o.id}
                    <div part="Shop.price" class="price" data-layout="row" data-gap="hair" data-short={p.short}>
                      <og-icon class="currency" icon={p.icon}></og-icon>
                      {#if p.was}<span part="Shop.former" class="was">{p.was}</span>{/if}
                      <span part="Shop.figure" class="figure" aria-label={p.was ? t("general.shop.amountWas", { amount: p.amount, currency: p.label, was: p.was }) : t("general.shop.amount", { amount: p.amount, currency: p.label })}>{p.amount}</span>
                    </div>
                  {/if}
                {/each}
              </div>
            </div>{#if o.id == selected}<div part="Shop.bracket" class="bracket"></div>{/if}</button>
          {:else}
            <button part="Shop.offer" class="offer" role="gridcell" data-refused={true} data-rarity={o.rarity || 'common'} data-featured={o.featured} data-sold-out={o.soldOut} data-owned={o.owned} data-affordable={o.affordable} aria-selected={o.id == selected} aria-disabled={true} aria-label={t("general.shop.refused", { name: o.name, status: o.soldOut ? t("general.shop.soldOut") : o.owned ? t("general.shop.owned") : t("general.shop.short") })}><div class="face" data-layout="stack" data-gap="tight">
              <div part="Shop.art" class="art" data-layout="cover"><og-icon class="art-icon" icon={o.icon}></og-icon></div>
              {#if o.featured}<span part="Shop.featured" class="ctag lead">{t("general.shop.featured")}</span>{/if}
              {#if o.badge}<span part="Shop.badge" class="ctag end" data-tone={o.badgeTone}>{o.badge}</span>{/if}
              <div class="text" data-layout="stack" data-gap="hair">
                <p part="Shop.name" class="name">{o.name}</p>
                {#if o.detail}<span part="Shop.detail Shop.caption" class="detail">{o.detail}</span>{/if}
              </div>
              <div part="Shop.foot" class="foot" data-layout="cluster" data-gap="tight" aria-label={t("general.shop.price")}>
                {#each prices as p (p.id)}
                  {#if p.offer == o.id}
                    <div part="Shop.price" class="price" data-layout="row" data-gap="hair" data-short={p.short}>
                      <og-icon class="currency" icon={p.icon}></og-icon>
                      {#if p.was}<span part="Shop.former" class="was">{p.was}</span>{/if}
                      <span part="Shop.figure" class="figure" aria-label={p.was ? t("general.shop.amountWas", { amount: p.amount, currency: p.label, was: p.was }) : t("general.shop.amount", { amount: p.amount, currency: p.label })}>{p.amount}</span>
                    </div>
                  {/if}
                {/each}
                {#if o.soldOut}<div part="Shop.status" class="status" data-layout="row" data-gap="hair" data-kind="soldOut"><span>{t("general.shop.soldOut")}</span></div>{/if}
                {#if o.owned && !o.soldOut}<div part="Shop.status" class="status" data-layout="row" data-gap="hair" data-kind="owned"><og-icon class="status-mark" icon="check"></og-icon><span>{t("general.shop.owned")}</span></div>{/if}
                {#if !o.affordable && !o.soldOut && !o.owned}<div part="Shop.status" class="status" data-layout="row" data-gap="hair" data-kind="short"><span>{t("general.shop.short")}</span></div>{/if}
              </div>
            </div>{#if o.id == selected}<div part="Shop.bracket" class="bracket"></div>{/if}</button>
          {/if}
        {/each}
      </div>
    </div>
  {:else}
    <div class="state" data-layout="cover" data-fill role="status"><og-icon class="state-mark" icon="bag"></og-icon><p class="state-text">{t("general.shop.empty")}</p></div>
  {/if}
</section>

<style>
/* The frame: two corners cut, a lit top edge. */
[part="Shop.frame"] { position: relative; min-block-size: 0; clip-path: polygon(var(--inv-frame-cut) 0, 100% 0, 100% calc(100% - var(--inv-frame-cut)), calc(100% - var(--inv-frame-cut)) 100%, 0 100%, 0 var(--inv-frame-cut)); }
.lit { position: absolute; inset-inline: var(--inv-frame-cut) 30%; top: 0; block-size: var(--stroke-hairline); background: var(--inv-lit-edge); pointer-events: none; }
[part="Shop.band"] { width: stretch; flex: none; align-items: flex-end; }
.heading { min-width: 0; flex-basis: calc(120 * var(--font-step-px)); }
.kicker { font-size: var(--inv-kicker-size); font-weight: var(--inv-name-weight); letter-spacing: var(--inv-kicker-tracking); text-transform: var(--inv-kicker-transform); color: var(--inv-kicker-ink); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
[part="Shop.title"] { margin: 0; min-width: 0; overflow-wrap: anywhere; line-clamp: 2; overflow: hidden; }
/* The balances: a chip each, the currency's mark before its amount. */
[part="Shop.wallet"] { margin: 0; padding: 0; list-style: none; justify-content: flex-end; max-width: stretch; }
.chip { flex: none; max-width: stretch; align-items: center; padding: var(--space-xxs) var(--space-sm) var(--space-xxs) var(--space-xs); border-radius: var(--inv-chip-radius); background: var(--inv-chip-fill); box-shadow: var(--inv-chip-rim); }
.chip-icon { flex: none; inline-size: calc(20 * var(--font-step-px)); block-size: calc(20 * var(--font-step-px)); }
.figure { white-space: nowrap; font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); }
.chip .figure { min-width: 0; overflow: hidden; text-overflow: ellipsis; }
/* Tabs: an underline strip between the bumper caps; the strip scrolls and
   fades out before the cap, so no label sits under or past it. */
.tabs { width: stretch; flex: none; align-items: center; border-block-end: var(--inv-rule); }
.cap { flex: none; }
.tabs-scroll { min-width: 0; overflow-x: auto; overflow-y: hidden; scrollbar-width: none; mask-image: var(--inv-tab-fade); }
.tabs-row { padding-inline-end: var(--size-lg); }
[part="Shop.tab"] { position: relative; flex: none; min-height: var(--size-touch); min-width: var(--size-touch); padding: 0 var(--space-sm); gap: var(--space-xs); border-radius: 0; background: var(--inv-bare-fill); box-shadow: none; color: var(--inv-tab-ink); }
[part="Shop.tab"][aria-selected=true] { color: var(--inv-tab-on-ink); }
.line { position: absolute; inset-inline: var(--space-xs); bottom: 0; block-size: var(--stroke-thick); background: var(--inv-tab-line); box-shadow: var(--inv-tab-glow); opacity: 0; }
[aria-selected=true] > .line { opacity: 1; }
.tab-label { white-space: nowrap; }
.count { padding: 0 var(--space-xs); border-radius: var(--inv-chip-radius); font-size: var(--inv-tag-size); font-weight: var(--inv-figure-weight); background: var(--inv-tab-count-fill); color: var(--inv-kicker-ink); }
[aria-selected=true] .count { background: var(--inv-tab-count-on-fill); color: var(--inv-accent-ink); }
/* The grid: as many columns as fit, a featured tile two of them. */
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(var(--shop-tile-min-width), 1fr)); gap: var(--space-xs); width: stretch; padding: var(--space-xs); }
/* A tile: cut at two corners, its rarity the light behind its art. */
.offer { --inv-rarity: var(--common); position: relative; width: stretch; height: stretch; min-width: 0; padding: var(--space-xs) var(--space-xs) var(--space-sm); align-items: stretch; justify-content: flex-start; border: none; border-radius: 0; background: var(--shop-tile-fill); box-shadow: var(--shop-tile-rim); clip-path: polygon(var(--shop-tile-cut) 0, 100% 0, 100% calc(100% - var(--shop-tile-cut)), calc(100% - var(--shop-tile-cut)) 100%, 0 100%, 0 var(--shop-tile-cut)); }
.offer:hover, .offer:active { background: var(--shop-tile-fill); box-shadow: var(--shop-tile-hover-rim); }
.offer[aria-selected=true] { box-shadow: var(--shop-tile-selected-rim); }
.offer[data-featured=true] { grid-column: span 2; }
.offer[data-rarity=uncommon] { --inv-rarity: var(--uncommon); }
.offer[data-rarity=rare] { --inv-rarity: var(--rare); }
.offer[data-rarity=epic] { --inv-rarity: var(--epic); }
.offer[data-rarity=legendary] { --inv-rarity: var(--legendary); }
.offer[data-rarity=mythic] { --inv-rarity: var(--mythic); }
.face { width: stretch; min-width: 0; }
.art { position: relative; flex: none; width: stretch; block-size: var(--shop-art-height); color: var(--inv-rarity); background: var(--shop-art-fill); clip-path: polygon(var(--inv-slot-cut) 0, 100% 0, 100% calc(100% - var(--inv-slot-cut)), calc(100% - var(--inv-slot-cut)) 100%, 0 100%, 0 var(--inv-slot-cut)); }
.art::before { content: ""; position: absolute; inset: 0; background: var(--shop-art-glow); opacity: var(--shop-art-glow-opacity); }
.art-icon { position: relative; inline-size: var(--shop-art-size); block-size: var(--shop-art-size); filter: var(--inv-slot-art-shadow); }
[data-refused=true] .art-icon { filter: var(--shop-art-refused); }
/* Corner tags over the art: Featured at the leading corner, the game's
   badge at the other, each ending in an ellipsis before it meets the other. */
.ctag { position: absolute; top: var(--space-sm); max-inline-size: calc(50% - var(--space-md)); padding: var(--space-xxs) var(--space-xs); font-size: var(--inv-tag-size); font-weight: var(--inv-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--inv-kicker-transform); line-height: 1.2; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; clip-path: polygon(var(--inv-tag-cut) 0, 100% 0, 100% calc(100% - var(--inv-tag-cut)), calc(100% - var(--inv-tag-cut)) 100%, 0 100%, 0 var(--inv-tag-cut)); }
.ctag.lead { inset-inline-start: var(--space-sm); color: var(--inv-accent-ink); background: var(--inv-accent); }
.ctag.end { inset-inline-end: var(--space-sm); border-radius: 0; box-shadow: none; color: var(--shop-badge-ink); background: var(--shop-badge-fill); }
.ctag.end[data-tone=success] { color: var(--inv-equipped-ink); background: var(--inv-equipped-fill); }
.offer:not([data-featured=true]) .ctag.end { max-inline-size: calc(100% - var(--space-lg)); }
.text { width: stretch; min-width: 0; }
.name { margin: 0; text-align: start; overflow-wrap: anywhere; font-size: var(--shop-tile-name-size); font-weight: var(--inv-figure-weight); line-height: 1.2; color: var(--ink-bright); }
[data-refused=true] .name { color: var(--ink-label); }
.detail { text-align: start; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
/* One ruled foot: the price, then why the game refuses it. */
.foot { width: stretch; margin-block-start: auto; padding-block-start: var(--space-xs); align-items: center; border-block-start: var(--shop-foot-rule); }
.price { flex: none; max-width: stretch; min-width: 0; align-items: center; font-size: var(--shop-price-size); color: var(--ink-bright); }
.price[data-short=true] { color: var(--shop-short-ink); }
.price .figure { min-width: 0; overflow: hidden; text-overflow: ellipsis; }
.currency { flex: none; inline-size: calc(18 * var(--font-step-px)); block-size: calc(18 * var(--font-step-px)); }
.was { margin-inline-end: var(--space-xs); white-space: nowrap; font-size: var(--shop-was-size); font-weight: var(--inv-name-weight); }
.status { flex: none; margin-inline-start: auto; align-items: center; padding: var(--space-xxs) var(--space-xs); font-size: var(--inv-tag-size); font-weight: var(--inv-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--inv-kicker-transform); white-space: nowrap; color: var(--shop-sold-ink); background: var(--shop-sold-fill); clip-path: polygon(var(--inv-tag-cut) 0, 100% 0, 100% calc(100% - var(--inv-tag-cut)), calc(100% - var(--inv-tag-cut)) 100%, 0 100%, 0 var(--inv-tag-cut)); }
.status[data-kind=owned] { color: var(--shop-owned-ink); background: var(--shop-owned-fill); }
.status[data-kind=short] { color: var(--shop-short-ink); background: var(--shop-short-fill); }
.status-mark { flex: none; inline-size: var(--inv-tag-size); block-size: var(--inv-tag-size); }
.bracket { position: absolute; inset: 0; background: var(--inv-bracket); filter: var(--inv-bracket-glow); pointer-events: none; }
.state { width: stretch; padding: var(--space-lg); }
.state-mark { width: var(--size-lg); height: var(--size-lg); }
.state-text { margin: 0; text-align: center; }
#Retry { min-height: var(--size-touch); }
/* Narrow: the balances under the title, two columns of tiles. */
@container (width < 480px) {
  .heading { flex-basis: 100%; }
  [part="Shop.wallet"] { justify-content: flex-start; }
  .grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .art { block-size: calc(var(--shop-art-height) * 0.62); }
  .art-icon { inline-size: calc(var(--shop-art-size) * 0.7); block-size: calc(var(--shop-art-size) * 0.7); }
}
</style>
