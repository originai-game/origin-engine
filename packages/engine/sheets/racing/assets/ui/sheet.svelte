<meta name="component" content="RacingSheet">

<div id="Sheet">
  {#if $page == 'garage'}
    <GaragePage />
  {:else}
    {#if $page == 'states'}
      <StatesPage />
    {:else}
      <RacePage />
    {/if}
  {/if}
</div>

<style>
#Sheet { width: stretch; height: stretch; }
</style>
