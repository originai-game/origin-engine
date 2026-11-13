<meta name="component" content="GeneralStorePage">

<!-- Desktop: the shop across two thirds, the purchase it asked for and the
     login calendar beside it. A phone: the shop over the confirmation. -->
<div id="Page" class="general-ground">
  <div id="Shop"><Shop title={$shopTitle} wallet={$shopWallet} tabs={$shopTabs} tab={$shopTab} offers={$shopOffers} prices={$shopPrices} selected={$shopSelected} loading={$no} error={$blank} /></div>
  <div id="Side">
    <div id="Confirm"><ShopConfirm title={$blank} name={$confirmName} icon={$confirmIcon} detail={$confirmDetail} prices={$confirmPrices} canBuy={true} busy={false} error={$blank} /></div>
    <div id="Rewards"><DailyRewards title={$blank} refresh={$rewardsRefresh} days={$rewardsDays} canClaim={true} /></div>
  </div>
</div>

<style>
#Page { display: grid; width: stretch; height: stretch; grid-template-columns: minmax(0, 1.7fr) minmax(0, 1fr); column-gap: var(--space-lg); padding: var(--space-lg); }
#Shop { height: stretch; min-height: 0; }
#Side { display: flex; flex-direction: column; height: stretch; min-height: 0; gap: var(--space-lg); }
#Confirm { width: stretch; flex: none; }
#Rewards { width: stretch; flex: 1; min-height: 0; }
@media (width < 600px) {
  #Page { display: flex; flex-direction: column; gap: var(--space-md); padding: var(--space-sm); }
  #Shop { width: stretch; flex: 1; min-height: 48%; }
  #Side { width: stretch; height: auto; flex: 1; min-height: 0; }
  #Confirm { flex: 1; min-height: 0; }
  #Rewards { display: none; }
}
</style>
