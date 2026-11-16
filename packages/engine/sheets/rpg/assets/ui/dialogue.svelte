<meta name="component" content="DialoguePage">
<main class="page">
  <h1>Dialogue · {$scenario}</h1>
  <div class="history"><RpgConversationBacklog lines={$shownHistory} hasOlder={true} loading={$scenario == 'loading'} playing={$scenario == 'playing' ? 'marissa' : ''} /></div>
</main>
<style>
.page { display: flex; flex-direction: column; inline-size: stretch; block-size: stretch; gap: var(--space-lg); padding: var(--space-lg); background: var(--panel); color: var(--ink); }
h1 { margin: 0; font-size: var(--font-step-lg); }
.history { flex: 1; min-block-size: 0; }
</style>
