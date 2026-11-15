<meta name="component" content="WorldSheet">

<div id="Sheet">
  {#if $page == 'crowd'}
    <WorldCrowdPage />
  {:else}
    <WorldFieldPage />
  {/if}
</div>

<style>
#Sheet { width: stretch; height: stretch; }
</style>
