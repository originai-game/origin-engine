<meta name="component" content="IdlePurchases">
<meta name="strings" content="./idle.en.ogt">
<meta name="parts" content="IdlePurchases.frame IdlePurchases.band IdlePurchases.title IdlePurchases.amount IdlePurchases.row IdlePurchases.name IdlePurchases.effect IdlePurchases.cost IdlePurchases.badge IdlePurchases.figure IdlePurchases.headline IdlePurchases.caption IdlePurchases.fade">
<meta name="prop:title" content="string" data-doc="The list's heading; empty takes the group's (Upgrades).">
<meta name="prop:amounts" content="list" data-doc="The buy amounts offered, keyed rows of { id, label }: label the amount as the game words it (it wins; empty reads the key max as Max and any other key as ×key: ×1, ×10, ×100). An empty list hides the choice.">
<meta name="prop:amount" content="string" data-doc="The buy amount chosen, one of the amounts' keys.">
<meta name="prop:upgrades" content="list" data-doc="The purchases in the game's order, keyed rows of { id, icon, name, effect, level, count, cost, costIcon, affordable, maxed, locked }: effect what it does as the game words it; level its level (0 hides it); count how many the chosen amount buys, as the game words it (×37; empty hides it); cost the price of that many, formatted; costIcon the currency's icon; affordable the player can pay it (else named Not enough); maxed nothing more to buy (named Maxed); locked not yet available (named Locked). Buy is offered only when affordable and neither maxed nor locked.">
<meta name="fixture:default" content='{"title":"","amounts":[{"id":"1","label":""},{"id":"10","label":""},{"id":"100","label":""},{"id":"max","label":""}],"amount":"1","upgrades":[{"id":"pick","icon":"sword","name":"Sharper Picks","effect":"Mines yield +10%","level":4,"count":"×1","cost":"1.2K","costIcon":"coin","affordable":true,"maxed":false,"locked":false},{"id":"cart","icon":"bag","name":"Bigger Carts","effect":"Wood store +50%","level":1,"count":"×1","cost":"48K","costIcon":"coin","affordable":false,"maxed":false,"locked":false},{"id":"crown","icon":"crown","name":"Royal Charter","effect":"All income ×2","level":5,"count":"","cost":"","costIcon":"gem","affordable":false,"maxed":true,"locked":false},{"id":"star","icon":"star","name":"Starlight","effect":"Unlocks at level 50","level":0,"count":"","cost":"1M","costIcon":"gem","affordable":false,"maxed":false,"locked":true}]}'>
<meta name="fixture:max" content='{"title":"","amounts":[{"id":"1","label":""},{"id":"10","label":""},{"id":"100","label":""},{"id":"max","label":""}],"amount":"max","upgrades":[{"id":"pick","icon":"sword","name":"Sharper Picks","effect":"Mines yield +10%","level":4,"count":"×37","cost":"412K","costIcon":"coin","affordable":true,"maxed":false,"locked":false},{"id":"cart","icon":"bag","name":"Bigger Carts","effect":"Wood store +50%","level":1,"count":"×0","cost":"48K","costIcon":"coin","affordable":false,"maxed":false,"locked":false}]}'>
<meta name="fixture:disabled" content='{"title":"","amounts":[{"id":"1","label":""},{"id":"10","label":""},{"id":"100","label":""},{"id":"max","label":""}],"amount":"100","upgrades":[{"id":"pick","icon":"sword","name":"Sharper Picks","effect":"Mines yield +10%","level":4,"count":"×100","cost":"9.8M","costIcon":"coin","affordable":false,"maxed":false,"locked":false},{"id":"star","icon":"star","name":"Starlight","effect":"Unlocks at level 50","level":0,"count":"","cost":"1M","costIcon":"gem","affordable":false,"maxed":false,"locked":true}]}'>
<meta name="fixture:maxed" content='{"title":"","amounts":[],"amount":"","upgrades":[{"id":"crown","icon":"crown","name":"Royal Charter","effect":"All income ×2","level":5,"count":"","cost":"","costIcon":"gem","affordable":false,"maxed":true,"locked":false},{"id":"key","icon":"key","name":"Vault Key","effect":"Opens the vault","level":1,"count":"","cost":"","costIcon":"coin","affordable":true,"maxed":true,"locked":false}]}'>
<meta name="fixture:empty" content='{"title":"","amounts":[{"id":"1","label":""},{"id":"10","label":""},{"id":"100","label":""},{"id":"max","label":""}],"amount":"1","upgrades":[]}'>
<meta name="fixture:overflow" content='{"title":"Research and Development of the Grand Arcane Academy","amounts":[{"id":"1","label":""},{"id":"10","label":""},{"id":"100","label":""},{"id":"max","label":""}],"amount":"10","upgrades":[{"id":"u1","icon":"gear","name":"Self-Replicating Clockwork Automaton Assembly Lines","effect":"Every generator in every realm yields an additional four hundred percent","level":1234,"count":"×10","cost":"123.45 quattuordecillion","costIcon":"coin","affordable":true,"maxed":false,"locked":false},{"id":"u2","icon":"sword","name":"Picks","effect":"+10%","level":4,"count":"×10","cost":"12K","costIcon":"coin","affordable":true,"maxed":false,"locked":false},{"id":"u3","icon":"bag","name":"Carts","effect":"+50%","level":1,"count":"×10","cost":"480K","costIcon":"coin","affordable":false,"maxed":false,"locked":false},{"id":"u4","icon":"potion","name":"Elixirs","effect":"+5% speed","level":9,"count":"×10","cost":"3.3M","costIcon":"coin","affordable":false,"maxed":false,"locked":false},{"id":"u5","icon":"crown","name":"Charter","effect":"×2","level":5,"count":"","cost":"","costIcon":"gem","affordable":false,"maxed":true,"locked":false},{"id":"u6","icon":"star","name":"Starlight","effect":"Level 50","level":0,"count":"","cost":"1M","costIcon":"gem","affordable":false,"maxed":false,"locked":true},{"id":"u7","icon":"bolt","name":"Dynamos","effect":"+1 per tap","level":2,"count":"×10","cost":"77K","costIcon":"coin","affordable":true,"maxed":false,"locked":false}]}'>

