<meta name="component" content="CardReward">
<meta name="strings" content="./card.en.ogt">
<meta name="parts" content="CardReward.frame CardReward.band CardReward.title CardReward.choice CardReward.relic">
<meta name="prop:title" content="string" data-doc="The heading as the game words it (Choose a card, Treasure).">
<meta name="prop:choices" content="list" data-doc="The offers the game rolled, keyed rows of { id, relic, name, cost, icon, art, type, typeLabel, rarity, text, enabled }: relic true draws a relic (its icon, name and text) instead of a card; enabled false when the game will not let it be taken.">
<meta name="prop:selected" content="string" data-doc="The offer the player is looking at (focused or tapped once); empty is none.">
<meta name="prop:skipLabel" content="string" data-doc="The skip button's text (Skip, Take nothing); empty hides it (Back still reports skip; a game that offers none ignores it).">
<meta name="action:back" content="Skip">
<meta name="fixture:default" content='{"title":"Choose a card","selected":"cleave","skipLabel":"Skip","choices":[{"id":"cleave","relic":false,"name":"Cleave","cost":"1","icon":"sword","art":0,"type":"attack","typeLabel":"Attack","rarity":"common","text":"Deal 8 damage to all enemies.","enabled":true},{"id":"mend","relic":false,"name":"Mend","cost":"2","icon":"heart","art":0,"type":"skill","typeLabel":"Skill","rarity":"uncommon","text":"Heal 6. Exhaust.","enabled":true},{"id":"surge","relic":false,"name":"Arcane Surge","cost":"3","icon":"bolt","art":0,"type":"power","typeLabel":"Power","rarity":"rare","text":"Draw 2 cards each turn.","enabled":true}]}'>
<meta name="fixture:relics" content='{"title":"Treasure","selected":"","skipLabel":"Take nothing","choices":[{"id":"lantern","relic":true,"name":"Lantern of Dusk","cost":"","icon":"crown","art":0,"type":"relic","typeLabel":"","rarity":"rare","text":"At the start of each combat, gain 1 energy.","enabled":true},{"id":"vial","relic":true,"name":"Bottled Storm","cost":"","icon":"drop","art":0,"type":"relic","typeLabel":"","rarity":"epic","text":"Your first attack each turn deals 4 more damage.","enabled":false}]}'>
<meta name="fixture:empty" content='{"title":"Choose a card","selected":"","skipLabel":"Continue","choices":[]}'>

<!--
  Pick one of the game's offers: cards drawn as cards, relics as an icon
  plate with their text, and a skip button. The game rolls the offers and
  applies the pick; the screen reports pick (an offer's key: tap, Enter,
  pad A) and skip (the button, Escape, pad B). An offer the game refuses
  is disabled. The offers sit in one row while it fits and wrap to the
  phone's width; with none to offer it says so.
-->
<section id="CardReward" part="CardReward.frame" class="bare" aria-label={title} onback={skip}>
  <header part="CardReward.band"><h2 part="CardReward.title">{title}</h2></header>
  <div class="choices" role="listbox" aria-label={t("card.reward.choices")}>
    {#each choices as c (c.id)}
      {#if c.relic}
        <button part="CardReward.relic" role="option" data-rarity={c.rarity} disabled={!c.enabled} aria-selected={c.id == selected} aria-label={c.name} aria-description={c.text} onclick={pick}><div class="relic-face">
          <div class="relic-mark"><og-icon class="relic-icon" icon={c.icon}></og-icon></div>
          <span class="relic-kind">{t("card.reward.relic")}</span>
          <span class="relic-name">{c.name}</span>
          <span class="relic-text">{c.text}</span>
        </div></button>
      {:else}
        <button part="CardReward.choice" role="option" data-type={c.type} data-rarity={c.rarity} disabled={!c.enabled} aria-selected={c.id == selected} aria-label={c.name} aria-description={c.text} onclick={pick}><div class="face">
          <CardFace name={c.name} cost={c.cost} icon={c.icon} art={c.art} type={c.type} typeLabel={c.typeLabel} rarity={c.rarity} text={c.text} />
        </div></button>
      {/if}
    {:else}
      <p class="empty-text" role="status">{t("card.reward.empty")}</p>
    {/each}
  </div>
  <footer class="foot" hidden={!skipLabel}>
    <button id="Skip" onclick={skip}>{skipLabel}</button>
  </footer>
</section>

<style>
[part="CardReward.frame"][hidden], [part="CardReward.frame"] [hidden] { display: none; }
[part="CardReward.frame"] { display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-lg); align-items: center; justify-content: center; container-type: size; }
[part="CardReward.band"] { display: flex; width: stretch; align-items: center; justify-content: center; flex: none; }
[part="CardReward.title"] { margin: 0; text-align: center; }
/* The offers share one row, each at most three icon sizes wide and no
   taller than the height left under the band and over Skip. */
.choices { display: flex; flex-wrap: wrap; width: stretch; min-height: 0; gap: var(--space-md); align-items: center; justify-content: center; }
[part="CardReward.choice"], [part="CardReward.relic"] { flex: 1 1 0; min-width: calc(var(--size-xxl) * 1.1); max-width: min(calc(var(--size-xxl) * 3), calc((100cqh - var(--size-lg) * 3) * 0.7)); aspect-ratio: 0.7; }
[part="CardReward.choice"] { padding: 0; align-items: stretch; justify-content: stretch; }
[part="CardReward.choice"] .face { width: stretch; height: stretch; }
[part="CardReward.relic"] { padding: var(--space-sm); align-items: center; justify-content: center; }
.relic-face { display: flex; flex-direction: column; width: stretch; height: stretch; align-items: center; justify-content: flex-start; gap: var(--space-xs); overflow: hidden; text-align: center; }
.relic-mark { display: flex; width: 46%; aspect-ratio: 1; flex: none; align-items: center; justify-content: center; }
.relic-icon { width: 60%; height: 60%; }
.relic-name, .relic-kind, .relic-text { width: stretch; text-align: center; }
.relic-kind { flex: none; }
.relic-name { flex: none; }
.relic-text { flex: 1; min-height: 0; overflow: hidden; }
.empty-text { margin: 0; text-align: center; }
.foot { display: flex; flex: none; justify-content: center; }
#Skip { min-width: calc(var(--size-xxl) * 2); min-height: var(--size-touch); }
/* A phone: thirds of the row, so an offer that wraps keeps the others' size. */
@container (width < 640px) {
  .choices { gap: var(--space-sm); }
  [part="CardReward.choice"], [part="CardReward.relic"] { flex: 0 0 calc((100% - var(--space-sm) * 2) / 3); min-width: 0; }
}
</style>
