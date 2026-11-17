<meta name="component" content="UngroupedSamplePage">
<meta name="parts" content="Sheet.card">

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
      <div class="host" part="Sheet.card" class:low={$sample.corner == 'bottomRight' || $sample.corner == 'bottomLeft'}><p class="host-name">Ember Blade</p><p class="host-note">Legendary sword</p><div class="pin" class:topLeft={$sample.corner == 'topLeft'} class:bottomRight={$sample.corner == 'bottomRight'} class:bottomLeft={$sample.corner == 'bottomLeft'} class:center={$sample.corner == 'center'}><Ribbon text={$sample.text} corner={$sample.corner} tone={$sample.tone} /></div></div>
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
    {:else if $member == 'passTrack'}
      <PassTrack title={$sample.title} season={$sample.season} refresh={$sample.refresh} progress={$sample.progress} progressText={$sample.progressText} premiumUnlocked={$sample.premiumUnlocked} upgradeLabel={$sample.upgradeLabel} rewards={$sample_rewards} loading={$sample.loading} error={$sample.error} disabled={$sample.disabled} />
    {:else if $member == 'reveal'}
      <Reveal title={$sample.title} body={$sample.body} state={$sample.state} rewards={$sample_rewards} canSkip={$sample.canSkip} continueLabel={$sample.continueLabel} disabled={$sample.disabled} error={$sample.error} />
    {:else if $member == 'inbox'}
      <Inbox title={$sample.title} summary={$sample.summary} messages={$sample_messages} attachments={$sample_attachments} selected={$sample.selected} loading={$sample.loading} busy={$sample.busy} error={$sample.error} />
    {:else if $member == 'news'}
      <News title={$sample.title} entries={$sample_entries} selected={$sample.selected} loading={$sample.loading} disabled={$sample.disabled} error={$sample.error} />
    {/if}
  </div>
</section>

<style>
:host { display: flex; flex-direction: column; inline-size: stretch; block-size: stretch; padding: var(--space-lg); gap: var(--space-lg); overflow: auto; }
.caption { margin: 0; font: var(--text-lg); color: var(--ink-muted); }
.stage { position: relative; display: flex; flex-direction: column; align-items: start; flex: 1; gap: var(--space-md); }
.pane { margin: 0; padding: var(--space-md); }
/* A ribbon is shown where a game puts it: in the corner it names of a
   card that clips, whose own corner cuts the strip's ends. */
.host { position: relative; overflow: hidden; display: flex; flex-direction: column; justify-content: end; gap: var(--space-xxs); inline-size: calc(var(--size-xxl) * 2.75); max-inline-size: 100%; block-size: calc(var(--size-xxl) * 1.75); flex: none; }
.host.low { justify-content: start; }
.host-name { margin: 0; font: var(--text-base); font-weight: var(--font-weight-semibold); color: var(--ink); }
.host-note { margin: 0; font: var(--text-caption); color: var(--ink-muted); }
.pin { position: absolute; top: 0; right: 0; }
.pin.topLeft { right: auto; left: 0; }
.pin.bottomRight { top: auto; bottom: 0; }
.pin.bottomLeft { top: auto; bottom: 0; right: auto; left: 0; }
.pin.center { top: 50%; left: 50%; right: auto; translate: -50% -50%; }
@media (width < 600px) { :host { padding: var(--space-sm); padding-top: env(safe-area-inset-top); } }
</style>
