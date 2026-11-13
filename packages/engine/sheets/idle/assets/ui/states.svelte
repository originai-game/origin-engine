<meta name="component" content="IdleStatesPage">

<!-- Every idle component in every fixture state it declares, a captioned
     cell each, in a grid that scrolls (docs/ui.md §6.1): page states shows
     them all, states-<member> one member's, which fit one desktop screen. -->
<scroll id="Page">
  <div id="Cells">
    {#if $page == 'states' || $page == 'states-resources'}
    <section class="cell" data-member="resources"><span class="caption">{$captions.resources_default}</span><div class="stage"><IdleResources currencies={$resources_default_currencies} loading={$resources_default.loading} /></div></section>
    <section class="cell" data-member="resources"><span class="caption">{$captions.resources_loading}</span><div class="stage"><IdleResources currencies={$resources_loading_currencies} loading={$resources_loading.loading} /></div></section>
    <section class="cell" data-member="resources"><span class="caption">{$captions.resources_full}</span><div class="stage"><IdleResources currencies={$resources_full_currencies} loading={$resources_full.loading} /></div></section>
    <section class="cell" data-member="resources"><span class="caption">{$captions.resources_draining}</span><div class="stage"><IdleResources currencies={$resources_draining_currencies} loading={$resources_draining.loading} /></div></section>
    <section class="cell" data-member="resources"><span class="caption">{$captions.resources_empty}</span><div class="stage"><IdleResources currencies={$resources_empty_currencies} loading={$resources_empty.loading} /></div></section>
    <section class="cell" data-member="resources"><span class="caption">{$captions.resources_overflow}</span><div class="stage"><IdleResources currencies={$resources_overflow_currencies} loading={$resources_overflow.loading} /></div></section>
    {/if}
    {#if $page == 'states' || $page == 'states-generators'}
    <section class="cell" data-member="generators"><span class="caption">{$captions.generators_default}</span><div class="stage"><IdleGenerators title={$generators_default.title} generators={$generators_default_generators} selected={$generators_default.selected} /></div></section>
    <section class="cell" data-member="generators"><span class="caption">{$captions.generators_empty}</span><div class="stage"><IdleGenerators title={$generators_empty.title} generators={$generators_empty_generators} selected={$generators_empty.selected} /></div></section>
    <section class="cell" data-member="generators"><span class="caption">{$captions.generators_disabled}</span><div class="stage"><IdleGenerators title={$generators_disabled.title} generators={$generators_disabled_generators} selected={$generators_disabled.selected} /></div></section>
    <section class="cell" data-member="generators"><span class="caption">{$captions.generators_selected}</span><div class="stage"><IdleGenerators title={$generators_selected.title} generators={$generators_selected_generators} selected={$generators_selected.selected} /></div></section>
    <section class="cell" data-member="generators"><span class="caption">{$captions.generators_overflow}</span><div class="stage"><IdleGenerators title={$generators_overflow.title} generators={$generators_overflow_generators} selected={$generators_overflow.selected} /></div></section>
    {/if}
    {#if $page == 'states' || $page == 'states-purchases'}
    <section class="cell" data-member="purchases"><span class="caption">{$captions.purchases_default}</span><div class="stage"><IdlePurchases title={$purchases_default.title} amounts={$purchases_default_amounts} amount={$purchases_default.amount} upgrades={$purchases_default_upgrades} /></div></section>
    <section class="cell" data-member="purchases"><span class="caption">{$captions.purchases_max}</span><div class="stage"><IdlePurchases title={$purchases_max.title} amounts={$purchases_max_amounts} amount={$purchases_max.amount} upgrades={$purchases_max_upgrades} /></div></section>
    <section class="cell" data-member="purchases"><span class="caption">{$captions.purchases_disabled}</span><div class="stage"><IdlePurchases title={$purchases_disabled.title} amounts={$purchases_disabled_amounts} amount={$purchases_disabled.amount} upgrades={$purchases_disabled_upgrades} /></div></section>
    <section class="cell" data-member="purchases"><span class="caption">{$captions.purchases_maxed}</span><div class="stage"><IdlePurchases title={$purchases_maxed.title} amounts={$purchases_maxed_amounts} amount={$purchases_maxed.amount} upgrades={$purchases_maxed_upgrades} /></div></section>
    <section class="cell" data-member="purchases"><span class="caption">{$captions.purchases_empty}</span><div class="stage"><IdlePurchases title={$purchases_empty.title} amounts={$purchases_empty_amounts} amount={$purchases_empty.amount} upgrades={$purchases_empty_upgrades} /></div></section>
    <section class="cell" data-member="purchases"><span class="caption">{$captions.purchases_overflow}</span><div class="stage"><IdlePurchases title={$purchases_overflow.title} amounts={$purchases_overflow_amounts} amount={$purchases_overflow.amount} upgrades={$purchases_overflow_upgrades} /></div></section>
    {/if}
    {#if $page == 'states' || $page == 'states-prestige'}
    <section class="cell" data-member="prestige"><span class="caption">{$captions.prestige_default}</span><div class="stage"><IdlePrestige title={$prestige_default.title} currency={$prestige_default.currency} icon={$prestige_default.icon} held={$prestige_default.held} gain={$prestige_default.gain} rows={$prestige_default_rows} requirement={$prestige_default.requirement} progress={$prestige_default.progress} progressText={$prestige_default.progressText} met={$prestige_default.met} confirming={$prestige_default.confirming} /></div></section>
    <section class="cell" data-member="prestige"><span class="caption">{$captions.prestige_disabled}</span><div class="stage"><IdlePrestige title={$prestige_disabled.title} currency={$prestige_disabled.currency} icon={$prestige_disabled.icon} held={$prestige_disabled.held} gain={$prestige_disabled.gain} rows={$prestige_disabled_rows} requirement={$prestige_disabled.requirement} progress={$prestige_disabled.progress} progressText={$prestige_disabled.progressText} met={$prestige_disabled.met} confirming={$prestige_disabled.confirming} /></div></section>
    <section class="cell" data-member="prestige"><span class="caption">{$captions.prestige_confirming}</span><div class="stage"><IdlePrestige title={$prestige_confirming.title} currency={$prestige_confirming.currency} icon={$prestige_confirming.icon} held={$prestige_confirming.held} gain={$prestige_confirming.gain} rows={$prestige_confirming_rows} requirement={$prestige_confirming.requirement} progress={$prestige_confirming.progress} progressText={$prestige_confirming.progressText} met={$prestige_confirming.met} confirming={$prestige_confirming.confirming} /></div></section>
    <section class="cell" data-member="prestige"><span class="caption">{$captions.prestige_empty}</span><div class="stage"><IdlePrestige title={$prestige_empty.title} currency={$prestige_empty.currency} icon={$prestige_empty.icon} held={$prestige_empty.held} gain={$prestige_empty.gain} rows={$prestige_empty_rows} requirement={$prestige_empty.requirement} progress={$prestige_empty.progress} progressText={$prestige_empty.progressText} met={$prestige_empty.met} confirming={$prestige_empty.confirming} /></div></section>
    <section class="cell" data-member="prestige"><span class="caption">{$captions.prestige_overflow}</span><div class="stage"><IdlePrestige title={$prestige_overflow.title} currency={$prestige_overflow.currency} icon={$prestige_overflow.icon} held={$prestige_overflow.held} gain={$prestige_overflow.gain} rows={$prestige_overflow_rows} requirement={$prestige_overflow.requirement} progress={$prestige_overflow.progress} progressText={$prestige_overflow.progressText} met={$prestige_overflow.met} confirming={$prestige_overflow.confirming} /></div></section>
    {/if}
    {#if $page == 'states' || $page == 'states-offline'}
    <section class="cell" data-member="offline"><span class="caption">{$captions.offline_default}</span><div class="stage"><IdleOffline title={$offline_default.title} away={$offline_default.away} seconds={$offline_default.seconds} earned={$offline_default_earned} canDouble={$offline_default.canDouble} doubleText={$offline_default.doubleText} /></div></section>
    <section class="cell" data-member="offline"><span class="caption">{$captions.offline_disabled}</span><div class="stage"><IdleOffline title={$offline_disabled.title} away={$offline_disabled.away} seconds={$offline_disabled.seconds} earned={$offline_disabled_earned} canDouble={$offline_disabled.canDouble} doubleText={$offline_disabled.doubleText} /></div></section>
    <section class="cell" data-member="offline"><span class="caption">{$captions.offline_empty}</span><div class="stage"><IdleOffline title={$offline_empty.title} away={$offline_empty.away} seconds={$offline_empty.seconds} earned={$offline_empty_earned} canDouble={$offline_empty.canDouble} doubleText={$offline_empty.doubleText} /></div></section>
    <section class="cell" data-member="offline"><span class="caption">{$captions.offline_overflow}</span><div class="stage"><IdleOffline title={$offline_overflow.title} away={$offline_overflow.away} seconds={$offline_overflow.seconds} earned={$offline_overflow_earned} canDouble={$offline_overflow.canDouble} doubleText={$offline_overflow.doubleText} /></div></section>
    {/if}
  </div>
</scroll>

<style>
#Page { width: stretch; height: stretch; }
#Cells { display: grid; grid-template-columns: repeat(auto-fill, minmax(calc(var(--size-xxl) * 3.6), 1fr)); gap: var(--space-lg); padding: var(--space-lg); }
.cell { display: flex; flex-direction: column; gap: var(--space-xs); min-width: 0; width: stretch; }
.caption { font: var(--text-caption); color: var(--ink-muted); }
.stage { display: flex; flex-direction: column; width: stretch; min-width: 0; height: calc(var(--size-xxl) * 3); align-items: stretch; padding: var(--space-sm); border-radius: var(--radius-sm); background: var(--well); }
.cell[data-member=resources] .stage { height: auto; min-height: calc(var(--size-xxl) * 1.2); }
@media (width < 600px) {
  #Cells { grid-template-columns: minmax(0, 1fr); padding: var(--space-sm); gap: var(--space-md); }
  .stage { height: calc(var(--size-xxl) * 5.5); }
}
</style>
