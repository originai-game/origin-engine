<meta name="component" content="GeneralPurchasePage">

<!-- The shop family's purchase page: the confirmation in each of its
     states, side by side at the height a confirmation takes: one the player
     can afford, one they cannot (the ledger says how much more), and one
     whose Buy failed. A phone shows the short one over the failed one, each half the
     page with its buttons pinned; the
     store's phone cell shows the affordable one. -->
<div id="Page" class="general-ground">
  <div id="Ready"><ShopConfirm title={$blank} name={$confirmName} icon={$confirmIcon} rarity={$confirmRarity} detail={$confirmDetail} prices={$confirmPrices} canBuy={true} reason={$blank} busy={false} error={$blank} /></div>
  <div id="Short"><ShopConfirm title={$blank} name={$shortName} icon={$shortIcon} rarity={$shortRarity} detail={$shortDetail} prices={$shortPrices} canBuy={false} reason={$blank} busy={false} error={$blank} /></div>
  <div id="Failed"><ShopConfirm title={$blank} name={$failName} icon={$failIcon} rarity={$failRarity} detail={$blank} prices={$failPrices} canBuy={true} reason={$blank} busy={false} error={$confirmError} /></div>
</div>

<style>
#Page { display: grid; width: stretch; height: stretch; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: var(--space-lg); padding: var(--space-lg); align-items: center; }
#Page > div { min-height: 0; }
@media (width < 600px) {
  #Page { display: flex; flex-direction: column; gap: var(--space-md); padding: var(--space-sm); overflow: hidden; }
  #Page > div { width: stretch; flex: 1; min-height: 0; height: auto; }
  #Ready { display: none; }
}
</style>
