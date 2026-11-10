<meta name="component" content="RpgSheet">

<div id="Sheet">
  {#if $page == 'journal'}
    <JournalPage />
  {:else}
    <InventoryPage />
  {/if}
</div>

<style>
#Sheet { width: stretch; height: stretch; }
</style>
