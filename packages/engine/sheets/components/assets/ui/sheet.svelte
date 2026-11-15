<meta name="component" content="UngroupedSheet">

<div id="Sheet">
  {#if $page == 'panels'}
    <UngroupedPanelsPage />
  {:else}
    {#if $page == 'states'}
      <UngroupedStatesPage />
    {:else}
      <UngroupedHudPage />
    {/if}
  {/if}
</div>

<style>
#Sheet { width: stretch; height: stretch; }
</style>
