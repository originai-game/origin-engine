<meta name="screen" content="RpgSheet" data-layer="menu" data-input="menu">

<div id="Sheet">
  {#if $page == 'journal'}
    <JournalPage />
  {:else if $page == 'equipment'}
    <EquipmentPage />
  {:else if $page == 'explore'}
    <ExplorePage />
  {:else if $page == 'combat'}
    <CombatPage />
  {:else if $page == 'primitives'}
    <PrimitivesPage />
  {:else}
    <InventoryPage />
  {/if}
</div>

<style>
#Sheet { width: stretch; height: stretch; }
</style>
