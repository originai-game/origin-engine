<meta name="component" content="IdlePurchases">
<meta name="strings" content="./idle.en.ogt">
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
<section class="frame" id="IdlePurchases" part="frame" layout="stack" fill gap="loose" aria-label={title != '' ? title : t("idle.purchases.name")}>
  <div class="lit"></div>
  <header class="band" part="band" layout="wrap" gap="loose">
    <h2 class="title" part="title" fill>{title != '' ? title : t("idle.purchases.name")}</h2>
    {#if amounts.length}
    <div class="amounts" layout="row" gap="hair" role="radiogroup" aria-label={t("idle.purchases.amount")}>
      {#each amounts as a (a.id)}
        <button class="amount" part="amount" role="radio" data-amount={a.id} onclick={chooseamount} class:checked={a.id == amount}>{a.label ? a.label : a.id == 'max' ? t("idle.purchases.max") : t("idle.purchases.times", { count: a.id })}</button>
      {/each}
    </div>
    {/if}
  </header>
  {#if upgrades.length}
    <div class="rows-shell" layout="stack" gap="none" fill>
    <div class="rows-scroll" layout="scroll" gap="none">
      <ul class="rows" layout="stack" gap="tight" aria-label={title != '' ? title : t("idle.purchases.name")}>
        {#each upgrades as u (u.id)}
          <li class="row" part="row" layout="row" gap="loose" aria-label={u.name} data-maxed={u.maxed} state:locked={u.locked} class:not-affordable={!(u.affordable)}>
            <div part="socket" class="art cover" layout="stack" fill><og-icon class="icon" icon={u.locked ? 'lock' : u.icon}></og-icon></div>
            <div class="body" layout="stack" gap="hair" fill>
              <div class="line" layout="row">
                <span part="name" class="name">{u.name}</span>
                {#if u.level > 0}<span class="badge" part="badge" class:tag-level={true}>{t("idle.purchases.level", { level: formatNumber(u.level) })}</span>{/if}
              </div>
              <p class="effect" part="caption">{u.effect}</p>
            </div>
            <div class="end" layout="stack" gap="tight">
              {#if u.maxed}<span class="badge status" part="badge">{t("idle.purchases.maxed")}</span>{/if}
              {#if !u.maxed && u.locked}<span class="badge status" part="badge">{t("idle.purchases.locked")}</span>{/if}
              {#if !u.maxed && !u.locked && !u.affordable}<span class="badge status" part="badge" class:tag-short={true}>{t("idle.purchases.short")}</span>{/if}
              {#if !u.maxed}
              <button id="Buy" class="go" variant="primary" disabled={!u.affordable || u.maxed || u.locked} aria-label={u.cost ? t("idle.purchases.buyFor", { name: u.name, cost: u.cost }) : t("idle.purchases.buyNamed", { name: u.name })} onclick={buy} class:not-affordable={!(u.affordable)}><div class="buy" layout="stack" gap="none">
                {#if u.count}<span class="count" part="figure">{u.count}</span>{/if}
                {#if u.cost}<div class="cost" part="figure" layout="row" gap="hair"><og-icon class="cost-icon" icon={u.costIcon}></og-icon><span class="cost-figure">{u.cost}</span></div>{:else}<span class="buy-word">{t("idle.purchases.buy")}</span>{/if}
              </div></button>
              {/if}
            </div>
          </li>
        {/each}
      </ul>
    </div>
    <div class="scroll-fade" part="fade" anchor="bottom-edge"></div>
    </div>
  {:else}
    <p class="empty-text" part="caption" role="status">{t("idle.purchases.empty")}</p>
  {/if}
</section>

<style>
.cover { justify-content: center; align-items: center; }
#IdlePurchases { inline-size: stretch; block-size: stretch; min-block-size: 0; }
/* The frame: two corners cut, a lit top edge (the shop family's look). */
.frame { position: relative; border-radius: var(--inv-frame-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.lit { position: absolute; inset-inline: var(--inv-frame-cut) 30%; top: 0; block-size: var(--stroke-hairline); background: var(--inv-lit-edge); pointer-events: none; }
.band { width: stretch; flex: none; align-items: flex-end; }
.title { margin: 0; flex: 1; min-width: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
/* The amounts: segments in a well, the chosen one lit. */
.amounts { flex: none; padding: var(--space-xxs); border-radius: var(--inv-seg-radius); background: var(--inv-chip-fill); box-shadow: var(--inv-chip-rim); }
.amount { min-height: var(--size-touch); min-width: var(--size-touch); padding: 0 var(--space-sm); border-radius: var(--inv-seg-radius); background: var(--inv-bare-fill); box-shadow: none; color: var(--inv-tab-ink); }
.amount.checked { color: var(--inv-tab-on-ink); background: var(--inv-tab-count-fill); box-shadow: var(--inv-tag-rim); }
.rows-shell { position: relative; min-block-size: 0; }
.rows-scroll { overflow-y: auto; flex: 1; min-height: 0; }
.rows { width: stretch; margin: 0; padding: 0; padding-block-end: var(--space-lg); list-style: none; }
/* A row: cut at two corners, its socket, words, then its state and Buy. */
.row { padding: var(--space-xs) var(--space-sm); align-items: center; background: var(--inv-row-fill); box-shadow: var(--inv-row-rim); border-inline-start: var(--inv-edge); border-inline-start-color: var(--inv-accent); border-radius: var(--inv-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.row:state(locked) .art, .row:state(locked) .body { opacity: var(--disabled-content-opacity); }
.art { flex: none; inline-size: calc(48 * var(--font-step-px)); block-size: calc(48 * var(--font-step-px)); background: var(--cell-common-fill); box-shadow: var(--cell-common-shadow); border-radius: var(--inv-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.icon { width: 64%; height: 64%; filter: var(--inv-slot-art-shadow); }
.body { min-width: 0; }
.line { width: stretch; min-width: 0; align-items: center; }
.name { flex: 0 1 auto; min-width: 0; text-align: start; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; font-size: var(--shop-tile-name-size); font-weight: var(--inv-name-weight); color: var(--ink-bright); }
.effect { margin: 0; text-align: start; overflow-wrap: anywhere; }
/* Small words: the level as a chip; a refusal in its tone on a wash of it. */
.badge { flex: none; white-space: nowrap; padding: 0 var(--space-xs); border-radius: var(--inv-chip-radius); font-size: var(--inv-tag-size); font-weight: var(--inv-figure-weight); color: var(--ink-label); background: var(--inv-tab-count-fill); box-shadow: none; }
.badge.status { letter-spacing: var(--hud-label-tracking); text-transform: var(--inv-kicker-transform); color: var(--shop-sold-ink); background: var(--shop-sold-fill); border-radius: var(--inv-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.badge.tag-short { color: var(--shop-short-ink); background: var(--shop-short-fill); }
.end { flex: none; align-items: flex-end; max-width: 50%; }
/* Buy: how many over what they cost; refused, a wash of the danger. */
#Buy { min-height: var(--size-touch); min-width: var(--shop-buy-width); max-width: stretch; padding: var(--space-xxs) var(--space-sm); }
#Buy.not-affordable { --button-primary-fill: var(--shop-buy-refused-fill); --button-primary-shadow: var(--shop-buy-refused-rim); color: var(--shop-short-ink); }
.buy { align-items: center; min-width: 0; max-width: stretch; --figure-ink: currentColor; }
.count { white-space: nowrap; font-size: var(--inv-tag-size); font-weight: var(--inv-figure-weight); opacity: 0.8; }
.cost { min-width: 0; max-width: stretch; align-items: center; font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); }
.cost-icon { width: var(--size-xs); height: var(--size-xs); flex: none; }
.cost-figure { min-width: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.empty-text { margin: 0; flex: 1; padding: var(--space-lg); text-align: center; }
/* A narrow panel (a phone, or a side column on a wide screen) puts the
   amounts under the title and each row's price and Buy under its name. */
.band .amounts:size(compact) { flex: 1 0 100%; }
.amount:size(compact) { flex: 1; min-width: 0; }
.row:size(compact) { flex-wrap: wrap; }
.art:size(compact) { inline-size: calc(40 * var(--font-step-px)); block-size: calc(40 * var(--font-step-px)); }
.body:size(compact) { flex: 1 1 0; }
.end:size(compact) { flex: 1 0 100%; max-width: stretch; flex-direction: row; justify-content: flex-end; align-items: center; }
</style>
