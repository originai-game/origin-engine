<meta name="component" content="GeneralStorePage">

<!-- Desktop: the shop across two thirds, the purchase it asked for and the
     login calendar beside it. A phone: the shop, the confirmation a sheet at its foot. -->
<div id="Page" class="general-ground">
  <div id="Shop"><Shop title={$shopTitle} kicker={$shopKicker} wallet={$shopWallet} tabs={$shopTabs} tab={$shopTab} offers={$shopOffers} prices={$shopPrices} selected={$shopSelected} loading={$no} error={$blank} /></div>
  <div id="Side">
    <div id="Confirm"><ShopConfirm title={$blank} name={$confirmName} icon={$confirmIcon} rarity={$confirmRarity} detail={$confirmDetail} prices={$confirmPrices} canBuy={true} reason={$blank} busy={false} error={$blank} /></div>
    <div id="Rewards"><DailyRewards title={$blank} refresh={$rewardsRefresh} days={$rewardsDays} canClaim={true} /></div>
  </div>
</div>

<style>
#Page { display: grid; width: stretch; height: stretch; grid-template-columns: minmax(0, 1.7fr) minmax(0, 1fr); column-gap: var(--space-lg); padding: var(--space-lg); }
#Shop { height: stretch; min-height: 0; }
#Side { display: flex; flex-direction: column; height: stretch; min-height: 0; gap: var(--space-lg); }
#Confirm { width: stretch; flex: none; }
#Rewards { width: stretch; flex: 1; min-height: 0; }
#Page:size(compact) { display: flex; flex-direction: column; gap: var(--space-md); padding: var(--space-sm); }
#Page:size(compact) { padding-block-end: 0; }
#Shop:size(compact) { width: stretch; flex: 1; min-height: 0; }
#Side:size(compact) { width: stretch; height: auto; flex: none; min-height: 0; max-height: 60%; }
#Confirm:size(compact) { flex: none; min-height: 0; }
#Rewards:size(compact) { display: none; }
</style>
