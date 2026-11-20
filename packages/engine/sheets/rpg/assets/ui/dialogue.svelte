<meta name="component" content="DialoguePage">
<main id="DialoguePage" class="page">
  <h1>Dialogue · {$scenario}</h1>
  <div class="conversation"><RpgConversation speaker={$talkSpeaker} line={$talkLine} replies={$shownReplies} selected={$talkSelected} showControls={true} canBacklog={$scenario != 'missing'} canSkip={$scenario != 'missing'} canAuto={$scenario != 'missing'} auto={$scenario == 'playing'} /></div>
  <div class="history"><RpgConversationBacklog lines={$shownHistory} hasOlder={true} loading={$scenario == 'loading'} playing={$scenario == 'playing' ? 'marissa' : ''} /></div>
</main>
<style>
.page { display: flex; flex-direction: column; inline-size: stretch; block-size: stretch; gap: var(--space-lg); padding: var(--space-lg); background: var(--panel); color: var(--ink); }
h1 { margin: 0; font-size: var(--font-step-lg); }
.history { flex: 1; min-block-size: 0; }
.conversation { block-size: calc(var(--size-xxl) * 2.6); flex: none; }
.conversation:size(compact) { block-size: calc(var(--size-xxl) * 4); }
.page:size(compact) { gap: var(--space-sm); padding: var(--space-sm); }
</style>
