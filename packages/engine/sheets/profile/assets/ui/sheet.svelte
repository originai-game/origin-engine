<meta name="component" content="ProfileSheet">
<main id="ProfileSheet">
  <Profile name={$vm.name} handle={$vm.handle} portrait={$vm.portrait} level={$vm.level} status={$vm.status} rank={$vm.rank} bio={$vm.bio} stats={$stats} actions={$actions} privateStats={$vm.privateStats} loading={$vm.loading} error={$vm.error} empty={$vm.empty} />
</main>
<style>
#ProfileSheet { display: flex; flex-direction: column; align-items: center; inline-size: stretch; block-size: stretch; padding: var(--space-lg); background: var(--panel); }
main { display: flex; flex-direction: column; inline-size: stretch; max-inline-size: 640px; block-size: stretch; }
</style>
