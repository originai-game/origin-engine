<meta name="screen" content="GeneralSheet" data-layer="menu" data-input="menu">

<div id="Sheet">
  {#if $page == 'progress'}
    <GeneralProgressPage />
  {:else if $page == 'social'}
    <GeneralSocialPage />
  {:else if $page == 'rewards'}
    <GeneralRewardsPage />
  {:else if $page == 'purchase'}
    <GeneralPurchasePage />
  {:else if $page == 'season'}
    <GeneralSeasonPage />
  {:else if $page == 'states'}
    <GeneralStatesPage />
  {:else}
    <GeneralStorePage />
  {/if}
</div>

<style>
#Sheet { width: stretch; height: stretch; }
</style>
