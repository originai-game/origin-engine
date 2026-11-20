<meta name="component" content="CurrencyStates">
<section id="CurrencyStates">
  <h1>Currency</h1>
  <p>Balances, capacity, loading and unavailable states</p>
  <div class="examples">
    <Currency name="Gold" icon="coin" amount={12540} rate="+48/s" />
    <Currency name="Gems" icon="gem" amount={0} />
    <Currency name="Energy" icon="bolt" amount={120} limit="120" full={true} />
    <Currency name="Gold" icon="coin" amount={999} loading={true} />
    <Currency name="Gold" icon="coin" amount={999} unavailable={true} />
    <Currency name="Sovereigns of the Eastern Treasury" icon="crown" valueText="123.45 quattuordecillion" limit="500 quattuordecillion" rate="−4.2 million per second" />
  </div>
</section>
<style>
#CurrencyStates { display: flex; flex-direction: column; inline-size: stretch; block-size: stretch; overflow-y: auto; padding: var(--space-lg); gap: var(--space-sm); background: var(--panel); color: var(--ink); }
h1, p { margin: 0; }
p { color: var(--ink-muted); }
.examples { display: grid; grid-template-columns: repeat(3, 1fr); gap: var(--space-md); }
.examples:size(compact) { grid-template-columns: 1fr; }
</style>
