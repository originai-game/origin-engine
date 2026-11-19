<meta name="component" content="ShopConfirm">
<meta name="strings" content="./general.en.ogt">
<meta name="parts" content="ShopConfirm.frame ShopConfirm.band ShopConfirm.title ShopConfirm.name ShopConfirm.art ShopConfirm.detail ShopConfirm.ledger ShopConfirm.price ShopConfirm.former ShopConfirm.after ShopConfirm.missing ShopConfirm.reason ShopConfirm.error ShopConfirm.actions ShopConfirm.figure ShopConfirm.caption">
<meta name="prop:title" content="string" data-doc="The panel's heading, a small line over the offer's name; empty takes the group's (Confirm purchase).">
<meta name="prop:name" content="string" data-doc="The offer's name.">
<meta name="prop:icon" content="string" data-doc="The offer's icon in the style's or the game's atlas.">
<meta name="prop:rarity" content="string" data-default="" data-doc="common, uncommon, rare, epic, legendary or mythic: the socket the icon sits in; empty is common.">
<meta name="prop:detail" content="string" data-doc="What the offer gives, as the game words it; empty hides it.">
<meta name="prop:prices" content="list" data-doc="What it costs, one group of rows per currency: keyed rows of { id, icon, label, amount, was, after, short, missing }: amount, was (struck, empty none) and after (the balance before and after, 3,200 → 2,500; empty hides it) as the game formats them; label the currency's name; short marks a currency the player lacks, and missing how much more it takes (1,600), shown in place of the balance.">
<meta name="prop:canBuy" content="bool" data-doc="The game allows the purchase; false disables Buy.">
<meta name="prop:reason" content="string" data-default="" data-doc="Why the purchase is refused when no price says it, as the game words it (Requires level 40); shown over the buttons while canBuy is false.">
<meta name="prop:busy" content="bool" data-doc="The purchase is in flight: Buy is disabled and says it is working.">
<meta name="prop:error" content="string" data-doc="Why the last attempt failed, as the game words it; empty is none.">
<meta name="action:back" content="Cancel">
<meta name="fixture:default" content='{"title":"","name":"Founder Bundle","icon":"crown","rarity":"legendary","detail":"3 items and 500 gems","prices":[{"id":"gems","icon":"gem","label":"Gems","amount":"700","was":"1,000","after":"3,200 → 2,500","short":false,"missing":""}],"canBuy":true,"reason":"","busy":false,"error":""}'>
<meta name="fixture:disabled" content='{"title":"","name":"Dragonscale Shield","icon":"shield","rarity":"legendary","detail":"A shield for the long roads","prices":[{"id":"gems","icon":"gem","label":"Gems","amount":"4,800","was":"","after":"","short":true,"missing":"1,600"},{"id":"gold","icon":"coin","label":"Gold","amount":"10,000","was":"","after":"12,450 → 2,450","short":false,"missing":""}],"canBuy":false,"reason":"","busy":false,"error":""}'>
<meta name="fixture:refused" content='{"title":"","name":"Vault Key","icon":"key","rarity":"rare","detail":"×1","prices":[{"id":"gold","icon":"coin","label":"Gold","amount":"5,000","was":"","after":"12,450 → 7,450","short":false,"missing":""}],"canBuy":false,"reason":"Requires level 40","busy":false,"error":""}'>
<meta name="fixture:loading" content='{"title":"","name":"Potion Pack","icon":"potion","rarity":"uncommon","detail":"×10","prices":[{"id":"gold","icon":"coin","label":"Gold","amount":"1,200","was":"","after":"12,450 → 11,250","short":false,"missing":""}],"canBuy":true,"reason":"","busy":true,"error":""}'>
<meta name="fixture:error" content='{"title":"","name":"Potion Pack","icon":"potion","rarity":"uncommon","detail":"×10","prices":[{"id":"gold","icon":"coin","label":"Gold","amount":"1,200","was":"","after":"12,450 → 11,250","short":false,"missing":""},{"id":"gems","icon":"gem","label":"Gems","amount":"40","was":"","after":"3,200 → 3,160","short":false,"missing":""}],"canBuy":true,"reason":"","busy":false,"error":"The purchase did not go through. Nothing was charged."}'>
<meta name="fixture:overflow" content='{"title":"Confirm your purchase from the Wandering Merchant","name":"The Founder Bundle of the Far Eastern Reaches, Collector Edition","icon":"crown","rarity":"mythic","detail":"Three legendary items, five hundred gems and a title that everyone on the server will see","prices":[{"id":"gold","icon":"coin","label":"Gold","amount":"123,456,789","was":"200,000,000","after":"1,234,567,890 → 1,111,111,101","short":false,"missing":""},{"id":"tokens","icon":"star","label":"Tokens of the Eastern Treasury","amount":"99","was":"","after":"","short":true,"missing":"87"}],"canBuy":false,"reason":"","busy":false,"error":""}'>

