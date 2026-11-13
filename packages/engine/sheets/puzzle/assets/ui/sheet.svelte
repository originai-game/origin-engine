<meta name="screen" content="PuzzleSheet" data-layer="menu" data-input="menu">

<div id="Sheet">
  {#if $page == 'map'}
    <PuzzleMapPage />
  {:else}
    {#if $page == 'states'}
      <PuzzleStatesPage />
    {:else}
      <PuzzlePlayPage />
    {/if}
  {/if}
</div>

<style>
#Sheet { width: stretch; height: stretch; }
</style>
