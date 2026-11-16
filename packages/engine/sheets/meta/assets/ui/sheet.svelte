<meta name="screen" content="GeneralSheet" data-layer="menu" data-input="menu">

<div id="Sheet">
  {#if $page == 'progress'}
    <GeneralProgressPage />
  {:else if $page == 'social'}
    <GeneralSocialPage />
  {:else if $page == 'states'}
    <GeneralStatesPage />
  {:else}
    <GeneralStorePage />
  {/if}
</div>

<style>
#Sheet { width: stretch; height: stretch; }
</style>
