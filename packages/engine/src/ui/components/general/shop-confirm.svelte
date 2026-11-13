<meta name="component" content="ShopConfirm">
<meta name="strings" content="./general.en.ogt">
<meta name="parts" content="ShopConfirm.frame ShopConfirm.band ShopConfirm.title ShopConfirm.name ShopConfirm.price ShopConfirm.error">
<meta name="prop:title" content="string" data-doc="The panel's heading; empty takes the group's (Confirm purchase).">
<meta name="prop:name" content="string" data-doc="The offer's name.">
<meta name="prop:icon" content="string" data-doc="The offer's icon in the style's atlas.">
<meta name="prop:detail" content="string" data-doc="What the offer gives, as the game words it; empty hides it.">
<meta name="prop:prices" content="list" data-doc="What it costs, one row per currency: keyed rows of { id, icon, label, amount, was, after, short }: amount, was (struck, empty none) and after (the balance left, empty hides it) as the game formats them; label the currency's name; short marks a currency the player lacks.">
<meta name="prop:canBuy" content="bool" data-doc="The game allows the purchase; false disables Buy.">
<meta name="prop:busy" content="bool" data-doc="The purchase is in flight: Buy is disabled and says it is working.">
<meta name="prop:error" content="string" data-doc="Why the last attempt failed, as the game words it; empty is none.">
<meta name="action:back" content="Cancel">
<meta name="fixture:default" content='{"title":"","name":"Founder Bundle","icon":"crown","detail":"3 items and 500 gems","prices":[{"id":"gems","icon":"gem","label":"Gems","amount":"700","was":"1,000","after":"320 → 0","short":false}],"canBuy":true,"busy":false,"error":""}'>
<meta name="fixture:disabled" content='{"title":"","name":"Dragon Mount","icon":"star","detail":"","prices":[{"id":"gems","icon":"gem","label":"Gems","amount":"2,400","was":"","after":"","short":true},{"id":"gold","icon":"coin","label":"Gold","amount":"10,000","was":"","after":"12,450 → 2,450","short":false}],"canBuy":false,"busy":false,"error":""}'>
<meta name="fixture:loading" content='{"title":"","name":"Potion Pack","icon":"potion","detail":"x10","prices":[{"id":"gold","icon":"coin","label":"Gold","amount":"1,200","was":"","after":"12,450 → 11,250","short":false}],"canBuy":true,"busy":true,"error":""}'>
<meta name="fixture:error" content='{"title":"","name":"Potion Pack","icon":"potion","detail":"x10","prices":[{"id":"gold","icon":"coin","label":"Gold","amount":"1,200","was":"","after":"","short":false}],"canBuy":true,"busy":false,"error":"The purchase did not go through. Nothing was charged."}'>

<!--
  The purchase confirmation the game shows after a shop's buy: the offer's
  art, name and detail, its cost in each currency with the balance left
  after, and Buy and Cancel. The game decides whether it may be bought and
  what it costs; the panel reports confirm (Buy) and cancel (Cancel, or
  Escape and pad B anywhere in it). Enter and pad A press the focused
  button, Cancel first, so no key buys by accident. While busy Buy is
  disabled. It is content, not a dialog: the host that shows it (the
  built-in confirm screen, or the game's own modal) owns modality and focus.
-->
<section id="ShopConfirm" part="ShopConfirm.frame" aria-label={title != '' ? title : t("general.confirm.name")} aria-busy={busy} onback={cancel}>
  <header part="ShopConfirm.band"><h2 part="ShopConfirm.title">{title != '' ? title : t("general.confirm.name")}</h2></header>
  <scroll class="body">
    <div class="body-inner">
      <div class="offer">
        <div class="art"><og-icon class="art-icon" icon={icon}></og-icon></div>
        <div class="text">
          <h3 part="ShopConfirm.name">{name}</h3>
          <p class="detail" hidden={detail == ''}>{detail}</p>
        </div>
      </div>
      <ul class="prices" aria-label={t("general.confirm.cost")}>
        {#each prices as p (p.id)}
          <li part="ShopConfirm.price" data-short={p.short}>
            <og-icon class="currency" icon={p.icon}></og-icon>
            <span class="was" hidden={p.was == ''} aria-label={t("general.shop.was", { price: p.was })}>{p.was}</span>
            <span class="figure" aria-label={t("general.shop.amount", { amount: p.amount, currency: p.label })}>{p.amount}</span>
            <div class="short" hidden={!p.short}><og-icon class="short-mark" icon="lock"></og-icon><span>{t("general.shop.short")}</span></div>
            <span class="after" hidden={p.after == ''}>{t("general.confirm.after", { amount: p.after })}</span>
          </li>
        {/each}
      </ul>
    </div>
  </scroll>
  <p part="ShopConfirm.error" role="alert" hidden={error == ''}>{error}</p>
  <footer class="actions">
    <button id="Cancel" variant="ghost" onclick={cancel}><span>{t("general.confirm.cancel")}</span></button>
    <button id="Buy" variant="primary" disabled={!canBuy || busy} aria-busy={busy} onclick={confirm}><span>{busy ? t("general.confirm.busy") : t("general.confirm.buy")}</span></button>
  </footer>
</section>

<style>
[part="ShopConfirm.frame"][hidden], [part="ShopConfirm.frame"] [hidden] { display: none; }
[part="ShopConfirm.frame"] { display: flex; flex-direction: column; width: stretch; max-width: var(--modal-width); max-height: 100%; min-height: 0; gap: var(--space-md); }
.body { width: stretch; flex: 0 1 auto; min-height: 0; }
.body-inner { display: flex; flex-direction: column; width: stretch; gap: var(--space-md); }
[part="ShopConfirm.band"] { display: flex; width: stretch; align-items: center; justify-content: center; flex: none; }
[part="ShopConfirm.title"] { margin: 0; text-align: center; }
.offer { display: flex; width: stretch; gap: var(--space-md); align-items: center; }
.art { display: flex; flex: none; width: var(--size-xxl); height: var(--size-xxl); align-items: center; justify-content: center; }
.art-icon { width: var(--size-xl); height: var(--size-xl); }
.text { display: flex; flex-direction: column; width: stretch; min-width: 0; gap: var(--space-xxs); }
[part="ShopConfirm.name"], .detail { margin: 0; overflow-wrap: anywhere; }
.prices { display: flex; flex-direction: column; width: stretch; margin: 0; padding: 0; list-style: none; gap: var(--space-xs); }
[part="ShopConfirm.price"] { display: flex; width: stretch; align-items: center; flex-wrap: wrap; gap: var(--space-xxs) var(--space-sm); padding: var(--space-xs) var(--space-sm); }
.currency, .short-mark { width: var(--size-xs); height: var(--size-xs); flex: none; }
.figure { font-variant-numeric: tabular-nums; white-space: nowrap; }
.was { text-decoration-line: line-through; white-space: nowrap; }
.short { display: flex; align-items: center; gap: var(--space-xxs); }
.after { margin-inline-start: auto; font-variant-numeric: tabular-nums; white-space: nowrap; }
[part="ShopConfirm.error"] { flex: none; margin: 0; }
.actions { display: flex; flex: none; width: stretch; justify-content: flex-end; gap: var(--space-sm); flex-wrap: wrap; }
.actions button { min-height: var(--size-touch); min-width: calc(var(--size-xxl) * 1.25); gap: var(--space-xs); }
@media (pointer: coarse) { .actions button { flex: 1; } }
</style>
