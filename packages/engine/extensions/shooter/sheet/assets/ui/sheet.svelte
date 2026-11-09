<meta name="component" content="ShooterSheet">

<div id="Sheet">
  {#if $page == 'loadout'}
    <LoadoutPage />
  {:else}
    <CombatPage />
  {/if}
</div>

<style>
#Sheet { width: stretch; height: stretch; }
</style>
