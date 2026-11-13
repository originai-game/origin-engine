<meta name="component" content="ArenaSheet">

<div id="Sheet">
  {#if $page == 'match'}
    <ArenaMatchPage />
  {:else}
    {#if $page == 'states'}
      <ArenaStatesPage />
    {:else}
      <ArenaSelectPage />
    {/if}
  {/if}
</div>

<style>
#Sheet { width: stretch; height: stretch; }
</style>
