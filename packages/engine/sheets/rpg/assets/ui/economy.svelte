<meta name="component" content="EconomyPage">

<main class="page">
  <h1>Economy · {$scenario}</h1>
  <div class="panel" data-page={$page}>
    {#if $page == 'crafting'}
      <RpgCrafting title="Alchemist’s bench" recipes={$shownRecipes} selected={$scenario == 'missing' ? 'elixir' : 'draught'} ingredients={$shownIngredients} quantity={1} canIncrease={true} cost="10 gold" loading={$scenario == 'loading'} busy={$scenario == 'busy'} error={$scenario == 'missing' ? 'Not enough ingredients.' : ''} />
    {:else if $page == 'trade'}
      <RpgTrade title="Mira’s supplies" mode={$scenario == 'sell' ? 'sell' : 'buy'} wallet="120 gold" goods={$shownGoods} selected={$scenario == 'missing' || $scenario == 'sell' ? 'gem' : 'potion'} loading={$scenario == 'loading'} busy={$scenario == 'busy'} error={$scenario == 'missing' ? 'The price changed. Review it before trying again.' : ''} />
    {:else}
      <RpgLoot title="Abbey chest" items={$shownLoot} canTakeAll={$scenario == 'default' || $scenario == 'overflow'} loading={$scenario == 'loading'} busy={$scenario == 'busy'} error={$scenario == 'missing' ? 'Make room in your inventory and try again.' : ''} />
    {/if}
  </div>
</main>

<style>
.page { display: flex; flex-direction: column; inline-size: stretch; block-size: stretch; align-items: center; gap: var(--space-md); padding: var(--space-lg); background: var(--panel); color: var(--ink); }
h1 { margin: 0; font-size: var(--font-step-lg); }
.panel { inline-size: min(100%, var(--measure-lg)); flex: 1; min-block-size: 0; }
.panel[data-page=crafting] { inline-size: min(100%, calc(820 * var(--font-step-px))); }
</style>
