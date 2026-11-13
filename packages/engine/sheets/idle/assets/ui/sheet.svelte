<meta name="component" content="IdleSheet">

<div id="Sheet">
  {#if $page == 'reset'}
    <IdleResetPage />
  {:else}
    {#if $page == 'states' || $page == 'states-resources' || $page == 'states-generators' || $page == 'states-purchases' || $page == 'states-prestige' || $page == 'states-offline'}
      <IdleStatesPage />
    {:else}
      <IdlePlayPage />
    {/if}
  {/if}
</div>

<style>
#Sheet { width: stretch; height: stretch; }
</style>
