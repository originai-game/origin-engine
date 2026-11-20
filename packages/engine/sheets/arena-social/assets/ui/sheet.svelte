<meta name="component" content="ArenaSocialSheet">
<main id="ArenaSocialSheet">
  {#if $page == 'ready'}
    <ArenaReady title={$vm.title} detail={$vm.detail} timeText={$vm.timeText} players={$rows} response={$vm.response} canRespond={$vm.canRespond} closed={$vm.closed} loading={$vm.loading} />
  {:else}{#if $page == 'vote'}
    <ArenaVote title={$vm.title} detail={$vm.detail} timeText={$vm.timeText} options={$rows} selected={$vm.selected} winner={$vm.winner} canVote={$vm.canVote} closed={$vm.closed} loading={$vm.loading} />
  {:else}
    <ArenaSpectator title={$vm.title} detail={$vm.detail} targets={$rows} selected={$vm.selected} freeCamera={$vm.freeCamera} canFree={$vm.canFree} canCycle={$vm.canCycle} canLeave={$vm.canLeave} loading={$vm.loading} />
  {/if}{/if}
</main>
<style>
#ArenaSocialSheet { display: flex; flex-direction: column; align-items: center; inline-size: stretch; block-size: stretch; padding: var(--space-lg); background: var(--panel); }
main { display: flex; flex-direction: column; inline-size: stretch; max-inline-size: 640px; block-size: stretch; }
</style>
