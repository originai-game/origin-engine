<meta name="component" content="RhythmSheet">

<div id="Sheet">
  {#if $page == 'play'}
    <PlayPage />
  {:else}
    {#if $page == 'states'}
      <StatesPage />
    {:else}
      <SongsPage />
    {/if}
  {/if}
</div>

<style>
#Sheet { width: stretch; height: stretch; }
</style>