<!--
  The upgrade shop of an idle game: a ×1 / ×10 / ×100 / Max choice of how
  many to buy (chooseamount, the amount's key), then one row per upgrade
  with its icon, name, effect, level, how many the chosen amount buys and
  what that costs, and Buy (buy, the upgrade's key). The game prices every
  amount and decides what can be bought: Buy is enabled only when the row
  is affordable and neither maxed nor locked, and each refused state is a
  word (Not enough, Maxed, Locked). It fills its height and scrolls.
-->
<section id="IdlePurchases" part="IdlePurchases.frame" data-layout="shell" data-gap="loose" aria-label={title != '' ? title : t("idle.purchases.name")}>
  <header part="IdlePurchases.band" data-layout="cluster" data-gap="loose">
    <h2 part="IdlePurchases.title" data-fill>{title != '' ? title : t("idle.purchases.name")}</h2>
    {#if amounts.length}
    <div class="amounts" data-layout="row" data-gap="hair" role="radiogroup" aria-label={t("idle.purchases.amount")}>
      {#each amounts as a (a.id)}
        <button part="IdlePurchases.amount" role="radio" aria-checked={a.id == amount} data-amount={a.id} onclick={chooseamount}>{a.label != '' ? a.label : a.id == 'max' ? t("idle.purchases.max") : t("idle.purchases.times", { count: a.id })}</button>
      {/each}
    </div>
    {/if}
  </header>
  {#if upgrades.length}
    <div class="rows-shell" data-layout="stack" data-gap="none" data-fill>
    <div class="rows-scroll" data-layout="scroll" data-gap="none">
      <ul class="rows" data-layout="stack" data-gap="tight" aria-label={title != '' ? title : t("idle.purchases.name")}>
        {#each upgrades as u (u.id)}
          <li part="IdlePurchases.row" data-layout="row" data-gap="loose" aria-label={u.name} data-affordable={u.affordable} data-maxed={u.maxed} data-locked={u.locked}>
            <div class="art" data-layout="ratio"><og-icon class="icon" icon={u.locked ? 'lock' : u.icon}></og-icon></div>
            <div class="body" data-layout="stack" data-gap="hair" data-fill>
              <div class="line" data-layout="row">
                <span part="IdlePurchases.name IdlePurchases.headline" class="name">{u.name}</span>
                {#if u.level > 0}<span part="IdlePurchases.badge" data-tag="level">{t("idle.purchases.level", { level: formatNumber(u.level) })}</span>{/if}
              </div>
              <p part="IdlePurchases.effect IdlePurchases.caption">{u.effect}</p>
            </div>
            <div class="end" data-layout="stack" data-gap="tight">
              {#if u.maxed}<span part="IdlePurchases.badge" data-tag="maxed">{t("idle.purchases.maxed")}</span>{/if}
              {#if !u.maxed && u.locked}<span part="IdlePurchases.badge" data-tag="locked">{t("idle.purchases.locked")}</span>{/if}
              {#if !u.maxed && !u.locked && !u.affordable}<span part="IdlePurchases.badge" data-tag="short">{t("idle.purchases.short")}</span>{/if}
              {#if !u.maxed}
              <button id="Buy" variant="primary" disabled={!u.affordable || u.maxed || u.locked} aria-label={u.cost != '' ? t("idle.purchases.buyFor", { name: u.name, cost: u.cost }) : t("idle.purchases.buyNamed", { name: u.name })} data-affordable={u.affordable} onclick={buy}><div class="buy" data-layout="stack" data-gap="none">
                {#if u.count != ''}<span class="count" part="IdlePurchases.figure">{u.count}</span>{/if}
                {#if u.cost != ''}<div part="IdlePurchases.cost IdlePurchases.figure" data-layout="row" data-gap="hair"><og-icon class="cost-icon" icon={u.costIcon}></og-icon><span class="cost-figure">{u.cost}</span></div>{:else}<span class="buy-word">{t("idle.purchases.buy")}</span>{/if}
              </div></button>
              {/if}
            </div>
          </li>
        {/each}
      </ul>
    </div>
    <div class="scroll-fade" part="IdlePurchases.fade" data-anchor="bottom-edge"></div>
    </div>
  {:else}
    <p class="empty-text" part="IdlePurchases.caption" role="status">{t("idle.purchases.empty")}</p>
  {/if}
</section>

<style>
:host { inline-size: stretch; block-size: stretch; min-block-size: 0; }
[part="IdlePurchases.band"] { width: stretch; flex: none; }
[part="IdlePurchases.title"] { margin: 0; flex: 1; min-width: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.amounts { flex: none; }
[part="IdlePurchases.amount"] { min-height: var(--size-touch); min-width: var(--size-touch); padding: 0 var(--space-sm); }
.rows-shell { position: relative; min-block-size: 0; }
.rows-scroll { overflow-y: auto; flex: 1; min-height: 0; }
.rows { width: stretch; margin: 0; padding: 0; padding-block-end: var(--space-lg); list-style: none; }
[part="IdlePurchases.row"] { padding: var(--space-sm); }
.art { --layout-ratio: 1; flex: none; width: var(--size-lg); height: var(--size-lg); }
.icon { width: var(--size-md); height: var(--size-md); }
.body { min-width: 0; }
.line { width: stretch; min-width: 0; }
.name { --headline-text: var(--text-lg); flex: 0 1 auto; min-width: 0; text-align: start; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
[part~="IdlePurchases.effect"] { margin: 0; text-align: start; overflow-wrap: anywhere; }
[part="IdlePurchases.badge"] { flex: none; white-space: nowrap; padding: 0 var(--space-xs); }
.end { flex: none; align-items: flex-end; max-width: 50%; }
#Buy { min-height: var(--size-touch); min-width: calc(var(--size-xxl) * 1.1); max-width: stretch; padding: var(--space-xxs) var(--space-sm); }
.buy { align-items: center; min-width: 0; max-width: stretch; --figure-ink: currentColor; }
.count { white-space: nowrap; }
[part="IdlePurchases.cost IdlePurchases.figure"] { min-width: 0; max-width: stretch; }
.cost-icon { width: var(--size-xs); height: var(--size-xs); flex: none; }
.cost-figure { min-width: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.empty-text { margin: 0; flex: 1; padding: var(--space-lg); text-align: center; }
/* A narrow panel (a phone, or a side column on a wide screen) puts the
   amounts under the title and each row's price and Buy under its name. */
@container (width < 560px) {
  [part="IdlePurchases.band"] .amounts { flex: 1 0 100%; }
  [part="IdlePurchases.amount"] { flex: 1; min-width: 0; }
  [part="IdlePurchases.row"] { flex-wrap: wrap; padding: var(--space-xs); }
  .art { width: var(--size-md); height: var(--size-md); }
  .icon { width: var(--size-sm); height: var(--size-sm); }
  .body { flex: 1 1 0; }
  .end { flex: 1 0 100%; max-width: stretch; justify-content: flex-end; }
}
</style>
