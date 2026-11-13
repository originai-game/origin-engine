<meta name="screen" content="CardSheet" data-layer="menu" data-input="menu">

<div id="Sheet">
  {#if $page == 'builder'}
    <BuilderPage />
  {:else if $page == 'run'}
    <RunPage />
  {:else}
    <TablePage />
  {/if}
</div>

<style>
#Sheet { width: stretch; height: stretch; }
</style>
