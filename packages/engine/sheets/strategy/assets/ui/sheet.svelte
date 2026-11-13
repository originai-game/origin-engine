<meta name="component" content="StrategySheet">

<div id="Sheet">
  {#if $page == 'defence'}
    <StrategyDefencePage />
  {:else}
    {#if $page == 'states'}
      <StrategyStatesPage />
    {:else}
      <StrategyFieldPage />
    {/if}
  {/if}
</div>

<style>
#Sheet { width: stretch; height: stretch; }
</style>
