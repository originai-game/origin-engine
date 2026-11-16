<meta name="component" content="UngroupedSamplePage">

<section class="page">
  <h1 class="caption">{$caption}</h1>
  <div class="stage">
    {#if $member == 'breadcrumbs'}
      <Breadcrumbs items={$sample_items} label={$sample.label} />
    {:else if $member == 'carousel'}
      <Carousel pages={$sample_pages} index={$sample.index} label={$sample.label} />
    {:else if $member == 'celebration'}
      <Celebration kind={$sample.kind} shots={$sample_shots} title={$sample.title} value={$sample.value} />
    {:else if $member == 'chart'}
      <Chart title={$sample.title} subtitle={$sample.subtitle} max={$sample.max} bars={$sample_bars} line={$sample.line} area={$sample.area} ticks={$sample_ticks} />
    {:else if $member == 'dataGrid'}
      <DataGrid columns={$sample_columns} tracks={$sample.tracks} rows={$sample_rows} selected={$sample.selected} label={$sample.label} loading={$sample.loading} error={$sample.error} empty={$sample.empty} hint={$sample.hint} />
    {:else if $member == 'menu'}
      <Menu entries={$sample_entries} submenu={$sample.submenu} label={$sample.label} />
    {:else if $member == 'ribbon'}
      <Ribbon text={$sample.text} corner={$sample.corner} tone={$sample.tone} />
    {:else if $member == 'skeletonGroup'}
      <SkeletonGroup shape={$sample.shape} items={$sample_items} name={$sample.name} />
    {:else if $member == 'spinner'}
      <Spinner size={$sample.size} name={$sample.name} />
    {:else if $member == 'splitView'}
      <SplitView ratio={$sample.ratio} column={$sample.column} label={$sample.label}>
        {#snippet start()}<p class="pane">Start pane</p>{/snippet}
        {#snippet end()}<p class="pane">End pane</p>{/snippet}
      </SplitView>
    {:else if $member == 'starBar'}
      <StarBar value={$sample.value} max={$sample.max} stars={$sample_stars} label={$sample.label} valueText={$sample.valueText} compact={$sample.compact} />
    {:else if $member == 'statusGrid'}
      <StatusGrid buffs={$sample_buffs} debuffs={$sample_debuffs} more={$sample.more} />
    {:else if $member == 'tooltipCard'}
      <TooltipCard title={$sample.title} subtitle={$sample.subtitle} rarity={$sample.rarity} icon={$sample.icon} image={$sample.image} body={$sample.body} stats={$sample_stats} compareTitle={$sample.compareTitle} compareStats={$sample_compareStats} footer={$sample.footer} hints={$sample_hints} />
    {:else if $member == 'actionHints'}
      <ActionHints hints={$sample_hints} label={$sample.label} />
    {:else if $member == 'emptyState'}
      <EmptyState title={$sample.title} body={$sample.body} hint={$sample.hint} icon={$sample.icon} actionLabel={$sample.actionLabel} disabled={$sample.disabled} />
    {:else if $member == 'errorState'}
      <ErrorState title={$sample.title} body={$sample.body} code={$sample.code} retryLabel={$sample.retryLabel} canRetry={$sample.canRetry} busy={$sample.busy} dismissLabel={$sample.dismissLabel} />
    {:else if $member == 'listItem'}
      <ListItem title={$sample.title} subtitle={$sample.subtitle} icon={$sample.icon} image={$sample.image} value={$sample.value} badge={$sample.badge} selected={$sample.selected} disabled={$sample.disabled} loading={$sample.loading} />
    {:else if $member == 'steps'}
      <Steps title={$sample.title} steps={$sample_steps} column={$sample.column} />
    {/if}
  </div>
</section>

<style>
:host { display: flex; flex-direction: column; inline-size: stretch; block-size: stretch; padding: var(--space-lg); gap: var(--space-lg); overflow: auto; }
.caption { margin: 0; font: var(--text-lg); color: var(--ink-muted); }
.stage { position: relative; display: flex; flex-direction: column; align-items: start; flex: 1; gap: var(--space-md); }
.pane { margin: 0; padding: var(--space-md); }
@media (width < 600px) { :host { padding: var(--space-sm); padding-top: env(safe-area-inset-top); } }
</style>