<!--
  The purchase confirmation the game shows after a shop's buy: a small
  heading over the offer's name, its art in a socket of its rarity beside
  its detail, and a ledger with one group per currency: the price (a
  discount's old price struck before it, and named: 700 Gems, was 1,000),
  then the balance before and after, or, for a currency the player lacks,
  Not enough and how much more it takes. The game decides whether it may
  be bought and what it costs; the panel reports confirm (Buy) and cancel
  (Cancel, or Escape and pad B anywhere in it). Enter and pad A press the
  focused button, Cancel first, so no key buys by accident. A refused
  purchase disables Buy and says why (the short currency's row, or the
  game's reason over the buttons); a failed one shows the game's error on
  a strip and leaves Buy to try again. While busy Buy is disabled. The
  ledger scrolls when it must; the buttons stay at the foot. It is
  content, not a dialog: the host that shows it (the built-in confirm
  screen, a sheet at the foot of a phone, or the game's own modal) owns
  modality and focus.
-->
<section id="ShopConfirm" part="ShopConfirm.frame" data-layout="shell" data-rarity={rarity || 'common'} aria-label={title != '' ? title : t("general.confirm.name")} aria-busy={busy} onback={cancel}>
  <div class="lit"></div>
  <header part="ShopConfirm.band" data-layout="stack" data-gap="hair">
    <h2 part="ShopConfirm.title" class="kicker">{title != '' ? title : t("general.confirm.name")}</h2>
    <h3 part="ShopConfirm.name" class="name">{name}</h3>
  </header>
  <div class="body" data-layout="scroll" data-gap="loose">
    <div class="offer" data-layout="row" data-gap="loose">
      <div part="ShopConfirm.art" class="socket" data-layout="cover"><og-icon class="art-icon" icon={icon}></og-icon></div>
      {#if detail}<p part="ShopConfirm.detail" class="detail" data-fill>{detail}</p>{/if}
    </div>
    <ul part="ShopConfirm.ledger" class="ledger" aria-label={t("general.confirm.cost")}>
      {#each prices as p (p.id)}
        <li class="entry" data-layout="stack" data-gap="tight" data-short={p.short}>
          <div class="line" data-layout="row" data-gap="loose">
            <span part="ShopConfirm.caption" class="key" data-fill>{t("general.shop.price")}</span>
            <div part="ShopConfirm.price" class="price" data-layout="row" data-gap="hair">
              <og-icon class="currency" icon={p.icon}></og-icon>
              {#if p.was}<span part="ShopConfirm.former" class="was">{p.was}</span>{/if}
              <span part="ShopConfirm.figure" class="figure" aria-label={p.was ? t("general.shop.amountWas", { amount: p.amount, currency: p.label, was: p.was }) : t("general.shop.amount", { amount: p.amount, currency: p.label })}>{p.amount}</span>
            </div>
          </div>
          {#if p.short}
            <div part="ShopConfirm.missing" class="line lack" data-layout="row" data-gap="loose" aria-label={p.missing ? t("general.confirm.missingSaid", { amount: p.missing, currency: p.label }) : t("general.confirm.shortOf", { currency: p.label })}>
              <span class="key" data-fill>{t("general.confirm.shortOf", { currency: p.label })}</span>
              {#if p.missing}<span class="more">{t("general.confirm.missing", { amount: p.missing })}</span>{/if}
            </div>
          {:else if p.after}
            <div part="ShopConfirm.after" class="line" data-layout="row" data-gap="loose" aria-label={t("general.confirm.afterSaid", { currency: p.label, after: p.after })}>
              <span part="ShopConfirm.caption" class="key" data-fill>{t("general.confirm.balance")}</span>
              <span class="balance">{p.after}</span>
            </div>
          {/if}
        </li>
      {/each}
    </ul>
  </div>
  {#if error}<p part="ShopConfirm.error" class="alert" role="alert">{error}</p>{/if}
  <footer part="ShopConfirm.actions" class="actions" data-layout="stack" data-gap="tight">
    {#if !canBuy && !busy && reason}<p part="ShopConfirm.reason" class="reason" role="status">{reason}</p>{/if}
    <div class="buttons" data-layout="row" data-gap="tight">
      <button id="Cancel" class="act" variant="ghost" aria-label={t("general.confirm.cancel")} onclick={cancel}><kbd class="cap" data-action="back"></kbd><span>{t("general.confirm.cancel")}</span></button>
      <button id="Buy" class="act main" variant="primary" disabled={!canBuy || busy} aria-busy={busy} aria-label={busy ? t("general.confirm.busy") : t("general.confirm.buy")} onclick={confirm}><span>{busy ? t("general.confirm.busy") : t("general.confirm.buy")}</span></button>
    </div>
  </footer>
</section>

<style>
:host { inline-size: stretch; max-inline-size: var(--modal-width); max-block-size: 100%; min-block-size: 0; }
/* The frame: two corners cut, a lit top edge; the rarity dresses the socket. */
[part="ShopConfirm.frame"] { --inv-rarity: var(--common); --inv-socket-fill: var(--cell-common-fill); --inv-socket-rim: var(--cell-common-shadow); position: relative; min-block-size: 0; clip-path: polygon(var(--inv-frame-cut) 0, 100% 0, 100% calc(100% - var(--inv-frame-cut)), calc(100% - var(--inv-frame-cut)) 100%, 0 100%, 0 var(--inv-frame-cut)); }
[part="ShopConfirm.frame"][data-rarity=uncommon] { --inv-rarity: var(--uncommon); --inv-socket-fill: var(--cell-uncommon-fill); --inv-socket-rim: var(--cell-uncommon-shadow); }
[part="ShopConfirm.frame"][data-rarity=rare] { --inv-rarity: var(--rare); --inv-socket-fill: var(--cell-rare-fill); --inv-socket-rim: var(--cell-rare-shadow); }
[part="ShopConfirm.frame"][data-rarity=epic] { --inv-rarity: var(--epic); --inv-socket-fill: var(--cell-epic-fill); --inv-socket-rim: var(--cell-epic-shadow); }
[part="ShopConfirm.frame"][data-rarity=legendary] { --inv-rarity: var(--legendary); --inv-socket-fill: var(--cell-legendary-fill); --inv-socket-rim: var(--cell-legendary-shadow); }
[part="ShopConfirm.frame"][data-rarity=mythic] { --inv-rarity: var(--mythic); --inv-socket-fill: var(--cell-mythic-fill); --inv-socket-rim: var(--cell-mythic-shadow); }
.lit { position: absolute; inset-inline: var(--inv-frame-cut) 30%; top: 0; block-size: var(--stroke-hairline); background: var(--inv-lit-edge); pointer-events: none; }
h2, h3, p { margin: 0; }
[part="ShopConfirm.band"] { width: stretch; flex: none; align-items: flex-start; justify-content: flex-start; text-align: start; }
.kicker { font-size: var(--inv-kicker-size); font-weight: var(--inv-name-weight); letter-spacing: var(--inv-kicker-tracking); text-transform: var(--inv-kicker-transform); color: var(--inv-kicker-ink); overflow-wrap: anywhere; }
.name { font-size: var(--inv-figure-xl-size); font-weight: var(--inv-figure-weight); color: var(--ink-bright); overflow-wrap: anywhere; line-clamp: 3; overflow: hidden; }
.body { flex: 0 1 auto; min-block-size: 0; width: stretch; }
.offer { flex: none; width: stretch; align-items: center; }
.socket { flex: none; inline-size: var(--shop-socket-size); block-size: var(--shop-socket-size); background: var(--inv-socket-fill); box-shadow: var(--inv-socket-rim); clip-path: polygon(var(--inv-slot-cut) 0, 100% 0, 100% calc(100% - var(--inv-slot-cut)), calc(100% - var(--inv-slot-cut)) 100%, 0 100%, 0 var(--inv-slot-cut)); }
.art-icon { inline-size: 74%; block-size: 74%; filter: var(--inv-slot-art-shadow); }
.detail { min-width: 0; overflow-wrap: anywhere; color: var(--ink-label); }
/* The ledger: a sunk well, one group of rows a currency, a rule between. */
.ledger { flex: none; display: flex; flex-direction: column; width: stretch; margin: 0; padding: var(--space-xs) var(--space-sm); list-style: none; background: var(--shop-ledger-fill); box-shadow: var(--shop-ledger-rim); }
.entry { width: stretch; padding-block: var(--space-xs); }
.entry + .entry { border-block-start: var(--shop-foot-rule); }
.line { width: stretch; align-items: center; flex-wrap: wrap; row-gap: var(--space-xxs); }
.key { min-width: 0; font-size: var(--inv-kicker-size); font-weight: var(--inv-name-weight); letter-spacing: var(--inv-kicker-tracking); text-transform: var(--inv-kicker-transform); color: var(--inv-kicker-ink); overflow-wrap: anywhere; }
.price { flex: none; max-width: stretch; min-width: 0; margin-inline-start: auto; align-items: center; font-size: var(--shop-price-lg-size); color: var(--ink-bright); }
.figure, .balance, .more { font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); }
.figure { white-space: nowrap; }
.currency { flex: none; inline-size: calc(20 * var(--font-step-px)); block-size: calc(20 * var(--font-step-px)); }
.was { white-space: nowrap; font-size: var(--shop-was-size); font-weight: var(--inv-name-weight); }
.balance, .more { flex: none; max-width: stretch; margin-inline-start: auto; text-align: end; overflow-wrap: anywhere; }
[data-short=true] .price, .lack .key, .lack .more { color: var(--shop-short-ink); }
.alert { flex: none; width: stretch; padding: var(--space-xs) var(--space-sm); overflow-wrap: anywhere; background: var(--inv-alert-fill); box-shadow: var(--inv-alert-rim); }
/* The buttons at the foot, the refusal in words over them. */
.actions { flex: none; width: stretch; }
.reason { color: var(--shop-short-ink); overflow-wrap: anywhere; }
.buttons { width: stretch; }
.act { flex: 1 1 0; min-width: 0; min-block-size: calc(var(--size-touch) * 1.15); gap: var(--space-xs); }
.main { flex-grow: 1.4; }
.cap { flex: none; }
/* A phone: a sheet, square at its foot, the socket smaller. */
@media (width < 600px) {
  [part="ShopConfirm.frame"] { clip-path: polygon(var(--inv-frame-cut) 0, 100% 0, 100% 100%, 0 100%, 0 var(--inv-frame-cut)); }
  .socket { inline-size: var(--shop-socket-compact-size); block-size: var(--shop-socket-compact-size); }
  .main { flex-grow: 1; }
}
</style>
